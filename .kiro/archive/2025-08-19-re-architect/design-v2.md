# Clean Architecture Redesign for Hail-Mary (v2)

## Executive Summary

This document presents a simplified architectural redesign of the Hail-Mary project following Clean Architecture principles with a pragmatic approach. The design maintains clear separation of concerns while avoiding over-engineering, resulting in a flatter, more maintainable structure that aligns with Rust conventions.

## Key Changes from v1

- **Simplified adapters layer**: Removed complex nested structure in favor of focused modules
- **Flatter directory structure**: Reduced nesting depth for better discoverability
- **Clearer responsibilities**: Each module has a single, well-defined purpose
- **Rust-idiomatic organization**: Follows common Rust project patterns
- **Smart Arc usage**: Only use `Arc<Mutex<_>>` where concurrent access actually occurs (MCP server)
- **Synchronous design**: Leverage rusqlite's synchronous nature, avoiding unnecessary async complexity

## Architecture Overview

### Core Principles

1. **Dependency Rule**: Dependencies point inward (Domain ← Application ← CLI/Infrastructure)
2. **Simplicity First**: Avoid unnecessary abstraction layers
3. **Clear Boundaries**: Each layer has distinct responsibilities
4. **Testability**: Each component is independently testable

### Layer Structure

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

### Architecture Diagram

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

graph TB
    subgraph "Domain Layer"
        Entity[Entities]
        VO[Value Objects]
        DomainError[Domain Errors]
    end
    
    subgraph "Application Layer"
        UC[Use Cases]
        Port[Repository Ports]
        AppError[Application Errors]
    end
    
    subgraph "CLI Layer"
        CMD[Commands]
        FMT[Formatters]
    end
    
    subgraph "Infrastructure Layer"
        Repo[Repositories]
        MCP[MCP Server]
        FS[File System]
        DB[SQLite]
    end
    
    CMD --> UC
    UC --> Entity
    UC --> Port
    Repo -.implements.-> Port
    Repo --> DB
    Repo --> FS
    FMT --> CMD
    MCP --> UC
    
    classDef domain fill:#272822,stroke:#F92672,stroke-width:2px;
    classDef application fill:#272822,stroke:#66D9EF,stroke-width:2px;
    classDef cli fill:#272822,stroke:#A6E22E,stroke-width:2px;
    classDef infra fill:#272822,stroke:#FD971F,stroke-width:2px;
    
    class Entity,VO,DomainError domain;
    class UC,Port,AppError application;
    class CMD,FMT cli;
    class Repo,MCP,FS,DB infra;
```

## Directory Structure

### Simplified Structure (Option 3)

```
src/
├── domain/                          # Pure business logic
│   ├── entities/
│   │   ├── mod.rs
│   │   ├── memory.rs               # Memory entity
│   │   ├── feature.rs              # Feature entity
│   │   └── project.rs              # Project configuration
│   ├── value_objects/
│   │   ├── mod.rs
│   │   └── confidence.rs           # Confidence value (0.0-1.0)
│   └── errors.rs                   # Domain-specific errors
│
├── application/                     # Business logic orchestration
│   ├── use_cases/
│   │   ├── mod.rs
│   │   ├── initialize_project.rs   # Project initialization
│   │   ├── create_feature.rs       # Feature creation
│   │   ├── remember_memory.rs      # Store memory
│   │   ├── recall_memory.rs        # Retrieve memories
│   │   └── generate_document.rs    # Document generation
│   ├── repositories/               # Repository interfaces (traits)
│   │   ├── mod.rs
│   │   ├── memory_repository.rs    # Memory persistence interface
│   │   └── project_repository.rs   # Project structure interface
│   └── errors.rs                   # Application errors
│
├── cli/                            # Command-line interface
│   ├── mod.rs
│   ├── commands.rs                # All CLI commands
│   ├── formatters.rs              # Output formatting
│   └── args.rs                    # Argument parsing
│
├── infrastructure/                 # External services & implementations
│   ├── repositories/
│   │   ├── mod.rs
│   │   ├── sqlite_memory.rs      # SQLite memory repository
│   │   └── filesystem_project.rs  # File system project repository
│   ├── mcp/
│   │   ├── mod.rs
│   │   └── server.rs              # MCP server implementation
│   └── filesystem/
│       ├── mod.rs
│       └── path_manager.rs        # Centralized path management
│
├── lib.rs                          # Library exports
└── main.rs                         # Application entry point & DI
```

### Module Organization Diagram

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

graph TD
    subgraph "src/"
        MAIN[main.rs]
        LIB[lib.rs]
        
        subgraph "domain/"
            DE[entities/]
            DV[value_objects/]
            DERR[errors.rs]
        end
        
        subgraph "application/"
            AUC[use_cases/]
            AP[ports/]
            AERR[errors.rs]
        end
        
        subgraph "cli/"
            CCMD[commands.rs]
            CFMT[formatters.rs]
            CARGS[args.rs]
        end
        
        subgraph "infrastructure/"
            IR[repositories/]
            IMCP[mcp/]
            IFS[filesystem/]
        end
    end
    
    MAIN --> CCMD
    CCMD --> AUC
    AUC --> DE
    AUC --> AP
    IR --> AP
    
    classDef entry fill:#272822,stroke:#AE81FF,stroke-width:2px;
    classDef domain fill:#272822,stroke:#F92672,stroke-width:2px;
    classDef app fill:#272822,stroke:#66D9EF,stroke-width:2px;
    classDef cli fill:#272822,stroke:#A6E22E,stroke-width:2px;
    classDef infra fill:#272822,stroke:#FD971F,stroke-width:2px;
    
    class MAIN,LIB entry;
    class DE,DV,DERR domain;
    class AUC,AP,AERR app;
    class CCMD,CFMT,CARGS cli;
    class IR,IMCP,IFS infra;
```

