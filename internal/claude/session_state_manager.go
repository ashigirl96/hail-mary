package claude

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"time"
)

// SessionState represents the state of a Claude session
type SessionState struct {
	SessionID      string    `json:"session_id"`
	StartedAt      time.Time `json:"started_at"`
	LastUpdated    time.Time `json:"last_updated"`
	TranscriptPath string    `json:"transcript_path"`
	ProjectDir     string    `json:"project_dir"`
}

// FeatureSessions represents a collection of sessions for a feature
type FeatureSessions []*SessionState

// FindBySessionID finds a session by its ID
func (ss FeatureSessions) FindBySessionID(sessionID string) (*SessionState, int) {
	for i, state := range ss {
		if state.SessionID == sessionID {
			return state, i
		}
	}
	return nil, -1
}

// AddOrUpdate adds a new session or updates an existing one
func (ss *FeatureSessions) AddOrUpdate(newState *SessionState) {
	existing, index := ss.FindBySessionID(newState.SessionID)
	if existing != nil {
		// Update existing session
		(*ss)[index] = newState
	} else {
		// Add new session at the beginning
		*ss = append([]*SessionState{newState}, *ss...)
	}
}

// SessionStateManager handles session state persistence with thread safety
type SessionStateManager struct {
	stateDir string
	mu       sync.RWMutex
}

// NewSessionStateManager creates a new session state manager with the specified directory
func NewSessionStateManager(stateDir string) *SessionStateManager {
	return &SessionStateManager{
		stateDir: stateDir,
	}
}

// NewFeatureSessionStateManager creates a session state manager for a specific feature
func NewFeatureSessionStateManager(featureDir string) *SessionStateManager {
	sessionsDir := filepath.Join(featureDir, "sessions")
	return &SessionStateManager{
		stateDir: sessionsDir,
	}
}

// SaveSessionState saves a session state to disk
func (sm *SessionStateManager) SaveSessionState(state *SessionState) error {
	sm.mu.Lock()
	defer sm.mu.Unlock()

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

// LoadSessionState loads a session state from disk
func (sm *SessionStateManager) LoadSessionState(sessionID string) (*SessionState, error) {
	sm.mu.RLock()
	defer sm.mu.RUnlock()

	filePath := filepath.Join(sm.stateDir, sessionID+".json")

	data, err := os.ReadFile(filePath)
	if err != nil {
		return nil, fmt.Errorf("failed to read state file: %w", err)
	}

	var state SessionState
	if err := json.Unmarshal(data, &state); err != nil {
		return nil, fmt.Errorf("failed to unmarshal state: %w", err)
	}

	return &state, nil
}

// DeleteSessionState removes a session state from disk
func (sm *SessionStateManager) DeleteSessionState(sessionID string) error {
	sm.mu.Lock()
	defer sm.mu.Unlock()

	filePath := filepath.Join(sm.stateDir, sessionID+".json")

	if err := os.Remove(filePath); err != nil {
		return fmt.Errorf("failed to delete state file: %w", err)
	}

	return nil
}

// ListSessionStates returns all session states in the directory
func (sm *SessionStateManager) ListSessionStates() ([]*SessionState, error) {
	sm.mu.RLock()
	defer sm.mu.RUnlock()

	entries, err := os.ReadDir(sm.stateDir)
	if err != nil {
		if os.IsNotExist(err) {
			return []*SessionState{}, nil
		}
		return nil, fmt.Errorf("failed to read state directory: %w", err)
	}

	var states []*SessionState
	for _, entry := range entries {
		if entry.IsDir() || !strings.HasSuffix(entry.Name(), ".json") {
			continue
		}

		sessionID := strings.TrimSuffix(entry.Name(), ".json")
		state, err := sm.loadStateUnsafe(sessionID) // Use unsafe version since we already have read lock
		if err != nil {
			// Skip invalid files but continue processing
			continue
		}

		states = append(states, state)
	}

	return states, nil
}

// loadStateUnsafe loads state without acquiring mutex (for internal use when already locked)
func (sm *SessionStateManager) loadStateUnsafe(sessionID string) (*SessionState, error) {
	filePath := filepath.Join(sm.stateDir, sessionID+".json")

	data, err := os.ReadFile(filePath)
	if err != nil {
		return nil, fmt.Errorf("failed to read state file: %w", err)
	}

	var state SessionState
	if err := json.Unmarshal(data, &state); err != nil {
		return nil, fmt.Errorf("failed to unmarshal state: %w", err)
	}

	return &state, nil
}

// LoadSessions loads all sessions from a sessions.json file
func (sm *SessionStateManager) LoadSessions() (FeatureSessions, error) {
	sm.mu.RLock()
	defer sm.mu.RUnlock()

	// For feature state manager, use the parent directory
	sessionsPath := filepath.Join(filepath.Dir(sm.stateDir), "sessions.json")

	data, err := os.ReadFile(sessionsPath)
	if err != nil {
		if os.IsNotExist(err) {
			// If file doesn't exist, try to migrate from individual files
			return sm.migrateFromIndividualFilesUnsafe()
		}
		return nil, fmt.Errorf("failed to read sessions file: %w", err)
	}

	var sessions FeatureSessions
	if err := json.Unmarshal(data, &sessions); err != nil {
		return nil, fmt.Errorf("failed to unmarshal sessions: %w", err)
	}

	return sessions, nil
}

// SaveSessions saves all sessions to a sessions.json file
func (sm *SessionStateManager) SaveSessions(sessions FeatureSessions) error {
	sm.mu.Lock()
	defer sm.mu.Unlock()

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
func (sm *SessionStateManager) AddOrUpdateSession(state *SessionState) error {
	// Load existing sessions
	sessions, err := sm.LoadSessions()
	if err != nil {
		// If error is not because file doesn't exist, return error
		if !os.IsNotExist(err) {
			return fmt.Errorf("failed to load sessions: %w", err)
		}
		// Initialize empty sessions if file doesn't exist
		sessions = FeatureSessions{}
	}

	// Add or update the session
	sessions.AddOrUpdate(state)

	// Save back to file
	return sm.SaveSessions(sessions)
}

// migrateFromIndividualFilesUnsafe migrates from individual JSON files to sessions.json
// This is the unsafe version that doesn't acquire locks (for internal use when already locked)
func (sm *SessionStateManager) migrateFromIndividualFilesUnsafe() (FeatureSessions, error) {
	// List all individual session files
	entries, err := os.ReadDir(sm.stateDir)
	if err != nil {
		if os.IsNotExist(err) {
			return FeatureSessions{}, nil
		}
		return nil, fmt.Errorf("failed to read state directory: %w", err)
	}

	var states []*SessionState
	for _, entry := range entries {
		if entry.IsDir() || !strings.HasSuffix(entry.Name(), ".json") {
			continue
		}

		sessionID := strings.TrimSuffix(entry.Name(), ".json")
		state, err := sm.loadStateUnsafe(sessionID)
		if err != nil {
			// Skip invalid files but continue processing
			continue
		}

		states = append(states, state)
	}

	// Convert to FeatureSessions
	sessions := FeatureSessions(states)

	// Note: Caller should handle saving to sessions.json if needed
	// since this function is called from LoadSessions which already holds read lock

	return sessions, nil
}
