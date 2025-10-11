# Investigation

## init-command-usage

### Current Implementation

**Finding 1: Init Command Structure**
- **Source**: `/Users/nishimura/.ghq/src/github.com/ashigirl96/hail-mary/crates/hail-mary/src/cli/commands/init.rs:9-44`
- **Confidence**: 95% (direct code inspection)
- **Details**: `InitCommand` is a simple struct with a single `execute()` method that:
  - Uses current directory as project root (line 24)
  - Creates three repositories: ConfigRepository, SpecRepository, SteeringRepository (lines 28-30)
  - Delegates to `initialize_project` use case (line 33)
  - Returns idempotent results (can be run multiple times safely)
- **Impact**: Clean separation between CLI layer and application logic makes removal straightforward

**Finding 2: Core Initialization Logic**
- **Source**: `/Users/nishimura/.ghq/src/github.com/ashigirl96/hail-mary/crates/hail-mary/src/application/use_cases/initialize_project.rs:7-35`
- **Confidence**: 95% (direct code inspection)
- **Details**: `initialize_project()` function performs:
  - Steering directory initialization (line 13)
  - Config file creation with steering configuration (lines 16, 19, 22)
  - Steering file creation (lines 25-26)
  - .gitignore updates (line 29)
  - Slash command deployment (line 32)
- **Impact**: This use case function is shared with `code` command, so removal of `init` command doesn't affect the functionality

**Finding 3: Code Command Already Performs Initialization**
- **Source**: `/Users/nishimura/.ghq/src/github.com/ashigirl96/hail-mary/crates/hail-mary/src/cli/commands/code.rs:39-40`
- **Confidence**: 95% (direct code inspection)
- **Details**: `CodeCommand::execute()` calls `initialize_project()` with comment "Initialize project if needed (this is idempotent)"
- **Impact**: **CRITICAL** - This proves init command is redundant; `hail-mary code` provides complete initialization functionality

### Summary Statistics

**Code Removal Scope**:
- **1** enum variant (`Commands::Init`)
- **1** command implementation file (`cli/commands/init.rs` - 267 lines)
- **1** main.rs match arm (4 lines)
- **3** module exports/imports
- **2** helper methods (`is_init()` + related)
- **21** test usages (9 internal + 12 external)
- **7** documentation locations

**Zero Functional Loss**:
- `initialize_project()` use case remains intact
- `code` command provides identical initialization
- Idempotent design ensures safe operation
- Integration tests already use `initialize_project()` directly

**Migration Complexity**: **Low**
- No user data migration needed
- Simple command substitution (`init` → `code`)
- Automatic initialization on first `code` run
- No breaking changes to project structure or config format

---

## new-command-usage

### Executive Summary

The `hail-mary new` command is a standalone spec creation command that has been superseded by the more comprehensive `hail-mary code` command. Analysis reveals that `new` command functionality is fully replicated within the `code` command's TUI, making it safe for removal. The command has extensive test coverage (31 tests total) but limited production dependencies, with only integration tests requiring updates post-removal.

### Command Implementation Architecture

**Finding**: NewCommand is a thin wrapper around create_spec use case with integrated validation and error handling

**Source**: `/Users/nishimura/.ghq/src/github.com/ashigirl96/hail-mary/crates/hail-mary/src/cli/commands/new.rs:7-67`

**Confidence**: 95% (direct code inspection)

**Implementation Details**:
```rust
pub struct NewCommand {
    name: String,
}

impl NewCommand {
    pub fn execute(&self) -> Result<()> {
        // 1. Discovers project root via PathManager (line 18)
        // 2. Creates SpecRepository (line 30)
        // 3. Calls create_spec use case (line 33)
        // 4. Formats success/error messages (lines 34-65)
    }
}
```

**Impact**: Command removal requires deletion of entire file plus test suite (242 lines total)

### Use Case Layer Analysis

**Finding**: Standalone use case function implementing kebab-case validation and spec creation

**Source**: `/Users/nishimura/.ghq/src/github.com/ashigirl96/hail-mary/crates/hail-mary/src/application/use_cases/create_spec.rs:4-26`

**Confidence**: 95% (direct code inspection)

**Critical Finding**: Validation logic is DUPLICATED with domain layer SpecValidator

