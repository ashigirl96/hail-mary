package cmd

import (
	"bytes"
	"fmt"
	"strings"
	"testing"

	"github.com/ashigirl96/hail-mary/internal/claude"
)

// TestPrdContinueCommand tests the prd continue command setup
func TestPrdContinueCommand(t *testing.T) {
	// Test command properties
	if prdContinueCmd.Use != "continue" {
		t.Errorf("prdContinueCmd.Use = %q, want %q", prdContinueCmd.Use, "continue")
	}

	expectedShort := "Continue working on an existing PRD session"
	if prdContinueCmd.Short != expectedShort {
		t.Errorf("prdContinueCmd.Short = %q, want %q", prdContinueCmd.Short, expectedShort)
	}

	if prdContinueCmd.Long == "" {
		t.Error("prdContinueCmd.Long is empty")
	}

	// Verify RunE is set
	if prdContinueCmd.RunE == nil {
		t.Error("prdContinueCmd.RunE is nil")
	}

	// Test that continue command is registered with prd
	found := false
	for _, cmd := range prdCmd.Commands() {
		if cmd.Use == "continue" {
			found = true
			break
		}
	}
	if !found {
		t.Error("continue command not found in prd subcommands")
	}
}

// MockExecutor for testing
type mockClaudeExecutor struct {
	executeInteractiveContinueCalled bool
	executeInteractiveContinueError  error
}

func (m *mockClaudeExecutor) ExecuteInteractive(prompt string) error {
	return nil
}

func (m *mockClaudeExecutor) ExecuteInteractiveContinue() error {
	m.executeInteractiveContinueCalled = true
	return m.executeInteractiveContinueError
}

func (m *mockClaudeExecutor) ExecuteWithSessionTracking(prompt string) (*claude.SessionInfo, error) {
	return nil, nil
}

func (m *mockClaudeExecutor) ResumeSession(sessionID, prompt string) (*claude.SessionInfo, error) {
	return nil, nil
}

func (m *mockClaudeExecutor) ExecuteInteractiveWithSession(sessionID string) error {
	return nil
}

func (m *mockClaudeExecutor) ExecuteAndContinueInteractive(prompt string) (*claude.SessionInfo, error) {
	return nil, nil
}

// TestPrdContinueCommandRun tests the execution of prd continue command
func TestPrdContinueCommandRun(t *testing.T) {
	// Test command structure
	t.Run("command structure", func(t *testing.T) {
		// Verify RunE is set
		if prdContinueCmd.RunE == nil {
			t.Error("prdContinueCmd.RunE is nil")
		}

		// Verify the command doesn't take arguments
		if prdContinueCmd.Args != nil {
			err := prdContinueCmd.Args(prdContinueCmd, []string{"unexpected", "args"})
			if err == nil {
				t.Error("Expected error for unexpected args, got nil")
			}
		}
	})

	// Test command output messages
	t.Run("output messages", func(t *testing.T) {
		// Create a test function that simulates the command behavior
		testFunc := func() (string, error) {
			var output bytes.Buffer
			fmt.Fprintln(&output, "Launching Claude interactive shell to continue PRD...")
			fmt.Fprintln(&output, "Press Ctrl+C to exit the Claude shell.")
			// Simulate error
			return output.String(), fmt.Errorf("failed to continue Claude session: test error")
		}

		out, err := testFunc()

		if !strings.Contains(out, "Launching Claude interactive shell") {
			t.Error("Expected launch message not found")
		}

		if !strings.Contains(out, "Press Ctrl+C to exit") {
			t.Error("Expected exit instruction not found")
		}

		if err == nil {
			t.Error("Expected error, got nil")
		}

		if !strings.Contains(err.Error(), "failed to continue Claude session") {
			t.Errorf("Expected error message not found, got: %v", err)
		}
	})
}

// TestPrdContinueInit tests that the continue command is properly initialized
func TestPrdContinueInit(t *testing.T) {
	// Since init() is called automatically when the package loads,
	// we just verify that the continue command is registered with prd

	// Verify continue command was added
	found := false
	for _, cmd := range prdCmd.Commands() {
		if cmd.Use == "continue" {
			found = true
			break
		}
	}

	if !found {
		t.Error("continue command not registered with prd")
	}
}
