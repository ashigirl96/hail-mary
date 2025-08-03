package session

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"time"
)

// State represents the state of a Claude session
type State struct {
	SessionID      string    `json:"session_id"`
	StartedAt      time.Time `json:"started_at"`
	LastUpdated    time.Time `json:"last_updated"`
	TranscriptPath string    `json:"transcript_path"`
	ProjectDir     string    `json:"project_dir"`
}

// Manager handles session state persistence
type Manager struct {
	stateDir string
	mu       sync.RWMutex
}

// NewManager creates a new session manager
func NewManager() (*Manager, error) {
	homeDir, err := os.UserHomeDir()
	if err != nil {
		return nil, fmt.Errorf("failed to get home directory: %w", err)
	}

	stateDir := filepath.Join(homeDir, ".hail-mary", "sessions")
	if err := os.MkdirAll(stateDir, 0755); err != nil {
		return nil, fmt.Errorf("failed to create state directory: %w", err)
	}

	return &Manager{stateDir: stateDir}, nil
}

// WriteSession atomically writes session state
func (sm *Manager) WriteSession(processID string, state *State) error {
	sm.mu.Lock()
	defer sm.mu.Unlock()

	statePath := filepath.Join(sm.stateDir, processID+".json")
	tempPath := statePath + ".tmp"

	data, err := json.MarshalIndent(state, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal session state: %w", err)
	}

	// Write to temp file
	if err := os.WriteFile(tempPath, data, 0644); err != nil {
		return fmt.Errorf("failed to write temp file: %w", err)
	}

	// Atomic rename
	if err := os.Rename(tempPath, statePath); err != nil {
		// Clean up temp file if rename fails
		os.Remove(tempPath)
		return fmt.Errorf("failed to rename temp file: %w", err)
	}

	return nil
}

// ReadSession reads session state for a process
func (sm *Manager) ReadSession(processID string) (*State, error) {
	sm.mu.RLock()
	defer sm.mu.RUnlock()

	statePath := filepath.Join(sm.stateDir, processID+".json")
	data, err := os.ReadFile(statePath)
	if err != nil {
		if os.IsNotExist(err) {
			return nil, fmt.Errorf("session not found for process %s", processID)
		}
		return nil, fmt.Errorf("failed to read session file: %w", err)
	}

	var state State
	if err := json.Unmarshal(data, &state); err != nil {
		return nil, fmt.Errorf("failed to unmarshal session state: %w", err)
	}

	return &state, nil
}

// UpdateSession updates the last_updated timestamp
func (sm *Manager) UpdateSession(processID string) error {
	sm.mu.Lock()
	defer sm.mu.Unlock()

	statePath := filepath.Join(sm.stateDir, processID+".json")
	data, err := os.ReadFile(statePath)
	if err != nil {
		return fmt.Errorf("failed to read session file: %w", err)
	}

	var state State
	if err := json.Unmarshal(data, &state); err != nil {
		return fmt.Errorf("failed to unmarshal session state: %w", err)
	}

	state.LastUpdated = time.Now()

	// Write back atomically
	tempPath := statePath + ".tmp"
	newData, err := json.MarshalIndent(state, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal updated state: %w", err)
	}

	if err := os.WriteFile(tempPath, newData, 0644); err != nil {
		return fmt.Errorf("failed to write temp file: %w", err)
	}

	if err := os.Rename(tempPath, statePath); err != nil {
		os.Remove(tempPath)
		return fmt.Errorf("failed to rename temp file: %w", err)
	}

	return nil
}

// CleanupSession removes session state
func (sm *Manager) CleanupSession(processID string) error {
	sm.mu.Lock()
	defer sm.mu.Unlock()

	statePath := filepath.Join(sm.stateDir, processID+".json")
	if err := os.Remove(statePath); err != nil && !os.IsNotExist(err) {
		return fmt.Errorf("failed to remove session file: %w", err)
	}

	return nil
}

// ListSessions returns all active sessions
func (sm *Manager) ListSessions() ([]*State, error) {
	sm.mu.RLock()
	defer sm.mu.RUnlock()

	entries, err := os.ReadDir(sm.stateDir)
	if err != nil {
		return nil, fmt.Errorf("failed to read state directory: %w", err)
	}

	var sessions []*State
	for _, entry := range entries {
		if entry.IsDir() || !strings.HasSuffix(entry.Name(), ".json") {
			continue
		}

		data, err := os.ReadFile(filepath.Join(sm.stateDir, entry.Name()))
		if err != nil {
			continue // Skip files that can't be read
		}

		var state State
		if err := json.Unmarshal(data, &state); err != nil {
			continue // Skip invalid JSON
		}

		sessions = append(sessions, &state)
	}

	return sessions, nil
}

// CleanupStale removes session files older than the specified duration
func (sm *Manager) CleanupStale(maxAge time.Duration) error {
	sm.mu.Lock()
	defer sm.mu.Unlock()

	entries, err := os.ReadDir(sm.stateDir)
	if err != nil {
		return fmt.Errorf("failed to read state directory: %w", err)
	}

	now := time.Now()
	for _, entry := range entries {
		if entry.IsDir() || !strings.HasSuffix(entry.Name(), ".json") {
			continue
		}

		info, err := entry.Info()
		if err != nil {
			continue
		}

		if now.Sub(info.ModTime()) > maxAge {
			os.Remove(filepath.Join(sm.stateDir, entry.Name()))
		}
	}

	return nil
}

// WriteSessionToFeature writes session state to a feature-specific directory
func (sm *Manager) WriteSessionToFeature(featureDir string, state *State) error {
	sm.mu.Lock()
	defer sm.mu.Unlock()

	// Create feature state manager
	featureManager := NewFeatureStateManager(featureDir)

	// Save state using the feature manager
	return featureManager.SaveState(state)
}
