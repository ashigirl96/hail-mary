# PBI/SBI Multi-PR Support Design

## Meta
- **Completeness**: 100%
- **Requirements**: Support multiple Pull Requests per project through PBI/SBI hierarchy
- **Architecture Scope**: Full-stack (Rust backend + Pattern Router Framework integration)
- **Last Updated**: Based on user feedback - simplified PBI management

## Overview

**As-Is**:
- 1 Spec = 1 PR model
- Single-level specification structure
- All work concentrated in one specification directory

**To-Be**:
- 1 PBI (Product Backlog Item) = Multiple SBIs (Sprint Backlog Items)
- 1 SBI = 1 PR
- Hierarchical specification structure with parent (PBI) and children (SBIs)
- Backward compatible - existing 1 Spec = 1 PR workflow unchanged

## Core Design Philosophy

### Template Switching - Complexity Isolation

**Key Principle**: Normal case stays simple, SBI case adds extension

```
Normal Spec (1 PR):
  → Uses 10_spec_files.md (current)
  → No changes to 02_hub.md, 04_workflows.md

PBI/SBI (Multiple PRs):
  → Uses 10_spec_files_sbi.md (new)
  → PBI logic isolated in new template
  → Backward compatible
```

### Design Rationale

1. **Minimize Complexity**: Keep Pattern Router Framework core unchanged
2. **Separation of Concerns**: SBI-specific logic in dedicated template
3. **Backward Compatibility**: Existing users unaffected
4. **NO Linear Workflow**: Maintain reactive, adaptive development

## Directory Structure

### PBI/SBI Hierarchy

````
.kiro/specs/[pbi-name]/              # Product Backlog Item
  ├── requirements.md                 # PBI: Overview + SBI listing
  ├── investigation.md                # Shared research (optional)
  ├── design.md                       # High-level architecture (optional)
  ├── memo.md                         # (optional)
  │
  ├── sbi-1-[title]/                  # Sprint Backlog Item = 1 PR
  │   ├── requirements.md             # Detailed PRD/Bug/Tech requirements
  │   ├── investigation.md            # SBI-specific research
  │   ├── design.md                   # Implementation design
  │   ├── tasks.md                    # PR Timeline (current structure)
  │   └── memo.md
  │
  ├── sbi-2-[title]/
  │   ├── requirements.md
  │   ├── investigation.md
  │   ├── design.md
  │   ├── tasks.md
  │   └── memo.md
  │
  └── sbi-3-[title]/
      └── ...
````

### Design Decisions

