package schemas

import (
	"encoding/json"
	"strings"
	"testing"
)

func TestParseUserMessage(t *testing.T) {
	jsonStr := `{
		"type": "user",
		"message": {
			"role": "user",
			"content": "Hello, Claude!"
		},
		"uuid": "test-uuid",
		"timestamp": "2024-01-01T00:00:00Z",
		"sessionId": "test-session",
		"parentUuid": null,
		"isSidechain": false,
		"userType": "external",
		"cwd": "/test",
		"version": "1.0"
	}`

	msg, err := UnmarshalSessionMessage([]byte(jsonStr))
	if err != nil {
		t.Fatalf("failed to unmarshal user message: %v", err)
	}

	if GetMessageType(msg) != "user" {
		t.Errorf("expected type user, got %s", GetMessageType(msg))
	}
	if ExtractContentText(msg) != "Hello, Claude!" {
		t.Errorf("expected content 'Hello, Claude!', got %s", ExtractContentText(msg))
	}
	if GetUUID(msg) != "test-uuid" {
		t.Errorf("expected UUID test-uuid, got %s", GetUUID(msg))
	}
	if GetTimestamp(msg) != "2024-01-01T00:00:00Z" {
		t.Errorf("expected timestamp 2024-01-01T00:00:00Z, got %s", GetTimestamp(msg))
	}
	if GetSessionID(msg) != "test-session" {
		t.Errorf("expected session ID test-session, got %s", GetSessionID(msg))
	}
}

func TestParseUserMessageWithArray(t *testing.T) {
	jsonStr := `{
		"type": "user",
		"message": {
			"role": "user",
			"content": [
				{"type": "text", "text": "First part"},
				{"type": "text", "text": "Second part"}
			]
		},
		"uuid": "test-uuid",
		"timestamp": "2024-01-01T00:00:00Z",
		"sessionId": "test-session",
		"parentUuid": null,
		"isSidechain": false,
		"userType": "external",
		"cwd": "/test",
		"version": "1.0"
	}`

	msg, err := UnmarshalSessionMessage([]byte(jsonStr))
	if err != nil {
		t.Fatalf("failed to unmarshal user message with array: %v", err)
	}

	expected := "First part\nSecond part"
	if ExtractContentText(msg) != expected {
		t.Errorf("expected content '%s', got '%s'", expected, ExtractContentText(msg))
	}
}

func TestParseAssistantMessageWithToolUse(t *testing.T) {
	jsonStr := `{
		"type": "assistant",
		"message": {
			"id": "msg_01",
			"type": "message",
			"role": "assistant",
			"model": "claude-3-5-sonnet",
			"content": [
				{"type": "text", "text": "I'll help you with that."},
				{
					"type": "tool_use",
					"id": "tool_1",
					"name": "read_file",
					"input": {"path": "test.txt"}
				}
			],
			"stop_reason": "tool_use",
			"stop_sequence": null,
			"usage": {
				"input_tokens": 100,
				"cache_creation_input_tokens": 0,
				"cache_read_input_tokens": 0,
				"output_tokens": 50
			}
		},
		"uuid": "assistant-uuid",
		"timestamp": "2024-01-01T00:00:01Z",
		"sessionId": "test-session",
		"parentUuid": "test-uuid",
		"isSidechain": false,
		"userType": "external",
		"cwd": "/test",
		"version": "1.0"
	}`

	msg, err := UnmarshalSessionMessage([]byte(jsonStr))
	if err != nil {
		t.Fatalf("failed to unmarshal assistant message: %v", err)
	}

	if GetMessageType(msg) != "assistant" {
		t.Errorf("expected type assistant, got %s", GetMessageType(msg))
	}
	expected := "I'll help you with that.\nread_file"
	if ExtractContentText(msg) != expected {
		t.Errorf("expected content '%s', got '%s'", expected, ExtractContentText(msg))
	}
}

