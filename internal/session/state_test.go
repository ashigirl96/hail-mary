package session

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"
)

// TestNewManager tests creating a new session manager
func TestNewManager(t *testing.T) {
	// Save original home dir
	origHome := os.Getenv("HOME")
	if origHome == "" {
		origHome = os.Getenv("USERPROFILE") // Windows
	}

	// Test with valid home directory
	manager, err := NewManager()
	if err != nil {
		t.Fatalf("NewManager() error = %v", err)
	}

	if manager == nil {
		t.Fatal("NewManager() returned nil")
	}

	expectedDir := filepath.Join(origHome, ".hail-mary", "sessions")
	if manager.stateDir != expectedDir {
		t.Errorf("stateDir = %q, want %q", manager.stateDir, expectedDir)
	}

	// Verify directory was created
	if _, err := os.Stat(manager.stateDir); os.IsNotExist(err) {
		t.Error("state directory was not created")
	}

	// Clean up
	os.RemoveAll(filepath.Join(origHome, ".hail-mary"))

	// Test when HOME is not set (simulate error)
	os.Unsetenv("HOME")
	os.Unsetenv("USERPROFILE")
	_, err = NewManager()
	if err == nil {
		t.Error("NewManager() error = nil, want error when HOME is not set")
	}

	// Restore HOME
	os.Setenv("HOME", origHome)
}

// TestWriteSession tests writing session state
func TestWriteSession(t *testing.T) {
	manager := setupTestManager(t)
	defer cleanupTestManager(t, manager)

	processID := "test-process-123"
	state := &State{
		SessionID:      "session-456",
		StartedAt:      time.Now(),
		LastUpdated:    time.Now(),
		TranscriptPath: "/path/to/transcript",
		ProjectDir:     "/path/to/project",
	}

	// Test successful write
	err := manager.WriteSession(processID, state)
	if err != nil {
		t.Fatalf("WriteSession() error = %v", err)
	}

	// Verify file exists
	expectedPath := filepath.Join(manager.stateDir, processID+".json")
	if _, err := os.Stat(expectedPath); os.IsNotExist(err) {
		t.Error("session file was not created")
	}

	// Verify file contents
	data, err := os.ReadFile(expectedPath)
	if err != nil {
		t.Fatalf("Failed to read session file: %v", err)
	}

	var savedState State
	if err := json.Unmarshal(data, &savedState); err != nil {
		t.Fatalf("Failed to unmarshal saved state: %v", err)
	}

	if savedState.SessionID != state.SessionID {
		t.Errorf("SessionID = %q, want %q", savedState.SessionID, state.SessionID)
	}
	if savedState.ProjectDir != state.ProjectDir {
		t.Errorf("ProjectDir = %q, want %q", savedState.ProjectDir, state.ProjectDir)
	}

	// Test concurrent writes
	done := make(chan bool, 2)
	errorChan := make(chan error, 2)

	go func() {
		err := manager.WriteSession("concurrent-1", state)
		errorChan <- err
		done <- true
	}()

	go func() {
		err := manager.WriteSession("concurrent-2", state)
		errorChan <- err
		done <- true
	}()

	<-done
	<-done

	close(errorChan)
	for err := range errorChan {
		if err != nil {
			t.Errorf("Concurrent WriteSession() error = %v", err)
		}
	}
}

