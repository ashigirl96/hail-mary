package hooks

import (
	"encoding/json"
	"strings"
	"testing"
)

// TestParseHookEvent tests parsing various hook events
func TestParseHookEvent(t *testing.T) {
	tests := []struct {
		name      string
		jsonData  string
		wantType  interface{}
		wantError bool
		errorMsg  string
	}{
		{
			name: "valid SessionStart event",
			jsonData: `{
				"session_id": "test-123",
				"transcript_path": "/tmp/transcript.txt",
				"cwd": "/tmp",
				"hook_event_name": "SessionStart",
				"source": "startup"
			}`,
			wantType:  SessionStartEvent{},
			wantError: false,
		},
		{
			name: "valid UserPromptSubmit event",
			jsonData: `{
				"session_id": "test-123",
				"transcript_path": "/tmp/transcript.txt",
				"cwd": "/tmp",
				"hook_event_name": "UserPromptSubmit",
				"prompt": "Create a function"
			}`,
			wantType:  UserPromptSubmitEvent{},
			wantError: false,
		},
		{
			name: "valid PreToolUse event",
			jsonData: `{
				"session_id": "test-123",
				"transcript_path": "/tmp/transcript.txt",
				"cwd": "/tmp",
				"hook_event_name": "PreToolUse",
				"tool_name": "Write",
				"tool_input": {"file": "test.txt", "content": "hello"}
			}`,
			wantType:  PreToolUseEvent{},
			wantError: false,
		},
		{
			name: "valid PostToolUse event",
			jsonData: `{
				"session_id": "test-123",
				"transcript_path": "/tmp/transcript.txt",
				"cwd": "/tmp",
				"hook_event_name": "PostToolUse",
				"tool_name": "Write",
				"tool_input": {"file": "test.txt"},
				"tool_response": {"success": true}
			}`,
			wantType:  PostToolUseEvent{},
			wantError: false,
		},
		{
			name: "valid Notification event",
			jsonData: `{
				"session_id": "test-123",
				"transcript_path": "/tmp/transcript.txt",
				"cwd": "/tmp",
				"hook_event_name": "Notification",
				"message": "Task completed"
			}`,
			wantType:  NotificationEvent{},
			wantError: false,
		},
		{
			name: "valid Stop event",
			jsonData: `{
				"session_id": "test-123",
				"transcript_path": "/tmp/transcript.txt",
				"cwd": "/tmp",
				"hook_event_name": "Stop",
				"stop_hook_active": true
			}`,
			wantType:  StopEvent{},
			wantError: false,
		},
		{
			name: "valid SubagentStop event",
			jsonData: `{
				"session_id": "test-123",
				"transcript_path": "/tmp/transcript.txt",
				"cwd": "/tmp",
				"hook_event_name": "SubagentStop",
				"stop_hook_active": false
			}`,
			wantType:  SubagentStopEvent{},
			wantError: false,
		},
		{
			name: "valid PreCompact event",
			jsonData: `{
				"session_id": "test-123",
				"transcript_path": "/tmp/transcript.txt",
				"cwd": "/tmp",
				"hook_event_name": "PreCompact",
				"trigger": "manual",
				"custom_instructions": "Keep important context"
			}`,
			wantType:  PreCompactEvent{},
			wantError: false,
		},
		{
			name:      "invalid JSON",
			jsonData:  `{invalid json`,
			wantError: true,
			errorMsg:  "failed to parse base event",
		},
		{
			name: "missing required field",
			jsonData: `{
				"transcript_path": "/tmp/transcript.txt",
				"cwd": "/tmp",
				"hook_event_name": "SessionStart",
				"source": "startup"
			}`,
			wantError: true,
			errorMsg:  "Key: 'SessionStartEvent.BaseHookEvent.SessionID' Error:Field validation for 'SessionID' failed on the 'required' tag",
		},
		{
			name: "invalid hook event name",
			jsonData: `{
				"session_id": "test-123",
				"transcript_path": "/tmp/transcript.txt",
				"cwd": "/tmp",
				"hook_event_name": "InvalidEvent"
			}`,
			wantError: true,
			errorMsg:  "unknown hook event type: InvalidEvent",
		},
		{
			name: "unknown hook event type",
			jsonData: `{
				"session_id": "test-123",
				"transcript_path": "/tmp/transcript.txt",
				"cwd": "/tmp",
				"hook_event_name": "UnknownEvent"
			}`,
			wantError: true,
			errorMsg:  "unknown hook event type: UnknownEvent",
		},
		{
			name: "SessionStart with invalid source",
			jsonData: `{
				"session_id": "test-123",
				"transcript_path": "/tmp/transcript.txt",
				"cwd": "/tmp",
				"hook_event_name": "SessionStart",
				"source": "invalid"
			}`,
			wantError: true,
			errorMsg:  "Key: 'SessionStartEvent.Source' Error:Field validation for 'Source' failed on the 'oneof' tag",
		},
		{
			name: "PreCompact with invalid trigger",
			jsonData: `{
				"session_id": "test-123",
				"transcript_path": "/tmp/transcript.txt",
				"cwd": "/tmp",
				"hook_event_name": "PreCompact",
				"trigger": "invalid"
			}`,
			wantError: true,
			errorMsg:  "Key: 'PreCompactEvent.Trigger' Error:Field validation for 'Trigger' failed on the 'oneof' tag",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			event, err := ParseHookEvent([]byte(tt.jsonData))

			if tt.wantError {
				if err == nil {
					t.Error("ParseHookEvent() error = nil, want error")
				} else if tt.errorMsg != "" && !strings.Contains(err.Error(), tt.errorMsg) {
					t.Errorf("ParseHookEvent() error = %q, want to contain %q", err.Error(), tt.errorMsg)
				}
				return
			}

			if err != nil {
				t.Errorf("ParseHookEvent() unexpected error = %v", err)
				return
			}

			// Check event type
			switch tt.wantType.(type) {
			case SessionStartEvent:
				if _, ok := event.(SessionStartEvent); !ok {
					t.Errorf("ParseHookEvent() = %T, want SessionStartEvent", event)
				}
			case UserPromptSubmitEvent:
				if _, ok := event.(UserPromptSubmitEvent); !ok {
					t.Errorf("ParseHookEvent() = %T, want UserPromptSubmitEvent", event)
				}
			case PreToolUseEvent:
				if _, ok := event.(PreToolUseEvent); !ok {
					t.Errorf("ParseHookEvent() = %T, want PreToolUseEvent", event)
				}
			case PostToolUseEvent:
				if _, ok := event.(PostToolUseEvent); !ok {
					t.Errorf("ParseHookEvent() = %T, want PostToolUseEvent", event)
				}
			case NotificationEvent:
				if _, ok := event.(NotificationEvent); !ok {
					t.Errorf("ParseHookEvent() = %T, want NotificationEvent", event)
				}
			case StopEvent:
				if _, ok := event.(StopEvent); !ok {
					t.Errorf("ParseHookEvent() = %T, want StopEvent", event)
				}
			case SubagentStopEvent:
				if _, ok := event.(SubagentStopEvent); !ok {
					t.Errorf("ParseHookEvent() = %T, want SubagentStopEvent", event)
				}
			case PreCompactEvent:
				if _, ok := event.(PreCompactEvent); !ok {
					t.Errorf("ParseHookEvent() = %T, want PreCompactEvent", event)
				}
			}
		})
	}
}

