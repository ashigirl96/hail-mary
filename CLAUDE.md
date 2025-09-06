# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is hail-mary, a sophisticated Rust CLI application that implements a Memory MCP (Model Context Protocol) server and Kiro project specification management system. The project demonstrates modern Rust architecture patterns with a focus on AI model context management, technical knowledge storage, and multilingual full-text search capabilities.

The project is structured as a Cargo workspace with the main application located in `crates/hail-mary/`.

## Core Architecture

The system follows a 4-layer Clean Architecture with clear separation of concerns and dependency inversion:

```
+-----------------------------------------------------------+
|                      CLI Layer                           |
|              (Commands, User Interaction)                |
+-----------------------------------------------------------+
|                   Application Layer                      |
|          (Use Cases, Business Logic, Ports)             |
+-----------------------------------------------------------+
|                     Domain Layer                         |
|        (Entities, Value Objects, Domain Rules)          |
+-----------------------------------------------------------+
|                 Infrastructure Layer                     |
|    (Database, File System, External Services)           |
+-----------------------------------------------------------+
```

### Domain Layer (`crates/hail-mary/src/domain/`)
- **Entities**: `Memory` and `ProjectConfig` - core business objects with identity
- **Value Objects**: `Confidence`, `SystemPrompt` - domain-specific types with validation (0.0-1.0)
- **Domain Rules**: Business invariants and validation logic embedded in entities
- **Domain Errors**: Business rule violations using thiserror

### Application Layer (`crates/hail-mary/src/application/`)
- **Use Cases**: Function-based business logic (`remember_memory`, `recall_memory`, `initialize_project`)
- **Repository Ports**: `MemoryRepository` and `ProjectRepository` traits defining data access interfaces
- **Business Orchestration**: Coordinates domain objects and enforces business rules
- **Application Errors**: Operation-level errors with proper conversion from domain errors

### CLI Layer (`crates/hail-mary/src/cli/`)
- **Commands**: Command implementations (`InitCommand`, `NewCommand`, `CodeCommand`, `MemoryCommand`)
- **Arguments**: Clap-based CLI argument parsing with validation
- **Formatters**: Output formatting for different display modes (text, JSON, markdown)

### Infrastructure Layer (`crates/hail-mary/src/infrastructure/`)
- **Repository Implementations**: `SqliteMemoryRepository` with FTS5 search and WAL mode
- **MCP Server**: Protocol implementation for AI model integration using rmcp
- **Filesystem**: `PathManager` for centralized path resolution and project discovery
- **Process Management**: `ClaudeProcessLauncher` for external process integration with TTY management
- **TUI Components**: Specification selector interface using ratatui/crossterm
- **Migrations**: Embedded database migrations using Refinery

### Database Design
- SQLite with FTS5 for multilingual full-text search (Japanese tokenization support)
- Refinery for versioned migrations (V001→V003)
- Automatic triggers maintain FTS index consistency
- Logical deletion with `deleted` flag

### Directory Structure

