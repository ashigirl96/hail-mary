# Requirements: hail-mary code Command

## Overview

The `hail-mary code` command integrates Kiro project specifications with Claude Code by launching Claude with a system prompt containing all relevant specification file paths and their roles. This enables a focused development workflow where Claude Code has complete context about the current task.

## User Stories

- As a **developer**, I want to launch Claude Code with my current Kiro specification context, so that Claude understands exactly what I'm working on
- As a **developer**, I want to select from existing specifications or create new ones, so that I can start working immediately without manual setup
- As a **developer**, I want Claude to have structured access to my requirements, design, tasks, and notes, so that it can provide accurate implementation guidance
- As a **project manager**, I want specifications to be consistently referenced in Claude Code, so that implementations follow the documented requirements

## Acceptance Criteria

### Core Functionality
- [x] Command `hail-mary code` launches a TUI for specification selection
- [x] TUI displays all existing specifications from `.kiro/specs/` directory
- [x] TUI includes a "Create new specification" option
- [x] Selecting an existing spec launches Claude Code with appropriate context
- [x] Creating a new spec prompts for name and creates all required files
- [x] System prompt includes XML-tagged paths for all specification files
- [x] System prompt includes clear descriptions of each file's purpose

### User Experience
- [x] TUI supports keyboard navigation (arrow keys, j/k for movement)
- [x] TUI supports selection with Enter key
- [x] TUI supports cancellation with q or Esc
- [x] Clear error messages when Claude Code is not installed
- [x] Validation prevents duplicate specification names
- [x] Progress indicators during spec creation

### Technical Requirements

#### Command Structure
- New command variant `Code` in CLI args enum
- Command implementation follows existing pattern (InitCommand, NewCommand, etc.)
- Integration with main.rs command routing

#### System Prompt Format
```xml
<kiro_spec_name>spec-name</kiro_spec_name>
<kiro_spec_path>.kiro/specs/spec-name/</kiro_spec_path>
<kiro_requirements_path>.kiro/specs/spec-name/requirements.md</kiro_requirements_path>
<kiro_design_path>.kiro/specs/spec-name/design.md</kiro_design_path>
<kiro_tasks_path>.kiro/specs/spec-name/tasks.md</kiro_tasks_path>
<kiro_memo_path>.kiro/specs/spec-name/memo.md</kiro_memo_path>
```

#### Process Execution
- Use `std::process::Command` for cross-platform compatibility
- Check for Claude CLI availability using `which claude` command
- Execute `claude --append-system-prompt <prompt>` with generated prompt
- Handle process spawn and wait for completion

#### Error Handling
- Graceful handling of missing Claude CLI installation
- Clear error messages with actionable guidance
- Rollback spec creation on failure
- Proper cleanup of TUI on error

## Non-Functional Requirements

### Performance
- TUI should respond to input within 100ms
- Spec list loading should complete within 500ms
- Claude launch should start within 2 seconds

### Compatibility
- Support macOS, Linux, and Windows
- Work with Claude Code CLI v0.1.0 and later
- Compatible with existing Kiro project structure

### Usability
- Intuitive TUI navigation matching other CLI tools
- Consistent with existing hail-mary command patterns
- Clear visual feedback for all actions
- Helpful error messages with recovery suggestions

## Dependencies

### External Dependencies
- Claude Code CLI must be installed and available in PATH
- Terminal must support TUI rendering (crossterm/ratatui)

### Internal Dependencies
- Existing ProjectRepository trait and implementation
- PathManager for project discovery
- Existing spec creation logic from NewCommand

## Testing Requirements

### Unit Tests
- Domain entity creation and validation
- System prompt generation with correct XML tags
- Use case logic for spec selection and creation

### Integration Tests
- TUI interaction flow testing
- Process launching with mock Claude command
- Error scenario handling

### Manual Testing
- Complete workflow from selection to Claude launch
- New spec creation and immediate use
- Error cases (no Claude, no project, etc.)
- Cross-platform verification

## Future Considerations

### Phase 2 Features
- Multiple spec selection for cross-reference work
- Session history and resumption
- Custom system prompt templates
- Integration with memory database

### Phase 3 Features
- Auto-sync between Claude Code and Kiro tasks
- Progress tracking from Claude sessions
- Collaborative features for team development
- Claude Code configuration profiles

## Success Metrics

- Reduction in time to start coding with context (target: <30 seconds)
- Increase in specification adherence (measurable through task completion)
- User satisfaction with integrated workflow
- Reduction in context-switching overhead