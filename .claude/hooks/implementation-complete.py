#!/usr/bin/env python3
"""
Hook script to detect when implementation work is completed.
Triggers on Stop event and analyzes the conversation to determine if implementation occurred.
"""

import json
import sys
import os
import re
from pathlib import Path


def read_transcript(transcript_path: str) -> list:
    """Read and parse the conversation transcript."""
    try:
        with open(transcript_path, 'r') as f:
            lines = f.readlines()
        
        transcript = []
        for line in lines:
            if line.strip():
                transcript.append(json.loads(line))
        return transcript
    except Exception as e:
        print(f"Error reading transcript: {e}", file=sys.stderr)
        return []


def analyze_implementation_activity(transcript: list) -> dict:
    """Analyze the transcript to detect implementation activity."""
    implementation_indicators = {
        'files_created': 0,
        'files_modified': 0,
        'code_written': False,
        'build_commands': False,
        'test_commands': False,
        'implementation_keywords': False,
        'tool_usage': []
    }
    
    # Keywords that suggest implementation work
    impl_keywords = [
        'implement', 'create', 'build', 'develop', 'code', 'write',
        'add feature', 'fix bug', 'refactor', 'optimize'
    ]
    
    build_keywords = ['make build', 'go build', 'npm run build', 'cargo build']
    test_keywords = ['make test', 'go test', 'npm test', 'cargo test']
    
    for entry in transcript:
        content = entry.get('content', [])
        
        for item in content:
            if item.get('type') == 'text':
                text = item.get('text', '').lower()
                
                # Check for implementation keywords
                if any(keyword in text for keyword in impl_keywords):
                    implementation_indicators['implementation_keywords'] = True
                
                # Check for build/test commands
                if any(keyword in text for keyword in build_keywords):
                    implementation_indicators['build_commands'] = True
                
                if any(keyword in text for keyword in test_keywords):
                    implementation_indicators['test_commands'] = True
            
            elif item.get('type') == 'tool_use':
                tool_name = item.get('name', '')
                implementation_indicators['tool_usage'].append(tool_name)
                
                # Track file operations
                if tool_name in ['Write', 'Edit', 'MultiEdit']:
                    if tool_name == 'Write':
                        implementation_indicators['files_created'] += 1
                    else:
                        implementation_indicators['files_modified'] += 1
                    implementation_indicators['code_written'] = True
    
    return implementation_indicators


def generate_completion_summary(indicators: dict) -> str:
    """Generate a summary of the implementation work completed."""
    summary_parts = []
    
    if indicators['files_created'] > 0:
        summary_parts.append(f"Created {indicators['files_created']} file(s)")
    
    if indicators['files_modified'] > 0:
        summary_parts.append(f"Modified {indicators['files_modified']} file(s)")
    
    if indicators['build_commands']:
        summary_parts.append("Executed build commands")
    
    if indicators['test_commands']:
        summary_parts.append("Ran tests")
    
    unique_tools = list(set(indicators['tool_usage']))
    if unique_tools:
        summary_parts.append(f"Used tools: {', '.join(unique_tools)}")
    
    return "; ".join(summary_parts) if summary_parts else "No specific implementation activity detected"


def main():
    try:
        input_data = json.load(sys.stdin)
    except json.JSONDecodeError as e:
        print(f"Error: Invalid JSON input: {e}", file=sys.stderr)
        sys.exit(1)
    
    # Only process Stop events, not SubagentStop
    if input_data.get("hook_event_name") != "Stop":
        sys.exit(0)
    
    # Avoid infinite loops - don't trigger if stop hook is already active
    if input_data.get("stop_hook_active", False):
        sys.exit(0)
    
    transcript_path = input_data.get("transcript_path", "")
    if not transcript_path or not os.path.exists(transcript_path):
        print("No transcript available", file=sys.stderr)
        sys.exit(1)
    
    # Read and analyze the conversation
    transcript = read_transcript(transcript_path)
    if not transcript:
        sys.exit(1)
    
    indicators = analyze_implementation_activity(transcript)
    
    # Determine if significant implementation work occurred
    implementation_score = 0
    
    if indicators['code_written']:
        implementation_score += 3
    if indicators['implementation_keywords']:
        implementation_score += 2
    if indicators['build_commands']:
        implementation_score += 2
    if indicators['test_commands']:
        implementation_score += 1
    if indicators['files_created'] > 0:
        implementation_score += indicators['files_created']
    if indicators['files_modified'] > 0:
        implementation_score += indicators['files_modified']
    
    # If significant implementation work was detected (score >= 3)
    if implementation_score >= 3:
        summary = generate_completion_summary(indicators)
        
        # Log completion to a file
        project_dir = os.environ.get('CLAUDE_PROJECT_DIR', os.getcwd())
        log_file = os.path.join(project_dir, '.claude', 'implementation-log.txt')
        
        try:
            with open(log_file, 'a') as f:
                from datetime import datetime
                timestamp = datetime.now().isoformat()
                f.write(f"{timestamp}: Implementation completed - {summary}\n")
        except Exception as e:
            print(f"Warning: Could not write to log file: {e}", file=sys.stderr)
        
        # Output for user notification
        print(f"🎉 Implementation completed! {summary}")
        
        # Run 'make all' automatically after implementation
        print("🔨 Running 'make all' to build and validate implementation...")
        
        import subprocess
        try:
            # Change to project directory and run make all
            result = subprocess.run(
                ['make', 'all'],
                cwd=project_dir,
                capture_output=True,
                text=True,
                timeout=300  # 5 minute timeout
            )
            
            if result.returncode == 0:
                print("✅ 'make all' completed successfully!")
                if result.stdout.strip():
                    print(f"Output: {result.stdout.strip()}")
            else:
                print(f"❌ 'make all' failed with exit code {result.returncode}")
                if result.stderr.strip():
                    print(f"Error: {result.stderr.strip()}")
                if result.stdout.strip():
                    print(f"Output: {result.stdout.strip()}")
                    
        except subprocess.TimeoutExpired:
            print("⏰ 'make all' timed out after 5 minutes")
        except FileNotFoundError:
            print("❌ 'make' command not found - please ensure make is installed")
        except Exception as e:
            print(f"❌ Error running 'make all': {e}")
    
    sys.exit(0)


if __name__ == "__main__":
    main()