// TestReadSession tests reading session state
func TestReadSession(t *testing.T) {
	manager := setupTestManager(t)
	defer cleanupTestManager(t, manager)

	processID := "test-read-123"
	state := &State{
		SessionID:      "session-789",
		StartedAt:      time.Now().Round(time.Second), // Round to avoid nanosecond precision issues
		LastUpdated:    time.Now().Round(time.Second),
		TranscriptPath: "/path/to/transcript",
		ProjectDir:     "/path/to/project",
	}

	// Write a session first
	if err := manager.WriteSession(processID, state); err != nil {
		t.Fatalf("WriteSession() error = %v", err)
	}

	// Test successful read
	readState, err := manager.ReadSession(processID)
	if err != nil {
		t.Fatalf("ReadSession() error = %v", err)
	}

	if readState.SessionID != state.SessionID {
		t.Errorf("SessionID = %q, want %q", readState.SessionID, state.SessionID)
	}
	if !readState.StartedAt.Equal(state.StartedAt) {
		t.Errorf("StartedAt = %v, want %v", readState.StartedAt, state.StartedAt)
	}

	// Test reading non-existent session
	_, err = manager.ReadSession("non-existent")
	if err == nil {
		t.Error("ReadSession(non-existent) error = nil, want error")
	}
	if !strings.Contains(err.Error(), "session not found") {
		t.Errorf("ReadSession(non-existent) error = %v, want 'session not found'", err)
	}

	// Test reading corrupted file
	corruptPath := filepath.Join(manager.stateDir, "corrupt.json")
	if err := os.WriteFile(corruptPath, []byte("invalid json"), 0644); err != nil {
		t.Fatalf("Failed to create corrupt file: %v", err)
	}

	_, err = manager.ReadSession("corrupt")
	if err == nil {
		t.Error("ReadSession(corrupt) error = nil, want error")
	}
	if !strings.Contains(err.Error(), "failed to unmarshal") {
		t.Errorf("ReadSession(corrupt) error = %v, want unmarshal error", err)
	}
}

// TestUpdateSession tests updating session timestamp
func TestUpdateSession(t *testing.T) {
	manager := setupTestManager(t)
	defer cleanupTestManager(t, manager)

	processID := "test-update-123"
	originalTime := time.Now().Add(-1 * time.Hour).Round(time.Second)
	state := &State{
		SessionID:      "session-update",
		StartedAt:      originalTime,
		LastUpdated:    originalTime,
		TranscriptPath: "/path/to/transcript",
		ProjectDir:     "/path/to/project",
	}

	// Write initial session
	if err := manager.WriteSession(processID, state); err != nil {
		t.Fatalf("WriteSession() error = %v", err)
	}

	// Sleep briefly to ensure time difference
	time.Sleep(10 * time.Millisecond)

	// Update session
	beforeUpdate := time.Now()
	err := manager.UpdateSession(processID)
	if err != nil {
		t.Fatalf("UpdateSession() error = %v", err)
	}

	// Read updated session
	updatedState, err := manager.ReadSession(processID)
	if err != nil {
		t.Fatalf("ReadSession() after update error = %v", err)
	}

	// Verify LastUpdated was changed
	if updatedState.LastUpdated.Before(beforeUpdate) {
		t.Error("LastUpdated was not updated")
	}

	// Verify other fields were not changed
	if !updatedState.StartedAt.Equal(originalTime) {
		t.Error("StartedAt was changed unexpectedly")
	}
	if updatedState.SessionID != state.SessionID {
		t.Errorf("SessionID = %q, want %q", updatedState.SessionID, state.SessionID)
	}

	// Test updating non-existent session
	err = manager.UpdateSession("non-existent")
	if err == nil {
		t.Error("UpdateSession(non-existent) error = nil, want error")
	}
	if !strings.Contains(err.Error(), "failed to read session file") {
		t.Errorf("UpdateSession(non-existent) error = %v, want read error", err)
	}
}

// TestCleanupSession tests removing session state
func TestCleanupSession(t *testing.T) {
	manager := setupTestManager(t)
	defer cleanupTestManager(t, manager)

	processID := "test-cleanup-123"
	state := &State{
		SessionID:      "session-cleanup",
		StartedAt:      time.Now(),
		LastUpdated:    time.Now(),
		TranscriptPath: "/path/to/transcript",
		ProjectDir:     "/path/to/project",
	}

	// Write a session
	if err := manager.WriteSession(processID, state); err != nil {
		t.Fatalf("WriteSession() error = %v", err)
	}

	// Verify file exists
	statePath := filepath.Join(manager.stateDir, processID+".json")
	if _, err := os.Stat(statePath); os.IsNotExist(err) {
		t.Fatal("session file was not created")
	}

	// Cleanup session
	err := manager.CleanupSession(processID)
	if err != nil {
		t.Fatalf("CleanupSession() error = %v", err)
	}

	// Verify file was removed
	if _, err := os.Stat(statePath); !os.IsNotExist(err) {
		t.Error("session file was not removed")
	}

	// Test cleaning up non-existent session (should not error)
	err = manager.CleanupSession("non-existent")
	if err != nil {
		t.Errorf("CleanupSession(non-existent) error = %v, want nil", err)
	}
}

