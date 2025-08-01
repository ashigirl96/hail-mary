package claude

import (
	"os"
	"os/exec"
)


// Executor handles Claude CLI execution
type Executor struct {
}

// NewExecutor creates a new Claude executor
func NewExecutor() *Executor {
	return &Executor{}
}



// ExecuteInteractive launches Claude CLI in interactive mode (actual shell)
func (e *Executor) ExecuteInteractive(prompt string) error {
	// Create a command for interactive Claude shell without JSON output
	cmd := exec.Command("bunx", "@anthropic-ai/claude-code@latest", "--dangerously-skip-permissions", prompt)
	
	// Set environment variables
	cmd.Env = append(os.Environ(),
		"ENABLE_BACKGROUND_TASKS=1",
		"CLAUDE_BASH_MAINTAIN_PROJECT_WORKING_DIR=1",
	)

	// Connect stdin, stdout, and stderr to the current terminal
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	// Run the command and wait for it to complete
	return cmd.Run()
}


// ExecuteInteractiveContinue continues the most recent Claude session in interactive mode
func (e *Executor) ExecuteInteractiveContinue() error {
	// Create a command for interactive Claude shell with --continue flag
	cmd := exec.Command("bunx", "@anthropic-ai/claude-code@latest", "--dangerously-skip-permissions", "--continue")
	
	// Set environment variables
	cmd.Env = append(os.Environ(),
		"ENABLE_BACKGROUND_TASKS=1",
		"CLAUDE_BASH_MAINTAIN_PROJECT_WORKING_DIR=1",
	)

	// Connect stdin, stdout, and stderr to the current terminal
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	// Run the command and wait for it to complete
	return cmd.Run()
}
