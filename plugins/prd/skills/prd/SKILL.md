---
name: prd
description: Manage feature development lifecycle - feature specs, user stories, and implementation plans. Use when user says "PRDを管理", "featureを定義", "user storyを考えて", "プランを作成", "実装したい", or needs to track feature progress.
argument-hint: "[feature | stories [--add|--edit US-N|--status US-N] | plan [US-N]]"
model: opus
allowed-tools: Read, Write, Edit, Glob, Grep, Bash, Task, AskUserQuestion, EnterPlanMode
---

# PRD Management Skill

## Workflow Overview

```
/prd feature  →  /prd stories  →  /prd plan  →  implement  →  /prd stories --status US-N
      ↓                ↓               ↓                              ↓
  Define WHAT     Break down       Create plan              Mark completed or failed
  + HOW + WHY     into stories     & implement                     ↓
                                                            If failed: /prd plan (retry)
```

## Routing

| Argument | Action | Reference |
|----------|--------|-----------|
| (none) | Show status, suggest next action | — |
| `feature` | Create/refine feature spec | [feature-spec.md](references/feature-spec.md) |
| `stories` | Generate stories from feature spec | [user-stories.md](references/user-stories.md) |
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

## Feature Spec

{free-form - what to build, how to implement, decisions made}

### Open Questions
- [ ] {unclear points remaining}

### Readiness Checklist
- [ ] WHY: Clear why this feature is needed
- [ ] WHAT: Clear what to build specifically
- [ ] HOW: Clear high-level implementation approach
- [ ] SCOPE: Clear what's in and out of scope
- [ ] Open Questions is empty

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
| No prd.md | "Run `/prd feature` to start" |
| No feature spec | "Run `/prd feature` to define what you're building" |
| Feature spec has Open Questions | "Run `/prd feature` to resolve open questions" |
| Feature spec ready (checklist complete) | "Run `/prd stories` to break it down" |
| All stories pending | "Run `/prd plan` to start implementing" |
| Story in_progress | Show which story, suggest continue or mark status |
| Story failed | "Run `/prd plan` to retry with lessons learned" |
| All completed | "All done! Start new feature or refine." |

3. Show concise status summary

## Error Handling

| Error | Response |
|-------|----------|
| `/prd stories` without feature spec | "No feature spec found. Run `/prd feature` first." |
| `/prd stories` with incomplete checklist | "Feature spec not ready. Run `/prd feature` to complete checklist." |
| `/prd plan` without stories | "No stories found. Run `/prd stories` first." |
| `/prd plan` all completed | "All stories completed. Add new stories or start new feature." |
| `/prd stories --status US-99` (not found) | "US-99 not found. Available: US-1, US-2, US-3" |
