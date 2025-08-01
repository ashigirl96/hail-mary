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
# Run specific commands quickly
make list      # Build and run list command
make ui        # Build and run ui command

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

This is a CLI application built with **Cobra** (command framework), **slog** (structured logging), and **Bubbletea** (TUI framework). The architecture follows a clean separation of concerns:

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
- **cmd/list.go**: Standard CLI subcommand example with flags (--all, --format)
- **cmd/ui.go**: TUI subcommand that launches Bubbletea interface with --text flag

### Key Design Patterns

1. **Centralized Logging**: All commands use slog through GetLogger() with configurable log levels (debug, info, warn, error)

2. **Command Initialization**: Each subcommand is registered in its init() function, keeping registration decoupled

3. **TUI Separation**: The Bubbletea TUI logic is isolated in internal/ui/model.go, following the Elm architecture pattern with Init(), Update(), and View() methods

4. **Flag Management**: Each command manages its own flags as package-level variables, with PersistentFlags on root for global options

### TUI Model Architecture
The internal/ui/model.go implements a text input field with:
- Cursor movement and positioning
- Character insertion/deletion at cursor position
- Confirmation state tracking
- Keyboard shortcuts (Enter to confirm, Esc to cancel)

The model passes the logger through to enable debugging of TUI events without interfering with the terminal UI rendering.

## Development Guidelines

### Code Best Practices
- プログラムにコメントを残すときはすべて英語にする