# Feature Spec Creation

## Process

1. **Check state**: If feature spec exists → offer to refine or resolve open questions
2. **Dialogue** via AskUserQuestion to clarify What/How/Scope
3. **Write spec** to prd.md
4. **Check readiness** via checklist

## Dialogue Approach

Ask questions to make the feature 100% clear. Focus on:

- **WHAT**: What are we building? What specific functionality?
- **HOW**: How do you envision implementing this? Technical approach?
- **SCOPE**: What's included and what's not?
- **WHY**: Why is this needed? (include naturally as background)

### Question Examples

**Clarifying WHAT**:
- What's the most important aspect of this feature?
- What will users be able to do specifically?
- How is this different from existing functionality?

**Clarifying HOW**:
- How do you envision implementing this technically?
- Which parts of the existing codebase will be modified?
- Any external services or libraries needed?

**Clarifying SCOPE**:
- Anything explicitly out of scope?
- What's the minimum viable version?
- What's deferred for later?

**Digging into unknowns**:
- Is this part still undecided?
- There are multiple approaches here - which do you prefer?

## Format

Free-form. Write in whatever style fits the feature (prose or bullets).
Weave decisions from dialogue naturally into the main text.

**Required sections**:
- Open Questions: List of unclear points
- Readiness Checklist: Completion criteria

## Output Template

When creating new:

```markdown
# PRD: {spec-name}

## Feature Spec

{free-form description}

### Open Questions
- [ ] {unclear point}

### Readiness Checklist
- [ ] WHY: Clear why this feature is needed
- [ ] WHAT: Clear what to build specifically
- [ ] HOW: Clear high-level implementation approach
- [ ] SCOPE: Clear what's in and out of scope
- [ ] Open Questions is empty

---

## User Stories

(To be defined with /prd stories)
```

## Readiness Check

Feature Spec is "complete" when:

1. All Open Questions are resolved
2. All Readiness Checklist items are checked

Both satisfied → proceed to `/prd stories`

## Refinement Flow

When updating existing Feature Spec:

1. Review current Open Questions
2. Resolve unknowns through dialogue
3. Update main text (weave in decisions)
4. Remove resolved Open Questions
5. Update Checklist
