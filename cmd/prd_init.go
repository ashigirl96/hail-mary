package cmd

import (
	"fmt"
	"os"

	"github.com/ashigirl96/hail-mary/internal/claude"
	"github.com/spf13/cobra"
)

// prdInitCmd represents the prd init command
var prdInitCmd = &cobra.Command{
	Use:   "init",
	Short: "Initialize a new PRD with Claude assistance",
	Long: `Initialize a new Product Requirements Document (PRD) with Claude AI assistance.

This command launches Claude CLI to help you create a comprehensive PRD by guiding you through
the requirements gathering process. Claude will ask relevant questions and help structure
your product requirements document.`,
	RunE: func(cmd *cobra.Command, args []string) error {
		logger := GetLogger()
		logger.Info("Initializing PRD with Claude assistance")

		// Create PRD directory if it doesn't exist
		prdDir := "prd"
		if err := os.MkdirAll(prdDir, 0755); err != nil {
			return fmt.Errorf("failed to create PRD directory: %w", err)
		}

		// Initialize Claude executor
		executor := claude.NewExecutor()

		// Prepare the initial prompt for PRD creation
		prompt := `I need help creating a Product Requirements Document (PRD). 
Please guide me through the process by asking relevant questions about:
- The product vision and goals
- Target users and their needs
- Key features and functionality
- Technical requirements and constraints
- Success metrics and KPIs
- Timeline and milestones

Let's start with understanding what product we're building.`

		// Launch Claude in interactive mode
		fmt.Println("Launching Claude interactive shell for PRD creation...")
		fmt.Println("Press Ctrl+C to exit the Claude shell.")
		fmt.Println("\nTip: Use 'hail-mary prd continue' to resume this conversation later.")

		err := executor.ExecuteInteractive(prompt)
		if err != nil {
			return fmt.Errorf("failed to execute Claude: %w", err)
		}

		fmt.Printf("\n\nPRD session completed.\n")
		return nil
	},
}

func init() {
	prdCmd.AddCommand(prdInitCmd)
}
