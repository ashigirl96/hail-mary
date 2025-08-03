package session

import (
	"encoding/json"
	"os"
	"path/filepath"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestFeatureStateManager(t *testing.T) {
	// Test NewFeatureStateManager
	t.Run("NewFeatureStateManager creates correct directory structure", func(t *testing.T) {
		tempDir := t.TempDir()
		featureDir := filepath.Join(tempDir, ".kiro", "spec", "my-feature")

		manager := NewFeatureStateManager(featureDir)

		expectedDir := filepath.Join(featureDir, "sessions")
		assert.Equal(t, expectedDir, manager.stateDir)
	})
}

func TestStateManager_LoadSessions(t *testing.T) {
	t.Run("load from valid sessions.json", func(t *testing.T) {
		// Arrange
		tempDir := t.TempDir()
		featureDir := filepath.Join(tempDir, ".kiro", "spec", "test-feature")
		err := os.MkdirAll(featureDir, 0755)
		require.NoError(t, err)

		// Create sessions.json with test data
		sessions := SessionsState{
			{
				SessionID:      "session-1",
				StartedAt:      time.Now().Add(-2 * time.Hour),
				LastUpdated:    time.Now().Add(-1 * time.Hour),
				TranscriptPath: "/path/to/transcript1.jsonl",
				ProjectDir:     "/project",
			},
			{
				SessionID:      "session-2",
				StartedAt:      time.Now().Add(-1 * time.Hour),
				LastUpdated:    time.Now(),
				TranscriptPath: "/path/to/transcript2.jsonl",
				ProjectDir:     "/project",
			},
		}

		data, err := json.MarshalIndent(sessions, "", "  ")
		require.NoError(t, err)

		sessionsPath := filepath.Join(featureDir, "sessions.json")
		err = os.WriteFile(sessionsPath, data, 0644)
		require.NoError(t, err)

		// Act
		manager := NewFeatureStateManager(featureDir)
		loadedSessions, err := manager.LoadSessions()

		// Assert
		require.NoError(t, err)
		require.Len(t, loadedSessions, 2)
		assert.Equal(t, "session-1", loadedSessions[0].SessionID)
		assert.Equal(t, "session-2", loadedSessions[1].SessionID)
	})

	t.Run("file not exists triggers migration", func(t *testing.T) {
		// Arrange
		tempDir := t.TempDir()
		featureDir := filepath.Join(tempDir, ".kiro", "spec", "test-feature")
		sessionsDir := filepath.Join(featureDir, "sessions")
		err := os.MkdirAll(sessionsDir, 0755)
		require.NoError(t, err)

		// Create individual session files
		session1 := &State{
			SessionID:      "migrate-1",
			StartedAt:      time.Now().Add(-2 * time.Hour),
			LastUpdated:    time.Now().Add(-1 * time.Hour),
			TranscriptPath: "/path/to/transcript1.jsonl",
		}

		session2 := &State{
			SessionID:      "migrate-2",
			StartedAt:      time.Now().Add(-1 * time.Hour),
			LastUpdated:    time.Now(),
			TranscriptPath: "/path/to/transcript2.jsonl",
		}

		// Save individual files
		data1, _ := json.MarshalIndent(session1, "", "  ")
		err = os.WriteFile(filepath.Join(sessionsDir, "migrate-1.json"), data1, 0644)
		require.NoError(t, err)

		data2, _ := json.MarshalIndent(session2, "", "  ")
		err = os.WriteFile(filepath.Join(sessionsDir, "migrate-2.json"), data2, 0644)
		require.NoError(t, err)

		// Act
		manager := NewFeatureStateManager(featureDir)
		loadedSessions, err := manager.LoadSessions()

		// Assert
		require.NoError(t, err)
		require.Len(t, loadedSessions, 2)

		// Verify sessions.json was created
		sessionsPath := filepath.Join(featureDir, "sessions.json")
		_, err = os.Stat(sessionsPath)
		assert.NoError(t, err, "sessions.json should be created after migration")

		// Verify individual files were removed
		_, err = os.Stat(filepath.Join(sessionsDir, "migrate-1.json"))
		assert.True(t, os.IsNotExist(err), "Individual files should be removed after migration")
	})

	t.Run("invalid JSON returns error", func(t *testing.T) {
		// Arrange
		tempDir := t.TempDir()
		featureDir := filepath.Join(tempDir, ".kiro", "spec", "test-feature")
		err := os.MkdirAll(featureDir, 0755)
		require.NoError(t, err)

		sessionsPath := filepath.Join(featureDir, "sessions.json")
		err = os.WriteFile(sessionsPath, []byte("invalid json"), 0644)
		require.NoError(t, err)

		// Act
		manager := NewFeatureStateManager(featureDir)
		_, err = manager.LoadSessions()

		// Assert
		require.Error(t, err)
		assert.Contains(t, err.Error(), "failed to unmarshal sessions")
	})
}

func TestStateManager_SaveSessions(t *testing.T) {
	t.Run("save sessions to file", func(t *testing.T) {
		// Arrange
		tempDir := t.TempDir()
		featureDir := filepath.Join(tempDir, ".kiro", "spec", "test-feature")

		sessions := SessionsState{
			{
				SessionID:      "save-1",
				StartedAt:      time.Now(),
				LastUpdated:    time.Now(),
				TranscriptPath: "/path/to/transcript.jsonl",
				ProjectDir:     "/project",
			},
		}

		// Act
		manager := NewFeatureStateManager(featureDir)
		err := manager.SaveSessions(sessions)

		// Assert
		require.NoError(t, err)

		// Verify file exists and content is correct
		sessionsPath := filepath.Join(featureDir, "sessions.json")
		data, err := os.ReadFile(sessionsPath)
		require.NoError(t, err)

		var loaded SessionsState
		err = json.Unmarshal(data, &loaded)
		require.NoError(t, err)
		require.Len(t, loaded, 1)
		assert.Equal(t, "save-1", loaded[0].SessionID)
	})

	t.Run("atomic save with temp file", func(t *testing.T) {
		// This test verifies that the save operation is atomic
		tempDir := t.TempDir()
		featureDir := filepath.Join(tempDir, ".kiro", "spec", "test-feature")
		err := os.MkdirAll(featureDir, 0755)
		require.NoError(t, err)

		// Create existing sessions.json
		existingPath := filepath.Join(featureDir, "sessions.json")
		err = os.WriteFile(existingPath, []byte(`[{"session_id": "existing"}]`), 0644)
		require.NoError(t, err)

		sessions := SessionsState{
			{SessionID: "new-session"},
		}

		// Act
		manager := NewFeatureStateManager(featureDir)
		err = manager.SaveSessions(sessions)

		// Assert
		require.NoError(t, err)

		// Verify temp file doesn't exist
		tempPath := existingPath + ".tmp"
		_, err = os.Stat(tempPath)
		assert.True(t, os.IsNotExist(err), "Temp file should be cleaned up")

		// Verify new content
		data, _ := os.ReadFile(existingPath)
		assert.Contains(t, string(data), "new-session")
		assert.NotContains(t, string(data), "existing")
	})
}

func TestStateManager_AddOrUpdateSession(t *testing.T) {
	t.Run("add new session when no sessions.json exists", func(t *testing.T) {
		// Arrange
		tempDir := t.TempDir()
		featureDir := filepath.Join(tempDir, ".kiro", "spec", "test-feature")

		newSession := &State{
			SessionID:      "brand-new",
			StartedAt:      time.Now(),
			LastUpdated:    time.Now(),
			TranscriptPath: "/path/to/transcript.jsonl",
		}

		// Act
		manager := NewFeatureStateManager(featureDir)
		err := manager.AddOrUpdateSession(newSession)

		// Assert
		require.NoError(t, err)

		// Load and verify
		sessions, err := manager.LoadSessions()
		require.NoError(t, err)
		require.Len(t, sessions, 1)
		assert.Equal(t, "brand-new", sessions[0].SessionID)
	})

	t.Run("update existing session", func(t *testing.T) {
		// Arrange
		tempDir := t.TempDir()
		featureDir := filepath.Join(tempDir, ".kiro", "spec", "test-feature")
		manager := NewFeatureStateManager(featureDir)

		// Add initial session
		initialSession := &State{
			SessionID:      "update-me",
			StartedAt:      time.Now().Add(-1 * time.Hour),
			LastUpdated:    time.Now().Add(-1 * time.Hour),
			TranscriptPath: "/old/path.jsonl",
		}
		err := manager.AddOrUpdateSession(initialSession)
		require.NoError(t, err)

		// Update session
		updatedSession := &State{
			SessionID:      "update-me",
			StartedAt:      initialSession.StartedAt,
			LastUpdated:    time.Now(),
			TranscriptPath: "/new/path.jsonl",
		}

		// Act
		err = manager.AddOrUpdateSession(updatedSession)

		// Assert
		require.NoError(t, err)

		sessions, err := manager.LoadSessions()
		require.NoError(t, err)
		require.Len(t, sessions, 1)
		assert.Equal(t, "/new/path.jsonl", sessions[0].TranscriptPath)
		assert.True(t, sessions[0].LastUpdated.After(initialSession.LastUpdated))
	})

	t.Run("add multiple different sessions", func(t *testing.T) {
		// Arrange
		tempDir := t.TempDir()
		featureDir := filepath.Join(tempDir, ".kiro", "spec", "test-feature")
		manager := NewFeatureStateManager(featureDir)

		session1 := &State{
			SessionID:   "session-1",
			StartedAt:   time.Now().Add(-2 * time.Hour),
			LastUpdated: time.Now().Add(-2 * time.Hour),
		}

		session2 := &State{
			SessionID:   "session-2",
			StartedAt:   time.Now().Add(-1 * time.Hour),
			LastUpdated: time.Now().Add(-1 * time.Hour),
		}

		// Act
		err := manager.AddOrUpdateSession(session1)
		require.NoError(t, err)

		err = manager.AddOrUpdateSession(session2)
		require.NoError(t, err)

		// Assert
		sessions, err := manager.LoadSessions()
		require.NoError(t, err)
		require.Len(t, sessions, 2)

		// Newer sessions should be first
		assert.Equal(t, "session-2", sessions[0].SessionID)
		assert.Equal(t, "session-1", sessions[1].SessionID)
	})
}

func TestStateManager_MigrateFromIndividualFiles(t *testing.T) {
	t.Run("successful migration", func(t *testing.T) {
		// Arrange
		tempDir := t.TempDir()
		featureDir := filepath.Join(tempDir, ".kiro", "spec", "test-feature")
		sessionsDir := filepath.Join(featureDir, "sessions")
		err := os.MkdirAll(sessionsDir, 0755)
		require.NoError(t, err)

		// Create individual session files
		sessions := []*State{
			{
				SessionID:   "old-1",
				StartedAt:   time.Now().Add(-3 * time.Hour),
				LastUpdated: time.Now().Add(-3 * time.Hour),
			},
			{
				SessionID:   "old-2",
				StartedAt:   time.Now().Add(-2 * time.Hour),
				LastUpdated: time.Now().Add(-2 * time.Hour),
			},
			{
				SessionID:   "old-3",
				StartedAt:   time.Now().Add(-1 * time.Hour),
				LastUpdated: time.Now().Add(-1 * time.Hour),
			},
		}

		for _, session := range sessions {
			data, _ := json.MarshalIndent(session, "", "  ")
			filePath := filepath.Join(sessionsDir, session.SessionID+".json")
			err = os.WriteFile(filePath, data, 0644)
			require.NoError(t, err)
		}

		// Act
		manager := NewFeatureStateManager(featureDir)
		migrated, err := manager.migrateFromIndividualFiles()

		// Assert
		require.NoError(t, err)
		require.Len(t, migrated, 3)

		// Verify sessions.json exists
		sessionsPath := filepath.Join(featureDir, "sessions.json")
		_, err = os.Stat(sessionsPath)
		assert.NoError(t, err)

		// Verify individual files are removed
		for _, session := range sessions {
			filePath := filepath.Join(sessionsDir, session.SessionID+".json")
			_, err = os.Stat(filePath)
			assert.True(t, os.IsNotExist(err), "Individual file should be removed: %s", filePath)
		}

		// Verify content
		data, _ := os.ReadFile(sessionsPath)
		assert.Contains(t, string(data), "old-1")
		assert.Contains(t, string(data), "old-2")
		assert.Contains(t, string(data), "old-3")
	})

	t.Run("empty directory migration", func(t *testing.T) {
		// Arrange
		tempDir := t.TempDir()
		featureDir := filepath.Join(tempDir, ".kiro", "spec", "test-feature")
		sessionsDir := filepath.Join(featureDir, "sessions")
		err := os.MkdirAll(sessionsDir, 0755)
		require.NoError(t, err)

		// Act
		manager := NewFeatureStateManager(featureDir)
		migrated, err := manager.migrateFromIndividualFiles()

		// Assert
		require.NoError(t, err)
		require.Len(t, migrated, 0)

		// Verify sessions.json is not created for empty migration
		sessionsPath := filepath.Join(featureDir, "sessions.json")
		_, err = os.Stat(sessionsPath)
		assert.True(t, os.IsNotExist(err), "sessions.json should not be created for empty migration")
	})
}
