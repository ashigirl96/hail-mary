package claude

import (
	"encoding/json"
	"fmt"
	"io"
	"os"
	"os/exec"
	"strings"
	"testing"
)

// TestDefaultConfig tests the DefaultConfig function
func TestDefaultConfig(t *testing.T) {
	config := DefaultConfig()

	if config.Command != claudeCommand {
		t.Errorf("Command = %q, want %q", config.Command, claudeCommand)
	}
	if config.Package != claudePackage {
		t.Errorf("Package = %q, want %q", config.Package, claudePackage)
	}
	if !config.EnableBackgroundTasks {
		t.Error("EnableBackgroundTasks = false, want true")
	}
	if !config.MaintainWorkingDir {
		t.Error("MaintainWorkingDir = false, want true")
	}
	if !config.SkipPermissions {
		t.Error("SkipPermissions = false, want true")
	}
	if config.MaxPromptLength != maxPromptLength {
		t.Errorf("MaxPromptLength = %d, want %d", config.MaxPromptLength, maxPromptLength)
	}
}

// TestNewExecutor tests the NewExecutor function
func TestNewExecutor(t *testing.T) {
	executor := NewExecutor()

	if executor == nil {
		t.Fatal("NewExecutor() returned nil")
	}
	if executor.config == nil {
		t.Fatal("executor.config is nil")
	}
	if executor.config.Command != claudeCommand {
		t.Errorf("Command = %q, want %q", executor.config.Command, claudeCommand)
	}
}

// TestNewExecutorWithConfig tests the NewExecutorWithConfig function
func TestNewExecutorWithConfig(t *testing.T) {
	tests := []struct {
		name   string
		config *Config
		want   *Config
	}{
		{
			name:   "nil config uses default",
			config: nil,
			want:   DefaultConfig(),
		},
		{
			name: "custom config",
			config: &Config{
				Command:               "custom-bunx",
				Package:               "custom-package",
				EnableBackgroundTasks: false,
				MaintainWorkingDir:    false,
				SkipPermissions:       false,
				MaxPromptLength:       5000,
			},
			want: &Config{
				Command:               "custom-bunx",
				Package:               "custom-package",
				EnableBackgroundTasks: false,
				MaintainWorkingDir:    false,
				SkipPermissions:       false,
				MaxPromptLength:       5000,
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			executor := NewExecutorWithConfig(tt.config)

			if executor == nil {
				t.Fatal("NewExecutorWithConfig() returned nil")
			}
			if executor.config == nil {
				t.Fatal("executor.config is nil")
			}

			// Check configuration values
			if executor.config.Command != tt.want.Command {
				t.Errorf("Command = %q, want %q", executor.config.Command, tt.want.Command)
			}
			if executor.config.Package != tt.want.Package {
				t.Errorf("Package = %q, want %q", executor.config.Package, tt.want.Package)
			}
			if executor.config.EnableBackgroundTasks != tt.want.EnableBackgroundTasks {
				t.Errorf("EnableBackgroundTasks = %v, want %v", executor.config.EnableBackgroundTasks, tt.want.EnableBackgroundTasks)
			}
			if executor.config.MaintainWorkingDir != tt.want.MaintainWorkingDir {
				t.Errorf("MaintainWorkingDir = %v, want %v", executor.config.MaintainWorkingDir, tt.want.MaintainWorkingDir)
			}
			if executor.config.SkipPermissions != tt.want.SkipPermissions {
				t.Errorf("SkipPermissions = %v, want %v", executor.config.SkipPermissions, tt.want.SkipPermissions)
			}
			if executor.config.MaxPromptLength != tt.want.MaxPromptLength {
				t.Errorf("MaxPromptLength = %d, want %d", executor.config.MaxPromptLength, tt.want.MaxPromptLength)
			}
		})
	}
}

// TestSetSettingsPath tests the SetSettingsPath method
func TestSetSettingsPath(t *testing.T) {
	executor := NewExecutor()
	path := "/path/to/settings.json"

	executor.SetSettingsPath(path)

	if executor.config.SettingsPath != path {
		t.Errorf("SettingsPath = %q, want %q", executor.config.SettingsPath, path)
	}
}

