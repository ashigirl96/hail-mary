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
	mock.SetupFailure(true, false, false, expectedError)

	// Act
	err := mock.ExecuteInteractive("test")

	// Assert
	require.Error(t, err)
	assert.Equal(t, expectedError, err)
	assert.Len(t, mock.CallLog, 1, "Should record the call even on failure")
}

func TestExecutor_ExecuteWithSessionTracking_Success(t *testing.T) {
	// Arrange
	mock := NewExecutor()
	prompt := "Test prompt"

	// Act
	result, err := mock.ExecuteWithSessionTracking(prompt)

	// Assert
	require.NoError(t, err)
	require.NotNil(t, result)
	assert.Equal(t, mock.SessionResult.ID, result.ID)
	assert.Equal(t, mock.SessionResult.Result, result.Result)
	assert.Equal(t, mock.SessionResult.CostUSD, result.CostUSD)

	assert.Len(t, mock.CallLog, 1)
	assert.Equal(t, "ExecuteWithSessionTracking", mock.CallLog[0].Method)
	assert.Equal(t, prompt, mock.CallLog[0].Args[0])
}

func TestExecutor_ExecuteWithSessionTracking_Failure(t *testing.T) {
	// Arrange
	mock := NewExecutor()
	expectedError := errors.New("tracking failed")
	mock.SetupFailure(false, true, false, expectedError)

	// Act
	result, err := mock.ExecuteWithSessionTracking("test")

	// Assert
	require.Error(t, err)
	assert.Equal(t, expectedError, err)
	assert.Nil(t, result)
	assert.Len(t, mock.CallLog, 1)
}

func TestExecutor_ResumeSession_Success(t *testing.T) {
	// Arrange
	mock := NewExecutor()
	sessionID := "custom-session-456"
	prompt := "Continue the conversation"

	// Act
	result, err := mock.ResumeSession(sessionID, prompt)

	// Assert
	require.NoError(t, err)
	require.NotNil(t, result)
	assert.Equal(t, sessionID, result.ID, "Result should use the provided session ID")
	assert.Equal(t, mock.SessionResult.Result, result.Result)
	assert.Equal(t, mock.SessionResult.CostUSD, result.CostUSD)

	assert.Len(t, mock.CallLog, 1)
	assert.Equal(t, "ResumeSession", mock.CallLog[0].Method)
	assert.Equal(t, sessionID, mock.CallLog[0].Args[0])
	assert.Equal(t, prompt, mock.CallLog[0].Args[1])
}

func TestExecutor_ResumeSession_Failure(t *testing.T) {
	// Arrange
	mock := NewExecutor()
	expectedError := errors.New("resume failed")
	mock.SetupFailure(false, false, true, expectedError)

	// Act
	result, err := mock.ResumeSession("session-123", "test")

	// Assert
	require.Error(t, err)
	assert.Equal(t, expectedError, err)
	assert.Nil(t, result)
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
	mock.ExecuteInteractive("test1")
	mock.ExecuteInteractive("test2")
	mock.ExecuteWithSessionTracking("test3")

	// Assert
	assert.Equal(t, 2, mock.GetCallCount("ExecuteInteractive"))
	assert.Equal(t, 1, mock.GetCallCount("ExecuteWithSessionTracking"))
	assert.Equal(t, 0, mock.GetCallCount("ResumeSession"))
}

func TestExecutor_GetLastCall(t *testing.T) {
	// Arrange
	mock := NewExecutor()

	// Act
	mock.ExecuteInteractive("first")
	mock.ExecuteInteractive("second")
	mock.ExecuteWithSessionTracking("tracking")

	// Assert
	lastInteractive := mock.GetLastCall("ExecuteInteractive")
	require.NotNil(t, lastInteractive)
	assert.Equal(t, "ExecuteInteractive", lastInteractive.Method)
	assert.Equal(t, "second", lastInteractive.Args[0])

	lastTracking := mock.GetLastCall("ExecuteWithSessionTracking")
	require.NotNil(t, lastTracking)
	assert.Equal(t, "ExecuteWithSessionTracking", lastTracking.Method)
	assert.Equal(t, "tracking", lastTracking.Args[0])

	nonExistent := mock.GetLastCall("NonExistentMethod")
	assert.Nil(t, nonExistent)
}

func TestExecutor_Reset(t *testing.T) {
	// Arrange
	mock := NewExecutor()
	mock.SetupFailure(true, true, true, errors.New("test error"))
	mock.ExecuteInteractive("test")

	// Pre-condition checks
	assert.Len(t, mock.CallLog, 1)
	assert.True(t, mock.ShouldFailInteractive)
	assert.True(t, mock.ShouldFailTracking)
	assert.True(t, mock.ShouldFailResume)

	// Act
	mock.Reset()

	// Assert
	assert.Empty(t, mock.CallLog, "CallLog should be cleared")
	assert.False(t, mock.ShouldFailInteractive, "ShouldFailInteractive should be reset")
	assert.False(t, mock.ShouldFailTracking, "ShouldFailTracking should be reset")
	assert.False(t, mock.ShouldFailResume, "ShouldFailResume should be reset")
}

func TestExecutor_SetupFailure(t *testing.T) {
	// Arrange
	mock := NewExecutor()
	testError := errors.New("custom test error")

	// Act
	mock.SetupFailure(true, false, true, testError)

	// Assert
	assert.True(t, mock.ShouldFailInteractive)
	assert.False(t, mock.ShouldFailTracking)
	assert.True(t, mock.ShouldFailResume)
	assert.Equal(t, testError, mock.ExecuteError)
	assert.Equal(t, testError, mock.InteractiveResult)
}

func TestExecutor_CallTimestamps(t *testing.T) {
	// Arrange
	mock := NewExecutor()
	start := time.Now()

	// Act
	mock.ExecuteInteractive("test")
	time.Sleep(1 * time.Millisecond) // Ensure timestamp difference
	mock.ExecuteWithSessionTracking("test")

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
	// 1. Start with session tracking
	result1, err1 := mock.ExecuteWithSessionTracking("Initial prompt")
	require.NoError(t, err1)

	// 2. Continue with interactive mode
	err2 := mock.ExecuteInteractiveWithSession(result1.ID)
	require.NoError(t, err2)

	// 3. Resume the session later
	result2, err3 := mock.ResumeSession(result1.ID, "Follow-up prompt")
	require.NoError(t, err3)

	// Assert
	assert.Len(t, mock.CallLog, 3, "Should have recorded 3 calls")

	// Verify call sequence
	assert.Equal(t, "ExecuteWithSessionTracking", mock.CallLog[0].Method)
	assert.Equal(t, "ExecuteInteractiveWithSession", mock.CallLog[1].Method)
	assert.Equal(t, "ResumeSession", mock.CallLog[2].Method)

	// Verify session ID consistency
	assert.Equal(t, "complex-session", result1.ID)
	assert.Equal(t, "complex-session", result2.ID)

	// Verify call counts
	assert.Equal(t, 1, mock.GetCallCount("ExecuteWithSessionTracking"))
	assert.Equal(t, 1, mock.GetCallCount("ExecuteInteractiveWithSession"))
	assert.Equal(t, 1, mock.GetCallCount("ResumeSession"))
}
