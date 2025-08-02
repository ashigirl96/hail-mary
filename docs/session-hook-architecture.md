# Session Hook Architecture Design

## Overview

This document describes the architecture for capturing and managing Claude session IDs through the hook system, replacing the current `--print(-p)` approach with a more robust hook-based mechanism.

## Architecture Components

### 1. Hook Event Data Structures (Go)

All hook event structures will be defined in `internal/hooks/events.go` using go-playground/validator for validation.

```go
package hooks

import (
    "time"
    "github.com/go-playground/validator/v10"
)

var validate = validator.New(validator.WithRequiredStructEnabled())

// BaseHookEvent contains common fields for all hook events
type BaseHookEvent struct {
    SessionID      string `json:"session_id" validate:"required,uuid"`
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
    Continue              bool                  `json:"continue,omitempty"`
    StopReason           string                `json:"stopReason,omitempty"`
    SuppressOutput       bool                  `json:"suppressOutput,omitempty"`
    Decision             string                `json:"decision,omitempty" validate:"omitempty,oneof=approve block allow deny ask"`
    Reason               string                `json:"reason,omitempty"`
    HookSpecificOutput   map[string]interface{} `json:"hookSpecificOutput,omitempty"`
}

// ParseHookEvent parses and validates a hook event from JSON
func ParseHookEvent(data []byte) (interface{}, error) {
    // Implementation to parse based on hook_event_name discriminator
}

// ValidateHookEvent validates any hook event struct
func ValidateHookEvent(event interface{}) error {
    return validate.Struct(event)
}
```

### 2. Session State Management

The session state will be managed through a file-based IPC mechanism with atomic operations.

```go
// internal/session/state.go
package session

import (
    "encoding/json"
    "os"
    "path/filepath"
    "sync"
    "time"
)

type SessionState struct {
    SessionID      string    `json:"session_id"`
    StartedAt      time.Time `json:"started_at"`
    LastUpdated    time.Time `json:"last_updated"`
    TranscriptPath string    `json:"transcript_path"`
    ProjectDir     string    `json:"project_dir"`
}

type SessionManager struct {
    stateDir string
    mu       sync.RWMutex
}

func NewSessionManager() *SessionManager {
    homeDir, _ := os.UserHomeDir()
    stateDir := filepath.Join(homeDir, ".hail-mary", "sessions")
    os.MkdirAll(stateDir, 0755)
    return &SessionManager{stateDir: stateDir}
}

// WriteSession atomically writes session state
func (sm *SessionManager) WriteSession(processID string, state *SessionState) error {
    sm.mu.Lock()
    defer sm.mu.Unlock()
    
    statePath := filepath.Join(sm.stateDir, processID+".json")
    tempPath := statePath + ".tmp"
    
    data, err := json.Marshal(state)
    if err != nil {
        return err
    }
    
    // Write to temp file
    if err := os.WriteFile(tempPath, data, 0644); err != nil {
        return err
    }
    
    // Atomic rename
    return os.Rename(tempPath, statePath)
}

// ReadSession reads session state for a process
func (sm *SessionManager) ReadSession(processID string) (*SessionState, error) {
    sm.mu.RLock()
    defer sm.mu.RUnlock()
    
    statePath := filepath.Join(sm.stateDir, processID+".json")
    data, err := os.ReadFile(statePath)
    if err != nil {
        return nil, err
    }
    
    var state SessionState
    if err := json.Unmarshal(data, &state); err != nil {
        return nil, err
    }
    
    return &state, nil
}

// CleanupSession removes session state
func (sm *SessionManager) CleanupSession(processID string) error {
    sm.mu.Lock()
    defer sm.mu.Unlock()
    
    statePath := filepath.Join(sm.stateDir, processID+".json")
    return os.Remove(statePath)
}
```

### 3. Hook Command Implementation

The `hail-mary hook` command will be the executable called by Claude's hook system.