// TestListSessions tests listing all active sessions
func TestListSessions(t *testing.T) {
	manager := setupTestManager(t)
	defer cleanupTestManager(t, manager)

	// Create multiple sessions
	sessions := []struct {
		processID string
		state     *State
	}{
		{
			processID: "process-1",
			state: &State{
				SessionID:      "session-1",
				StartedAt:      time.Now(),
				LastUpdated:    time.Now(),
				TranscriptPath: "/path/1",
				ProjectDir:     "/project/1",
			},
		},
		{
			processID: "process-2",
			state: &State{
				SessionID:      "session-2",
				StartedAt:      time.Now(),
				LastUpdated:    time.Now(),
				TranscriptPath: "/path/2",
				ProjectDir:     "/project/2",
			},
		},
		{
			processID: "process-3",
			state: &State{
				SessionID:      "session-3",
				StartedAt:      time.Now(),
				LastUpdated:    time.Now(),
				TranscriptPath: "/path/3",
				ProjectDir:     "/project/3",
			},
		},
	}

	for _, s := range sessions {
		if err := manager.WriteSession(s.processID, s.state); err != nil {
			t.Fatalf("WriteSession(%s) error = %v", s.processID, err)
		}
	}

	// Create a non-JSON file (should be ignored)
	nonJSONPath := filepath.Join(manager.stateDir, "not-json.txt")
	if err := os.WriteFile(nonJSONPath, []byte("not json"), 0644); err != nil {
		t.Fatalf("Failed to create non-JSON file: %v", err)
	}

	// Create a directory (should be ignored)
	dirPath := filepath.Join(manager.stateDir, "subdir")
	if err := os.Mkdir(dirPath, 0755); err != nil {
		t.Fatalf("Failed to create subdirectory: %v", err)
	}

	// Create an invalid JSON file (should be skipped)
	invalidJSONPath := filepath.Join(manager.stateDir, "invalid.json")
	if err := os.WriteFile(invalidJSONPath, []byte("{invalid json}"), 0644); err != nil {
		t.Fatalf("Failed to create invalid JSON file: %v", err)
	}

	// List sessions
	listedSessions, err := manager.ListSessions()
	if err != nil {
		t.Fatalf("ListSessions() error = %v", err)
	}

	// Should have exactly 3 valid sessions
	if len(listedSessions) != 3 {
		t.Errorf("ListSessions() returned %d sessions, want 3", len(listedSessions))
	}

	// Verify all sessions are present
	sessionMap := make(map[string]*State)
	for _, s := range listedSessions {
		sessionMap[s.SessionID] = s
	}

	for _, expected := range sessions {
		if _, found := sessionMap[expected.state.SessionID]; !found {
			t.Errorf("Session %s not found in list", expected.state.SessionID)
		}
	}

	// Test with unreadable file (simulate permission error)
	unreadablePath := filepath.Join(manager.stateDir, "unreadable.json")
	if err := os.WriteFile(unreadablePath, []byte("{}"), 0000); err != nil {
		t.Fatalf("Failed to create unreadable file: %v", err)
	}
	defer os.Chmod(unreadablePath, 0644) // Restore permissions for cleanup

	// Should still return other sessions
	listedSessions2, err := manager.ListSessions()
	if err != nil {
		t.Fatalf("ListSessions() with unreadable file error = %v", err)
	}

	// Should still have 3 valid sessions (unreadable is skipped)
	if len(listedSessions2) != 3 {
		t.Errorf("ListSessions() with unreadable file returned %d sessions, want 3", len(listedSessions2))
	}
}