// TestValidateHookEvent tests direct validation of hook events
func TestValidateHookEvent(t *testing.T) {
	tests := []struct {
		name      string
		event     interface{}
		wantError bool
		errorMsg  string
	}{
		{
			name: "valid SessionStartEvent",
			event: SessionStartEvent{
				BaseHookEvent: BaseHookEvent{
					SessionID:      "test-123",
					TranscriptPath: "/tmp/transcript.txt",
					CWD:            "/tmp",
					HookEventName:  "SessionStart",
				},
				Source: "startup",
			},
			wantError: false,
		},
		{
			name: "SessionStartEvent missing source",
			event: SessionStartEvent{
				BaseHookEvent: BaseHookEvent{
					SessionID:      "test-123",
					TranscriptPath: "/tmp/transcript.txt",
					CWD:            "/tmp",
					HookEventName:  "SessionStart",
				},
			},
			wantError: true,
			errorMsg:  "Key: 'SessionStartEvent.Source' Error:Field validation for 'Source' failed on the 'required' tag",
		},
		{
			name: "UserPromptSubmitEvent missing prompt",
			event: UserPromptSubmitEvent{
				BaseHookEvent: BaseHookEvent{
					SessionID:      "test-123",
					TranscriptPath: "/tmp/transcript.txt",
					CWD:            "/tmp",
					HookEventName:  "UserPromptSubmit",
				},
			},
			wantError: true,
			errorMsg:  "Key: 'UserPromptSubmitEvent.Prompt' Error:Field validation for 'Prompt' failed on the 'required' tag",
		},
		{
			name: "PreToolUseEvent missing tool name",
			event: PreToolUseEvent{
				BaseHookEvent: BaseHookEvent{
					SessionID:      "test-123",
					TranscriptPath: "/tmp/transcript.txt",
					CWD:            "/tmp",
					HookEventName:  "PreToolUse",
				},
				ToolInput: make(ToolInput),
			},
			wantError: true,
			errorMsg:  "Key: 'PreToolUseEvent.ToolName' Error:Field validation for 'ToolName' failed on the 'required' tag",
		},
		{
			name: "valid NotificationEvent",
			event: NotificationEvent{
				BaseHookEvent: BaseHookEvent{
					SessionID:      "test-123",
					TranscriptPath: "/tmp/transcript.txt",
					CWD:            "/tmp",
					HookEventName:  "Notification",
				},
				Message: "Test notification",
			},
			wantError: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := ValidateHookEvent(tt.event)

			if tt.wantError {
				if err == nil {
					t.Error("ValidateHookEvent() error = nil, want error")
				} else if tt.errorMsg != "" && err.Error() != tt.errorMsg {
					t.Errorf("ValidateHookEvent() error = %q, want %q", err.Error(), tt.errorMsg)
				}
			} else {
				if err != nil {
					t.Errorf("ValidateHookEvent() unexpected error = %v", err)
				}
			}
		})
	}
}

