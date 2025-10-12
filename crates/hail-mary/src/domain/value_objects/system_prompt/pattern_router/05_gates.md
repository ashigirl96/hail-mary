## Strategy-Specific Validation Gates

**Gate Application by Pipeline**:

| Pipeline | Required Gates | Optional Gates | Bypass |
|----------|---------------|----------------|--------|
| Command | All document validation gates | - | Never |
| Review | None | - | All validation gates |

## Document Validation Gates (Command Pipeline Only)

**Design without Requirements**:
- Check: `requirements.md status = complete` in <tasks-file>
- Action: ❌ **BLOCK**
- Message: "❗ Requirements must be complete first (check tasks.md)"
- Applies to: Command Pipeline only

**Design without 100% Investigation**:
- Check: Investigation coverage in <tasks-file>
- Action: ❌ **BLOCK** with missing list
- Message: "❗ All investigations must be complete first. Missing: [list]"
- Applies to: Command Pipeline only

**Investigation without Topics**:
- Check: Timeline contains investigation items
- Action: ⚠️ **WARNING** with suggestion
- Message: "⚠️ Define investigation topics in requirements first?"
- Applies to: Command Pipeline only

**Incomplete Investigations Proceeding**:
- Check: Coverage < 100% but user requests design
- Action: ⚠️ **WARNING** with confirmation
- Message: "⚠️ Missing investigations: [list]. Complete these first?"
- Applies to: Command Pipeline only

**Missing Evidence in Design**:
- Check: Design lacks investigation references
- Action: ⚠️ **WARNING** with auto-fix offer
- Message: "⚠️ Design lacks evidence. Adding references from investigation.md..."
- Applies to: Command Pipeline only

**Timeline Planning without Design**:
- Check: <design-file> exists and has content
- Action: ⚠️ **WARNING** with guidance
- Message: "⚠️ No design found. Create design first with `/spec:design`, or describe what you want to implement?"
- Applies to: Command Pipeline only

## Gate Invocation Examples

```
Command Pipeline Example:
Input: "/spec:design"
Gates Applied: All document validation gates
Result: May block if requirements/investigation incomplete

Review Pipeline Example:
Input: "/spec:requirements --review"
Gates Applied: None
Result: Direct draft generation and review dialogue
```

## Key Principles

- **Pipeline-Specific**: Each pipeline has its own set of applicable gates
- **Efficiency**: Review Pipeline skips validation for fast feedback
- **Safety**: Command Pipeline maintains full validation
- **Isolation**: Gates only invoked when specified by pipeline strategy
