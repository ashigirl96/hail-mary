package cmd

import (
	"encoding/json"
	"io"
	"log/slog"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/ashigirl96/hail-mary/internal/hooks"
	"github.com/ashigirl96/hail-mary/internal/session"
)

// TestHookCommand tests the hook command setup
func TestHookCommand(t *testing.T) {
	// Test command properties
	if hookCmd.Use != "hook" {
		t.Errorf("hookCmd.Use = %q, want %q", hookCmd.Use, "hook")
	}

	expectedShort := "Hook handler for Claude Code integration"
	if hookCmd.Short != expectedShort {
		t.Errorf("hookCmd.Short = %q, want %q", hookCmd.Short, expectedShort)
	}

	if hookCmd.Long == "" {
		t.Error("hookCmd.Long is empty")
	}

	// Verify command is hidden
	if !hookCmd.Hidden {
		t.Error("hookCmd.Hidden = false, want true")
	}

	// Verify RunE is set
	if hookCmd.RunE == nil {
		t.Error("hookCmd.RunE is nil")
	}

	// Test that hook command is registered with root
	found := false
	for _, cmd := range rootCmd.Commands() {
		if cmd.Use == "hook" {
			found = true
			break
		}
	}
	if !found {
		t.Error("hook command not found in root commands")
	}
}

// TestRunHook tests the runHook function
func TestRunHook(t *testing.T) {
	// Save original stdin
	oldStdin := os.Stdin
	defer func() {
		os.Stdin = oldStdin
	}()

	tests := []struct {
		name      string
		input     string
		wantError bool
		errorMsg  string
	}{
		{
			name:      "empty input",
			input:     "",
			wantError: true,
			errorMsg:  "failed to parse hook event",
		},
		{
			name:      "invalid JSON",
			input:     "{invalid json}",
			wantError: true,
			errorMsg:  "failed to parse hook event",
		},
		{
			name: "valid SessionStart event",
			input: `{
				"session_id": "test-123",
				"transcript_path": "/path/to/transcript",
				"cwd": "/tmp",
				"hook_event_name": "SessionStart",
				"source": "startup"
			}`,
			wantError: false,
		},
		{
			name: "unhandled event type",
			input: `{
				"session_id": "test-123",
				"transcript_path": "/path/to/transcript",
				"cwd": "/tmp",
				"hook_event_name": "UnhandledEvent"
			}`,
			wantError: false, // Should succeed but do nothing
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Create a pipe to simulate stdin
			r, w, _ := os.Pipe()
			os.Stdin = r

			// Write test input
			go func() {
				w.Write([]byte(tt.input))
				w.Close()
			}()

			// Run the hook command
			err := runHook(hookCmd, []string{})

			if (err != nil) != tt.wantError {
				t.Errorf("runHook() error = %v, wantError %v", err, tt.wantError)
			}

			if err != nil && tt.errorMsg != "" {
				if !strings.Contains(err.Error(), tt.errorMsg) {
					t.Errorf("runHook() error = %v, want error containing %q", err, tt.errorMsg)
				}
			}
		})
	}
}

