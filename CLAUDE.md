# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is hail-mary, a sophisticated Rust CLI application focused on Kiro project specification management and file-based context steering. The project demonstrates modern Rust architecture patterns with emphasis on specification-driven development workflows, Claude Code integration, and version-controllable project knowledge management.

The project is structured as a Cargo workspace with the main application located in `crates/hail-mary/`.

## Core Architecture

The system follows clean layered architecture with clear separation of concerns and dependency inversion:

```
+-----------------------------------------------------------+
|                      CLI Layer                           |
|              (Commands, User Interaction)                |
+-----------------------------------------------------------+
|                   Application Layer                      |
|            (Use Cases, Business Logic, Ports)           |
+-----------------------------------------------------------+
|                     Domain Layer                         |
|      (Project Entities, Value Objects, Domain Rules)    |
+-----------------------------------------------------------+
|                 Infrastructure Layer                     |
|      (File System, Process Management, TUI Components)   |
+-----------------------------------------------------------+
```

### Domain Layer (`crates/hail-mary/src/domain/`)
- **Entities**: `ProjectConfig` and `SteeringConfig` - core business objects with identity
- **Value Objects**: `SystemPrompt` - domain-specific types for Claude Code integration
- **Steering System**: `SteeringType`, `Criterion` - file-based context management entities
- **Domain Rules**: Business invariants and validation logic embedded in entities
- **Domain Errors**: Business rule violations using thiserror

### Application Layer (`crates/hail-mary/src/application/`)
- **Use Cases**: Function-based business logic (`initialize_project`, `create_feature`, `complete_features`, `launch_claude_with_spec`)
- **Repository Ports**: `ProjectRepository` trait defining file system operation interfaces
- **Business Orchestration**: Coordinates domain objects and enforces business rules
- **Application Errors**: Operation-level errors with proper conversion from domain errors

### CLI Layer (`crates/hail-mary/src/cli/`)
- **Commands**: Command implementations (`InitCommand`, `NewCommand`, `CodeCommand`, `CompleteCommand`, `CompletionCommand`)
- **Arguments**: Clap-based CLI argument parsing with validation
- **Formatters**: Output formatting for different display modes

### Infrastructure Layer (`crates/hail-mary/src/infrastructure/`)
- **Repository Implementations**: `FilesystemProjectRepository` for file system operations
- **Filesystem**: `PathManager` for centralized path resolution and project discovery
- **Process Management**: `ClaudeProcessLauncher` for external process integration with TTY management
- **TUI Components**: Specification selector interface using ratatui/crossterm

### File System Design
- TOML-based configuration management (`.kiro/config.toml`)
- Markdown-based steering files for version-controllable context
- Structured specification templates with comprehensive documentation phases
- Automatic backup creation before file modifications

### Directory Structure

```
crates/
└── hail-mary/
    └── src/
        ├── domain/                          # Pure business logic
        │   ├── entities/
        │   │   ├── project.rs              # Project configuration
        │   │   └── steering.rs             # Steering system entities (SteeringType, Criterion, SteeringConfig)
        │   ├── value_objects/
        │   │   └── system_prompt.rs        # System prompt for Claude Code
        │   └── errors.rs                   # Domain-specific errors
        │
        ├── application/                     # Business logic orchestration
        │   ├── use_cases/
        │   │   ├── initialize_project.rs   # Project initialization logic
        │   │   ├── create_feature.rs       # Feature creation logic
        │   │   ├── complete_features.rs    # Archive completed specs logic
        │   │   └── launch_claude_with_spec.rs # Claude Code integration logic
        │   ├── repositories/               # Repository interfaces (traits)
        │   │   └── project_repository.rs   # Project structure interface (includes spec archiving)
        │   ├── test_helpers/               # Mock repositories for testing
        │   └── errors.rs                   # Application errors
        │
        ├── cli/                            # Command-line interface
        │   ├── commands/
        │   │   ├── init.rs                # Init command implementation
        │   │   ├── new.rs                 # New feature command
        │   │   ├── complete.rs            # Complete command with TUI (ratatui)
        │   │   ├── code.rs                # Code command for Claude Code integration
        │   │   └── completion.rs          # Shell completion generation
        │   ├── formatters.rs              # Output formatting
        │   └── args.rs                    # Argument parsing structures
        │
        ├── infrastructure/                 # External services & implementations
        │   ├── repositories/
        │   │   └── project.rs             # Project repository
        │   ├── filesystem/
        │   │   └── path_manager.rs        # Centralized path management
        │   ├── process/
        │   │   └── claude_launcher.rs     # Claude Code launcher with TTY management
        │   └── tui/
        │       └── spec_selector.rs       # Specification selector TUI
        │
        ├── lib.rs                          # Library exports
        └── main.rs                         # Application entry point & DI
```

