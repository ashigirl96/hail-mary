package settings

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"testing"
)

func TestMergeSettings(t *testing.T) {
	// Create a temporary directory for testing
	tempDir := t.TempDir()
	settingsPath := filepath.Join(tempDir, "settings.json")

	// Create an existing settings file
	existingSettings := &ClaudeSettings{
		Hooks: map[string][]HookMatcher{
			"PreToolUse": {
				{
					Matcher: "Write",
					Hooks: []HookEntry{
						{
							Type:    "command",
							Command: "echo 'existing hook'",
						},
					},
				},
			},
		},
		Extra: map[string]interface{}{
			"customSetting": "value",
		},
	}

	// Save existing settings
	if err := existingSettings.SaveToFile(settingsPath); err != nil {
		t.Fatalf("Failed to save existing settings: %v", err)
	}

	// Define new hooks to merge
	newHooks := map[string][]HookMatcher{
		"SessionStart": {
			{
				Hooks: []HookEntry{
					{
						Type:    "command",
						Command: "echo 'session start'",
						Timeout: 5,
					},
				},
			},
		},
		"PreToolUse": {
			{
				Matcher: "Read",
				Hooks: []HookEntry{
					{
						Type:    "command",
						Command: "echo 'new read hook'",
					},
				},
			},
		},
	}

	// Create merged settings
	merged, err := CreateMergedSettings(settingsPath, newHooks)
	if err != nil {
		t.Fatalf("Failed to create merged settings: %v", err)
	}

	// Verify the merge results
	// 1. Check that existing PreToolUse hook is preserved
	if len(merged.Hooks["PreToolUse"]) != 2 {
		t.Errorf("Expected 2 PreToolUse matchers, got %d", len(merged.Hooks["PreToolUse"]))
	}

	// 2. Check that new SessionStart hook is added
	if len(merged.Hooks["SessionStart"]) != 1 {
		t.Errorf("Expected 1 SessionStart matcher, got %d", len(merged.Hooks["SessionStart"]))
	}

	// 3. Check that extra settings are preserved
	if merged.Extra["customSetting"] != "value" {
		t.Errorf("Expected customSetting to be 'value', got %v", merged.Extra["customSetting"])
	}

	// Save merged settings and verify JSON structure
	mergedPath := filepath.Join(tempDir, "merged.json")
	if err := merged.SaveToFile(mergedPath); err != nil {
		t.Fatalf("Failed to save merged settings: %v", err)
	}

	// Read and parse the saved file to verify structure
	data, err := os.ReadFile(mergedPath)
	if err != nil {
		t.Fatalf("Failed to read merged file: %v", err)
	}

	var result map[string]interface{}
	if err := json.Unmarshal(data, &result); err != nil {
		t.Fatalf("Failed to parse merged JSON: %v", err)
	}

	// Verify that customSetting is at the top level
	if result["customSetting"] != "value" {
		t.Errorf("Expected customSetting at top level, not found")
	}

	// Verify hooks structure
	if _, ok := result["hooks"].(map[string]interface{}); !ok {
		t.Errorf("Expected hooks to be a map")
	}
}

func TestLoadNonExistentSettings(t *testing.T) {
	// Test loading a non-existent file
	settings, err := LoadSettings("/tmp/non-existent-settings.json")
	if err != nil {
		t.Fatalf("Expected no error for non-existent file, got: %v", err)
	}

	// Should return empty settings
	if settings.Hooks == nil || len(settings.Hooks) != 0 {
		t.Errorf("Expected empty hooks map, got %v", settings.Hooks)
	}

	if settings.Extra == nil || len(settings.Extra) != 0 {
		t.Errorf("Expected empty extra map, got %v", settings.Extra)
	}
}

