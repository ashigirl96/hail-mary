package schemas

import (
	"encoding/json"
	"fmt"
	"strings"
)

// BaseMessage contains common fields for all message types
type BaseMessage struct {
	ParentUUID  *string `json:"parentUuid,omitempty"`
	IsSidechain bool    `json:"isSidechain"`
	UserType    string  `json:"userType" validate:"required"`
	CWD         string  `json:"cwd" validate:"required"`
	SessionID   string  `json:"sessionId" validate:"required"`
	Version     string  `json:"version" validate:"required"`
	UUID        string  `json:"uuid" validate:"required"`
	Timestamp   string  `json:"timestamp" validate:"required"`
}

// Content types - simplified without interface
type TextContent struct {
	Type string `json:"type" validate:"required,eq=text"`
	Text string `json:"text" validate:"required"`
}

type ToolUseContent struct {
	Type  string          `json:"type" validate:"required,eq=tool_use"`
	ID    string          `json:"id" validate:"required"`
	Name  string          `json:"name" validate:"required"`
	Input json.RawMessage `json:"input" validate:"required"`
}

type ToolResultContent struct {
	Type      string          `json:"type" validate:"required,eq=tool_result"`
	ToolUseID string          `json:"tool_use_id" validate:"required"`
	Content   json.RawMessage `json:"content,omitempty"`
	IsError   *bool           `json:"is_error,omitempty"`
}

type ThinkingContent struct {
	Type      string `json:"type" validate:"required,eq=thinking"`
	Thinking  string `json:"thinking" validate:"required"`
	Signature string `json:"signature" validate:"required"`
}

type ImageSource struct {
	Type      string  `json:"type" validate:"required"`
	Data      *string `json:"data,omitempty"`
	MediaType *string `json:"media_type,omitempty"`
}

type ImageContent struct {
	Type   string      `json:"type" validate:"required,eq=image"`
	Source ImageSource `json:"source" validate:"required"`
}

// Usage information
type Usage struct {
	InputTokens              int32          `json:"input_tokens" validate:"min=0"`
	CacheCreationInputTokens int32          `json:"cache_creation_input_tokens" validate:"min=0"`
	CacheReadInputTokens     int32          `json:"cache_read_input_tokens" validate:"min=0"`
	OutputTokens             int32          `json:"output_tokens" validate:"min=0"`
	ServiceTier              *string        `json:"service_tier,omitempty"`
	ServerToolUse            *ServerToolUse `json:"server_tool_use,omitempty"`
}

type ServerToolUse struct {
	WebSearchRequests int32 `json:"web_search_requests" validate:"min=0"`
}

// Message content types - simplified
type UserMessageContent struct {
	Role    string          `json:"role" validate:"required,eq=user"`
	Content json.RawMessage `json:"content" validate:"required"`
}

type AssistantMessageContent struct {
	ID           string          `json:"id" validate:"required"`
	Type         string          `json:"type" validate:"required,eq=message"`
	Role         string          `json:"role" validate:"required,eq=assistant"`
	Model        string          `json:"model" validate:"required"`
	Content      json.RawMessage `json:"content" validate:"required"` // Simplified to RawMessage
	StopReason   *string         `json:"stop_reason,omitempty"`
	StopSequence *string         `json:"stop_sequence,omitempty"`
	Usage        Usage           `json:"usage" validate:"required"`
}

// Session message types - no interface needed
type SummaryMessage struct {
	Type     string `json:"type" validate:"required,eq=summary"`
	Summary  string `json:"summary" validate:"required"`
	LeafUUID string `json:"leafUuid" validate:"required"`
}

type SystemMessage struct {
	Type        string  `json:"type" validate:"required,eq=system"`
	BaseMessage         // Embedded struct
	Content     string  `json:"content" validate:"required"`
	IsMeta      bool    `json:"isMeta"`
	ToolUseID   *string `json:"toolUseID,omitempty"`
	Level       *string `json:"level,omitempty"`
	GitBranch   *string `json:"gitBranch,omitempty"`
	RequestID   *string `json:"requestId,omitempty"`
}

type UserMessage struct {
	Type                      string             `json:"type" validate:"required,eq=user"`
	BaseMessage                                  // Embedded struct
	Message                   UserMessageContent `json:"message" validate:"required"`
	GitBranch                 *string            `json:"gitBranch,omitempty"`
	IsMeta                    *bool              `json:"isMeta,omitempty"`
	IsCompactSummary          *bool              `json:"isCompactSummary,omitempty"`
	IsVisibleInTranscriptOnly *bool              `json:"isVisibleInTranscriptOnly,omitempty"`
	ToolUseResult             *json.RawMessage   `json:"toolUseResult,omitempty"`
}

