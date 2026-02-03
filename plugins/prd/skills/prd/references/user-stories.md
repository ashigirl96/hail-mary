# User Stories Management

## Modes

### Generate (default: `/prd stories`)

1. Read objective from prd.md
2. Generate 3-5 stories that fulfill objective
3. Review with user via AskUserQuestion
4. Write to prd.md

### Add (`--add`)

Gather via AskUserQuestion:
- Title (verb-first: "Add", "Enable", "Show")
- Description (1-2 sentences)
- Priority (1-5, lower = higher)
- Acceptance criteria (3-5 items)

### Edit (`--edit US-N`)

1. Show current story content
2. Ask what to change via AskUserQuestion:
   - Title
   - Description
   - Priority
   - Acceptance criteria
3. Update prd.md

### Status (`--status US-N`)

1. Show current status
2. Get new status via AskUserQuestion
3. **If failed**: Ask for root cause
4. Update prd.md (add root cause to Plan History if failed)

## Story Guidelines

**Titles**: Verb-first, under 60 chars
- "Add user authentication"
- "Enable dark mode"
- "Show error messages"

**Criteria**: Checkbox, observable, testable
```markdown
- [ ] User can log in with email/password
- [ ] Invalid credentials show error message
- [ ] Session persists across page refresh
```

## Story Template

```markdown
### US-{N}: {title}

- **Status**: pending
- **Priority**: {1-5}
- **Description**: {1-2 sentences}

**Acceptance Criteria**:
- [ ] {criterion}
- [ ] {criterion}
- [ ] {criterion}

**Plan History**:
(No plans yet)
```

## Recording Failure

When status → failed, update Plan History:

```markdown
**Plan History**:
1. [plan-US-1-v1.md](plans/plan-US-1-v1.md)
   - **Root Cause**: テストが不安定でCIが通らなかった
```

This enables learning in the next `/prd plan`.
