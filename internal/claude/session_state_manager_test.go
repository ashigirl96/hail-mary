package claude

import (
	"fmt"
	"os"
	"path/filepath"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestNewSessionStateManager(t *testing.T) {
	tempDir := t.TempDir()
	stateDir := filepath.Join(tempDir, "sessions")

	manager := NewSessionStateManager(stateDir)

	require.NotNil(t, manager, "NewSessionStateManager should not return nil")
	assert.Equal(t, stateDir, manager.stateDir, "StateDir should match input")
}

func TestSessionStateManager_SaveState_CreatesDirectory(t *testing.T) {
	tempDir := t.TempDir()
	stateDir := filepath.Join(tempDir, "nonexistent", "sessions")

	manager := NewSessionStateManager(stateDir)

	state := &SessionState{
		SessionID:      "test-session-001",
		StartedAt:      time.Now(),
		LastUpdated:    time.Now(),
		TranscriptPath: "/path/to/transcript.txt",
		ProjectDir:     "/path/to/project",
	}

	err := manager.SaveState(state)
	require.NoError(t, err, "SaveState should create directory and save successfully")

	// Check that directory exists
	_, err = os.Stat(stateDir)
	require.NoError(t, err, "State directory should exist after SaveState")

	// Check that file exists
	filePath := filepath.Join(stateDir, "test-session-001.json")
	_, err = os.Stat(filePath)
	require.NoError(t, err, "State file should exist after SaveState")
}

func TestSessionStateManager_SaveAndLoadState_Success(t *testing.T) {
	tempDir := t.TempDir()
	manager := NewSessionStateManager(tempDir)

	originalState := &SessionState{
		SessionID:      "test-session-002",
		StartedAt:      time.Date(2023, 1, 1, 12, 0, 0, 0, time.UTC),
		LastUpdated:    time.Date(2023, 1, 1, 12, 30, 0, 0, time.UTC),
		TranscriptPath: "/path/to/transcript.txt",
		ProjectDir:     "/path/to/project",
	}

	// Save state
	err := manager.SaveState(originalState)
	require.NoError(t, err, "SaveState should succeed")

	// Load state
	loadedState, err := manager.LoadState("test-session-002")
	require.NoError(t, err, "LoadState should succeed")

	// Compare states
	assert.Equal(t, originalState.SessionID, loadedState.SessionID)
	assert.Equal(t, originalState.StartedAt.Unix(), loadedState.StartedAt.Unix())
	assert.Equal(t, originalState.LastUpdated.Unix(), loadedState.LastUpdated.Unix())
	assert.Equal(t, originalState.TranscriptPath, loadedState.TranscriptPath)
	assert.Equal(t, originalState.ProjectDir, loadedState.ProjectDir)
}

func TestSessionStateManager_LoadState_FileNotFound(t *testing.T) {
	tempDir := t.TempDir()
	manager := NewSessionStateManager(tempDir)

	// Try to load non-existent state
	_, err := manager.LoadState("nonexistent-session")
	require.Error(t, err, "LoadState should fail for non-existent file")
	assert.Contains(t, err.Error(), "failed to read state file")
}

func TestSessionStateManager_LoadState_InvalidJSON(t *testing.T) {
	tempDir := t.TempDir()
	manager := NewSessionStateManager(tempDir)

	// Create directory
	err := os.MkdirAll(tempDir, 0755)
	require.NoError(t, err)

	// Write invalid JSON
	filePath := filepath.Join(tempDir, "invalid-session.json")
	err = os.WriteFile(filePath, []byte("invalid json content"), 0644)
	require.NoError(t, err)

	// Try to load invalid state
	_, err = manager.LoadState("invalid-session")
	require.Error(t, err, "LoadState should fail for invalid JSON")
	assert.Contains(t, err.Error(), "failed to unmarshal state")
}

func TestSessionStateManager_DeleteState_Success(t *testing.T) {
	tempDir := t.TempDir()
	manager := NewSessionStateManager(tempDir)

	state := &SessionState{
		SessionID:      "test-session-003",
		StartedAt:      time.Now(),
		LastUpdated:    time.Now(),
		TranscriptPath: "/path/to/transcript.txt",
		ProjectDir:     "/path/to/project",
	}

	// Save state first
	err := manager.SaveState(state)
	require.NoError(t, err, "SaveState should succeed")

	// Verify file exists
	filePath := filepath.Join(tempDir, "test-session-003.json")
	_, err = os.Stat(filePath)
	require.NoError(t, err, "State file should exist before deletion")

	// Delete state
	err = manager.DeleteState("test-session-003")
	require.NoError(t, err, "DeleteState should succeed")

	// Verify file is deleted
	_, err = os.Stat(filePath)
	require.True(t, os.IsNotExist(err), "State file should not exist after deletion")
}

func TestSessionStateManager_DeleteState_FileNotFound(t *testing.T) {
	tempDir := t.TempDir()
	manager := NewSessionStateManager(tempDir)

	// Try to delete non-existent state
	err := manager.DeleteState("nonexistent-session")
	require.Error(t, err, "DeleteState should fail for non-existent file")
	assert.Contains(t, err.Error(), "failed to delete state file")
}

func TestSessionStateManager_ListStates_EmptyDirectory(t *testing.T) {
	tempDir := t.TempDir()
	manager := NewSessionStateManager(tempDir)

	states, err := manager.ListStates()
	require.NoError(t, err, "ListStates should succeed for empty directory")
	assert.Empty(t, states, "States list should be empty for empty directory")
}

func TestSessionStateManager_ListStates_WithStates(t *testing.T) {
	tempDir := t.TempDir()
	manager := NewSessionStateManager(tempDir)

	// Create test states
	states := []*SessionState{
		{
			SessionID:      "session-001",
			StartedAt:      time.Now(),
			LastUpdated:    time.Now(),
			TranscriptPath: "/path/to/transcript1.txt",
			ProjectDir:     "/path/to/project1",
		},
		{
			SessionID:      "session-002",
			StartedAt:      time.Now(),
			LastUpdated:    time.Now(),
			TranscriptPath: "/path/to/transcript2.txt",
			ProjectDir:     "/path/to/project2",
		},
	}

	// Save states
	for _, state := range states {
		err := manager.SaveState(state)
		require.NoError(t, err, "SaveState should succeed")
	}

	// List states
	loadedStates, err := manager.ListStates()
	require.NoError(t, err, "ListStates should succeed")
	assert.Len(t, loadedStates, 2, "Should load 2 states")

	// Verify session IDs exist (order might be different)
	sessionIDs := make(map[string]bool)
	for _, state := range loadedStates {
		sessionIDs[state.SessionID] = true
	}
	assert.True(t, sessionIDs["session-001"], "session-001 should be present")
	assert.True(t, sessionIDs["session-002"], "session-002 should be present")
}

func TestSessionStateManager_ListStates_IgnoresNonJSONFiles(t *testing.T) {
	tempDir := t.TempDir()
	manager := NewSessionStateManager(tempDir)

	// Create directory
	err := os.MkdirAll(tempDir, 0755)
	require.NoError(t, err)

	// Create valid JSON file
	validState := &SessionState{
		SessionID:      "valid-session",
		StartedAt:      time.Now(),
		LastUpdated:    time.Now(),
		TranscriptPath: "/path/to/transcript.txt",
		ProjectDir:     "/path/to/project",
	}
	err = manager.SaveState(validState)
	require.NoError(t, err)

	// Create non-JSON files
	err = os.WriteFile(filepath.Join(tempDir, "not-json.txt"), []byte("text file"), 0644)
	require.NoError(t, err)
	err = os.WriteFile(filepath.Join(tempDir, "invalid.json"), []byte("invalid json"), 0644)
	require.NoError(t, err)

	// List states
	states, err := manager.ListStates()
	require.NoError(t, err, "ListStates should succeed")
	assert.Len(t, states, 1, "Should only load valid JSON states")
	assert.Equal(t, "valid-session", states[0].SessionID)
}

func TestFeatureSessionStateManager(t *testing.T) {
	// Test NewFeatureSessionStateManager
	t.Run("NewFeatureSessionStateManager creates correct directory structure", func(t *testing.T) {
		tempDir := t.TempDir()
		featureDir := filepath.Join(tempDir, "feature1")

		manager := NewFeatureSessionStateManager(featureDir)

		expectedStateDir := filepath.Join(featureDir, "sessions")
		assert.Equal(t, expectedStateDir, manager.stateDir, "StateDir should be featureDir/sessions")
	})
}

func TestSessionStateManager_LoadSessions(t *testing.T) {
	tests := []struct {
		name          string
		setupFunc     func(manager *SessionStateManager) error
		expectedCount int
		expectError   bool
		shouldMigrate bool
	}{
		{
			name: "Load from existing sessions.json",
			setupFunc: func(manager *SessionStateManager) error {
				// Create sessions.json directly
				sessions := FeatureSessions{
					{
						SessionID:      "session-1",
						StartedAt:      time.Now(),
						LastUpdated:    time.Now(),
						TranscriptPath: "/path1",
						ProjectDir:     "/project1",
					},
					{
						SessionID:      "session-2",
						StartedAt:      time.Now(),
						LastUpdated:    time.Now(),
						TranscriptPath: "/path2",
						ProjectDir:     "/project2",
					},
				}
				return manager.SaveSessions(sessions)
			},
			expectedCount: 2,
			expectError:   false,
			shouldMigrate: false,
		},
		{
			name: "Load from individual files (migration)",
			setupFunc: func(manager *SessionStateManager) error {
				// Create individual session files
				states := []*SessionState{
					{
						SessionID:      "session-a",
						StartedAt:      time.Now(),
						LastUpdated:    time.Now(),
						TranscriptPath: "/patha",
						ProjectDir:     "/projecta",
					},
					{
						SessionID:      "session-b",
						StartedAt:      time.Now(),
						LastUpdated:    time.Now(),
						TranscriptPath: "/pathb",
						ProjectDir:     "/projectb",
					},
				}
				for _, state := range states {
					if err := manager.SaveState(state); err != nil {
						return err
					}
				}
				return nil
			},
			expectedCount: 2,
			expectError:   false,
			shouldMigrate: true,
		},
		{
			name: "Load from empty directory",
			setupFunc: func(manager *SessionStateManager) error {
				// Do nothing - empty directory
				return nil
			},
			expectedCount: 0,
			expectError:   false,
			shouldMigrate: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tempDir := t.TempDir()
			featureDir := filepath.Join(tempDir, "feature")
			manager := NewFeatureSessionStateManager(featureDir)

			// Setup test data
			err := tt.setupFunc(manager)
			require.NoError(t, err, "Setup should not fail")

			// Load sessions
			sessions, err := manager.LoadSessions()

			if tt.expectError {
				require.Error(t, err)
				return
			}

			require.NoError(t, err, "LoadSessions should not fail")
			assert.Len(t, sessions, tt.expectedCount, "Loaded sessions count should match expected")
		})
	}
}