type AssistantMessage struct {
	Type              string                  `json:"type" validate:"required,eq=assistant"`
	BaseMessage                               // Embedded struct
	Message           AssistantMessageContent `json:"message" validate:"required"`
	RequestID         *string                 `json:"requestId,omitempty"`
	GitBranch         *string                 `json:"gitBranch,omitempty"`
	IsAPIErrorMessage *bool                   `json:"isApiErrorMessage,omitempty"`
}

// UnmarshalSessionMessage unmarshals JSON into the appropriate message type
func UnmarshalSessionMessage(data []byte) (interface{}, error) {
	var typeOnly struct {
		Type string `json:"type"`
	}

	if err := json.Unmarshal(data, &typeOnly); err != nil {
		return nil, err
	}

	var msg interface{}

	switch typeOnly.Type {
	case "summary":
		var m SummaryMessage
		if err := json.Unmarshal(data, &m); err != nil {
			return nil, fmt.Errorf("failed to parse summary message: %w", err)
		}
		msg = m

	case "system":
		var m SystemMessage
		if err := json.Unmarshal(data, &m); err != nil {
			return nil, fmt.Errorf("failed to parse system message: %w", err)
		}
		msg = m

	case "user":
		var m UserMessage
		if err := json.Unmarshal(data, &m); err != nil {
			return nil, fmt.Errorf("failed to parse user message: %w", err)
		}
		msg = m

	case "assistant":
		var m AssistantMessage
		if err := json.Unmarshal(data, &m); err != nil {
			return nil, fmt.Errorf("failed to parse assistant message: %w", err)
		}
		msg = m

	default:
		return nil, fmt.Errorf("unknown message type: %s", typeOnly.Type)
	}

	// Validate the message
	if err := Validate.Struct(msg); err != nil {
		return nil, fmt.Errorf("validation failed: %w", err)
	}

	return msg, nil
}

// Helper functions for extracting content text
func ExtractContentText(msg interface{}) string {
	switch m := msg.(type) {
	case SummaryMessage:
		return m.Summary
	case SystemMessage:
		return m.Content
	case UserMessage:
		return extractUserMessageContent(m.Message.Content)
	case AssistantMessage:
		return extractAssistantMessageContent(m.Message.Content)
	default:
		return ""
	}
}

func extractUserMessageContent(content json.RawMessage) string {
	// Try as string first
	var str string
	if err := json.Unmarshal(content, &str); err == nil {
		return str
	}

	// Try as array
	var contents []json.RawMessage
	if err := json.Unmarshal(content, &contents); err != nil {
		return ""
	}

	var texts []string
	for _, c := range contents {
		var typeOnly struct {
			Type string `json:"type"`
			Text string `json:"text,omitempty"`
		}
		if err := json.Unmarshal(c, &typeOnly); err == nil && typeOnly.Type == "text" {
			texts = append(texts, typeOnly.Text)
		}
	}

	return strings.Join(texts, "\n")
}

func extractAssistantMessageContent(content json.RawMessage) string {
	var contents []json.RawMessage
	if err := json.Unmarshal(content, &contents); err != nil {
		return ""
	}

	var texts []string
	for _, c := range contents {
		var typeOnly struct {
			Type     string `json:"type"`
			Text     string `json:"text,omitempty"`
			Thinking string `json:"thinking,omitempty"`
			Name     string `json:"name,omitempty"`
		}

		if err := json.Unmarshal(c, &typeOnly); err != nil {
			continue
		}

		switch typeOnly.Type {
		case "text":
			texts = append(texts, typeOnly.Text)
		case "thinking":
			texts = append(texts, typeOnly.Thinking)
		case "tool_use":
			texts = append(texts, formatToolUse(c))
		}
	}

	return strings.Join(texts, "\n")
}

func formatToolUse(data json.RawMessage) string {
	var tool struct {
		Name  string          `json:"name"`
		Input json.RawMessage `json:"input"`
	}

	if err := json.Unmarshal(data, &tool); err != nil {
		return ""
	}

	// Extract meaningful information based on tool type
	var inputMap map[string]interface{}
	if err := json.Unmarshal(tool.Input, &inputMap); err == nil {
		switch tool.Name {
		case "Bash":
			if cmd, ok := inputMap["command"].(string); ok {
				if len(cmd) > 50 {
					return fmt.Sprintf("%s: %s...", tool.Name, cmd[:50])
				}
				return fmt.Sprintf("%s: %s", tool.Name, cmd)
			}
		case "Read", "Write", "Edit":
			if path, ok := inputMap["file_path"].(string); ok {
				parts := strings.Split(path, "/")
				return fmt.Sprintf("%s: %s", tool.Name, parts[len(parts)-1])
			}
		case "Grep":
			if pattern, ok := inputMap["pattern"].(string); ok {
				if len(pattern) > 30 {
					return fmt.Sprintf("%s: %s...", tool.Name, pattern[:30])
				}
				return fmt.Sprintf("%s: %s", tool.Name, pattern)
			}
		default:
			if desc, ok := inputMap["description"].(string); ok {
				if len(desc) > 40 {
					return fmt.Sprintf("%s: %s...", tool.Name, desc[:40])
				}
				return fmt.Sprintf("%s: %s", tool.Name, desc)
			}
		}
	}

	return tool.Name
}