func TestParseAssistantMessageWithThinking(t *testing.T) {
	jsonStr := `{
		"type": "assistant",
		"message": {
			"id": "msg_02",
			"type": "message",
			"role": "assistant",
			"model": "claude-3-5-sonnet",
			"content": [
				{
					"type": "thinking",
					"thinking": "Let me think about this problem...",
					"signature": "signature"
				},
				{"type": "text", "text": "Here's my answer."}
			],
			"stop_reason": "end_turn",
			"stop_sequence": null,
			"usage": {
				"input_tokens": 100,
				"cache_creation_input_tokens": 0,
				"cache_read_input_tokens": 0,
				"output_tokens": 50
			}
		},
		"uuid": "assistant-uuid-2",
		"timestamp": "2024-01-01T00:00:02Z",
		"sessionId": "test-session",
		"parentUuid": "test-uuid",
		"isSidechain": false,
		"userType": "external",
		"cwd": "/test",
		"version": "1.0"
	}`

	msg, err := UnmarshalSessionMessage([]byte(jsonStr))
	if err != nil {
		t.Fatalf("failed to unmarshal assistant message with thinking: %v", err)
	}

	expected := "Let me think about this problem...\nHere's my answer."
	if ExtractContentText(msg) != expected {
		t.Errorf("expected content '%s', got '%s'", expected, ExtractContentText(msg))
	}
}

func TestParseSystemMessage(t *testing.T) {
	jsonStr := `{
		"type": "system",
		"content": "System notification: Task completed",
		"uuid": "system-uuid",
		"timestamp": "2024-01-01T00:00:03Z",
		"sessionId": "test-session",
		"parentUuid": null,
		"isSidechain": false,
		"userType": "external",
		"cwd": "/test",
		"version": "1.0",
		"isMeta": false
	}`

	msg, err := UnmarshalSessionMessage([]byte(jsonStr))
	if err != nil {
		t.Fatalf("failed to unmarshal system message: %v", err)
	}

	if GetMessageType(msg) != "system" {
		t.Errorf("expected type system, got %s", GetMessageType(msg))
	}
	if ExtractContentText(msg) != "System notification: Task completed" {
		t.Errorf("expected content 'System notification: Task completed', got %s", ExtractContentText(msg))
	}
}

func TestParseSummaryMessage(t *testing.T) {
	jsonStr := `{
		"type": "summary",
		"summary": "This is a summary of the conversation",
		"leafUuid": "leaf-uuid-123"
	}`

	msg, err := UnmarshalSessionMessage([]byte(jsonStr))
	if err != nil {
		t.Fatalf("failed to unmarshal summary message: %v", err)
	}

	if GetMessageType(msg) != "summary" {
		t.Errorf("expected type summary, got %s", GetMessageType(msg))
	}
	if ExtractContentText(msg) != "This is a summary of the conversation" {
		t.Errorf("expected summary content, got %s", ExtractContentText(msg))
	}
	if GetUUID(msg) != "leaf-uuid-123" {
		t.Errorf("expected leaf UUID leaf-uuid-123, got %s", GetUUID(msg))
	}
	// Summary messages don't have timestamps, session IDs, or CWD
	if GetTimestamp(msg) != "" {
		t.Errorf("expected empty timestamp for summary message, got %v", GetTimestamp(msg))
	}
	if GetSessionID(msg) != "" {
		t.Errorf("expected empty session ID for summary message, got %v", GetSessionID(msg))
	}
}

func TestToolResultContent(t *testing.T) {
	tests := []struct {
		name     string
		json     string
		expected string
	}{
		{
			name: "string content",
			json: `{
				"type": "tool_result",
				"tool_use_id": "tool_123",
				"content": "Result text"
			}`,
			expected: "Result text",
		},
		{
			name: "text array content",
			json: `{
				"type": "tool_result",
				"tool_use_id": "tool_124",
				"content": [
					{"type": "text", "text": "Line 1"},
					{"type": "text", "text": "Line 2"}
				]
			}`,
			expected: "Line 1\nLine 2",
		},
		{
			name: "empty content with error",
			json: `{
				"type": "tool_result",
				"tool_use_id": "tool_125",
				"content": "",
				"is_error": true
			}`,
			expected: "[Tool Result: tool_125 (error)]",
		},
		{
			name: "no content",
			json: `{
				"type": "tool_result",
				"tool_use_id": "tool_126"
			}`,
			expected: "[Tool Result: tool_126]",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var content ToolResultContent
			if err := json.Unmarshal([]byte(tt.json), &content); err != nil {
				t.Fatalf("failed to unmarshal tool result content: %v", err)
			}

			// We need to add a helper for tool result content
			var text string
			if len(content.Content) > 0 {
				var str string
				if err := json.Unmarshal(content.Content, &str); err == nil {
					text = str
				} else {
					var textArr []struct {
						Type string `json:"type"`
						Text string `json:"text"`
					}
					if err := json.Unmarshal(content.Content, &textArr); err == nil {
						texts := make([]string, len(textArr))
						for i, t := range textArr {
							texts[i] = t.Text
						}
						text = strings.Join(texts, "\n")
					}
				}
			}

			if text == "" {
				errorSuffix := ""
				if content.IsError != nil && *content.IsError {
					errorSuffix = " (error)"
				}
				text = "[Tool Result: " + content.ToolUseID + errorSuffix + "]"
			}

			if text != tt.expected {
				t.Errorf("expected '%s', got '%s'", tt.expected, text)
			}
		})
	}
}

