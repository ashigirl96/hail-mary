# Objective Creation

## Process

1. **Check state**: If objective exists → offer to refine
2. **Socratic dialogue** via AskUserQuestion
3. **Write prose** to prd.md

## Socratic Questions

Ask one at a time, dig deeper on "why":

- What problem are you solving?
- Who benefits?
- What does success look like?
- Why is this important now?

Redirect "how" answers back to "why/what for".

## Writing Rules

- NO bullet points
- NO technical details
- 2-4 flowing paragraphs
- Answer: "Why does this exist?"

## Example

**Good**:
> Development teams struggle with maintaining focus during long coding sessions. They start with a clear goal but lose sight of the bigger picture as they dive into implementation.
>
> This feature provides a lightweight way to articulate what they're building and why before writing code. The goal is not bureaucratic documentation but a moment of clarity that guides decisions.

**Bad**:
> - Implement CLI command
> - Store in SQLite
> - Provide API endpoints

This is implementation, not purpose.

## Output

Update prd.md Objective section. If new file, create with:

```markdown
# PRD: {spec-name}

## Objective

{prose}

---

## User Stories

(To be defined with /prd stories)
```