// TestValidatePrompt tests the validatePrompt method
func TestValidatePrompt(t *testing.T) {
	executor := NewExecutor()

	tests := []struct {
		name    string
		prompt  string
		wantErr bool
		errMsg  string
	}{
		{
			name:    "valid prompt",
			prompt:  "Create a function to parse JSON",
			wantErr: false,
		},
		{
			name:    "empty prompt",
			prompt:  "",
			wantErr: true,
			errMsg:  "prompt validation failed: prompt cannot be empty or contain only whitespace",
		},
		{
			name:    "whitespace only prompt",
			prompt:  "   \t\n  ",
			wantErr: true,
			errMsg:  "prompt validation failed: prompt cannot be empty or contain only whitespace",
		},
		{
			name:    "prompt at max length",
			prompt:  strings.Repeat("a", maxPromptLength),
			wantErr: false,
		},
		{
			name:    "prompt exceeds max length",
			prompt:  strings.Repeat("a", maxPromptLength+1),
			wantErr: true,
			errMsg:  fmt.Sprintf("prompt validation failed: prompt length (%d) exceeds maximum allowed length (%d)", maxPromptLength+1, maxPromptLength),
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := executor.validatePrompt(tt.prompt)

			if tt.wantErr {
				if err == nil {
					t.Error("validatePrompt() error = nil, want error")
				} else if err.Error() != tt.errMsg {
					t.Errorf("validatePrompt() error = %q, want %q", err.Error(), tt.errMsg)
				}
			} else {
				if err != nil {
					t.Errorf("validatePrompt() error = %v, want nil", err)
				}
			}
		})
	}
}

// TestValidateSessionID tests the validateSessionID function
func TestValidateSessionID(t *testing.T) {
	tests := []struct {
		name      string
		sessionID string
		wantErr   bool
		errMsg    string
	}{
		{
			name:      "valid session ID",
			sessionID: "session123",
			wantErr:   false,
		},
		{
			name:      "valid UUID session ID",
			sessionID: "550e8400-e29b-41d4-a716-446655440000",
			wantErr:   false,
		},
		{
			name:      "empty session ID",
			sessionID: "",
			wantErr:   true,
			errMsg:    "session ID validation failed: session ID cannot be empty or contain only whitespace",
		},
		{
			name:      "whitespace only session ID",
			sessionID: "   \t\n  ",
			wantErr:   true,
			errMsg:    "session ID validation failed: session ID cannot be empty or contain only whitespace",
		},
		{
			name:      "session ID too short",
			sessionID: "short",
			wantErr:   true,
			errMsg:    fmt.Sprintf("session ID validation failed: session ID length (%d) must be between %d and %d characters", 5, minSessionIDLength, maxSessionIDLength),
		},
		{
			name:      "session ID at min length",
			sessionID: strings.Repeat("a", minSessionIDLength),
			wantErr:   false,
		},
		{
			name:      "session ID at max length",
			sessionID: strings.Repeat("a", maxSessionIDLength),
			wantErr:   false,
		},
		{
			name:      "session ID too long",
			sessionID: strings.Repeat("a", maxSessionIDLength+1),
			wantErr:   true,
			errMsg:    fmt.Sprintf("session ID validation failed: session ID length (%d) must be between %d and %d characters", maxSessionIDLength+1, minSessionIDLength, maxSessionIDLength),
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := validateSessionID(tt.sessionID)

			if tt.wantErr {
				if err == nil {
					t.Error("validateSessionID() error = nil, want error")
				} else if err.Error() != tt.errMsg {
					t.Errorf("validateSessionID() error = %q, want %q", err.Error(), tt.errMsg)
				}
			} else {
				if err != nil {
					t.Errorf("validateSessionID() error = %v, want nil", err)
				}
			}
		})
	}
}

