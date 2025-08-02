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
	assert.NotNil(t, mock.SessionResult, "SessionResult should be initialized")
	assert.Equal(t, "mock-session-123", mock.SessionResult.ID, "Default session ID should be set")
	assert.Equal(t, "Mock response", mock.SessionResult.Result, "Default result should be set")
	assert.Equal(t, 0.01, mock.SessionResult.CostUSD, "Default cost should be set")
	assert.Empty(t, mock.CallLog, "CallLog should be empty initially")
}

func TestExecutor_ImplementsInterface(t *testing.T) {
	// Contract test
	var _ claude.Executor = (*Executor)(nil)
}

func TestExecutor_ExecuteInteractive_Success(t *testing.T) {
	// Arrange
	mock := NewExecutor()
	prompt := "Create a function"

	// Act
	err := mock.ExecuteInteractive(prompt)

	// Assert
	require.NoError(t, err)
	assert.Len(t, mock.CallLog, 1, "Should record one call")
	assert.Equal(t, "ExecuteInteractive", mock.CallLog[0].Method)
	assert.Equal(t, prompt, mock.CallLog[0].Args[0])
}

func TestExecutor_ExecuteInteractive_Failure(t *testing.T) {
	// Arrange
	mock := NewExecutor()
	expectedError := errors.New("interactive execution failed")
	mock.SetupFailure(true, false, expectedError)

	// Act
	err := mock.ExecuteInteractive("test")

	// Assert
	require.Error(t, err)
	assert.Equal(t, expectedError, err)
	assert.Len(t, mock.CallLog, 1, "Should record the call even on failure")
}

func TestExecutor_ExecuteInteractiveContinue(t *testing.T) {
	// Arrange
	mock := NewExecutor()

	// Act
	err := mock.ExecuteInteractiveContinue()

	// Assert
	require.NoError(t, err)
	assert.Len(t, mock.CallLog, 1)
	assert.Equal(t, "ExecuteInteractiveContinue", mock.CallLog[0].Method)
	assert.Empty(t, mock.CallLog[0].Args, "Should have no arguments")
}

func TestExecutor_ExecuteInteractiveWithSession(t *testing.T) {
	// Arrange
	mock := NewExecutor()
	sessionID := "session-789"

	// Act
	err := mock.ExecuteInteractiveWithSession(sessionID)

	// Assert
	require.NoError(t, err)
	assert.Len(t, mock.CallLog, 1)
	assert.Equal(t, "ExecuteInteractiveWithSession", mock.CallLog[0].Method)
	assert.Equal(t, sessionID, mock.CallLog[0].Args[0])
}

func TestExecutor_ExecuteAndContinueInteractive(t *testing.T) {
	// Arrange
	mock := NewExecutor()
	prompt := "Start and continue"

	// Act
	result, err := mock.ExecuteAndContinueInteractive(prompt)

	// Assert
	require.NoError(t, err)
	require.NotNil(t, result)
	assert.Equal(t, mock.SessionResult.ID, result.ID)

	assert.Len(t, mock.CallLog, 1)
	assert.Equal(t, "ExecuteAndContinueInteractive", mock.CallLog[0].Method)
	assert.Equal(t, prompt, mock.CallLog[0].Args[0])
}

func TestExecutor_GetCallCount(t *testing.T) {
	// Arrange
	mock := NewExecutor()

	// Act
	_ = mock.ExecuteInteractive("test1")
	_ = mock.ExecuteInteractive("test2")
	_ = mock.ExecuteInteractiveContinue()

	// Assert
	assert.Equal(t, 2, mock.GetCallCount("ExecuteInteractive"))
	assert.Equal(t, 1, mock.GetCallCount("ExecuteInteractiveContinue"))
}