### CLI Commands
- `hail-mary init`: Initialize .kiro directory structure and configuration (creates steering system)
- `hail-mary new <name>`: Create feature specification templates with validation (includes requirements.md, design.md, tasks.md, memo.md, investigation.md, and spec.json)
- `hail-mary complete`: Interactive TUI for marking specifications as complete (archives to .kiro/archive)
- `hail-mary code [--no-danger]`: Launch Claude Code with Kiro specification context
- `hail-mary shell-completions <shell>`: Generate shell completion scripts for bash, zsh, fish, PowerShell, or elvish

## Development Commands

### Just Task Runner
Use `just` for all development tasks:

```bash
# Core development workflow
just build              # Standard build
just test               # Run all tests (unit + integration)
just fmt                # Format code
just lint               # Clippy with -D warnings
just ci                 # Full CI pipeline (format check + lint + test)

# Development utilities
just run init           # Initialize project
just dev                # Watch mode (check + test + run)
just test-watch         # Watch mode for tests only

# Advanced testing
cargo test -- --nocapture                     # Test output visible
```

### Key Testing Patterns
- Repository tests use filesystem operations with proper isolation
- Integration tests create temporary directories with full cleanup
- Use case tests validate business logic and file system operations
- TUI interaction testing with proper terminal management

## Specification Management Workflow

1. **Initialize**: Set up `.kiro` directory and steering system with `hail-mary init`
2. **Create**: Generate new feature specifications with `hail-mary new <feature-name>`
3. **Develop**: Work on features in `.kiro/specs/<date-feature-name>/` directories
4. **Integrate**: Use `hail-mary code` to launch Claude Code with specification context
5. **Complete**: Use interactive TUI with `hail-mary complete` to mark specs as done
6. **Archive**: Completed specs are moved to `.kiro/archive/` for reference

## Steering System (File-Based Context Management)

The steering system provides transparent, version-controllable context management for project knowledge:

### Key Features
- **File-Based Storage**: Context stored as markdown files in `.kiro/steering/`
- **Version Control Friendly**: All steering files can be tracked in git
- **Claude Code Integration**: Slash commands for managing steering content
- **Smart Configuration**: Automatically adds [steering] section to existing config.toml

### Directory Structure
```
.kiro/
├── steering/
│   ├── product.md              # Product overview and value proposition
│   ├── tech.md                 # Technical stack and development environment
│   ├── structure.md            # Code organization patterns
│   └── backup/                 # Backups before modifications
└── config.toml                 # Contains [steering] configuration
```

### Steering Types
1. **Product**: Product overview, core features, use cases, value proposition
2. **Tech**: Architecture, frontend/backend stack, development environment, commands
3. **Structure**: Directory organization, code patterns, naming conventions, principles

### Slash Commands
- `/hm:steering-remember [title]`: Save new learnings directly to steering files
- `/hm:steering [--verbose] [--dry-run]`: Update and maintain steering files

### Workflow
1. **Initialize**: `hail-mary init` creates steering directories and default files
2. **Capture**: Use `/hm:steering-remember` during Claude Code sessions to save important context directly to steering files
3. **Maintain**: Use `/hm:steering` to update and refresh steering files
4. **Reference**: Steering files provide persistent context for future development sessions

## Key Implementation Details