func TestGetSearchableText(t *testing.T) {
	jsonStr := `{
		"type": "user",
		"message": {
			"role": "user",
			"content": "Search this text"
		},
		"uuid": "uuid-123",
		"timestamp": "2024-01-01T00:00:00Z",
		"sessionId": "session-456",
		"parentUuid": null,
		"isSidechain": false,
		"userType": "external",
		"cwd": "/test",
		"version": "1.0"
	}`

	msg, err := UnmarshalSessionMessage([]byte(jsonStr))
	if err != nil {
		t.Fatalf("failed to unmarshal message: %v", err)
	}

	searchable := GetSearchableText(msg)
	expected := "Search this text session-456 uuid-123"
	if searchable != expected {
		t.Errorf("expected searchable text '%s', got '%s'", expected, searchable)
	}
}

func TestValidation(t *testing.T) {
	tests := []struct {
		name    string
		json    string
		wantErr bool
		errMsg  string
	}{
		{
			name: "valid user message",
			json: `{
				"type": "user",
				"message": {
					"role": "user",
					"content": "Hello"
				},
				"uuid": "test-uuid",
				"timestamp": "2024-01-01T00:00:00Z",
				"sessionId": "test-session",
				"parentUuid": null,
				"isSidechain": false,
				"userType": "external",
				"cwd": "/test",
				"version": "1.0"
			}`,
			wantErr: false,
		},
		{
			name: "missing required field",
			json: `{
				"type": "user",
				"message": {
					"role": "user",
					"content": "Hello"
				},
				"uuid": "test-uuid",
				"timestamp": "2024-01-01T00:00:00Z",
				"parentUuid": null,
				"isSidechain": false,
				"userType": "external",
				"cwd": "/test",
				"version": "1.0"
			}`,
			wantErr: true,
			errMsg:  "validation failed",
		},
		{
			name: "invalid message type",
			json: `{
				"type": "invalid",
				"content": "test"
			}`,
			wantErr: true,
			errMsg:  "unknown message type: invalid",
		},
		{
			name: "invalid role in user message",
			json: `{
				"type": "user",
				"message": {
					"role": "assistant",
					"content": "Hello"
				},
				"uuid": "test-uuid",
				"timestamp": "2024-01-01T00:00:00Z",
				"sessionId": "test-session",
				"parentUuid": null,
				"isSidechain": false,
				"userType": "external",
				"cwd": "/test",
				"version": "1.0"
			}`,
			wantErr: true,
			errMsg:  "validation failed",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			_, err := UnmarshalSessionMessage([]byte(tt.json))
			if tt.wantErr {
				if err == nil {
					t.Errorf("expected error containing '%s', got nil", tt.errMsg)
				} else if tt.errMsg != "" && !strings.Contains(err.Error(), tt.errMsg) {
					t.Errorf("expected error containing '%s', got '%v'", tt.errMsg, err)
				}
			} else if err != nil {
				t.Errorf("unexpected error: %v", err)
			}
		})
	}
}

func TestToolUseContentFormatting(t *testing.T) {
	tests := []struct {
		name     string
		json     string
		expected string
	}{
		{
			name: "Bash command",
			json: `{
				"type": "tool_use",
				"id": "tool_1",
				"name": "Bash",
				"input": {"command": "ls -la /home/user/documents"}
			}`,
			expected: "Bash: ls -la /home/user/documents",
		},
		{
			name: "Bash command truncated",
			json: `{
				"type": "tool_use",
				"id": "tool_2",
				"name": "Bash",
				"input": {"command": "find / -type f -name '*.log' -exec grep -l 'error' {} + | head -20"}
			}`,
			expected: "Bash: find / -type f -name '*.log' -exec grep -l 'error'...",
		},
		{
			name: "Read file",
			json: `{
				"type": "tool_use",
				"id": "tool_3",
				"name": "Read",
				"input": {"file_path": "/home/user/project/src/main.go"}
			}`,
			expected: "Read: main.go",
		},
		{
			name: "Grep pattern",
			json: `{
				"type": "tool_use",
				"id": "tool_4",
				"name": "Grep",
				"input": {"pattern": "func (\\w+)\\s*\\("}
			}`,
			expected: "Grep: func (\\w+)\\s*\\(",
		},
		{
			name: "Grep pattern truncated",
			json: `{
				"type": "tool_use",
				"id": "tool_5",
				"name": "Grep",
				"input": {"pattern": "(?:public|private|protected)\\s+(?:static\\s+)?(?:final\\s+)?\\w+\\s+\\w+\\s*\\([^)]*\\)\\s*\\{"}
			}`,
			expected: "Grep: (?:public|private|protected)\\s...",
		},
		{
			name: "Unknown tool with description",
			json: `{
				"type": "tool_use",
				"id": "tool_6",
				"name": "CustomTool",
				"input": {"description": "Analyze code complexity", "path": "/src"}
			}`,
			expected: "CustomTool: Analyze code complexity",
		},
		{
			name: "Unknown tool without description",
			json: `{
				"type": "tool_use",
				"id": "tool_7",
				"name": "AnotherTool",
				"input": {"foo": "bar"}
			}`,
			expected: "AnotherTool",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := formatToolUse(json.RawMessage(tt.json))
			if result != tt.expected {
				t.Errorf("expected '%s', got '%s'", tt.expected, result)
			}
		})
	}
}

