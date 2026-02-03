---
name: prd
description: Manage feature development lifecycle - objectives, user stories, and implementation plans. Use when user says "PRDを管理", "objectiveを作成", "user storyを考えて", "プランを作成", "実装したい", or needs to track feature progress.
argument-hint: "[objective | stories [--add|--edit US-N|--status US-N] | plan [US-N]]"
model: opus
allowed-tools: Read, Write, Edit, Glob, Grep, Bash, Task, AskUserQuestion, EnterPlanMode
---

# PRD Management Skill

## Workflow Overview

```
/prd objective  →  /prd stories  →  /prd plan  →  implement  →  /prd stories --status US-N
     ↓                  ↓               ↓                              ↓
  Define WHY      Break down       Create plan              Mark completed or failed
                  into stories     & implement                     ↓
                                                            If failed: /prd plan (retry)
```

## Routing

| Argument | Action | Reference |
|----------|--------|-----------|
| (none) | Show status, suggest next action | — |
| `objective` | Create/refine prose objective | [objective.md](references/objective.md) |
| `stories` | Generate stories from objective | [user-stories.md](references/user-stories.md) |
| `stories --add` | Add new story interactively | [user-stories.md](references/user-stories.md) |
| `stories --edit US-N` | Edit existing story | [user-stories.md](references/user-stories.md) |
| `stories --status US-N` | Update status (+ root cause if failed) | [user-stories.md](references/user-stories.md) |
| `plan [US-N]` | Create plan with **dig process** | [plan-story.md](references/plan-story.md) |

### Plan's Dig + Decomposition Process

`/prd plan` includes two key phases:

**Dig (Clarify)**:
1. Identify ambiguities in story/criteria
2. Ask structured questions (2-4 options with pros/cons)
3. Document decisions in table format

**Decompose (Break down)**:
1. Identify major components (3-7 phases)
2. Convert to granular todos (5-30 min each)
3. Each todo has: What/Where/How/Why/Verify

This ensures plans are concrete, actionable, and executable.

## Context

- **Spec Path**: `<spec-path>` (from system prompt)
- **PRD File**: `<prd-file>` (from system prompt)
- **Plans Dir**: `<plans-dir>` (from system prompt)

## PRD Format

```markdown
# PRD: {spec-name}

## Objective

{prose - no bullets, no tech details, focus on WHY}

---

## User Stories

### US-1: {title}

- **Status**: pending | in_progress | completed | failed
- **Priority**: 1
- **Description**: {1-2 sentences}

**Acceptance Criteria**:
- [ ] {criterion}

**Plan History**:
1. [plan-US-1-v1.md](plans/plan-US-1-v1.md)
   - **Root Cause**: {why it failed}
2. [plan-US-1-v2.md](plans/plan-US-1-v2.md)
```

### Status Values

| Status | Meaning |
|--------|---------|
| `pending` | Not started |
| `in_progress` | Currently implementing |
| `completed` | Done |
| `failed` | Failed (root cause recorded in Plan History) |

### Plan History Semantics

- Entry **with** Root Cause = that plan failed
- Entry **without** Root Cause = current/successful plan

## Default Behavior (no args)

1. Check if `<spec-path>/prd.md` exists
2. Analyze state and suggest next action:

| State | Suggestion |
|-------|------------|
| No prd.md | "Run `/prd objective` to start" |
| No objective | "Run `/prd objective` to define what you're building" |
| Objective only | "Run `/prd stories` to break it down" |
| All stories pending | "Run `/prd plan` to start implementing" |
| Story in_progress | Show which story, suggest continue or mark status |
| Story failed | "Run `/prd plan` to retry with lessons learned" |
| All completed | "All done! Start new feature or refine." |

3. Show concise status summary

## Error Handling

| Error | Response |
|-------|----------|
| `/prd stories` without objective | "No objective found. Run `/prd objective` first." |
| `/prd plan` without stories | "No stories found. Run `/prd stories` first." |
| `/prd plan` all completed | "All stories completed. Add new stories or start new feature." |
| `/prd stories --status US-99` (not found) | "US-99 not found. Available: US-1, US-2, US-3" |