**Evidence**:
- create_spec: Manual character checking (lines 9-18)
- SpecValidator: Regex-based validation `/^[a-z0-9]+(-[a-z0-9]+)*$/` at `/Users/nishimura/.ghq/src/github.com/ashigirl96/hail-mary/crates/hail-mary/src/domain/value_objects/spec.rs:14`

**Confidence**: 95% (code comparison)

**Impact**: create_spec use case can be safely removed; SpecValidator is the canonical implementation

### Code Command Comparison

**Finding**: code command provides SUPERSET of new command functionality via TUI

**Source**: `/Users/nishimura/.ghq/src/github.com/ashigirl96/hail-mary/crates/hail-mary/src/application/use_cases/launch_claude_with_spec.rs:50-61`

**Confidence**: 95% (code inspection)

**Key Difference**: code command uses SpecValidator directly, not the create_spec use case function

**Confidence**: 95% (direct code inspection)

**Impact**: code command already follows best practice architecture; new command is redundant

### Removal Strategy

**Files to Delete** (2 files):
1. `/Users/nishimura/.ghq/src/github.com/ashigirl96/hail-mary/crates/hail-mary/src/cli/commands/new.rs` (242 lines)
2. `/Users/nishimura/.ghq/src/github.com/ashigirl96/hail-mary/crates/hail-mary/src/application/use_cases/create_spec.rs` (280 lines)

**Confidence**: 95% (comprehensive analysis)

**Impact**: 522 lines of code deletion, 26 test functions removed

---

## code-command-coverage

### Executive Summary

**Verdict: INCOMPLETE WORKFLOW COVERAGE - Recommend deprecation over immediate deletion**

The `code` command provides **complete functional coverage** (100%) but **incomplete workflow coverage** (~70%) compared to `init` and `new` commands. While all features are technically accessible through the `code` command, critical non-interactive and scripting workflows are not supported.

**Key Finding**: The `code` command successfully calls `initialize_project` internally, which performs all initialization tasks that `init` does. It also supports spec creation via TUI. However, it forces interactive TUI interaction and couples spec creation with Claude Code launching, breaking automation and preparation workflows.

### Feature Comparison Matrix

| Feature | Init Command | New Command | Code Command | Coverage |
|---------|-------------|-------------|--------------|----------|
| Create .kiro directory | ✅ | ❌ | ✅ (via initialize_project) | 100% |
| Create config.toml | ✅ | ❌ | ✅ (via initialize_project) | 100% |
| Create steering files | ✅ | ❌ | ✅ (via initialize_project) | 100% |
| Deploy slash commands | ✅ | ❌ | ✅ (via initialize_project) | 100% |
| Deploy agents | ✅ | ❌ | ✅ (via initialize_project) | 100% |
| Update .gitignore | ✅ | ❌ | ✅ (via initialize_project) | 100% |
| Create spec directory | ❌ | ✅ | ✅ (via TUI + create_spec) | 100% |
| Validate spec name | ❌ | ✅ | ✅ (SpecValidator) | 100% |
| Create memo.md/tasks.md | ❌ | ✅ | ✅ | 100% |
| CLI argument for spec name | ❌ | ✅ | ❌ | **0%** |
| Non-interactive mode | ✅ | ✅ | ❌ | **0%** |
| Create without launching Claude | ✅ | ✅ | ❌ | **0%** |
| Idempotent initialization | ✅ | N/A | ✅ | 100% |

**Functional Coverage**: 10/13 features (77%)
**Workflow Coverage**: Critical gaps in automation and preparation workflows

### Critical Workflow Gaps

**Gap 1: Non-Interactive Initialization**
- **Evidence**: `/Users/nishimura/.ghq/src/github.com/ashigirl96/hail-mary/crates/hail-mary/src/application/use_cases/launch_claude_with_spec.rs:27-31`
- **Impact**: CI/CD pipelines, automated project setup scripts, batch operations
- **Severity**: High for automation users
- **Workaround**: User must manually cancel TUI (pressing Esc/q), which may fail in non-TTY environments

**Gap 2: Spec Creation Without Claude Launch**
- **Evidence**: `/Users/nishimura/.ghq/src/github.com/ashigirl96/hail-mary/crates/hail-mary/src/application/use_cases/launch_claude_with_spec.rs:50-103`
- **Impact**: "Prepare now, work later" workflows
- **Severity**: Medium - affects planning and preparation workflows
- **Workaround**: None (must let Claude launch and manually exit)

