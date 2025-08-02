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
