// Package claude provides a Go wrapper for the Claude CLI tool.
// It offers both interactive and programmatic interfaces for executing
// Claude commands with session management and configuration options.
package claude

import (
	"fmt"
	"log/slog"
	"os"
	"os/exec"
	"strings"
)

// SessionInfo contains information about a Claude session
type SessionInfo struct {
	ID       string  `json:"session_id"`
	Result   string  `json:"result"`
	CostUSD  float64 `json:"cost_usd"`
	Duration string  `json:"duration,omitempty"`
	Turns    int     `json:"turns,omitempty"`
}

// Executor defines the interface for Claude CLI operations.
// This interface allows for easy mocking in tests and alternative implementations.
type Executor interface {
	// Execute launches Claude CLI with the given options
	Execute(opts ExecuteOptions) error
	// ExecuteWithSession resumes a specific session with optional additional options
	ExecuteWithSession(sessionID string, opts ExecuteOptions) error
}

// ExecutorImpl handles Claude CLI execution with configurable options.
// It implements the Executor interface and provides a wrapper around
// the Claude CLI command-line tool.
type ExecutorImpl struct {
	config *Config
}

// SetSettingsPath sets the path to a settings file to use with --settings flag
func (e *ExecutorImpl) SetSettingsPath(path string) {
	e.config.SettingsPath = path
}

// ensure ExecutorImpl implements Executor interface
var _ Executor = (*ExecutorImpl)(nil)

// validatePrompt validates the input prompt for security and usability.
// It ensures the prompt is not empty and doesn't exceed the configured length limit.
func (e *ExecutorImpl) validatePrompt(prompt string) error {
	trimmed := strings.TrimSpace(prompt)
	if trimmed == "" {
		return fmt.Errorf("prompt validation failed: prompt cannot be empty or contain only whitespace")
	}
	if len(prompt) > e.config.MaxPromptLength {
		return fmt.Errorf("prompt validation failed: prompt length (%d) exceeds maximum allowed length (%d)",
			len(prompt), e.config.MaxPromptLength)
	}
	return nil
}

// validateSessionID validates the session ID format.
// Session IDs are expected to be between 8 and 100 characters,
// which accommodates various ID formats including UUIDs.
func validateSessionID(sessionID string) error {
	trimmed := strings.TrimSpace(sessionID)
	if trimmed == "" {
		return fmt.Errorf("session ID validation failed: session ID cannot be empty or contain only whitespace")
	}
	if len(trimmed) < minSessionIDLength || len(trimmed) > maxSessionIDLength {
		return fmt.Errorf("session ID validation failed: session ID length (%d) must be between %d and %d characters",
			len(trimmed), minSessionIDLength, maxSessionIDLength)
	}
	return nil
}

// validatePermissionMode validates the permission mode.
// Valid modes are: "acceptEdits", "bypassPermissions", "default", "plan"
func validatePermissionMode(mode string) error {
	if mode == "" {
		return nil // Empty mode is valid (uses default)
	}
	validModes := []string{"acceptEdits", "bypassPermissions", "default", "plan"}
	trimmed := strings.TrimSpace(mode)

	for _, validMode := range validModes {
		if trimmed == validMode {
			return nil
		}
	}

	return fmt.Errorf("permission mode validation failed: invalid mode '%s', valid options are: %s",
		trimmed, strings.Join(validModes, ", "))
}

// NewExecutor creates a new Claude executor with default configuration.
// This is the recommended way to create an executor for most use cases.
func NewExecutor() *ExecutorImpl {
	return &ExecutorImpl{
		config: DefaultConfig(),
	}
}

// NewExecutorWithConfig creates a new Claude executor with custom configuration.
// Use this when you need to customize the Claude CLI behavior, such as:
// - Using a different command or package version
// - Disabling background tasks or working directory maintenance
// - Changing validation limits
func NewExecutorWithConfig(config *Config) *ExecutorImpl {
	if config == nil {
		config = DefaultConfig()
	}
	return &ExecutorImpl{
		config: config,
	}
}

// Execute launches Claude CLI with the given options.
// Session management is automatically handled by the Claude Code hook system.
func (e *ExecutorImpl) Execute(opts ExecuteOptions) error {
	// Validate options
	if opts.Prompt != "" {
		if err := e.validatePrompt(opts.Prompt); err != nil {
			return fmt.Errorf("execute: %w", err)
		}
	}
	if err := validatePermissionMode(opts.Mode); err != nil {
		return fmt.Errorf("execute: %w", err)
	}

	// Build command arguments
	args := e.config.BuildArgs()
	args = opts.BuildArgs(args)

	// Create command
	cmd := exec.Command(e.config.Command, args...)

	// Set environment variables
	cmd.Env = e.config.SetEnvironment(os.Environ())

	// Connect stdin, stdout, and stderr to the current terminal
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	// Log command details for debugging
	logger := slog.Default()
	logger.Debug("Executing Claude command",
		"command", e.config.Command,
		"args", args,
		"mode", opts.Mode,
		"has_system_prompt", opts.SystemPrompt != "")

	// Run the command and wait for it to complete
	if err := cmd.Run(); err != nil {
		return fmt.Errorf("execute: failed to run Claude CLI: %w", err)
	}
	return nil
}

// ExecuteWithSession resumes a specific session with optional additional options.
// Session management is automatically handled by the Claude Code hook system.
func (e *ExecutorImpl) ExecuteWithSession(sessionID string, opts ExecuteOptions) error {
	// Validate session ID
	if err := validateSessionID(sessionID); err != nil {
		return fmt.Errorf("execute with session: %w", err)
	}

	// Validate options
	if err := validatePermissionMode(opts.Mode); err != nil {
		return fmt.Errorf("execute with session: %w", err)
	}

	// Note: When resuming a session, we ignore opts.Prompt as the conversation already exists

	// Build command arguments with --resume flag
	args := e.config.BuildArgs(resumeFlag, sessionID)
	// Add mode and system prompt if specified
	if opts.Mode != "" {
		args = append(args, permissionModeFlag, opts.Mode)
	}
	if opts.SystemPrompt != "" {
		args = append(args, appendSystemPromptFlag, opts.SystemPrompt)
	}

	// Create command
	cmd := exec.Command(e.config.Command, args...)

	// Set environment variables
	cmd.Env = e.config.SetEnvironment(os.Environ())

	// Connect stdin, stdout, and stderr to the current terminal
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	// Log command details for debugging
	logger := slog.Default()
	logger.Debug("Resuming Claude session",
		"command", e.config.Command,
		"args", args,
		"session_id", sessionID,
		"mode", opts.Mode)

	// Run the command and wait for it to complete
	if err := cmd.Run(); err != nil {
		return fmt.Errorf("execute with session %s: failed to run Claude CLI: %w", sessionID, err)
	}
	return nil
}