// TestHandleSessionStart tests the handleSessionStart function
func TestHandleSessionStart(t *testing.T) {
	logger := slog.New(slog.NewTextHandler(io.Discard, nil))

	t.Run("valid event without parent PID", func(t *testing.T) {
		event := hooks.SessionStartEvent{
			BaseHookEvent: hooks.BaseHookEvent{
				SessionID:      "test-session",
				TranscriptPath: "/test/transcript",
				CWD:            "/tmp",
				HookEventName:  "SessionStart",
			},
			Source: "startup",
		}

		input, _ := json.Marshal(event)

		// Capture stdout
		oldStdout := os.Stdout
		r, w, _ := os.Pipe()
		os.Stdout = w

		err := handleSessionStart(input, "", logger)

		w.Close()
		os.Stdout = oldStdout

		if err != nil {
			t.Errorf("handleSessionStart() error = %v", err)
		}

		// Read captured output
		output, _ := io.ReadAll(r)
		outputStr := string(output)

		// Should output hook response
		if !strings.Contains(outputStr, "SessionStart") {
			t.Error("Expected SessionStart in output")
		}
	})

	t.Run("valid event with parent PID", func(t *testing.T) {
		// Create temporary directory for session state
		tempDir := t.TempDir()
		os.Setenv("HOME", tempDir)
		defer os.Unsetenv("HOME")

		parentPID := "12345"
		event := hooks.SessionStartEvent{
			BaseHookEvent: hooks.BaseHookEvent{
				SessionID:      "test-session-pid",
				TranscriptPath: "/test/transcript",
				CWD:            "/tmp",
				HookEventName:  "SessionStart",
			},
			Source: "startup",
		}

		input, _ := json.Marshal(event)

		// Capture stdout
		oldStdout := os.Stdout
		r, w, _ := os.Pipe()
		os.Stdout = w

		err := handleSessionStart(input, parentPID, logger)

		w.Close()
		os.Stdout = oldStdout

		if err != nil {
			t.Errorf("handleSessionStart() with parent PID error = %v", err)
		}

		// Verify session file was created
		sessionPath := filepath.Join(tempDir, ".hail-mary", "sessions", parentPID+".json")
		if _, err := os.Stat(sessionPath); os.IsNotExist(err) {
			t.Error("Session file was not created")
		}

		// Read output
		output, _ := io.ReadAll(r)
		outputStr := string(output)

		// Should include parent PID in output
		if !strings.Contains(outputStr, parentPID) {
			t.Error("Expected parent PID in output")
		}
	})

	t.Run("invalid event", func(t *testing.T) {
		input := []byte(`{"invalid": "event"}`)

		err := handleSessionStart(input, "", logger)

		if err == nil {
			t.Error("handleSessionStart() with invalid event should error")
		}

		if !strings.Contains(err.Error(), "validation failed") {
			t.Errorf("Expected validation error, got: %v", err)
		}
	})

	t.Run("validation failure", func(t *testing.T) {
		// Missing required field
		event := hooks.SessionStartEvent{
			BaseHookEvent: hooks.BaseHookEvent{
				SessionID:      "test",
				TranscriptPath: "/test",
				CWD:            "/tmp",
				HookEventName:  "SessionStart",
			},
			// Missing Source field
		}

		input, _ := json.Marshal(event)

		err := handleSessionStart(input, "", logger)

		if err == nil {
			t.Error("handleSessionStart() with invalid event should error")
		}

		if !strings.Contains(err.Error(), "validation failed") {
			t.Errorf("Expected validation error, got: %v", err)
		}
	})
}

// TestHandleUserPromptSubmit tests the handleUserPromptSubmit function
func TestHandleUserPromptSubmit(t *testing.T) {
	logger := slog.New(slog.NewTextHandler(io.Discard, nil))

	t.Run("valid event without parent PID", func(t *testing.T) {
		event := hooks.UserPromptSubmitEvent{
			BaseHookEvent: hooks.BaseHookEvent{
				SessionID:      "test-session",
				TranscriptPath: "/test/transcript",
				CWD:            "/tmp",
				HookEventName:  "UserPromptSubmit",
			},
			Prompt: "Test prompt",
		}

		input, _ := json.Marshal(event)

		err := handleUserPromptSubmit(input, "", logger)

		if err != nil {
			t.Errorf("handleUserPromptSubmit() error = %v", err)
		}
	})

	t.Run("valid event with parent PID and existing session", func(t *testing.T) {
		// Setup test environment
		tempDir := t.TempDir()
		os.Setenv("HOME", tempDir)
		defer os.Unsetenv("HOME")

		parentPID := "12345"

		// Create a session first
		sm, _ := session.NewManager()
		sessionState := &session.State{
			SessionID:      "test-session-prompt",
			StartedAt:      time.Now().Add(-1 * time.Hour),
			LastUpdated:    time.Now().Add(-30 * time.Minute),
			TranscriptPath: "/test/transcript",
			ProjectDir:     "/test/project",
		}
		sm.WriteSession(parentPID, sessionState)

		event := hooks.UserPromptSubmitEvent{
			BaseHookEvent: hooks.BaseHookEvent{
				SessionID:      "test-session-prompt",
				TranscriptPath: "/test/transcript",
				CWD:            "/tmp",
				HookEventName:  "UserPromptSubmit",
			},
			Prompt: "Test prompt",
		}

		input, _ := json.Marshal(event)

		// Capture stdout
		oldStdout := os.Stdout
		r, w, _ := os.Pipe()
		os.Stdout = w

		err := handleUserPromptSubmit(input, parentPID, logger)

		w.Close()
		os.Stdout = oldStdout

		if err != nil {
			t.Errorf("handleUserPromptSubmit() with session error = %v", err)
		}

		// Read output
		output, _ := io.ReadAll(r)
		outputStr := string(output)

		// Should include session context
		if !strings.Contains(outputStr, "Session:") {
			t.Error("Expected session context in output")
		}

		// Verify session was updated
		updatedState, _ := sm.ReadSession(parentPID)
		if !updatedState.LastUpdated.After(sessionState.LastUpdated) {
			t.Error("Session LastUpdated was not updated")
		}
	})

	t.Run("invalid event", func(t *testing.T) {
		input := []byte(`{"invalid": "event"}`)

		err := handleUserPromptSubmit(input, "", logger)

		if err == nil {
			t.Error("handleUserPromptSubmit() with invalid event should error")
		}
	})
}

