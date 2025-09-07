# Design: File-Based Steering System
## Reference

- @reference/slash-commands.md 

## Architecture Overview

### Responsibility Division

#### hail-mary CLI (Implementation Scope)
- **Single Responsibility**: Initialize steering system via `hail-mary init` command only
- Create directories, generate default files, update config.toml
- No draft management (YAGNI principle)

#### Claude Code (Slash Commands)
- **Primary Responsibility**: Complete draft file lifecycle management
- `/hm:steering-remember`: Create drafts for new learnings
- `/hm:steering`: Read drafts, categorize, append to steering files, delete processed drafts

### System Architecture
```
┌─────────────────────────────────────────────────────┐
│                  Claude Code                        │
│  ┌─────────────────────────────────────────────┐   │
│  │         Slash Commands                      │   │
│  │  /hm:steering-remember | /hm:steering       │   │
│  └────────────────┬────────────────────────────┘   │
│                   │                                 │
│                   ▼ Direct File I/O                 │
│  ┌─────────────────────────────────────────────┐   │
│  │     .kiro/steering/ and draft/ files        │   │
│  └─────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│              hail-mary CLI (init only)              │
│  ┌─────────────────────────────────────────────┐   │
│  │            init command                     │   │
│  └────────────────┬────────────────────────────┘   │
│                   │                                 │
│  ┌────────────────▼────────────────────────────┐   │
│  │         initialize_project                  │   │
│  └────────────────┬────────────────────────────┘   │
│                   │                                 │
│  ┌────────────────▼────────────────────────────┐   │
│  │  SteeringConfig (simple initialization)     │   │
│  └─────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
```

## Domain Model Design

### Core Entities

```rust
// Domain Layer: crates/hail-mary/src/domain/entities/steering.rs

#[derive(Debug, Clone, PartialEq)]
pub struct SteeringType {
    pub name: String,
    pub purpose: String,
    pub criterions: Vec<Criterion>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Criterion {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SteeringConfig {
    pub types: Vec<SteeringType>,
}

impl SteeringConfig {
    pub fn default_for_new_project() -> Self {
        Self {
            types: vec![
                SteeringType {
                    name: "product".to_string(),
                    purpose: "Product overview and value proposition".to_string(),
                    criterions: vec![
                        Criterion {
                            name: "Product Overview".to_string(),
                            description: "Brief description of what the product is".to_string(),
                        },
                        Criterion {
                            name: "Core Features".to_string(),
                            description: "Bulleted list of main capabilities".to_string(),
                        },
                        Criterion {
                            name: "Target Use Case".to_string(),
                            description: "Specific scenarios the product addresses".to_string(),
                        },
                        Criterion {
                            name: "Key Value Proposition".to_string(),
                            description: "Unique benefits and differentiators".to_string(),
                        },
                    ],
                },
                SteeringType {
                    name: "tech".to_string(),
                    purpose: "Technical stack and development environment".to_string(),
                    criterions: vec![
                        Criterion {
                            name: "Architecture".to_string(),
                            description: "High-level system design".to_string(),
                        },
                        Criterion {
                            name: "Frontend".to_string(),
                            description: "Frameworks, libraries, build tools (if applicable)".to_string(),
                        },
                        Criterion {
                            name: "Backend".to_string(),
                            description: "Language, framework, server technology (if applicable)".to_string(),
                        },
                        Criterion {
                            name: "Development Environment".to_string(),
                            description: "Required tools and setup".to_string(),
                        },
                        Criterion {
                            name: "Common Commands".to_string(),
                            description: "Frequently used development commands".to_string(),
                        },
                        Criterion {
                            name: "Environment Variables".to_string(),
                            description: "Key configuration variables".to_string(),
                        },
                        Criterion {
                            name: "Port Configuration".to_string(),
                            description: "Standard ports used by services".to_string(),
                        },
                    ],
                },
                SteeringType {
                    name: "structure".to_string(),
                    purpose: "Code organization and project structure patterns".to_string(),
                    criterions: vec![
                        Criterion {
                            name: "Root Directory Organization".to_string(),
                            description: "Top-level structure with descriptions".to_string(),
                        },
                        Criterion {
                            name: "Subdirectory Structures".to_string(),
                            description: "Detailed breakdown of key directories".to_string(),
                        },
                        Criterion {
                            name: "Code Organization Patterns".to_string(),
                            description: "How code is structured".to_string(),
                        },
                        Criterion {
                            name: "File Naming Conventions".to_string(),
                            description: "Standards for naming files and directories".to_string(),
                        },
                        Criterion {
                            name: "Import Organization".to_string(),
                            description: "How imports/dependencies are organized".to_string(),
                        },
                        Criterion {
                            name: "Key Architectural Principles".to_string(),
                            description: "Core design decisions and patterns".to_string(),
                        },
                    ],
                },
            ],
        }
    }
}
```

