package settings

import (
	"encoding/json"
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestNewClaudeSettings(t *testing.T) {
	// Act
	settings := NewClaudeSettings()

	// Assert
	require.NotNil(t, settings, "NewClaudeSettings should not return nil")
	assert.NotNil(t, settings.Hooks, "Hooks should be initialized")
	assert.NotNil(t, settings.Extra, "Extra should be initialized")
	assert.Empty(t, settings.Hooks, "Hooks should be empty initially")
	assert.Empty(t, settings.Extra, "Extra should be empty initially")
}

func TestClaudeSettings_AddHook(t *testing.T) {
	// Arrange
	settings := NewClaudeSettings()
	hookEntry := HookEntry{
		Type:    "command",
		Command: "echo test",
	}

	// Act
	settings.AddHook("TestEvent", "Write", hookEntry)

	// Assert
	require.Contains(t, settings.Hooks, "TestEvent", "TestEvent should be added to hooks")
	require.Len(t, settings.Hooks["TestEvent"], 1, "TestEvent should have one matcher")
	
	matcher := settings.Hooks["TestEvent"][0]
	assert.Equal(t, "Write", matcher.Matcher)
	require.Len(t, matcher.Hooks, 1, "Matcher should have one hook entry")
	assert.Equal(t, hookEntry, matcher.Hooks[0])
}

func TestClaudeSettings_AddHook_MultipleMatchers(t *testing.T) {
	// Arrange
	settings := NewClaudeSettings()
	hookEntry1 := HookEntry{Type: "command", Command: "echo 1"}
	hookEntry2 := HookEntry{Type: "command", Command: "echo 2"}

	// Act
	settings.AddHook("TestEvent", "Write", hookEntry1)
	settings.AddHook("TestEvent", "Read", hookEntry2)

	// Assert
	require.Contains(t, settings.Hooks, "TestEvent")
	require.Len(t, settings.Hooks["TestEvent"], 2, "TestEvent should have two matchers")
	
	// Find matchers (order not guaranteed)
	var writeMatch, readMatch *HookMatcher
	for i := range settings.Hooks["TestEvent"] {
		if settings.Hooks["TestEvent"][i].Matcher == "Write" {
			writeMatch = &settings.Hooks["TestEvent"][i]
		} else if settings.Hooks["TestEvent"][i].Matcher == "Read" {
			readMatch = &settings.Hooks["TestEvent"][i]
		}
	}
	
	require.NotNil(t, writeMatch, "Write matcher should exist")
	require.NotNil(t, readMatch, "Read matcher should exist")
	assert.Equal(t, hookEntry1, writeMatch.Hooks[0])
	assert.Equal(t, hookEntry2, readMatch.Hooks[0])
}

func TestClaudeSettings_AddHook_SameMatcher(t *testing.T) {
	// Arrange
	settings := NewClaudeSettings()
	hookEntry1 := HookEntry{Type: "command", Command: "echo 1"}
	hookEntry2 := HookEntry{Type: "command", Command: "echo 2"}

	// Act
	settings.AddHook("TestEvent", "Write", hookEntry1)
	settings.AddHook("TestEvent", "Write", hookEntry2)

	// Assert
	require.Contains(t, settings.Hooks, "TestEvent")
	require.Len(t, settings.Hooks["TestEvent"], 1, "TestEvent should have one matcher")
	
	matcher := settings.Hooks["TestEvent"][0]
	assert.Equal(t, "Write", matcher.Matcher)
	require.Len(t, matcher.Hooks, 2, "Matcher should have two hook entries")
	assert.Equal(t, hookEntry1, matcher.Hooks[0])
	assert.Equal(t, hookEntry2, matcher.Hooks[1])
}

func TestClaudeSettings_MergeWith_EmptySource(t *testing.T) {
	// Arrange
	target := NewClaudeSettings()
	target.AddHook("ExistingEvent", "Write", HookEntry{Type: "command", Command: "existing"})
	target.Extra["existing"] = "value"
	
	source := NewClaudeSettings()

	// Act
	err := target.MergeWith(source)

	// Assert
	require.NoError(t, err, "Merge should succeed")
	assert.Len(t, target.Hooks, 1, "Target should retain existing hook")
	assert.Equal(t, "value", target.Extra["existing"], "Target should retain existing extra data")
}

func TestClaudeSettings_MergeWith_NewHooks(t *testing.T) {
	// Arrange
	target := NewClaudeSettings()
	target.AddHook("ExistingEvent", "Write", HookEntry{Type: "command", Command: "existing"})
	
	source := NewClaudeSettings()
	source.AddHook("NewEvent", "Read", HookEntry{Type: "command", Command: "new"})

	// Act
	err := target.MergeWith(source)

	// Assert
	require.NoError(t, err, "Merge should succeed")
	assert.Len(t, target.Hooks, 2, "Target should have both hooks")
	assert.Contains(t, target.Hooks, "ExistingEvent")
	assert.Contains(t, target.Hooks, "NewEvent")
}

func TestClaudeSettings_MergeWith_ConflictingHooks(t *testing.T) {
	// Arrange
	target := NewClaudeSettings()
	target.AddHook("SameEvent", "Write", HookEntry{Type: "command", Command: "target"})
	
	source := NewClaudeSettings()
	source.AddHook("SameEvent", "Write", HookEntry{Type: "command", Command: "source"})

	// Act
	err := target.MergeWith(source)

	// Assert
	require.NoError(t, err, "Merge should succeed")
	assert.Len(t, target.Hooks, 1, "Target should have one event")
	assert.Len(t, target.Hooks["SameEvent"], 1, "Event should have one matcher")
	assert.Len(t, target.Hooks["SameEvent"][0].Hooks, 2, "Matcher should have both hook entries")
	
	// Verify both commands are present
	commands := []string{
		target.Hooks["SameEvent"][0].Hooks[0].Command,
		target.Hooks["SameEvent"][0].Hooks[1].Command,
	}
	assert.Contains(t, commands, "target")
	assert.Contains(t, commands, "source")
}

func TestClaudeSettings_MergeWith_ExtraData(t *testing.T) {
	// Arrange
	target := NewClaudeSettings()
	target.Extra["existing"] = "target_value"
	target.Extra["shared"] = "target_shared"
	
	source := NewClaudeSettings()
	source.Extra["new"] = "source_value"
	source.Extra["shared"] = "source_shared"

	// Act
	err := target.MergeWith(source)

	// Assert
	require.NoError(t, err, "Merge should succeed")
	assert.Equal(t, "target_value", target.Extra["existing"], "Existing value should be preserved")
	assert.Equal(t, "source_value", target.Extra["new"], "New value should be added")
	assert.Equal(t, "source_shared", target.Extra["shared"], "Source should override target for shared keys")
}

func TestClaudeSettings_LoadFromFile_Success(t *testing.T) {
	// Arrange
	tempDir := t.TempDir()
	settingsPath := filepath.Join(tempDir, "settings.json")
	
	testSettings := &ClaudeSettings{
		Hooks: map[string][]HookMatcher{
			"TestEvent": {
				{
					Matcher: "Write",
					Hooks: []HookEntry{
						{Type: "command", Command: "echo test"},
					},
				},
			},
		},
		Extra: map[string]interface{}{
			"test_field": "test_value",
		},
	}
	
	data, err := json.MarshalIndent(testSettings, "", "  ")
	require.NoError(t, err, "Setup should succeed")
	
	err = os.WriteFile(settingsPath, data, 0644)
	require.NoError(t, err, "Setup should succeed")

	// Act
	loadedSettings, err := LoadFromFile(settingsPath)

	// Assert
	require.NoError(t, err, "LoadFromFile should succeed")
	require.NotNil(t, loadedSettings, "LoadFromFile should return non-nil settings")
	
	assert.Equal(t, testSettings.Hooks, loadedSettings.Hooks)
	assert.Equal(t, testSettings.Extra, loadedSettings.Extra)
}

func TestClaudeSettings_LoadFromFile_FileNotFound(t *testing.T) {
	// Arrange
	nonExistentPath := "/path/that/does/not/exist/settings.json"

	// Act
	settings, err := LoadFromFile(nonExistentPath)

	// Assert
	require.Error(t, err, "LoadFromFile should return error for non-existent file")
	assert.Nil(t, settings, "LoadFromFile should return nil on error")
	assert.Contains(t, err.Error(), "failed to read settings file", "Error should indicate file read failure")
}

func TestClaudeSettings_LoadFromFile_InvalidJSON(t *testing.T) {
	// Arrange
	tempDir := t.TempDir()
	settingsPath := filepath.Join(tempDir, "invalid.json")
	
	err := os.WriteFile(settingsPath, []byte("invalid json content"), 0644)
	require.NoError(t, err, "Setup should succeed")

	// Act
	settings, err := LoadFromFile(settingsPath)

	// Assert
	require.Error(t, err, "LoadFromFile should return error for invalid JSON")
	assert.Nil(t, settings, "LoadFromFile should return nil on error")
	assert.Contains(t, err.Error(), "failed to unmarshal settings", "Error should indicate JSON parse failure")
}

func TestClaudeSettings_SaveToFile_Success(t *testing.T) {
	// Arrange
	tempDir := t.TempDir()
	settingsPath := filepath.Join(tempDir, "save_test.json")
	
	settings := NewClaudeSettings()
	settings.AddHook("SaveEvent", "Write", HookEntry{Type: "command", Command: "echo save"})
	settings.Extra["save_field"] = "save_value"

	// Act
	err := settings.SaveToFile(settingsPath)

	// Assert
	require.NoError(t, err, "SaveToFile should succeed")
	
	// Verify file exists and contains correct data
	data, err := os.ReadFile(settingsPath)
	require.NoError(t, err, "File should be readable")
	
	var loaded ClaudeSettings
	err = json.Unmarshal(data, &loaded)
	require.NoError(t, err, "Saved file should contain valid JSON")
	
	assert.Equal(t, settings.Hooks, loaded.Hooks)
	assert.Equal(t, settings.Extra, loaded.Extra)
}

func TestClaudeSettings_SaveToFile_CreateDirectory(t *testing.T) {
	// Arrange
	tempDir := t.TempDir()
	nestedPath := filepath.Join(tempDir, "nested", "directory", "settings.json")
	
	settings := NewClaudeSettings()
	settings.Extra["test"] = "value"

	// Act
	err := settings.SaveToFile(nestedPath)

	// Assert
	require.NoError(t, err, "SaveToFile should create nested directories")
	
	// Verify file exists
	_, err = os.Stat(nestedPath)
	assert.NoError(t, err, "File should exist in nested directory")
}

func TestClaudeSettings_JSONMarshaling(t *testing.T) {
	// Arrange
	original := &ClaudeSettings{
		Hooks: map[string][]HookMatcher{
			"TestEvent": {
				{
					Matcher: "Write",
					Hooks: []HookEntry{
						{
							Type:    "command",
							Command: "echo test",
						},
					},
				},
			},
		},
		Extra: map[string]interface{}{
			"string_field": "test_value",
			"number_field": 42.0, // Use float64 to match JSON unmarshaling
			"bool_field":   true,
			"nested": map[string]interface{}{
				"inner": "value",
			},
		},
	}

	// Act - Marshal
	data, err := json.Marshal(original)
	require.NoError(t, err, "Marshal should succeed")

	// Act - Unmarshal
	var decoded ClaudeSettings
	err = json.Unmarshal(data, &decoded)
	require.NoError(t, err, "Unmarshal should succeed")

	// Assert
	assert.Equal(t, original.Hooks, decoded.Hooks)
	assert.Equal(t, original.Extra, decoded.Extra)
}

func TestClaudeSettings_ComplexMergeScenario(t *testing.T) {
	// Arrange - Create target with existing hooks and data
	target := NewClaudeSettings()
	target.AddHook("SessionStart", "Write", HookEntry{Type: "command", Command: "existing_start"})
	target.AddHook("SessionEnd", "Read", HookEntry{Type: "command", Command: "existing_end"})
	target.Extra["app_name"] = "target_app"
	target.Extra["version"] = "1.0.0"
	
	// Arrange - Create source with overlapping and new hooks
	source := NewClaudeSettings()
	source.AddHook("SessionStart", "Write", HookEntry{Type: "command", Command: "new_start"})
	source.AddHook("SessionStart", "Read", HookEntry{Type: "command", Command: "start_read"})
	source.AddHook("UserPrompt", "Write", HookEntry{Type: "command", Command: "prompt_hook"})
	source.Extra["version"] = "2.0.0"
	source.Extra["debug"] = true

	// Act
	err := target.MergeWith(source)

	// Assert
	require.NoError(t, err, "Complex merge should succeed")
	
	// Verify hooks
	assert.Len(t, target.Hooks, 3, "Should have 3 events")
	assert.Contains(t, target.Hooks, "SessionStart")
	assert.Contains(t, target.Hooks, "SessionEnd")
	assert.Contains(t, target.Hooks, "UserPrompt")
	
	// Verify SessionStart has multiple matchers
	sessionStartMatchers := target.Hooks["SessionStart"]
	assert.Len(t, sessionStartMatchers, 2, "SessionStart should have 2 matchers")
	
	// Find Write matcher for SessionStart
	var writeCommands []string
	for _, matcher := range sessionStartMatchers {
		if matcher.Matcher == "Write" {
			for _, hook := range matcher.Hooks {
				writeCommands = append(writeCommands, hook.Command)
			}
		}
	}
	assert.Contains(t, writeCommands, "existing_start")
	assert.Contains(t, writeCommands, "new_start")
	
	// Verify extra data merge
	assert.Equal(t, "target_app", target.Extra["app_name"])
	assert.Equal(t, "2.0.0", target.Extra["version"]) // Source should override
	assert.Equal(t, true, target.Extra["debug"])
}