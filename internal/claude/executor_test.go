package claude

import (
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestNewExecutor(t *testing.T) {
	// Act
	executor := NewExecutor()

	// Assert
	require.NotNil(t, executor, "NewExecutor should not return nil")
	
	// Verify it implements the interface
	var _ Executor = (Executor)(executor)
}

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
			name: "custom config is preserved",
			config: &Config{
				Command:               "custom-bunx",
				Package:               "custom-package",
				EnableBackgroundTasks: false,
				MaintainWorkingDir:    false,
				SkipPermissions:       false,
				MaxPromptLength:       5000,
				SettingsPath:          "/custom/path",
			},
			want: &Config{
				Command:               "custom-bunx",
				Package:               "custom-package",
				EnableBackgroundTasks: false,
				MaintainWorkingDir:    false,
				SkipPermissions:       false,
				MaxPromptLength:       5000,
				SettingsPath:          "/custom/path",
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Act
			executor := NewExecutorWithConfig(tt.config)

			// Assert
			require.NotNil(t, executor, "NewExecutorWithConfig should not return nil")
			
			impl := executor
			
			assert.Equal(t, tt.want.Command, impl.config.Command)
			assert.Equal(t, tt.want.Package, impl.config.Package)
			assert.Equal(t, tt.want.EnableBackgroundTasks, impl.config.EnableBackgroundTasks)
			assert.Equal(t, tt.want.MaintainWorkingDir, impl.config.MaintainWorkingDir)
			assert.Equal(t, tt.want.SkipPermissions, impl.config.SkipPermissions)
			assert.Equal(t, tt.want.MaxPromptLength, impl.config.MaxPromptLength)
			assert.Equal(t, tt.want.SettingsPath, impl.config.SettingsPath)
		})
	}
}

func TestExecutorImpl_SetSettingsPath(t *testing.T) {
	// Arrange
	executor := NewExecutor()
	expectedPath := "/path/to/settings.json"

	// Act
	executor.SetSettingsPath(expectedPath)

	// Assert
	impl := executor
	assert.Equal(t, expectedPath, impl.config.SettingsPath, "SettingsPath should be updated")
}