func TestExecutor_GetLastCall(t *testing.T) {
	// Arrange
	mock := NewExecutor()

	// Act
	_ = mock.ExecuteInteractive("first")
	_ = mock.ExecuteInteractive("second")
	_ = mock.ExecuteInteractiveContinue()

	// Assert
	lastInteractive := mock.GetLastCall("ExecuteInteractive")
	require.NotNil(t, lastInteractive)
	assert.Equal(t, "ExecuteInteractive", lastInteractive.Method)
	assert.Equal(t, "second", lastInteractive.Args[0])

	lastContinue := mock.GetLastCall("ExecuteInteractiveContinue")
	require.NotNil(t, lastContinue)
	assert.Equal(t, "ExecuteInteractiveContinue", lastContinue.Method)

	nonExistent := mock.GetLastCall("NonExistentMethod")
	assert.Nil(t, nonExistent)
}

func TestExecutor_Reset(t *testing.T) {
	// Arrange
	mock := NewExecutor()
	mock.SetupFailure(true, true, errors.New("test error"))
	_ = mock.ExecuteInteractive("test")

	// Pre-condition checks
	assert.Len(t, mock.CallLog, 1)
	assert.True(t, mock.ShouldFailInteractive)
	assert.True(t, mock.ShouldFailTracking)

	// Act
	mock.Reset()

	// Assert
	assert.Empty(t, mock.CallLog, "CallLog should be cleared")
	assert.False(t, mock.ShouldFailInteractive, "ShouldFailInteractive should be reset")
	assert.False(t, mock.ShouldFailTracking, "ShouldFailTracking should be reset")
}

func TestExecutor_SetupFailure(t *testing.T) {
	// Arrange
	mock := NewExecutor()
	testError := errors.New("custom test error")

	// Act
	mock.SetupFailure(true, false, testError)

	// Assert
	assert.True(t, mock.ShouldFailInteractive)
	assert.False(t, mock.ShouldFailTracking)
	assert.Equal(t, testError, mock.ExecuteError)
	assert.Equal(t, testError, mock.InteractiveResult)
}

func TestExecutor_CallTimestamps(t *testing.T) {
	// Arrange
	mock := NewExecutor()
	start := time.Now()

	// Act
	_ = mock.ExecuteInteractive("test")
	time.Sleep(1 * time.Millisecond) // Ensure timestamp difference
	_, _ = mock.ExecuteAndContinueInteractive("test")

	// Assert
	require.Len(t, mock.CallLog, 2)

	firstCall := mock.CallLog[0]
	secondCall := mock.CallLog[1]

	assert.True(t, firstCall.Time.After(start) || firstCall.Time.Equal(start))
	assert.True(t, secondCall.Time.After(firstCall.Time))
}

func TestExecutor_ComplexScenario(t *testing.T) {
	// Arrange
	mock := NewExecutor()
	mock.SessionResult = &claude.SessionInfo{
		ID:       "complex-session",
		Result:   "Complex result",
		CostUSD:  0.05,
		Duration: "5s",
		Turns:    3,
	}

	// Act - Simulate a complex workflow
	// 1. Start with interactive execution
	result1, err1 := mock.ExecuteAndContinueInteractive("Initial prompt")
	require.NoError(t, err1)

	// 2. Continue with interactive mode
	err2 := mock.ExecuteInteractiveWithSession(result1.ID)
	require.NoError(t, err2)

	// Assert
	assert.Len(t, mock.CallLog, 2, "Should have recorded 2 calls")

	// Verify call sequence
	assert.Equal(t, "ExecuteAndContinueInteractive", mock.CallLog[0].Method)
	assert.Equal(t, "ExecuteInteractiveWithSession", mock.CallLog[1].Method)

	// Verify session ID consistency
	assert.Equal(t, "complex-session", result1.ID)

	// Verify call counts
	assert.Equal(t, 1, mock.GetCallCount("ExecuteAndContinueInteractive"))
	assert.Equal(t, 1, mock.GetCallCount("ExecuteInteractiveWithSession"))
}
