package mocks

import (
	"errors"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/ashigirl96/hail-mary/internal/claude"
)

func TestNewExecutor(t *testing.T) {
	// Act
	mock := NewExecutor()

	// Assert
	require.NotNil(t, mock, "NewExecutor should not return nil")
	assert.Empty(t, mock.CallLog, "CallLog should be empty initially")
	assert.False(t, mock.ShouldFail, "ShouldFail should be false by default")
	assert.Nil(t, mock.ExecuteError, "ExecuteError should be nil by default")
}

func TestExecutor_ImplementsInterface(t *testing.T) {
	// Contract test
	var _ claude.Executor = (*Executor)(nil)
}

func TestExecutor_Execute_Success(t *testing.T) {
	// Arrange
	mock := NewExecutor()
	opts := claude.ExecuteOptions{
		Prompt:       "Create a function",
		Mode:         "plan",
		SystemPrompt: "Be helpful",
	}

	// Act
	err := mock.Execute(opts)

	// Assert
	require.NoError(t, err)
	assert.Len(t, mock.CallLog, 1, "Should record one call")
	assert.Equal(t, "Execute", mock.CallLog[0].Method)
	assert.Equal(t, opts, mock.CallLog[0].Args[0])
}

func TestExecutor_Execute_Failure(t *testing.T) {
	// Arrange
	mock := NewExecutor()
	expectedError := errors.New("execution failed")
	mock.SetupFailure(expectedError)
	opts := claude.ExecuteOptions{Prompt: "test"}

	// Act
	err := mock.Execute(opts)

	// Assert
	require.Error(t, err)
	assert.Equal(t, expectedError, err)
	assert.Len(t, mock.CallLog, 1, "Should record the call even on failure")
}

func TestExecutor_ExecuteWithSession(t *testing.T) {
	// Arrange
	mock := NewExecutor()
	sessionID := "session-789"
	opts := claude.ExecuteOptions{
		Mode:         "plan",
		SystemPrompt: "Continue conversation",
	}

	// Act
	err := mock.ExecuteWithSession(sessionID, opts)

	// Assert
	require.NoError(t, err)
	assert.Len(t, mock.CallLog, 1)
	assert.Equal(t, "ExecuteWithSession", mock.CallLog[0].Method)
	assert.Equal(t, sessionID, mock.CallLog[0].Args[0])
	assert.Equal(t, opts, mock.CallLog[0].Args[1])
}

func TestExecutor_GetCallCount(t *testing.T) {
	// Arrange
	mock := NewExecutor()

	// Act
	_ = mock.Execute(claude.ExecuteOptions{Prompt: "test1"})
	_ = mock.Execute(claude.ExecuteOptions{Prompt: "test2"})
	_ = mock.ExecuteWithSession("session-test", claude.ExecuteOptions{})

	// Assert
	assert.Equal(t, 2, mock.GetCallCount("Execute"))
	assert.Equal(t, 1, mock.GetCallCount("ExecuteWithSession"))
}

func TestExecutor_GetLastCall(t *testing.T) {
	// Arrange
	mock := NewExecutor()

	// Act
	_ = mock.Execute(claude.ExecuteOptions{Prompt: "first"})
	_ = mock.Execute(claude.ExecuteOptions{Prompt: "second"})
	_ = mock.ExecuteWithSession("session-test", claude.ExecuteOptions{Mode: "plan"})

	// Assert
	lastExecute := mock.GetLastCall("Execute")
	require.NotNil(t, lastExecute)
	assert.Equal(t, "Execute", lastExecute.Method)
	opts := lastExecute.Args[0].(claude.ExecuteOptions)
	assert.Equal(t, "second", opts.Prompt)

	lastSession := mock.GetLastCall("ExecuteWithSession")
	require.NotNil(t, lastSession)
	assert.Equal(t, "ExecuteWithSession", lastSession.Method)

	nonExistent := mock.GetLastCall("NonExistentMethod")
	assert.Nil(t, nonExistent)
}

func TestExecutor_Reset(t *testing.T) {
	// Arrange
	mock := NewExecutor()
	mock.SetupFailure(errors.New("error"))
	_ = mock.Execute(claude.ExecuteOptions{Prompt: "test"})

	// Act
	mock.Reset()

	// Assert
	assert.Empty(t, mock.CallLog, "CallLog should be empty after reset")
	assert.False(t, mock.ShouldFail, "ShouldFail should be false after reset")

	// Verify behavior is reset
	err := mock.Execute(claude.ExecuteOptions{Prompt: "after reset"})
	assert.NoError(t, err, "Should not fail after reset")
}

func TestExecutor_SetupFailure(t *testing.T) {
	// Arrange
	mock := NewExecutor()
	expectedError := errors.New("test error")

	// Act
	mock.SetupFailure(expectedError)

	// Assert
	assert.True(t, mock.ShouldFail)
	assert.Equal(t, expectedError, mock.ExecuteError)

	// Verify failure behavior
	err := mock.Execute(claude.ExecuteOptions{})
	assert.Equal(t, expectedError, err)
}

func TestExecutor_CallTimestamps(t *testing.T) {
	// Arrange
	mock := NewExecutor()

	// Act
	timeBefore := time.Now()
	time.Sleep(10 * time.Millisecond) // Small delay to ensure time difference
	_ = mock.Execute(claude.ExecuteOptions{Prompt: "test"})
	timeAfter := time.Now()

	// Assert
	assert.Len(t, mock.CallLog, 1)
	callTime := mock.CallLog[0].Time
	assert.True(t, callTime.After(timeBefore), "Call time should be after start time")
	assert.True(t, callTime.Before(timeAfter), "Call time should be before end time")
}

func TestExecutor_ComplexScenario(t *testing.T) {
	// This test simulates a more complex scenario with multiple calls and failures
	mock := NewExecutor()

	// First call succeeds
	err := mock.Execute(claude.ExecuteOptions{Prompt: "first", Mode: "plan"})
	assert.NoError(t, err)

	// Setup failure for next calls
	mock.SetupFailure(errors.New("service unavailable"))

	// Second call fails
	err = mock.Execute(claude.ExecuteOptions{Prompt: "second"})
	assert.Error(t, err)

	// Session call also fails
	err = mock.ExecuteWithSession("session-123", claude.ExecuteOptions{})
	assert.Error(t, err)

	// Verify call history
	assert.Equal(t, 3, len(mock.CallLog))
	assert.Equal(t, 2, mock.GetCallCount("Execute"))
	assert.Equal(t, 1, mock.GetCallCount("ExecuteWithSession"))

	// Reset and verify normal operation
	mock.Reset()
	err = mock.Execute(claude.ExecuteOptions{Prompt: "after reset"})
	assert.NoError(t, err)
	assert.Equal(t, 1, len(mock.CallLog), "CallLog should only have one call after reset")
}