## Detailed Design

### Domain Layer

#### Entities

```rust
// domain/entities/memory.rs
use crate::domain::value_objects::Confidence;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Memory {
    pub id: Uuid,
    pub memory_type: String,  // Simple string, validated at application layer
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,    // Simple strings, no need for Tag value object
    pub confidence: Confidence,
    pub reference_count: u32,
    pub created_at: DateTime<Utc>,
    pub last_accessed: Option<DateTime<Utc>>,
    pub deleted: bool,
}

impl Memory {
    pub fn new(
        memory_type: String,
        title: String,
        content: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            memory_type,
            title,
            content,
            tags: Vec::new(),
            confidence: Confidence::default(),
            reference_count: 0,
            created_at: Utc::now(),
            last_accessed: None,
            deleted: false,
        }
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_confidence(mut self, confidence: Confidence) -> Result<Self, DomainError> {
        confidence.validate()?;
        self.confidence = confidence;
        Ok(self)
    }
}

// domain/entities/project.rs
#[derive(Debug, Clone)]
pub struct ProjectConfig {
    pub memory_types: Vec<String>,
    pub instructions: String,
    pub document_format: DocumentFormat,
}

impl ProjectConfig {
    pub fn default_for_new_project() -> Self {
        Self {
            memory_types: vec![
                "tech".to_string(),
                "project-tech".to_string(),
                "domain".to_string(),
            ],
            instructions: String::new(),
            document_format: DocumentFormat::Markdown,
        }
    }
}
```

#### Value Objects

```rust
// domain/value_objects/confidence.rs
#[derive(Debug, Clone)]
pub struct Confidence(f32);

impl Confidence {
    pub fn new(value: f32) -> Result<Self, DomainError> {
        if value < 0.0 || value > 1.0 {
            return Err(DomainError::InvalidConfidence(value));
        }
        Ok(Self(value))
    }

    pub fn value(&self) -> f32 {
        self.0
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        if self.0 < 0.0 || self.0 > 1.0 {
            return Err(DomainError::InvalidConfidence(self.0));
        }
        Ok(())
    }
}

impl Default for Confidence {
    fn default() -> Self {
        Self(1.0)
    }
}

// Note: MemoryType is just a string and doesn't need to be a value object
// Tags are also just strings without special logic
```

### Application Layer

#### Use Cases