// TestBuildCommand tests the buildCommand method
func TestBuildCommand(t *testing.T) {
	tests := []struct {
		name     string
		config   *Config
		args     []string
		wantArgs []string
		wantEnv  map[string]string
	}{
		{
			name:   "default config with no args",
			config: DefaultConfig(),
			args:   []string{},
			wantArgs: []string{
				claudePackage,
				dangerousFlag,
			},
			wantEnv: map[string]string{
				envBackgroundTasks: "1",
				envMaintainWorkDir: "1",
			},
		},
		{
			name: "config with settings path",
			config: &Config{
				Command:               claudeCommand,
				Package:               claudePackage,
				EnableBackgroundTasks: true,
				MaintainWorkingDir:    true,
				SkipPermissions:       true,
				SettingsPath:          "/path/to/settings.json",
			},
			args: []string{"--test"},
			wantArgs: []string{
				claudePackage,
				dangerousFlag,
				"--settings",
				"/path/to/settings.json",
				"--test",
			},
			wantEnv: map[string]string{
				envBackgroundTasks: "1",
				envMaintainWorkDir: "1",
			},
		},
		{
			name: "config with disabled features",
			config: &Config{
				Command:               claudeCommand,
				Package:               claudePackage,
				EnableBackgroundTasks: false,
				MaintainWorkingDir:    false,
				SkipPermissions:       false,
			},
			args: []string{"prompt"},
			wantArgs: []string{
				claudePackage,
				"prompt",
			},
			wantEnv: map[string]string{},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			executor := NewExecutorWithConfig(tt.config)
			cmd := executor.buildCommand(tt.args...)

			// Check command path (cmd.Path may be resolved to full path)
			if !strings.HasSuffix(cmd.Path, tt.config.Command) {
				t.Errorf("cmd.Path = %q, want to end with %q", cmd.Path, tt.config.Command)
			}

			// Check args
			if !stringSliceEqual(cmd.Args[1:], tt.wantArgs) {
				t.Errorf("cmd.Args = %v, want %v", cmd.Args[1:], tt.wantArgs)
			}

			// Check environment variables
			for key, wantValue := range tt.wantEnv {
				found := false
				for _, env := range cmd.Env {
					if strings.HasPrefix(env, key+"=") {
						found = true
						if env != key+"="+wantValue {
							t.Errorf("env %s = %q, want %q", key, env, key+"="+wantValue)
						}
						break
					}
				}
				if !found {
					t.Errorf("env %s not found", key)
				}
			}
		})
	}
}

// TestExecutorInterfaceCompliance verifies ExecutorImpl implements Executor interface
func TestExecutorInterfaceCompliance(t *testing.T) {
	var _ Executor = (*ExecutorImpl)(nil)
}

// Mock helper for testing command execution
type mockExecCommand struct {
	stdout   string
	stderr   string
	exitCode int
}

func (m *mockExecCommand) run(cmd *exec.Cmd) error {
	if m.stdout != "" {
		cmd.Stdout.Write([]byte(m.stdout))
	}
	if m.stderr != "" && cmd.Stderr != nil {
		cmd.Stderr.Write([]byte(m.stderr))
	}
	if m.exitCode != 0 {
		return fmt.Errorf("exit status %d", m.exitCode)
	}
	return nil
}

// TestExecuteWithSessionTracking tests the ExecuteWithSessionTracking method
func TestExecuteWithSessionTracking(t *testing.T) {
	// This test checks the method structure and error handling
	// In a real test, you would mock the exec.Command functionality
	executor := NewExecutor()

	// Test validation error
	_, err := executor.ExecuteWithSessionTracking("")
	if err == nil {
		t.Error("ExecuteWithSessionTracking(\"\") error = nil, want error")
	}
	if !strings.Contains(err.Error(), "execute with session tracking: prompt validation failed") {
		t.Errorf("ExecuteWithSessionTracking(\"\") error = %v, want prompt validation error", err)
	}
}