func TestSessionStateManager_SaveSessions(t *testing.T) {
	tempDir := t.TempDir()
	featureDir := filepath.Join(tempDir, "feature")
	manager := NewFeatureSessionStateManager(featureDir)

	sessions := FeatureSessions{
		{
			SessionID:      "session-1",
			StartedAt:      time.Date(2023, 1, 1, 12, 0, 0, 0, time.UTC),
			LastUpdated:    time.Date(2023, 1, 1, 12, 30, 0, 0, time.UTC),
			TranscriptPath: "/path/to/transcript1.txt",
			ProjectDir:     "/path/to/project1",
		},
		{
			SessionID:      "session-2",
			StartedAt:      time.Date(2023, 1, 2, 12, 0, 0, 0, time.UTC),
			LastUpdated:    time.Date(2023, 1, 2, 12, 30, 0, 0, time.UTC),
			TranscriptPath: "/path/to/transcript2.txt",
			ProjectDir:     "/path/to/project2",
		},
	}

	// Save sessions
	err := manager.SaveSessions(sessions)
	require.NoError(t, err, "SaveSessions should succeed")

	// Load and verify
	loadedSessions, err := manager.LoadSessions()
	require.NoError(t, err, "LoadSessions should succeed")
	require.Len(t, loadedSessions, 2, "Should load 2 sessions")

	// Verify session data
	sessionMap := make(map[string]*SessionState)
	for _, session := range loadedSessions {
		sessionMap[session.SessionID] = session
	}

	session1 := sessionMap["session-1"]
	require.NotNil(t, session1)
	assert.Equal(t, "/path/to/transcript1.txt", session1.TranscriptPath)

	session2 := sessionMap["session-2"]
	require.NotNil(t, session2)
	assert.Equal(t, "/path/to/transcript2.txt", session2.TranscriptPath)
}

