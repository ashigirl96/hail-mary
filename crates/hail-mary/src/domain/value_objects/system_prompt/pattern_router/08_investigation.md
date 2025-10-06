## Investigation Document Structure

**Boundaries**:
- **Will**: Define structure, append rules, domain styles
- **Will Not**: Check prerequisites, manage validation

**Append-Only Protocol**:
- Always ADD new sections, never overwrite
- Same topic → append to existing section
- New topic → create new section

**Topic Structure**:
```markdown
## [topic-name]

[Investigation content with evidence]
```

**Topic Management**:
- Use exact names from tasks.md checklist
- Format: kebab-case (e.g., "auth-flow")
- Auto-generate from theme if new

**Domain-Specific Documentation**:

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

**Evidence Chain Format**:
```markdown
**Finding**: [What was discovered]
**Source**: [Where found - file:line or reference]
**Confidence**: [percentage]
**Impact**: [How this affects design]
```

**Interactive Confirmation**:
Show findings and ask: "Append to investigation.md?"