```go
// cmd/hook.go
package cmd

import (
    "encoding/json"
    "fmt"
    "io"
    "os"
    "strconv"
    
    "github.com/spf13/cobra"
    "github.com/ashigirl96/hail-mary/internal/hooks"
    "github.com/ashigirl96/hail-mary/internal/session"
)

var hookCmd = &cobra.Command{
    Use:   "hook",
    Short: "Hook handler for Claude Code integration",
    Long:  `Processes hook events from Claude Code and manages session state`,
    RunE:  runHook,
}

func init() {
    rootCmd.AddCommand(hookCmd)
}

func runHook(cmd *cobra.Command, args []string) error {
    // Read JSON from stdin
    input, err := io.ReadAll(os.Stdin)
    if err != nil {
        return fmt.Errorf("failed to read stdin: %w", err)
    }
    
    // Parse base event to determine type
    var baseEvent hooks.BaseHookEvent
    if err := json.Unmarshal(input, &baseEvent); err != nil {
        return fmt.Errorf("failed to parse hook event: %w", err)
    }
    
    // Get parent process ID from environment variable
    parentPID := os.Getenv("HAIL_MARY_PARENT_PID")
    if parentPID == "" {
        // Not launched by hail-mary, just exit successfully
        return nil
    }
    
    // Handle based on event type
    switch baseEvent.HookEventName {
    case "SessionStart":
        var event hooks.SessionStartEvent
        if err := json.Unmarshal(input, &event); err != nil {
            return err
        }
        
        if err := hooks.ValidateHookEvent(event); err != nil {
            return fmt.Errorf("validation failed: %w", err)
        }
        
        // Write session state for parent process
        sm := session.NewSessionManager()
        state := &session.SessionState{
            SessionID:      event.SessionID,
            StartedAt:      time.Now(),
            LastUpdated:    time.Now(),
            TranscriptPath: event.TranscriptPath,
            ProjectDir:     event.CWD,
        }
        
        if err := sm.WriteSession(parentPID, state); err != nil {
            return fmt.Errorf("failed to write session state: %w", err)
        }
        
        // Log for debugging
        logger := GetLogger()
        logger.Debug("SessionStart hook processed",
            "session_id", event.SessionID,
            "parent_pid", parentPID,
            "source", event.Source)
    
    case "UserPromptSubmit":
        // Could track prompts or add context here
        var event hooks.UserPromptSubmitEvent
        if err := json.Unmarshal(input, &event); err != nil {
            return err
        }
        
        // Add context about current session
        if parentPID != "" {
            sm := session.NewSessionManager()
            if state, err := sm.ReadSession(parentPID); err == nil {
                output := hooks.HookOutput{
                    HookSpecificOutput: map[string]interface{}{
                        "hookEventName": "UserPromptSubmit",
                        "additionalContext": fmt.Sprintf("Current session: %s (started %s)",
                            state.SessionID, state.StartedAt.Format(time.RFC3339)),
                    },
                }
                outputJSON, _ := json.Marshal(output)
                fmt.Println(string(outputJSON))
            }
        }
        
    // Handle other event types as needed...
    }
    
    return nil
}
```

### 4. Modified PRD Init Command

Update the `prd init` command to use the hook system:

```go
// internal/prd/init.go modifications

func (p *PRD) Init(ctx context.Context) error {
    // ... existing setup code ...
    
    // Set up hook configuration
    hookConfig := map[string]interface{}{
        "hooks": map[string]interface{}{
            "SessionStart": []map[string]interface{}{
                {
                    "hooks": []map[string]interface{}{
                        {
                            "type": "command",
                            "command": fmt.Sprintf("HAIL_MARY_PARENT_PID=%d %s hook",
                                os.Getpid(), os.Args[0]),
                        },
                    },
                },
            },
        },
    }
    
    // Write temporary hook configuration
    tempHookPath := filepath.Join(os.TempDir(), fmt.Sprintf("hail-mary-hooks-%d.json", os.Getpid()))
    hookData, _ := json.MarshalIndent(hookConfig, "", "  ")
    os.WriteFile(tempHookPath, hookData, 0644)
    defer os.Remove(tempHookPath)
    
    // Start monitoring for session updates
    sm := session.NewSessionManager()
    sessionChan := make(chan *session.SessionState)
    go p.monitorSession(sm, fmt.Sprintf("%d", os.Getpid()), sessionChan)
    
    // Launch Claude with hook configuration
    executor := claude.NewExecutor()
    executor.SetHookConfig(tempHookPath)
    
    // Start interactive session
    go func() {
        err := executor.ExecuteInteractive(p.generatePRDPrompt())
        if err != nil {
            logger.Error("Claude execution failed", "error", err)
        }
    }()
    
    // Wait for session to be established
    select {
    case state := <-sessionChan:
        p.sessionID = state.SessionID
        logger.Info("Session established", "session_id", state.SessionID)
    case <-time.After(30 * time.Second):
        return fmt.Errorf("timeout waiting for session establishment")
    case <-ctx.Done():
        return ctx.Err()
    }
    
    // Continue with interactive session...
    return nil
}

func (p *PRD) monitorSession(sm *session.SessionManager, processID string, sessionChan chan<- *session.SessionState) {
    ticker := time.NewTicker(100 * time.Millisecond)
    defer ticker.Stop()
    
    for {
        select {
        case <-ticker.C:
            if state, err := sm.ReadSession(processID); err == nil {
                sessionChan <- state
                return
            }
        }
    }
}
```

## Data Flow

1. **Session Initialization**:
   - `hail-mary prd init` starts and sets `HAIL_MARY_PARENT_PID` environment variable
   - Creates temporary hook configuration pointing to `hail-mary hook`
   - Launches Claude with the hook configuration

2. **Hook Execution**:
   - Claude starts a session and triggers `SessionStart` hook
   - `hail-mary hook` receives JSON via stdin with session details
   - Hook validates the event and writes session state to file system
   - Parent process monitors for session file creation

3. **Session Tracking**:
   - Parent process reads session ID from state file
   - Continues with interactive session, now knowing the session ID
   - Can track session changes through subsequent hook events

## Benefits

1. **Real-time Updates**: Session ID is captured immediately when session starts
2. **Robust IPC**: File-based communication with atomic operations prevents race conditions
3. **Extensible**: Can easily add more hook handlers for other events
4. **Clean Architecture**: Separates hook handling from main application logic
5. **Validation**: All hook events are validated using go-playground/validator

## Security Considerations

1. State files are stored in user's home directory with restricted permissions
2. Process ID validation ensures only legitimate parent processes can read state
3. Temporary files use atomic rename operations to prevent partial reads
4. Hook configuration is cleaned up after use

## Future Enhancements

1. Support for multiple concurrent sessions
2. Session history and analytics
3. Hook event replay for debugging
4. Integration with other hook events for comprehensive tracking