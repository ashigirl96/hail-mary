package prd

import (
	"context"
	"fmt"
	"log/slog"
	"os"
	"time"

	"github.com/ashigirl96/hail-mary/internal/claude"
	"github.com/ashigirl96/hail-mary/internal/kiro"
	"github.com/ashigirl96/hail-mary/internal/prompt"
	"github.com/ashigirl96/hail-mary/internal/session"
	"github.com/ashigirl96/hail-mary/internal/ui"
	"github.com/spf13/cobra"
)

// GetLogger is a temporary function to access the logger
// This will be removed when we update root.go imports
var GetLogger func() *slog.Logger

// Command-line flags
// (no flags for prd init - always uses plan mode)

// prdInitCmd represents the prd init command
var prdInitCmd = &cobra.Command{
	Use:   "init",
	Short: "Initialize a new PRD with Claude assistance",
	Long: `Initialize a new Product Requirements Document (PRD) with Claude AI assistance.

This command launches Claude CLI in plan mode to help you create a comprehensive PRD by guiding you through
the requirements gathering process. Claude will ask relevant questions and help structure
your product requirements document.

Note: This command automatically runs in plan mode for non-destructive analysis and safe exploration.`,
	RunE: func(cmd *cobra.Command, args []string) error {
		logger := GetLogger()
		logger.Info("Initializing PRD with Claude assistance")

		// Get feature title from user
		featureTitle, err := ui.RunFeatureInput()
		if err != nil {
			return fmt.Errorf("failed to get feature title: %w", err)
		}

		logger.Info("Feature title received", slog.String("feature_title", featureTitle))

		// Create spec manager and feature directory
		specManager := kiro.NewSpecManager()
		featurePath, err := specManager.CreateFeatureDir(featureTitle)
		if err != nil {
			return fmt.Errorf("failed to create feature directory: %w", err)
		}

		logger.Info("Feature directory created", slog.String("path", featurePath))

		// Create PRD directory if it doesn't exist
		prdDir := "prd"
		if err := os.MkdirAll(prdDir, 0755); err != nil {
			return fmt.Errorf("failed to create PRD directory: %w", err)
		}

		// Use hook-based session tracking with plan mode
		ctx := context.Background()
		return initPRDWithHooks(ctx, logger, "plan", featureTitle, featurePath, specManager)
	},
}

func init() {
	Cmd.AddCommand(prdInitCmd)
	// No flags needed - prd init always uses plan mode
}

// initPRDWithHooks initializes PRD with hook-based session tracking
func initPRDWithHooks(ctx context.Context, logger *slog.Logger, mode string, featureTitle string, featurePath string, specManager *kiro.SpecManager) error {
	// Setup hook configuration with feature path
	hookConfigPath, cleanup, err := claude.SetupHookConfigWithFeature(logger, featurePath)
	if err != nil {
		return fmt.Errorf("failed to setup hooks: %w", err)
	}
	defer cleanup()

	// Create Claude executor with settings path
	config := claude.DefaultConfig()
	config.SettingsPath = hookConfigPath
	executor := claude.NewExecutorWithConfig(config)

	// Prepare the initial prompt for PRD creation with feature context
	//	initialPrompt := fmt.Sprintf(`I'm creating a Product Requirements Document (PRD) for a feature titled: "%s"
	//
	//Please help me develop a comprehensive PRD for this feature. Let's start by understanding the problem space and requirements.`, featureTitle)
	initialPrompt := ``

	// Start monitoring for session
	sessionChan := make(chan *session.State, 1)
	errChan := make(chan error, 1)

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

	// Launch Claude in a goroutine
	go func() {
		logger.Debug("Starting Claude interactive session")
		fmt.Println("Launching Claude interactive shell for PRD creation...")
		fmt.Println("Press Ctrl+C to exit the Claude shell.")

		logger.Debug("Executing Claude with config",
			"settings_path", hookConfigPath,
			"executor_config", config)

		// Read system prompt from file if it exists
		systemPrompt, err := prompt.ReadPRDSystemPrompt(logger)
		if err != nil {
			logger.Warn("Failed to read system prompt file, continuing without it", "error", err)
			systemPrompt = ""
		}

		// Create execution options
		opts := claude.ExecuteOptions{
			Prompt:       initialPrompt,
			Mode:         mode,
			SystemPrompt: systemPrompt,
		}

		if err := executor.Execute(opts); err != nil {
			logger.Error("Failed to execute Claude interactive session",
				"error", err,
				"prompt_length", len(initialPrompt),
				"settings_path", hookConfigPath)
			errChan <- err
			return
		}
		logger.Debug("Claude interactive session completed successfully")
		errChan <- nil
	}()

	// Start monitoring for session establishment
	go monitorSessionEstablishment(ctx, logger, sessionChan, featurePath)

	// Wait for session or error
	select {
	case sessionState := <-sessionChan:
		logger.Info("Session established",
			"session_id", sessionState.SessionID,
			"transcript_path", sessionState.TranscriptPath)
		fmt.Printf("\nSession ID: %s\n", sessionState.SessionID[:8])

	case <-time.After(30 * time.Second):
		// Session tracking failed, but continue anyway
		logger.Warn("Session tracking timeout, continuing without session ID")

	case err := <-errChan:
		if err != nil {
			return fmt.Errorf("claude execution failed: %w", err)
		}
		fmt.Printf("\n\nPRD session completed.\n")
		return nil
	}

	// Wait for Claude to finish
	err = <-errChan
	if err != nil {
		return fmt.Errorf("claude execution failed: %w", err)
	}

	fmt.Printf("\n\nPRD session completed.\n")

	// Display where to save the PRD
	requirementsPath, err := specManager.GetRequirementsPath(featureTitle)
	if err == nil {
		fmt.Printf("\nPlease save your PRD to: %s\n", requirementsPath)
		fmt.Printf("\nYou can copy the PRD content from the Claude session and save it using:\n")
		fmt.Printf("  echo 'YOUR_PRD_CONTENT' > %s\n", requirementsPath)
	}

	return nil
}

// monitorSessionEstablishment monitors for session file creation
func monitorSessionEstablishment(ctx context.Context, logger *slog.Logger, sessionChan chan<- *session.State, featurePath string) {
	processID := fmt.Sprintf("%d", os.Getpid())
	fmt.Printf("Monitoring for session establishment (PID: %s)...\n", processID)

	sm, err := session.NewManager()
	if err != nil {
		logger.Error("Failed to create session manager", "error", err)
		return
	}

	ticker := time.NewTicker(100 * time.Millisecond)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return

		case <-ticker.C:
			state, err := sm.ReadSession(processID)
			if err == nil {
				// Save session to feature directory
				if err := sm.WriteSessionToFeature(featurePath, state); err != nil {
					logger.Error("Failed to write session to feature directory",
						"error", err,
						"feature_path", featurePath)
				} else {
					logger.Info("Session saved to feature directory",
						"session_id", state.SessionID,
						"feature_path", featurePath)
				}
				sessionChan <- state
				return
			}
		}
	}
}
