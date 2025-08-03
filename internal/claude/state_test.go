package claude

import (
	"encoding/json"
	"os"
	"path/filepath"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestNewStateManager(t *testing.T) {
	// Arrange
	stateDir := "/tmp/test-sessions"

	// Act
	manager := NewStateManager(stateDir)

	// Assert
	require.NotNil(t, manager, "NewStateManager should not return nil")
	assert.Equal(t, stateDir, manager.stateDir, "State directory should be set correctly")
}

func TestStateManager_SaveState_CreatesDirectory(t *testing.T) {
	// Arrange
	tempDir := t.TempDir()
	stateDir := filepath.Join(tempDir, "sessions")
	manager := NewStateManager(stateDir)

	state := &State{
		SessionID:      "test-session-123",
		StartedAt:      time.Now(),
		LastUpdated:    time.Now(),
		TranscriptPath: "/path/to/transcript",
		ProjectDir:     "/path/to/project",
	}

	// Act
	err := manager.SaveState(state)

	// Assert
	require.NoError(t, err, "SaveState should not return error")

	// Verify directory was created
	_, err = os.Stat(stateDir)
	assert.NoError(t, err, "State directory should be created")
}

func TestStateManager_SaveAndLoadState_Success(t *testing.T) {
	// Arrange
	tempDir := t.TempDir()
	manager := NewStateManager(tempDir)

	expectedState := &State{
		SessionID:      "test-session-456",
		StartedAt:      time.Now().Truncate(time.Second), // Truncate for JSON precision
		LastUpdated:    time.Now().Truncate(time.Second),
		TranscriptPath: "/test/transcript.txt",
		ProjectDir:     "/test/project",
	}

	// Act - Save
	err := manager.SaveState(expectedState)
	require.NoError(t, err, "SaveState should not return error")

	// Act - Load
	actualState, err := manager.LoadState("test-session-456")

	// Assert
	require.NoError(t, err, "LoadState should not return error")
	require.NotNil(t, actualState, "LoadState should return non-nil state")

	assert.Equal(t, expectedState.SessionID, actualState.SessionID)
	assert.True(t, expectedState.StartedAt.Equal(actualState.StartedAt), "StartedAt should match")
	assert.True(t, expectedState.LastUpdated.Equal(actualState.LastUpdated), "LastUpdated should match")
	assert.Equal(t, expectedState.TranscriptPath, actualState.TranscriptPath)
	assert.Equal(t, expectedState.ProjectDir, actualState.ProjectDir)
}

func TestStateManager_LoadState_FileNotFound(t *testing.T) {
	// Arrange
	tempDir := t.TempDir()
	manager := NewStateManager(tempDir)

	// Act
	state, err := manager.LoadState("non-existent-session")

	// Assert
	require.Error(t, err, "LoadState should return error for non-existent file")
	assert.Nil(t, state, "LoadState should return nil state on error")
	assert.Contains(t, err.Error(), "failed to read state file", "Error should indicate file read failure")
}

func TestStateManager_LoadState_InvalidJSON(t *testing.T) {
	// Arrange
	tempDir := t.TempDir()
	manager := NewStateManager(tempDir)

	// Create invalid JSON file
	sessionID := "invalid-json-session"
	filePath := filepath.Join(tempDir, sessionID+".json")
	err := os.WriteFile(filePath, []byte("invalid json content"), 0644)
	require.NoError(t, err, "Setup should succeed")

	// Act
	state, err := manager.LoadState(sessionID)

	// Assert
	require.Error(t, err, "LoadState should return error for invalid JSON")
	assert.Nil(t, state, "LoadState should return nil state on error")
	assert.Contains(t, err.Error(), "failed to unmarshal state", "Error should indicate JSON parse failure")
}

func TestStateManager_DeleteState_Success(t *testing.T) {
	// Arrange
	tempDir := t.TempDir()
	manager := NewStateManager(tempDir)

	state := &State{
		SessionID:   "deletable-session",
		StartedAt:   time.Now(),
		LastUpdated: time.Now(),
	}

	// Save first
	err := manager.SaveState(state)
	require.NoError(t, err, "Setup should succeed")

	// Act
	err = manager.DeleteState("deletable-session")

	// Assert
	require.NoError(t, err, "DeleteState should not return error")

	// Verify file is deleted
	filePath := filepath.Join(tempDir, "deletable-session.json")
	_, err = os.Stat(filePath)
	assert.True(t, os.IsNotExist(err), "State file should be deleted")

	// Verify LoadState now fails
	_, err = manager.LoadState("deletable-session")
	assert.Error(t, err, "LoadState should fail after deletion")
}

func TestStateManager_DeleteState_FileNotFound(t *testing.T) {
	// Arrange
	tempDir := t.TempDir()
	manager := NewStateManager(tempDir)

	// Act
	err := manager.DeleteState("non-existent-session")

	// Assert
	require.Error(t, err, "DeleteState should return error for non-existent file")
	assert.Contains(t, err.Error(), "failed to delete state file", "Error should indicate deletion failure")
}

func TestStateManager_ListStates_EmptyDirectory(t *testing.T) {
	// Arrange
	tempDir := t.TempDir()
	manager := NewStateManager(tempDir)

	// Act
	states, err := manager.ListStates()

	// Assert
	require.NoError(t, err, "ListStates should not return error for empty directory")
	assert.Empty(t, states, "ListStates should return empty slice for empty directory")
}

func TestStateManager_ListStates_WithStates(t *testing.T) {
	// Arrange
	tempDir := t.TempDir()
	manager := NewStateManager(tempDir)

	// Create test states
	states := []*State{
		{
			SessionID:   "session-1",
			StartedAt:   time.Now().Add(-2 * time.Hour),
			LastUpdated: time.Now().Add(-1 * time.Hour),
		},
		{
			SessionID:   "session-2",
			StartedAt:   time.Now().Add(-1 * time.Hour),
			LastUpdated: time.Now(),
		},
	}

	for _, state := range states {
		err := manager.SaveState(state)
		require.NoError(t, err, "Setup should succeed")
	}

	// Act
	actualStates, err := manager.ListStates()

	// Assert
	require.NoError(t, err, "ListStates should not return error")
	require.Len(t, actualStates, 2, "ListStates should return 2 states")

	// Verify session IDs are present (order might vary)
	sessionIDs := make(map[string]bool)
	for _, state := range actualStates {
		sessionIDs[state.SessionID] = true
	}
	assert.True(t, sessionIDs["session-1"], "session-1 should be present")
	assert.True(t, sessionIDs["session-2"], "session-2 should be present")
}

func TestStateManager_ListStates_IgnoresNonJSONFiles(t *testing.T) {
	// Arrange
	tempDir := t.TempDir()
	manager := NewStateManager(tempDir)

	// Create a valid state
	state := &State{
		SessionID:   "valid-session",
		StartedAt:   time.Now(),
		LastUpdated: time.Now(),
	}
	err := manager.SaveState(state)
	require.NoError(t, err, "Setup should succeed")

	// Create non-JSON files
	err = os.WriteFile(filepath.Join(tempDir, "readme.txt"), []byte("not a json file"), 0644)
	require.NoError(t, err, "Setup should succeed")

	err = os.WriteFile(filepath.Join(tempDir, "config.yaml"), []byte("yaml: content"), 0644)
	require.NoError(t, err, "Setup should succeed")

	// Act
	states, err := manager.ListStates()

	// Assert
	require.NoError(t, err, "ListStates should not return error")
	require.Len(t, states, 1, "ListStates should return only 1 state (ignoring non-JSON files)")
	assert.Equal(t, "valid-session", states[0].SessionID)
}

func TestStateManager_UpdateState(t *testing.T) {
	// Arrange
	tempDir := t.TempDir()
	manager := NewStateManager(tempDir)

	// Create initial state
	initialState := &State{
		SessionID:      "update-session",
		StartedAt:      time.Now().Add(-1 * time.Hour),
		LastUpdated:    time.Now().Add(-1 * time.Hour),
		TranscriptPath: "/old/path",
		ProjectDir:     "/old/project",
	}
	err := manager.SaveState(initialState)
	require.NoError(t, err, "Setup should succeed")

	// Act - Update state
	updatedState := &State{
		SessionID:      "update-session",
		StartedAt:      initialState.StartedAt, // Keep original start time
		LastUpdated:    time.Now(),
		TranscriptPath: "/new/path",
		ProjectDir:     "/new/project",
	}
	err = manager.SaveState(updatedState)
	require.NoError(t, err, "Update should succeed")

	// Assert
	loadedState, err := manager.LoadState("update-session")
	require.NoError(t, err, "Load should succeed")

	assert.Equal(t, updatedState.SessionID, loadedState.SessionID)
	assert.True(t, updatedState.StartedAt.Equal(loadedState.StartedAt), "StartedAt should be preserved")
	assert.True(t, updatedState.LastUpdated.After(initialState.LastUpdated), "LastUpdated should be newer")
	assert.Equal(t, "/new/path", loadedState.TranscriptPath)
	assert.Equal(t, "/new/project", loadedState.ProjectDir)
}

func TestState_JSONMarshaling(t *testing.T) {
	// Arrange
	original := &State{
		SessionID:      "json-test-session",
		StartedAt:      time.Date(2024, 1, 15, 10, 30, 0, 0, time.UTC),
		LastUpdated:    time.Date(2024, 1, 15, 11, 30, 0, 0, time.UTC),
		TranscriptPath: "/test/transcript.log",
		ProjectDir:     "/test/project",
	}

	// Act - Marshal
	data, err := json.Marshal(original)
	require.NoError(t, err, "Marshal should succeed")

	// Act - Unmarshal
	var decoded State
	err = json.Unmarshal(data, &decoded)
	require.NoError(t, err, "Unmarshal should succeed")

	// Assert
	assert.Equal(t, original.SessionID, decoded.SessionID)
	assert.True(t, original.StartedAt.Equal(decoded.StartedAt), "StartedAt should match")
	assert.True(t, original.LastUpdated.Equal(decoded.LastUpdated), "LastUpdated should match")
	assert.Equal(t, original.TranscriptPath, decoded.TranscriptPath)
	assert.Equal(t, original.ProjectDir, decoded.ProjectDir)
}

func TestStateManager_ConcurrentAccess(t *testing.T) {
	// This is a basic test for concurrent access patterns
	// In a production system, you might want more sophisticated concurrent testing

	// Arrange
	tempDir := t.TempDir()
	manager := NewStateManager(tempDir)

	state := &State{
		SessionID:   "concurrent-session",
		StartedAt:   time.Now(),
		LastUpdated: time.Now(),
	}

	// Act & Assert - Multiple save operations should not conflict
	for i := 0; i < 10; i++ {
		state.LastUpdated = time.Now()
		err := manager.SaveState(state)
		assert.NoError(t, err, "Concurrent save should succeed")
	}

	// Verify final state can be loaded
	loadedState, err := manager.LoadState("concurrent-session")
	require.NoError(t, err, "Final load should succeed")
	assert.Equal(t, "concurrent-session", loadedState.SessionID)
}

// Tests for SessionsState type

func TestSessionsState_FindBySessionID(t *testing.T) {
	// Arrange
	sessions := SessionsState{
		{SessionID: "session-1", StartedAt: time.Now()},
		{SessionID: "session-2", StartedAt: time.Now()},
		{SessionID: "session-3", StartedAt: time.Now()},
	}

	// Test cases
	tests := []struct {
		name          string
		sessionID     string
		expectedFound bool
		expectedIndex int
	}{
		{
			name:          "find existing session",
			sessionID:     "session-2",
			expectedFound: true,
			expectedIndex: 1,
		},
		{
			name:          "find first session",
			sessionID:     "session-1",
			expectedFound: true,
			expectedIndex: 0,
		},
		{
			name:          "find last session",
			sessionID:     "session-3",
			expectedFound: true,
			expectedIndex: 2,
		},
		{
			name:          "session not found",
			sessionID:     "non-existent",
			expectedFound: false,
			expectedIndex: -1,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Act
			state, index := sessions.FindBySessionID(tt.sessionID)

			// Assert
			if tt.expectedFound {
				require.NotNil(t, state, "State should be found")
				assert.Equal(t, tt.sessionID, state.SessionID)
				assert.Equal(t, tt.expectedIndex, index)
			} else {
				assert.Nil(t, state, "State should not be found")
				assert.Equal(t, -1, index)
			}
		})
	}
}

