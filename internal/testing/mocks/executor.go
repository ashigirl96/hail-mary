package mocks

import (
	"time"

	"github.com/ashigirl96/hail-mary/internal/claude"
)

// Executor is a mock implementation of the claude.Executor interface for testing.
// It allows for controlled testing of components that depend on Claude CLI
// without actually executing external processes.
type Executor struct {
	// Configuration for return values
	SessionResult     *claude.SessionInfo
	ExecuteError      error
	InteractiveResult error

	// Call tracking for verification
	CallLog []MethodCall

	// Behavior control
	ShouldFailInteractive bool
	ShouldFailTracking    bool
}

// MethodCall represents a recorded method invocation
type MethodCall struct {
	Method string
	Args   []interface{}
	Time   time.Time
}

// NewExecutor creates a new mock Executor with default success values
func NewExecutor() *Executor {
	return &Executor{
		SessionResult: &claude.SessionInfo{
			ID:       "mock-session-123",
			Result:   "Mock response",
			CostUSD:  0.01,
			Duration: "1s",
			Turns:    1,
		},
		CallLog: make([]MethodCall, 0),
	}
}

// ExecuteInteractive simulates interactive execution
func (m *Executor) ExecuteInteractive(prompt string) error {
	m.recordCall("ExecuteInteractive", prompt)

	if m.ShouldFailInteractive {
		return m.InteractiveResult
	}
	return nil
}

// ExecuteInteractiveContinue simulates continuing the most recent session
func (m *Executor) ExecuteInteractiveContinue() error {
	m.recordCall("ExecuteInteractiveContinue")

	if m.ShouldFailInteractive {
		return m.InteractiveResult
	}
	return nil
}

// ExecuteInteractiveWithSession simulates interactive execution with a specific session
func (m *Executor) ExecuteInteractiveWithSession(sessionID string) error {
	m.recordCall("ExecuteInteractiveWithSession", sessionID)

	if m.ShouldFailInteractive {
		return m.InteractiveResult
	}
	return nil
}

// ExecuteAndContinueInteractive simulates execution followed by interactive continuation
func (m *Executor) ExecuteAndContinueInteractive(prompt string) (*claude.SessionInfo, error) {
	m.recordCall("ExecuteAndContinueInteractive", prompt)

	if m.ShouldFailTracking {
		return nil, m.ExecuteError
	}
	return m.SessionResult, nil
}

// recordCall adds a method call to the call log for verification
func (m *Executor) recordCall(method string, args ...interface{}) {
	m.CallLog = append(m.CallLog, MethodCall{
		Method: method,
		Args:   args,
		Time:   time.Now(),
	})
}

// GetCallCount returns the number of times a specific method was called
func (m *Executor) GetCallCount(method string) int {
	count := 0
	for _, call := range m.CallLog {
		if call.Method == method {
			count++
		}
	}
	return count
}

// GetLastCall returns the last call made to a specific method, or nil if not found
func (m *Executor) GetLastCall(method string) *MethodCall {
	for i := len(m.CallLog) - 1; i >= 0; i-- {
		if m.CallLog[i].Method == method {
			return &m.CallLog[i]
		}
	}
	return nil
}

// Reset clears the call log and resets behavior flags
func (m *Executor) Reset() {
	m.CallLog = make([]MethodCall, 0)
	m.ShouldFailInteractive = false
	m.ShouldFailTracking = false
}

// SetupFailure configures the mock to fail for specific operations
func (m *Executor) SetupFailure(interactive, tracking bool, err error) {
	m.ShouldFailInteractive = interactive
	m.ShouldFailTracking = tracking
	m.ExecuteError = err
	m.InteractiveResult = err
}