```
crates/
└── hail-mary/
    └── src/
        ├── domain/                          # Pure business logic
        │   ├── entities/
        │   │   ├── memory.rs               # Memory entity with UUID, type, content
        │   │   └── project.rs              # Project configuration
        │   ├── value_objects/
        │   │   ├── confidence.rs           # Confidence value (0.0-1.0)
        │   │   └── system_prompt.rs        # System prompt for Claude Code
        │   └── errors.rs                   # Domain-specific errors
        │
        ├── application/                     # Business logic orchestration
        │   ├── use_cases/
        │   │   ├── initialize_project.rs   # Project initialization logic
        │   │   ├── create_feature.rs       # Feature creation logic
        │   │   ├── complete_features.rs    # Archive completed specs logic
        │   │   ├── launch_claude_with_spec.rs # Claude Code integration logic
        │   │   ├── remember_memory.rs      # Store memory logic
        │   │   ├── recall_memory.rs        # Retrieve memories logic
        │   │   ├── generate_document.rs    # Document generation logic
        │   │   └── reindex_memories.rs     # Database optimization logic
        │   ├── repositories/               # Repository interfaces (traits)
        │   │   ├── memory_repository.rs    # Memory persistence interface
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
        │   │   └── memory.rs              # Memory subcommands 
        │   ├── formatters.rs              # Output formatting
        │   └── args.rs                    # Argument parsing structures
        │
        ├── infrastructure/                 # External services & implementations
        │   ├── repositories/
        │   │   ├── memory.rs              # Memory repository (SQLite)
        │   │   └── project.rs             # Project repository
        │   ├── mcp/
        │   │   └── server.rs              # MCP server implementation
        │   ├── filesystem/
        │   │   └── path_manager.rs        # Centralized path management
        │   ├── process/
        │   │   └── claude_launcher.rs     # Claude Code launcher with TTY management
        │   ├── tui/
        │   │   └── spec_selector.rs       # Specification selector TUI
        │   └── migrations/
        │       └── embedded.rs            # Refinery migrations
        │
        ├── lib.rs                          # Library exports
        └── main.rs                         # Application entry point & DI
```

### CLI Commands
- `hail-mary init`: Initialize .kiro directory structure and configuration
- `hail-mary new <name>`: Create feature specification templates with validation (includes requirements.md, design.md, tasks.md, memo.md, investigation.md, and spec.json)
- `hail-mary complete`: Interactive TUI for marking specifications as complete (archives to .kiro/archive)
- `hail-mary code`: Launch Claude Code with Kiro specification context
- `hail-mary memory serve`: Start MCP server for AI model integration
- `hail-mary memory document [--type <type>]`: Generate documentation from memories
- `hail-mary memory reindex [--dry-run] [--verbose]`: Database optimization and cleanup

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
just run memory serve  # Start MCP server
just dev                # Watch mode (check + test + run)
just test-watch         # Watch mode for tests only

# Advanced testing
cargo test --test integration_repository_test  # SQLite integration tests
cargo test test_migration_creates_tables      # Database migration tests
cargo test -- --nocapture                     # Test output visible
```

### Database Development
```bash
# Database is automatically initialized on first run
# Migrations are embedded and run automatically via Refinery
# Test with temporary databases in integration tests
```

### Key Testing Patterns
- Repository tests use both trait implementations (SQLite + InMemory)
- Integration tests create temporary databases with full migration cycle
- Service layer tests validate business logic and async operations
- Japanese text search specifically tested in FTS5 integration tests

## Memory Management Workflow

1. **Configuration**: `.kiro/config.toml` defines memory types and paths
2. **Storage**: Memories have type (tech/project-tech/domain), confidence scores, and tags
3. **Search**: FTS5 with business logic filtering (type, tags, confidence)
4. **Reference Tracking**: Async updates to `reference_count` and `last_accessed`
5. **Export**: Markdown generation with memory grouping by type

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

### Database Patterns
- Parameterized queries prevent SQL injection
- Transaction-based batch operations for consistency
- Logical deletion preserves data while hiding from queries
- FTS5 triggers automatically maintain search index

### Memory Types System
- **String-based Types**: Flexible string types defined in configuration (tech, project-tech, domain, workflow, decision)
- **Configuration Validation**: ProjectConfig validates memory types against allowed values
- **Type Safety**: Use case functions validate memory types before entity creation

### Testing Infrastructure
- `tests/common/` provides shared test utilities
- `TestDirectory` RAII pattern for safe current directory changes
- Temporary databases with full schema setup
- Japanese content specifically tested in multilingual scenarios
- **Use `TestDirectory` instead of `temp_dir.path()`** for thread-safe temporary directory management with automatic cleanup

## MCP Protocol Integration

The Memory MCP server provides AI models with structured access to technical knowledge:
- Real-time memory storage and retrieval
- Confidence-based quality scoring
- Multilingual search with Japanese tokenization
- Async protocol implementation using rmcp v0.5.0

When working with MCP features, remember that the server maintains persistent connections and memory updates are immediately available for search.

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