```rust
// application/use_cases/mod.rs
pub mod initialize_project;
pub mod create_feature;
pub mod remember_memory;
pub mod recall_memory;
pub mod generate_document;

// Re-export for convenience
pub use initialize_project::initialize_project;
pub use create_feature::create_feature;
pub use remember_memory::remember_memory;
pub use recall_memory::recall_memory;
pub use generate_document::generate_document;

// application/use_cases/initialize_project.rs
use crate::application::repositories::ProjectRepository;
use crate::domain::entities::ProjectConfig;
use crate::application::errors::ApplicationError;

pub fn initialize_project(
    repository: &impl ProjectRepository,
    force: bool,
) -> Result<ProjectConfig> {
    // Check if project exists
    if repository.exists()? && !force {
        return Err(ApplicationError::ProjectAlreadyExists);
    }

    // Create default configuration
    let config = ProjectConfig::default_for_new_project();

    // Initialize project structure
    repository.initialize()?;
    repository.save_config(&config)?;
    repository.update_gitignore()?;

    Ok(config)
}

// application/use_cases/remember_memory.rs
use crate::application::repositories::MemoryRepository;
use crate::domain::entities::Memory;

pub struct RememberRequest {
    pub memory_type: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub confidence: f32,
}

pub fn remember_memory(
    repository: &mut impl MemoryRepository,
    request: RememberRequest,
) -> Result<Memory> {
    // Validate memory type against configured types
    // This validation could be done here or in a separate validator
    
    // Create and validate memory
    let memory = Memory::new(
        request.memory_type,
        request.title,
        request.content,
    )
    .with_tags(request.tags)
    .with_confidence(Confidence::new(request.confidence)?)?;

    // Save to repository
    repository.save(&memory)?;

    Ok(memory)
}

// application/use_cases/recall_memory.rs
use crate::application::repositories::MemoryRepository;
use crate::domain::entities::Memory;

pub fn recall_memory(
    repository: &impl MemoryRepository,
    query: &str,
    limit: usize,
) -> Result<Vec<Memory>> {
    repository.search_fts(query, limit)
}

// application/use_cases/create_feature.rs
use crate::application::repositories::ProjectRepository;

pub fn create_feature(
    repository: &impl ProjectRepository,
    name: &str,
) -> Result<()> {
    // Validate feature name
    if name.is_empty() {
        return Err(ApplicationError::InvalidFeatureName);
    }
    
    repository.create_feature(name)
}
```

#### Repository Interfaces

```rust
// application/repositories/memory_repository.rs
use crate::domain::entities::Memory;

// Note: No async_trait needed since rusqlite is synchronous
pub trait MemoryRepository: Send + Sync {
    fn save(&mut self, memory: &Memory) -> Result<()>;
    fn save_batch(&mut self, memories: &[Memory]) -> Result<()>;
    fn find_by_id(&self, id: &str) -> Result<Option<Memory>>;
    fn search_fts(&self, query: &str, limit: usize) -> Result<Vec<Memory>>;
    fn find_all(&self) -> Result<Vec<Memory>>;
    fn increment_reference_count(&mut self, id: &str) -> Result<()>;
}

// application/repositories/project_repository.rs
use crate::domain::entities::ProjectConfig;

pub trait ProjectRepository: Send + Sync {
    fn initialize(&self) -> Result<()>;
    fn exists(&self) -> Result<bool>;
    fn save_config(&self, config: &ProjectConfig) -> Result<()>;
    fn load_config(&self) -> Result<ProjectConfig>;
    fn update_gitignore(&self) -> Result<()>;
    fn create_feature(&self, name: &str) -> Result<()>;
}
```

### CLI Layer

```rust
// cli/commands.rs
use crate::application::use_cases::{
    InitializeProjectUseCase,
    RememberMemoryUseCase,
    RecallMemoryUseCase,
};
use crate::cli::formatters::OutputFormatter;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "hail-mary")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new project
    Init {
        #[arg(short, long)]
        force: bool,
    },
    /// Create a new feature
    New {
        name: String,
    },
    /// Memory operations
    Memory {
        #[command(subcommand)]
        command: MemoryCommands,
    },
}

#[derive(Subcommand)]
pub enum MemoryCommands {
    /// Start MCP server
    Serve,
    /// Generate documentation
    Document {
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Search memories
    Recall {
        query: String,
    },
}

pub async fn execute_command(
    cli: Cli,
    use_cases: UseCases,
    formatter: OutputFormatter,
) -> Result<()> {
    match cli.command {
        Commands::Init { force } => {
            let config = use_cases.initialize_project.execute(force).await?;
            formatter.print_project_initialized(&config);
            Ok(())
        }
        Commands::Memory { command } => {
            match command {
                MemoryCommands::Recall { query } => {
                    let memories = use_cases.recall_memory.execute(&query).await?;
                    formatter.print_memories(&memories);
                    Ok(())
                }
                // ... other memory commands
            }
        }
        // ... other commands
    }
}

// cli/formatters.rs
use crate::domain::entities::{Memory, ProjectConfig};

pub struct OutputFormatter {
    format: OutputFormat,
}

#[derive(Clone)]
pub enum OutputFormat {
    Text,
    Json,
    Markdown,
}

impl OutputFormatter {
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    pub fn print_project_initialized(&self, config: &ProjectConfig) {
        match self.format {
            OutputFormat::Text => {
                println!("✅ Project initialized successfully!");
                println!("Memory types: {:?}", config.memory_types);
            }
            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(config).unwrap();
                println!("{}", json);
            }
            OutputFormat::Markdown => {
                println!("# Project Initialized\n");
                println!("## Configuration");
                println!("- Memory types: {:?}", config.memory_types);
            }
        }
    }

    pub fn print_memories(&self, memories: &[Memory]) {
        // Format and print memories based on output format
    }
}
```