// TestResumeSession tests the ResumeSession method
func TestResumeSession(t *testing.T) {
	executor := NewExecutor()

	// Test session ID validation error
	_, err := executor.ResumeSession("", "test prompt")
	if err == nil {
		t.Error("ResumeSession(\"\", ...) error = nil, want error")
	}
	if !strings.Contains(err.Error(), "resume session: session ID validation failed") {
		t.Errorf("ResumeSession(\"\", ...) error = %v, want session ID validation error", err)
	}

	// Test prompt validation error
	_, err = executor.ResumeSession("valid-session-id", "")
	if err == nil {
		t.Error("ResumeSession(..., \"\") error = nil, want error")
	}
	if !strings.Contains(err.Error(), "resume session: prompt validation failed") {
		t.Errorf("ResumeSession(..., \"\") error = %v, want prompt validation error", err)
	}
}

// TestExecuteInteractive tests the ExecuteInteractive method
func TestExecuteInteractive(t *testing.T) {
	executor := NewExecutor()

	// Test validation error
	err := executor.ExecuteInteractive("")
	if err == nil {
		t.Error("ExecuteInteractive(\"\") error = nil, want error")
	}
	if !strings.Contains(err.Error(), "execute interactive: prompt validation failed") {
		t.Errorf("ExecuteInteractive(\"\") error = %v, want prompt validation error", err)
	}
}

// TestExecuteInteractiveWithSession tests the ExecuteInteractiveWithSession method
func TestExecuteInteractiveWithSession(t *testing.T) {
	executor := NewExecutor()

	// Test validation error
	err := executor.ExecuteInteractiveWithSession("")
	if err == nil {
		t.Error("ExecuteInteractiveWithSession(\"\") error = nil, want error")
	}
	if !strings.Contains(err.Error(), "execute interactive with session: session ID validation failed") {
		t.Errorf("ExecuteInteractiveWithSession(\"\") error = %v, want session ID validation error", err)
	}
}

// TestExecuteAndContinueInteractive tests the ExecuteAndContinueInteractive method
func TestExecuteAndContinueInteractive(t *testing.T) {
	executor := NewExecutor()

	// Test validation error
	_, err := executor.ExecuteAndContinueInteractive("")
	if err == nil {
		t.Error("ExecuteAndContinueInteractive(\"\") error = nil, want error")
	}
	if !strings.Contains(err.Error(), "execute and continue interactive: prompt validation failed") {
		t.Errorf("ExecuteAndContinueInteractive(\"\") error = %v, want prompt validation error", err)
	}
}

// TestSessionInfoJSON tests the SessionInfo JSON marshaling/unmarshaling
func TestSessionInfoJSON(t *testing.T) {
	info := &SessionInfo{
		ID:       "test-session-123",
		Result:   "Test result",
		CostUSD:  0.05,
		Duration: "2m30s",
		Turns:    3,
	}

	// Marshal to JSON
	data, err := json.Marshal(info)
	if err != nil {
		t.Fatalf("json.Marshal() error = %v", err)
	}

	// Unmarshal back
	var decoded SessionInfo
	if err := json.Unmarshal(data, &decoded); err != nil {
		t.Fatalf("json.Unmarshal() error = %v", err)
	}

	// Compare fields
	if decoded.ID != info.ID {
		t.Errorf("ID = %q, want %q", decoded.ID, info.ID)
	}
	if decoded.Result != info.Result {
		t.Errorf("Result = %q, want %q", decoded.Result, info.Result)
	}
	if decoded.CostUSD != info.CostUSD {
		t.Errorf("CostUSD = %f, want %f", decoded.CostUSD, info.CostUSD)
	}
	if decoded.Duration != info.Duration {
		t.Errorf("Duration = %q, want %q", decoded.Duration, info.Duration)
	}
	if decoded.Turns != info.Turns {
		t.Errorf("Turns = %d, want %d", decoded.Turns, info.Turns)
	}
}

// Helper function to compare string slices
func stringSliceEqual(a, b []string) bool {
	if len(a) != len(b) {
		return false
	}
	for i, v := range a {
		if v != b[i] {
			return false
		}
	}
	return true
}