func TestValidatePrompt(t *testing.T) {
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
			errMsg:  "prompt cannot be empty",
		},
		{
			name:    "whitespace only prompt",
			prompt:  "   \t\n  ",
			wantErr: true,
			errMsg:  "prompt cannot be empty",
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
			errMsg:  "exceeds maximum allowed length",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Arrange
			executor := NewExecutor()

			// Act
			err := executor.validatePrompt(tt.prompt)

			// Assert
			if tt.wantErr {
				require.Error(t, err)
				assert.Contains(t, err.Error(), tt.errMsg)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func TestValidateSessionID(t *testing.T) {
	tests := []struct {
		name      string
		sessionID string
		wantErr   bool
		errMsg    string
	}{
		{
			name:      "valid session ID",
			sessionID: "session-123",
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
			errMsg:    "session ID cannot be empty",
		},
		{
			name:      "whitespace only session ID",
			sessionID: "   \t\n  ",
			wantErr:   true,
			errMsg:    "session ID cannot be empty",
		},
		{
			name:      "session ID too short",
			sessionID: "short",
			wantErr:   true,
			errMsg:    "session ID length",
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
			errMsg:    "session ID length",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Act
			err := validateSessionID(tt.sessionID)

			// Assert
			if tt.wantErr {
				require.Error(t, err)
				assert.Contains(t, err.Error(), tt.errMsg)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func TestExecutorImpl_BuildCommand(t *testing.T) {
	tests := []struct {
		name     string
		config   *Config
		args     []string
		wantArgs []string
		checkEnv map[string]string
	}{
		{
			name:   "default config with no args",
			config: DefaultConfig(),
			args:   []string{},
			wantArgs: []string{
				claudePackage,
				dangerousFlag,
			},
			checkEnv: map[string]string{
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
			checkEnv: map[string]string{
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
			checkEnv: map[string]string{},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Arrange
			executor := NewExecutorWithConfig(tt.config)

			// Act
			cmd := executor.buildCommand(tt.args...)

			// Assert
			assert.True(t, strings.HasSuffix(cmd.Path, tt.config.Command), 
				"Command path should end with %s, got %s", tt.config.Command, cmd.Path)
			
			// Skip the first arg (command name) and check the rest
			assert.Equal(t, tt.wantArgs, cmd.Args[1:], "Command arguments should match expected")

			// Check environment variables
			for key, expectedValue := range tt.checkEnv {
				found := false
				for _, env := range cmd.Env {
					if strings.HasPrefix(env, key+"=") {
						found = true
						assert.Equal(t, key+"="+expectedValue, env, "Environment variable should match")
						break
					}
				}
				assert.True(t, found, "Environment variable %s should be present", key)
			}
		})
	}
}

// Contract test to ensure ExecutorImpl implements Executor interface
func TestExecutorImpl_ImplementsInterface(t *testing.T) {
	var _ Executor = (*ExecutorImpl)(nil)
}

// Test that validates all interface methods exist and have correct signatures
func TestExecutorImpl_InterfaceCompliance(t *testing.T) {
	executor := NewExecutor()
	
	// Test that all interface methods can be called (they will fail due to no claude CLI, but should compile)
	tests := []struct {
		name string
		fn   func() error
	}{
		{
			name: "ExecuteInteractive",
			fn: func() error {
				return executor.ExecuteInteractive("test")
			},
		},
		{
			name: "ExecuteInteractiveContinue",
			fn: func() error {
				return executor.ExecuteInteractiveContinue()
			},
		},
		{
			name: "ExecuteInteractiveWithSession",
			fn: func() error {
				return executor.ExecuteInteractiveWithSession("test-session")
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// These will fail since we don't have claude CLI available,
			// but they should not panic and should return an error
			err := tt.fn()
			assert.Error(t, err, "Should return error when claude CLI is not available")
		})
	}
	
	// Test methods that return values
	t.Run("ExecuteWithSessionTracking", func(t *testing.T) {
		result, err := executor.ExecuteWithSessionTracking("test")
		assert.Error(t, err, "Should return error when claude CLI is not available")
		assert.Nil(t, result, "Should return nil result on error")
	})
	
	t.Run("ResumeSession", func(t *testing.T) {
		result, err := executor.ResumeSession("test-session", "test")
		assert.Error(t, err, "Should return error when claude CLI is not available")
		assert.Nil(t, result, "Should return nil result on error")
	})
	
	t.Run("ExecuteAndContinueInteractive", func(t *testing.T) {
		result, err := executor.ExecuteAndContinueInteractive("test")
		assert.Error(t, err, "Should return error when claude CLI is not available")
		assert.Nil(t, result, "Should return nil result on error")
	})
}

func TestExecutorImpl_ValidationInPublicMethods(t *testing.T) {
	executor := NewExecutor()

	t.Run("ExecuteInteractive validates prompt", func(t *testing.T) {
		err := executor.ExecuteInteractive("")
		require.Error(t, err)
		assert.Contains(t, err.Error(), "prompt validation failed")
	})

	t.Run("ExecuteWithSessionTracking validates prompt", func(t *testing.T) {
		_, err := executor.ExecuteWithSessionTracking("")
		require.Error(t, err)
		assert.Contains(t, err.Error(), "prompt validation failed")
	})

	t.Run("ResumeSession validates session ID", func(t *testing.T) {
		_, err := executor.ResumeSession("", "valid prompt")
		require.Error(t, err)
		assert.Contains(t, err.Error(), "session ID validation failed")
	})

	t.Run("ResumeSession validates prompt", func(t *testing.T) {
		_, err := executor.ResumeSession("valid-session-id", "")
		require.Error(t, err)
		assert.Contains(t, err.Error(), "prompt validation failed")
	})

	t.Run("ExecuteInteractiveWithSession validates session ID", func(t *testing.T) {
		err := executor.ExecuteInteractiveWithSession("")
		require.Error(t, err)
		assert.Contains(t, err.Error(), "session ID validation failed")
	})

	t.Run("ExecuteAndContinueInteractive validates prompt", func(t *testing.T) {
		_, err := executor.ExecuteAndContinueInteractive("")
		require.Error(t, err)
		assert.Contains(t, err.Error(), "prompt validation failed")
	})
}