func TestSessionsState_AddOrUpdate(t *testing.T) {
	// Test adding new sessions
	t.Run("add new sessions", func(t *testing.T) {
		// Arrange
		sessions := SessionsState{}

		session1 := &State{
			SessionID:   "new-session-1",
			StartedAt:   time.Now(),
			LastUpdated: time.Now(),
		}

		session2 := &State{
			SessionID:   "new-session-2",
			StartedAt:   time.Now(),
			LastUpdated: time.Now(),
		}

		// Act
		sessions.AddOrUpdate(session1)
		sessions.AddOrUpdate(session2)

		// Assert
		require.Len(t, sessions, 2, "Should have 2 sessions")
		// New sessions are added at the beginning
		assert.Equal(t, "new-session-2", sessions[0].SessionID)
		assert.Equal(t, "new-session-1", sessions[1].SessionID)
	})

	// Test updating existing session
	t.Run("update existing session", func(t *testing.T) {
		// Arrange
		originalTime := time.Now().Add(-1 * time.Hour)
		sessions := SessionsState{
			{
				SessionID:      "update-me",
				StartedAt:      originalTime,
				LastUpdated:    originalTime,
				TranscriptPath: "/old/path",
			},
			{
				SessionID:   "keep-me",
				StartedAt:   originalTime,
				LastUpdated: originalTime,
			},
		}

		updatedSession := &State{
			SessionID:      "update-me",
			StartedAt:      originalTime,
			LastUpdated:    time.Now(),
			TranscriptPath: "/new/path",
		}

		// Act
		sessions.AddOrUpdate(updatedSession)

		// Assert
		require.Len(t, sessions, 2, "Should still have 2 sessions")

		// Find the updated session
		updated, index := sessions.FindBySessionID("update-me")
		require.NotNil(t, updated)
		assert.Equal(t, 0, index, "Updated session should remain at same position")
		assert.Equal(t, "/new/path", updated.TranscriptPath)
		assert.True(t, updated.LastUpdated.After(originalTime))

		// Verify other session unchanged
		other, _ := sessions.FindBySessionID("keep-me")
		require.NotNil(t, other)
		assert.True(t, other.LastUpdated.Equal(originalTime))
	})
}