// TestExecuteInteractiveContinue tests the ExecuteInteractiveContinue method
func TestExecuteInteractiveContinue(t *testing.T) {
	executor := NewExecutor()

	// This will fail since we don't have Claude CLI installed,
	// but we expect it to build the command correctly and fail on execution
	err := executor.ExecuteInteractiveContinue()
	if err == nil {
		t.Error("Expected error when Claude CLI is not available")
	}

	if !strings.Contains(err.Error(), "execute interactive continue") {
		t.Errorf("Expected error to contain 'execute interactive continue', got: %v", err)
	}
}

// TestBuildCommandEnvironment tests that buildCommand properly sets environment variables
func TestBuildCommandEnvironment(t *testing.T) {
	// Save and clean original environment for these specific vars
	origBackgroundTasks := os.Getenv("ENABLE_BACKGROUND_TASKS")
	origMaintainWorkDir := os.Getenv("CLAUDE_BASH_MAINTAIN_PROJECT_WORKING_DIR")

	// Clean these vars from environment for testing
	os.Unsetenv("ENABLE_BACKGROUND_TASKS")
	os.Unsetenv("CLAUDE_BASH_MAINTAIN_PROJECT_WORKING_DIR")

	defer func() {
		// Restore original values
		if origBackgroundTasks != "" {
			os.Setenv("ENABLE_BACKGROUND_TASKS", origBackgroundTasks)
		}
		if origMaintainWorkDir != "" {
			os.Setenv("CLAUDE_BASH_MAINTAIN_PROJECT_WORKING_DIR", origMaintainWorkDir)
		}
	}()

	tests := []struct {
		name                  string
		enableBackgroundTasks bool
		maintainWorkingDir    bool
		wantBackgroundTasks   bool
		wantMaintainWorkDir   bool
	}{
		{
			name:                  "both enabled",
			enableBackgroundTasks: true,
			maintainWorkingDir:    true,
			wantBackgroundTasks:   true,
			wantMaintainWorkDir:   true,
		},
		{
			name:                  "both disabled",
			enableBackgroundTasks: false,
			maintainWorkingDir:    false,
			wantBackgroundTasks:   false,
			wantMaintainWorkDir:   false,
		},
		{
			name:                  "background tasks only",
			enableBackgroundTasks: true,
			maintainWorkingDir:    false,
			wantBackgroundTasks:   true,
			wantMaintainWorkDir:   false,
		},
		{
			name:                  "working dir only",
			enableBackgroundTasks: false,
			maintainWorkingDir:    true,
			wantBackgroundTasks:   false,
			wantMaintainWorkDir:   true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			config := &Config{
				Command:               claudeCommand,
				Package:               claudePackage,
				EnableBackgroundTasks: tt.enableBackgroundTasks,
				MaintainWorkingDir:    tt.maintainWorkingDir,
			}
			executor := NewExecutorWithConfig(config)
			cmd := executor.buildCommand("test")

			// Check environment variables
			hasBackgroundTasks := false
			hasMaintainWorkDir := false

			for _, env := range cmd.Env {
				// Check only for the specific env vars we care about
				parts := strings.SplitN(env, "=", 2)
				if len(parts) == 2 {
					switch parts[0] {
					case "ENABLE_BACKGROUND_TASKS":
						hasBackgroundTasks = true
						if parts[1] != "1" {
							t.Errorf("%s = %q, want %q", parts[0], parts[1], "1")
						}
					case "CLAUDE_BASH_MAINTAIN_PROJECT_WORKING_DIR":
						hasMaintainWorkDir = true
						if parts[1] != "1" {
							t.Errorf("%s = %q, want %q", parts[0], parts[1], "1")
						}
					}
				}
			}

			if hasBackgroundTasks != tt.wantBackgroundTasks {
				t.Errorf("hasBackgroundTasks = %v, want %v", hasBackgroundTasks, tt.wantBackgroundTasks)
			}
			if hasMaintainWorkDir != tt.wantMaintainWorkDir {
				t.Errorf("hasMaintainWorkDir = %v, want %v", hasMaintainWorkDir, tt.wantMaintainWorkDir)
			}

			// Verify environment has expected content
			// The environment should have at least the same number of vars as current environment
			// (after our unsetenv calls), plus any we explicitly added
			currentEnvCount := len(os.Environ())
			expectedAdditional := 0
			if tt.wantBackgroundTasks {
				expectedAdditional++
			}
			if tt.wantMaintainWorkDir {
				expectedAdditional++
			}

			if len(cmd.Env) < currentEnvCount {
				t.Errorf("Command environment has %d vars, expected at least %d", len(cmd.Env), currentEnvCount)
			}
		})
	}
}

