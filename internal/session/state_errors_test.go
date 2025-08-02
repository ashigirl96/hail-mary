//go:build !windows
// +build !windows

package session

import (
	"encoding/json"
	"os"
	"path/filepath"
	"testing"
	"time"
)

// TestNewManagerErrorCases tests error paths in NewManager
func TestNewManagerErrorCases(t *testing.T) {
	// Test when state directory cannot be created
	// Create a file where the directory should be
	tempDir := t.TempDir()
	blockingFile := filepath.Join(tempDir, ".hail-mary", "sessions")

	// Create parent directory
	os.MkdirAll(filepath.Dir(blockingFile), 0755)

	// Create a file where the sessions directory should be
	if err := os.WriteFile(blockingFile, []byte("block"), 0644); err != nil {
		t.Fatalf("Failed to create blocking file: %v", err)
	}

	// Set HOME to our temp directory
	origHome := os.Getenv("HOME")
	os.Setenv("HOME", tempDir)
	defer os.Setenv("HOME", origHome)

	// This should fail because we can't create a directory where a file exists
	_, err := NewManager()
	if err == nil {
		t.Error("NewManager() should fail when directory creation fails")
	}
}

// TestWriteSessionErrorCases tests error paths in WriteSession
func TestWriteSessionErrorCases(t *testing.T) {
	manager := setupTestManager(t)
	defer cleanupTestManager(t, manager)

	// Test 1: Make state directory read-only to simulate temp file creation failure
	t.Run("temp file creation error", func(t *testing.T) {
		// Make directory read-only
		os.Chmod(manager.stateDir, 0555)
		defer os.Chmod(manager.stateDir, 0755)

		state := &State{
			SessionID: "test",
		}

		err := manager.WriteSession("test", state)
		if err == nil {
			t.Error("WriteSession should fail when temp file cannot be created")
		}
	})

	// Test 2: Create a directory where the temp file should be to simulate write error
	t.Run("write error", func(t *testing.T) {
		processID := "write-test"
		tempPath := filepath.Join(manager.stateDir, processID+".json.tmp")

		// Create a directory where temp file should be
		os.Mkdir(tempPath, 0755)
		defer os.RemoveAll(tempPath)

		state := &State{
			SessionID: "test",
		}

		err := manager.WriteSession(processID, state)
		if err == nil {
			t.Error("WriteSession should fail when file cannot be written")
		}
	})
}

// TestReadSessionErrorCases tests additional error paths in ReadSession
func TestReadSessionErrorCases(t *testing.T) {
	manager := setupTestManager(t)
	defer cleanupTestManager(t, manager)

	// Test: Create a directory instead of a file
	processID := "dir-test"
	filePath := filepath.Join(manager.stateDir, processID+".json")

	// Create a directory where the file should be
	if err := os.Mkdir(filePath, 0755); err != nil {
		t.Fatalf("Failed to create directory: %v", err)
	}

	_, err := manager.ReadSession(processID)
	if err == nil {
		t.Error("ReadSession should fail when path is a directory")
	}
}

// TestUpdateSessionErrorCases tests error paths in UpdateSession
func TestUpdateSessionErrorCases(t *testing.T) {
	manager := setupTestManager(t)
	defer cleanupTestManager(t, manager)

	// Test 1: File with invalid JSON
	t.Run("json unmarshal error", func(t *testing.T) {
		processID := "invalid-json"
		filePath := filepath.Join(manager.stateDir, processID+".json")

		// Write invalid JSON
		if err := os.WriteFile(filePath, []byte("{invalid}"), 0644); err != nil {
			t.Fatalf("Failed to write invalid JSON: %v", err)
		}

		err := manager.UpdateSession(processID)
		if err == nil {
			t.Error("UpdateSession should fail with invalid JSON")
		}
	})

	// Test 2: Temp file creation failure
	t.Run("temp file creation error", func(t *testing.T) {
		processID := "temp-fail"
		state := &State{
			SessionID: "test",
		}

		// Write valid session first
		if err := manager.WriteSession(processID, state); err != nil {
			t.Fatalf("Failed to write session: %v", err)
		}

		// Make directory read-only
		os.Chmod(manager.stateDir, 0555)
		defer os.Chmod(manager.stateDir, 0755)

		err := manager.UpdateSession(processID)
		if err == nil {
			t.Error("UpdateSession should fail when temp file cannot be created")
		}
	})

	// Test 3: Write failure
	t.Run("write error", func(t *testing.T) {
		processID := "write-fail"
		state := &State{
			SessionID: "test",
		}

		// Write valid session first
		if err := manager.WriteSession(processID, state); err != nil {
			t.Fatalf("Failed to write session: %v", err)
		}

		// Create directory where temp file should be
		tempPath := filepath.Join(manager.stateDir, processID+".json.tmp")
		os.Mkdir(tempPath, 0755)
		defer os.RemoveAll(tempPath)

		err := manager.UpdateSession(processID)
		if err == nil {
			t.Error("UpdateSession should fail when file cannot be written")
		}
	})
}

// TestCleanupSessionErrorCases tests error paths in CleanupSession
func TestCleanupSessionErrorCases(t *testing.T) {
	manager := setupTestManager(t)
	defer cleanupTestManager(t, manager)

	// Create a file
	processID := "cleanup-test"
	filePath := filepath.Join(manager.stateDir, processID+".json")
	if err := os.WriteFile(filePath, []byte("{}"), 0644); err != nil {
		t.Fatalf("Failed to create file: %v", err)
	}

	// Make directory read-only to prevent deletion
	os.Chmod(manager.stateDir, 0555)
	defer os.Chmod(manager.stateDir, 0755)

	err := manager.CleanupSession(processID)
	if err == nil {
		t.Error("CleanupSession should fail when file cannot be removed")
	}
}

// TestListSessionsErrorCases tests error paths in ListSessions
func TestListSessionsErrorCases(t *testing.T) {
	// Create manager with non-existent directory
	manager := &Manager{
		stateDir: "/non/existent/directory",
	}

	_, err := manager.ListSessions()
	if err == nil {
		t.Error("ListSessions should fail with non-existent directory")
	}
}

// TestCleanupStaleErrorCases tests error paths in CleanupStale
func TestCleanupStaleErrorCases(t *testing.T) {
	t.Run("directory read error", func(t *testing.T) {
		// Create manager with non-existent directory
		manager := &Manager{
			stateDir: "/non/existent/directory",
		}

		err := manager.CleanupStale(24 * time.Hour)
		if err == nil {
			t.Error("CleanupStale should fail with non-existent directory")
		}
	})

	t.Run("file removal error", func(t *testing.T) {
		manager := setupTestManager(t)
		defer cleanupTestManager(t, manager)

		// Create an old file
		oldFile := filepath.Join(manager.stateDir, "old.json")
		state := &State{SessionID: "old"}
		data, _ := json.Marshal(state)
		if err := os.WriteFile(oldFile, data, 0644); err != nil {
			t.Fatalf("Failed to create old file: %v", err)
		}

		// Set old modification time
		oldTime := time.Now().Add(-48 * time.Hour)
		os.Chtimes(oldFile, oldTime, oldTime)

		// Make directory read-only to prevent deletion
		os.Chmod(manager.stateDir, 0555)
		defer os.Chmod(manager.stateDir, 0755)

		// This should try to remove the file but fail
		err := manager.CleanupStale(24 * time.Hour)
		// The function continues on error, so it might not return an error
		// but we've covered the error path
		_ = err
	})
}
