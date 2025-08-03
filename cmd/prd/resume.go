package prd

import (
	"context"
	"fmt"
	"log/slog"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"github.com/ashigirl96/hail-mary/internal/claude"
	"github.com/ashigirl96/hail-mary/internal/kiro"
	"github.com/ashigirl96/hail-mary/internal/prompt"
	"github.com/ashigirl96/hail-mary/internal/ui"
	"github.com/spf13/cobra"
)

// prdResumeCmd represents the prd resume command
var prdResumeCmd = &cobra.Command{
	Use:   "resume",
	Short: "Resume a PRD session with Claude assistance",
	Long: `Resume an existing Product Requirements Document (PRD) session with Claude AI assistance.

This command displays a TUI with two panes:
- Left pane: List of existing features in .kiro/spec/
- Right pane: Related Claude session history for the selected feature

Navigation:
- j/k: Move up/down within the current pane
- h/l: Switch between left and right panes
- Enter: Select and resume a session
- q/Esc: Quit`,
	RunE: func(cmd *cobra.Command, args []string) error {
		logger := GetLogger()
		logger.Info("Starting PRD resume interface")

		// Create spec manager
		specManager := kiro.NewSpecManager()

		// Get list of existing features
		features, err := getFeatureList(specManager)
		if err != nil {
			return fmt.Errorf("failed to get feature list: %w", err)
		}

		if len(features) == 0 {
			return fmt.Errorf("no features found in .kiro/spec/")
		}

		// Launch the TUI
		selectedFeature, selectedSession, err := ui.RunPRDResume(features)
		if err != nil {
			return fmt.Errorf("failed to run PRD resume UI: %w", err)
		}

		if selectedFeature == "" || selectedSession == "" {
			// User cancelled
			return nil
		}

		logger.Info("Feature and session selected",
			slog.String("feature", selectedFeature),
			slog.String("session", selectedSession))

		// Resume the session
		return resumePRDSession(cmd.Context(), logger, selectedFeature, selectedSession, specManager)
	},
}

func init() {
	Cmd.AddCommand(prdResumeCmd)
}

// getFeatureList returns a list of feature directories from .kiro/spec/
func getFeatureList(specManager *kiro.SpecManager) ([]string, error) {
	specDir := filepath.Join(kiro.KiroDir, kiro.SpecDir)

	entries, err := os.ReadDir(specDir)
	if err != nil {
		if os.IsNotExist(err) {
			return []string{}, nil
		}
		return nil, fmt.Errorf("failed to read spec directory: %w", err)
	}

	var features []string
	for _, entry := range entries {
		if entry.IsDir() {
			features = append(features, entry.Name())
		}
	}

	// Sort features alphabetically
	sort.Strings(features)

	return features, nil
}

// resumePRDSession resumes a Claude session for PRD editing
func resumePRDSession(ctx context.Context, logger *slog.Logger, featureTitle string, sessionID string, specManager *kiro.SpecManager) error {
	// Get feature path
	featurePath := filepath.Join(".kiro", "spec", featureTitle)

	// Setup hook configuration with feature path
	hookConfigPath, cleanup, err := claude.SetupHookConfigWithFeature(logger, featurePath)
	if err != nil {
		return fmt.Errorf("failed to setup hooks: %w", err)
	}
	defer cleanup()

	// Create Claude executor with settings path
	config := claude.DefaultConfig()
	config.SkipPermissions = false
	config.SettingsPath = hookConfigPath
	executor := claude.NewExecutorWithConfig(config)

	// Get the feature title from directory name (convert back from kebab-case)
	readableTitle := strings.ReplaceAll(featureTitle, "-", " ")

	// Prepare the resume prompt
	resumePrompt := fmt.Sprintf(`I'm resuming work on the Product Requirements Document (PRD) for the feature: "%s"

Please continue helping me develop this PRD. The previous session ID is: %s

Let's continue where we left off.`, readableTitle, sessionID[:8])

	// Display merged settings content
	settingsContent, err := os.ReadFile(hookConfigPath)
	if err != nil {
		logger.Warn("Failed to read merged settings", "error", err)
	} else {
		// Always show settings path
		fmt.Printf("\nUsing merged settings: %s\n", hookConfigPath)

		// Show settings content if debug logging is enabled
		if logger.Enabled(context.Background(), slog.LevelDebug) {
			fmt.Println("\n=== Merged Settings ===")
			fmt.Println(string(settingsContent))
			fmt.Println("======================")
		}
	}

	// Launch Claude
	fmt.Println("Resuming Claude interactive shell for PRD editing...")
	fmt.Println("Press Ctrl+C to exit the Claude shell.")

	// Read system prompt from file if it exists
	systemPrompt, err := prompt.ReadPRDSystemPrompt(logger)
	if err != nil {
		logger.Warn("Failed to read system prompt file, continuing without it", "error", err)
		systemPrompt = ""
	}

	// Create execution options for resuming
	opts := claude.ExecuteOptions{
		Prompt:       resumePrompt,
		Mode:         "plan", // Always use plan mode for PRD
		SystemPrompt: systemPrompt,
	}

	if err := executor.ExecuteWithSession(sessionID, opts); err != nil {
		logger.Error("Failed to execute Claude interactive session",
			"error", err,
			"prompt_length", len(resumePrompt),
			"settings_path", hookConfigPath)
		return fmt.Errorf("claude execution failed: %w", err)
	}

	fmt.Printf("\n\nPRD session completed.\n")

	// Display where to save the PRD
	requirementsPath, err := specManager.GetRequirementsPath(readableTitle)
	if err == nil {
		fmt.Printf("\nPlease save your PRD to: %s\n", requirementsPath)
		fmt.Printf("\nYou can copy the PRD content from the Claude session and save it using:\n")
		fmt.Printf("  echo 'YOUR_PRD_CONTENT' > %s\n", requirementsPath)
	}

	return nil
}