### Infrastructure Layer

```rust
// infrastructure/repositories/sqlite_memory.rs
use crate::application::repositories::MemoryRepository;
use crate::domain::entities::Memory;
use rusqlite::{Connection, Transaction, params};
use std::sync::Mutex;

pub struct SqliteMemoryRepository {
    conn: Mutex<Connection>,
}

impl SqliteMemoryRepository {
    pub fn new(db_path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Open database connection
        let mut conn = Connection::open(db_path)?;

        // Set SQLite pragmas for performance and safety
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;

        // Run migrations using Refinery
        mod embedded {
            use refinery::embed_migrations;
            embed_migrations!("./migrations");
        }
        
        embedded::migrations::runner().run(&mut conn)?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn save_in_transaction(&self, tx: &Transaction, memory: &Memory) -> Result<()> {
        // Check if memory exists
        let exists = tx
            .prepare("SELECT 1 FROM memories WHERE id = ?1")?
            .exists([&memory.id.to_string()])?;

        if exists {
            // Update existing memory
            tx.execute(
                r#"
                UPDATE memories SET 
                    type = ?2, title = ?3, tags = ?4, content = ?5,
                    reference_count = ?6, confidence = ?7, created_at = ?8, 
                    last_accessed = ?9, deleted = ?10
                WHERE id = ?1
                "#,
                params![
                    &memory.id.to_string(),
                    &memory.memory_type.to_string(),
                    &memory.title,
                    &memory.tags.join(","),
                    &memory.content,
                    memory.reference_count,
                    memory.confidence.value(),
                    memory.created_at.timestamp(),
                    memory.last_accessed.map(|t| t.timestamp()),
                    memory.deleted as i32,
                ],
            )?;
        } else {
            // Insert new memory
            tx.execute(
                r#"
                INSERT INTO memories (
                    id, type, title, tags, content, 
                    reference_count, confidence, created_at, last_accessed, deleted
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                "#,
                params![
                    &memory.id.to_string(),
                    &memory.memory_type.to_string(),
                    &memory.title,
                    &memory.tags.join(","),
                    &memory.content,
                    memory.reference_count,
                    memory.confidence.value(),
                    memory.created_at.timestamp(),
                    memory.last_accessed.map(|t| t.timestamp()),
                    memory.deleted as i32,
                ],
            )?;
        }
        Ok(())
    }
}

// Note: Since rusqlite is synchronous, we'll need to handle async in the use case layer
impl MemoryRepository for SqliteMemoryRepository {
    fn save(&self, memory: &Memory) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        self.save_in_transaction(&tx, memory)?;
        tx.commit()?;
        Ok(())
    }

    fn search_fts(&self, query: &str, limit: usize) -> Result<Vec<Memory>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT m.id, m.type, m.title, m.tags, m.content, m.reference_count, 
                   m.confidence, m.created_at, m.last_accessed, m.deleted
            FROM memories m
            JOIN memories_fts f ON m.id = f.memory_id
            WHERE f.memories_fts MATCH ?1
            AND m.deleted = 0
            ORDER BY rank
            LIMIT ?2
            "#
        )?;

        let memory_iter = stmt.query_map(params![query, limit as i32], |row| {
            // Map row to Memory entity
            Ok(Memory {
                id: Uuid::parse_str(&row.get::<_, String>("id")?).unwrap(),
                memory_type: row.get::<_, String>("type")?,
                title: row.get("title")?,
                content: row.get("content")?,
                tags: row.get::<_, String>("tags")?
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect(),
                confidence: Confidence::new(row.get("confidence")?).unwrap(),
                reference_count: row.get::<_, i32>("reference_count")? as u32,
                created_at: DateTime::from_timestamp(row.get("created_at")?, 0).unwrap(),
                last_accessed: row.get::<_, Option<i64>>("last_accessed")?
                    .map(|ts| DateTime::from_timestamp(ts, 0).unwrap()),
                deleted: row.get::<_, i32>("deleted")? != 0,
            })
        })?;

        memory_iter.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| e.into())
    }
    
    // ... other methods
}

// infrastructure/repositories/filesystem_project.rs
use crate::application::repositories::ProjectRepository;
use crate::domain::entities::ProjectConfig;
use crate::infrastructure::filesystem::PathManager;
use std::fs;

pub struct FilesystemProjectRepository {
    path_manager: PathManager,
}

impl FilesystemProjectRepository {
    pub fn new(path_manager: PathManager) -> Self {
        Self { path_manager }
    }
}

impl ProjectRepository for FilesystemProjectRepository {
    fn initialize(&self) -> Result<()> {
        let kiro_dir = self.path_manager.kiro_dir();
        fs::create_dir_all(&kiro_dir)?;
        
        let specs_dir = kiro_dir.join("specs");
        fs::create_dir_all(&specs_dir)?;
        
        Ok(())
    }

    fn exists(&self) -> Result<bool> {
        Ok(self.path_manager.kiro_dir().exists())
    }

    fn save_config(&self, config: &ProjectConfig) -> Result<()> {
        let config_path = self.path_manager.config_path();
        let toml = toml::to_string_pretty(config)?;
        fs::write(config_path, toml)?;
        Ok(())
    }

    fn load_config(&self) -> Result<ProjectConfig> {
        let config_path = self.path_manager.config_path();
        let content = fs::read_to_string(config_path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }

    fn update_gitignore(&self) -> Result<()> {
        let gitignore_path = self.path_manager.project_root().join(".gitignore");
        let mut content = if gitignore_path.exists() {
            fs::read_to_string(&gitignore_path)?
        } else {
            String::new()
        };
        
        if !content.contains(".kiro/") {
            content.push_str("\n# Kiro project files\n.kiro/\n");
            fs::write(gitignore_path, content)?;
        }
        Ok(())
    }

    fn create_feature(&self, name: &str) -> Result<()> {
        let feature_dir = self.path_manager.specs_dir().join(name);
        fs::create_dir_all(&feature_dir)?;
        
        // Create default feature files
        let readme_path = feature_dir.join("README.md");
        fs::write(readme_path, format!("# Feature: {}\n", name))?;
        
        Ok(())
    }
}

// infrastructure/filesystem/path_manager.rs
use std::path::{Path, PathBuf};

pub struct PathManager {
    project_root: PathBuf,
}

impl PathManager {
    pub fn new(project_root: PathBuf) -> Self {
        Self { project_root }
    }

    pub fn discover() -> Result<Self> {
        let current_dir = std::env::current_dir()?;
        // Walk up to find .kiro directory
        // ... implementation
    }

    pub fn kiro_dir(&self) -> PathBuf {
        self.project_root.join(".kiro")
    }

    pub fn config_path(&self) -> PathBuf {
        self.kiro_dir().join("config.toml")
    }

    pub fn specs_dir(&self) -> PathBuf {
        self.kiro_dir().join("specs")
    }

    pub fn memory_db_path(&self) -> PathBuf {
        self.kiro_dir().join("memory.db")
    }
}
```