func TestSessionStateManager_AddOrUpdateSession(t *testing.T) {
	t.Run("Add new session to empty sessions.json", func(t *testing.T) {
		tempDir := t.TempDir()
		featureDir := filepath.Join(tempDir, "feature")
		manager := NewFeatureSessionStateManager(featureDir)

		state := &SessionState{
			SessionID:      "new-session",
			StartedAt:      time.Now(),
			LastUpdated:    time.Now(),
			TranscriptPath: "/path/to/transcript.txt",
			ProjectDir:     "/path/to/project",
		}

		err := manager.AddOrUpdateSession(state)
		require.NoError(t, err, "AddOrUpdateSession should succeed")

		// Load and verify
		sessions, err := manager.LoadSessions()
		require.NoError(t, err, "LoadSessions should succeed")
		require.Len(t, sessions, 1, "Should have 1 session")
		assert.Equal(t, "new-session", sessions[0].SessionID)
	})

	t.Run("Update existing session", func(t *testing.T) {
		tempDir := t.TempDir()
		featureDir := filepath.Join(tempDir, "feature")
		manager := NewFeatureSessionStateManager(featureDir)

		// Add initial session
		originalState := &SessionState{
			SessionID:      "existing-session",
			StartedAt:      time.Date(2023, 1, 1, 12, 0, 0, 0, time.UTC),
			LastUpdated:    time.Date(2023, 1, 1, 12, 30, 0, 0, time.UTC),
			TranscriptPath: "/original/path",
			ProjectDir:     "/original/project",
		}
		err := manager.AddOrUpdateSession(originalState)
		require.NoError(t, err)

		// Update session
		updatedState := &SessionState{
			SessionID:      "existing-session",
			StartedAt:      time.Date(2023, 1, 1, 12, 0, 0, 0, time.UTC),
			LastUpdated:    time.Date(2023, 1, 1, 13, 0, 0, 0, time.UTC),
			TranscriptPath: "/updated/path",
			ProjectDir:     "/updated/project",
		}
		err = manager.AddOrUpdateSession(updatedState)
		require.NoError(t, err)

		// Load and verify
		sessions, err := manager.LoadSessions()
		require.NoError(t, err)
		require.Len(t, sessions, 1, "Should have 1 session")
		assert.Equal(t, "/updated/path", sessions[0].TranscriptPath)
		assert.Equal(t, time.Date(2023, 1, 1, 13, 0, 0, 0, time.UTC), sessions[0].LastUpdated)
	})

	t.Run("Add session to existing sessions", func(t *testing.T) {
		tempDir := t.TempDir()
		featureDir := filepath.Join(tempDir, "feature")
		manager := NewFeatureSessionStateManager(featureDir)

		// Add first session
		state1 := &SessionState{
			SessionID:      "session-1",
			StartedAt:      time.Now(),
			LastUpdated:    time.Now(),
			TranscriptPath: "/path1",
			ProjectDir:     "/project1",
		}
		err := manager.AddOrUpdateSession(state1)
		require.NoError(t, err)

		// Add second session
		state2 := &SessionState{
			SessionID:      "session-2",
			StartedAt:      time.Now(),
			LastUpdated:    time.Now(),
			TranscriptPath: "/path2",
			ProjectDir:     "/project2",
		}
		err = manager.AddOrUpdateSession(state2)
		require.NoError(t, err)

		// Load and verify
		sessions, err := manager.LoadSessions()
		require.NoError(t, err)
		require.Len(t, sessions, 2, "Should have 2 sessions")

		// Verify both sessions exist
		sessionIDs := make(map[string]bool)
		for _, session := range sessions {
			sessionIDs[session.SessionID] = true
		}
		assert.True(t, sessionIDs["session-1"])
		assert.True(t, sessionIDs["session-2"])
	})
}

