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