// TestHookOutputJSON tests JSON marshaling/unmarshaling of HookOutput
func TestHookOutputJSON(t *testing.T) {
	tests := []struct {
		name   string
		output HookOutput
	}{
		{
			name: "full HookOutput",
			output: HookOutput{
				Continue:       true,
				StopReason:     "User requested stop",
				SuppressOutput: true,
				Decision:       "approve",
				Reason:         "All checks passed",
				HookSpecificOutput: map[string]interface{}{
					"custom": "value",
				},
			},
		},
		{
			name: "minimal HookOutput",
			output: HookOutput{
				Continue: true,
			},
		},
		{
			name: "HookOutput with decision",
			output: HookOutput{
				Decision: "block",
				Reason:   "Security violation detected",
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Marshal to JSON
			data, err := json.Marshal(tt.output)
			if err != nil {
				t.Fatalf("json.Marshal() error = %v", err)
			}

			// Unmarshal back
			var decoded HookOutput
			if err := json.Unmarshal(data, &decoded); err != nil {
				t.Fatalf("json.Unmarshal() error = %v", err)
			}

			// Compare fields
			if decoded.Continue != tt.output.Continue {
				t.Errorf("Continue = %v, want %v", decoded.Continue, tt.output.Continue)
			}
			if decoded.StopReason != tt.output.StopReason {
				t.Errorf("StopReason = %q, want %q", decoded.StopReason, tt.output.StopReason)
			}
			if decoded.SuppressOutput != tt.output.SuppressOutput {
				t.Errorf("SuppressOutput = %v, want %v", decoded.SuppressOutput, tt.output.SuppressOutput)
			}
			if decoded.Decision != tt.output.Decision {
				t.Errorf("Decision = %q, want %q", decoded.Decision, tt.output.Decision)
			}
			if decoded.Reason != tt.output.Reason {
				t.Errorf("Reason = %q, want %q", decoded.Reason, tt.output.Reason)
			}
		})
	}
}

// TestSpecificDecisionStructs tests the specific decision structs
func TestSpecificDecisionStructs(t *testing.T) {
	// Test PreToolUseDecision
	preToolDecision := PreToolUseDecision{
		HookEventName:            "PreToolUse",
		PermissionDecision:       "allow",
		PermissionDecisionReason: "Tool is safe to use",
	}

	if err := validate.Struct(preToolDecision); err != nil {
		t.Errorf("PreToolUseDecision validation failed: %v", err)
	}

	// Test invalid PreToolUseDecision
	invalidPreToolDecision := PreToolUseDecision{
		HookEventName:      "WrongEvent",
		PermissionDecision: "allow",
	}

	if err := validate.Struct(invalidPreToolDecision); err == nil {
		t.Error("Expected validation error for invalid HookEventName")
	}

	// Test UserPromptSubmitDecision
	userPromptDecision := UserPromptSubmitDecision{
		HookEventName:     "UserPromptSubmit",
		AdditionalContext: "User is authenticated",
	}

	if err := validate.Struct(userPromptDecision); err != nil {
		t.Errorf("UserPromptSubmitDecision validation failed: %v", err)
	}

	// Test SessionStartDecision
	sessionStartDecision := SessionStartDecision{
		HookEventName:     "SessionStart",
		AdditionalContext: "Previous session recovered",
	}

	if err := validate.Struct(sessionStartDecision); err != nil {
		t.Errorf("SessionStartDecision validation failed: %v", err)
	}
}