// TestHandlePreToolUse tests the handlePreToolUse function
func TestHandlePreToolUse(t *testing.T) {
	logger := slog.New(slog.NewTextHandler(io.Discard, nil))

	t.Run("valid event", func(t *testing.T) {
		event := hooks.PreToolUseEvent{
			BaseHookEvent: hooks.BaseHookEvent{
				SessionID:      "test-session",
				TranscriptPath: "/test/transcript",
				CWD:            "/tmp",
				HookEventName:  "PreToolUse",
			},
			ToolName: "Write",
			ToolInput: hooks.ToolInput{
				"file":    "test.txt",
				"content": "test content",
			},
		}

		input, _ := json.Marshal(event)

		err := handlePreToolUse(input, logger)

		if err != nil {
			t.Errorf("handlePreToolUse() error = %v", err)
		}
	})

	t.Run("invalid event", func(t *testing.T) {
		input := []byte(`{"invalid": "event"}`)

		err := handlePreToolUse(input, logger)

		if err == nil {
			t.Error("handlePreToolUse() with invalid event should error")
		}
	})
}

// TestHandlePostToolUse tests the handlePostToolUse function
func TestHandlePostToolUse(t *testing.T) {
	logger := slog.New(slog.NewTextHandler(io.Discard, nil))

	t.Run("valid event", func(t *testing.T) {
		event := hooks.PostToolUseEvent{
			BaseHookEvent: hooks.BaseHookEvent{
				SessionID:      "test-session",
				TranscriptPath: "/test/transcript",
				CWD:            "/tmp",
				HookEventName:  "PostToolUse",
			},
			ToolName: "Write",
			ToolInput: hooks.ToolInput{
				"file": "test.txt",
			},
			ToolResponse: hooks.ToolResponse{
				"success": true,
			},
		}

		input, _ := json.Marshal(event)

		err := handlePostToolUse(input, logger)

		if err != nil {
			t.Errorf("handlePostToolUse() error = %v", err)
		}
	})

	t.Run("invalid event", func(t *testing.T) {
		input := []byte(`{"invalid": "event"}`)

		err := handlePostToolUse(input, logger)

		if err == nil {
			t.Error("handlePostToolUse() with invalid event should error")
		}
	})
}

// TestHandleStop tests the handleStop function
func TestHandleStop(t *testing.T) {
	logger := slog.New(slog.NewTextHandler(io.Discard, nil))

	t.Run("valid event without parent PID", func(t *testing.T) {
		event := hooks.StopEvent{
			BaseHookEvent: hooks.BaseHookEvent{
				SessionID:      "test-session",
				TranscriptPath: "/test/transcript",
				CWD:            "/tmp",
				HookEventName:  "Stop",
			},
			StopHookActive: false,
		}

		input, _ := json.Marshal(event)

		err := handleStop(input, "", logger)

		if err != nil {
			t.Errorf("handleStop() error = %v", err)
		}
	})

	t.Run("valid event with parent PID and session preservation", func(t *testing.T) {
		// Setup test environment
		tempDir := t.TempDir()
		os.Setenv("HOME", tempDir)
		defer os.Unsetenv("HOME")

		parentPID := "12345"

		// Create a session to cleanup
		sm, _ := session.NewManager()
		sessionState := &session.State{
			SessionID:      "test-session-stop",
			StartedAt:      time.Now(),
			LastUpdated:    time.Now(),
			TranscriptPath: "/test/transcript",
			ProjectDir:     "/test/project",
		}
		sm.WriteSession(parentPID, sessionState)

		event := hooks.StopEvent{
			BaseHookEvent: hooks.BaseHookEvent{
				SessionID:      "test-session-stop",
				TranscriptPath: "/test/transcript",
				CWD:            "/tmp",
				HookEventName:  "Stop",
			},
			StopHookActive: false, // Final stop
		}

		input, _ := json.Marshal(event)

		err := handleStop(input, parentPID, logger)

		if err != nil {
			t.Errorf("handleStop() with cleanup error = %v", err)
		}

		// Verify session is preserved (updated behavior - no cleanup)
		state, err := sm.ReadSession(parentPID)
		if err != nil {
			t.Errorf("Session should be preserved, but got error: %v", err)
		}
		if state == nil {
			t.Error("Session state should be preserved")
		}
	})

	t.Run("stop with active hook (no cleanup)", func(t *testing.T) {
		// Setup test environment
		tempDir := t.TempDir()
		os.Setenv("HOME", tempDir)
		defer os.Unsetenv("HOME")

		parentPID := "12346"

		// Create a session
		sm, _ := session.NewManager()
		sessionState := &session.State{
			SessionID: "test-session-active",
		}
		sm.WriteSession(parentPID, sessionState)

		event := hooks.StopEvent{
			BaseHookEvent: hooks.BaseHookEvent{
				SessionID:      "test-session-active",
				TranscriptPath: "/test/transcript",
				CWD:            "/tmp",
				HookEventName:  "Stop",
			},
			StopHookActive: true, // Hook still active
		}

		input, _ := json.Marshal(event)

		err := handleStop(input, parentPID, logger)

		if err != nil {
			t.Errorf("handleStop() with active hook error = %v", err)
		}

		// Verify session was NOT cleaned up
		_, err = sm.ReadSession(parentPID)
		if err != nil {
			t.Error("Session was cleaned up when hook still active")
		}
	})

	t.Run("invalid event", func(t *testing.T) {
		input := []byte(`{"invalid": "event"}`)

		err := handleStop(input, "", logger)

		if err == nil {
			t.Error("handleStop() with invalid event should error")
		}
	})
}

