package prd

import (
	"fmt"

	"github.com/ashigirl96/hail-mary/internal/claude"
	"github.com/spf13/cobra"
)

// prdContinueCmd represents the prd continue command
var prdContinueCmd = &cobra.Command{
	Use:   "continue",
	Short: "Continue working on an existing PRD session",
	Long: `Continue a previous PRD session with Claude AI.

This command resumes the most recent PRD conversation, maintaining the full context
of your previous discussion. Claude will remember all the requirements and decisions
made in earlier sessions.`,
	RunE: func(cmd *cobra.Command, args []string) error {
		logger := GetLogger()
		logger.Info("Continuing most recent PRD session")

		// Initialize Claude executor
		executor := claude.NewExecutor()

		// Launch Claude in interactive mode with --continue flag
		fmt.Println("Launching Claude interactive shell to continue PRD...")
		fmt.Println("Press Ctrl+C to exit the Claude shell.")

		err := executor.ExecuteInteractiveContinue()
		if err != nil {
			return fmt.Errorf("failed to continue Claude session: %w", err)
		}

		fmt.Printf("\n\nPRD session completed.\n")
		return nil
	},
}

func init() {
	PrdCmd.AddCommand(prdContinueCmd)
}