# Hook Implementation Guide

## Overview

This guide explains how to implement and use the hook-based session tracking system in hail-mary.

## Key Components

### 1. Hook Event Structures (`internal/hooks/events.go`)
- Defines all Claude Code hook event types using go-playground/validator
- Provides validation for incoming hook events
- Supports all standard Claude Code hooks (SessionStart, UserPromptSubmit, etc.)

### 2. Session State Management (`internal/session/state.go`)
- Manages persistent session state on disk
- Uses atomic file operations for safety
- Supports multiple concurrent sessions
- Automatic cleanup of stale sessions

### 3. Hook Command (`cmd/hook.go`)
- Executable that Claude Code calls via hooks
- Processes hook events and updates session state
- Communicates with parent process via file-based IPC

### 4. Hook Integration (`internal/prd/hook_integration.go`)
- Helper functions for setting up hook configuration
- Monitors for session establishment
- Manages hook lifecycle and cleanup

## Implementation Steps

### Step 1: Build the Project

```bash
make build
```

### Step 2: Test the Hook Command

Use the provided test script to verify hook functionality:

```bash
./examples/test_hook.sh
```

### Step 3: Integrate with PRD Init

Update your PRD init command to use the new hook-based approach:

```go
func (p *PRD) Init(ctx context.Context) error {
    // Use the new hook-based initialization
    return p.InitWithHooks(ctx)
}
```

## How It Works

### Session Establishment Flow

1. **Parent Process Setup**:
   - `hail-mary prd init` starts and records its PID
   - Creates temporary hook configuration file
   - Sets `HAIL_MARY_PARENT_PID` environment variable

2. **Claude Launch**:
   - Claude Code starts with hook configuration
   - Triggers `SessionStart` hook when session begins

3. **Hook Execution**:
   - `hail-mary hook` receives JSON via stdin
   - Validates the event structure
   - Writes session state to `~/.hail-mary/sessions/{PID}.json`

4. **Parent Monitoring**:
   - Parent process polls for session file
   - Reads session ID once file appears
   - Continues with interactive session

### Hook Configuration Format

The system generates a hook configuration like this:

```json
{
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "HAIL_MARY_PARENT_PID=12345 /path/to/hail-mary hook",
            "timeout": 5
          }
        ]
      }
    ],
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "HAIL_MARY_PARENT_PID=12345 /path/to/hail-mary hook",
            "timeout": 2
          }
        ]
      }
    ]
  }
}
```

### Session State Format

Session state is stored in JSON format:

```json
{
  "session_id": "abc123-def456-...",
  "started_at": "2024-01-01T10:00:00Z",
  "last_updated": "2024-01-01T10:30:00Z",
  "transcript_path": "/Users/.../.claude/projects/.../transcript.jsonl",
  "project_dir": "/Users/.../project"
}
```

## Advanced Usage

### Custom Hook Handlers

You can extend the hook command to handle additional events:

```go
// In cmd/hook.go
case "PreToolUse":
    // Add validation or logging for tool usage
    return handlePreToolUse(input, logger)
```

### Session Monitoring

Monitor session updates in real-time:

```go
err := prd.MonitorSession(ctx, func(state *session.State) {
    fmt.Printf("Session updated: %s at %s\n", 
        state.SessionID, state.LastUpdated)
})
```

### Cleanup Old Sessions

Periodically clean up stale session files:

```go
sm, _ := session.NewManager()
err := sm.CleanupStale(24 * time.Hour)
```

## Troubleshooting

### Session Not Detected

1. Check that hook command is executable
2. Verify `HAIL_MARY_PARENT_PID` is set correctly
3. Look for session files in `~/.hail-mary/sessions/`
4. Enable debug logging: `hail-mary --log-level=debug prd init`

### Hook Failures

1. Test hook manually: `echo '{"session_id":"test",...}' | hail-mary hook`
2. Check Claude Code logs for hook execution errors
3. Verify JSON structure matches expected schema

### Permission Issues

1. Ensure `~/.hail-mary/sessions/` directory is writable
2. Check that hook configuration file is readable by Claude Code
3. Verify executable permissions on hail-mary binary

## Security Considerations

1. **Process Isolation**: Only the parent process can read its session state
2. **Atomic Operations**: File writes use atomic rename to prevent corruption
3. **Cleanup**: Session files are automatically removed on Stop event
4. **Validation**: All hook inputs are validated before processing

## Future Enhancements

1. **Encryption**: Encrypt session state files for additional security
2. **IPC Alternatives**: Support Unix sockets or named pipes for faster IPC
3. **Session History**: Track session history and analytics
4. **Multi-Session**: Support multiple concurrent Claude sessions
5. **Hook Replay**: Ability to replay hook events for debugging