// TestExecutorMethodsIntegration tests that all methods properly handle errors
func TestExecutorMethodsIntegration(t *testing.T) {
	executor := NewExecutor()

	// Test that all methods validate their inputs
	tests := []struct {
		name string
		fn   func() error
		want string
	}{
		{
			name: "ExecuteInteractive with empty prompt",
			fn: func() error {
				return executor.ExecuteInteractive("")
			},
			want: "prompt validation failed",
		},
		{
			name: "ExecuteInteractiveWithSession with empty session",
			fn: func() error {
				return executor.ExecuteInteractiveWithSession("")
			},
			want: "session ID validation failed",
		},
		{
			name: "ExecuteAndContinueInteractive with empty prompt",
			fn: func() error {
				_, err := executor.ExecuteAndContinueInteractive("")
				return err
			},
			want: "prompt validation failed",
		},
		{
			name: "ExecuteWithSessionTracking with empty prompt",
			fn: func() error {
				_, err := executor.ExecuteWithSessionTracking("")
				return err
			},
			want: "prompt validation failed",
		},
		{
			name: "ResumeSession with empty session",
			fn: func() error {
				_, err := executor.ResumeSession("", "test")
				return err
			},
			want: "session ID validation failed",
		},
		{
			name: "ResumeSession with empty prompt",
			fn: func() error {
				_, err := executor.ResumeSession("valid-session", "")
				return err
			},
			want: "prompt validation failed",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.fn()
			if err == nil {
				t.Error("expected error, got nil")
			} else if !strings.Contains(err.Error(), tt.want) {
				t.Errorf("error = %q, want to contain %q", err.Error(), tt.want)
			}
		})
	}
}

// TestCommandOutput tests handling of command output
func TestCommandOutput(t *testing.T) {
	executor := NewExecutor()

	// Test JSON parsing in ExecuteWithSessionTracking
	// This is a structural test to ensure proper error handling
	cmd := executor.buildCommand(printFlag, outputJSONFlag, "test")

	// Verify command has correct flags
	expectedFlags := []string{claudePackage, dangerousFlag, printFlag, outputJSONFlag, "test"}
	if !stringSliceEqual(cmd.Args[1:], expectedFlags) {
		t.Errorf("cmd.Args = %v, want %v", cmd.Args[1:], expectedFlags)
	}

	// Test that Stdout is set to capture output
	if cmd.Stdout != nil {
		t.Error("cmd.Stdout should be nil initially for Output() to work")
	}
}

// TestConfigEdgeCases tests edge cases in configuration
func TestConfigEdgeCases(t *testing.T) {
	// Test with zero-value config
	config := &Config{}
	executor := NewExecutorWithConfig(config)

	if executor.config.Command != "" {
		t.Errorf("Command = %q, want empty string", executor.config.Command)
	}
	if executor.config.Package != "" {
		t.Errorf("Package = %q, want empty string", executor.config.Package)
	}
	if executor.config.MaxPromptLength != 0 {
		t.Errorf("MaxPromptLength = %d, want 0", executor.config.MaxPromptLength)
	}

	// Test validation with zero MaxPromptLength
	err := executor.validatePrompt("any prompt")
	if err == nil {
		t.Error("validatePrompt() with MaxPromptLength=0 should fail")
	}
}