### Value Objects

```rust
// Draft management is handled by Claude Code slash commands,
// so no Draft entity is needed in hail-mary CLI (YAGNI principle)
```

## Repository Design

### Application Layer Interface (Simplified)

```rust
// Application Layer: crates/hail-mary/src/application/repositories/project_repository.rs
// Extending existing ProjectRepository

pub trait ProjectRepository {
    // Existing methods
    fn initialize(&self) -> Result<(), ApplicationError>;
    fn exists(&self) -> Result<bool, ApplicationError>;
    fn save_config(&self, config: &ProjectConfig) -> Result<(), ApplicationError>;
    fn update_gitignore(&self) -> Result<(), ApplicationError>;
    
    // New methods for steering initialization
    fn initialize_steering(&self) -> Result<(), ApplicationError>;
    fn create_steering_files(&self, config: &SteeringConfig) -> Result<(), ApplicationError>;
}
```

### Infrastructure Implementation (Simplified)

```rust
// Infrastructure Layer: crates/hail-mary/src/infrastructure/repositories/project.rs
// Adding to existing ProjectRepository

impl ProjectRepository {
    fn steering_dir(&self) -> PathBuf {
        self.path_manager.kiro_dir(true).join("steering")
    }
    
    fn draft_dir(&self) -> PathBuf {
        self.steering_dir().join("draft")
    }
    
    fn initialize_steering(&self) -> Result<(), ApplicationError> {
        // Create directories only
        fs::create_dir_all(self.steering_dir())?;
        fs::create_dir_all(self.draft_dir())?;
        Ok(())
    }
    
    fn create_steering_files(&self, config: &SteeringConfig) -> Result<(), ApplicationError> {
        for steering_type in &config.types {
            let file_path = self.steering_dir().join(format!("{}.md", steering_type.name));
            
            // Never overwrite existing files
            if file_path.exists() {
                continue;
            }
            
            // Generate simple template
            let content = format!(
                "# {}\n\n{}\n\n## Criterions\n{}\n",
                steering_type.name,
                steering_type.purpose,
                steering_type.criterions.iter()
                    .map(|c| format!("- {}: {}", c.name, c.description))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
            fs::write(file_path, content)?;
        }
        Ok(())
    }
}
```

## Use Case Design

### Initialize Project with Steering (Simplified)

```rust
// Application Layer: crates/hail-mary/src/application/use_cases/initialize_project.rs

pub fn initialize_project(
    repository: &impl ProjectRepository,
    force: bool,
) -> Result<(), ApplicationError> {
    // Check if project exists
    if repository.exists()? && !force {
        return Err(ApplicationError::ProjectAlreadyExists);
    }
    
    // Initialize base directories
    repository.initialize()?;
    
    // Initialize steering directories
    repository.initialize_steering()?;
    
    // Create default configuration
    let config = SteeringConfig::default_for_new_project();
    
    // Never overwrite existing config.toml (even with --force)
    let config_path = repository.config_path();
    if !config_path.exists() {
        repository.save_config(&config)?;
    }
    
    // Create steering files (never overwrite existing)
    repository.create_steering_files(&config)?;
    
    // Update .gitignore
    repository.update_gitignore()?;
    
    Ok(())
}
```

