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
	// ExecuteInteractive launches Claude CLI in interactive mode
	ExecuteInteractive(prompt string) error
	// ExecuteInteractiveWithMode launches Claude CLI in interactive mode with a specific permission mode
	ExecuteInteractiveWithMode(prompt, mode string) error
	// ExecuteInteractiveWithSystemPrompt launches Claude CLI in interactive mode with a system prompt
	ExecuteInteractiveWithSystemPrompt(prompt, systemPrompt string) error
	// ExecuteInteractiveWithModeAndSystemPrompt launches Claude CLI in interactive mode with permission mode and system prompt
	ExecuteInteractiveWithModeAndSystemPrompt(prompt, mode, systemPrompt string) error
	// ExecuteInteractiveWithSession launches interactive mode with a specific session
	ExecuteInteractiveWithSession(sessionID string) error
	// ExecuteInteractiveWithSessionAndMode launches interactive mode with a specific session and permission mode
	ExecuteInteractiveWithSessionAndMode(sessionID, mode string) error
	// ExecuteAndContinueInteractive executes a prompt and then continues in interactive mode
	ExecuteAndContinueInteractive(prompt string) (*SessionInfo, error)
	// ExecuteAndContinueInteractiveWithMode executes a prompt with mode and then continues in interactive mode
	ExecuteAndContinueInteractiveWithMode(prompt, mode string) (*SessionInfo, error)
	// ExecuteAndContinueInteractiveWithSystemPrompt executes a prompt with system prompt and then continues in interactive mode
	ExecuteAndContinueInteractiveWithSystemPrompt(prompt, systemPrompt string) (*SessionInfo, error)
	// ExecuteAndContinueInteractiveWithModeAndSystemPrompt executes a prompt with mode and system prompt and then continues in interactive mode
	ExecuteAndContinueInteractiveWithModeAndSystemPrompt(prompt, mode, systemPrompt string) (*SessionInfo, error)
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
	validModes := []string{"acceptEdits", "bypassPermissions", "default", "plan"}
	trimmed := strings.TrimSpace(mode)
	if trimmed == "" {
		return fmt.Errorf("permission mode validation failed: mode cannot be empty or contain only whitespace")
	}

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

// buildCommand creates an exec.Cmd with common configuration.
// It applies all configuration options and environment variables
// needed for Claude CLI execution.
func (e *ExecutorImpl) buildCommand(args ...string) *exec.Cmd {
	// Prepare command arguments using config
	cmdArgs := e.config.BuildArgs(args...)

	// Create command
	cmd := exec.Command(e.config.Command, cmdArgs...)

	// Set environment variables using config
	env := e.config.SetEnvironment(os.Environ())
	cmd.Env = env

	// Log command details for debugging
	logger := slog.Default()
	logger.Debug("Building Claude command",
		"command", e.config.Command,
		"args", cmdArgs,
		"settings_path", e.config.SettingsPath,
		"enable_background_tasks", e.config.EnableBackgroundTasks,
		"maintain_working_dir", e.config.MaintainWorkingDir)

	return cmd
}

// ExecuteInteractive launches Claude CLI in interactive mode with an initial prompt.
// Session management is automatically handled by the Claude Code hook system.
func (e *ExecutorImpl) ExecuteInteractive(prompt string) error {
	return e.ExecuteInteractiveWithMode(prompt, "")
}

// ExecuteInteractiveWithMode launches Claude CLI in interactive mode with an initial prompt and permission mode.
// Session management is automatically handled by the Claude Code hook system.
func (e *ExecutorImpl) ExecuteInteractiveWithMode(prompt, mode string) error {
	return e.ExecuteInteractiveWithModeAndSystemPrompt(prompt, mode, "")
}

// ExecuteInteractiveWithSystemPrompt launches Claude CLI in interactive mode with an initial prompt and system prompt.
// Session management is automatically handled by the Claude Code hook system.
func (e *ExecutorImpl) ExecuteInteractiveWithSystemPrompt(prompt, systemPrompt string) error {
	return e.ExecuteInteractiveWithModeAndSystemPrompt(prompt, "", systemPrompt)
}

// ExecuteInteractiveWithModeAndSystemPrompt launches Claude CLI in interactive mode with an initial prompt, permission mode, and system prompt.
// Session management is automatically handled by the Claude Code hook system.
func (e *ExecutorImpl) ExecuteInteractiveWithModeAndSystemPrompt(prompt, mode, systemPrompt string) error {
	if err := e.validatePrompt(prompt); err != nil {
		return fmt.Errorf("execute interactive with mode and system prompt: %w", err)
	}

	// Validate permission mode if provided
	if mode != "" {
		if err := validatePermissionMode(mode); err != nil {
			return fmt.Errorf("execute interactive with mode and system prompt: %w", err)
		}
	}

	// Set the permission mode and system prompt temporarily if provided
	originalMode := e.config.PermissionMode
	originalSystemPrompt := e.config.AppendSystemPrompt
	if mode != "" {
		e.config.PermissionMode = mode
		defer func() { e.config.PermissionMode = originalMode }()
	}
	if systemPrompt != "" {
		e.config.AppendSystemPrompt = systemPrompt
		defer func() { e.config.AppendSystemPrompt = originalSystemPrompt }()
	}

	// Create command for interactive Claude shell with initial prompt
	cmd := e.buildCommand(prompt)

	// Connect stdin, stdout, and stderr to the current terminal
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	// Run the command and wait for it to complete
	if err := cmd.Run(); err != nil {
		return fmt.Errorf("execute interactive with mode and system prompt: failed to run Claude CLI: %w", err)
	}
	return nil
}