// Helper functions for common message operations
func GetMessageType(msg interface{}) string {
	switch msg.(type) {
	case SummaryMessage:
		return "summary"
	case SystemMessage:
		return "system"
	case UserMessage:
		return "user"
	case AssistantMessage:
		return "assistant"
	default:
		return ""
	}
}

func GetSessionID(msg interface{}) string {
	switch m := msg.(type) {
	case SystemMessage:
		return m.SessionID
	case UserMessage:
		return m.SessionID
	case AssistantMessage:
		return m.SessionID
	default:
		return ""
	}
}

func GetUUID(msg interface{}) string {
	switch m := msg.(type) {
	case SummaryMessage:
		return m.LeafUUID
	case SystemMessage:
		return m.UUID
	case UserMessage:
		return m.UUID
	case AssistantMessage:
		return m.UUID
	default:
		return ""
	}
}

func GetTimestamp(msg interface{}) string {
	switch m := msg.(type) {
	case SystemMessage:
		return m.Timestamp
	case UserMessage:
		return m.Timestamp
	case AssistantMessage:
		return m.Timestamp
	default:
		return ""
	}
}

func GetSearchableText(msg interface{}) string {
	content := ExtractContentText(msg)
	sessionID := GetSessionID(msg)
	uuid := GetUUID(msg)

	parts := []string{content}
	if sessionID != "" {
		parts = append(parts, sessionID)
	}
	if uuid != "" {
		parts = append(parts, uuid)
	}

	return strings.Join(parts, " ")
}

// Backward compatibility functions
type SessionMessage interface{} // Empty interface for compatibility

func (s SummaryMessage) GetType() string        { return "summary" }
func (s SummaryMessage) GetContentText() string { return s.Summary }
func (s SummaryMessage) GetUUID() *string       { return &s.LeafUUID }
func (s SummaryMessage) GetTimestamp() *string  { return nil }
func (s SummaryMessage) GetSessionID() *string  { return nil }
func (s SummaryMessage) GetCWD() *string        { return nil }
func (s SummaryMessage) GetSearchableText() string {
	return fmt.Sprintf("%s %s", s.Summary, s.LeafUUID)
}

func (s SystemMessage) GetType() string        { return "system" }
func (s SystemMessage) GetContentText() string { return s.Content }
func (s SystemMessage) GetUUID() *string       { return &s.UUID }
func (s SystemMessage) GetTimestamp() *string  { return &s.Timestamp }
func (s SystemMessage) GetSessionID() *string  { return &s.SessionID }
func (s SystemMessage) GetCWD() *string        { return &s.CWD }
func (s SystemMessage) GetSearchableText() string {
	return fmt.Sprintf("%s %s %s", s.Content, s.SessionID, s.UUID)
}

func (u UserMessage) GetType() string        { return "user" }
func (u UserMessage) GetContentText() string { return extractUserMessageContent(u.Message.Content) }
func (u UserMessage) GetUUID() *string       { return &u.UUID }
func (u UserMessage) GetTimestamp() *string  { return &u.Timestamp }
func (u UserMessage) GetSessionID() *string  { return &u.SessionID }
func (u UserMessage) GetCWD() *string        { return &u.CWD }
func (u UserMessage) GetSearchableText() string {
	return fmt.Sprintf("%s %s %s", u.GetContentText(), u.SessionID, u.UUID)
}

func (a AssistantMessage) GetType() string { return "assistant" }
func (a AssistantMessage) GetContentText() string {
	return extractAssistantMessageContent(a.Message.Content)
}
func (a AssistantMessage) GetUUID() *string      { return &a.UUID }
func (a AssistantMessage) GetTimestamp() *string { return &a.Timestamp }
func (a AssistantMessage) GetSessionID() *string { return &a.SessionID }
func (a AssistantMessage) GetCWD() *string       { return &a.CWD }
func (a AssistantMessage) GetSearchableText() string {
	return fmt.Sprintf("%s %s %s", a.GetContentText(), a.SessionID, a.UUID)
}
