# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is hail-mary, a sophisticated Rust CLI application that implements a Memory MCP (Model Context Protocol) server and Kiro project specification management system. The project demonstrates modern Rust architecture patterns with a focus on AI model context management, technical knowledge storage, and multilingual full-text search capabilities.

## Core Architecture

The system follows hexagonal architecture with clear separation of concerns:

### Domain Models (`src/models/`)
- `Memory`: Core entity with UUID, MemoryType enum (Tech/ProjectTech/Domain), confidence scoring, and metadata
- `KiroConfig`: TOML-based configuration management with project root discovery
- `MemoryError`: Structured error handling using thiserror

### Repository Pattern (`src/repositories/memory.rs`)
- `MemoryRepository` trait: Abstracted data access interface
- `SqliteMemoryRepository`: Production implementation with FTS5 search, WAL mode, and transaction support
- `InMemoryRepository`: Testing implementation using HashMap

### Service Layer (`src/services/`)
- `MemoryService`: Business logic with validation, async operations, and batch processing
- `MemoryMcpService`: MCP protocol implementation for AI model integration

### Database Design
- SQLite with FTS5 for multilingual full-text search (Japanese tokenization support)
- Refinery for versioned migrations (V001→V003)
- Automatic triggers maintain FTS index consistency
- Logical deletion with `deleted` flag

### CLI Commands (`src/commands/`)
- `init`: Initialize .kiro directory and configuration
- `new`: Create feature specification templates
- `memory serve`: Start MCP server
- `memory document`: Generate documentation from memories
- `memory reindex`: Database optimization and cleanup

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
- `anyhow::Result` for application-level errors
- `MemoryError` enum with `thiserror` for domain errors
- Repository methods return `Result<T>` with proper error propagation

### Async Patterns
- Tokio runtime with async/await throughout service layer
- `Arc<Mutex<Repository>>` for safe concurrent access
- Non-blocking reference count updates via `tokio::spawn`

### Database Patterns
- Parameterized queries prevent SQL injection
- Transaction-based batch operations for consistency
- Logical deletion preserves data while hiding from queries
- FTS5 triggers automatically maintain search index

### Memory Types System
- Extensible enum with string serialization
- Configuration validation against allowed types
- Display/FromStr traits for CLI argument parsing

### Testing Infrastructure
- `tests/common/` provides shared test utilities
- `TestDirectory` RAII pattern for safe current directory changes
- Temporary databases with full schema setup
- Japanese content specifically tested in multilingual scenarios

## MCP Protocol Integration

The Memory MCP server provides AI models with structured access to technical knowledge:
- Real-time memory storage and retrieval
- Confidence-based quality scoring
- Multilingual search with Japanese tokenization
- Async protocol implementation using rmcp v0.5.0

When working with MCP features, remember that the server maintains persistent connections and memory updates are immediately available for search.