## Slash Command Design

**Important**: These slash commands are implemented on the Claude Code side and handle all draft management responsibilities.
The hail-mary CLI only performs initialization and does not handle draft management (YAGNI principle).

### Command Organization
Commands will be organized under `.claude/commands/hm/` directory to namespace them clearly.
When listed in `/help`, they will show as "(project:hm)" to indicate they are project-specific hail-mary commands.

### /hm:steering-remember

```markdown
# .claude/commands/hm/steering-remember.md
---
description: Save new learnings not already in steering files as draft
allowed-tools: Read, Write, Bash(date:*), Glob
argument-hint: [title] [--verbose]
---

## Triggers
- User identifies new learning or pattern to remember
- Context needs to be preserved for future reference
- Information not already documented in existing steering files

## Usage
```
/hm:steering-remember [title] [--verbose]
```

## Behavioral Flow

1. **Validate**: Check if content represents new learning not in existing steering files
2. **Generate**: Create descriptive filename using timestamp: !`date +%Y%m%d-%H%M%S`-$1.md
3. **Save**: Write draft to @.kiro/steering/draft/ with structured content
4. **Confirm**: Provide user feedback on successful save or any errors

Key behaviors:
- Only save genuinely new learnings not already documented
- Use descriptive titles for easy identification
- Include context about why this learning is important
- Structure content for easy categorization later

## Examples

### Basic Usage
```
/hm:steering-remember "const-vs-function"
# Saves learning about const vs function preference to draft
```

### Verbose Mode
```
/hm:steering-remember "api-design-pattern" --verbose
# Saves with detailed output showing what was captured
```

### /hm:steering

```markdown
# .claude/commands/hm/steering.md
---
description: Process and categorize steering drafts into appropriate files
allowed-tools: Read, Write, MultiEdit, Glob, Bash(ls:*, rm:*, cp:*, mkdir:*)
argument-hint: [--verbose] [--dry-run]
---

## Triggers
- Drafts accumulated in @.kiro/steering/draft/ need processing
- User wants to organize and categorize saved learnings
- Periodic maintenance of draft directory

## Usage
```
/hm:steering [--verbose] [--dry-run]
```

## Behavioral Flow

1. **Discover**: Find all draft files in @.kiro/steering/draft/ using Glob
   - Draft count: !`ls .kiro/steering/draft/*.md 2>/dev/null | wc -l`
   
2. **Backup**: Create @.kiro/steering/backup/ and copy existing steering files
   - !`mkdir -p .kiro/steering/backup && cp .kiro/steering/*.md .kiro/steering/backup/ 2>/dev/null`
   
3. **Categorize**: For each draft:
   - Read content and analyze against criterions in @.kiro/config.toml
   - Match draft content to most appropriate steering type
   - Use $1 for verbose output of categorization logic
   
4. **Append**: Use MultiEdit to add categorized content to steering files
   - Preserve existing content
   - Add new content with appropriate formatting
   - Use $2 for dry-run mode (preview without changes)
   
5. **Clean**: Remove successfully processed drafts
   - Delete draft files after successful categorization
   - Keep failed drafts with error log

Key behaviors:
- Never overwrite existing steering content
- Always backup before modifications  
- Clear feedback on categorization decisions
- Graceful handling of categorization failures

## Examples

### Standard Processing
```
/hm:steering
# Processes all drafts, categorizes, and updates steering files
```

### Verbose Mode
```
/hm:steering --verbose
# Shows detailed categorization logic for each draft
```

### Dry Run
```
/hm:steering --dry-run
# Preview categorization without making changes
```

