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
	ExecuteError error

	// Call tracking for verification
	CallLog []MethodCall

	// Behavior control
	ShouldFail bool
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
		CallLog: make([]MethodCall, 0),
	}
}

// Execute simulates executing Claude with options
func (m *Executor) Execute(opts claude.ExecuteOptions) error {
	m.recordCall("Execute", opts)

	if m.ShouldFail {
		return m.ExecuteError
	}
	return nil
}

// ExecuteWithSession simulates executing Claude with a session ID and options
func (m *Executor) ExecuteWithSession(sessionID string, opts claude.ExecuteOptions) error {
	m.recordCall("ExecuteWithSession", sessionID, opts)

	if m.ShouldFail {
		return m.ExecuteError
	}
	return nil
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
	m.ShouldFail = false
}

// SetupFailure configures the mock to fail with the given error
func (m *Executor) SetupFailure(err error) {
	m.ShouldFail = true
	m.ExecuteError = err
}
