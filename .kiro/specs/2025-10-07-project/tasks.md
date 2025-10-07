# Tasks

## Required Investigations
*Topics will be defined after requirements completion*

## State Tracking

| Document | Status | Coverage | Next Action |
|----------|--------|----------|-------------|
| requirements.md | pending | - | Define requirements |
| investigation.md | pending | - | Start investigation after requirements |
| design.md | complete | - | - |
| tasks.md#Timeline | complete | 12/12 (100%) | ✅ All phases complete |

## Timeline

- [x] Feature spec created → project
- [x] Design documentation → design.md
- [x] phase1: MVP - Core PBI/SBI Infrastructure → design.md#implementation-phases
  - [x] Pattern Router: Create 10_spec_files_sbi.md
  - [x] Pattern Router: Update 07_requirements.md (PBI template)
  - [x] Pattern Router: Update .claude/commands/hm/requirements.md (add pbi)
  - [x] Pattern Router: Create .claude/commands/pbi/decompose.md
  - [x] Pattern Router: Create .claude/commands/pbi/add-sbi.md
  - [x] Rust: Implement template switching in mod.rs (10_spec_files vs 10_spec_files_sbi)
  - [x] Rust: Implement spec.rs (is_pbi, list_sbis, create_sbi)
  - [x] Rust: Update TUI spec_selector.rs (nested SBI selection)
  - [x] Rust: Update launch_claude_with_spec.rs (SBI context handling)
  - [x] Rust: Unit tests (154 passed)
- [x] phase2: SBI Management (Merged into Phase 1)
  - [x] Pattern Router: Create .claude/commands/pbi/add-sbi.md (already done)
  - [x] Rust: TUI "Create new SBI" option (implemented in Phase 1)
  - [ ] Rust: Error handling improvements
- [x] phase3: Production Ready → design.md#phase-3-polish--documentation
  - [x] PBI-level archive (mark_spec_complete works for PBI)
  - [x] Documentation updates (design.md + README.md完成)
  - [ ] User guide creation (optional)
  - [x] Core implementation complete and tested
