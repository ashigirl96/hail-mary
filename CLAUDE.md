# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Hail Mary** is a comprehensive PRD (Product Requirements Document) management system with deep Claude Code integration. It provides an intuitive TUI interface for managing requirements documentation using the EARS (Easy Approach to Requirements Syntax) format.

### Key Features
- **PRD Management**: Create and manage product requirements documents with EARS format
- **Claude Code Integration**: Automatic session tracking through hook system
- **Interactive TUI**: Rich terminal interface built with Bubbletea and Lipgloss
- **Session Resume**: Continue or redo conversations from any point in history
- **Template System**: Embedded templates with mermaid diagram support
- **Japanese/English Support**: Mixed language documentation with proper formatting
- **WASM Integration**: Rust WebAssembly modules for performance-critical operations

## Build and Development Commands

### Essential Commands
```bash
# Build the binary (includes WASM)
just build

# Build Go only (no WASM rebuild)
just build-go

# Run tests (Go and Rust)
just test

# Run tests with coverage
just coverage

# Format code (Go and Rust)
just fmt

# Run linters (Go and Rust)
just lint

# Run all checks (fmt, lint, test, build)
just all

# Clean build artifacts
just clean

# Install all required tools
just install-tools
```

### Development Workflow
```bash
# Live development with file watching
just dev     # Requires air
just watch   # Alternative file watchers

# Cross-platform builds
just build-linux
just build-windows
just build-mac
just build-all

# Parallel test and build
just parallel-checks
```

### WASM Development
```bash
# Build WASM module only
just wasm-build

# Run WASM example
just run-wasm-example

# Format Rust code
just wasm-fmt

# Lint Rust code
just wasm-lint
```

### Post-Build Requirements
After running `just build`, always execute the following commands to update zsh completions:
```bash
./bin/hail-mary completion zsh > ~/.local/share/zinit/completions/_hm
./scripts/fix-completion.sh ~/.local/share/zinit/completions/_hm
```

Note: The fix-completion.sh script corrects a known Cobra bug where array appending syntax in zsh completions causes duplicate display.

## Architecture Overview

### Technology Stack
- **Framework**: Cobra (CLI framework) with structured command hierarchy
- **Build System**: Just (command runner) with Rust/Go integration
- **WebAssembly Runtime**: Wazero for running WASM modules
- **Rust Integration**: WASM modules for performance-critical operations
- **Logging**: slog with configurable log levels
- **TUI**: Bubbletea (TUI framework) with Lipgloss (styling)
- **Testing**: testify for assertions and mocking
- **Validation**: go-playground/validator for input validation

### Project Structure
```
.
├── cmd/                    # Command definitions
│   ├── root.go            # Root command and global configuration
│   ├── prd/               # PRD management command
│   └── hook/              # Claude Code hook handler
├── internal/              # Internal packages
│   ├── claude/            # Claude CLI integration
│   │   ├── executor.go    # Claude CLI executor
│   │   ├── hook.go        # Hook configuration
│   │   ├── session_state_manager.go  # Session state management
│   │   └── schemas/       # Hook event schemas
│   ├── prd/               # PRD business logic
│   │   ├── service.go     # PRD service layer
│   │   ├── types.go       # Domain types
│   │   └── constants.go   # PRD constants
│   ├── kiro/              # Template and spec management
│   │   ├── spec.go        # Specification management
│   │   ├── templates.go   # Template rendering
│   │   └── templates/     # Embedded templates
│   ├── ui/                # Terminal UI components
│   │   ├── prd_resume.go  # PRD resume interface
│   │   └── feature_input.go  # Feature input interface
│   ├── wasm/              # WebAssembly integration
│   │   ├── loader.go      # WASM module loader
│   │   └── module.wasm    # Compiled WASM module
│   ├── settings/          # Settings management
│   └── testing/           # Test utilities and mocks
├── rust-wasm/             # Rust WASM module source
│   ├── Cargo.toml         # Rust project configuration
│   └── src/
│       └── lib.rs         # Rust WASM implementation
├── examples/              # Example programs
│   └── wasm-hello/        # WASM integration example
├── docs/                  # Documentation
├── reference/             # External references
├── scripts/               # Utility scripts
└── justfile              # Build automation with Just
```

## Command Reference

### PRD Command
The main command for managing Product Requirements Documents:

```bash
# Launch PRD management interface
hail-mary prd

# Features:
# - Create new requirements documents
# - Resume existing sessions
# - Redo conversations from any point
# - Automatic session tracking

# Navigation:
# - j/k: Move up/down
# - h/l: Switch panes
# - Enter: Select
# - q/Esc: Quit
```