// ExecuteInteractiveWithSession launches Claude CLI in interactive mode with a specific session ID.
// This combines the benefits of interactive mode with the ability to resume a specific
// previous session, allowing users to continue a conversation interactively.
func (e *ExecutorImpl) ExecuteInteractiveWithSession(sessionID string) error {
	return e.ExecuteInteractiveWithSessionAndMode(sessionID, "")
}

// ExecuteInteractiveWithSessionAndMode launches Claude CLI in interactive mode with a specific session ID and permission mode.
// This combines the benefits of interactive mode with the ability to resume a specific
// previous session, allowing users to continue a conversation interactively.
func (e *ExecutorImpl) ExecuteInteractiveWithSessionAndMode(sessionID, mode string) error {
	if err := validateSessionID(sessionID); err != nil {
		return fmt.Errorf("execute interactive with session and mode: %w", err)
	}

	// Validate permission mode if provided
	if mode != "" {
		if err := validatePermissionMode(mode); err != nil {
			return fmt.Errorf("execute interactive with session and mode: %w", err)
		}
	}

	// Set the permission mode temporarily if provided
	originalMode := e.config.PermissionMode
	if mode != "" {
		e.config.PermissionMode = mode
		defer func() { e.config.PermissionMode = originalMode }()
	}

	// Create command for interactive Claude shell with --resume flag
	cmd := e.buildCommand(resumeFlag, sessionID)

	// Connect stdin, stdout, and stderr to the current terminal
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	// Run the command and wait for it to complete
	if err := cmd.Run(); err != nil {
		return fmt.Errorf("execute interactive with session %s and mode %s: failed to run Claude CLI: %w", sessionID, mode, err)
	}
	return nil
}

// ExecuteAndContinueInteractive executes a prompt and then continues in interactive mode.
// Session management is automatically handled by the Claude Code hook system.
// Returns dummy SessionInfo for backward compatibility.
func (e *ExecutorImpl) ExecuteAndContinueInteractive(prompt string) (*SessionInfo, error) {
	return e.ExecuteAndContinueInteractiveWithMode(prompt, "")
}

// ExecuteAndContinueInteractiveWithMode executes a prompt with mode and then continues in interactive mode.
// Session management is automatically handled by the Claude Code hook system.
// Returns dummy SessionInfo for backward compatibility.
func (e *ExecutorImpl) ExecuteAndContinueInteractiveWithMode(prompt, mode string) (*SessionInfo, error) {
	return e.ExecuteAndContinueInteractiveWithModeAndSystemPrompt(prompt, mode, "")
}

// ExecuteAndContinueInteractiveWithSystemPrompt executes a prompt with system prompt and then continues in interactive mode.
// Session management is automatically handled by the Claude Code hook system.
// Returns dummy SessionInfo for backward compatibility.
func (e *ExecutorImpl) ExecuteAndContinueInteractiveWithSystemPrompt(prompt, systemPrompt string) (*SessionInfo, error) {
	return e.ExecuteAndContinueInteractiveWithModeAndSystemPrompt(prompt, "", systemPrompt)
}

// ExecuteAndContinueInteractiveWithModeAndSystemPrompt executes a prompt with mode and system prompt and then continues in interactive mode.
// Session management is automatically handled by the Claude Code hook system.
// Returns dummy SessionInfo for backward compatibility.
func (e *ExecutorImpl) ExecuteAndContinueInteractiveWithModeAndSystemPrompt(prompt, mode, systemPrompt string) (*SessionInfo, error) {
	if err := e.validatePrompt(prompt); err != nil {
		return nil, fmt.Errorf("execute and continue interactive with mode and system prompt: %w", err)
	}

	// Execute in interactive mode with mode and system prompt - session management handled by hooks
	if err := e.ExecuteInteractiveWithModeAndSystemPrompt(prompt, mode, systemPrompt); err != nil {
		return nil, fmt.Errorf("execute and continue interactive with mode and system prompt: failed to start interactive mode: %w", err)
	}

	// Return dummy SessionInfo for backward compatibility
	// Real session info is tracked by the hook system
	return &SessionInfo{
		ID:     "tracked-by-hooks",
		Result: "Session completed in interactive mode",
	}, nil
}