// TestCleanupStale tests removing old session files
func TestCleanupStale(t *testing.T) {
	manager := setupTestManager(t)
	defer cleanupTestManager(t, manager)

	// Create sessions with different ages
	now := time.Now()
	oldTime := now.Add(-25 * time.Hour)
	recentTime := now.Add(-1 * time.Hour)

	// Create old session
	oldPath := filepath.Join(manager.stateDir, "old.json")
	oldState := &State{
		SessionID:   "old-session",
		StartedAt:   oldTime,
		LastUpdated: oldTime,
	}
	data, _ := json.MarshalIndent(oldState, "", "  ")
	if err := os.WriteFile(oldPath, data, 0644); err != nil {
		t.Fatalf("Failed to create old session: %v", err)
	}
	// Set modification time to old
	os.Chtimes(oldPath, oldTime, oldTime)

	// Create recent session
	recentPath := filepath.Join(manager.stateDir, "recent.json")
	recentState := &State{
		SessionID:   "recent-session",
		StartedAt:   recentTime,
		LastUpdated: recentTime,
	}
	data, _ = json.MarshalIndent(recentState, "", "  ")
	if err := os.WriteFile(recentPath, data, 0644); err != nil {
		t.Fatalf("Failed to create recent session: %v", err)
	}
	os.Chtimes(recentPath, recentTime, recentTime)

	// Create non-JSON file
	nonJSONPath := filepath.Join(manager.stateDir, "not-json.txt")
	if err := os.WriteFile(nonJSONPath, []byte("text"), 0644); err != nil {
		t.Fatalf("Failed to create non-JSON file: %v", err)
	}
	os.Chtimes(nonJSONPath, oldTime, oldTime)

	// Create directory
	dirPath := filepath.Join(manager.stateDir, "subdir")
	if err := os.Mkdir(dirPath, 0755); err != nil {
		t.Fatalf("Failed to create directory: %v", err)
	}

	// Cleanup stale sessions older than 24 hours
	err := manager.CleanupStale(24 * time.Hour)
	if err != nil {
		t.Fatalf("CleanupStale() error = %v", err)
	}

	// Verify old session was removed
	if _, err := os.Stat(oldPath); !os.IsNotExist(err) {
		t.Error("old session was not removed")
	}

	// Verify recent session was kept
	if _, err := os.Stat(recentPath); os.IsNotExist(err) {
		t.Error("recent session was removed unexpectedly")
	}

	// Verify non-JSON file was kept (not removed)
	if _, err := os.Stat(nonJSONPath); os.IsNotExist(err) {
		t.Error("non-JSON file was removed")
	}

	// Verify directory was kept
	if _, err := os.Stat(dirPath); os.IsNotExist(err) {
		t.Error("directory was removed")
	}
}

// TestConcurrentAccess tests concurrent read/write operations
func TestConcurrentAccess(t *testing.T) {
	manager := setupTestManager(t)
	defer cleanupTestManager(t, manager)

	// Test concurrent writes to different sessions
	numGoroutines := 10
	done := make(chan bool, numGoroutines)
	errors := make(chan error, numGoroutines)

	for i := 0; i < numGoroutines; i++ {
		go func(id int) {
			processID := string(rune('A' + id))
			state := &State{
				SessionID:      processID + "-session",
				StartedAt:      time.Now(),
				LastUpdated:    time.Now(),
				TranscriptPath: "/path/" + processID,
				ProjectDir:     "/project/" + processID,
			}

			if err := manager.WriteSession(processID, state); err != nil {
				errors <- err
			}

			// Also test read
			if _, err := manager.ReadSession(processID); err != nil {
				errors <- err
			}

			done <- true
		}(i)
	}

	// Wait for all goroutines
	for i := 0; i < numGoroutines; i++ {
		<-done
	}

	close(errors)
	for err := range errors {
		t.Errorf("Concurrent operation error: %v", err)
	}

	// Verify all sessions were created
	sessions, err := manager.ListSessions()
	if err != nil {
		t.Fatalf("ListSessions() error = %v", err)
	}

	if len(sessions) != numGoroutines {
		t.Errorf("Expected %d sessions, got %d", numGoroutines, len(sessions))
	}
}

// TestAtomicWrite tests that writes are atomic
func TestAtomicWrite(t *testing.T) {
	manager := setupTestManager(t)
	defer cleanupTestManager(t, manager)

	processID := "atomic-test"
	statePath := filepath.Join(manager.stateDir, processID+".json")

	// Write initial state
	state1 := &State{
		SessionID:  "session-1",
		ProjectDir: "initial",
	}
	if err := manager.WriteSession(processID, state1); err != nil {
		t.Fatalf("WriteSession() error = %v", err)
	}

	// Simulate interrupted write by creating temp file
	tempPath := statePath + ".tmp"
	if err := os.WriteFile(tempPath, []byte("partial write"), 0644); err != nil {
		t.Fatalf("Failed to create temp file: %v", err)
	}

	// Write should still succeed and clean up temp file
	state2 := &State{
		SessionID:  "session-2",
		ProjectDir: "updated",
	}
	if err := manager.WriteSession(processID, state2); err != nil {
		t.Fatalf("WriteSession() after interrupted write error = %v", err)
	}

	// Verify temp file was cleaned up
	if _, err := os.Stat(tempPath); !os.IsNotExist(err) {
		t.Error("temp file was not cleaned up")
	}

	// Verify correct state was written
	readState, err := manager.ReadSession(processID)
	if err != nil {
		t.Fatalf("ReadSession() error = %v", err)
	}
	if readState.SessionID != state2.SessionID {
		t.Errorf("SessionID = %q, want %q", readState.SessionID, state2.SessionID)
	}
}