func TestMarshalUnmarshal(t *testing.T) {
	// Create a complex assistant message
	contentJSON := []map[string]interface{}{
		{
			"type": "text",
			"text": "Here's the answer",
		},
		{
			"type": "tool_use",
			"id":   "tool_1",
			"name": "Read",
			"input": map[string]string{
				"file_path": "/test.txt",
			},
		},
	}

	contentBytes, _ := json.Marshal(contentJSON)

	original := AssistantMessage{
		Type: "assistant",
		BaseMessage: BaseMessage{
			UUID:        "test-uuid",
			Timestamp:   "2024-01-01T00:00:00Z",
			SessionID:   "test-session",
			IsSidechain: false,
			UserType:    "external",
			CWD:         "/test",
			Version:     "1.0",
		},
		Message: AssistantMessageContent{
			ID:      "msg_01",
			Type:    "message",
			Role:    "assistant",
			Model:   "claude-3-5-sonnet",
			Content: contentBytes,
			Usage: Usage{
				InputTokens:              100,
				CacheCreationInputTokens: 0,
				CacheReadInputTokens:     0,
				OutputTokens:             50,
			},
		},
	}

	// Marshal
	data, err := json.Marshal(original)
	if err != nil {
		t.Fatalf("failed to marshal message: %v", err)
	}

	// Unmarshal
	msg, err := UnmarshalSessionMessage(data)
	if err != nil {
		t.Fatalf("failed to unmarshal message: %v", err)
	}

	// Verify
	if GetMessageType(msg) != "assistant" {
		t.Errorf("expected type assistant, got %s", GetMessageType(msg))
	}
	if ExtractContentText(msg) != "Here's the answer\nRead: test.txt" {
		t.Errorf("unexpected content text: %s", ExtractContentText(msg))
	}
}

// Test backward compatibility methods
func TestBackwardCompatibility(t *testing.T) {
	jsonStr := `{
		"type": "user",
		"message": {
			"role": "user",
			"content": "Test message"
		},
		"uuid": "test-uuid",
		"timestamp": "2024-01-01T00:00:00Z",
		"sessionId": "test-session",
		"parentUuid": null,
		"isSidechain": false,
		"userType": "external",
		"cwd": "/test",
		"version": "1.0"
	}`

	msg, err := UnmarshalSessionMessage([]byte(jsonStr))
	if err != nil {
		t.Fatalf("failed to unmarshal message: %v", err)
	}

	// Test that backward compatibility methods work
	if userMsg, ok := msg.(UserMessage); ok {
		if userMsg.GetType() != "user" {
			t.Errorf("GetType() expected 'user', got %s", userMsg.GetType())
		}
		if userMsg.GetContentText() != "Test message" {
			t.Errorf("GetContentText() expected 'Test message', got %s", userMsg.GetContentText())
		}
		if *userMsg.GetUUID() != "test-uuid" {
			t.Errorf("GetUUID() expected 'test-uuid', got %s", *userMsg.GetUUID())
		}
		if *userMsg.GetSessionID() != "test-session" {
			t.Errorf("GetSessionID() expected 'test-session', got %s", *userMsg.GetSessionID())
		}
		if userMsg.GetSearchableText() != "Test message test-session test-uuid" {
			t.Errorf("GetSearchableText() unexpected result: %s", userMsg.GetSearchableText())
		}
	} else {
		t.Fatalf("expected UserMessage type")
	}
}