// TestEventParsing tests parsing edge cases
func TestEventParsing(t *testing.T) {
	// Test empty tool input/response
	emptyToolEvent := `{
		"session_id": "test-123",
		"transcript_path": "/tmp/transcript.txt",
		"cwd": "/tmp",
		"hook_event_name": "PostToolUse",
		"tool_name": "Write",
		"tool_input": {},
		"tool_response": {}
	}`

	event, err := ParseHookEvent([]byte(emptyToolEvent))
	if err != nil {
		t.Errorf("ParseHookEvent() error = %v, want nil", err)
	}

	postToolEvent, ok := event.(PostToolUseEvent)
	if !ok {
		t.Fatal("Expected PostToolUseEvent")
	}

	if len(postToolEvent.ToolInput) != 0 {
		t.Errorf("ToolInput length = %d, want 0", len(postToolEvent.ToolInput))
	}
	if len(postToolEvent.ToolResponse) != 0 {
		t.Errorf("ToolResponse length = %d, want 0", len(postToolEvent.ToolResponse))
	}

	// Test complex tool input/response
	complexToolEvent := `{
		"session_id": "test-123",
		"transcript_path": "/tmp/transcript.txt",
		"cwd": "/tmp",
		"hook_event_name": "PreToolUse",
		"tool_name": "Write",
		"tool_input": {
			"file": "test.txt",
			"content": "hello world",
			"options": {
				"encoding": "utf-8",
				"mode": 644
			},
			"array": [1, 2, 3]
		}
	}`

	event2, err := ParseHookEvent([]byte(complexToolEvent))
	if err != nil {
		t.Errorf("ParseHookEvent() error = %v, want nil", err)
	}

	preToolEvent, ok := event2.(PreToolUseEvent)
	if !ok {
		t.Fatal("Expected PreToolUseEvent")
	}

	if preToolEvent.ToolInput["file"] != "test.txt" {
		t.Errorf("ToolInput[file] = %v, want test.txt", preToolEvent.ToolInput["file"])
	}

	options, ok := preToolEvent.ToolInput["options"].(map[string]interface{})
	if !ok {
		t.Fatal("Expected options to be a map")
	}
	if options["encoding"] != "utf-8" {
		t.Errorf("options[encoding] = %v, want utf-8", options["encoding"])
	}
}

// TestOptionalFields tests events with optional fields
func TestOptionalFields(t *testing.T) {
	// Test Stop event without optional field
	stopEventMin := `{
		"session_id": "test-123",
		"transcript_path": "/tmp/transcript.txt",
		"cwd": "/tmp",
		"hook_event_name": "Stop"
	}`

	event, err := ParseHookEvent([]byte(stopEventMin))
	if err != nil {
		t.Errorf("ParseHookEvent() error = %v, want nil", err)
	}

	stopEvent, ok := event.(StopEvent)
	if !ok {
		t.Fatal("Expected StopEvent")
	}

	if stopEvent.StopHookActive != false {
		t.Errorf("StopHookActive = %v, want false (default)", stopEvent.StopHookActive)
	}

	// Test PreCompact without custom instructions
	preCompactMin := `{
		"session_id": "test-123",
		"transcript_path": "/tmp/transcript.txt",
		"cwd": "/tmp",
		"hook_event_name": "PreCompact",
		"trigger": "auto"
	}`

	event2, err := ParseHookEvent([]byte(preCompactMin))
	if err != nil {
		t.Errorf("ParseHookEvent() error = %v, want nil", err)
	}

	preCompactEvent, ok := event2.(PreCompactEvent)
	if !ok {
		t.Fatal("Expected PreCompactEvent")
	}

	if preCompactEvent.CustomInstructions != "" {
		t.Errorf("CustomInstructions = %q, want empty", preCompactEvent.CustomInstructions)
	}
}

// TestValidationTags tests all validation tags
func TestValidationTags(t *testing.T) {
	// Test invalid decision value in HookOutput
	output := HookOutput{
		Decision: "invalid_decision",
	}

	if err := validate.Struct(output); err == nil {
		t.Error("Expected validation error for invalid decision value")
	}

	// Test invalid permission decision
	preToolDecision := PreToolUseDecision{
		HookEventName:      "PreToolUse",
		PermissionDecision: "invalid",
	}

	if err := validate.Struct(preToolDecision); err == nil {
		t.Error("Expected validation error for invalid permission decision")
	}
}

// Benchmark tests
func BenchmarkParseHookEvent(b *testing.B) {
	jsonData := []byte(`{
		"session_id": "test-123",
		"transcript_path": "/tmp/transcript.txt",
		"cwd": "/tmp",
		"hook_event_name": "SessionStart",
		"source": "startup"
	}`)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, _ = ParseHookEvent(jsonData)
	}
}

func BenchmarkValidateHookEvent(b *testing.B) {
	event := SessionStartEvent{
		BaseHookEvent: BaseHookEvent{
			SessionID:      "test-123",
			TranscriptPath: "/tmp/transcript.txt",
			CWD:            "/tmp",
			HookEventName:  "SessionStart",
		},
		Source: "startup",
	}

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = ValidateHookEvent(event)
	}
}
