# Tasks

## Overview
Complete removal of Memory MCP components from the hail-mary codebase, as the project has migrated to a file-based steering system for context management.

## References
- Investigation report: `.kiro/specs/2025-09-07-no-memory-mcp/investigation.md`
- Files to remove: ~20 files
- Files to update: ~10 files  
- Dependencies to remove: 4 (rmcp, rusqlite, refinery, schemars)

## Phase 1: Remove CLI Integration ✅
Removes command-line interface for memory commands while maintaining compilation.

- [x] Update `src/main.rs` to remove MemoryCommand handling
- [x] Update `src/cli/args.rs` to remove MemoryCommands enum
- [x] Delete `src/cli/commands/memory.rs` file
- [x] Update main.rs tests to remove memory command tests
- [x] Update CLI integration tests that import memory components
- [x] Update `src/cli/commands/mod.rs` to remove memory module export
- [x] Update `src/cli/mod.rs` to remove MemoryCommand and MemoryCommands exports

## Phase 2: Remove Application Layer
Removes business logic and use cases related to memory management.

- [ ] Delete `src/application/use_cases/remember_memory.rs`
- [ ] Delete `src/application/use_cases/recall_memory.rs`
- [ ] Delete `src/application/use_cases/generate_document.rs`
- [ ] Delete `src/application/use_cases/reindex_memories.rs`
- [ ] Remove `src/application/repositories/memory_repository.rs` trait
- [ ] Delete `src/application/test_helpers/mock_memory_repository.rs`
- [ ] Update `src/application/errors.rs` to remove memory-related error variants
- [ ] Update `src/application/mod.rs` to remove memory-related exports

## Phase 3: Remove Infrastructure & Core Components
Removes MCP server, database, and domain entities.

### MCP Server Removal
- [ ] Delete `src/infrastructure/mcp/server.rs`
- [ ] Delete `src/infrastructure/mcp/mod.rs`
- [ ] Update `src/infrastructure/mod.rs` to remove MCP module export

### Database Components Removal
- [ ] Delete `src/infrastructure/repositories/memory.rs` (SQLite repository)
- [ ] Delete `src/infrastructure/migrations/embedded.rs`
- [ ] Delete `src/infrastructure/migrations/mod.rs`
- [ ] Delete `migrations/V001__initial_schema.sql`
- [ ] Delete `migrations/V002__add_fts5_index.sql`
- [ ] Delete `migrations/V003__add_triggers.sql`

### Domain Entities Removal
- [ ] Delete `src/domain/entities/memory.rs`
- [ ] Delete `src/domain/value_objects/confidence.rs`
- [ ] Update `src/domain/entities/mod.rs` to remove Memory export
- [ ] Update `src/domain/value_objects/mod.rs` to remove Confidence export
- [ ] Update `src/domain/errors.rs` to remove memory-related domain errors

## Phase 4: Cleanup & Dependencies
Final cleanup of configuration, initialization, and dependencies.

### Configuration Updates
- [ ] Update `src/cli/commands/init.rs` to remove memory directory creation
- [ ] Update `src/application/use_cases/initialize_project.rs` to remove memory initialization
- [ ] Update `src/infrastructure/repositories/project.rs` to remove memory path management
- [ ] Update config.toml template to remove `[memory]` section
- [ ] Update ProjectConfig to remove memory_types field

### Dependency Removal
- [ ] Remove `rmcp` from `Cargo.toml` dependencies
- [ ] Remove `rusqlite` from `Cargo.toml` dependencies
- [ ] Remove `refinery` from `Cargo.toml` dependencies
- [ ] Remove `schemars` from `Cargo.toml` dependencies (if not used elsewhere)

### Test Cleanup
- [ ] Remove memory repository integration tests
- [ ] Remove migration tests
- [ ] Remove memory use case tests
- [ ] Update any remaining tests that reference memory components

## Phase 5: Verification
Ensures the removal is complete and the project still functions correctly.

- [ ] Run `just fix` to ensure format
- [ ] Run `just ci` to check for test
- [ ] Run `cargo build` to verify compilation
- [ ] Test remaining commands: `init`, `new`, `code`, `complete`
- [ ] Verify steering system still works correctly
- [ ] Update README.md if it references memory commands
