# Plan Creation

## Story Selection

### With argument (`/prd plan US-2`)
Use specified story.

### Without argument (`/prd plan`)
Auto-select:
1. Filter: status = `pending` or `failed`
2. Sort by priority (lower = higher priority)
3. Prefer `failed` over `pending` (retry first)

---

## Process Overview

```
Load Context
    ↓
Explore Codebase ──→ Identify Major Components
    ↓
Dig (Clarify Ambiguities) ←─┐
    ↓                       │
Learn from Failures         │
    ↓                       │
Decompose into Todos        │
    ↓                       │
Review ────────────────────→┘ (loop if gaps)
    ↓
Create Plan File
    ↓
Update prd.md & Execute
```

---

## Phase 1: Load Context

Read from prd.md:
- **Objective**: Overall direction
- **Story**: Description, acceptance criteria
- **Plan History**: Past root causes

---

## Phase 2: Explore Codebase

Use Task tool (subagent_type=Explore) to understand:

| What to Find | Why |
|--------------|-----|
| Project structure | Know where code belongs |
| Existing patterns | Follow conventions |
| Affected files | Identify change scope |
| Dependencies | Avoid breaking changes |
| Test patterns | Write consistent tests |

---

## Phase 3: Identify Major Components

Break the story into distinct work phases:

1. List 3-7 major components/phases
2. Identify interdependencies
3. Determine execution sequence

**Example**:
```
US-3: Add user authentication

Components:
1. Database schema (users table) - no deps
2. Auth service (login/logout) - depends on 1
3. API endpoints - depends on 2
4. UI login form - depends on 3
5. Tests - depends on 1-4
```

---

## Phase 4: Dig - Clarify Ambiguities

**Purpose**: Surface and resolve unclear points BEFORE decomposing.

### 4.1 Identify Ambiguities

| Category | Examples |
|----------|----------|
| **Scope** | "Does this include password reset?" |
| **Approach** | "JWT or session-based auth?" |
| **Ordering** | "Can UI work start before API is done?" |
| **Granularity** | "How detailed should error messages be?" |
| **Acceptance** | "What counts as 'secure enough'?" |
| **Risk** | "What if the migration fails?" |

### 4.2 Structured Questions

Use AskUserQuestion. For each ambiguity:

- 2-4 concrete options
- Brief pros/cons per option
- NO open-ended questions

```
Question: How should we store user sessions?

1. JWT in localStorage
   - Pros: Stateless, scalable
   - Cons: XSS vulnerable, can't revoke

2. HTTP-only cookies with server sessions
   - Pros: More secure, revocable
   - Cons: Server state, CSRF protection needed

3. JWT in HTTP-only cookie
   - Pros: Balance of both
   - Cons: More complex setup
```

### 4.3 Document Decisions

```markdown
## Decisions

| Item | Choice | Reason | Notes |
|------|--------|--------|-------|
| Session storage | JWT in HTTP-only cookie | Security + stateless | Add CSRF token |
| Password hashing | bcrypt | Industry standard | cost=12 |
```

### 4.4 Iterate

After decisions, re-check: any NEW ambiguities revealed? If yes, loop back.

**Exit**: You can describe every step with confidence.

---

## Phase 5: Learn from Failures

If Plan History has root causes:

1. List each past failure
2. Understand what went wrong
3. Add dig question: "How do we prevent {failure}?"
4. Record mitigation in Decisions table

---

## Phase 6: Decompose into Todos

Convert each major component into granular, executable todos.

### Todo Criteria

Each todo must be:

| Criterion | Meaning |
|-----------|---------|
| **Specific** | Clear verb, exact paths, expected I/O |
| **Achievable** | No blockers, complete info to start |
| **Small** | 5-30 minutes to complete |
| **Verifiable** | Concrete check that it's done |

### Todo Structure (Rich Description)

```markdown
### TODO: Create users table migration

**What**: Create migration file for users table with email, password_hash, created_at

**Where**:
- `migrations/20240101_create_users.sql`
- Update `migrations/mod.rs`

**How**:
1. Create SQL file with CREATE TABLE
2. Add unique index on email
3. Register in migrations module

**Why**: Foundation for auth - must exist before AuthService

**Verify**:
- [ ] Migration runs without error
- [ ] `users` table exists with correct columns
- [ ] Email uniqueness constraint works
```

### Bad vs Good Todos

**Bad** (vague):
> "Set up authentication"

**Good** (specific):
> "Create `AuthService::login()` that takes email/password, queries users table, verifies bcrypt hash, returns JWT token. File: `src/services/auth.rs:45-80`"

---

## Phase 7: Review & Validate

Before creating plan file, check:

### Coverage Checklist
- [ ] Every acceptance criterion has todos covering it
- [ ] All major components have todos
- [ ] Dependencies are in correct order
- [ ] Past failure mitigations are included

### Todo Quality Checklist
- [ ] No todo takes >30 minutes
- [ ] Every todo has Where (file paths)
- [ ] Every todo has Verify steps
- [ ] No "figure out" or "decide later" language

**If gaps found**: Loop back to Phase 4 (Dig) or Phase 6 (Decompose).

---

## Phase 8: Create Plan File

Path: `<spec-path>/plans/plan-US-{N}-v{version}.md`

```markdown
# Plan: US-{N} - {title}

## Context

**Story**: {description}

**Acceptance Criteria**:
- {criterion}

## Decisions

| Item | Choice | Reason | Notes |
|------|--------|--------|-------|
| {item} | {choice} | {reason} | {notes} |

## Lessons from Previous Attempts

{Summary of failures and mitigations}
{Or "First implementation attempt."}

## Major Components

1. {Component} - {brief description}
2. {Component} - depends on 1
3. ...

## Todos

### Component 1: {name}

#### TODO 1.1: {title}

**What**: {action}
**Where**: {file paths}
**How**: {steps}
**Why**: {purpose}
**Verify**:
- [ ] {check}

#### TODO 1.2: {title}
...

### Component 2: {name}
...

## Verification Summary

| Acceptance Criterion | Covered by Todos |
|---------------------|------------------|
| {criterion 1} | 1.1, 1.3, 2.2 |
| {criterion 2} | 2.1, 3.1 |

## Risks & Mitigations

| Risk | Mitigation | Related Decision |
|------|------------|------------------|
| {risk} | {mitigation} | {decision} |
```

---

## Phase 9: Update & Execute

1. **Update prd.md**:
   - Status → `in_progress`
   - Add plan to Plan History

2. **Enter Plan Mode**:
   - Call EnterPlanMode
   - Execute todos in order

---

## Versioning

- `v1`: First attempt
- `v2`: After v1 failed
- Each version = fresh attempt incorporating lessons

---

## Failure Loop

```
Execute Todos → Fail
       ↓
/prd stories --status US-N → failed + root cause
       ↓
/prd plan → auto-selects failed story
       ↓
Phase 5: "How do we prevent {root cause}?"
       ↓
New decomposition with lessons → Retry
```