// TestUnmarshalJSON tests the custom UnmarshalJSON method
func TestUnmarshalJSON(t *testing.T) {
	tests := []struct {
		name     string
		json     string
		wantErr  bool
		validate func(*ClaudeSettings) error
	}{
		{
			name: "valid settings with hooks and extra fields",
			json: `{
				"hooks": {
					"SessionStart": [{
						"hooks": [{"type": "command", "command": "test"}]
					}]
				},
				"customField": "customValue",
				"anotherField": 123
			}`,
			wantErr: false,
			validate: func(cs *ClaudeSettings) error {
				if len(cs.Hooks) != 1 {
					return fmt.Errorf("expected 1 hook, got %d", len(cs.Hooks))
				}
				if cs.Extra["customField"] != "customValue" {
					return fmt.Errorf("customField not preserved")
				}
				// JSON numbers are parsed as float64
				if cs.Extra["anotherField"] != float64(123) {
					return fmt.Errorf("anotherField not preserved")
				}
				return nil
			},
		},
		{
			name: "settings with only extra fields",
			json: `{
				"customField": "value",
				"nested": {"key": "value"}
			}`,
			wantErr: false,
			validate: func(cs *ClaudeSettings) error {
				if len(cs.Hooks) != 0 {
					return fmt.Errorf("expected no hooks")
				}
				if cs.Extra["customField"] != "value" {
					return fmt.Errorf("customField not preserved")
				}
				nested, ok := cs.Extra["nested"].(map[string]interface{})
				if !ok || nested["key"] != "value" {
					return fmt.Errorf("nested field not preserved correctly")
				}
				return nil
			},
		},
		{
			name: "settings with only hooks",
			json: `{
				"hooks": {
					"PreToolUse": [{
						"matcher": "Write",
						"hooks": [{"type": "command", "command": "test"}]
					}]
				}
			}`,
			wantErr: false,
			validate: func(cs *ClaudeSettings) error {
				if len(cs.Hooks) != 1 {
					return fmt.Errorf("expected 1 hook type")
				}
				if len(cs.Extra) != 0 {
					return fmt.Errorf("expected no extra fields")
				}
				return nil
			},
		},
		{
			name:    "invalid JSON",
			json:    `{invalid json}`,
			wantErr: true,
		},
		{
			name: "invalid hooks structure",
			json: `{
				"hooks": "not an object"
			}`,
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var cs ClaudeSettings
			err := json.Unmarshal([]byte(tt.json), &cs)

			if (err != nil) != tt.wantErr {
				t.Errorf("UnmarshalJSON() error = %v, wantErr %v", err, tt.wantErr)
			}

			if !tt.wantErr && tt.validate != nil {
				if err := tt.validate(&cs); err != nil {
					t.Errorf("Validation failed: %v", err)
				}
			}
		})
	}
}

// TestMarshalJSON tests the custom MarshalJSON method
func TestMarshalJSON(t *testing.T) {
	tests := []struct {
		name     string
		settings ClaudeSettings
		validate func(string) error
	}{
		{
			name: "settings with hooks and extra fields",
			settings: ClaudeSettings{
				Hooks: map[string][]HookMatcher{
					"SessionStart": {
						{
							Hooks: []HookEntry{
								{Type: "command", Command: "test"},
							},
						},
					},
				},
				Extra: map[string]interface{}{
					"customField": "value",
					"number":      123,
				},
			},
			validate: func(output string) error {
				var result map[string]interface{}
				if err := json.Unmarshal([]byte(output), &result); err != nil {
					return err
				}
				if result["customField"] != "value" {
					return fmt.Errorf("customField not in output")
				}
				if _, ok := result["hooks"]; !ok {
					return fmt.Errorf("hooks not in output")
				}
				return nil
			},
		},
		{
			name: "settings with only extra fields",
			settings: ClaudeSettings{
				Hooks: map[string][]HookMatcher{},
				Extra: map[string]interface{}{
					"field1": "value1",
					"field2": "value2",
				},
			},
			validate: func(output string) error {
				var result map[string]interface{}
				if err := json.Unmarshal([]byte(output), &result); err != nil {
					return err
				}
				if result["field1"] != "value1" || result["field2"] != "value2" {
					return fmt.Errorf("extra fields not preserved")
				}
				// Empty hooks should not be included
				if _, ok := result["hooks"]; ok {
					return fmt.Errorf("empty hooks should not be in output")
				}
				return nil
			},
		},
		{
			name: "settings with only hooks",
			settings: ClaudeSettings{
				Hooks: map[string][]HookMatcher{
					"PreToolUse": {
						{
							Matcher: "Write",
							Hooks: []HookEntry{
								{Type: "command", Command: "test"},
							},
						},
					},
				},
				Extra: map[string]interface{}{},
			},
			validate: func(output string) error {
				var result map[string]interface{}
				if err := json.Unmarshal([]byte(output), &result); err != nil {
					return err
				}
				if _, ok := result["hooks"]; !ok {
					return fmt.Errorf("hooks not in output")
				}
				// Should only have hooks field
				if len(result) != 1 {
					return fmt.Errorf("expected only hooks field, got %d fields", len(result))
				}
				return nil
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			data, err := json.Marshal(&tt.settings)
			if err != nil {
				t.Fatalf("MarshalJSON() error = %v", err)
			}

			if tt.validate != nil {
				if err := tt.validate(string(data)); err != nil {
					t.Errorf("Validation failed: %v", err)
				}
			}
		})
	}
}

