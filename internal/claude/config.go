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
	// continueFlag continues the most recent session
	continueFlag = "--continue"

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
	// PermissionMode is the permission mode to use for the session
	// Valid options: "acceptEdits", "bypassPermissions", "default", "plan"
	PermissionMode string
	// AppendSystemPrompt is the system prompt to append to the default system prompt
	AppendSystemPrompt string
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
	if c.PermissionMode != "" {
		args = append(args, permissionModeFlag, c.PermissionMode)
	}
	if c.AppendSystemPrompt != "" {
		args = append(args, appendSystemPromptFlag, c.AppendSystemPrompt)
	}
	args = append(args, additionalArgs...)
	return args
}
