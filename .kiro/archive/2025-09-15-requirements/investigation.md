# Investigation

## Research for Unnecessary File Auto-Generation

## Current State Analysis

### File Creation Flow
1. **`hail-mary new <name>`** command flow:
   - `new.rs` → `create_feature()` use case → `SpecRepository::create_feature()` → `create_template_files()`

2. **`hail-mary code`** command flow (when creating new spec):
   - `code.rs` → `launch_claude_with_spec()` → TUI selection "Create New" → `spec_repo.create_feature()` → `create_template_files()`

### Current Implementation
- **Location**: `crates/hail-mary/src/infrastructure/repositories/spec.rs`
- **Method**: `create_template_files()` (lines 30-156)
- **Behavior**: Hardcoded to always create all 6 files:
  - requirements.md (lines 131-133)
  - design.md (lines 135-137)
  - tasks.md (lines 139-141)
  - memo.md (lines 143-145)
  - investigation.md (lines 147-149)
  - spec.json (lines 151-153)

## Problem Space

### Core Issue
The `create_template_files()` method unconditionally creates all specification files, regardless of actual need. This creates unnecessary clutter and goes against the principle of on-demand file creation.

### Impact Analysis
- **User Experience**: Cluttered spec directories with empty template files
- **Workflow**: Users must navigate around unnecessary files
- **Philosophy**: Violates the "create only what's needed" principle

## Existing Solutions

### Option 1: Minimal File Creation (Recommended)
- **Implementation**: Modify `create_template_files()` to only create essential files
- **Essential Files**: design.md, investigation.md, memo.md
- **On-Demand Files**: requirements.md, spec.json, tasks.md (via slash commands)
- **Pros**:
  - Cleaner initial spec structure
  - Aligns with on-demand philosophy
  - Backward compatible (slash commands can create missing files)
- **Cons**:
  - Requires modifying existing tests
  - Slash commands must handle file creation

### Option 2: Configuration-Based Control
- **Implementation**: Add configuration option to control which files are created
- **Config Location**: `.kiro/config.toml` with `[spec.files]` section
- **Pros**:
  - User customizable
  - Flexible for different workflows
  - No breaking changes for existing users
- **Cons**:
  - More complex implementation
  - Requires configuration management

### Option 3: Command Flags
- **Implementation**: Add flags like `--minimal` to commands
- **Usage**: `hail-mary new my-feature --minimal`
- **Pros**:
  - User control per invocation
  - No config file changes needed
- **Cons**:
  - Requires remembering to use flag
  - Inconsistent behavior between invocations

## Technical Research

### Affected Components
1. **Primary Changes Required**:
   - `spec.rs::create_template_files()`: Modify to create only essential files
   - Remove creation of: requirements.md, spec.json, tasks.md

2. **Test Updates Required**:
   - `new.rs` tests (lines 100-104): Update assertions
   - Remove assertions for requirements.md, tasks.md, spec.json
   - Keep assertions for design.md, investigation.md, memo.md

3. **Slash Command Considerations**:
   - `/hm:requirements`: Must create requirements.md if it doesn't exist
   - `/hm:tasks`: Must create tasks.md if it doesn't exist
   - Slash commands already use Write tool which creates files as needed

### Implementation Details
```rust
// Modified create_template_files() - only essential files
fn create_template_files(&self, feature_dir: &Path, name: &str) -> Result<()> {
    // Create only essential files
    let design_content = format!("# Design\n\n## Overview\n[High-level architecture for {}]\n", name);
    let investigation_content = format!("# Investigation\n\n## Research for {}\n...", name);
    let memo_content = format!("# Memo: {}\n\n", name);

    fs::write(feature_dir.join("design.md"), design_content)?;
    fs::write(feature_dir.join("investigation.md"), investigation_content)?;
    fs::write(feature_dir.join("memo.md"), memo_content)?;

    // Don't create: requirements.md, tasks.md, spec.json
    Ok(())
}
```

## Questions & Uncertainties

- [x] Which files are truly essential? → design.md, investigation.md, memo.md
- [x] How do slash commands handle missing files? → Write tool creates them automatically
- [ ] Should spec.json be created on-demand or always? → On-demand (it's metadata)
- [ ] Should we provide a migration path for existing projects? → Not needed, backward compatible

## Resources & References

- Source Code: `crates/hail-mary/src/infrastructure/repositories/spec.rs`
- Test Files: `crates/hail-mary/src/cli/commands/new.rs` (tests)
- Slash Commands: `.claude/commands/hm/requirements.md`, `tasks.md`
- Design Philosophy: Keep initial setup minimal, create on-demand