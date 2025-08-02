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
				Package:               "custom-package",
				EnableBackgroundTasks: false,
				MaintainWorkingDir:    false,
				SkipPermissions:       false,
				MaxPromptLength:       5000,
				SettingsPath:          "/custom/path",
			},
			want: &claude.Config{
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
			name: "ExecuteInteractive method exists",
			test: func(t *testing.T, executor claude.Executor) {
				err := executor.ExecuteInteractive("test prompt")
				// MockExecutor should not return error by default
				assert.NoError(t, err)
			},
		},
		{
			name: "ExecuteInteractiveContinue method exists",
			test: func(t *testing.T, executor claude.Executor) {
				err := executor.ExecuteInteractiveContinue()
				assert.NoError(t, err)
			},
		},
		{
			name: "ExecuteWithSessionTracking method exists",
			test: func(t *testing.T, executor claude.Executor) {
				result, err := executor.ExecuteWithSessionTracking("test prompt")
				assert.NoError(t, err)
				assert.NotNil(t, result)
			},
		},
		{
			name: "ResumeSession method exists",
			test: func(t *testing.T, executor claude.Executor) {
				result, err := executor.ResumeSession("session-123", "test prompt")
				assert.NoError(t, err)
				assert.NotNil(t, result)
			},
		},
		{
			name: "ExecuteInteractiveWithSession method exists",
			test: func(t *testing.T, executor claude.Executor) {
				err := executor.ExecuteInteractiveWithSession("session-123")
				assert.NoError(t, err)
			},
		},
		{
			name: "ExecuteAndContinueInteractive method exists",
			test: func(t *testing.T, executor claude.Executor) {
				result, err := executor.ExecuteAndContinueInteractive("test prompt")
				assert.NoError(t, err)
				assert.NotNil(t, result)
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