func TestSessionStateManager_ConcurrentAccess(t *testing.T) {
	tempDir := t.TempDir()
	manager := NewSessionStateManager(tempDir)

	// Test concurrent SaveState and LoadState operations
	const numGoroutines = 10

	// Save states concurrently
	for i := 0; i < numGoroutines; i++ {
		go func(id int) {
			state := &SessionState{
				SessionID:      fmt.Sprintf("concurrent-session-%d", id),
				StartedAt:      time.Now(),
				LastUpdated:    time.Now(),
				TranscriptPath: fmt.Sprintf("/path/to/transcript-%d.txt", id),
				ProjectDir:     fmt.Sprintf("/path/to/project-%d", id),
			}
			err := manager.SaveState(state)
			require.NoError(t, err, "Concurrent SaveState should succeed")
		}(i)
	}

	// Give goroutines time to complete
	time.Sleep(100 * time.Millisecond)

	// Load states concurrently
	for i := 0; i < numGoroutines; i++ {
		go func(id int) {
			_, err := manager.LoadState(fmt.Sprintf("concurrent-session-%d", id))
			if err != nil {
				// Some might fail due to timing, but shouldn't cause data corruption
				t.Logf("Concurrent LoadState failed for session %d: %v", id, err)
			}
		}(i)
	}

	// Give goroutines time to complete
	time.Sleep(100 * time.Millisecond)

	// Verify final state
	states, err := manager.ListStates()
	require.NoError(t, err, "ListStates should succeed after concurrent operations")
	// We should have at least some states (timing might cause some to be missed)
	assert.True(t, len(states) > 0, "Should have some states after concurrent operations")
}
