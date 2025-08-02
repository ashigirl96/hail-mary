package prd

import (
	"context"
	"fmt"
	"log/slog"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/ashigirl96/hail-mary/internal/claude"
	"github.com/ashigirl96/hail-mary/internal/session"
	"github.com/ashigirl96/hail-mary/internal/settings"
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

		// Create PRD directory if it doesn't exist
		prdDir := "prd"
		if err := os.MkdirAll(prdDir, 0755); err != nil {
			return fmt.Errorf("failed to create PRD directory: %w", err)
		}

		// Use hook-based session tracking with plan mode
		ctx := context.Background()
		return initPRDWithHooks(ctx, logger, "plan")
	},
}

func init() {
	PrdCmd.AddCommand(prdInitCmd)
	// No flags needed - prd init always uses plan mode
}

// initPRDWithHooks initializes PRD with hook-based session tracking
func initPRDWithHooks(ctx context.Context, logger *slog.Logger, mode string) error {
	// Setup hook configuration
	hookConfigPath, cleanup, err := setupHookConfig(logger)
	if err != nil {
		return fmt.Errorf("failed to setup hooks: %w", err)
	}
	defer cleanup()

	// Create Claude executor with settings path
	config := claude.DefaultConfig()
	config.SettingsPath = hookConfigPath
	executor := claude.NewExecutorWithConfig(config)

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
		systemPrompt, err := readSystemPromptFile(logger)
		if err != nil {
			logger.Warn("Failed to read system prompt file, continuing without it", "error", err)
			systemPrompt = ""
		}

		if err := executor.ExecuteInteractiveWithModeAndSystemPrompt(prompt, mode, systemPrompt); err != nil {
			logger.Error("Failed to execute Claude interactive session",
				"error", err,
				"prompt_length", len(prompt),
				"settings_path", hookConfigPath)
			errChan <- err
			return
		}
		logger.Debug("Claude interactive session completed successfully")
		errChan <- nil
	}()

	// Start monitoring for session establishment
	go monitorSessionEstablishment(ctx, logger, sessionChan)

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
	return nil
}

// setupHookConfig creates temporary hook configuration
func setupHookConfig(logger *slog.Logger) (string, func(), error) {
	// Get executable path for hook command
	execPath, err := os.Executable()
	if err != nil {
		return "", nil, fmt.Errorf("failed to get executable path: %w", err)
	}

	// Create hook configuration
	hookCmd := fmt.Sprintf("HAIL_MARY_PARENT_PID=%d %s hook", os.Getpid(), execPath)

	// Define our hooks
	hailMaryHooks := map[string][]settings.HookMatcher{
		"SessionStart": {
			{
				Hooks: []settings.HookEntry{
					{
						Type:    "command",
						Command: hookCmd,
						Timeout: 5,
					},
				},
			},
		},
		"UserPromptSubmit": {
			{
				Hooks: []settings.HookEntry{
					{
						Type:    "command",
						Command: hookCmd,
						Timeout: 2,
					},
				},
			},
		},
		"Stop": {
			{
				Hooks: []settings.HookEntry{
					{
						Type:    "command",
						Command: hookCmd,
						Timeout: 2,
					},
				},
			},
		},
	}

	// Check for existing .claude/settings.json
	wd, err := os.Getwd()
	if err != nil {
		return "", nil, fmt.Errorf("failed to get working directory: %w", err)
	}
	existingSettingsPath := filepath.Join(wd, ".claude", "settings.json")

	// Create merged settings
	mergedSettings, err := settings.CreateMergedSettings(existingSettingsPath, hailMaryHooks)
	if err != nil {
		return "", nil, fmt.Errorf("failed to create merged settings: %w", err)
	}

	// Write temporary merged settings
	tempDir := os.TempDir()
	tempHookPath := filepath.Join(tempDir, fmt.Sprintf("hail-mary-settings-%d.json", os.Getpid()))

	if err := mergedSettings.SaveToFile(tempHookPath); err != nil {
		return "", nil, fmt.Errorf("failed to save merged settings: %w", err)
	}

	// Cleanup function
	cleanup := func() {
		os.Remove(tempHookPath)
		// Session files are preserved for future reference
	}

	logger.Debug("Merged settings created",
		"config_path", tempHookPath,
		"parent_pid", os.Getpid(),
		"existing_settings", existingSettingsPath)

	return tempHookPath, cleanup, nil
}

// monitorSessionEstablishment monitors for session file creation
func monitorSessionEstablishment(ctx context.Context, logger *slog.Logger, sessionChan chan<- *session.State) {
	processID := fmt.Sprintf("%d", os.Getpid())

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
				sessionChan <- state
				return
			}
		}
	}
}

// readSystemPromptFile reads the PRD system prompt from file
func readSystemPromptFile(logger *slog.Logger) (string, error) {
	// Get current working directory
	wd, err := os.Getwd()
	if err != nil {
		return "", fmt.Errorf("failed to get working directory: %w", err)
	}

	// Try to read from system-prompt/prd.md
	systemPromptPath := filepath.Join(wd, "system-prompt", "prd.md")
	content, err := os.ReadFile(systemPromptPath)
	if err != nil {
		return "", fmt.Errorf("failed to read system prompt file %s: %w", systemPromptPath, err)
	}

	// Convert to string and trim whitespace
	systemPrompt := strings.TrimSpace(string(content))
	if systemPrompt == "" {
		return "", fmt.Errorf("system prompt file is empty")
	}

	logger.Debug("Loaded system prompt from file",
		"path", systemPromptPath,
		"length", len(systemPrompt))

	return systemPrompt, nil
}
