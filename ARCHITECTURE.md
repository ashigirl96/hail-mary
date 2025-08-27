# Hail-Mary Project Architecture

## Overview

Hail-Mary is a sophisticated Rust CLI application that implements a Memory MCP (Model Context Protocol) server and Kiro project specification management system. The project demonstrates modern Rust architecture patterns with a focus on performance, reliability, and maintainability.

**Primary Purpose**: CLI tool for Memory MCP server and Rust project specification management
**Key Features**: Memory database with FTS search, MCP protocol implementation, configuration management
**Target Use Cases**: AI model context management, technical knowledge storage, project documentation

## 🏗️ System Architecture

### Architectural Patterns
- **Hexagonal Architecture**: Clear separation between domain logic and infrastructure
- **Repository Pattern**: Abstracted data access with multiple implementations
- **Service Layer**: Business logic encapsulation with validation
- **Command Pattern**: CLI commands with structured execution
- **Builder Pattern**: Fluent configuration and object construction

### Core Components

```mermaid
%%{init: {
  'theme': 'dark',
  'themeCSS': '
    .nodeLabel { color: #FD971F !important; }
    .edgeLabel { color: #A6E22E !important; background-color: transparent !important; }
    .cluster rect { fill: #272822 !important; stroke: #F92672 !important; stroke-width: 2px !important; rx: 5px !important; ry: 5px !important; }
    .node rect, .node circle, .node ellipse, .node polygon, .node path { fill: #272822 !important; stroke: #A6E22E !important; stroke-width: 2px !important; }
    .flowchart-link { stroke: #66D9EF !important; stroke-width: 2px !important; }
  '
}}%%

flowchart TB
    subgraph CLI ["🖥️ CLI Interface Layer"]
        Main["main.rs<br/>Command routing & argument parsing"]
    end
    
    subgraph CMD ["⚡ Command Layer"]
        Init["InitCommand<br/>Project setup"]
        New["NewCommand<br/>Spec creation"]
        Memory["MemoryCommands<br/>MCP operations"]
    end
    
    subgraph SVC ["🔧 Service Layer"]
        MemSvc["MemoryService<br/>Business logic & validation"]
        McpSvc["MemoryMcpService<br/>MCP protocol implementation"]
    end
    
    subgraph REPO ["📦 Repository Layer"]
        RepoTrait["MemoryRepository<br/>(trait interface)"]
        SqliteRepo["SqliteMemoryRepository<br/>(production)"]
        InMemRepo["InMemoryRepository<br/>(testing)"]
    end
    
    subgraph DATA ["🗄️ Data Layer"]
        DB["SQLite Database<br/>with FTS5 search"]
        Migration["Migration System<br/>(Refinery)"]
        Config["TOML Configuration<br/>Files"]
    end
    
    Main --> Init
    Main --> New
    Main --> Memory
    
    Init --> MemSvc
    New --> MemSvc
    Memory --> MemSvc
    Memory --> McpSvc
    
    MemSvc --> RepoTrait
    McpSvc --> RepoTrait
    
    RepoTrait -.-> SqliteRepo
    RepoTrait -.-> InMemRepo
    
    SqliteRepo --> DB
    SqliteRepo --> Migration
    MemSvc --> Config
    
    classDef cli fill:#272822,stroke:#66D9EF,stroke-width:2px;
    classDef command fill:#272822,stroke:#A6E22E,stroke-width:2px;
    classDef service fill:#272822,stroke:#F92672,stroke-width:2px;
    classDef repo fill:#272822,stroke:#AE81FF,stroke-width:2px;
    classDef data fill:#272822,stroke:#FD971F,stroke-width:2px;
    classDef interface fill:#272822,stroke:#66D9EF,stroke-width:2px,stroke-dasharray: 5 5;
    
    class Main cli;
    class Init,New,Memory command;
    class MemSvc,McpSvc service;
    class RepoTrait interface;
    class SqliteRepo,InMemRepo repo;
    class DB,Migration,Config data;
```

## 📁 Directory Structure

