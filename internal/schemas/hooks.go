package schemas

import (
	"encoding/json"
	"fmt"
)

// BaseHookEvent contains common fields for all hook events
type BaseHookEvent struct {
	SessionID      string `json:"session_id" validate:"required"`
	TranscriptPath string `json:"transcript_path" validate:"required,filepath"`
	CWD            string `json:"cwd" validate:"required,dir"`
	HookEventName  string `json:"hook_event_name" validate:"required,oneof=UserPromptSubmit PostToolUse Stop SubagentStop Notification PreCompact PreToolUse SessionStart"`
}

// SessionStartEvent is sent when a Claude session starts
type SessionStartEvent struct {
	BaseHookEvent
	Source string `json:"source" validate:"required,oneof=startup resume clear"`
}

// UserPromptSubmitEvent is sent when user submits a prompt
type UserPromptSubmitEvent struct {
	BaseHookEvent
	Prompt string `json:"prompt" validate:"required"`
}

// ToolInput represents generic tool input data
type ToolInput map[string]interface{}

// ToolResponse represents generic tool response data
type ToolResponse map[string]interface{}

// PreToolUseEvent is sent before a tool is used
type PreToolUseEvent struct {
	BaseHookEvent
	ToolName  string    `json:"tool_name" validate:"required"`
	ToolInput ToolInput `json:"tool_input" validate:"required"`
}

// PostToolUseEvent is sent after a tool is used
type PostToolUseEvent struct {
	BaseHookEvent
	ToolName     string       `json:"tool_name" validate:"required"`
	ToolInput    ToolInput    `json:"tool_input" validate:"required"`
	ToolResponse ToolResponse `json:"tool_response" validate:"required"`
}

// NotificationEvent is sent for notifications
type NotificationEvent struct {
	BaseHookEvent
	Message string `json:"message" validate:"required"`
}

// StopEvent is sent when Claude stops
type StopEvent struct {
	BaseHookEvent
	StopHookActive bool `json:"stop_hook_active,omitempty"`
}

// SubagentStopEvent is sent when a subagent stops
type SubagentStopEvent struct {
	BaseHookEvent
	StopHookActive bool `json:"stop_hook_active,omitempty"`
}

// PreCompactEvent is sent before compaction
type PreCompactEvent struct {
	BaseHookEvent
	Trigger            string `json:"trigger" validate:"required,oneof=manual auto"`
	CustomInstructions string `json:"custom_instructions,omitempty"`
}

// HookOutput represents the response from a hook
type HookOutput struct {
	Continue           bool                   `json:"continue,omitempty"`
	StopReason         string                 `json:"stopReason,omitempty"`
	SuppressOutput     bool                   `json:"suppressOutput,omitempty"`
	Decision           string                 `json:"decision,omitempty" validate:"omitempty,oneof=approve block allow deny ask"`
	Reason             string                 `json:"reason,omitempty"`
	HookSpecificOutput map[string]interface{} `json:"hookSpecificOutput,omitempty"`
}

// PreToolUseDecision represents PreToolUse specific output
type PreToolUseDecision struct {
	HookEventName            string `json:"hookEventName" validate:"required,eq=PreToolUse"`
	PermissionDecision       string `json:"permissionDecision" validate:"required,oneof=allow deny ask"`
	PermissionDecisionReason string `json:"permissionDecisionReason,omitempty"`
}

// UserPromptSubmitDecision represents UserPromptSubmit specific output
type UserPromptSubmitDecision struct {
	HookEventName     string `json:"hookEventName" validate:"required,eq=UserPromptSubmit"`
	AdditionalContext string `json:"additionalContext,omitempty"`
}

// SessionStartDecision represents SessionStart specific output
type SessionStartDecision struct {
	HookEventName     string `json:"hookEventName" validate:"required,eq=SessionStart"`
	AdditionalContext string `json:"additionalContext,omitempty"`
}

