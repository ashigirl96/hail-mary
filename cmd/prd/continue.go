package prd

import (
	"fmt"

	"github.com/ashigirl96/hail-mary/internal/claude"
	"github.com/spf13/cobra"
)

// Command-line flags
var (
	continuePermissionMode string
)

// prdContinueCmd represents the prd continue command
var prdContinueCmd = &cobra.Command{
	Use:   "continue",
	Short: "Continue working on an existing PRD session",
	Long: `Continue a previous PRD session with Claude AI.

This command resumes the most recent PRD conversation, maintaining the full context
of your previous discussion. Claude will remember all the requirements and decisions
made in earlier sessions.

Permission modes:
- acceptEdits: Automatically accept all file edits
- bypassPermissions: Skip all permission prompts
- default: Default Claude permission behavior
- plan: Plan mode for non-destructive analysis`,
	RunE: func(cmd *cobra.Command, args []string) error {
		logger := GetLogger()
		logger.Info("Continuing most recent PRD session")

		// Initialize Claude executor
		executor := claude.NewExecutor()

		// Launch Claude in interactive mode with --continue flag
		fmt.Println("Launching Claude interactive shell to continue PRD...")
		fmt.Println("Press Ctrl+C to exit the Claude shell.")

		// Use continue with permission mode if specified
		var err error
		if continuePermissionMode != "" {
			// For continue with mode, we need to set the config and use continue
			config := claude.DefaultConfig()
			config.PermissionMode = continuePermissionMode
			executor = claude.NewExecutorWithConfig(config)
		}

		err = executor.ExecuteInteractiveContinue()
		if err != nil {
			return fmt.Errorf("failed to continue Claude session: %w", err)
		}

		fmt.Printf("\n\nPRD session completed.\n")
		return nil
	},
}

func init() {
	PrdCmd.AddCommand(prdContinueCmd)

	// Add permission mode flag
	prdContinueCmd.Flags().StringVarP(&continuePermissionMode, "permission-mode", "m", "", "Permission mode to use for the session (choices: acceptEdits, bypassPermissions, default, plan)")
}
