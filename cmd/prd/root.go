package prd

import (
	"context"
	"errors"
	"fmt"
	"log/slog"

	"github.com/ashigirl96/hail-mary/internal/prd"
	"github.com/ashigirl96/hail-mary/internal/ui"
	"github.com/spf13/cobra"
)

// Cmd represents the prd command
var Cmd = &cobra.Command{
	Use:   "prd",
	Short: "Manage Product Requirements Documents with Claude AI",
	Long: `PRD (Product Requirements Document) management with Claude AI assistance.

This command launches an interactive UI to manage your product requirements documents.
You can create new PRDs or resume existing sessions with full conversation history.

Features:
- Create new requirements documents with guided assistance
- Resume and continue existing PRD sessions
- Redo conversations from any point in history
- Automatic session tracking and management

Navigation:
- j/k: Move up/down in lists
- h/l: Switch between panes
- Enter: Select an item
- q/Esc: Quit the interface`,
	RunE: runPRDCommand,
}

func Init(rootCmd *cobra.Command) {
	rootCmd.AddCommand(Cmd)
}

// GetLogger is a temporary function to access the logger
// This will be removed when we update root.go imports
var GetLogger func() *slog.Logger

// runPRDCommand is the main entry point for the unified PRD command
func runPRDCommand(cmd *cobra.Command, _ []string) error {
	logger := GetLogger()
	logger.Info("Starting PRD management interface")

	// Create PRD service
	service := prd.NewService(logger)

	// Get list of existing features
	features, err := service.GetFeatureList()
	if err != nil {
		return fmt.Errorf("failed to get feature list: %w", err)
	}

	// Launch the interactive UI
	selectedFeature, selectedSession, selectedInput, isContinue, err := ui.RunPRDResume(features)
	if err != nil {
		// Check if Create New Requirements was selected
		var createNewSelectedError *ui.CreateNewSelectedError
		if errors.As(err, &createNewSelectedError) {
			logger.Info("Create New Requirements selected")
			return service.CreateNewPRD(cmd.Context())
		}
		return fmt.Errorf("failed to run PRD UI: %w", err)
	}

	// Check if user cancelled
	if selectedFeature == "" || selectedSession == "" {
		logger.Info("User cancelled PRD operation")
		return nil
	}

	// Log selection details
	logger.Info("Feature and session selected",
		slog.String("feature", selectedFeature),
		slog.String("session", selectedSession),
		slog.Bool("is_continue", isContinue))

	// Handle redo vs continue
	if selectedInput != nil && !isContinue {
		logger.Info("Redoing conversation from specific input",
			slog.Int("redo_from_turn", selectedInput.TurnNumber),
			slog.String("removed_content", selectedInput.Content[:min(50, len(selectedInput.Content))]))
	} else if isContinue {
		logger.Info("Continuing from latest session")
	}

	// Resume the selected session
	ctx := context.Background()
	return service.ResumePRDSession(ctx, selectedFeature, selectedSession, selectedInput, isContinue)
}