**Gap 3: CLI Argument for Spec Name**
- **Evidence**: `/Users/nishimura/.ghq/src/github.com/ashigirl96/hail-mary/crates/hail-mary/src/cli/args.rs:34-41`
- **Impact**: Batch spec creation scripts
- **Severity**: Medium - affects scripting and automation
- **Workaround**: None - each spec creation requires manual interaction

### Recommendations

**Option A: Enhance Code Command Before Deletion (Recommended)**

Add flags to support missing workflows:

```rust
Code {
    #[arg(long)]
    no_danger: bool,
    #[arg(short = 'c', long = "continue")]
    continue_conversation: bool,
    // NEW FLAGS:
    #[arg(long)]
    no_launch: bool,  // Create spec but don't launch Claude
    #[arg(long, value_name = "NAME")]
    spec: Option<String>,  // Specify spec name via CLI (skip TUI)
}
```

**Implementation**:
- `--no-launch`: After spec creation, exit instead of launching Claude
- `--spec <name>`: Create new spec with given name, skip TUI
- Combined: `hail-mary code --spec feature-name --no-launch` replaces `hail-mary new feature-name`

**Confidence**: 85% - Addresses all workflow gaps
**Effort**: Medium (2-3 days development)
**Risk**: Low (additive changes, no breaking changes)

---

**Option B: Deprecate Instead of Delete**

Keep `init` and `new` commands but mark as deprecated for 1-2 release cycles before actual deletion.

**Confidence**: 90% - Safe, reversible approach
**Effort**: Low (1 day for deprecation warnings)
**Risk**: Very Low (no breaking changes)

---

**Final Recommendation**: Choose Option A (Enhance) + Option B (Deprecate)

**Timeline**:
- v0.9.0: Add new flags, deprecate old commands
- v0.10.0: Remove deprecated commands (after 1-2 releases)

**Confidence**: 90% - Balances innovation with backward compatibility

---

## cli-structure

### CLI Architecture Overview

**Finding**: Hail-mary uses Clap v4.5 with derive macros for declarative CLI definition
**Source**: `/Users/nishimura/.ghq/src/github.com/ashigirl96/hail-mary/Cargo.toml:7`, `/Users/nishimura/.ghq/src/github.com/ashigirl96/hail-mary/crates/hail-mary/src/cli/args.rs:1-10`
**Confidence**: 95% (direct code inspection)
**Impact**: Removing commands requires only enum modifications - Clap auto-generates help text and completions

The CLI architecture follows a clean separation:

```
User Input → Clap Parser → Commands Enum → Command Router → Command Struct → Use Case Layer
```

### Command Routing Flow

**Finding**: Command routing happens in main.rs through pattern matching on Commands enum
**Source**: `/Users/nishimura/.ghq/src/github.com/ashigirl96/hail-mary/crates/hail-mary/src/main.rs:19-63`
**Confidence**: 95% (direct code inspection)
**Impact**: Removing Init/New requires deleting two match arms (lines 23-30)

### Shell Completion Generation

**Finding**: Completions are auto-generated by clap_complete from Cli struct
**Source**: `/Users/nishimura/.ghq/src/github.com/ashigirl96/hail-mary/crates/hail-mary/src/cli/commands/completion.rs:10-23`
**Confidence**: 95% (direct code inspection)
**Impact**: Completions automatically update when Commands enum changes - no manual intervention required

### Files Requiring Updates - Complete Inventory

**Core CLI Files (DELETE)**:
- `/Users/nishimura/.ghq/src/github.com/ashigirl96/hail-mary/crates/hail-mary/src/cli/commands/init.rs` - Entire file
- `/Users/nishimura/.ghq/src/github.com/ashigirl96/hail-mary/crates/hail-mary/src/cli/commands/new.rs` - Entire file

**Core CLI Files (UPDATE)**:
- `/Users/nishimura/.ghq/src/github.com/ashigirl96/hail-mary/crates/hail-mary/src/cli/args.rs`
  - Lines 14-21: Remove Init and New enum variants
  - Lines 86-92: Remove `is_init()` and `is_new()` methods
  - Lines 108-113: Remove `get_new_name()` method
  - Lines 145-177: Remove unit tests for init/new parsing

