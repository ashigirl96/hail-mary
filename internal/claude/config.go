package claude

import (
	"fmt"
)

// Constants for Claude CLI execution
const (
	// claudeCommand is the command to execute Claude CLI
	claudeCommand = "bunx"
	// claudePackage is the Claude Code package identifier
	claudePackage = "@anthropic-ai/claude-code@latest"
	// dangerousFlag allows Claude to execute without permissions prompt
	dangerousFlag = "--dangerously-skip-permissions"
	// permissionModeFlag sets the permission mode
	permissionModeFlag = "--permission-mode"
	// appendSystemPromptFlag appends a system prompt
	appendSystemPromptFlag = "--append-system-prompt"
	// resumeFlag resumes a previous session
	resumeFlag = "--resume"

	// Environment variable names
	envBackgroundTasks = "ENABLE_BACKGROUND_TASKS"
	envMaintainWorkDir = "CLAUDE_BASH_MAINTAIN_PROJECT_WORKING_DIR"

	// Validation constants
	maxPromptLength    = 10000
	minSessionIDLength = 8
	maxSessionIDLength = 100
)

// Config holds configuration options for the Claude executor
type Config struct {
	// Command is the command to execute (default: "bunx")
	Command string
	// Package is the Claude package identifier (default: "@anthropic-ai/claude-code@latest")
	Package string
	// EnableBackgroundTasks enables background task execution in Claude CLI.
	// When enabled, Claude can perform background operations like file watching.
	EnableBackgroundTasks bool
	// MaintainWorkingDir maintains the project working directory across Claude operations.
	// This ensures Claude commands execute in the correct project context.
	MaintainWorkingDir bool
	// SkipPermissions skips the permissions prompt
	SkipPermissions bool
	// MaxPromptLength is the maximum allowed prompt length
	MaxPromptLength int
	// SettingsPath is the path to a settings file to use with --settings flag
	SettingsPath string
}

// DefaultConfig returns the default configuration
func DefaultConfig() *Config {
	return &Config{
		Command:               claudeCommand,
		Package:               claudePackage,
		EnableBackgroundTasks: true,
		MaintainWorkingDir:    true,
		SkipPermissions:       true,
		MaxPromptLength:       maxPromptLength,
	}
}

// SetEnvironment applies configuration to environment variables
func (c *Config) SetEnvironment(env []string) []string {
	if c.EnableBackgroundTasks {
		env = append(env, fmt.Sprintf("%s=1", envBackgroundTasks))
	}
	if c.MaintainWorkingDir {
		env = append(env, fmt.Sprintf("%s=1", envMaintainWorkDir))
	}
	return env
}

// BuildArgs constructs command line arguments from configuration
func (c *Config) BuildArgs(additionalArgs ...string) []string {
	args := []string{c.Package}
	if c.SkipPermissions {
		args = append(args, dangerousFlag)
	}
	if c.SettingsPath != "" {
		args = append(args, "--settings", c.SettingsPath)
	}
	args = append(args, additionalArgs...)
	return args
}

// NewConfigWithDefaults creates a Config with default values, applying any overrides
func NewConfigWithDefaults(overrides Config) *Config {
	// Start with default values
	config := &Config{
		Command:               claudeCommand,
		Package:               claudePackage,
		EnableBackgroundTasks: true,
		MaintainWorkingDir:    true,
		SkipPermissions:       true,
		MaxPromptLength:       maxPromptLength,
	}

	// Apply overrides
	if overrides.Command != "" {
		config.Command = overrides.Command
	}
	if overrides.Package != "" {
		config.Package = overrides.Package
	}
	if overrides.SettingsPath != "" {
		config.SettingsPath = overrides.SettingsPath
	}
	if overrides.MaxPromptLength > 0 {
		config.MaxPromptLength = overrides.MaxPromptLength
	}
	// Note: Boolean fields retain their default values (true)
	// unless explicitly overridden. This is a limitation of Go's zero values.
	// In production code, you might want to use pointers to distinguish
	// between "not set" and "explicitly set to false"

	return config
}
