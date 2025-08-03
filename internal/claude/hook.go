package claude

import (
	"fmt"
	"log/slog"
	"os"
	"path/filepath"

	"github.com/ashigirl96/hail-mary/internal/settings"
)

// SetupHookConfigWithFeature creates temporary hook configuration with feature path
// Returns the path to the temporary config file and a cleanup function
func SetupHookConfigWithFeature(logger *slog.Logger, featurePath string) (string, func(), error) {
	// Get executable path for hook command
	execPath, err := os.Executable()
	if err != nil {
		return "", nil, fmt.Errorf("failed to get executable path: %w", err)
	}

	// Create hook configuration with feature path if provided
	envVars := fmt.Sprintf("HAIL_MARY_PARENT_PID=%d", os.Getpid())
	if featurePath != "" {
		envVars = fmt.Sprintf("%s HAIL_MARY_FEATURE_PATH=%s", envVars, featurePath)
	}
	hookCmd := fmt.Sprintf("%s %s hook", envVars, execPath)

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