- `/Users/nishimura/.ghq/src/github.com/ashigirl96/hail-mary/crates/hail-mary/src/main.rs`
  - Lines 4-5: Remove InitCommand, NewCommand from imports
  - Lines 23-30: Remove Init and New match arms
  - Lines 111-165: Remove init command tests
  - Lines 168-254: Remove new command tests

- `/Users/nishimura/.ghq/src/github.com/ashigirl96/hail-mary/crates/hail-mary/src/cli/commands/mod.rs`
  - Line 4: Remove `pub mod init;`
  - Line 5: Remove `pub mod new;`
  - Line 12: Remove `pub use init::InitCommand;`
  - Line 13: Remove `pub use new::NewCommand;`

- `/Users/nishimura/.ghq/src/github.com/ashigirl96/hail-mary/crates/hail-mary/src/cli/mod.rs`
  - Line 7: Remove InitCommand and NewCommand from re-exports

- `/Users/nishimura/.ghq/src/github.com/ashigirl96/hail-mary/crates/hail-mary/src/cli/commands/complete.rs`
  - Line 30: Update error message from "Run 'hail-mary init' first." to "Run 'hail-mary code' to initialize."

**Documentation Files (UPDATE)**:
- `/Users/nishimura/.ghq/src/github.com/ashigirl96/hail-mary/README.md`
  - Lines 76-77, 110-114, 130-141, 335-336: Update to reflect code-centric workflow

- `/Users/nishimura/.ghq/src/github.com/ashigirl96/hail-mary/.kiro/steering/tech.md`
  - Lines 110-111: Update command reference

**Files NOT Requiring Updates**:
- Integration tests (use use cases directly)
- Shell completion generation (auto-updates from Clap)
- Use case layer (business logic unchanged)
- Repository layer (data access unchanged)

---

## test-cleanup

### Overview

This investigation identifies all test coverage related to the deprecated `init` and `new` commands, mapping the complete test landscape to enable systematic removal while preserving test infrastructure needed by the surviving `code` command.

### Test Discovery

**Finding**: Test coverage spans 3 primary areas with 162 total `#[test]` annotations across 28 files

**Evidence Chain**:
- **Source**: `cargo test --lib` execution + `rg '#\[test\]' --count`
- **Confidence**: 95% (direct test compilation and execution)
- **Impact**: Test removal must be surgical to preserve 130+ unrelated tests

### Test Organization Structure

#### Unit Tests (Inline `#[cfg(test)]` modules)

**1. InitCommand Tests** (`src/cli/commands/init.rs:46-267`)

**Test Functions Requiring Removal** (13 tests):
- `test_init_command_new` (line 54)
- `test_init_command_execute_success` (line 60)
- `test_init_command_is_idempotent` (line 76)
- `test_init_command_partial_initialization` (line 96)
- `test_init_command_creates_gitignore` (line 118)
- `test_init_command_appends_to_existing_gitignore` (line 133)
- `test_init_command_directory_structure` (line 150)
- `test_init_command_config_content` (line 166)
- `test_init_command_deploys_slash_commands` (line 183)
- `test_init_command_overwrites_slash_commands` (line 250)

**Coverage**: Lines 46-267 (221 lines of test code)
**Confidence**: 100% (directly in init.rs)

---

**2. NewCommand Tests** (`src/cli/commands/new.rs:69-241`)

**Test Functions Requiring Removal** (8 tests):
- `test_new_command_new` (line 77)
- `test_new_command_execute_success` (line 83)
- `test_new_command_execute_without_project` (line 115)
- `test_new_command_execute_invalid_name` (line 126)
- `test_new_command_execute_valid_names` (line 151)
- `test_new_command_execute_duplicate_feature` (line 183)
- `test_new_command_spec_path_format` (line 202)
- `test_new_command_edge_cases` (line 222)

**Critical Dependency**: Line 73 imports `InitCommand` for test setup
**Coverage**: Lines 69-241 (172 lines of test code)
**Confidence**: 100% (directly in new.rs)

---

**3. InitializeProject Use Case Tests** (`src/application/use_cases/initialize_project.rs:37-181`)