### Hook Command (Hidden)
Processes Claude Code hook events for session management:

```bash
# Automatically called by Claude Code
hail-mary hook
```

## Claude Code Integration

### Hook-based Session Management

The application integrates with Claude Code's hook system for automatic session tracking:

1. **Hook Events Handled**:
   - `SessionStart`: Captures new session IDs
   - `UserPromptSubmit`: Updates session timestamps
   - `Stop`: Finalizes session state
   - `PreToolUse`/`PostToolUse`: Tool usage tracking

2. **Session Storage**:
   - Feature-specific: `.kiro/spec/{feature}/sessions/sessions.json`
   - Contains session IDs, timestamps, and transcript paths

3. **Hook Configuration**:
   - Automatic merging with existing `.claude/settings.json`
   - Temporary configuration for Claude CLI execution
   - Environment variables for process tracking

### Claude Executor

The `internal/claude/executor.go` provides Claude CLI integration:

- **Execute**: Launch Claude with initial prompt and options
- **ExecuteWithSession**: Resume specific sessions
- **Configuration**: Customizable CLI options and environment
- **Validation**: Input validation for security

### Session State Management

The `internal/claude/session_state_manager.go` handles session persistence:

- **Thread-safe**: Concurrent access protection
- **Feature-based**: Sessions organized by feature
- **JSON persistence**: Human-readable session files

## PRD System Architecture

### Service Layer (`internal/prd/service.go`)

- **Feature Management**: Create and list feature directories
- **Session Execution**: Handle new and resumed sessions
- **Hook Integration**: Automatic hook configuration
- **Transcript Management**: Handle redo operations

### Template System (`internal/kiro/`)

1. **Requirements Template**: System prompt for PRD specialist persona
2. **Initial Template**: Starting structure for new requirements
3. **Dynamic Rendering**: Template variables for paths and configuration

### EARS Format

The system uses EARS (Easy Approach to Requirements Syntax) format:

- **Event-Driven**: `<u>WHEN</u> [event] <u>THEN</u> [system] <u>SHALL</u> [response]`
- **State-Based**: `<u>IF</u> [condition] <u>THEN</u> [system] <u>SHALL</u> [response]`
- **Continuous**: `<u>WHILE</u> [condition] <u>THE SYSTEM</u> <u>SHALL</u> [behavior]`
- **Contextual**: `<u>WHERE</u> [context] <u>THE SYSTEM</u> <u>SHALL</u> [behavior]`

### Language Convention

- **Documentation**: Japanese content
- **EARS Keywords**: English with `<u>` tags for emphasis
- **Mermaid Diagrams**: Support for flowcharts and sequence diagrams

## TUI Components

### PRD Resume Interface (`internal/ui/prd_resume.go`)

- **Three-pane Layout**:
  - Left: Markdown preview of requirements
  - Right Top: Feature list
  - Right Bottom: Session inputs
- **Monokai Color Scheme**: Professional dark theme
- **Keyboard Navigation**: Vim-like bindings
- **Dynamic Content**: Real-time markdown rendering

### Feature Input Interface (`internal/ui/feature_input.go`)

- **Text Input**: Feature title entry
- **Validation**: Input sanitization
- **Confirmation**: Enter to confirm, Esc to cancel

## Testing Strategy

### Test Organization

- **Unit Tests**: Package-level `*_test.go` files
- **Mocks**: `internal/testing/mocks/` for interfaces
- **Integration**: Hook and session management tests
- **Coverage**: Target 80% coverage minimum

### Running Tests

```bash
# Run all tests (Go and Rust)
just test

# Run with coverage
just coverage

# Run specific package
go test ./internal/prd/...

# Run with race detection
go test -race ./...

# Run Rust tests only
cd rust-wasm && cargo test
```

## Development Guidelines

### Code Organization

1. **Domain-Driven Design**: Separate business logic from infrastructure
2. **Interface-Based**: Define interfaces for testability
3. **Error Handling**: Wrap errors with context
4. **Logging**: Use structured logging with slog

### Best Practices

- **Comments**: All code comments in English
- **Error Messages**: Descriptive with context
- **Validation**: Input validation at boundaries
- **Thread Safety**: Use sync primitives for shared state
- **Resource Cleanup**: Always defer cleanup functions

### Git Workflow