### Workspace Organization
The project uses a Cargo workspace structure for better modularity and future extensibility.

```
.
├── Cargo.toml                    # Workspace root configuration
├── crates/
│   └── hail-mary/                    # Main application crate
│       ├── Cargo.toml                # Application package configuration  
│       └── src/                      # Source code
│           ├── main.rs               # CLI entry point and command routing
│           ├── lib.rs                # Library exports for integration tests
│           ├── domain/               # Pure business logic
│           │   ├── entities/        # Core domain objects
│           │   │   ├── memory.rs    # Memory entity with business rules
│           │   │   └── project.rs   # Project configuration entity
│           │   ├── value_objects/   # Domain-specific types
│           │   │   └── confidence.rs # Confidence value (0.0-1.0)
│           │   └── errors.rs        # Domain errors
│           ├── application/          # Business logic orchestration
│           │   ├── use_cases/       # Application services
│           │   │   ├── initialize_project.rs
│           │   │   ├── create_feature.rs
│           │   │   ├── remember_memory.rs
│           │   │   └── recall_memory.rs
│           │   ├── repositories/    # Repository trait definitions
│           │   │   ├── memory_repository.rs
│           │   │   └── project_repository.rs
│           │   └── errors.rs        # Application errors
│           ├── cli/                 # Command-line interface
│           │   ├── commands/        # Command implementations
│           │   │   ├── init.rs     # Project initialization
│           │   │   ├── new.rs      # Feature creation
│           │   │   ├── complete.rs # Complete features with TUI
│           │   │   └── memory.rs   # Memory subcommands
│           │   ├── formatters.rs   # Output formatting
│           │   └── args.rs         # CLI argument parsing
│           └── infrastructure/      # External services
│               ├── repositories/    # Repository implementations
│               │   ├── memory.rs   # SQLite memory repository
│               │   └── project.rs  # Filesystem project repository
│               ├── mcp/            # MCP protocol
│               │   └── server.rs   # MCP server implementation
│               ├── filesystem/      # File system operations
│               │   └── path_manager.rs
│               └── migrations/      # Database migrations
│                   └── embedded.rs  # Embedded migration system
```

### External Organization
```
migrations/               # Database schema management
├── V001__initial_schema.sql    # Core tables and indexes
├── V002__add_fts5_index.sql   # Full-text search setup
└── V003__add_triggers.sql     # Automatic FTS maintenance

tests/                   # Integration tests
├── common/             # Shared test utilities
├── integration/        # Cross-module integration tests
├── new_command.rs      # Command testing
└── integration_repository_test.rs  # Database integration tests
```

## 🔧 Core Components Detail

### CLI Interface (`main.rs`)
**Purpose**: Command routing and argument parsing with clap
**Key Features**:
- Structured command hierarchy with subcommands
- Type-safe argument parsing using derive macros
- Async runtime support with Tokio
- Comprehensive error handling

**Command Structure**:
```rust
Commands::Init(InitCommand)        // Project initialization
Commands::New(NewCommand)          // Specification creation
Commands::Memory {
    MemoryCommands::Serve          // Start MCP server
    MemoryCommands::Document       // Generate documentation
    MemoryCommands::Reindex        // Database optimization
}
```

### Memory Domain Model (`models/memory.rs`)
**Core Entity**: `Memory` struct with rich metadata
**Memory Types**: Tech, ProjectTech, Domain (extensible enum)
**Key Features**:
- UUID-based identification
- Builder pattern for fluent construction
- SQLite integration with custom row mapping
- Confidence scoring and reference tracking
- Logical deletion support

**Data Structure**:
```rust
pub struct Memory {
    pub id: String,                    // UUID v4
    pub memory_type: MemoryType,       // Categorization
    pub title: String,                 // Human-readable title
    pub tags: Vec<String>,             // Searchable metadata
    pub content: String,               // Main content
    pub reference_count: u32,          // Usage tracking
    pub confidence: f32,               // Quality score (0.0-1.0)
    pub created_at: i64,              // Creation timestamp
    pub last_accessed: Option<i64>,   // Access tracking
    pub deleted: bool,                // Logical deletion
}
```

