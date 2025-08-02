# Settings Merge Guide

## Overview

The `hail-mary prd init` command now intelligently merges its hook configuration with any existing `.claude/settings.json` file in your project, preserving your custom settings while adding the necessary session tracking hooks.

## How It Works

### 1. Settings Detection
When you run `hail-mary prd init`, it:
- Checks for an existing `.claude/settings.json` in your project
- If found, loads and parses the existing settings
- If not found, starts with empty settings

### 2. Hook Merging
The system adds these hooks to track sessions:
- `SessionStart`: Captures session ID when Claude starts
- `UserPromptSubmit`: Updates session timestamps on each prompt
- `Stop`: Cleans up session state when Claude stops

These hooks are **appended** to any existing hooks you may have configured.

### 3. Settings Preservation
All other settings in your `.claude/settings.json` are preserved:
- Custom hooks remain intact
- Other Claude settings are maintained
- Unknown fields are preserved as-is

### 4. Temporary File Creation
The merged settings are written to a temporary file and passed to Claude using the `--settings` flag, leaving your original settings file untouched.

## Example

If you have an existing `.claude/settings.json`:
```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Write",
        "hooks": [
          {
            "type": "command",
            "command": "prettier --write"
          }
        ]
      }
    ]
  },
  "customSetting": "value"
}
```

After running `hail-mary prd init`, the merged settings will include:
```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Write",
        "hooks": [
          {
            "type": "command",
            "command": "prettier --write"
          }
        ]
      }
    ],
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
    ],
    "Stop": [
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
  },
  "customSetting": "value"
}
```

## Benefits

1. **Non-invasive**: Your original settings file is never modified
2. **Flexible**: Works with or without existing settings
3. **Compatible**: Preserves all Claude settings and custom hooks
4. **Clean**: Uses Claude's standard `--settings` flag instead of environment variables

## Troubleshooting

### Debug Mode
Run with debug logging to see the merged settings path:
```bash
hail-mary --log-level=debug prd init
```

### Verify Merged Settings
The debug log will show the temporary settings file path. You can inspect it:
```bash
cat /tmp/hail-mary-settings-*.json | jq .
```

### Common Issues

1. **Hooks not triggering**: Ensure the hail-mary binary is built and executable
2. **Settings not found**: Check that `.claude/settings.json` is valid JSON
3. **Permission errors**: Ensure you have write access to the temp directory