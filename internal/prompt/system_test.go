package prompt

import (
	"log/slog"
	"strings"
	"testing"
)

func TestReadPRDSystemPrompt(t *testing.T) {
	// Create a no-op logger for testing
	logger := slog.New(slog.NewTextHandler(nil, nil))

	// Read the PRD system prompt
	prompt, err := ReadPRDSystemPrompt(logger)
	if err != nil {
		t.Fatalf("Failed to read PRD system prompt: %v", err)
	}

	// Check that prompt is not empty
	if prompt == "" {
		t.Error("PRD system prompt is empty")
	}

	// Check that it contains expected content
	expectedStrings := []string{
		"Product Requirements Document",
		"PRD) Assistant",
		"Core Responsibilities",
		"PRD Framework",
		"Success Metrics",
	}

	for _, expected := range expectedStrings {
		if !strings.Contains(prompt, expected) {
			t.Errorf("PRD system prompt missing expected content: %q", expected)
		}
	}

	// Verify it's the correct length (should be similar to the original file)
	if len(prompt) < 3000 {
		t.Errorf("PRD system prompt seems too short: %d characters", len(prompt))
	}
}