// TestExecuteMethodsCoverage tests execution methods more thoroughly
func TestExecuteMethodsCoverage(t *testing.T) {
	executor := NewExecutor()

	t.Run("ExecuteInteractive full flow", func(t *testing.T) {
		// ExecuteInteractive with valid prompt - will fail at command execution
		err := executor.ExecuteInteractive("test prompt")
		if err == nil {
			t.Error("Expected error when Claude CLI is not available")
		}
		// Should fail in ExecuteWithSessionTracking
		if !strings.Contains(err.Error(), "execute interactive") {
			t.Errorf("Expected error from ExecuteInteractive, got: %v", err)
		}
	})

	t.Run("ExecuteWithSessionTracking with command failure", func(t *testing.T) {
		// This tests the actual command execution path
		sessionInfo, err := executor.ExecuteWithSessionTracking("test prompt")
		if err == nil {
			t.Error("Expected error when Claude CLI is not available")
		}
		if sessionInfo != nil {
			t.Error("Expected nil session info on error")
		}
		if !strings.Contains(err.Error(), "execute with session tracking") {
			t.Errorf("Expected error from ExecuteWithSessionTracking, got: %v", err)
		}
	})

	t.Run("ResumeSession with command failure", func(t *testing.T) {
		// This tests the resume session command execution
		sessionInfo, err := executor.ResumeSession("test-session-123", "continue prompt")
		if err == nil {
			t.Error("Expected error when Claude CLI is not available")
		}
		if sessionInfo != nil {
			t.Error("Expected nil session info on error")
		}
		if !strings.Contains(err.Error(), "resume session") {
			t.Errorf("Expected error from ResumeSession, got: %v", err)
		}
	})

	t.Run("ExecuteInteractiveWithSession with command failure", func(t *testing.T) {
		// This tests interactive session execution
		err := executor.ExecuteInteractiveWithSession("test-session-123")
		if err == nil {
			t.Error("Expected error when Claude CLI is not available")
		}
		if !strings.Contains(err.Error(), "execute interactive with session") {
			t.Errorf("Expected error from ExecuteInteractiveWithSession, got: %v", err)
		}
	})

	t.Run("ExecuteAndContinueInteractive full flow", func(t *testing.T) {
		// This tests the combined execution and continue
		sessionInfo, err := executor.ExecuteAndContinueInteractive("test prompt")
		if err == nil {
			t.Error("Expected error when Claude CLI is not available")
		}
		if sessionInfo != nil {
			t.Error("Expected nil session info on error")
		}
		// Should fail in ExecuteWithSessionTracking
		if !strings.Contains(err.Error(), "execute and continue interactive") {
			t.Errorf("Expected error from ExecuteAndContinueInteractive, got: %v", err)
		}
	})
}

