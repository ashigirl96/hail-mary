# Design

## Architecture

### Overview
This feature embeds .claude/commands/hm markdown files into the hail-mary binary at build time and deploys them during `hail-mary init`. This ensures all projects have consistent, up-to-date slash command documentation.

### Components

#### 1. Embedded Resources Module (`infrastructure/embedded_resources.rs`)
- Uses `include_str!` macro to embed markdown files at compile time
- Provides static references to embedded content
- No runtime file dependencies

#### 2. Repository Pattern Extension
- New method in `ProjectRepository` trait: `deploy_slash_commands()`
- Maintains clean abstraction for testing
- Implementation handles file system operations

#### 3. Initialize Project Use Case
- Integrates slash command deployment into existing init workflow
- Ensures commands are deployed after basic project structure

## Data Flow

1. **Build Time**: Markdown files are embedded into binary using `include_str!`
2. **Runtime - Init Command**: 
   - User executes `hail-mary init`
   - Initialize project structure (existing)
   - Deploy embedded slash commands (new)
   - Commands written to `.claude/commands/hm/`

## Key Design Decisions

### Force Overwrite Behavior
Unlike other init operations which are idempotent and preserve existing files, the .claude/commands/hm directory is always overwritten. This ensures:
- Projects always have the latest command definitions
- No version inconsistencies between projects
- Simplified upgrade path when commands are updated

### Embedded Resources Approach
Using `include_str!` instead of runtime file loading because:
- Single binary distribution with no external dependencies
- Guaranteed availability of command files
- Compile-time validation of file existence
- Small file sizes make embedding practical

### Clean Architecture Compliance
- Follows existing repository pattern
- Maintains separation of concerns
- Testable through mock repositories
- No domain layer changes required
