# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Development Commands

### Essential Commands
```bash
# Build the binary
make build

# Run tests
make test

# Run tests with coverage
make coverage

# Format code
make fmt

# Run linter (requires golangci-lint)
make lint

# Run all checks (fmt, lint, test, build)
make all

# Clean build artifacts
make clean

# Install golangci-lint if needed
go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest
```

### Development Workflow
```bash
# Cross-platform builds
make build-linux
make build-windows
make build-mac
make build-all
```

### Post-Build Requirements
After running `make build`, always execute the following commands to update zsh completions:
```bash
./bin/hail-mary completion zsh > ~/.local/share/zinit/completions/_hm
./scripts/fix-completion.sh ~/.local/share/zinit/completions/_hm
```

Note: The fix-completion.sh script corrects a known Cobra bug where array appending syntax in zsh completions causes duplicate display.

## Architecture Overview

This is a CLI application built with **Cobra** (command framework) and **slog** (structured logging). The architecture follows a clean separation of concerns:

### Claude Session Management

The `internal/claude/executor.go` provides enhanced session management capabilities:

- **SessionInfo**: Struct containing session ID, result, cost, duration, and turns
- **ExecuteInteractive**: Now automatically gets session ID and continues interactively (NEW!)
- **ExecuteAndContinueInteractive**: Same as ExecuteInteractive but also returns SessionInfo
- **ExecuteWithSessionTracking**: Execute prompts and retrieve session information (print mode)
- **ResumeSession**: Resume specific sessions by session ID (print mode)
- **ExecuteInteractiveWithSession**: Resume specific sessions in interactive mode
- **Input Validation**: Security validation for prompts and session IDs

Note: Session IDs are automatically returned in the JSON response when using print mode (`-p --output-format=json`).

#### Hook-based Session Tracking (NEW!)

The application now supports automatic session tracking through Claude Code's hook system:

1. **Hook Command**: `hail-mary hook` processes Claude Code hook events
2. **Session State**: Stored in `~/.hail-mary/sessions/{PID}.json`
3. **Automatic Tracking**: `prd init` uses hooks to capture session IDs automatically

To enable hook-based session tracking in your own Claude Code projects:

1. Copy `.claude/settings.json.example` to `.claude/settings.json`
2. Ensure `hail-mary` binary is built and accessible
3. The hook will automatically track sessions when using supported commands

Hook Configuration Example:
```json
{
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "$CLAUDE_PROJECT_DIR/bin/hail-mary hook"
          }
        ]
      }
    ]
  }
}
```

#### Usage Example
```go
executor := claude.NewExecutor()

// NEW: Interactive mode with automatic session tracking
err := executor.ExecuteInteractive("Create a function")
// This automatically:
// 1. Executes the prompt and gets session ID
// 2. Shows initial response
// 3. Continues in interactive mode

// Alternative: Get SessionInfo while starting interactive mode
sessionInfo, err := executor.ExecuteAndContinueInteractive("Create a function")

// Programmatic execution (non-interactive)
sessionInfo, err := executor.ExecuteWithSessionTracking("Create a function")
resumedInfo, err := executor.ResumeSession(sessionInfo.ID, "Add error handling")
```

### Command Structure
- **main.go**: Entry point that calls cmd.Execute()
- **cmd/root.go**: Root command setup with global flags (--log-level) and slog configuration

### Key Design Patterns

1. **Centralized Logging**: All commands use slog through GetLogger() with configurable log levels (debug, info, warn, error)

2. **Command Initialization**: Each subcommand is registered in its init() function, keeping registration decoupled

3. **Flag Management**: Each command manages its own flags as package-level variables, with PersistentFlags on root for global options

## Development Guidelines

### Code Best Practices
- プログラムにコメントを残すときはすべて英語にする