// TestStateJSON tests JSON marshaling/unmarshaling of State
func TestStateJSON(t *testing.T) {
	state := &State{
		SessionID:      "test-session",
		StartedAt:      time.Now().Round(time.Second),
		LastUpdated:    time.Now().Round(time.Second),
		TranscriptPath: "/path/to/transcript",
		ProjectDir:     "/path/to/project",
	}

	// Marshal to JSON
	data, err := json.Marshal(state)
	if err != nil {
		t.Fatalf("json.Marshal() error = %v", err)
	}

	// Unmarshal back
	var decoded State
	if err := json.Unmarshal(data, &decoded); err != nil {
		t.Fatalf("json.Unmarshal() error = %v", err)
	}

	// Compare fields
	if decoded.SessionID != state.SessionID {
		t.Errorf("SessionID = %q, want %q", decoded.SessionID, state.SessionID)
	}
	if !decoded.StartedAt.Equal(state.StartedAt) {
		t.Errorf("StartedAt = %v, want %v", decoded.StartedAt, state.StartedAt)
	}
	if !decoded.LastUpdated.Equal(state.LastUpdated) {
		t.Errorf("LastUpdated = %v, want %v", decoded.LastUpdated, state.LastUpdated)
	}
	if decoded.TranscriptPath != state.TranscriptPath {
		t.Errorf("TranscriptPath = %q, want %q", decoded.TranscriptPath, state.TranscriptPath)
	}
	if decoded.ProjectDir != state.ProjectDir {
		t.Errorf("ProjectDir = %q, want %q", decoded.ProjectDir, state.ProjectDir)
	}
}

// Helper functions
func setupTestManager(t *testing.T) *Manager {
	tempDir := t.TempDir()
	stateDir := filepath.Join(tempDir, "sessions")
	if err := os.MkdirAll(stateDir, 0755); err != nil {
		t.Fatalf("Failed to create test state directory: %v", err)
	}
	return &Manager{stateDir: stateDir}
}

func cleanupTestManager(t *testing.T, manager *Manager) {
	if err := os.RemoveAll(manager.stateDir); err != nil {
		t.Errorf("Failed to cleanup test directory: %v", err)
	}
}

// Benchmark tests
func BenchmarkWriteSession(b *testing.B) {
	manager := &Manager{stateDir: b.TempDir()}
	state := &State{
		SessionID:      "bench-session",
		StartedAt:      time.Now(),
		LastUpdated:    time.Now(),
		TranscriptPath: "/bench/transcript",
		ProjectDir:     "/bench/project",
	}

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		processID := "bench-" + string(rune(i))
		_ = manager.WriteSession(processID, state)
	}
}

func BenchmarkReadSession(b *testing.B) {
	manager := &Manager{stateDir: b.TempDir()}
	processID := "bench-read"
	state := &State{
		SessionID:      "bench-session",
		StartedAt:      time.Now(),
		LastUpdated:    time.Now(),
		TranscriptPath: "/bench/transcript",
		ProjectDir:     "/bench/project",
	}
	_ = manager.WriteSession(processID, state)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, _ = manager.ReadSession(processID)
	}
}

func BenchmarkListSessions(b *testing.B) {
	manager := &Manager{stateDir: b.TempDir()}

	// Create 100 sessions
	for i := 0; i < 100; i++ {
		processID := "bench-" + string(rune(i))
		state := &State{
			SessionID:      "session-" + string(rune(i)),
			StartedAt:      time.Now(),
			LastUpdated:    time.Now(),
			TranscriptPath: "/bench/transcript",
			ProjectDir:     "/bench/project",
		}
		_ = manager.WriteSession(processID, state)
	}

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, _ = manager.ListSessions()
	}
}