// TestExecuteMethodsWithMockOutput tests the execution methods with mocked command output
func TestExecuteMethodsWithMockOutput(t *testing.T) {
	// Get path to mock executable
	mockPath := "./testdata/mock_claude"

	// Check if mock executable exists
	if _, err := os.Stat(mockPath); os.IsNotExist(err) {
		t.Skip("Mock executable not found, skipping mock output tests")
	}

	t.Run("ExecuteWithSessionTracking success", func(t *testing.T) {
		config := &Config{
			Command:         mockPath,
			Package:         "", // Mock doesn't need package
			SkipPermissions: false,
			MaxPromptLength: maxPromptLength,
		}
		executor := NewExecutorWithConfig(config)

		sessionInfo, err := executor.ExecuteWithSessionTracking("test prompt")
		if err != nil {
			t.Fatalf("ExecuteWithSessionTracking failed: %v", err)
		}

		if sessionInfo.ID != "test-session-123" {
			t.Errorf("expected session ID 'test-session-123', got %s", sessionInfo.ID)
		}
		if sessionInfo.Result != "Test response" {
			t.Errorf("expected result 'Test response', got %s", sessionInfo.Result)
		}
		if sessionInfo.CostUSD != 0.01 {
			t.Errorf("expected cost 0.01, got %f", sessionInfo.CostUSD)
		}
	})

	t.Run("ResumeSession success", func(t *testing.T) {
		config := &Config{
			Command:         mockPath,
			Package:         "",
			SkipPermissions: false,
			MaxPromptLength: maxPromptLength,
		}
		executor := NewExecutorWithConfig(config)

		sessionInfo, err := executor.ResumeSession("session-456", "continue prompt")
		if err != nil {
			t.Fatalf("ResumeSession failed: %v", err)
		}

		if sessionInfo.ID != "session-456" {
			t.Errorf("expected session ID 'session-456', got %s", sessionInfo.ID)
		}
		if sessionInfo.Result != "Resumed successfully" {
			t.Errorf("expected result 'Resumed successfully', got %s", sessionInfo.Result)
		}
		if sessionInfo.CostUSD != 0.02 {
			t.Errorf("expected cost 0.02, got %f", sessionInfo.CostUSD)
		}
	})

	t.Run("ExecuteInteractive success", func(t *testing.T) {
		config := &Config{
			Command:         mockPath,
			Package:         "",
			SkipPermissions: false,
			MaxPromptLength: maxPromptLength,
		}
		executor := NewExecutorWithConfig(config)

		// Capture stdout
		old := os.Stdout
		r, w, _ := os.Pipe()
		os.Stdout = w

		err := executor.ExecuteInteractive("test prompt")

		// Restore stdout
		w.Close()
		os.Stdout = old

		if err != nil {
			t.Fatalf("ExecuteInteractive failed: %v", err)
		}

		// Read captured output
		out, _ := io.ReadAll(r)
		output := string(out)

		if !strings.Contains(output, "Session initialized") {
			t.Error("expected output to contain 'Session initialized'")
		}
		if !strings.Contains(output, "test-session-123") {
			t.Error("expected output to contain session ID")
		}
	})

	t.Run("ExecuteInteractiveContinue success", func(t *testing.T) {
		config := &Config{
			Command:         mockPath,
			Package:         "",
			SkipPermissions: false,
			MaxPromptLength: maxPromptLength,
		}
		executor := NewExecutorWithConfig(config)

		err := executor.ExecuteInteractiveContinue()
		if err != nil {
			t.Fatalf("ExecuteInteractiveContinue failed: %v", err)
		}
	})

	t.Run("ExecuteInteractiveWithSession success", func(t *testing.T) {
		config := &Config{
			Command:         mockPath,
			Package:         "",
			SkipPermissions: false,
			MaxPromptLength: maxPromptLength,
		}
		executor := NewExecutorWithConfig(config)

		err := executor.ExecuteInteractiveWithSession("session-789")
		if err != nil {
			t.Fatalf("ExecuteInteractiveWithSession failed: %v", err)
		}
	})

	t.Run("ExecuteAndContinueInteractive success", func(t *testing.T) {
		config := &Config{
			Command:         mockPath,
			Package:         "",
			SkipPermissions: false,
			MaxPromptLength: maxPromptLength,
		}
		executor := NewExecutorWithConfig(config)

		// Capture stdout
		old := os.Stdout
		r, w, _ := os.Pipe()
		os.Stdout = w

		sessionInfo, err := executor.ExecuteAndContinueInteractive("test prompt")

		// Restore stdout
		w.Close()
		os.Stdout = old

		if err != nil {
			t.Fatalf("ExecuteAndContinueInteractive failed: %v", err)
		}

		if sessionInfo.ID != "test-session-123" {
			t.Errorf("expected session ID 'test-session-123', got %s", sessionInfo.ID)
		}

		// Read captured output
		out, _ := io.ReadAll(r)
		output := string(out)

		if !strings.Contains(output, "Session initialized") {
			t.Error("expected output to contain 'Session initialized'")
		}
	})
}

// Benchmark tests
func BenchmarkValidatePrompt(b *testing.B) {
	executor := NewExecutor()
	prompt := strings.Repeat("test prompt ", 100)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = executor.validatePrompt(prompt)
	}
}

func BenchmarkValidateSessionID(b *testing.B) {
	sessionID := "550e8400-e29b-41d4-a716-446655440000"

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = validateSessionID(sessionID)
	}
}

func BenchmarkBuildCommand(b *testing.B) {
	executor := NewExecutor()

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = executor.buildCommand("test", "args", "here")
	}
}
