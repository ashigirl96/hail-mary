# Bug Report - Unnecessary Files Auto-Generated in Spec Creation

## Metadata
- **Severity**: Medium
- **Completeness**: 100%
- **Source**: user-input
- **References**:
  - `crates/hail-mary/src/infrastructure/repositories/spec.rs` (primary implementation)
  - `crates/hail-mary/src/cli/commands/new.rs` (command implementation and tests)
  - `crates/hail-mary/src/cli/commands/code.rs` (TUI integration)
  - `crates/hail-mary/src/application/use_cases/launch_claude_with_spec.rs` (spec creation flow)

## 1. Problem
- **What's broken**: When executing `hail-mary code` or `hail-mary new XXX`, the system automatically creates multiple files in `.kiro/specs` directory that are not immediately necessary
- **How to reproduce**:
  1. Run `hail-mary code` or `hail-mary new <feature-name>`
  2. Check the created spec directory under `.kiro/specs/`
  3. Observe that requirements.md, spec.json, and tasks.md are automatically created
- **Current behavior**: The following files are auto-generated:
  - requirements.md (not needed initially)
  - spec.json (not needed initially)
  - tasks.md (not needed initially)
  - design.md (should be kept)
  - investigation.md (should be kept)
  - memo.md (should be kept)

## 2. Expected
- **Should do**:
  - Only create minimal files necessary for initial development:
    - design.md
    - investigation.md
    - memo.md
  - Do NOT auto-create:
    - requirements.md
    - spec.json
    - tasks.md
  - These files should be created on-demand via slash commands when actually needed
- **Success criteria**:
  - Running `hail-mary code` or `hail-mary new` creates only design.md, investigation.md, and memo.md
  - requirements.md, spec.json, and tasks.md can be created later via slash commands
  - Existing slash commands continue to work for creating these files when needed

## 3. Technical Details
- **Root cause**: The `create_template_files()` method in `spec.rs` (lines 30-156) unconditionally creates all 6 specification files
- **Affected files and functions**:
  - `crates/hail-mary/src/infrastructure/repositories/spec.rs::create_template_files()`
  - Called by `SpecRepository::create_feature()` (line 186)
  - Triggered by both `hail-mary new` and `hail-mary code` (via TUI "Create New")
- **Implementation approach**:
  - Modify `create_template_files()` to only create essential files (design.md, investigation.md, memo.md)
  - Remove automatic creation of requirements.md, spec.json, and tasks.md
  - These files will be created on-demand by slash commands using Write tool
- **Impact on existing workflows**:
  - Slash commands already handle missing files gracefully (Write tool creates as needed)
  - No impact on existing specs (they already have all files)
  - Cleaner initial spec structure for new features
- **Backward compatibility**:
  - Fully backward compatible - existing specs remain unchanged
  - Slash commands continue to work (they create files if missing)
  - No migration needed for existing projects

---

## Completeness Scoring Rule
- **0-70%**: Problem documentation
  - Symptoms, reproduction steps, expected behavior
  - Maximum achievable through user reporting
- **70-100%**: Root cause identification
  - Root cause, affected components, technical context
  - Requires codebase investigation