### Combined Options
```
/hm:steering --verbose --dry-run
# Detailed preview of what would happen
```

## File Structure

```
.kiro/
├── config.toml                 # [steering] instead of [memory]
├── steering/
│   ├── product.md              # Product overview and value proposition
│   ├── tech.md                 # Technical stack and development environment
│   ├── structure.md            # Code organization patterns
│   ├── backup/                 # Backups of steering files before modification
│   └── draft/
│       ├── 20250907-143022-const-function.md
│       └── 20250907-145511-api-design.md
├── specs/
└── archive/

.claude/
└── commands/
    └── hm/
        ├── steering-remember.md
        └── steering.md
```

## Configuration Format

```toml
# .kiro/config.toml

[steering]
# Steering types are defined as an array of tables
[[steering.types]]
name = "product"
purpose = "Product overview and value proposition"
criterions = [
    "Product Overview: Brief description of what the product is",
    "Core Features: Bulleted list of main capabilities",
    "Target Use Case: Specific scenarios the product addresses",
    "Key Value Proposition: Unique benefits and differentiators"
]

[[steering.types]]
name = "tech"
purpose = "Technical stack and development environment"
criterions = [
    "Architecture: High-level system design",
    "Frontend: Frameworks, libraries, build tools (if applicable)",
    "Backend: Language, framework, server technology (if applicable)",
    "Development Environment: Required tools and setup",
    "Common Commands: Frequently used development commands",
    "Environment Variables: Key configuration variables",
    "Port Configuration: Standard ports used by services"
]

[[steering.types]]
name = "structure"
purpose = "Code organization and project structure patterns"
criterions = [
    "Root Directory Organization: Top-level structure with descriptions",
    "Subdirectory Structures: Detailed breakdown of key directories",
    "Code Organization Patterns: How code is structured",
    "File Naming Conventions: Standards for naming files and directories",
    "Import Organization: How imports/dependencies are organized",
    "Key Architectural Principles: Core design decisions and patterns"
]
```

## Implementation Phases

### Phase 1: hail-mary CLI Implementation (Current Scope)
1. Add SteeringConfig, SteeringType, Criterion entities to Domain layer
2. Add steering initialization methods to existing ProjectRepository
3. Update initialize_project use case
4. Change config.toml structure to [steering] section
5. Generate default steering file templates
6. Protect existing files (never overwrite even with --force)

### Phase 2: Claude Code Slash Commands Implementation (Separate Task)
1. Create `.claude/commands/hm/` directory
2. Create `/hm:steering-remember` command file
3. Create `/hm:steering` command file
4. Implement categorization algorithm logic (Claude Code side)

### Not Implemented (YAGNI)
- Migration tool from Memory MCP
- Draft management commands in hail-mary CLI
- Database-related code
- Complex categorization algorithm (implemented in Claude Code)

## Testing Strategy

### Unit Tests
- Domain entity validation
- Categorization algorithm accuracy
- File protection logic
- Draft sanitization

### Integration Tests
- Full initialization flow
- Draft to steering pipeline
- Config persistence
- File system operations

### End-to-End Tests
- Slash command execution
- Complete workflow from remember to categorize
- Migration from SQLite to files

## Error Handling

### Graceful Failures
- Missing draft directory → Create automatically
- Invalid draft title → Sanitize to kebab-case
- Categorization failure → Keep in draft with error log
- File write failure → Retry with backoff

### User Feedback
- Clear error messages for common issues
- Verbose mode for debugging
- Success confirmations for operations
- Progress indicators for batch operations

## Security Considerations

- No executable files in steering directory
- Validate all file paths to prevent directory traversal
- Sanitize draft titles to prevent injection
- Read-only access for categorization process

## Performance Optimizations

- Lazy loading of steering files
- Batch processing for multiple drafts
- Caching of configuration and criterions
- Parallel categorization when possible
