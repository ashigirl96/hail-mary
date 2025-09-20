# Technology Stack and Development Environment

## Architecture

### Language and Edition
- **Rust 1.89.0**: Primary implementation language
- **Edition 2024**: Main hail-mary crate
- **Edition 2021**: anthropic-client crate

### Architectural Patterns
- **Clean Architecture**: Layered separation of concerns
- **Repository Pattern**: Abstracted data access
- **Use Case Pattern**: Business logic encapsulation
- **Command Pattern**: CLI command routing
- **Value Objects**: Domain-specific immutable types

### Error Handling Strategy
- **anyhow::Result**: Application-level error handling
- **thiserror**: Domain-specific error types
- **Error Context**: Rich error messages with context chain

## Frontend

*Not applicable - CLI application without web frontend*

## Backend

### Core Framework
- **Rust CLI Application**: Native binary with no server component
- **Clap v4.5**: Command-line argument parsing with derive macros
- **Tokio v1**: Async runtime (anthropic-client only)

### Key Libraries
- **serde v1**: Serialization/deserialization with derive
- **toml v0.9**: Configuration file parsing
- **chrono v0.4**: Date/time handling
- **regex v1**: Pattern matching

### TUI Components
- **ratatui v0.29**: Terminal UI framework
- **crossterm v0.29**: Cross-platform terminal manipulation

### HTTP Client (anthropic-client)
- **reqwest v0.12**: HTTP client with rustls-tls-native-roots
- **OAuth Support**: Token management and refresh
- **Authentication**: ~/.local/share/opencode/auth.json

## Development Environment

### Required Tools
- **Rust Toolchain**: rustc, cargo, rustfmt, clippy
- **Just**: Task runner for development workflow
- **Git**: Version control system

### Project Structure
- **Cargo Workspace**: Multi-crate organization
- **Workspace Members**: hail-mary, anthropic-client
- **Build Target**: Native binary for host platform

### Development Dependencies
- **tempfile v3**: Test isolation with temporary directories
- **pretty_assertions v1**: Readable test output (unused but available)
- **rstest v0.23**: Parameterized testing (unused but available)

## Common Commands

### Build and Run
```bash
just build              # Standard build
just run init          # Run with arguments
just dev               # Watch mode (check + test + run)
```

### Testing
```bash
just test              # Run all tests
just test-verbose      # Tests with output
just test-watch        # Watch mode for tests
just ci                # Full CI pipeline (fmt + lint + test)
```

### Code Quality
```bash
just fmt               # Format code
just lint              # Comprehensive clippy checks
just lint-basic        # Basic clippy checks
just fix               # Format before testing
```

### Application Usage
```bash
# Project management
hail-mary init                           # Initialize .kiro directory
hail-mary new <feature-name>             # Create feature specification
hail-mary complete                       # Interactive spec completion
hail-mary code [--no-danger]             # Launch Claude Code

# Steering operations
hail-mary steering backup                # Backup steering files
hail-mary steering remind <type> <topic> <content>  # Remind steering content

# Utility commands
hail-mary shell-completions <shell>      # Generate shell completions
```

### Git Workflow
```bash
# Working with git through chezmoi (if applicable)
chezmoi git -- status
chezmoi git -- add .
chezmoi git -- commit -m "message"
chezmoi git -- push
```

## Environment Variables

### Runtime Configuration
- **RUST_LOG**: Logging level (debug, info, warn, error)
- **RUST_BACKTRACE**: Error backtrace display (0, 1, full)
- **CARGO_MANIFEST_DIR**: Project root for integration tests

### Build Environment
- **CARGO_HOME**: Cargo cache directory
- **RUSTUP_HOME**: Rust toolchain directory

## Port Configuration

*Not applicable - No network services or port bindings*

## Testing Strategy

### Test Organization
- **Unit Tests**: Embedded in source files with `#[cfg(test)]`
- **Integration Tests**: `tests/` directory
- **Test Helpers**: `application/test_helpers/` with mocks

### Test Patterns
- **TestDirectory**: Thread-safe test isolation with mutex
- **Mock Repositories**: Test doubles for repository traits
- **RAII Cleanup**: Automatic resource cleanup on drop

### Testing Commands
```bash
# Recommended workflow
just fix                # Format first
just ci                 # Complete validation

# Direct testing (when needed)
cargo test                                    # All tests
cargo test -- --nocapture                    # With output
RUST_BACKTRACE=1 cargo test -- --nocapture  # With traces
```

## Dependency Management

### Core Dependencies
```toml
# CLI and configuration
clap = { version = "4.5", features = ["derive"] }
clap_complete = "4.5"
toml = "0.9"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# TUI
ratatui = "0.29"
crossterm = "0.29"

# Utilities
anyhow = "1"
thiserror = "2"
chrono = "0.4"
regex = "1"

# Testing
tempfile = "3"  # dev dependency
```

### Anthropic Client Dependencies
```toml
# HTTP and async
reqwest = { version = "0.12", features = ["rustls-tls-native-roots", "json", "cookies", "gzip", "brotli", "deflate", "zstd", "charset", "http2", "stream"] }
tokio = { version = "1", features = ["full", "test-util"] }

# Utilities
dirs = "5.0"
anyhow = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

## Development Best Practices

### Code Style
- **Formatting**: Always run `just fmt` before commits
- **Linting**: Run `just lint` for comprehensive checks
- **Naming**: snake_case for files/functions, PascalCase for types
- **Comments**: Avoid unless necessary for complex logic

### Workflow Guidelines
1. **Before Starting**: Run `just ci` to ensure clean state
2. **During Development**: Use `just dev` for watch mode
3. **Before Commit**: Run `just fix` then `just ci`
4. **Testing**: Always test with `just test` not direct cargo

### Error Handling
- Use `?` operator for error propagation
- Add context with `.context()` for clarity
- Create domain errors with thiserror
- Never panic in production code

### Performance Considerations
- Minimize string allocations
- Use references where possible
- Batch file operations
- Efficient path resolution with caching