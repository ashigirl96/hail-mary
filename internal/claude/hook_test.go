package claude

import (
	"encoding/json"
	"fmt"
	"log/slog"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/ashigirl96/hail-mary/internal/kiro"
	"github.com/ashigirl96/hail-mary/internal/settings"
)

func TestSetupHookConfigWithFeature(t *testing.T) {
	logger := slog.New(slog.NewTextHandler(os.Stderr, &slog.HandlerOptions{Level: slog.LevelError}))

	tests := []struct {
		name             string
		featurePath      string
		existingSettings bool
		wantErr          bool
		checkConfig      func(t *testing.T, configPath string)
	}{
		{
			name:             "basic hook configuration without feature",
			featurePath:      "",
			existingSettings: false,
			wantErr:          false,
			checkConfig: func(t *testing.T, configPath string) {
				// Read the created config file
				data, err := os.ReadFile(configPath)
				if err != nil {
					t.Fatalf("Failed to read config file: %v", err)
				}

				var config map[string]interface{}
				if err := json.Unmarshal(data, &config); err != nil {
					t.Fatalf("Failed to parse config JSON: %v", err)
				}

				// Check hooks exist
				hooks, ok := config["hooks"].(map[string]interface{})
				if !ok {
					t.Fatal("hooks not found in config")
				}

				// Verify SessionStart hook
				if _, ok := hooks["SessionStart"]; !ok {
					t.Error("SessionStart hook not found")
				}

				// Verify UserPromptSubmit hook
				if _, ok := hooks["UserPromptSubmit"]; !ok {
					t.Error("UserPromptSubmit hook not found")
				}

				// Verify Stop hook
				if _, ok := hooks["Stop"]; !ok {
					t.Error("Stop hook not found")
				}
			},
		},
		{
			name:             "hook configuration with feature path",
			featurePath:      filepath.Join(kiro.KiroDir, kiro.SpecDir, "my-feature"),
			existingSettings: false,
			wantErr:          false,
			checkConfig: func(t *testing.T, configPath string) {
				// Read the created config file
				data, err := os.ReadFile(configPath)
				if err != nil {
					t.Fatalf("Failed to read config file: %v", err)
				}

				// Check that the hook command contains HAIL_MARY_FEATURE_PATH
				content := string(data)
				if !strings.Contains(content, fmt.Sprintf("HAIL_MARY_FEATURE_PATH=%s", filepath.Join(kiro.KiroDir, kiro.SpecDir, "my-feature"))) {
					t.Error("HAIL_MARY_FEATURE_PATH not found in hook command")
				}
			},
		},
		{
			name:             "merge with existing settings",
			featurePath:      "",
			existingSettings: true,
			wantErr:          false,
			checkConfig: func(t *testing.T, configPath string) {
				// Read the created config file
				data, err := os.ReadFile(configPath)
				if err != nil {
					t.Fatalf("Failed to read config file: %v", err)
				}

				var config map[string]interface{}
				if err := json.Unmarshal(data, &config); err != nil {
					t.Fatalf("Failed to parse config JSON: %v", err)
				}

				// Check that both existing and new hooks are present
				hooks, ok := config["hooks"].(map[string]interface{})
				if !ok {
					t.Fatal("hooks not found in config")
				}

				// Check for hail-mary hooks
				if _, ok := hooks["SessionStart"]; !ok {
					t.Error("SessionStart hook not found")
				}

				// Check for existing custom hook
				if customHook, ok := hooks["CustomHook"]; !ok {
					t.Error("CustomHook not found")
				} else {
					// Verify it contains our test hook
					customHookData, _ := json.Marshal(customHook)
					if !strings.Contains(string(customHookData), "echo 'custom hook'") {
						t.Error("Custom hook command not preserved")
					}
				}

				// Check for existing data
				if customData, ok := config["customData"]; !ok {
					t.Error("customData not found")
				} else if customData != "preserved" {
					t.Error("customData not preserved correctly")
				}
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Create a temporary directory for test
			tmpDir := t.TempDir()
			oldWd, err := os.Getwd()
			if err != nil {
				t.Fatalf("Failed to get working directory: %v", err)
			}
			if err := os.Chdir(tmpDir); err != nil {
				t.Fatalf("Failed to change directory: %v", err)
			}
			defer func() {
				if err := os.Chdir(oldWd); err != nil {
					t.Errorf("Failed to restore working directory: %v", err)
				}
			}()

			// Create existing settings if needed
			if tt.existingSettings {
				claudeDir := filepath.Join(tmpDir, ".claude")
				if err := os.MkdirAll(claudeDir, 0755); err != nil {
					t.Fatalf("Failed to create .claude directory: %v", err)
				}

				existingSettings := settings.ClaudeSettings{
					Hooks: map[string][]settings.HookMatcher{
						"CustomHook": {
							{
								Hooks: []settings.HookEntry{
									{
										Type:    "command",
										Command: "echo 'custom hook'",
									},
								},
							},
						},
					},
					Extra: map[string]interface{}{
						"customData": "preserved",
					},
				}

				existingPath := filepath.Join(claudeDir, "settings.json")
				if err := existingSettings.SaveToFile(existingPath); err != nil {
					t.Fatalf("Failed to create existing settings: %v", err)
				}
			}

			// Call the function
			configPath, cleanup, err := SetupHookConfigWithFeature(logger, tt.featurePath)
			if (err != nil) != tt.wantErr {
				t.Errorf("SetupHookConfigWithFeature() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if err != nil {
				return
			}

			// Ensure cleanup is called
			defer cleanup()

			// Check the config file was created
			if _, err := os.Stat(configPath); os.IsNotExist(err) {
				t.Fatal("Config file was not created")
			}

			// Check the config file is in temp directory
			if !strings.HasPrefix(configPath, os.TempDir()) {
				t.Errorf("Config file not in temp directory: %s", configPath)
			}

			// Run custom checks
			if tt.checkConfig != nil {
				tt.checkConfig(t, configPath)
			}

			// Test cleanup
			cleanup()
			if _, err := os.Stat(configPath); !os.IsNotExist(err) {
				t.Error("Config file was not cleaned up")
			}
		})
	}
}

// Note: Testing os.Executable error is not feasible without dependency injection
// The function is tested indirectly through other tests

func TestSetupHookConfigWithFeature_HookCommand(t *testing.T) {
	logger := slog.New(slog.NewTextHandler(os.Stderr, &slog.HandlerOptions{Level: slog.LevelError}))

	tests := []struct {
		name        string
		featurePath string
		wantEnvVars []string
	}{
		{
			name:        "without feature path",
			featurePath: "",
			wantEnvVars: []string{"HAIL_MARY_PARENT_PID="},
		},
		{
			name:        "with feature path",
			featurePath: filepath.Join(kiro.KiroDir, kiro.SpecDir, "test-feature"),
			wantEnvVars: []string{
				"HAIL_MARY_PARENT_PID=",
				fmt.Sprintf("HAIL_MARY_FEATURE_PATH=%s", filepath.Join(kiro.KiroDir, kiro.SpecDir, "test-feature")),
			},
		},
		{
			name:        "with special characters in path",
			featurePath: filepath.Join(kiro.KiroDir, kiro.SpecDir, "feature with spaces"),
			wantEnvVars: []string{
				"HAIL_MARY_PARENT_PID=",
				fmt.Sprintf("HAIL_MARY_FEATURE_PATH=%s", filepath.Join(kiro.KiroDir, kiro.SpecDir, "feature with spaces")),
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			configPath, cleanup, err := SetupHookConfigWithFeature(logger, tt.featurePath)
			if err != nil {
				t.Fatalf("SetupHookConfigWithFeature() error = %v", err)
			}
			defer cleanup()

			// Read the config file
			data, err := os.ReadFile(configPath)
			if err != nil {
				t.Fatalf("Failed to read config file: %v", err)
			}

			content := string(data)
			for _, envVar := range tt.wantEnvVars {
				if !strings.Contains(content, envVar) {
					t.Errorf("Expected env var %q not found in config", envVar)
				}
			}

			// Verify the hook command ends with "hook"
			if !strings.Contains(content, " hook") {
				t.Error("Hook command should end with 'hook'")
			}
		})
	}
}

func TestSetupHookConfigWithFeature_PIDInFilename(t *testing.T) {
	logger := slog.New(slog.NewTextHandler(os.Stderr, &slog.HandlerOptions{Level: slog.LevelError}))

	configPath, cleanup, err := SetupHookConfigWithFeature(logger, "")
	if err != nil {
		t.Fatalf("SetupHookConfigWithFeature() error = %v", err)
	}
	defer cleanup()

	// Check that the filename contains the PID
	expectedPattern := fmt.Sprintf("hail-mary-settings-%d.json", os.Getpid())
	if !strings.Contains(configPath, expectedPattern) {
		t.Errorf("Config path doesn't contain expected pattern. Got: %s, want pattern: %s", configPath, expectedPattern)
	}
}
