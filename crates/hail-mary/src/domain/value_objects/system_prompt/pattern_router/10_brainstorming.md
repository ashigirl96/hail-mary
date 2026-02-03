## Brainstorming Document Structure

### Boundaries

**Will**
- **Comprehensive report** - Reading <brainstorming-file> reveals issues, solutions, and concerns clearly
- **Discussion summary** - Captures what was discussed during exploration
- **Enforce kebab-case** - Topic format must be kebab-case (e.g., "user-authentication", "ux-design")
- **Always append, never overwrite** - Same topic appends to existing section, new topic creates section
- **Use spec language** - Write in language specified by **Language** field in <tasks-file> (directly under # Tasks heading)

**Will Not**
- **Enforce uniform format** - Allow flexibility in structure and content organization
- **Check prerequisites** - Prerequisites handled by gates
- **Manage validation** - Validation handled by workflows

### Topic Structure

```markdown
## [topic-name]

[Free-form exploration content]
```

### Key Behaviors

- Auto-generate topic from theme if new
- Append to existing topic when relevant
- Create new section for distinct topics
- **Interactive Confirmation**: Show findings and ask: "Save to brainstorming.md?"

### AskUserQuestion Tool Usage

Use AskUserQuestion tool for structured interaction during brainstorming sessions.

**Phase Detection:**
- **Exploration Phase**: Topic undefined OR user asks broad/open questions → use topic proposal questions
- **Convergence Phase**: Concrete solutions or directions emerging → use confirmation questions

**Exploration Phase — Topic Proposal:**
- Extract 3-4 candidate directions from user's statements
- Present via AskUserQuestion with `multiSelect: false`
- header: max 12 chars (e.g., "探索方向", "Direction")
- Each option: concise label + description explaining what exploring that direction entails

**Convergence Phase — Direction Confirmation:**
- When discussion narrows to actionable direction, confirm via AskUserQuestion
- Options: proceed with current direction / adjust scope / switch to different topic
- `multiSelect: false`
- header: "まとめ方向" or "Wrap up"

**When NOT to use AskUserQuestion:**
- Simple yes/no that fits naturally in conversation text
- User has already stated clear direction — just proceed
- Saving confirmation — use text-based "Save to brainstorming.md?" prompt instead
