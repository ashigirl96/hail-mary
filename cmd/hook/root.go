package hook

import (
	"encoding/json"
	"fmt"
	"io"
	"log/slog"
	"os"
	"time"

	"github.com/spf13/cobra"

	"github.com/ashigirl96/hail-mary/internal/hooks"
	"github.com/ashigirl96/hail-mary/internal/session"
)

var HookCmd = &cobra.Command{
	Use:   "hook",
	Short: "Hook handler for Claude Code integration",
	Long: `Processes hook events from Claude Code and manages session state.

This command is designed to be called by Claude Code's hook system and
processes various hook events like SessionStart, UserPromptSubmit, etc.

It reads JSON input from stdin and can write session state for parent
processes to track Claude sessions.`,
	Hidden: true, // Hide from help since it's meant to be called by Claude
	RunE:   runHook,
}

func Init(rootCmd *cobra.Command) {
	rootCmd.AddCommand(HookCmd)
}

// GetLogger is a temporary function to access the logger
// This will be removed when we update root.go imports
var GetLogger func() *slog.Logger

func runHook(cmd *cobra.Command, args []string) error {
	logger := GetLogger()

	// Read JSON from stdin
	input, err := io.ReadAll(os.Stdin)
	if err != nil {
		logger.Error("Failed to read stdin", "error", err)
		return fmt.Errorf("failed to read stdin: %w", err)
	}

	logger.Debug("Hook received input", "size", len(input))

	// Parse base event to determine type
	var baseEvent hooks.BaseHookEvent
	if err := json.Unmarshal(input, &baseEvent); err != nil {
		logger.Error("Failed to parse hook event", "error", err)
		return fmt.Errorf("failed to parse hook event: %w", err)
	}

	logger.Debug("Processing hook event",
		"event_type", baseEvent.HookEventName,
		"session_id", baseEvent.SessionID)

	// Get parent process ID from environment variable
	parentPID := os.Getenv("HAIL_MARY_PARENT_PID")
	if parentPID == "" {
		logger.Debug("No parent PID found, running in standalone mode")
	}

	// Handle based on event type
	switch baseEvent.HookEventName {
	case "SessionStart":
		return handleSessionStart(input, parentPID, logger)

	case "UserPromptSubmit":
		return handleUserPromptSubmit(input, parentPID, logger)

	case "PreToolUse":
		return handlePreToolUse(input, logger)

	case "PostToolUse":
		return handlePostToolUse(input, logger)

	case "Stop":
		return handleStop(input, parentPID, logger)

	default:
		logger.Debug("Unhandled hook event type", "type", baseEvent.HookEventName)
		// Exit successfully for unhandled events
		return nil
	}
}

func handleSessionStart(input []byte, parentPID string, logger *slog.Logger) error {
	var event hooks.SessionStartEvent
	if err := json.Unmarshal(input, &event); err != nil {
		return fmt.Errorf("failed to parse SessionStart event: %w", err)
	}

	if err := hooks.ValidateHookEvent(event); err != nil {
		return fmt.Errorf("validation failed: %w", err)
	}

	// If we have a parent PID, write session state
	if parentPID != "" {
		sm, err := session.NewManager()
		if err != nil {
			logger.Error("Failed to create session manager", "error", err)
			return err
		}

		state := &session.State{
			SessionID:      event.SessionID,
			StartedAt:      time.Now(),
			LastUpdated:    time.Now(),
			TranscriptPath: event.TranscriptPath,
			ProjectDir:     event.CWD,
		}

		if err := sm.WriteSession(parentPID, state); err != nil {
			logger.Error("Failed to write session state", "error", err)
			return fmt.Errorf("failed to write session state: %w", err)
		}

		logger.Info("Session state written",
			"session_id", event.SessionID,
			"parent_pid", parentPID,
			"source", event.Source)
	}

	// Optionally add context to the session
	if event.Source == "startup" {
		output := hooks.HookOutput{
			HookSpecificOutput: map[string]interface{}{
				"hookEventName":     "SessionStart",
				"additionalContext": fmt.Sprintf("Hail Mary session tracking enabled (PID: %s)", parentPID),
			},
		}
		outputJSON, _ := json.Marshal(output)
		fmt.Println(string(outputJSON))
	}

	return nil
}

func handleUserPromptSubmit(input []byte, parentPID string, logger *slog.Logger) error {
	var event hooks.UserPromptSubmitEvent
	if err := json.Unmarshal(input, &event); err != nil {
		return fmt.Errorf("failed to parse UserPromptSubmit event: %w", err)
	}

	if err := hooks.ValidateHookEvent(event); err != nil {
		return fmt.Errorf("validation failed: %w", err)
	}

	// Update session timestamp if we're tracking
	if parentPID != "" {
		sm, err := session.NewManager()
		if err == nil {
			if state, err := sm.ReadSession(parentPID); err == nil {
				_ = sm.UpdateSession(parentPID)

				// Add session context
				output := hooks.HookOutput{
					HookSpecificOutput: map[string]interface{}{
						"hookEventName": "UserPromptSubmit",
						"additionalContext": fmt.Sprintf("[Session: %s, Started: %s]",
							state.SessionID[:8], state.StartedAt.Format("15:04:05")),
					},
				}
				outputJSON, _ := json.Marshal(output)
				fmt.Println(string(outputJSON))
			}
		}
	}

	return nil
}

func handlePreToolUse(input []byte, logger *slog.Logger) error {
	var event hooks.PreToolUseEvent
	if err := json.Unmarshal(input, &event); err != nil {
		return fmt.Errorf("failed to parse PreToolUse event: %w", err)
	}

	if err := hooks.ValidateHookEvent(event); err != nil {
		return fmt.Errorf("validation failed: %w", err)
	}

	logger.Debug("PreToolUse event",
		"tool", event.ToolName,
		"session_id", event.SessionID)

	// Could add tool-specific validation or logging here
	// For now, just allow all tools
	return nil
}

func handlePostToolUse(input []byte, logger *slog.Logger) error {
	var event hooks.PostToolUseEvent
	if err := json.Unmarshal(input, &event); err != nil {
		return fmt.Errorf("failed to parse PostToolUse event: %w", err)
	}

	if err := hooks.ValidateHookEvent(event); err != nil {
		return fmt.Errorf("validation failed: %w", err)
	}

	logger.Debug("PostToolUse event",
		"tool", event.ToolName,
		"session_id", event.SessionID)

	// Could track tool usage statistics here
	return nil
}

func handleStop(input []byte, parentPID string, logger *slog.Logger) error {
	var event hooks.StopEvent
	if err := json.Unmarshal(input, &event); err != nil {
		return fmt.Errorf("failed to parse Stop event: %w", err)
	}

	if err := hooks.ValidateHookEvent(event); err != nil {
		return fmt.Errorf("validation failed: %w", err)
	}

	logger.Debug("Stop event",
		"session_id", event.SessionID,
		"stop_hook_active", event.StopHookActive)

	// Update session timestamp on stop (but don't delete the session file)
	if parentPID != "" && !event.StopHookActive {
		sm, err := session.NewManager()
		if err == nil {
			if err := sm.UpdateSession(parentPID); err != nil {
				logger.Debug("Failed to update session timestamp", "error", err)
			} else {
				logger.Debug("Session timestamp updated", "parent_pid", parentPID)
			}
		}
	}

	return nil
}