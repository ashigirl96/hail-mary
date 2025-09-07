# Tasks: hail-mary code Command Implementation

## References
- [Design Document](./design.md)
- [Requirements](./requirements.md)
- [ARCHITECTURE.md](../../../../ARCHITECTURE.md)

## Phase 1: Domain Layer Setup
- [ ] Create `domain/value_objects/` directory if not exists
- [ ] Implement `domain/value_objects/system_prompt.rs`
  - [ ] Define `SystemPrompt` struct with `content: String`
  - [ ] Implement `new(spec_name, spec_path)` method
  - [ ] Implement `as_str()` method
  - [ ] Add XML tag formatting for Kiro paths
- [ ] Add module declaration in `domain/value_objects/mod.rs`
- [ ] Export from `domain/mod.rs`

## Phase 2: Infrastructure Layer - Process Management
- [ ] Create `infrastructure/process/` directory
- [ ] Create `infrastructure/process/mod.rs`
- [ ] Implement `infrastructure/process/claude_launcher.rs`
  - [ ] Define `ClaudeProcessLauncher` struct
  - [ ] Implement `new()` method
  - [ ] Implement `launch(system_prompt)` method
  - [ ] Add `which claude` check for CLI availability
  - [ ] Handle process spawn with `--append-system-prompt`
  - [ ] Add error handling for missing Claude CLI

## Phase 3: Infrastructure Layer - TUI
- [ ] Create `infrastructure/tui/` directory
- [ ] Create `infrastructure/tui/mod.rs`
- [ ] Implement `infrastructure/tui/spec_selector.rs`
  - [ ] Define `SpecSelectorTui` struct
  - [ ] Define `SpecSelectionResult` enum
  - [ ] Implement `new(specs)` method
  - [ ] Implement `run()` method with terminal setup/cleanup
  - [ ] Implement `draw_ui()` method
  - [ ] Add keyboard navigation (↑/↓/j/k/Enter/q/Esc)
  - [ ] Add "Create new specification" option at top
  - [ ] Filter out archived specs

## Phase 4: Application Layer
- [ ] Create `application/use_cases/launch_claude_with_spec.rs`
  - [ ] Implement main `launch_claude_with_spec` function
  - [ ] Add spec listing via `ProjectRepository`
  - [ ] Integrate TUI for spec selection
  - [ ] Handle new spec creation flow
  - [ ] Implement `prompt_for_spec_name()` helper
  - [ ] Implement `validate_spec_name()` helper
  - [ ] Generate system prompt
  - [ ] Launch Claude process
- [ ] Add module export in `application/use_cases/mod.rs`

## Phase 5: CLI Layer
- [ ] Create `cli/commands/code.rs`
  - [ ] Define `CodeCommand` struct
  - [ ] Implement `new()` method
  - [ ] Implement `execute()` method
  - [ ] Add project discovery logic
  - [ ] Call `launch_claude_with_spec` use case
- [ ] Update `cli/commands/mod.rs` to include code module
- [ ] Update `cli/args.rs`
  - [ ] Add `Code(CodeCommand)` variant to `Commands` enum
  - [ ] Add command description

## Phase 6: Main Integration
- [ ] Update `main.rs`
  - [ ] Add match arm for `Commands::Code`
  - [ ] Wire up command execution

## Phase 7: ProjectRepository Extension
- [ ] Add `get_spec_path(&self, name: &str)` method to trait
- [ ] Implement in `infrastructure/repositories/project.rs`
- [ ] Implement in mock repository for testing

## Phase 8: Testing
- [ ] Unit tests for `SystemPrompt`
  - [ ] Test XML tag generation
  - [ ] Test path formatting
- [ ] Unit tests for `launch_claude_with_spec`
  - [ ] Test with mock repository
  - [ ] Test spec name validation
  - [ ] Test cancellation flow
- [ ] Integration test for TUI
  - [ ] Test navigation
  - [ ] Test selection
  - [ ] Test cancellation
- [ ] Integration test for process launcher
  - [ ] Mock process execution
  - [ ] Test error cases

## Phase 9: Manual Testing
- [ ] Build project: `cargo build`
- [ ] Test without Claude installed (should show error)
- [ ] Install Claude Code CLI if not present
- [ ] Test spec selection from existing specs
- [ ] Test new spec creation flow
- [ ] Test cancellation (q/Esc)
- [ ] Verify system prompt appears in Claude
- [ ] Test on different platforms if possible

## Phase 10: Documentation
- [ ] Update main README.md with new command
- [ ] Add usage examples
- [ ] Document Claude Code CLI requirement
- [ ] Update ARCHITECTURE.md if needed

## Completion Checklist
- [ ] All tests passing: `cargo test`
- [ ] Code formatted: `cargo fmt`
- [ ] Linting clean: `cargo clippy`
- [ ] Manual testing completed
- [ ] Documentation updated
- [ ] Commit with meaningful message