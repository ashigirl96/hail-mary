## Investigation Document Structure

### Boundaries

**Will**
- **Define structure** - Topic-based append-only format
- **Always append, never overwrite** - Same topic appends to existing section, new topic creates section
- **Use exact topic names** - From tasks.md checklist
- **Enforce kebab-case** - Topic format must be kebab-case (e.g., "auth-flow")
- **Provide domain styles** - Technical vs Business/Scientific documentation patterns

**Will Not**
- **Check prerequisites** - Prerequisites handled by gates
- **Manage validation** - Validation handled by workflows

### Topic Structure

```markdown
## [topic-name]

[Investigation content with evidence]
```

### Domain-Specific Documentation

**Technical Domain**:
- File paths with line numbers: `src/auth/login.ts:142`
- Implementation mapping: URL → Router → Component
- Search patterns documentation
- Concrete code examples

**Business/Scientific Domain**:
- Plain language explanations
- Context: "exposure = risk amount"
- "Why this matters" sections
- Core concept simplification

### Evidence Chain Format

```markdown
**Finding**: [What was discovered]
**Source**: [Where found - file:line or reference]
**Confidence**: [percentage]
**Impact**: [How this affects design]
```

### Key Behaviors

- Auto-generate topic from theme if new
- Append to existing topic when relevant
- Create new section for distinct topics
- **Interactive Confirmation**: Show findings and ask: "Append to investigation.md?"