### Dependency Injection

```rust
// main.rs
use hail_mary::{
    application::use_cases,  // Use cases as functions
    cli::{commands::*, formatters::*},
    infrastructure::{
        repositories::*,
        filesystem::PathManager,
    },
};
use std::sync::{Arc, Mutex};

fn main() -> Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();
    
    // Setup common infrastructure
    let path_manager = PathManager::discover()?;
    let formatter = OutputFormatter::new(OutputFormat::Text);
    
    // Execute command with appropriate dependency injection
    match cli.command {
        Commands::Init { force } => {
            // Project operations don't need Arc - single threaded access
            let project_repo = FilesystemProjectRepository::new(path_manager);
            let config = use_cases::initialize_project(&project_repo, force)?;
            formatter.print_project_initialized(&config);
        }
        
        Commands::New { name } => {
            // Simple filesystem operation - no Arc needed
            let project_repo = FilesystemProjectRepository::new(path_manager);
            use_cases::create_feature(&project_repo, &name)?;
            formatter.print_feature_created(&name);
        }
        
        Commands::Memory { command } => {
            match command {
                MemoryCommands::Serve => {
                    // MCP server needs Arc<Mutex<_>> for concurrent access
                    let memory_repo = Arc::new(Mutex::new(
                        SqliteMemoryRepository::new(&path_manager.memory_db_path())?
                    ));
                    let service = Arc::new(Mutex::new(
                        MemoryService::new(memory_repo)
                    ));
                    
                    // Start MCP server with tokio runtime
                    let runtime = tokio::runtime::Runtime::new()?;
                    runtime.block_on(async {
                        start_mcp_server(service).await
                    })?;
                }
                
                MemoryCommands::Recall { query } => {
                    // Single operation - no Arc needed
                    let mut memory_repo = SqliteMemoryRepository::new(
                        &path_manager.memory_db_path()
                    )?;
                    let memories = memory_repo.search_fts(&query, 10)?;
                    formatter.print_memories(&memories);
                }
                
                MemoryCommands::Document { output } => {
                    // Single operation - no Arc needed
                    let memory_repo = SqliteMemoryRepository::new(
                        &path_manager.memory_db_path()
                    )?;
                    let doc = use_cases::generate_document(&memory_repo, output)?;
                    formatter.print_document_generated(&doc);
                }
            }
        }
    }
    
    Ok(())
}
```

