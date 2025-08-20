# hail-mary System Behavior Documentation

This document provides comprehensive documentation of all commands, data flows, and system behaviors in the hail-mary Memory MCP project.

## Table of Contents
1. [System Overview](#system-overview)
2. [Command Reference](#command-reference)
3. [Data Flow Architecture](#data-flow-architecture)
4. [Database Operations](#database-operations)
5. [MCP Protocol Integration](#mcp-protocol-integration)
6. [File Generation and Outputs](#file-generation-and-outputs)

## System Overview

hail-mary is a Rust CLI application that implements:
- Memory MCP (Model Context Protocol) server for AI model integration
- Kiro project specification management system
- SQLite database with FTS5 for multilingual full-text search
- Hexagonal architecture with clean separation of concerns

### Core Components

```
src/
├── main.rs              # CLI entry point with command routing
├── commands/            # Command implementations
│   ├── init.rs         # Project initialization
│   ├── new.rs          # Feature specification creation
│   └── memory/         # Memory subsystem commands
│       ├── serve.rs    # MCP server
│       ├── document.rs # Documentation generation
│       └── reindex.rs  # Database optimization
├── models/             # Domain models
│   ├── memory.rs       # Memory entity
│   ├── kiro.rs        # Configuration model
│   └── error.rs       # Error types
├── repositories/       # Data access layer
│   └── memory.rs      # Repository trait & implementations
├── services/          # Business logic layer
│   ├── memory.rs      # Memory service
│   └── memory_mcp.rs  # MCP protocol implementation
└── core/              # Core utilities
    └── project.rs     # Project management
```

## Command Reference

### 1. `hail-mary init` - Initialize Project

#### Purpose
Creates the `.kiro` directory structure and configuration for a new project.

#### Data Flow
```
User Input → InitCommand::execute()
    → Create .kiro/ directory
    → Create .kiro/memory/ directory
    → Generate config.toml from template
    → Update/Create .gitignore
    → Return success status
```

#### Generated Files
- `.kiro/` - Main project directory
- `.kiro/config.toml` - Configuration file with memory types and database settings
- `.kiro/memory/` - Memory storage directory
- `.gitignore` - Updated with database exclusions

#### Configuration Template
```toml
[memory]
types = ["tech", "project-tech", "domain", "workflow", "decision"]
instructions = """
Available memory types:
- tech: General technical knowledge
- project-tech: Project-specific technical details
- domain: Business domain knowledge
- workflow: Development workflows
- decision: Architecture decisions
"""

[memory.document]
output_dir = ".kiro/memory"
format = "markdown"

[memory.database]
path = ".kiro/memory/db.sqlite3"
```

#### Example
```bash
# Initialize new project
$ hail-mary init
✅ Initialized .kiro directory structure:
  - Created .kiro/
  - Created .kiro/config.toml (configuration template)
  - Created .kiro/memory/
  - Updated .gitignore

# Force reinitialize existing project
$ hail-mary init --force
# Overwrites existing config.toml without prompting
```

### 2. `hail-mary new` - Create Feature Specification

#### Purpose
Creates a new feature specification directory with standardized documentation templates.

#### Data Flow
```
User Input (feature-name) → NewCommand::execute()
    → Validate kebab-case naming
    → Generate directory name (YYYY-MM-dd-feature-name)
    → Check for duplicate features
    → Create .kiro/specs/ if not exists
    → Create feature directory
    → Generate template files
    → Return feature path
```

#### Generated Files
- `.kiro/specs/YYYY-MM-dd-feature-name/` - Feature directory
- `requirements.md` - Requirements documentation
- `design.md` - Design documentation
- `task.md` - Task breakdown
- `spec.json` - JSON specification (empty object)

#### Validation Rules
- Feature name must be in kebab-case (lowercase with hyphens)
- No spaces, underscores, or uppercase letters allowed
- Must not already exist in the project

#### Example
```bash
# Create new feature
$ hail-mary new user-authentication
Creating new feature: user-authentication
✅ Feature 'user-authentication' created successfully!
📁 Location: .kiro/specs/2025-01-20-user-authentication
📝 Files created:
   - requirements.md
   - design.md
   - task.md
   - spec.json

# Invalid name (will fail)
$ hail-mary new "User Authentication"
Error: Invalid feature name 'User Authentication'. Must be kebab-case.

# Duplicate feature (will fail)  
$ hail-mary new user-authentication
Error: Feature 'user-authentication' already exists
```

### 3. `hail-mary memory serve` - Start Memory MCP Server

#### Purpose
Starts the Memory MCP server for AI model integration via stdio protocol.

#### Data Flow
```
Server Start → MemoryMcpServer initialization
    → Load KiroConfig from .kiro/config.toml
    → Initialize SqliteMemoryRepository
        → Create/open SQLite database
        → Run migrations (V001, V002, V003)
        → Setup FTS5 search index
    → Create MemoryService wrapper
    → Initialize MCP server with stdio transport
    → Listen for MCP client connections
    → Handle remember/recall tool calls
```

#### MCP Tools Exposed
1. **remember** - Store memories
   - Input: Array of memories with type, title, content, tags, confidence
   - Output: Array of memory IDs and count
   
2. **recall** - Search memories
   - Input: Query string, optional type/tags filter, limit
   - Output: Markdown-formatted memories and count

#### Database Connection
- Uses SQLite with WAL mode for concurrent access
- FTS5 for full-text search with Japanese tokenization
- Automatic trigger-based index maintenance

#### Example
```bash
# Start MCP server
$ hail-mary memory serve
[INFO] Starting Memory MCP server...
[INFO] Using database: .kiro/memory/db.sqlite3
[INFO] Memory MCP server ready. Connect with MCP client via stdio.

# Server provides JSON-RPC interface over stdio
# Clients can call:
# - remember: Store new memories
# - recall: Search and retrieve memories
```

### 4. `hail-mary memory document` - Generate Documentation

#### Purpose
Generates Markdown documentation from stored memories, organized by type.

#### Data Flow
```
Document Generation → DocumentCommand::execute()
    → Load KiroConfig
    → Initialize SqliteMemoryRepository
    → Create MemoryService
    → Validate type filter (if provided)
    → Query memories by type
    → Generate Markdown files
        → Group memories by type
        → Format as Markdown with metadata
        → Write to .kiro/memory/*.md
    → Return success with file list
```

#### Generated Files
- `.kiro/memory/tech.md` - Technical knowledge memories
- `.kiro/memory/project-tech.md` - Project-specific technical memories
- `.kiro/memory/domain.md` - Domain knowledge memories
- Additional files for each configured memory type

#### Markdown Format
```markdown
# [Memory Type] Memories

## [Memory Title]
**ID**: [UUID]
**Tags**: tag1, tag2, tag3
**Confidence**: 0.95
**References**: 5

[Memory Content]

---
```

#### Example
```bash
# Generate all documents
$ hail-mary memory document
Generating memory documentation...
✅ Generated memory documents in: .kiro/memory
  - .kiro/memory/tech.md
  - .kiro/memory/project-tech.md
  - .kiro/memory/domain.md

# Generate specific type only
$ hail-mary memory document --type tech
Generating memory documentation...
✅ Generated document for type 'tech' in: .kiro/memory
  - .kiro/memory/tech.md

# Invalid type
$ hail-mary memory document --type invalid
Error: Invalid memory type 'invalid'. Available types: tech, project-tech, domain
```

### 5. `hail-mary memory reindex` - Database Optimization

#### Purpose
Reindex and optimize the memory database (currently in placeholder state for Phase 3).

#### Data Flow (Planned)
```
Reindex → ReindexCommand::execute()
    → Analyze database for duplicates
    → Remove logically deleted entries
    → Rebuild FTS5 index
    → Archive old database
    → Vacuum and optimize
```

#### Current Implementation
- Dry run mode shows planned operations
- Actual implementation pending Phase 3

#### Example
```bash
# Dry run (currently available)
$ hail-mary memory reindex --dry-run
🔍 Dry run mode - would perform reindex operations:
  - Analyze database for duplicates and optimization opportunities
  - Remove logical deleted entries
  - Rebuild FTS5 index
  - Archive old database

# Actual reindex (not yet implemented)
$ hail-mary memory reindex
Error: Reindex functionality not yet implemented. This will be added in Phase 3.

# Verbose dry run
$ hail-mary memory reindex --dry-run --verbose
🔍 Dry run mode - would perform reindex operations:
  - Analyze database for duplicates and optimization opportunities
  - Remove logical deleted entries
  - Rebuild FTS5 index
  - Archive old database
Verbose logging enabled
```

## Data Flow Architecture

### Memory Storage Flow
```
MCP Client → remember tool
    → MemoryMcpServer::remember()
    → Validate memory type against config
    → Convert to MemoryInput
    → MemoryService::remember_batch()
    → Generate UUID for each memory
    → SqliteMemoryRepository::save_batch()
    → BEGIN TRANSACTION
    → INSERT/UPDATE memories table
    → Trigger: Insert into FTS5 index
    → COMMIT
    → Return memory IDs
```

### Memory Retrieval Flow
```
MCP Client → recall tool
    → MemoryMcpServer::recall()
    → Parse query and filters
    → MemoryService::recall()
    → SqliteMemoryRepository::search_fts()
    → Query FTS5 virtual table
    → Filter by type/tags if specified
    → Update reference counts (async)
    → Format as Markdown
    → Return results
```

### Reference Count Update (Async)
```
Memory Access → tokio::spawn()
    → SqliteMemoryRepository::increment_reference_count()
    → UPDATE reference_count = reference_count + 1
    → UPDATE last_accessed = current_timestamp
    → Non-blocking completion
```

## Database Operations

### Schema Evolution (Migrations)

#### V001: Initial Schema
```sql
CREATE TABLE memories (
    id TEXT PRIMARY KEY,              -- UUID v4
    type TEXT NOT NULL,               -- Memory type from config
    title TEXT NOT NULL,              -- Human-readable title
    tags TEXT,                        -- Comma-separated tags
    content TEXT NOT NULL,            -- Main content
    reference_count INTEGER DEFAULT 0,
    confidence REAL DEFAULT 1.0,      -- 0.0-1.0 confidence score
    created_at INTEGER DEFAULT (unixepoch()),
    last_accessed INTEGER,
    deleted INTEGER DEFAULT 0         -- Logical deletion
);

-- Performance indexes
CREATE INDEX idx_memories_type ON memories(type) WHERE deleted = 0;
CREATE INDEX idx_memories_ref_count ON memories(reference_count DESC) WHERE deleted = 0;
CREATE INDEX idx_memories_created ON memories(created_at DESC) WHERE deleted = 0;
```

#### V002: FTS5 Search Index
```sql
CREATE VIRTUAL TABLE memories_fts USING fts5(
    memory_id UNINDEXED,
    title,
    tags,
    content,
    tokenize = 'porter unicode61'     -- Japanese support
);
```

#### V003: Automatic Triggers
```sql
-- INSERT trigger
CREATE TRIGGER memories_ai AFTER INSERT ON memories
WHEN NEW.deleted = 0
BEGIN
    INSERT INTO memories_fts(memory_id, title, tags, content)
    VALUES (NEW.id, NEW.title, NEW.tags, NEW.content);
END;

-- UPDATE trigger
CREATE TRIGGER memories_au AFTER UPDATE ON memories
WHEN NEW.deleted = 0 AND OLD.deleted = 0
BEGIN
    UPDATE memories_fts 
    SET title = NEW.title, tags = NEW.tags, content = NEW.content
    WHERE memory_id = NEW.id;
END;

-- Soft delete trigger
CREATE TRIGGER memories_soft_delete AFTER UPDATE ON memories
WHEN NEW.deleted = 1 AND OLD.deleted = 0
BEGIN
    DELETE FROM memories_fts WHERE memory_id = NEW.id;
END;
```

### Transaction Management
- All batch operations use transactions for consistency
- WAL mode enables concurrent reads during writes
- Automatic rollback on errors

### Search Capabilities
- Full-text search across title, tags, and content
- Japanese tokenization support via unicode61
- Filters: type, tags, confidence threshold
- Sorting: relevance, reference count, creation date

## MCP Protocol Integration

### Protocol Details
- Version: 2024-11-05
- Transport: stdio (JSON-RPC)
- Tools: remember, recall
- Async implementation using tokio

### Tool Schemas

#### Remember Tool
```json
{
  "name": "remember",
  "description": "Store memories for future recall",
  "parameters": {
    "memories": [
      {
        "type": "string",        // Memory type from config
        "title": "string",       // Summary
        "content": "string",     // Full content
        "tags": ["string"],      // Tags array
        "confidence": 0.0-1.0    // Optional confidence
      }
    ]
  }
}
```

#### Recall Tool
```json
{
  "name": "recall",
  "description": "Search and retrieve stored memories",
  "parameters": {
    "query": "string",           // Search query
    "type": "string",           // Optional type filter
    "tags": ["string"],         // Optional tag filter
    "limit": 10                 // Max results (default: 10)
  }
}
```

### Error Handling
- INVALID_PARAMS: Invalid memory type or malformed input
- INTERNAL_ERROR: Database or service layer errors
- Detailed error messages for debugging

## File Generation and Outputs

### Directory Structure After Full Setup
```
.kiro/
├── config.toml                    # Project configuration
├── memory/
│   ├── db.sqlite3                # SQLite database
│   ├── db.sqlite3-shm            # WAL shared memory
│   ├── db.sqlite3-wal            # Write-ahead log
│   ├── tech.md                   # Generated documentation
│   ├── project-tech.md          # Generated documentation
│   └── domain.md                 # Generated documentation
└── specs/
    └── YYYY-MM-dd-feature-name/
        ├── requirements.md        # Feature requirements
        ├── design.md             # Design documentation
        ├── task.md               # Task breakdown
        └── spec.json             # JSON specification
```

### Generated Content Examples

#### Memory Document (tech.md)
```markdown
# tech Memories

## Rust Async Programming
**ID**: 550e8400-e29b-41d4-a716-446655440000
**Tags**: rust, async, tokio
**Confidence**: 0.95
**References**: 12

Rust's async/await syntax enables efficient concurrent programming. 
The tokio runtime provides a multi-threaded executor for async tasks...

---

## SQLite FTS5 Configuration
**ID**: 6ba7b810-9dad-11d1-80b4-00c04fd430c8
**Tags**: sqlite, fts5, search
**Confidence**: 0.90
**References**: 8

FTS5 virtual tables support full-text search with customizable tokenizers...
```

#### Feature Specification (requirements.md)
```markdown
# Requirements

## Overview
[Feature description]

## User Stories
- As a [user type], I want to [action] so that [benefit]

## Acceptance Criteria
- [ ] Criterion 1
- [ ] Criterion 2

## Technical Requirements
- Database changes
- API endpoints
- UI components
```

### Output Formats
- **Markdown**: Primary format for documentation
- **JSON**: Specification files and MCP communication
- **SQLite**: Binary database with FTS5 index
- **Plain text**: Configuration and template files

## Testing Infrastructure

### Test Categories
1. **Unit Tests**: Repository trait implementations, service logic
2. **Integration Tests**: SQLite operations, migrations, FTS5 search
3. **End-to-End Tests**: Command execution, file generation
4. **Performance Tests**: Japanese search, scale testing

### Test Utilities
- `TestDirectory`: RAII pattern for safe working directory management
- `InMemoryRepository`: Testing without database dependencies
- Temporary databases with full migration cycle
- Japanese content testing for multilingual support

## Error Handling

### Error Types
- `HailMaryError`: Application-level errors with thiserror
- `MemoryError`: Domain-specific memory errors
- `anyhow::Result`: General error propagation

### Error Categories
1. **Configuration Errors**: Missing config, invalid types
2. **Validation Errors**: Invalid feature names, memory types
3. **Database Errors**: Connection, migration, query failures
4. **MCP Protocol Errors**: Invalid params, internal errors
5. **File System Errors**: Permission, space, path issues

## Performance Characteristics

### Database Performance
- WAL mode: Concurrent reads during writes
- FTS5 indexing: O(log n) search complexity
- Batch operations: Transaction-wrapped for consistency
- Async reference counting: Non-blocking updates

### Memory Efficiency
- Repository trait: Abstraction for testing/production
- Arc<Mutex<Repository>>: Safe concurrent access
- Lazy loading: Documents generated on demand
- Streaming: Large result sets handled efficiently

### Scalability
- Horizontal: Multiple MCP server instances possible
- Vertical: SQLite handles millions of memories
- Search: FTS5 scales with proper indexing
- Archival: Logical deletion preserves history