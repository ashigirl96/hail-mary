#!/bin/bash

# Test script for hail-mary hook command
# This simulates how Claude Code would call the hook

# Set up test environment
export HAIL_MARY_PARENT_PID=$$
SESSION_ID="test-session-$(date +%s)"

# Test SessionStart hook
echo "Testing SessionStart hook..."
cat <<EOF | ./bin/hail-mary hook
{
  "session_id": "$SESSION_ID",
  "transcript_path": "$HOME/.claude/projects/test/transcript.jsonl",
  "cwd": "$(pwd)",
  "hook_event_name": "SessionStart",
  "source": "startup"
}
EOF

echo "Exit code: $?"
echo

# Check if session file was created
SESSION_FILE="$HOME/.hail-mary/sessions/$$.json"
if [ -f "$SESSION_FILE" ]; then
    echo "Session file created successfully:"
    cat "$SESSION_FILE" | jq .
else
    echo "ERROR: Session file not created at $SESSION_FILE"
fi
echo

# Test UserPromptSubmit hook
echo "Testing UserPromptSubmit hook..."
cat <<EOF | ./bin/hail-mary hook
{
  "session_id": "$SESSION_ID",
  "transcript_path": "$HOME/.claude/projects/test/transcript.jsonl",
  "cwd": "$(pwd)",
  "hook_event_name": "UserPromptSubmit",
  "prompt": "Create a test function"
}
EOF

echo "Exit code: $?"
echo

# Test Stop hook (should cleanup session)
echo "Testing Stop hook..."
cat <<EOF | ./bin/hail-mary hook
{
  "session_id": "$SESSION_ID",
  "transcript_path": "$HOME/.claude/projects/test/transcript.jsonl",
  "cwd": "$(pwd)",
  "hook_event_name": "Stop",
  "stop_hook_active": false
}
EOF

echo "Exit code: $?"
echo

# Check if session file was cleaned up
if [ -f "$SESSION_FILE" ]; then
    echo "ERROR: Session file still exists after Stop hook"
else
    echo "Session file cleaned up successfully"
fi