// GetBaseHookEvent extracts base event information from any hook event
func GetBaseHookEvent(event interface{}) *BaseHookEvent {
	switch e := event.(type) {
	case *SessionStartEvent:
		return &e.BaseHookEvent
	case *UserPromptSubmitEvent:
		return &e.BaseHookEvent
	case *PreToolUseEvent:
		return &e.BaseHookEvent
	case *PostToolUseEvent:
		return &e.BaseHookEvent
	case *NotificationEvent:
		return &e.BaseHookEvent
	case *StopEvent:
		return &e.BaseHookEvent
	case *SubagentStopEvent:
		return &e.BaseHookEvent
	case *PreCompactEvent:
		return &e.BaseHookEvent
	case SessionStartEvent:
		return &e.BaseHookEvent
	case UserPromptSubmitEvent:
		return &e.BaseHookEvent
	case PreToolUseEvent:
		return &e.BaseHookEvent
	case PostToolUseEvent:
		return &e.BaseHookEvent
	case NotificationEvent:
		return &e.BaseHookEvent
	case StopEvent:
		return &e.BaseHookEvent
	case SubagentStopEvent:
		return &e.BaseHookEvent
	case PreCompactEvent:
		return &e.BaseHookEvent
	default:
		return nil
	}
}

// ParseHookEvent parses and validates a hook event from JSON
func ParseHookEvent(data []byte) (interface{}, error) {
	// First parse to get the event name
	var base BaseHookEvent
	if err := json.Unmarshal(data, &base); err != nil {
		return nil, fmt.Errorf("failed to parse base event: %w", err)
	}

	// Validate base event first to catch missing hook_event_name
	if err := Validate.Struct(base); err != nil {
		return nil, fmt.Errorf("validation failed: %w", err)
	}

	// Parse based on event type
	var event interface{}
	switch base.HookEventName {
	case "SessionStart":
		var e SessionStartEvent
		if err := json.Unmarshal(data, &e); err != nil {
			return nil, fmt.Errorf("failed to parse SessionStart event: %w", err)
		}
		event = e
	case "UserPromptSubmit":
		var e UserPromptSubmitEvent
		if err := json.Unmarshal(data, &e); err != nil {
			return nil, fmt.Errorf("failed to parse UserPromptSubmit event: %w", err)
		}
		event = e
	case "PreToolUse":
		var e PreToolUseEvent
		if err := json.Unmarshal(data, &e); err != nil {
			return nil, fmt.Errorf("failed to parse PreToolUse event: %w", err)
		}
		event = e
	case "PostToolUse":
		var e PostToolUseEvent
		if err := json.Unmarshal(data, &e); err != nil {
			return nil, fmt.Errorf("failed to parse PostToolUse event: %w", err)
		}
		event = e
	case "Notification":
		var e NotificationEvent
		if err := json.Unmarshal(data, &e); err != nil {
			return nil, fmt.Errorf("failed to parse Notification event: %w", err)
		}
		event = e
	case "Stop":
		var e StopEvent
		if err := json.Unmarshal(data, &e); err != nil {
			return nil, fmt.Errorf("failed to parse Stop event: %w", err)
		}
		event = e
	case "SubagentStop":
		var e SubagentStopEvent
		if err := json.Unmarshal(data, &e); err != nil {
			return nil, fmt.Errorf("failed to parse SubagentStop event: %w", err)
		}
		event = e
	case "PreCompact":
		var e PreCompactEvent
		if err := json.Unmarshal(data, &e); err != nil {
			return nil, fmt.Errorf("failed to parse PreCompact event: %w", err)
		}
		event = e
	default:
		return nil, fmt.Errorf("unknown hook event type: %s", base.HookEventName)
	}

	// Validate the event
	if err := Validate.Struct(event); err != nil {
		return nil, fmt.Errorf("validation failed: %w", err)
	}

	return event, nil
}
