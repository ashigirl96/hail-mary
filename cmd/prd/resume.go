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
- Enter: Select and redo conversation from this point (removes selected input and everything after)
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
		selectedFeature, selectedSession, selectedInput, isContinue, err := ui.RunPRDResume(features)
		if err != nil {
			return fmt.Errorf("failed to run PRD resume UI: %w", err)
		}

		if selectedFeature == "" || selectedSession == "" {
			// User cancelled
			return nil
		}

		logger.Info("Feature and session selected",
			slog.String("feature", selectedFeature),
			slog.String("session", selectedSession),
			slog.Bool("is_continue", isContinue))

		if selectedInput != nil && !isContinue {
			logger.Info("Redoing conversation from specific input",
				slog.Int("redo_from_turn", selectedInput.TurnNumber),
				slog.String("removed_content", truncateForLog(selectedInput.Content, 50)))
		} else if isContinue {
			logger.Info("Continuing from latest session")
		}

		// Resume the session
		return resumePRDSession(cmd.Context(), logger, selectedFeature, selectedSession, selectedInput, isContinue, specManager)
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

// truncateForLog truncates a string for logging purposes
func truncateForLog(s string, maxLen int) string {
	if len(s) <= maxLen {
		return s
	}
	return s[:maxLen-3] + "..."
}

// resumePRDSession resumes a Claude session for PRD editing
func resumePRDSession(ctx context.Context, logger *slog.Logger, featureTitle string, sessionID string, selectedInput *ui.UserInput, isContinue bool, specManager *kiro.SpecManager) error {
	// Get feature path
	featurePath := filepath.Join(".kiro", "spec", featureTitle)

	// If a specific input was selected (and not continue), we need to truncate the transcript
	if selectedInput != nil && !isContinue {
		// Load session state to get transcript path
		sessionManager := claude.NewFeatureSessionStateManager(featurePath)
		sessions, err := sessionManager.LoadSessions()
		if err != nil {
			return fmt.Errorf("failed to load sessions: %w", err)
		}

		// Find the session
		sessionState, _ := sessions.FindBySessionID(sessionID)
		if sessionState == nil {
			return fmt.Errorf("session %s not found", sessionID)
		}

		// Truncate without warning - user already confirmed selection in UI
		// Remove the selected input and everything after it (redo from that point)
		truncateAtTurn := selectedInput.TurnNumber - 1 // Exclude the selected user input itself
		logger.Info("Truncating session to redo from selected turn",
			slog.Int("selected_turn", selectedInput.TurnNumber),
			slog.Int("truncate_at_turn", truncateAtTurn),
			slog.String("redo_content", truncateForLog(selectedInput.Content, 80)))

		// Truncate the transcript (everything from selected turn onwards will be removed)
		truncatedPath, err := claude.TruncateTranscript(sessionState.TranscriptPath, truncateAtTurn)
		if err != nil {
			return fmt.Errorf("failed to truncate transcript: %w", err)
		}

		// Replace the original transcript with the truncated one
		if err := os.Rename(truncatedPath, sessionState.TranscriptPath); err != nil {
			return fmt.Errorf("failed to replace transcript: %w", err)
		}

		logger.Info("Transcript truncated successfully for redo",
			slog.Int("removed_from_turn", selectedInput.TurnNumber),
			slog.String("transcript", sessionState.TranscriptPath))
	}

	// Setup hook configuration with feature path
	hookConfigPath, cleanup, err := claude.SetupHookConfigWithFeature(logger, featurePath)
	if err != nil {
		return fmt.Errorf("failed to setup hooks: %w", err)
	}
	defer cleanup()

	// Create Claude executor with settings path
	config := claude.DefaultConfig()
	config.SkipPermissions = true
	config.SettingsPath = hookConfigPath
	executor := claude.NewExecutorWithConfig(config)

	// Get the feature title from directory name (convert back from kebab-case)
	readableTitle := strings.ReplaceAll(featureTitle, "-", " ")

	// Prepare the resume prompt
	var resumePrompt string
	if selectedInput != nil && !isContinue {
		resumePrompt = fmt.Sprintf(`I'm resuming work on the Product Requirements Document (PRD) for the feature: "%s"

I want to redo the conversation from Turn %d onwards. The session ID is: %s
The conversation was truncated, removing the following input and everything after it:
"%s"

Let's continue with a fresh approach from where we left off.`, readableTitle, selectedInput.TurnNumber, sessionID[:8], truncateForLog(selectedInput.Content, 120))
	} else {
		resumePrompt = fmt.Sprintf(`I'm resuming work on the Product Requirements Document (PRD) for the feature: "%s"

Please continue helping me develop this PRD. The previous session ID is: %s

Let's continue where we left off.`, readableTitle, sessionID[:8])
	}

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

	// Get requirements path for the feature
	requirementsPath, err := specManager.GetRequirementsPath(readableTitle)
	if err != nil {
		logger.Warn("Failed to get requirements path", "error", err)
		requirementsPath = fmt.Sprintf(".kiro/spec/%s/requirements.md", featureTitle)
	}

	// Get system prompt with the requirements path
	systemPrompt := kiro.GetRequirementsTemplate(requirementsPath)
	logger.Debug("Loaded PRD system prompt", "length", len(systemPrompt))

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

	// Display where to save the PRD (reuse requirementsPath from above)
	fmt.Printf("\nPlease save your PRD to: %s\n", requirementsPath)
	fmt.Printf("\nYou can copy the PRD content from the Claude session and save it using:\n")
	fmt.Printf("  echo 'YOUR_PRD_CONTENT' > %s\n", requirementsPath)

	return nil
}