// TestMergeHooks tests the MergeHooks method
func TestMergeHooks(t *testing.T) {
	tests := []struct {
		name       string
		initial    map[string][]HookMatcher
		newHooks   map[string][]HookMatcher
		wantCounts map[string]int
	}{
		{
			name:    "merge into nil hooks",
			initial: nil,
			newHooks: map[string][]HookMatcher{
				"SessionStart": {{Hooks: []HookEntry{{Type: "command", Command: "test"}}}},
			},
			wantCounts: map[string]int{"SessionStart": 1},
		},
		{
			name: "merge into existing hooks",
			initial: map[string][]HookMatcher{
				"PreToolUse": {{Matcher: "Write", Hooks: []HookEntry{{Type: "command", Command: "existing"}}}},
			},
			newHooks: map[string][]HookMatcher{
				"PreToolUse":   {{Matcher: "Read", Hooks: []HookEntry{{Type: "command", Command: "new"}}}},
				"SessionStart": {{Hooks: []HookEntry{{Type: "command", Command: "test"}}}},
			},
			wantCounts: map[string]int{"PreToolUse": 2, "SessionStart": 1},
		},
		{
			name: "merge empty hooks",
			initial: map[string][]HookMatcher{
				"PreToolUse": {{Hooks: []HookEntry{{Type: "command", Command: "existing"}}}},
			},
			newHooks:   map[string][]HookMatcher{},
			wantCounts: map[string]int{"PreToolUse": 1},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			cs := &ClaudeSettings{Hooks: tt.initial}
			cs.MergeHooks(tt.newHooks)

			for event, wantCount := range tt.wantCounts {
				if gotCount := len(cs.Hooks[event]); gotCount != wantCount {
					t.Errorf("Hook %q count = %d, want %d", event, gotCount, wantCount)
				}
			}
		})
	}
}