```bash
# Feature branch
git checkout -b feature/your-feature

# Make changes and test
just all

# Commit with conventional commits
git commit -m "feat(prd): add new feature"

# Push and create PR
git push origin feature/your-feature
```

### Adding New Commands

1. Create command file in `cmd/yourcommand/root.go`
2. Define command structure with cobra.Command
3. Implement Init() function for registration
4. Add to initSubcommands() in `cmd/root.go`
5. Add tests in `cmd/yourcommand/root_test.go`

### Extending PRD System

1. **New Templates**: Add to `internal/kiro/templates/`
2. **New Modes**: Update constants in `internal/prd/constants.go`
3. **New UI Components**: Create in `internal/ui/`
4. **New Hooks**: Update `internal/claude/hook.go`

## Installation and Setup

### Prerequisites
- Go 1.24.4 or later
- Rust and Cargo (for WASM development)
- Just command runner

### Installing Just
```bash
# macOS
brew install just

# Linux
curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to ~/bin

# Cargo
cargo install just
```

### Installing Required Tools
```bash
# Install all required tools (Go linters, Rust target, wasm-opt)
just install-tools

# Install Rust WASM target only
just install-rust-target

# Install dependencies
just deps

# Tidy dependencies
just tidy
```

## WASM Development

### Overview
The project includes Rust WebAssembly modules for performance-critical operations, running on the Wazero runtime (pure Go WebAssembly runtime).

### Rust WASM Module Structure
- **Location**: `rust-wasm/` directory
- **Target**: `wasm32-unknown-unknown`
- **Runtime**: Wazero (no CGO dependencies)
- **Integration**: Embedded in Go binary via `go:embed`

### WASM Development Commands
```bash
# Build WASM module
just wasm-build

# Run WASM example
just run-wasm-example

# Format Rust code
just wasm-fmt

# Lint Rust code
just wasm-lint

# Build everything (WASM + Go)
just build

# Build Go only (skip WASM)
just build-go
```

### Adding New WASM Functions
1. Add function to `rust-wasm/src/lib.rs` with `#[no_mangle]` and `extern "C"`
2. Update `internal/wasm/loader.go` to expose the new function
3. Run `just wasm-build` to compile
4. Test with example in `examples/` directory

## Environment Variables

- `HAIL_MARY_PARENT_PID`: Parent process ID for hook tracking
- `HAIL_MARY_FEATURE_PATH`: Current feature directory path
- `CLAUDE_*`: Claude CLI environment variables

## Security Considerations

1. **Input Validation**: All user inputs are validated
2. **Path Traversal**: Absolute paths only, no traversal
3. **Command Injection**: Proper argument escaping
4. **Session Security**: Session IDs validated for format

## Performance Considerations

1. **Lazy Loading**: Load sessions on demand
2. **Caching**: Template caching for performance
3. **Parallel Operations**: Tests and builds run in parallel
4. **Resource Management**: Proper cleanup and limits

## Troubleshooting

### Common Issues

1. **Hook Not Working**: Check executable path and permissions
2. **Session Not Found**: Verify feature path and session files
3. **TUI Rendering Issues**: Check terminal capabilities
4. **Build Failures**: Ensure Go 1.24.4+ and dependencies

### Debug Mode

```bash
# Enable debug logging
hail-mary --log-level debug prd

# Check hook execution
tail -f /tmp/hail-mary-*.log

# Verify session files
ls -la .kiro/spec/*/sessions/
```

## Additional Notes

### Code Style
- Follow Go conventions and idioms
- Use gofmt and goimports for formatting
- Run staticcheck and golangci-lint before commits
- プログラムにコメントを残すときはすべて英語にする

## Just Build System

### Why Just?
- **Simple Syntax**: No tabs vs spaces issues like Make
- **Cross-Platform**: Works consistently on Windows, macOS, and Linux
- **Better for Multi-Language**: Native support for both Rust and Go workflows
- **Parameter Support**: Easy command parameterization
- **Built-in Functions**: Rich set of built-in functions and string manipulation

### Key Just Commands
```bash
# Show all available commands
just --list
just help

# Run default recipe
just

# Run specific recipe
just build

# See what a recipe would do (dry run)
just --dry-run build

# Pass arguments to recipes
just build-linux

# Choose a specific justfile
just --justfile path/to/justfile build
```

### Justfile Organization
- **Variables**: Defined at the top for reusability
- **Default Recipe**: `default: all` runs when no recipe specified
- **Dependencies**: Recipes can depend on other recipes
- **Conditional Logic**: Support for if/else and OS detection
- **Shell Commands**: Direct shell command execution with error handling