package schemas

import (
	"encoding/json"
	"os"
	"strings"
	"testing"
)

func TestParseSessionStartEvent(t *testing.T) {
	// Create a temporary directory for testing
	tmpDir, err := os.MkdirTemp("", "hooks_test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tmpDir)

	tests := []struct {
		name    string
		json    string
		want    SessionStartEvent
		wantErr bool
	}{
		{
			name: "valid startup event",
			json: `{
				"session_id": "test-session-123",
				"transcript_path": "transcript.jsonl",
				"cwd": "` + tmpDir + `",
				"hook_event_name": "SessionStart",
				"source": "startup"
			}`,
			want: SessionStartEvent{
				BaseHookEvent: BaseHookEvent{
					SessionID:      "test-session-123",
					TranscriptPath: "transcript.jsonl",
					CWD:            tmpDir,
					HookEventName:  "SessionStart",
				},
				Source: "startup",
			},
			wantErr: false,
		},
		{
			name: "valid resume event",
			json: `{
				"session_id": "test-session-456",
				"transcript_path": "transcript2.jsonl",
				"cwd": "` + tmpDir + `",
				"hook_event_name": "SessionStart",
				"source": "resume"
			}`,
			want: SessionStartEvent{
				BaseHookEvent: BaseHookEvent{
					SessionID:      "test-session-456",
					TranscriptPath: "transcript2.jsonl",
					CWD:            tmpDir,
					HookEventName:  "SessionStart",
				},
				Source: "resume",
			},
			wantErr: false,
		},
		{
			name: "invalid source",
			json: `{
				"session_id": "test-session-789",
				"transcript_path": "transcript3.jsonl",
				"cwd": "` + tmpDir + `",
				"hook_event_name": "SessionStart",
				"source": "invalid"
			}`,
			wantErr: true,
		},
		{
			name: "missing required field",
			json: `{
				"transcript_path": "transcript.jsonl",
				"cwd": "` + tmpDir + `",
				"hook_event_name": "SessionStart",
				"source": "startup"
			}`,
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			event, err := ParseHookEvent([]byte(tt.json))
			if (err != nil) != tt.wantErr {
				t.Errorf("ParseHookEvent() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !tt.wantErr {
				got, ok := event.(SessionStartEvent)
				if !ok {
					t.Errorf("Expected SessionStartEvent, got %T", event)
					return
				}
				if got != tt.want {
					t.Errorf("ParseHookEvent() = %v, want %v", got, tt.want)
				}
			}
		})
	}
}

func TestParseUserPromptSubmitEvent(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "hooks_test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tmpDir)

	tests := []struct {
		name    string
		json    string
		want    UserPromptSubmitEvent
		wantErr bool
	}{
		{
			name: "valid event",
			json: `{
				"session_id": "test-session-123",
				"transcript_path": "transcript.jsonl",
				"cwd": "` + tmpDir + `",
				"hook_event_name": "UserPromptSubmit",
				"prompt": "Create a function to calculate fibonacci"
			}`,
			want: UserPromptSubmitEvent{
				BaseHookEvent: BaseHookEvent{
					SessionID:      "test-session-123",
					TranscriptPath: "transcript.jsonl",
					CWD:            tmpDir,
					HookEventName:  "UserPromptSubmit",
				},
				Prompt: "Create a function to calculate fibonacci",
			},
			wantErr: false,
		},
		{
			name: "empty prompt",
			json: `{
				"session_id": "test-session-123",
				"transcript_path": "transcript.jsonl",
				"cwd": "` + tmpDir + `",
				"hook_event_name": "UserPromptSubmit",
				"prompt": ""
			}`,
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			event, err := ParseHookEvent([]byte(tt.json))
			if (err != nil) != tt.wantErr {
				t.Errorf("ParseHookEvent() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !tt.wantErr {
				got, ok := event.(UserPromptSubmitEvent)
				if !ok {
					t.Errorf("Expected UserPromptSubmitEvent, got %T", event)
					return
				}
				if got != tt.want {
					t.Errorf("ParseHookEvent() = %v, want %v", got, tt.want)
				}
			}
		})
	}
}

func TestParsePreToolUseEvent(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "hooks_test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tmpDir)

	jsonStr := `{
		"session_id": "test-session-123",
		"transcript_path": "transcript.jsonl",
		"cwd": "` + tmpDir + `",
		"hook_event_name": "PreToolUse",
		"tool_name": "Bash",
		"tool_input": {
			"command": "ls -la",
			"timeout": 5000
		}
	}`

	event, err := ParseHookEvent([]byte(jsonStr))
	if err != nil {
		t.Fatalf("ParseHookEvent() error = %v", err)
	}

	got, ok := event.(PreToolUseEvent)
	if !ok {
		t.Fatalf("Expected PreToolUseEvent, got %T", event)
	}

	if got.ToolName != "Bash" {
		t.Errorf("ToolName = %v, want %v", got.ToolName, "Bash")
	}

	// Check tool input
	command, ok := got.ToolInput["command"].(string)
	if !ok || command != "ls -la" {
		t.Errorf("ToolInput command = %v, want 'ls -la'", got.ToolInput["command"])
	}

	timeout, ok := got.ToolInput["timeout"].(float64)
	if !ok || timeout != 5000 {
		t.Errorf("ToolInput timeout = %v, want 5000", got.ToolInput["timeout"])
	}
}

func TestParsePostToolUseEvent(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "hooks_test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tmpDir)

	jsonStr := `{
		"session_id": "test-session-123",
		"transcript_path": "transcript.jsonl",
		"cwd": "` + tmpDir + `",
		"hook_event_name": "PostToolUse",
		"tool_name": "Read",
		"tool_input": {
			"file_path": "/tmp/test.txt"
		},
		"tool_response": {
			"content": "File contents here",
			"status": "success"
		}
	}`

	event, err := ParseHookEvent([]byte(jsonStr))
	if err != nil {
		t.Fatalf("ParseHookEvent() error = %v", err)
	}

	got, ok := event.(PostToolUseEvent)
	if !ok {
		t.Fatalf("Expected PostToolUseEvent, got %T", event)
	}

	if got.ToolName != "Read" {
		t.Errorf("ToolName = %v, want %v", got.ToolName, "Read")
	}

	// Check tool response
	content, ok := got.ToolResponse["content"].(string)
	if !ok || content != "File contents here" {
		t.Errorf("ToolResponse content = %v, want 'File contents here'", got.ToolResponse["content"])
	}

	status, ok := got.ToolResponse["status"].(string)
	if !ok || status != "success" {
		t.Errorf("ToolResponse status = %v, want 'success'", got.ToolResponse["status"])
	}
}

func TestParseNotificationEvent(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "hooks_test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tmpDir)

	jsonStr := `{
		"session_id": "test-session-123",
		"transcript_path": "transcript.jsonl",
		"cwd": "` + tmpDir + `",
		"hook_event_name": "Notification",
		"message": "Task completed successfully"
	}`

	event, err := ParseHookEvent([]byte(jsonStr))
	if err != nil {
		t.Fatalf("ParseHookEvent() error = %v", err)
	}

	got, ok := event.(NotificationEvent)
	if !ok {
		t.Fatalf("Expected NotificationEvent, got %T", event)
	}

	if got.Message != "Task completed successfully" {
		t.Errorf("Message = %v, want %v", got.Message, "Task completed successfully")
	}
}

func TestParseStopEvent(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "hooks_test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tmpDir)

	tests := []struct {
		name string
		json string
		want StopEvent
	}{
		{
			name: "with stop hook active",
			json: `{
				"session_id": "test-session-123",
				"transcript_path": "transcript.jsonl",
				"cwd": "` + tmpDir + `",
				"hook_event_name": "Stop",
				"stop_hook_active": true
			}`,
			want: StopEvent{
				BaseHookEvent: BaseHookEvent{
					SessionID:      "test-session-123",
					TranscriptPath: "transcript.jsonl",
					CWD:            tmpDir,
					HookEventName:  "Stop",
				},
				StopHookActive: true,
			},
		},
		{
			name: "without stop hook active",
			json: `{
				"session_id": "test-session-123",
				"transcript_path": "transcript.jsonl",
				"cwd": "` + tmpDir + `",
				"hook_event_name": "Stop"
			}`,
			want: StopEvent{
				BaseHookEvent: BaseHookEvent{
					SessionID:      "test-session-123",
					TranscriptPath: "transcript.jsonl",
					CWD:            tmpDir,
					HookEventName:  "Stop",
				},
				StopHookActive: false,
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			event, err := ParseHookEvent([]byte(tt.json))
			if err != nil {
				t.Fatalf("ParseHookEvent() error = %v", err)
			}

			got, ok := event.(StopEvent)
			if !ok {
				t.Fatalf("Expected StopEvent, got %T", event)
			}

			if got != tt.want {
				t.Errorf("ParseHookEvent() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestParseSubagentStopEvent(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "hooks_test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tmpDir)

	jsonStr := `{
		"session_id": "test-session-123",
		"transcript_path": "transcript.jsonl",
		"cwd": "` + tmpDir + `",
		"hook_event_name": "SubagentStop",
		"stop_hook_active": true
	}`

	event, err := ParseHookEvent([]byte(jsonStr))
	if err != nil {
		t.Fatalf("ParseHookEvent() error = %v", err)
	}

	got, ok := event.(SubagentStopEvent)
	if !ok {
		t.Fatalf("Expected SubagentStopEvent, got %T", event)
	}

	if got.StopHookActive != true {
		t.Errorf("StopHookActive = %v, want %v", got.StopHookActive, true)
	}
}

func TestParsePreCompactEvent(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "hooks_test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tmpDir)

	tests := []struct {
		name string
		json string
		want PreCompactEvent
	}{
		{
			name: "manual trigger with instructions",
			json: `{
				"session_id": "test-session-123",
				"transcript_path": "transcript.jsonl",
				"cwd": "` + tmpDir + `",
				"hook_event_name": "PreCompact",
				"trigger": "manual",
				"custom_instructions": "Keep security-related discussions"
			}`,
			want: PreCompactEvent{
				BaseHookEvent: BaseHookEvent{
					SessionID:      "test-session-123",
					TranscriptPath: "transcript.jsonl",
					CWD:            tmpDir,
					HookEventName:  "PreCompact",
				},
				Trigger:            "manual",
				CustomInstructions: "Keep security-related discussions",
			},
		},
		{
			name: "auto trigger without instructions",
			json: `{
				"session_id": "test-session-123",
				"transcript_path": "transcript.jsonl",
				"cwd": "` + tmpDir + `",
				"hook_event_name": "PreCompact",
				"trigger": "auto"
			}`,
			want: PreCompactEvent{
				BaseHookEvent: BaseHookEvent{
					SessionID:      "test-session-123",
					TranscriptPath: "transcript.jsonl",
					CWD:            tmpDir,
					HookEventName:  "PreCompact",
				},
				Trigger:            "auto",
				CustomInstructions: "",
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			event, err := ParseHookEvent([]byte(tt.json))
			if err != nil {
				t.Fatalf("ParseHookEvent() error = %v", err)
			}

			got, ok := event.(PreCompactEvent)
			if !ok {
				t.Fatalf("Expected PreCompactEvent, got %T", event)
			}

			if got != tt.want {
				t.Errorf("ParseHookEvent() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestGetBaseHookEvent(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "hooks_test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tmpDir)

	base := BaseHookEvent{
		SessionID:      "test-session",
		TranscriptPath: "transcript.jsonl",
		CWD:            tmpDir,
		HookEventName:  "Test",
	}

	tests := []struct {
		name  string
		event interface{}
		want  *BaseHookEvent
	}{
		{
			name:  "SessionStartEvent",
			event: SessionStartEvent{BaseHookEvent: base, Source: "startup"},
			want:  &base,
		},
		{
			name:  "UserPromptSubmitEvent",
			event: UserPromptSubmitEvent{BaseHookEvent: base, Prompt: "test"},
			want:  &base,
		},
		{
			name:  "PreToolUseEvent",
			event: PreToolUseEvent{BaseHookEvent: base, ToolName: "Bash", ToolInput: ToolInput{}},
			want:  &base,
		},
		{
			name:  "PostToolUseEvent",
			event: PostToolUseEvent{BaseHookEvent: base, ToolName: "Read", ToolInput: ToolInput{}, ToolResponse: ToolResponse{}},
			want:  &base,
		},
		{
			name:  "NotificationEvent",
			event: NotificationEvent{BaseHookEvent: base, Message: "test"},
			want:  &base,
		},
		{
			name:  "StopEvent",
			event: StopEvent{BaseHookEvent: base},
			want:  &base,
		},
		{
			name:  "SubagentStopEvent",
			event: SubagentStopEvent{BaseHookEvent: base},
			want:  &base,
		},
		{
			name:  "PreCompactEvent",
			event: PreCompactEvent{BaseHookEvent: base, Trigger: "manual"},
			want:  &base,
		},
		{
			name:  "unknown type",
			event: "unknown",
			want:  nil,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := GetBaseHookEvent(tt.event)
			if tt.want == nil {
				if got != nil {
					t.Errorf("GetBaseHookEvent() = %v, want nil", got)
				}
			} else {
				if got == nil {
					t.Errorf("GetBaseHookEvent() = nil, want %v", tt.want)
				} else if *got != *tt.want {
					t.Errorf("GetBaseHookEvent() = %v, want %v", got, tt.want)
				}
			}
		})
	}
}

func TestParseHookEvent_InvalidJSON(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "hooks_test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tmpDir)

	tests := []struct {
		name    string
		json    string
		wantErr string
	}{
		{
			name:    "invalid JSON",
			json:    `{"invalid json`,
			wantErr: "failed to parse base event",
		},
		{
			name:    "unknown event type",
			json:    `{"session_id": "test", "transcript_path": "t.jsonl", "cwd": "` + tmpDir + `", "hook_event_name": "UnknownEvent"}`,
			wantErr: "validation failed",
		},
		{
			name:    "missing hook_event_name",
			json:    `{"session_id": "test", "transcript_path": "t.jsonl", "cwd": "` + tmpDir + `"}`,
			wantErr: "validation failed",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			_, err := ParseHookEvent([]byte(tt.json))
			if err == nil {
				t.Errorf("ParseHookEvent() expected error containing %q, got nil", tt.wantErr)
			} else if !strings.Contains(err.Error(), tt.wantErr) {
				t.Errorf("ParseHookEvent() error = %v, want error containing %q", err, tt.wantErr)
			}
		})
	}
}

func TestHookOutput_JSON(t *testing.T) {
	output := HookOutput{
		Continue:       true,
		StopReason:     "User requested stop",
		SuppressOutput: false,
		Decision:       "approve",
		Reason:         "All checks passed",
		HookSpecificOutput: map[string]interface{}{
			"custom_field": "custom_value",
		},
	}

	// Marshal
	data, err := json.Marshal(output)
	if err != nil {
		t.Fatalf("Failed to marshal HookOutput: %v", err)
	}

	// Unmarshal
	var got HookOutput
	if err := json.Unmarshal(data, &got); err != nil {
		t.Fatalf("Failed to unmarshal HookOutput: %v", err)
	}

	// Compare
	if got.Continue != output.Continue {
		t.Errorf("Continue = %v, want %v", got.Continue, output.Continue)
	}
	if got.StopReason != output.StopReason {
		t.Errorf("StopReason = %v, want %v", got.StopReason, output.StopReason)
	}
	if got.Decision != output.Decision {
		t.Errorf("Decision = %v, want %v", got.Decision, output.Decision)
	}
	if got.Reason != output.Reason {
		t.Errorf("Reason = %v, want %v", got.Reason, output.Reason)
	}
}

func TestPreToolUseDecision_Validation(t *testing.T) {
	tests := []struct {
		name    string
		json    string
		wantErr bool
	}{
		{
			name: "valid allow decision",
			json: `{
				"hookEventName": "PreToolUse",
				"permissionDecision": "allow",
				"permissionDecisionReason": "Tool is safe to use"
			}`,
			wantErr: false,
		},
		{
			name: "valid deny decision",
			json: `{
				"hookEventName": "PreToolUse",
				"permissionDecision": "deny",
				"permissionDecisionReason": "Tool access not permitted"
			}`,
			wantErr: false,
		},
		{
			name: "valid ask decision",
			json: `{
				"hookEventName": "PreToolUse",
				"permissionDecision": "ask"
			}`,
			wantErr: false,
		},
		{
			name: "invalid permission decision",
			json: `{
				"hookEventName": "PreToolUse",
				"permissionDecision": "invalid"
			}`,
			wantErr: true,
		},
		{
			name: "wrong hook event name",
			json: `{
				"hookEventName": "WrongName",
				"permissionDecision": "allow"
			}`,
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var decision PreToolUseDecision
			err := json.Unmarshal([]byte(tt.json), &decision)
			if err != nil && !tt.wantErr {
				t.Fatalf("Failed to unmarshal: %v", err)
			}

			if !tt.wantErr {
				err = Validate.Struct(decision)
				if (err != nil) != tt.wantErr {
					t.Errorf("Validation error = %v, wantErr %v", err, tt.wantErr)
				}
			}
		})
	}
}

func TestBaseHookEvent_RequiredFields(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "hooks_test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tmpDir)

	tests := []struct {
		name    string
		event   BaseHookEvent
		wantErr bool
	}{
		{
			name: "all fields valid",
			event: BaseHookEvent{
				SessionID:      "session-123",
				TranscriptPath: "transcript.jsonl",
				CWD:            tmpDir,
				HookEventName:  "SessionStart",
			},
			wantErr: false,
		},
		{
			name: "empty session ID",
			event: BaseHookEvent{
				SessionID:      "",
				TranscriptPath: "transcript.jsonl",
				CWD:            tmpDir,
				HookEventName:  "SessionStart",
			},
			wantErr: true,
		},
		{
			name: "invalid hook event name",
			event: BaseHookEvent{
				SessionID:      "session-123",
				TranscriptPath: "transcript.jsonl",
				CWD:            tmpDir,
				HookEventName:  "InvalidEvent",
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := Validate.Struct(tt.event)
			if (err != nil) != tt.wantErr {
				t.Errorf("Validation error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

func TestGetBaseHookEvent_PointerTypes(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "hooks_test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tmpDir)

	base := BaseHookEvent{
		SessionID:      "test-session",
		TranscriptPath: "transcript.jsonl",
		CWD:            tmpDir,
		HookEventName:  "Test",
	}

	// Test with pointer types
	sessionStart := &SessionStartEvent{BaseHookEvent: base, Source: "startup"}
	got := GetBaseHookEvent(sessionStart)
	if got == nil || *got != base {
		t.Errorf("GetBaseHookEvent(&SessionStartEvent) failed")
	}

	userPrompt := &UserPromptSubmitEvent{BaseHookEvent: base, Prompt: "test"}
	got = GetBaseHookEvent(userPrompt)
	if got == nil || *got != base {
		t.Errorf("GetBaseHookEvent(&UserPromptSubmitEvent) failed")
	}

	preToolUse := &PreToolUseEvent{BaseHookEvent: base, ToolName: "Bash", ToolInput: ToolInput{}}
	got = GetBaseHookEvent(preToolUse)
	if got == nil || *got != base {
		t.Errorf("GetBaseHookEvent(&PreToolUseEvent) failed")
	}

	postToolUse := &PostToolUseEvent{BaseHookEvent: base, ToolName: "Read", ToolInput: ToolInput{}, ToolResponse: ToolResponse{}}
	got = GetBaseHookEvent(postToolUse)
	if got == nil || *got != base {
		t.Errorf("GetBaseHookEvent(&PostToolUseEvent) failed")
	}

	notification := &NotificationEvent{BaseHookEvent: base, Message: "test"}
	got = GetBaseHookEvent(notification)
	if got == nil || *got != base {
		t.Errorf("GetBaseHookEvent(&NotificationEvent) failed")
	}

	stop := &StopEvent{BaseHookEvent: base}
	got = GetBaseHookEvent(stop)
	if got == nil || *got != base {
		t.Errorf("GetBaseHookEvent(&StopEvent) failed")
	}

	subagentStop := &SubagentStopEvent{BaseHookEvent: base}
	got = GetBaseHookEvent(subagentStop)
	if got == nil || *got != base {
		t.Errorf("GetBaseHookEvent(&SubagentStopEvent) failed")
	}

	preCompact := &PreCompactEvent{BaseHookEvent: base, Trigger: "manual"}
	got = GetBaseHookEvent(preCompact)
	if got == nil || *got != base {
		t.Errorf("GetBaseHookEvent(&PreCompactEvent) failed")
	}
}
