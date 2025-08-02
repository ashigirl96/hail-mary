package settings

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
)

// ClaudeSettings represents the Claude settings structure
type ClaudeSettings struct {
	Hooks map[string][]HookMatcher `json:"hooks,omitempty"`
	// Add other Claude settings fields as needed
	// This is a flexible structure to preserve unknown fields
	Extra map[string]interface{} `json:"-"`
}

// HookMatcher represents a hook matcher configuration
type HookMatcher struct {
	Matcher string      `json:"matcher,omitempty"`
	Hooks   []HookEntry `json:"hooks"`
}

// HookEntry represents a single hook configuration
type HookEntry struct {
	Type    string `json:"type"`
	Command string `json:"command"`
	Timeout int    `json:"timeout,omitempty"`
}

// UnmarshalJSON implements custom unmarshalling to preserve unknown fields
func (cs *ClaudeSettings) UnmarshalJSON(data []byte) error {
	// First, unmarshal into a map to capture all fields
	var raw map[string]interface{}
	if err := json.Unmarshal(data, &raw); err != nil {
		return err
	}

	// Extract hooks if present
	if hooksRaw, ok := raw["hooks"]; ok {
		hooksData, err := json.Marshal(hooksRaw)
		if err != nil {
			return err
		}
		if err := json.Unmarshal(hooksData, &cs.Hooks); err != nil {
			return err
		}
		delete(raw, "hooks")
	}

	// Store remaining fields in Extra
	cs.Extra = raw
	return nil
}

// MarshalJSON implements custom marshalling to include unknown fields
func (cs *ClaudeSettings) MarshalJSON() ([]byte, error) {
	// Start with extra fields
	result := make(map[string]interface{})
	for k, v := range cs.Extra {
		result[k] = v
	}

	// Add hooks if present
	if len(cs.Hooks) > 0 {
		result["hooks"] = cs.Hooks
	}

	return json.Marshal(result)
}

// LoadSettings loads Claude settings from a file
func LoadSettings(path string) (*ClaudeSettings, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		if os.IsNotExist(err) {
			// Return empty settings if file doesn't exist
			return &ClaudeSettings{
				Hooks: make(map[string][]HookMatcher),
				Extra: make(map[string]interface{}),
			}, nil
		}
		return nil, fmt.Errorf("failed to read settings file: %w", err)
	}

	var settings ClaudeSettings
	if err := json.Unmarshal(data, &settings); err != nil {
		return nil, fmt.Errorf("failed to parse settings file: %w", err)
	}

	// Ensure maps are initialized
	if settings.Hooks == nil {
		settings.Hooks = make(map[string][]HookMatcher)
	}
	if settings.Extra == nil {
		settings.Extra = make(map[string]interface{})
	}

	return &settings, nil
}

// MergeHooks merges new hooks into existing settings
// If a hook event already exists, it appends new matchers to it
func (cs *ClaudeSettings) MergeHooks(newHooks map[string][]HookMatcher) {
	if cs.Hooks == nil {
		cs.Hooks = make(map[string][]HookMatcher)
	}

	for event, matchers := range newHooks {
		cs.Hooks[event] = append(cs.Hooks[event], matchers...)
	}
}

// SaveToFile saves settings to a file with proper formatting
func (cs *ClaudeSettings) SaveToFile(path string) error {
	data, err := json.MarshalIndent(cs, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal settings: %w", err)
	}

	// Ensure directory exists
	dir := filepath.Dir(path)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return fmt.Errorf("failed to create directory: %w", err)
	}

	if err := os.WriteFile(path, data, 0644); err != nil {
		return fmt.Errorf("failed to write settings file: %w", err)
	}

	return nil
}

// CreateMergedSettings loads existing settings and merges with new hooks
func CreateMergedSettings(existingPath string, newHooks map[string][]HookMatcher) (*ClaudeSettings, error) {
	// Load existing settings (or create empty if doesn't exist)
	settings, err := LoadSettings(existingPath)
	if err != nil {
		return nil, err
	}

	// Merge new hooks
	settings.MergeHooks(newHooks)

	return settings, nil
}