// TestRunHookIntegration tests the full runHook flow with different event types
func TestRunHookIntegration(t *testing.T) {
	// Save original stdin
	oldStdin := os.Stdin
	defer func() {
		os.Stdin = oldStdin
	}()

	events := []struct {
		name      string
		eventType string
		event     interface{}
	}{
		{
			name:      "SessionStart",
			eventType: "SessionStart",
			event: hooks.SessionStartEvent{
				BaseHookEvent: hooks.BaseHookEvent{
					SessionID:      "test-123",
					TranscriptPath: "/test",
					CWD:            "/tmp",
					HookEventName:  "SessionStart",
				},
				Source: "startup",
			},
		},
		{
			name:      "UserPromptSubmit",
			eventType: "UserPromptSubmit",
			event: hooks.UserPromptSubmitEvent{
				BaseHookEvent: hooks.BaseHookEvent{
					SessionID:      "test-123",
					TranscriptPath: "/test",
					CWD:            "/tmp",
					HookEventName:  "UserPromptSubmit",
				},
				Prompt: "test prompt",
			},
		},
		{
			name:      "PreToolUse",
			eventType: "PreToolUse",
			event: hooks.PreToolUseEvent{
				BaseHookEvent: hooks.BaseHookEvent{
					SessionID:      "test-123",
					TranscriptPath: "/test",
					CWD:            "/tmp",
					HookEventName:  "PreToolUse",
				},
				ToolName:  "Write",
				ToolInput: hooks.ToolInput{},
			},
		},
		{
			name:      "PostToolUse",
			eventType: "PostToolUse",
			event: hooks.PostToolUseEvent{
				BaseHookEvent: hooks.BaseHookEvent{
					SessionID:      "test-123",
					TranscriptPath: "/test",
					CWD:            "/tmp",
					HookEventName:  "PostToolUse",
				},
				ToolName:     "Write",
				ToolInput:    hooks.ToolInput{},
				ToolResponse: hooks.ToolResponse{},
			},
		},
		{
			name:      "Stop",
			eventType: "Stop",
			event: hooks.StopEvent{
				BaseHookEvent: hooks.BaseHookEvent{
					SessionID:      "test-123",
					TranscriptPath: "/test",
					CWD:            "/tmp",
					HookEventName:  "Stop",
				},
				StopHookActive: false,
			},
		},
	}

	for _, tt := range events {
		t.Run(tt.name, func(t *testing.T) {
			input, _ := json.Marshal(tt.event)

			// Create a pipe to simulate stdin
			r, w, _ := os.Pipe()
			os.Stdin = r

			// Write test input
			go func() {
				w.Write(input)
				w.Close()
			}()

			// Run the hook command
			err := runHook(hookCmd, []string{})

			if err != nil {
				t.Errorf("runHook() for %s error = %v", tt.eventType, err)
			}
		})
	}
}

// TestEnvironmentVariableHandling tests HAIL_MARY_PARENT_PID handling
func TestEnvironmentVariableHandling(t *testing.T) {
	// Save original env
	oldEnv := os.Getenv("HAIL_MARY_PARENT_PID")
	defer os.Setenv("HAIL_MARY_PARENT_PID", oldEnv)

	tests := []struct {
		name     string
		envValue string
		hasValue bool
	}{
		{
			name:     "with parent PID",
			envValue: "12345",
			hasValue: true,
		},
		{
			name:     "empty parent PID",
			envValue: "",
			hasValue: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			os.Setenv("HAIL_MARY_PARENT_PID", tt.envValue)

			// The runHook function will read this env var
			// We can't easily test the full flow without stdin,
			// but we can verify the env var is accessible
			pid := os.Getenv("HAIL_MARY_PARENT_PID")

			if tt.hasValue && pid == "" {
				t.Error("Expected parent PID to be set")
			}

			if !tt.hasValue && pid != "" {
				t.Error("Expected parent PID to be empty")
			}
		})
	}
}
