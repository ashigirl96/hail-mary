# Project Structure and Organization

## Root Directory Organization

```
hail-mary/                        # Project root
├── Cargo.toml                   # Workspace configuration
├── crates/                      # Multi-crate workspace
│   ├── hail-mary/              # Main CLI application
│   └── anthropic-client/        # OAuth client library
├── .kiro/                       # Kiro specification system
│   ├── config.toml             # Steering configuration
│   ├── specs/                  # Active specifications
│   ├── archive/                # Completed specifications
│   └── steering/               # Steering documentation
├── .claude/                     # Claude Code integration
│   ├── agents/                 # Specialized agents
│   ├── commands/               # Custom slash commands
│   └── hooks/                  # Hook configurations
├── reference/                   # External documentation
├── experimental/               # Prototype code
└── tests/                      # Integration tests
```

## Subdirectory Structures

### Core Application Structure (`crates/hail-mary/src/`)
```
src/
├── main.rs                     # CLI entry point
├── lib.rs                      # Library exports
├── domain/                     # Business logic layer
│   ├── entities/              # Core domain objects
│   │   ├── project.rs        # Project configuration
│   │   ├── steering.rs       # Steering entities
│   │   └── steering_reminder.rs
│   ├── value_objects/         # Immutable domain types
│   │   ├── spec.rs           # Spec validation
│   │   ├── system_prompt/    # System prompt with templates
│   │   │   ├── mod.rs
│   │   │   ├── template.md
│   │   │   └── specification_section_template.md
│   │   ├── steering_reminder/ # Reminder with template
│   │   │   ├── mod.rs
│   │   │   └── template.md
│   │   ├── steering_analysis/ # Analysis with template
│   │   │   ├── mod.rs
│   │   │   └── prompt_template.md
│   │   └── steering.rs       # Steering value objects
│   └── errors.rs              # Domain errors
├── application/                # Use case layer
│   ├── use_cases/            # Business operations
│   │   ├── initialize_project.rs
│   │   ├── create_feature.rs
│   │   ├── complete_features.rs
│   │   ├── backup_steering.rs
│   │   ├── remind_steering.rs
│   │   └── launch_claude_with_spec.rs
│   ├── repositories/          # Repository interfaces
│   │   ├── config_repository.rs
│   │   ├── spec_repository.rs
│   │   ├── steering_repository.rs
│   │   └── anthropic_repository.rs
│   └── test_helpers/          # Test utilities
├── cli/                        # Presentation layer
│   ├── commands/             # Command implementations
│   │   ├── init.rs
│   │   ├── new.rs
│   │   ├── code.rs
│   │   ├── complete.rs
│   │   ├── completion.rs
│   │   ├── steering_backup.rs
│   │   └── steering_remind.rs
│   ├── args.rs               # CLI argument parsing
│   └── formatters.rs         # Output formatting
└── infrastructure/            # External services layer
    ├── repositories/         # Repository implementations
    │   ├── config.rs
    │   ├── spec.rs
    │   ├── steering.rs
    │   └── anthropic.rs
    ├── filesystem/           # File operations
    │   └── path_manager.rs
    ├── process/              # Process management
    │   └── claude_launcher.rs
    ├── tui/                  # Terminal UI
    │   ├── spec_selector.rs
    │   └── completion_ui.rs
    └── embedded_resources.rs # Template resources
```

### Kiro System Structure (`.kiro/`)
```
.kiro/
├── config.toml                 # Project configuration
├── specs/                      # Active specifications
│   └── YYYY-MM-DD-{name}/    # Date-prefixed specs
│       ├── requirements.md    # User stories & criteria
│       ├── design.md          # Technical decisions
│       ├── tasks.md           # Implementation tasks
│       ├── investigation.md   # Research findings
│       ├── memo.md           # Additional notes
│       └── spec.json         # Metadata
├── archive/                    # Completed specs
│   └── [archived specs]       # Same structure as specs/
├── steering/                   # Context management
│   ├── product.md            # Product overview
│   ├── tech.md               # Technology stack
│   ├── structure.md          # This file
│   ├── documentation.md      # Doc standards
│   ├── prompt-engineering.md # Claude patterns
│   ├── rust-dev.md          # Rust specifics
│   ├── subagent.md          # Agent patterns
│   └── backup/              # Timestamped backups
│       └── YYYYMMDD-HHMMSS/ # Backup snapshots
└── draft/                     # Temporary drafts
```

### Claude Integration Structure (`.claude/`)
```
.claude/
├── agents/                    # Specialized agents
│   ├── steering-investigator.md
│   └── root-cause-investigator.md
├── commands/                  # Custom commands
│   └── hm/                   # Hail-mary commands
│       ├── steering.md
│       ├── steering-remember.md
│       ├── requirements.md
│       └── investigate.md
└── hooks/                     # Hook scripts
    ├── README.md
    └── user_prompt_submit.sh
```

## Code Organization Patterns

### Clean Architecture Layers
1. **Domain Layer** (`domain/`)
   - Pure business logic
   - No external dependencies
   - Entities and value objects

2. **Application Layer** (`application/`)
   - Use case orchestration
   - Repository interfaces
   - Business rule enforcement

3. **Infrastructure Layer** (`infrastructure/`)
   - External service implementations
   - File system operations
   - Process management

4. **CLI Layer** (`cli/`)
   - Command routing
   - Argument parsing
   - Output formatting

### Repository Pattern
- **Trait Interfaces**: Define contracts in `application/repositories/`
- **Implementations**: Concrete implementations in `infrastructure/repositories/`
- **Test Doubles**: Mock implementations in `application/test_helpers/`

### Command Pattern
- Each CLI command as separate module
- Consistent execute pattern
- Separation of parsing and execution

## File Naming Conventions

### Rust Files
- **Module files**: `snake_case.rs` (e.g., `path_manager.rs`)
- **Test files**: Embedded in source with `#[cfg(test)]`
- **Integration tests**: `tests/{name}_test.rs`

### Kiro Specifications
- **Spec directories**: `YYYY-MM-DD-{kebab-case-name}/`
- **Template files**: `{name}.md` within spec directory
- **Metadata**: `spec.json` for tracking

### Configuration Files
- **TOML configs**: `{name}.toml` (e.g., `config.toml`)
- **Shell scripts**: `{kebab-case}.sh`
- **Markdown docs**: `{kebab-case}.md`

## Import Organization

### Standard Import Order
```rust
// 1. Standard library
use std::fs;
use std::path::PathBuf;

// 2. External crates
use anyhow::Result;
use clap::Parser;

// 3. Internal modules
use crate::domain::entities::Project;
use crate::application::use_cases;

// 4. Super/self imports
use super::Repository;
use self::helpers::*;
```

### Module Organization
- Public API at module root
- Private implementation in submodules
- Re-export pattern for clean interfaces

## Key Architectural Principles

### Separation of Concerns
- Each layer has single responsibility
- Dependencies point inward (clean architecture)
- Business logic isolated from infrastructure

### Dependency Inversion
- Depend on abstractions (traits), not concretions
- Infrastructure implements domain interfaces
- Testability through dependency injection

### RAII Pattern
- Resource management through ownership
- Automatic cleanup with Drop trait
- TestDirectory pattern for test isolation

### Error Propagation
- `anyhow::Result` for application errors
- `thiserror` for domain-specific errors
- Context preservation through error chain

### Template Management
- Embedded resources for distribution
- Structured templates for specifications
- Version-controlled steering files