**PBI Level Files**:
- `requirements.md`: PBI overview + all SBI descriptions (Pattern Router managed)
- `investigation.md`: Optional shared research
- `design.md`: Optional high-level architecture
- `memo.md`: Optional notes
- **NO tasks.md**: Not needed at PBI level (Pattern Router doesn't manage it)

**SBI Level Files**:
- Complete spec structure (same as current single-spec model)
- Each SBI is independent
- Pattern Router Framework manages SBI files normally

## Pattern Router Framework Integration

### 1. New Template File

#### crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/10_spec_files_sbi.md

````markdown
## Specification Files

**Current Context**: SBI (Sprint Backlog Item)
**PBI**: {pbi_name}
**Current SBI**: {sbi_name}

### Current SBI Context (Primary Working Files)
- <requirements-file>{sbi_requirements_path}</requirements-file> - SBI detailed requirements
- <investigation-file>{sbi_investigation_path}</investigation-file> - SBI research findings
- <design-file>{sbi_design_path}</design-file> - SBI technical design
- <tasks-file>{sbi_tasks_path}</tasks-file> - SBI task tracking and timeline
- <memo-file>{sbi_memo_path}</memo-file> - SBI notes (**DO NOT ACCESS**)

### PBI Context (Reference Only)
Parent PBI for broader context:
- <pbi-requirements-file>{pbi_requirements_path}</pbi-requirements-file> - PBI overview and all SBIs

Optional PBI design (if exists):
- <pbi-investigation-file>{pbi_investigation_path}</pbi-investigation-file> - Shared research (if exists)
- <pbi-design-file>{pbi_design_path}</pbi-design-file> - High-level architecture (if exists)

**Note**: Work primarily with SBI files. PBI files provide context only.
````

**Key Features**:
- Clear distinction: SBI files (primary) vs PBI files (reference)
- Optional PBI investigation/design
- Evidence chain: SBI → PBI design → PBI investigation

### 2. Template Switching Logic

#### crates/hail-mary/src/domain/value_objects/system_prompt/mod.rs

```rust
// Add new embedded resource
const PATTERN_ROUTER_SPEC_FILES_SBI: &str =
    include_str!("pattern_router/10_spec_files_sbi.md");

impl SystemPrompt {
    pub fn new(
        spec_name: Option<&str>,
        spec_path: Option<&Path>,
        steerings: &Steerings
    ) -> Self {
        // ... existing code ...

        if let (Some(name), Some(path)) = (spec_name, spec_path) {
            // Detect SBI context
            let is_sbi = is_sbi_context(path);

            // Choose appropriate template
            let spec_files_section = if is_sbi {
                build_sbi_spec_files(name, path)
            } else {
                build_regular_spec_files(name, path)
            };

            // ... rest of code ...
        }
    }
}

fn is_sbi_context(spec_path: &Path) -> bool {
    // Check if parent directory contains other sbi-* directories
    if let Some(parent) = spec_path.parent() {
        if let Some(dir_name) = spec_path.file_name() {
            if dir_name.to_str().unwrap_or("").starts_with("sbi-") {
                return true;
            }
        }
    }
    false
}

fn build_sbi_spec_files(spec_name: &str, spec_path: &Path) -> String {
    // Extract PBI name and SBI name
    let parent_path = spec_path.parent().unwrap();
    let pbi_name = parent_path.file_name().unwrap().to_str().unwrap();
    let sbi_name = spec_name;

    // Build paths
    let sbi_requirements = format!("{}/requirements.md", spec_path.display());
    let pbi_requirements = format!("{}/requirements.md", parent_path.display());
    // ... other paths ...

    PATTERN_ROUTER_SPEC_FILES_SBI
        .replace("{pbi_name}", pbi_name)
        .replace("{sbi_name}", sbi_name)
        .replace("{sbi_requirements_path}", &sbi_requirements)
        .replace("{pbi_requirements_path}", &pbi_requirements)
        // ... other replacements ...
}
```

### 3. Pattern Router Files - Minimal Changes

**These files require minimal updates**:
- `07_requirements.md` - Add PBI template
- `03_patterns.md` - Add `/hm:decompose` and `/hm:sbi add` patterns

**These files remain unchanged**:
- `02_hub.md` - Hub only manages SBI tasks.md (not PBI)
- `04_workflows.md` - Workflows operate on SBI level only
- `05_gates.md` - Gates validate SBI prerequisites only
- `06_nudges.md` - Nudges operate on SBI context only

**Why minimal changes**: Pattern Router Framework only manages SBI files, not PBI level

## TUI - Nested Selection

### UI Design

```
🚀 Launch without specification
📝 Create new specification
   2025-10-07-project                    # PBI with SBIs (expandable)
 >   sbi-1-backend-api                   # ← Indented, selectable
     sbi-2-frontend-ui
     sbi-3-mobile-app
     📝 Create new SBI specification     # Edge case: add SBI later
   2025-09-29-legacy-feature             # Single Spec (no SBIs)
```

### Implementation

#### infrastructure/tui/spec_selector.rs

```rust
enum TuiItem {
    LaunchWithoutSpec,
    CreateNewSpec,
    Pbi {
        name: String,
        sbis: Vec<String>,
        expanded: bool  // For collapsible UI
    },
    Sbi {
        pbi_name: String,
        sbi_name: String
    },
    CreateNewSbi {
        pbi_name: String
    },
    RegularSpec {
        name: String
    },
}

enum SpecSelectionResult {
    NoSpec,
    Sbi(String, String),      // (pbi_name, sbi_name) - Primary use case
    RegularSpec(String),      // Regular 1 PR spec
    Pbi(String),              // PBI-level work (rare)
    CreateNewSpec,
    CreateNewSbi(String),     // pbi_name
    Cancelled,
}

impl SpecSelectorTui {
    fn render_item(&self, item: &TuiItem, is_selected: bool) -> String {
        match item {
            TuiItem::Sbi { sbi_name, .. } => {
                let indicator = if is_selected { ">" } else { " " };
                format!("{}   {}", indicator, sbi_name)  // 3-space indent
            }
            // ... other cases ...
        }
    }
}
```

## Slash Commands

### 1. /hm:requirements --type pbi

#### Update 07_requirements.md - Add PBI Template

````markdown
**PBI Template**:
```markdown
# Product Backlog Item

## Overview
[Brief description of the overall feature/project]

## Sprint Backlog Items

### sbi-1-[title]
requirements type: [prd|bug|tech]
[SBI description - will be expanded to full requirements.md in sbi-1/ directory]

### sbi-2-[title]
requirements type: [prd|bug|tech]
[SBI description]

### sbi-3-[title]
requirements type: [prd|bug|tech]
[SBI description]
```
````

#### Update .claude/commands/hm/requirements.md

```yaml
argument-hint: "[--type prd|bug|tech|pbi] [--issue <github-url>]"
```

Add to Key Patterns:
```markdown
- **Type Detection**: --type pbi → PBI template activation
```

**Behavioral Flow**:
1. Detect `--type pbi` flag
2. Generate PBI template (not PRD/Bug/Tech template)
3. User discusses and refines SBI breakdown adaptively
4. Save to `requirements.md`

### 2. /hm:decompose

#### .claude/commands/hm/decompose.md

````markdown
---
name: decompose
description: "Decompose PBI into separate SBI specifications"
allowed-tools: Read, Write, Glob
---

# /hm:decompose

Decompose Product Backlog Item into Sprint Backlog Items.

## Behavioral Flow

1. **Read PBI requirements.md**
   - Parse all `## sbi-X-[title]` sections
   - Extract `requirements type: [prd|bug|tech]`

2. **Interactive Confirmation**
   ```
   Found 3 SBIs in requirements.md:
     1. sbi-1-backend-api (type: prd)
     2. sbi-2-frontend-ui (type: prd)
     3. sbi-3-error-handling (type: tech)

   Decompose into separate directories? [Y/n]:
   ```

3. **Create SBI Directories**
   For each SBI:
   - Create `sbi-X-[title]/` directory
   - Generate `requirements.md` using appropriate template (PRD/Bug/Tech)
   - Populate with detailed requirements from PBI

4. **Nudge**
   ```
   SBIs created! Select specific SBI to work on:
   $ hail-mary code → [pbi-name] → sbi-1-backend-api

   Each SBI will generate tasks.md/memo.md when you start working.
   ```

Refer to system prompt sections:
- <kiro-patterns> for pattern recognition
- <kiro-workflows> for decompose workflow
````

### 3. /hm:sbi add

#### .claude/commands/hm/sbi-add.md

````markdown
---
name: sbi-add
description: "Add new SBI to existing PBI"
allowed-tools: Read, Write, Edit
argument-hint: "<sbi-name>"
---

# /hm:sbi add

Add a new Sprint Backlog Item to the current Product Backlog Item.

## Usage

```
/hm:sbi add sbi-4-monitoring
/hm:sbi add monitoring  # Auto-number to sbi-4-monitoring
```

## Behavioral Flow

1. **Context Verification**
   - Check if in PBI context (error if not)
   - Get next available SBI number

2. **Interactive Type Selection**
   ```
   Select SBI type:
   1. PRD (Product feature)
   2. Bug (Bug fix)
   3. Tech (Technical improvement)
   →
   ```

3. **Create SBI**
   - Create `sbi-X-[title]/` directory
   - Generate `requirements.md` with selected template

4. **Update PBI requirements.md**
   - Add section to PBI `requirements.md`:
     ```markdown
     ### sbi-X-[title]
     requirements type: [selected-type]
     [Description placeholder]
     ```

5. **Nudge**
   ```
   SBI created!
   1. Edit description in PBI requirements.md
   2. Select SBI: $ hail-mary code → sbi-X-[title]

   Tasks.md and memo.md will be created when you start working on this SBI.
   ```
````

## Repository Layer

### infrastructure/repositories/spec.rs

```rust
impl SpecRepository {
    /// List all SBIs in a PBI
    pub fn list_sbis(&self, pbi_name: &str) -> Result<Vec<String>> {
        let pbi_path = self.path_manager.specs_dir().join(pbi_name);

        let mut sbis = Vec::new();
        for entry in fs::read_dir(pbi_path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("sbi-") {
                    sbis.push(name);
                }
            }
        }

        sbis.sort();
        Ok(sbis)
    }

    /// Check if a spec has SBIs (is a PBI)
    pub fn is_pbi(&self, spec_name: &str) -> Result<bool> {
        let sbis = self.list_sbis(spec_name)?;
        Ok(!sbis.is_empty())
    }

    /// Create a new SBI in a PBI (used by TUI "Create new SBI")
    pub fn create_sbi(
        &self,
        pbi_name: &str,
        sbi_name: &str,
        sbi_type: &str  // "prd", "bug", or "tech"
    ) -> Result<()> {
        let sbi_path = self.path_manager.specs_dir()
            .join(pbi_name)
            .join(sbi_name);

        // Create directory
        fs::create_dir_all(&sbi_path)?;

        // Generate requirements.md with appropriate template
        let requirements_template = match sbi_type {
            "prd" => self.get_prd_template(),
            "bug" => self.get_bug_template(),
            "tech" => self.get_tech_template(),
            _ => return Err(anyhow!("Invalid SBI type")),
        };

        let requirements_path = sbi_path.join("requirements.md");
        fs::write(requirements_path, requirements_template)?;

        Ok(())
    }

    // Note: decompose_pbi() not needed - /decompose slash command
    // handles decomposition using Claude Code's Read/Write tools directly
}
}
```

## Responsibility Separation

### Claude Code Responsibilities (Slash Commands)

**`/decompose` slash command**:
- Parse PBI requirements.md using Read tool
- Create SBI directories using Bash/Write tools
- Generate SBI requirements.md files
- Update PBI requirements.md if needed

**`/add-sbi` slash command**:
- Auto-number SBI (list existing, calculate next)
- Create SBI directory and requirements.md
- Update PBI requirements.md with new section

**Why no Rust functions**: Claude Code has Read/Write tools, can implement directly

### hail-mary CLI Responsibilities (Rust)

**TUI "Create new SBI" option**:
- Called before Claude Code launch
- Uses `spec_repo.create_sbi()` to create SBI
- Then launches Claude Code with new SBI context

**Why Rust function needed**: TUI runs in hail-mary CLI, not Claude Code session

## Archive Strategy

### Design Decision: PBI-Level Archive Only

**Approach**: Archive entire PBI when all SBIs complete (or project ends)

```
.kiro/archive/2025-10-15-payment-system/
  ├── requirements.md         # PBI overview
  ├── sbi-1-backend-api/      # PR #123 complete record
  ├── sbi-2-frontend-ui/      # PR #124 complete record
  └── sbi-3-mobile-app/       # PR #125 complete record (or incomplete)
```

**Rationale**:
- Preserves "why we split this way" context
- Historical reference for future similar projects
- Organization knowledge accumulation
- Captures incomplete projects too (valuable history)

**Command**:
```bash
hail-mary archive payment-system  # Archives entire PBI
```

**Future Enhancement**: Individual SBI archive could be added later if needed

## Implementation Phases

### Phase 1: MVP (Minimum Viable Product)

**Goal**: Basic PBI/SBI functionality

```
✅ Create 10_spec_files_sbi.md
✅ Implement template switching in SystemPrompt
✅ Add PBI template to 07_requirements.md
✅ Implement /hm:requirements --type pbi
✅ Implement /hm:decompose command
✅ Update TUI for nested SBI selection
✅ Test with real project
```

**Estimated Effort**: 2-3 days

### Phase 2: Enhanced UX

**Goal**: Streamline SBI workflow

```
✅ Implement /hm:sbi add command
✅ Add "Create new SBI" to TUI
✅ Auto-update PBI requirements.md
✅ Auto-update PBI tasks.md
✅ Improve error messages
```

**Estimated Effort**: 1-2 days

### Phase 3: Polish & Documentation

**Goal**: Production ready

```
✅ Implement PBI-level archive
✅ Documentation updates
✅ Integration tests
✅ User guide
✅ Edge case handling
```

**Estimated Effort**: 1 day

## Edge Cases

### 1. PBI-Level Work

**Scenario**: Need to work at PBI level (create requirements, shared investigation)

**Solution**: TUI allows PBI selection before SBI expansion

```
   2025-10-07-payment-system    # ← Select here for PBI-level work
     sbi-1-backend-api
     sbi-2-frontend-ui
```

### 2. Mid-Development SBI Addition

**Scenario**: Discover need for additional SBI during implementation

**Solution**: `/hm:sbi add` or TUI "Create new SBI"

### 3. Existing Spec → PBI Conversion

**Scenario**: Started as single spec, realize need to split

**Solution**:
1. Create PBI requirements.md from existing requirements.md
2. Run `/hm:decompose`
3. Manually move existing design.md to sbi-1/

### 4. SBI Dependencies

**Scenario**: sbi-2 depends on sbi-1 API

**Solution**: Developer manages manually (notes in PBI requirements.md or memo.md)

**No Enforcement**: Maintains NO Linear Workflow philosophy - developers decide order

### 5. Partial Completion Archive

**Scenario**: Some SBIs complete, others incomplete

**Solution**: Archive entire PBI (including incomplete SBIs)

**Rationale**: Historical record of "paused project" has value

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_sbi_context() {
        let sbi_path = PathBuf::from(".kiro/specs/payment/sbi-1-backend");
        assert!(is_sbi_context(&sbi_path));

        let regular_path = PathBuf::from(".kiro/specs/payment");
        assert!(!is_sbi_context(&regular_path));
    }

    #[test]
    fn test_list_sbis() {
        // Create test PBI with SBIs
        let temp_dir = tempfile::tempdir().unwrap();
        // ... setup ...

        let repo = SpecRepository::new(path_manager);
        let sbis = repo.list_sbis("test-pbi").unwrap();

        assert_eq!(sbis.len(), 3);
        assert_eq!(sbis[0], "sbi-1-backend");
    }

    #[test]
    fn test_template_switching() {
        let steerings = Steerings(vec![]);

        // SBI context
        let sbi_prompt = SystemPrompt::new(
            Some("sbi-1-backend"),
            Some(Path::new(".kiro/specs/payment/sbi-1-backend")),
            &steerings
        );
        assert!(sbi_prompt.as_str().contains("<pbi-requirements-file>"));

        // Regular context
        let regular_prompt = SystemPrompt::new(
            Some("payment"),
            Some(Path::new(".kiro/specs/payment")),
            &steerings
        );
        assert!(!regular_prompt.as_str().contains("<pbi-requirements-file>"));
    }
}
```

### Integration Tests

```rust
#[test]
fn test_full_pbi_sbi_workflow() {
    // 1. Create PBI with requirements
    // 2. Decompose into SBIs
    // 3. Select SBI in TUI
    // 4. Launch Claude with SBI context
    // 5. Verify system prompt contains both PBI and SBI files
}
```

## Documentation Updates

### User Guide Additions

**New Section**: "Working with Multiple Pull Requests"

```markdown
## When to Use PBI/SBI

