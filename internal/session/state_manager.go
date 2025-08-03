package session

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

// StateManager handles session state persistence with a configurable directory
type StateManager struct {
	stateDir string
}

// NewStateManager creates a new session state manager with the specified directory
func NewStateManager(stateDir string) *StateManager {
	return &StateManager{
		stateDir: stateDir,
	}
}

// NewFeatureStateManager creates a state manager for a specific feature
func NewFeatureStateManager(featureDir string) *StateManager {
	sessionsDir := filepath.Join(featureDir, "sessions")
	return &StateManager{
		stateDir: sessionsDir,
	}
}

// SaveState saves a session state to disk
func (sm *StateManager) SaveState(state *State) error {
	// Ensure directory exists
	if err := os.MkdirAll(sm.stateDir, 0755); err != nil {
		return fmt.Errorf("failed to create state directory: %w", err)
	}

	// Marshal state to JSON
	data, err := json.MarshalIndent(state, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal state: %w", err)
	}

	// Write to file
	filePath := filepath.Join(sm.stateDir, state.SessionID+".json")
	if err := os.WriteFile(filePath, data, 0644); err != nil {
		return fmt.Errorf("failed to write state file: %w", err)
	}

	return nil
}

// LoadState loads a session state from disk
func (sm *StateManager) LoadState(sessionID string) (*State, error) {
	filePath := filepath.Join(sm.stateDir, sessionID+".json")

	data, err := os.ReadFile(filePath)
	if err != nil {
		return nil, fmt.Errorf("failed to read state file: %w", err)
	}

	var state State
	if err := json.Unmarshal(data, &state); err != nil {
		return nil, fmt.Errorf("failed to unmarshal state: %w", err)
	}

	return &state, nil
}

// DeleteState removes a session state from disk
func (sm *StateManager) DeleteState(sessionID string) error {
	filePath := filepath.Join(sm.stateDir, sessionID+".json")

	if err := os.Remove(filePath); err != nil {
		return fmt.Errorf("failed to delete state file: %w", err)
	}

	return nil
}

// ListStates returns all session states in the directory
func (sm *StateManager) ListStates() ([]*State, error) {
	entries, err := os.ReadDir(sm.stateDir)
	if err != nil {
		if os.IsNotExist(err) {
			return []*State{}, nil
		}
		return nil, fmt.Errorf("failed to read state directory: %w", err)
	}

	var states []*State
	for _, entry := range entries {
		if entry.IsDir() || !strings.HasSuffix(entry.Name(), ".json") {
			continue
		}

		sessionID := strings.TrimSuffix(entry.Name(), ".json")
		state, err := sm.LoadState(sessionID)
		if err != nil {
			// Skip invalid files but continue processing
			continue
		}

		states = append(states, state)
	}

	return states, nil
}

// LoadSessions loads all sessions from a sessions.json file
func (sm *StateManager) LoadSessions() (SessionsState, error) {
	// For feature state manager, use the parent directory
	sessionsPath := filepath.Join(filepath.Dir(sm.stateDir), "sessions.json")

	data, err := os.ReadFile(sessionsPath)
	if err != nil {
		if os.IsNotExist(err) {
			// If file doesn't exist, try to migrate from individual files
			return sm.migrateFromIndividualFiles()
		}
		return nil, fmt.Errorf("failed to read sessions file: %w", err)
	}

	var sessions SessionsState
	if err := json.Unmarshal(data, &sessions); err != nil {
		return nil, fmt.Errorf("failed to unmarshal sessions: %w", err)
	}

	return sessions, nil
}

// SaveSessions saves all sessions to a sessions.json file
func (sm *StateManager) SaveSessions(sessions SessionsState) error {
	// For feature state manager, use the parent directory
	sessionsPath := filepath.Join(filepath.Dir(sm.stateDir), "sessions.json")

	// Ensure parent directory exists
	if err := os.MkdirAll(filepath.Dir(sessionsPath), 0755); err != nil {
		return fmt.Errorf("failed to create directory: %w", err)
	}

	// Marshal sessions to JSON with pretty formatting
	data, err := json.MarshalIndent(sessions, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal sessions: %w", err)
	}

	// Write to temp file first for atomic operation
	tempPath := sessionsPath + ".tmp"
	if err := os.WriteFile(tempPath, data, 0644); err != nil {
		return fmt.Errorf("failed to write temp file: %w", err)
	}

	// Atomic rename
	if err := os.Rename(tempPath, sessionsPath); err != nil {
		// Clean up temp file if rename fails
		os.Remove(tempPath)
		return fmt.Errorf("failed to rename temp file: %w", err)
	}

	return nil
}

// AddOrUpdateSession adds or updates a session in the sessions.json file
func (sm *StateManager) AddOrUpdateSession(state *State) error {
	// Load existing sessions
	sessions, err := sm.LoadSessions()
	if err != nil {
		// If error is not because file doesn't exist, return error
		if !os.IsNotExist(err) {
			return fmt.Errorf("failed to load sessions: %w", err)
		}
		// Initialize empty sessions if file doesn't exist
		sessions = SessionsState{}
	}

	// Add or update the session
	sessions.AddOrUpdate(state)

	// Save back to file
	return sm.SaveSessions(sessions)
}

// migrateFromIndividualFiles migrates from individual JSON files to sessions.json
func (sm *StateManager) migrateFromIndividualFiles() (SessionsState, error) {
	// List all individual session files
	states, err := sm.ListStates()
	if err != nil {
		return nil, fmt.Errorf("failed to list states for migration: %w", err)
	}

	// Convert to SessionsState
	sessions := SessionsState(states)

	// Save to sessions.json
	if len(sessions) > 0 {
		if err := sm.SaveSessions(sessions); err != nil {
			return nil, fmt.Errorf("failed to save migrated sessions: %w", err)
		}

		// Optionally clean up old individual files
		for _, state := range states {
			filePath := filepath.Join(sm.stateDir, state.SessionID+".json")
			os.Remove(filePath)
		}
	}

	return sessions, nil
}