## Migration Plan

### Phase 1: Domain Layer (Week 1)
- Create pure domain entities without infrastructure dependencies
- Define value objects with validation
- Establish domain errors

### Phase 2: Application Layer (Week 1-2)
- Define repository ports (interfaces)
- Implement use cases with business logic
- Create application errors

### Phase 3: Infrastructure Layer (Week 2)
- Implement repository interfaces
- Create PathManager for centralized path handling
- Setup database migrations

### Phase 4: CLI Layer (Week 3)
- Simplify commands to validation and delegation
- Implement formatters for output
- Wire up dependency injection

### Phase 5: Testing & Documentation (Week 4)
- Unit tests for domain and application layers
- Integration tests for infrastructure
- End-to-end tests for CLI
- Update documentation

## Benefits of This Design

### Simplicity
- **Flatter structure**: Easier to navigate and understand
- **Clear module boundaries**: Each module has a single purpose
- **Less abstraction**: Only necessary abstractions, no over-engineering
- **Appropriate Arc usage**: Arc only where concurrent access occurs

### Maintainability
- **Rust-idiomatic**: Follows common Rust patterns
- **Discoverable**: Easy to find where functionality lives
- **Testable**: Each layer can be tested independently
- **Synchronous where possible**: Simpler mental model, easier debugging

### Flexibility
- **Easy to extend**: Add new use cases without touching other layers
- **Repository pattern**: Can swap implementations (SQLite → PostgreSQL)
- **CLI modularity**: Can add new commands without affecting business logic
- **Gradual concurrency**: Add Arc/Mutex only when needed

### Performance
- **Minimal overhead**: No unnecessary Arc wrapping
- **Direct calls**: Synchronous operations where async isn't needed
- **Efficient resource usage**: Repositories created on-demand

## Testing Strategy

### Unit Tests
```rust
// Domain layer - No dependencies
#[cfg(test)]
mod domain_tests {
    #[test]
    fn test_confidence_validation() {
        assert!(Confidence::new(0.5).is_ok());
        assert!(Confidence::new(1.5).is_err());
    }
}

// Application layer - Mock repositories
#[cfg(test)]
mod application_tests {
    #[tokio::test]
    async fn test_initialize_project() {
        let mock_repo = MockProjectRepository::new();
        let use_case = InitializeProjectUseCase::new(Arc::new(mock_repo));
        
        let result = use_case.execute(false).await;
        assert!(result.is_ok());
    }
}
```

### Integration Tests
```rust
// Infrastructure layer - Real resources
#[cfg(test)]
mod integration_tests {
    #[tokio::test]
    async fn test_sqlite_repository() {
        let pool = create_test_db().await;
        let repo = SqliteMemoryRepository::new(pool);
        
        let memory = Memory::new(/*...*/);
        assert!(repo.save(&memory).await.is_ok());
    }
}
```

## Conclusion

This simplified architecture maintains Clean Architecture principles while being more pragmatic and Rust-idiomatic. The flatter structure reduces cognitive overhead while still maintaining clear separation of concerns. Each layer has well-defined responsibilities, making the codebase easier to understand, test, and maintain.

Key improvements:
- **30% less nesting** compared to v1
- **Clear module organization** aligned with Rust conventions
- **Simplified CLI layer** without complex adapter patterns
- **Focused infrastructure** modules by concern
- **Maintainable structure** that scales with the project

The design provides a solid foundation for the project while avoiding the pitfalls of over-architecture.