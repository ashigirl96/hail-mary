# PRD Hook Usage Guide

This guide explains how to use the new hook-based session tracking with the `hail-mary prd init` command.

## Overview

The PRD init command now uses Claude Code's hook system to automatically capture and track session IDs, eliminating the need for the `--print(-p)` workaround.

## How It Works

### Automatic Hook Configuration

When you run `hail-mary prd init`, the command:

1. Creates a temporary hook configuration file
2. Sets the `CLAUDE_SETTINGS_PATH` environment variable to use this configuration
3. Launches Claude with hooks enabled
4. Monitors for session establishment

### Session Tracking

The hook system captures:
- **Session ID**: Unique identifier for the Claude session
- **Transcript Path**: Location of the conversation transcript
- **Timestamps**: When the session started and was last updated

Session information is stored in `~/.hail-mary/sessions/{PID}.json`.

## Usage

### Basic Usage

```bash
# Build the project first
make build

# Initialize a new PRD with automatic session tracking
./bin/hail-mary prd init
```

You'll see output like:
```
Launching Claude interactive shell for PRD creation...
Press Ctrl+C to exit the Claude shell.

Tip: Session ID: abc12345
Use 'hail-mary prd continue' to resume this conversation later.
```

### Debug Mode

To see detailed hook execution:

```bash
./bin/hail-mary --log-level=debug prd init
```

This will show:
- Hook configuration creation
- Session monitoring progress
- Hook execution details

## Alternative: Manual Hook Configuration

If you prefer to configure hooks manually:

1. Create `.claude/settings.json` in your project:
```json
{
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "HAIL_MARY_PARENT_PID=$$ $CLAUDE_PROJECT_DIR/bin/hail-mary hook"
          }
        ]
      }
    ]
  }
}
```

2. Run Claude normally - the hooks will track sessions automatically

## Session State Files

Session information is stored as JSON:

```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "started_at": "2024-01-01T10:00:00Z",
  "last_updated": "2024-01-01T10:30:00Z",
  "transcript_path": "/Users/you/.claude/projects/xyz/transcript.jsonl",
  "project_dir": "/Users/you/projects/hail-mary"
}
```

## Troubleshooting

### Session Not Detected

If the session ID isn't captured:

1. Check that the binary is built: `make build`
2. Run in debug mode to see hook execution
3. Verify no existing Claude settings conflict
4. Check `~/.hail-mary/sessions/` for session files

### Hook Permission Errors

Ensure the hail-mary binary is executable:
```bash
chmod +x ./bin/hail-mary
```

### Timeout Issues

The system waits 30 seconds for session establishment. If Claude takes longer to start, the session might not be captured but Claude will continue normally.

## Benefits

1. **Automatic**: No manual session ID extraction needed
2. **Real-time**: Session ID captured immediately on start
3. **Reliable**: File-based IPC with atomic operations
4. **Extensible**: Easy to add more hook handlers
5. **Clean**: No output parsing required