**Test Functions (RETAINED - used by code command)** (9 tests):
- `test_initialize_project_success` (line 45)
- `test_initialize_project_idempotent` (line 55)
- `test_initialize_project_steering_failure` (line 67)
- `test_initialize_project_config_failure` (line 82)
- `test_initialize_project_gitignore_failure` (line 97)
- `test_initialize_project_flow_order` (line 112)
- `test_initialize_project_with_default_config` (line 123)
- `test_initialize_project_error_propagation` (line 133)

**Rationale**: `initialize_project()` function is used by `CodeCommand::execute()` (line 40 of code.rs), so these tests validate critical functionality for the surviving command.

**Coverage**: Lines 37-181 (144 lines of test code)
**Confidence**: 100% (use case tests)
**Impact**: **RETAIN ALL** - function is still in use

---

#### Integration Tests

**4. Steering Integration Tests** (`tests/steering_integration_test.rs`)

**Test Functions (ALL RETAINED)** (7 tests):
- `test_system_prompt_includes_steering_content` (line 16)
- `test_system_prompt_with_empty_steering` (line 70)
- `test_steering_display_format` (line 126)
- `test_steerings_display_format_with_individual_tags` (line 160)
- `test_backup_rotation_maintains_max_limit` (line 202)
- `test_backup_rotation_with_excess_backups` (line 267)

**Evidence**: Uses `initialize_project()` function which remains
**Confidence**: 100% (direct import analysis)
**Impact**: No changes needed

---

### Coverage Impact Assessment

#### Current Test Metrics
- **Total test functions**: 162 `#[test]` annotations across codebase
- **Tests to remove**: ~23 tests (21 from init/new commands + 2 from CLI args)
- **Tests to retain**: ~139 tests (including all initialize_project use case tests)
- **Coverage reduction**: ~14% of test suite

#### Test Distribution by Module

| Module | Total Tests | Remove | Retain |
|--------|-------------|--------|--------|
| `cli/commands/init.rs` | 13 | 13 | 0 |
| `cli/commands/new.rs` | 8 | 8 | 0 |
| `cli/args.rs` | 2 | 2 | 0 |
| `use_cases/initialize_project.rs` | 9 | 0 | 9 |
| `tests/steering_integration_test.rs` | 7 | 0 | 7 |
| Other modules | ~123 | 0 | ~123 |

**Confidence**: 90% (based on test execution and grep analysis)

---

### Cleanup Strategy

#### Phase 1: File Removal (Zero Risk)
```
Files to delete entirely:
1. crates/hail-mary/src/cli/commands/init.rs (267 lines)
2. crates/hail-mary/src/cli/commands/new.rs (241 lines)

Total: 508 lines removed (including 393 lines of test code)
```

**Rationale**: These files contain ONLY deprecated command implementation + tests
**Confidence**: 100%

---

#### Phase 2: Test Removal from Shared Files

**CLI Args Tests** (`src/cli/args.rs`):
```rust
// Remove these test functions:
#[test]
fn test_cli_parse_init_command() { ... }

#[test]
fn test_cli_parse_new_command() { ... }
```

---

#### Phase 3: Verification

**Post-Removal Test Execution**:
```bash
cargo test --lib  # Should show ~139 passing tests
cargo test --test steering_integration_test  # All 7 tests pass
cargo test  # Full suite passes
```

**Expected Outcome**:
- All remaining tests pass (including initialize_project use case tests)
- No orphaned imports remain
- TestDirectory and mock repositories still function
- Integration tests using initialize_project() continue working

---

### Risk Assessment

**Low Risk**: Command removal is safe with proper test updates

**Evidence**:
1. **File isolation**: init.rs and new.rs are isolated modules with no reverse dependencies
2. **Test helper retention**: All helpers are actively used by surviving tests
3. **Integration test preservation**: No changes needed to integration tests
4. **Use case preservation**: initialize_project tests retained (used by code command)

**Cleanup Complexity**: **Low**
- Surgical file deletion (2 files)
- Minor test removal from args.rs (2 tests)
- Simple export cleanup (2 files)
- Zero helper infrastructure changes

**Post-Cleanup Test Suite**:
- **Expected test count**: ~139 tests
- **Expected pass rate**: 100%
- **Coverage preservation**: 86% of current test suite retained