### Configuration System (`models/kiro.rs`)
**Purpose**: Project configuration management with TOML
**Key Features**:
- Hierarchical configuration with defaults
- Project root discovery (.kiro directory)
- Memory type validation
- Document output configuration
- Database path management

**Configuration Hierarchy**:
```toml
[memory]
types = ["tech", "project-tech", "domain"]
instructions = "Memory type descriptions"

[memory.document]
output_dir = ".kiro/memory"
format = "markdown"

[memory.database]
path = ".kiro/memory/db.sqlite3"
```

### Repository Layer (`repositories/memory.rs`)
**Pattern**: Repository pattern with trait-based abstraction
**Implementations**:
- `SqliteMemoryRepository`: Production persistence with FTS5
- `InMemoryRepository`: Testing implementation with HashMap

**Core Operations**:
```rust
pub trait MemoryRepository: Send {
    fn save(&mut self, memory: &Memory) -> Result<()>;
    fn save_batch(&mut self, memories: &[Memory]) -> Result<()>;
    fn find_by_id(&self, id: &str) -> Result<Option<Memory>>;
    fn search_fts(&self, query: &str, limit: usize) -> Result<Vec<Memory>>;
    fn find_all(&self) -> Result<Vec<Memory>>;
    fn increment_reference_count(&mut self, id: &str) -> Result<()>;
}
```

### Service Layer (`services/memory.rs`)
**Purpose**: Business logic and validation with async support
**Key Features**:
- Input validation against configuration
- Batch operations with transaction support
- Search with filtering and sorting
- Asynchronous reference count updates
- Markdown document generation

**Business Logic Flow**:
1. **Input Validation**: Memory type, required fields, confidence range
2. **Memory Creation**: UUID generation, defaults, builder pattern
3. **Persistence**: Repository delegation with error handling
4. **Search Logic**: FTS queries with business rule filtering
5. **Reference Tracking**: Async updates without blocking

## 🗄️ Database Architecture

### Schema Design
**Primary Table**: `memories` with comprehensive metadata
**Search Index**: `memories_fts` using SQLite FTS5
**Performance Indexes**: Type, reference count, creation time
**Data Integrity**: Constraints, foreign keys, check constraints

### Migration Strategy
**Tool**: Refinery for versioned migrations
**Approach**: Forward-only migrations with embedded SQL
**Versioning**: V001, V002, V003 prefix convention

**Migration Sequence**:
1. **V001**: Core schema with tables and basic indexes
2. **V002**: FTS5 virtual table with Japanese tokenization support
3. **V003**: Automatic triggers for FTS index maintenance

### FTS5 Search Implementation
**Tokenizer**: `porter unicode61` for multilingual support
**Features**: Japanese text search, phrase queries, ranking
**Maintenance**: Automatic via triggers (INSERT, UPDATE, DELETE)
**Performance**: Indexed search with relevance ranking

**Trigger System**:
```sql
-- Automatic FTS index maintenance
CREATE TRIGGER memories_ai AFTER INSERT ON memories    -- Add to FTS
CREATE TRIGGER memories_au AFTER UPDATE ON memories    -- Update FTS
CREATE TRIGGER memories_ad AFTER DELETE ON memories    -- Remove from FTS
CREATE TRIGGER memories_soft_delete AFTER UPDATE       -- Handle logical deletion
```

## 🧪 Testing Architecture

### Testing Strategy
**Levels**: Unit, Integration, Repository, Service
**Tools**: `rstest`, `pretty_assertions`, `tempfile`
**Approach**: Comprehensive coverage with realistic scenarios

### Test Organization
**Unit Tests**: Embedded in source files (`#[cfg(test)]`)
**Integration Tests**: Separate `tests/` directory
**Test Utilities**: Shared infrastructure in `tests/common/`
**Database Tests**: Temporary SQLite databases