### Error Handling Strategy
- **Domain Errors**: `DomainError` enum with `thiserror` for business rule violations (e.g., invalid confidence values)
- **Application Errors**: `ApplicationError` enum for operation-level errors with automatic conversion from domain errors
- **Repository Results**: All repository methods return `Result<T, ApplicationError>` for consistent error handling
- **Error Propagation**: Use `?` operator for seamless error conversion through the layers

### Clean Architecture Patterns
- **Dependency Inversion**: CLI and Infrastructure depend on Application/Domain abstractions
- **Function-based Use Cases**: Simple functions instead of complex service classes
- **Repository Pattern**: Traits in Application layer, implementations in Infrastructure
- **Dependency Injection**: Main function composes dependencies and injects into CLI commands

### File System Patterns
- Safe file operations with proper validation and error handling
- Atomic file operations to prevent corruption
- Backup creation before modifications
- Path validation and sanitization for security

### Steering Types System
- **String-based Types**: Flexible string types defined in configuration (product, tech, structure)
- **Configuration Validation**: ProjectConfig validates steering types against allowed values
- **Type Safety**: Use case functions validate steering types before file operations

### Steering System Architecture
- **Criterion Parsing**: Parses "Name: Description" format from config.toml
- **File Protection**: Never overwrites existing steering files or config.toml (even with --force)
- **Smart Configuration**: Automatically adds [steering] section to existing config.toml when missing
- **Template Generation**: Creates structured markdown files with criteria for easy categorization

### Testing Infrastructure
- `tests/common/` provides shared test utilities
- `TestDirectory` RAII pattern for safe current directory changes
- Temporary directories with full cleanup
- **Use `TestDirectory` instead of `temp_dir.path()`** for thread-safe temporary directory management with automatic cleanup

## Claude Code Integration

The `hail-mary code` command provides seamless integration with Claude Code by launching it with Kiro specification context:

### Key Features
- **Interactive Specification Selection**: TUI for choosing existing specs or creating new ones
- **Structured System Prompts**: XML-tagged file paths for easy reference in Claude Code
- **TTY Management**: Proper terminal handling for backgrounding and process control
- **Clean Architecture Integration**: Follows the project's 4-layer architecture pattern

### System Prompt Structure
When launching Claude Code, the following XML tags provide structured access to specification files:
```xml
<kiro_spec_name>spec-name</kiro_spec_name>
<kiro_spec_path>.kiro/specs/spec-name/</kiro_spec_path>
<kiro_requirements_path>.kiro/specs/spec-name/requirements.md</kiro_requirements_path>
<kiro_design_path>.kiro/specs/spec-name/design.md</kiro_design_path>
<kiro_tasks_path>.kiro/specs/spec-name/tasks.md</kiro_tasks_path>
<kiro_memo_path>.kiro/specs/spec-name/memo.md</kiro_memo_path>
<kiro_investigation_path>.kiro/specs/spec-name/investigation.md</kiro_investigation_path>
```

### Usage
```bash
# Launch Claude Code with Kiro context
hail-mary code
# Select from existing specifications or create new ones
# Claude Code launches with full specification context
```

### File Descriptions
- **requirements.md**: Comprehensive requirements including user stories and acceptance criteria
- **design.md**: Technical design with architecture decisions and implementation approach  
- **tasks.md**: Implementation tasks with priorities and dependencies
- **memo.md**: Additional notes and context from the user
- **investigation.md**: Research findings, key discoveries, and technical considerations from investigation phase

## Anthropic Client Integration

The workspace now includes `crates/anthropic-client` which provides OAuth authentication and API client functionality for Anthropic's Claude API:

### Key Features
- **OAuth Token Management**: Automatic token refresh when expired
- **Authentication**: Loads tokens from `~/.local/share/opencode/auth.json`
- **API Client**: Non-streaming calls to Claude models
- **Cloudflare Protection**: Configured with `rustls-tls-native-roots` to avoid bot detection

### Usage Example
```bash
# Run the example chat client
cargo run --example basic_chat -- "Your question here"
```

### Important Implementation Details
- Uses `reqwest` with specific features to mimic browser behavior
- Handles OAuth2 refresh token flow automatically
- Provides public API functions: `load_auth()`, `refresh_token()`, `complete()`
- Requires proper reqwest configuration to bypass Cloudflare bot detection