// TestLoadSettings tests various LoadSettings scenarios
func TestLoadSettings(t *testing.T) {
	tempDir := t.TempDir()

	tests := []struct {
		name      string
		setupFile func() string
		wantErr   bool
		validate  func(*ClaudeSettings) error
	}{
		{
			name: "valid settings file",
			setupFile: func() string {
				path := filepath.Join(tempDir, "valid.json")
				data := `{"hooks": {"SessionStart": [{"hooks": [{"type": "command", "command": "test"}]}]}}`
				os.WriteFile(path, []byte(data), 0644)
				return path
			},
			wantErr: false,
			validate: func(cs *ClaudeSettings) error {
				if len(cs.Hooks) != 1 {
					return fmt.Errorf("expected 1 hook type")
				}
				return nil
			},
		},
		{
			name: "non-existent file returns empty settings",
			setupFile: func() string {
				return filepath.Join(tempDir, "non-existent.json")
			},
			wantErr: false,
			validate: func(cs *ClaudeSettings) error {
				if len(cs.Hooks) != 0 || len(cs.Extra) != 0 {
					return fmt.Errorf("expected empty settings")
				}
				return nil
			},
		},
		{
			name: "file with invalid JSON",
			setupFile: func() string {
				path := filepath.Join(tempDir, "invalid.json")
				os.WriteFile(path, []byte("{invalid json}"), 0644)
				return path
			},
			wantErr: true,
		},
		{
			name: "file with no hooks or extra",
			setupFile: func() string {
				path := filepath.Join(tempDir, "empty.json")
				os.WriteFile(path, []byte("{}"), 0644)
				return path
			},
			wantErr: false,
			validate: func(cs *ClaudeSettings) error {
				if cs.Hooks == nil || cs.Extra == nil {
					return fmt.Errorf("maps should be initialized")
				}
				return nil
			},
		},
		{
			name: "unreadable file",
			setupFile: func() string {
				path := filepath.Join(tempDir, "unreadable.json")
				os.WriteFile(path, []byte("{}"), 0000)
				return path
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			path := tt.setupFile()
			settings, err := LoadSettings(path)

			if (err != nil) != tt.wantErr {
				t.Errorf("LoadSettings() error = %v, wantErr %v", err, tt.wantErr)
			}

			if !tt.wantErr && tt.validate != nil {
				if err := tt.validate(settings); err != nil {
					t.Errorf("Validation failed: %v", err)
				}
			}
		})
	}
}

// TestSaveToFile tests various SaveToFile scenarios
func TestSaveToFile(t *testing.T) {
	tempDir := t.TempDir()

	tests := []struct {
		name     string
		settings *ClaudeSettings
		path     string
		wantErr  bool
	}{
		{
			name: "save to new file",
			settings: &ClaudeSettings{
				Hooks: map[string][]HookMatcher{
					"SessionStart": {{Hooks: []HookEntry{{Type: "command", Command: "test"}}}},
				},
				Extra: map[string]interface{}{"key": "value"},
			},
			path:    filepath.Join(tempDir, "new.json"),
			wantErr: false,
		},
		{
			name: "save to nested directory (creates dirs)",
			settings: &ClaudeSettings{
				Hooks: map[string][]HookMatcher{},
				Extra: map[string]interface{}{},
			},
			path:    filepath.Join(tempDir, "nested", "dir", "file.json"),
			wantErr: false,
		},
		{
			name:     "save to invalid path",
			settings: &ClaudeSettings{},
			path:     "/root/cannot-write-here.json",
			wantErr:  true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.settings.SaveToFile(tt.path)

			if (err != nil) != tt.wantErr {
				t.Errorf("SaveToFile() error = %v, wantErr %v", err, tt.wantErr)
			}

			if !tt.wantErr {
				// Verify file exists and is valid JSON
				data, err := os.ReadFile(tt.path)
				if err != nil {
					t.Errorf("Failed to read saved file: %v", err)
				}

				var result map[string]interface{}
				if err := json.Unmarshal(data, &result); err != nil {
					t.Errorf("Saved file contains invalid JSON: %v", err)
				}
			}
		})
	}
}

// TestCreateMergedSettings tests error cases
func TestCreateMergedSettings(t *testing.T) {
	tempDir := t.TempDir()

	tests := []struct {
		name      string
		setupFile func() string
		newHooks  map[string][]HookMatcher
		wantErr   bool
	}{
		{
			name: "merge with non-existent file",
			setupFile: func() string {
				return filepath.Join(tempDir, "non-existent.json")
			},
			newHooks: map[string][]HookMatcher{
				"SessionStart": {{Hooks: []HookEntry{{Type: "command", Command: "test"}}}},
			},
			wantErr: false,
		},
		{
			name: "merge with invalid JSON file",
			setupFile: func() string {
				path := filepath.Join(tempDir, "invalid.json")
				os.WriteFile(path, []byte("{invalid}"), 0644)
				return path
			},
			newHooks: map[string][]HookMatcher{},
			wantErr:  true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			path := tt.setupFile()
			_, err := CreateMergedSettings(path, tt.newHooks)

			if (err != nil) != tt.wantErr {
				t.Errorf("CreateMergedSettings() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}