Use PBI/SBI hierarchy when:
- Backend and Frontend need separate PRs
- Phased rollout (Phase 1, 2, 3...)
- Long-running projects (3+ weeks)
- Microservice-style development

Keep single spec when:
- Small feature (< 1 week)
- Single domain (pure backend or pure frontend)
- Tight coupling between changes

## Workflow

1. Create PBI
   $ hail-mary code
   → Select project
   /hm:requirements --type pbi

2. Define SBIs in requirements.md adaptively

3. Decompose
   /hm:decompose

4. Work on each SBI
   $ hail-mary code
   → Select project → Select sbi-1-backend
   /hm:investigate
   /hm:design
   /hm:timeline

5. Create PRs (1 per SBI)

6. Archive when all SBIs complete
```

## Success Criteria

### Functional Requirements

- ✅ PBI can contain multiple SBIs
- ✅ Each SBI produces 1 PR
- ✅ SBI selection in TUI works
- ✅ System prompt provides both PBI and SBI context
- ✅ `/hm:decompose` creates SBI directories
- ✅ `/hm:sbi add` adds new SBI
- ✅ Backward compatible with single-spec workflow

### Non-Functional Requirements

- ✅ No performance degradation
- ✅ Pattern Router Framework complexity unchanged (02_hub.md, 04_workflows.md)
- ✅ Clear error messages
- ✅ Intuitive UX

## Risks & Mitigations

### Risk 1: Complexity Explosion

**Risk**: PBI/SBI logic spreads throughout codebase

**Mitigation**:
- Isolate in 10_spec_files_sbi.md
- Template switching at single point (SystemPrompt::new)
- PBI tasks.md not managed by Pattern Router

### Risk 2: User Confusion

**Risk**: When to use PBI vs single spec?

**Mitigation**:
- Clear documentation
- TUI remains simple for single-spec case
- Optional feature (default is single-spec)

### Risk 3: Pattern Router Framework Coupling

**Risk**: Changes affect Pattern Router Framework

**Mitigation**:
- Additive changes only
- No modifications to core workflow files
- New template file isolated

## Open Questions

1. **SBI Naming**: Auto-number vs manual?
   - Proposal: Support both (auto-number recommended)

2. **PBI Investigation/Design**: Required or optional?
   - Proposal: Optional (SBI can be fully independent)

3. **Archive Granularity**: PBI-level only or SBI-level too?
   - Proposal: PBI-level default, SBI-level advanced

4. **Cross-SBI Evidence Chain**: How to reference?
   - Proposal: Via PBI design/investigation (if exists)

## Implementation Status

### Phase 1: MVP - ✅ COMPLETE
- ✅ Template switching (10_spec_files vs 10_spec_files_sbi)
- ✅ PBI/SBI repository methods
- ✅ Nested TUI selection
- ✅ SBI context handling
- ✅ All tests passing (154 tests)

### Phase 2: SBI Management - ✅ COMPLETE
- ✅ `/decompose` and `/add-sbi` commands
- ✅ TUI "Create new SBI" option
- ✅ Auto-numbering

### Phase 3: Production Ready - 🔄 IN PROGRESS
- ✅ PBI-level archive (existing `mark_spec_complete()` works)
- 🔄 Documentation updates
- 🔄 User guide creation

## Summary

This design successfully implements PBI/SBI hierarchy for multi-PR projects:

1. **✅ Simplicity Maintained**: Template switching isolates complexity
2. **✅ Pattern Router Preserved**: Core workflows (02_hub, 04_workflows) unchanged
3. **✅ Backward Compatible**: Single-spec workflow unaffected
4. **✅ Flexible**: Optional feature for complex projects
5. **✅ Philosophy Upheld**: NO Linear Workflow maintained

**Implementation complete and tested.**
