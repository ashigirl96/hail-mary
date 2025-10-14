# Tasks

**Language**: ja

## State Tracking

| Document | Status | Coverage | Next Action |
|----------|--------|----------|-------------|
| requirements.md | complete | - | - |
| investigation.md | complete | 5/5 (100%) | - |
| design.md | complete | - | - |
| tasks.md#Timeline | complete | phase1: 12/12 (100%) | - |

## Timeline

- [x] Spec created → brainstorming
- [x] Requirements definition → requirements.md#overview
- [x] investigation: brainstorm-pipeline-design → investigation.md#brainstorm-pipeline-design
- [x] investigation: slash-command-structure → investigation.md#slash-command-structure
- [x] investigation: pattern-recognition-extension → investigation.md#pattern-recognition-extension
- [x] investigation: repository-layer-implementation → investigation.md#repository-layer-implementation
- [x] investigation: testing-strategy → investigation.md#testing-strategy
- [x] Design documentation → design.md#overview
- [x] phase1: Brainstorm Pipeline実装 → design.md#implementation-order
  - [x] pattern_router/10_brainstorming.md作成
  - [x] pattern_router/03_patterns.md更新（BRAINSTORM pattern）
  - [x] pattern_router/04_workflows.md更新（Brainstorm Pipeline）
  - [x] pattern_router/06_nudges.md更新（Brainstorm templates）
  - [x] pattern_router/11_spec_files.md更新（brainstorming-file）
  - [x] pattern_router/index.md更新（brainstorming変数）
  - [x] system_prompt/mod.rs更新（include_str! + path + tests）
  - [x] .claude/commands/spec/brainstorm.md作成
  - [x] application/repositories/spec_repository.rs更新（Interface）
  - [x] infrastructure/repositories/spec.rs更新（Implementation + tests）
  - [x] infrastructure/embedded_resources.rs更新（Templates）
  - [x] Run just test（154 tests → 156+ tests維持）
