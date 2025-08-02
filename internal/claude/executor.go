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
	// ExecuteInteractiveContinue continues the most recent session
	ExecuteInteractiveContinue() error
	// ExecuteInteractiveWithSession launches interactive mode with a specific session
	ExecuteInteractiveWithSession(sessionID string) error
	// ExecuteAndContinueInteractive executes a prompt and then continues in interactive mode
	ExecuteAndContinueInteractive(prompt string) (*SessionInfo, error)
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
// Session tracking is automatically handled by the Claude Code hook system.
func (e *ExecutorImpl) ExecuteInteractive(prompt string) error {
	if err := e.validatePrompt(prompt); err != nil {
		return fmt.Errorf("execute interactive: %w", err)
	}

	// Create command for interactive Claude shell with initial prompt
	cmd := e.buildCommand(prompt)

	// Connect stdin, stdout, and stderr to the current terminal
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	// Run the command and wait for it to complete
	if err := cmd.Run(); err != nil {
		return fmt.Errorf("execute interactive: failed to run Claude CLI: %w", err)
	}
	return nil
}

// ExecuteInteractiveContinue continues the most recent Claude session in interactive mode.
// This is useful for resuming a conversation after the CLI has exited.
func (e *ExecutorImpl) ExecuteInteractiveContinue() error {
	// Create command for interactive Claude shell with --continue flag
	cmd := e.buildCommand(continueFlag)

	// Connect stdin, stdout, and stderr to the current terminal
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	// Run the command and wait for it to complete
	if err := cmd.Run(); err != nil {
		return fmt.Errorf("execute interactive continue: failed to continue Claude session: %w", err)
	}
	return nil
}

// ExecuteInteractiveWithSession launches Claude CLI in interactive mode with a specific session ID.
// This combines the benefits of interactive mode with the ability to resume a specific
// previous session, allowing users to continue a conversation interactively.
func (e *ExecutorImpl) ExecuteInteractiveWithSession(sessionID string) error {
	if err := validateSessionID(sessionID); err != nil {
		return fmt.Errorf("execute interactive with session: %w", err)
	}

	// Create command for interactive Claude shell with --resume flag
	cmd := e.buildCommand(resumeFlag, sessionID)

	// Connect stdin, stdout, and stderr to the current terminal
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	// Run the command and wait for it to complete
	if err := cmd.Run(); err != nil {
		return fmt.Errorf("execute interactive with session %s: failed to run Claude CLI: %w", sessionID, err)
	}
	return nil
}

// ExecuteAndContinueInteractive executes a prompt and then continues in interactive mode.
// Session tracking is automatically handled by the Claude Code hook system.
// Returns dummy SessionInfo for backward compatibility.
func (e *ExecutorImpl) ExecuteAndContinueInteractive(prompt string) (*SessionInfo, error) {
	if err := e.validatePrompt(prompt); err != nil {
		return nil, fmt.Errorf("execute and continue interactive: %w", err)
	}

	// Execute in interactive mode - session tracking handled by hooks
	if err := e.ExecuteInteractive(prompt); err != nil {
		return nil, fmt.Errorf("execute and continue interactive: failed to start interactive mode: %w", err)
	}

	// Return dummy SessionInfo for backward compatibility
	// Real session info is tracked by the hook system
	return &SessionInfo{
		ID:     "tracked-by-hooks",
		Result: "Session completed in interactive mode",
	}, nil
}
