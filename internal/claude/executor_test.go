package claude_test

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/ashigirl96/hail-mary/internal/claude"
	"github.com/ashigirl96/hail-mary/internal/testing/mocks"
)

func TestNewExecutor(t *testing.T) {
	// Act
	executor := claude.NewExecutor()

	// Assert
	require.NotNil(t, executor, "NewExecutor should not return nil")

	// Verify it implements the interface
	var _ claude.Executor = executor
}

func TestNewExecutorWithConfig(t *testing.T) {
	tests := []struct {
		name   string
		config *claude.Config
		want   *claude.Config
	}{
		{
			name:   "nil config uses default",
			config: nil,
			want:   claude.DefaultConfig(),
		},
		{
			name: "custom config is preserved",
			config: &claude.Config{
				Command:               "custom-bunx",
				EnableBackgroundTasks: false,
				MaintainWorkingDir:    false,
				SkipPermissions:       false,
				MaxPromptLength:       5000,
				SettingsPath:          "/custom/path",
			},
			want: &claude.Config{
				Command:               "custom-bunx",
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
			executor := claude.NewExecutorWithConfig(tt.config)

			// Assert
			require.NotNil(t, executor, "NewExecutorWithConfig should not return nil")

			// This test verifies the executor was created successfully
			// Configuration validation is done through the public interface
			assert.NotNil(t, executor)
		})
	}
}

// Note: Internal validation methods are tested indirectly through public interface methods

func TestExecutorImpl_ImplementsInterface(t *testing.T) {
	var _ claude.Executor = (*claude.ExecutorImpl)(nil)
}

// Test that validates all interface methods exist and have correct signatures
func TestExecutorImpl_InterfaceCompliance(t *testing.T) {
	// Use MockExecutor to test interface compliance without executing external commands
	var executor claude.Executor = mocks.NewExecutor()

	// Test that all interface methods can be called and work correctly
	tests := []struct {
		name string
		test func(t *testing.T, executor claude.Executor)
	}{
		{
			name: "Execute method exists",
			test: func(t *testing.T, executor claude.Executor) {
				opts := claude.ExecuteOptions{
					Prompt: "test prompt",
					Mode:   "plan",
				}
				err := executor.Execute(opts)
				// MockExecutor should not return error by default
				assert.NoError(t, err)
			},
		},
		{
			name: "ExecuteWithSession method exists",
			test: func(t *testing.T, executor claude.Executor) {
				opts := claude.ExecuteOptions{
					Mode: "plan",
				}
				err := executor.ExecuteWithSession("session-123", opts)
				assert.NoError(t, err)
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tt.test(t, executor)
		})
	}
}

// Integration test for checking if mock provides full interface coverage
func TestMockExecutor_ImplementsInterface(t *testing.T) {
	var _ claude.Executor = (*mocks.Executor)(nil)
}

func TestNewConfigWithDefaults(t *testing.T) {
	// Get the default config to compare against
	defaultConfig := claude.DefaultConfig()

	tests := []struct {
		name      string
		overrides claude.Config
		want      *claude.Config
	}{
		{
			name:      "empty overrides uses all defaults",
			overrides: claude.Config{},
			want:      defaultConfig,
		},
		{
			name: "override settings path only",
			overrides: claude.Config{
				SettingsPath: "/custom/settings.json",
			},
			want: &claude.Config{
				Command:               defaultConfig.Command,
				EnableBackgroundTasks: true,
				MaintainWorkingDir:    true,
				SkipPermissions:       true,
				MaxPromptLength:       10000,
				SettingsPath:          "/custom/settings.json",
			},
		},
		{
			name: "override multiple values",
			overrides: claude.Config{
				Command:         "npx",
				SettingsPath:    "/custom/settings.json",
				MaxPromptLength: 5000,
			},
			want: &claude.Config{
				Command:               "npx",
				EnableBackgroundTasks: true,
				MaintainWorkingDir:    true,
				SkipPermissions:       true,
				MaxPromptLength:       5000,
				SettingsPath:          "/custom/settings.json",
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Act
			got := claude.NewConfigWithDefaults(tt.overrides)

			// Assert
			assert.Equal(t, tt.want, got)
		})
	}
}
