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
)

// Service provides PRD business logic operations
type Service struct {
	logger      *slog.Logger
	specManager *kiro.SpecManager
}

// NewService creates a new PRD service instance
func NewService(logger *slog.Logger) *Service {
	return &Service{
		logger:      logger,
		specManager: kiro.NewSpecManager(),
	}
}

// GetFeatureList returns a list of feature directories from .kiro/spec/
func (s *Service) GetFeatureList() ([]string, error) {
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

// CreateNewPRD handles the creation of a new PRD
func (s *Service) CreateNewPRD(ctx context.Context) error {
	// Get feature title from user
	featureTitle, err := ui.RunFeatureInput()
	if err != nil {
		return fmt.Errorf("failed to get feature title: %w", err)
	}

	s.logger.Info("Feature title received", slog.String("feature_title", featureTitle))

	// Create feature directory
	featurePath, err := s.specManager.CreateFeatureDir(featureTitle)
	if err != nil {
		return fmt.Errorf("failed to create feature directory: %w", err)
	}

	s.logger.Info("Feature directory created", slog.String("path", featurePath))

	// Save the initial requirements.md file
	if err := s.specManager.SaveRequirements(featureTitle, ""); err != nil {
		return fmt.Errorf("failed to create requirements.md: %w", err)
	}

	requirementsPath, err := s.specManager.GetRequirementsPath(featureTitle)
	if err != nil {
		s.logger.Warn("Failed to get requirements path", "error", err)
		requirementsPath = filepath.Join(featurePath, "requirements.md")
	}

	s.logger.Info("Requirements file created", slog.String("path", requirementsPath))
	fmt.Printf("Created requirements template at: %s\n", requirementsPath)

	// Execute PRD session
	return s.executePRDSession(ctx, DefaultPRDMode, featureTitle, featurePath, requirementsPath, "", nil)
}

// ResumePRDSession resumes an existing PRD session
func (s *Service) ResumePRDSession(ctx context.Context, featureTitle string, sessionID string, selectedInput *ui.UserInput, isContinue bool) error {
	featurePath := filepath.Join(".kiro", "spec", featureTitle)

	// Handle transcript truncation if needed
	if err := s.handleTranscriptTruncation(featurePath, sessionID, selectedInput, isContinue); err != nil {
		return err
	}

	// Get requirements path
	readableTitle := strings.ReplaceAll(featureTitle, "-", " ")
	requirementsPath, err := s.specManager.GetRequirementsPath(readableTitle)
	if err != nil {
		s.logger.Warn("Failed to get requirements path", "error", err)
		requirementsPath = fmt.Sprintf(".kiro/spec/%s/requirements.md", featureTitle)
	}

	opts := &SessionOptions{
		SelectedInput: selectedInput,
		IsContinue:    isContinue,
	}

	return s.executePRDSession(ctx, DefaultPRDMode, featureTitle, featurePath, requirementsPath, sessionID, opts)
}

// executePRDSession handles the common PRD session execution logic
func (s *Service) executePRDSession(ctx context.Context, mode string, featureTitle string, featurePath string, requirementsPath string, sessionID string, opts *SessionOptions) error {
	// Setup hook configuration with feature path
	hookConfigPath, cleanup, err := claude.SetupHookConfigWithFeature(s.logger, featurePath)
	if err != nil {
		return fmt.Errorf("failed to setup hooks: %w", err)
	}
	defer cleanup()

	// Create Claude executor
	executor := s.createClaudeExecutor(hookConfigPath, opts)

	// Display settings if debug enabled
	s.displaySettingsIfDebug(hookConfigPath)

	// Prepare prompt
	prompt := s.preparePrompt(featureTitle, sessionID, opts)

	// Get system prompt
	systemPrompt := kiro.GetRequirementsTemplate(requirementsPath)
	s.logger.Debug("Loaded PRD system prompt", "length", len(systemPrompt))

	// Create execution options
	execOpts := claude.ExecuteOptions{
		Prompt:       prompt,
		Mode:         mode,
		SystemPrompt: systemPrompt,
	}

	// Launch Claude session
	if sessionID != "" {
		return s.executeWithSession(executor, sessionID, execOpts, requirementsPath, hookConfigPath)
	}
	return s.executeNewSession(executor, execOpts, requirementsPath, hookConfigPath)
}

// handleTranscriptTruncation handles transcript truncation for redo functionality
func (s *Service) handleTranscriptTruncation(featurePath string, sessionID string, selectedInput *ui.UserInput, isContinue bool) error {
	if selectedInput == nil || isContinue {
		return nil
	}

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

	// Truncate transcript
	truncateAtTurn := selectedInput.TurnNumber - 1
	s.logger.Info("Truncating session to redo from selected turn",
		slog.Int("selected_turn", selectedInput.TurnNumber),
		slog.Int("truncate_at_turn", truncateAtTurn),
		slog.String("redo_content", truncateForLog(selectedInput.Content, LogContentTruncateMedium)))

	truncatedPath, err := claude.TruncateTranscript(sessionState.TranscriptPath, truncateAtTurn)
	if err != nil {
		return fmt.Errorf("failed to truncate transcript: %w", err)
	}

	// Replace the original transcript
	if err := os.Rename(truncatedPath, sessionState.TranscriptPath); err != nil {
		return fmt.Errorf("failed to replace transcript: %w", err)
	}

	s.logger.Info("Transcript truncated successfully for redo",
		slog.Int("removed_from_turn", selectedInput.TurnNumber),
		slog.String("transcript", sessionState.TranscriptPath))

	return nil
}

// createClaudeExecutor creates a Claude executor with appropriate configuration
func (s *Service) createClaudeExecutor(hookConfigPath string, opts *SessionOptions) claude.Executor {
	config := claude.DefaultConfig()
	if opts != nil {
		config.SkipPermissions = true
	}
	config.SettingsPath = hookConfigPath
	return claude.NewExecutorWithConfig(config)
}

// displaySettingsIfDebug displays settings content if debug logging is enabled
func (s *Service) displaySettingsIfDebug(hookConfigPath string) {
	fmt.Printf("\nUsing merged settings: %s\n", hookConfigPath)

	if !s.logger.Enabled(context.Background(), slog.LevelDebug) {
		return
	}

	settingsContent, err := os.ReadFile(hookConfigPath)
	if err != nil {
		s.logger.Warn("Failed to read merged settings", "error", err)
		return
	}

	fmt.Println("\n=== Merged Settings ===")
	fmt.Println(string(settingsContent))
	fmt.Println("======================")
}

// preparePrompt prepares the appropriate prompt based on session options
func (s *Service) preparePrompt(featureTitle string, sessionID string, opts *SessionOptions) string {
	readableTitle := strings.ReplaceAll(featureTitle, "-", " ")

	// New session
	if sessionID == "" {
		return ""
	}

	// Resume with redo
	if opts != nil && opts.SelectedInput != nil && !opts.IsContinue {
		return fmt.Sprintf(`I'm resuming work on the Product Requirements Document (PRD) for the feature: "%s"

I want to redo the conversation from Turn %d onwards. The session ID is: %s
The conversation was truncated, removing the following input and everything after it:
"%s"

Let's continue with a fresh approach from where we left off.`,
			readableTitle,
			opts.SelectedInput.TurnNumber,
			sessionID[:SessionIDDisplayLength],
			truncateForLog(opts.SelectedInput.Content, LogContentTruncateLong))
	}

	// Continue session
	return fmt.Sprintf(`I'm resuming work on the Product Requirements Document (PRD) for the feature: "%s"

Please continue helping me develop this PRD. The previous session ID is: %s

Let's continue where we left off.`,
		readableTitle,
		sessionID[:SessionIDDisplayLength])
}

// executeWithSession executes Claude with an existing session
func (s *Service) executeWithSession(executor claude.Executor, sessionID string, opts claude.ExecuteOptions, requirementsPath string, hookConfigPath string) error {
	fmt.Println("Resuming Claude interactive shell for PRD editing...")
	fmt.Println("Press Ctrl+C to exit the Claude shell.")

	if err := executor.ExecuteWithSession(sessionID, opts); err != nil {
		s.logger.Error("Failed to execute Claude interactive session",
			"error", err,
			"prompt_length", len(opts.Prompt),
			"settings_path", hookConfigPath)
		return fmt.Errorf("claude execution failed: %w", err)
	}

	displayCompletionMessage(requirementsPath)
	return nil
}

// executeNewSession executes Claude with a new session
func (s *Service) executeNewSession(executor claude.Executor, opts claude.ExecuteOptions, requirementsPath string, hookConfigPath string) error {
	fmt.Println("Launching Claude interactive shell for PRD creation...")
	fmt.Println("Press Ctrl+C to exit the Claude shell.")

	s.logger.Debug("Starting Claude interactive session")

	// Execute synchronously without timeout for interactive sessions
	if err := executor.Execute(opts); err != nil {
		s.logger.Error("Failed to execute Claude interactive session",
			"error", err,
			"prompt_length", len(opts.Prompt),
			"settings_path", hookConfigPath)
		return fmt.Errorf("claude execution failed: %w", err)
	}

	s.logger.Debug("Claude interactive session completed successfully")
	displayCompletionMessage(requirementsPath)
	return nil
}

// truncateForLog truncates a string for logging purposes
func truncateForLog(s string, maxLen int) string {
	if len(s) <= maxLen {
		return s
	}
	return s[:maxLen-3] + "..."
}

// displayCompletionMessage displays the completion message with requirements path
func displayCompletionMessage(requirementsPath string) {
	fmt.Printf("\n\nPRD session completed.\n")
	fmt.Printf("\nRequirements file location: %s\n", requirementsPath)
	fmt.Printf("\nYou can now edit this file to update your PRD based on the Claude session.\n")
	fmt.Printf("  vim %s  # or use your preferred editor\n", requirementsPath)
}
