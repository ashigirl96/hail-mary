---
name: status
description: "Show current spec progress - triggered by: status, session resume, progress check"
allowed-tools: Read
---

# /spec:status

Show comprehensive status of current specification for session resumption.

Refer to system prompt for data structures:
- <kiro-hub> for State Tracking Table format
- <kiro-patterns> for QUERY pattern routing (this command uses Diagnostic Pipeline)

## Behavioral Flow

1. **Context Detection**: Check for <pbi-requirements-file> tag in system prompt
   - **Present** → PBI/SBI context (use PBI/SBI format)
   - **Absent** → Single Spec context (use Single Spec format)

2. **Read State**: Read <tasks-file> State Tracking Table
   - Extract document statuses (requirements, investigation, design, timeline)
   - Extract coverage percentages where applicable
   - Identify next actions from State Tracking

3. **Identify Gaps**:
   - **Single Spec**: Check Required Investigations checklist for incomplete items
   - **PBI/SBI**: Check other SBI statuses from parent PBI

4. **Format Report**: Based on context (see Output Format below)

5. **Recommend Next Action**:
   - **Blocked by investigations** → `/spec:investigate --topic [topic-name]`
   - **Investigations complete** → `/spec:design`
   - **Design complete** → `/spec:timeline`
   - **Timeline exists** → "Resume implementation: [next-task]"
   - **SBI complete** → Suggest next SBI or mark PBI complete

## Status Mapping

- `complete` → ✅ (Document finished)
- `in-progress` → 🔄 (Currently working)
- `pending` → ⏸️ (Not started or blocked)

## Output Format

### Single Spec Format

```markdown
📊 **Spec Status**: [spec-name]

## Progress Summary
| Document | Status | Coverage | Next Action |
|----------|--------|----------|-------------|
| requirements.md | [✅/🔄/⏸️] [status] | - | [next or -] |
| investigation.md | [✅/🔄/⏸️] [status] | [X/Y (N%)] | [topic or -] |
| design.md | [✅/🔄/⏸️] [status] | [N%] | [action or -] |
| tasks.md#Timeline | [✅/🔄/⏸️] [status] | [phase: X/Y] | [task or -] |

## Missing/Incomplete
[List incomplete investigation topics or timeline tasks]
- [ ] [topic-1]
- [ ] [topic-2]

## 🎯 Recommended Next Action
[Specific `/spec:*` command with exact parameters based on current state]
```

### PBI/SBI Context Format

```markdown
📊 **PBI Status**: [pbi-name]
🎯 **Current SBI**: [sbi-name]

## Current SBI Progress ([sbi-name])
| Document | Status | Coverage |
|----------|--------|----------|
| requirements.md | [✅/🔄/⏸️] [status] | - |
| investigation.md | [✅/🔄/⏸️] [status] | [X/Y (N%)] |
| design.md | [✅/🔄/⏸️] [status] | [N%] |
| tasks.md#Timeline | [✅/🔄/⏸️] [status] | [phase: X/Y] |

## PBI Overview
- [sbi-1-name]: [🔄/⏸️/✅] [N%] (current/not started/complete)
- [sbi-2-name]: [🔄/⏸️/✅] [N%]
- [sbi-3-name]: [🔄/⏸️/✅] [N%]

## 🎯 Recommended Next Action
[Context-aware recommendation for current SBI or next SBI]
- If current SBI incomplete: Suggest next step
- If current SBI complete: Suggest next SBI or PBI completion
```

## Key Behaviors

- **Read-only operation**: Never modify any files (Diagnostic Pipeline)
- **Context-aware**: Automatically detect and adapt to PBI/SBI vs Single Spec
- **Actionable**: Always provide specific next command with parameters
- **Coverage formatting**: "X/Y (N%)" for investigations, "phase: X/Y (N%)" for timeline
- **Data source**: All data from <tasks-file> State Tracking Table
