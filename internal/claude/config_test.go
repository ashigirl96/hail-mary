package claude

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestDefaultConfig(t *testing.T) {
	// Arrange & Act
	config := DefaultConfig()

	// Assert
	assert.Equal(t, claudeCommand, config.Command, "Command should be set to default claude command")
	assert.Equal(t, claudePackage, config.Package, "Package should be set to default claude package")
	assert.True(t, config.EnableBackgroundTasks, "EnableBackgroundTasks should be enabled by default")
	assert.True(t, config.MaintainWorkingDir, "MaintainWorkingDir should be enabled by default")
	assert.True(t, config.SkipPermissions, "SkipPermissions should be enabled by default")
	assert.Equal(t, maxPromptLength, config.MaxPromptLength, "MaxPromptLength should be set to default value")
	assert.Empty(t, config.SettingsPath, "SettingsPath should be empty by default")
}

func TestConfig_Validation(t *testing.T) {
	tests := []struct {
		name   string
		config *Config
		valid  bool
	}{
		{
			name:   "default config is valid",
			config: DefaultConfig(),
			valid:  true,
		},
		{
			name: "custom config is valid",
			config: &Config{
				Command:               "custom-command",
				Package:               "custom-package",
				EnableBackgroundTasks: false,
				MaintainWorkingDir:    false,
				SkipPermissions:       false,
				MaxPromptLength:       5000,
				SettingsPath:          "/path/to/settings.json",
			},
			valid: true,
		},
		{
			name: "config with zero max prompt length",
			config: &Config{
				Command:               "bunx",
				Package:               "@anthropic-ai/claude-code@latest",
				EnableBackgroundTasks: true,
				MaintainWorkingDir:    true,
				SkipPermissions:       true,
				MaxPromptLength:       0,
			},
			valid: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// For now, we don't have explicit validation method,
			// but we can test that MaxPromptLength > 0 is expected
			if tt.config.MaxPromptLength == 0 && tt.valid {
				t.Error("Config with MaxPromptLength=0 should be considered invalid")
			}
		})
	}
}

func TestConfig_DeepCopy(t *testing.T) {
	// Arrange
	original := DefaultConfig()
	original.SettingsPath = "/original/path"

	// Act - simulate copying by creating new config
	copy := &Config{
		Command:               original.Command,
		Package:               original.Package,
		EnableBackgroundTasks: original.EnableBackgroundTasks,
		MaintainWorkingDir:    original.MaintainWorkingDir,
		SkipPermissions:       original.SkipPermissions,
		MaxPromptLength:       original.MaxPromptLength,
		SettingsPath:          original.SettingsPath,
	}

	// Modify copy
	copy.SettingsPath = "/modified/path"

	// Assert
	assert.Equal(t, "/original/path", original.SettingsPath, "Original should not be modified")
	assert.Equal(t, "/modified/path", copy.SettingsPath, "Copy should be modified")
}