### Key Testing Patterns
**Repository Testing**:
- Abstract trait testing for both implementations
- Transaction behavior validation
- FTS search functionality including Japanese
- Trigger system verification

**Service Testing**:
- Business logic validation
- Error condition handling
- Async operation testing
- Configuration validation

**Integration Testing**:
- End-to-end workflows
- Real database operations
- Multi-language content
- Performance characteristics

## 🚀 Concurrency & Performance

### Async Design
**Runtime**: Tokio with full feature set
**Patterns**: Async/await throughout service layer
**Concurrency**: Non-blocking reference count updates
**Resource Management**: Connection pooling with Arc<Mutex<>>

### Performance Optimizations
**Database**: SQLite WAL mode, optimized pragmas
**Indexing**: Strategic indexes for common queries
**Caching**: In-memory repository for testing performance
**Batch Operations**: Transaction-based bulk operations

### Resource Management
**Memory**: Efficient string handling, minimal cloning
**Database**: Connection reuse, prepared statements
**Threading**: Safe sharing with Arc<Mutex<>> patterns
**Error Handling**: Zero-cost error propagation

## 🔒 Error Handling Strategy

### Error Architecture
**Primary**: `anyhow::Result` for application errors
**Custom**: `thiserror` for domain-specific errors
**Propagation**: `?` operator for clean error flow
**Context**: Rich error messages with context

**Error Types**:
```rust
pub enum MemoryError {
    NotFound(String),           // Resource not found
    InvalidInput(String),       // Validation failures
    DatabaseError(String),      // Persistence issues
    ConfigurationError(String), // Config problems
    Io(std::io::Error),        // File system errors
}
```

### Error Handling Patterns
**Validation**: Early validation with descriptive messages
**Recovery**: Graceful degradation where possible
**Logging**: Structured logging with tracing
**Propagation**: Context preservation through error chain

## 🔧 Dependencies & External Libraries

### Core Dependencies
**CLI Framework**: `clap` v4.5 with derive features
**Async Runtime**: `tokio` v1 with full features
**Database**: `rusqlite` v0.31 with bundled SQLite
**Migrations**: `refinery` v0.8 for schema management
**Serialization**: `serde` v1 with derive features

### Development Dependencies
**Testing**: `rstest` v0.23 for parameterized tests
**Assertions**: `pretty_assertions` v1 for readable output
**Temporary Files**: `tempfile` v3 for test isolation

### Protocol Dependencies
**MCP Protocol**: `rmcp` v0.5.0 for server implementation
**JSON Schema**: `schemars` v1 for API documentation
**JSON Processing**: `serde_json` v1 for message handling

## 🎯 Quality Attributes

### Maintainability
**Modularity**: Clear separation of concerns
**Documentation**: Comprehensive inline documentation
**Testing**: High test coverage with realistic scenarios
**Code Style**: Consistent formatting and naming

### Reliability
**Error Handling**: Comprehensive error coverage
**Data Integrity**: Database constraints and validation
**Transaction Safety**: ACID compliance for batch operations
**Testing**: Thorough edge case coverage

### Performance
**Database**: Optimized queries with proper indexing
**Memory**: Efficient data structures and minimal copying
**Concurrency**: Non-blocking operations where beneficial
**Caching**: Strategic use of in-memory storage

### Security
**Input Validation**: Comprehensive validation at boundaries
**SQL Injection**: Parameterized queries throughout
**Path Safety**: Secure file system operations
**Error Information**: No sensitive data in error messages

## 🔮 Extension Points

### Architectural Flexibility
**Repository Pattern**: Easy to add new storage backends
**Service Layer**: Business logic changes isolated from infrastructure
**Command Pattern**: Simple to add new CLI commands
**Configuration**: TOML-based extensible configuration

### Planned Extensions
**Memory Types**: Additional categorization schemes
**Search Features**: Advanced query syntax and filters
**Export Formats**: Multiple output formats beyond Markdown
**MCP Features**: Extended protocol capability implementation

---

*This architecture documentation reflects the current implementation and design decisions. The system demonstrates solid architectural principles with room for growth and adaptation.*