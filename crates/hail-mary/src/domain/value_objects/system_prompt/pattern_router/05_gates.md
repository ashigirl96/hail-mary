## Strategy-Specific Validation Gates

**Gate Application by Pipeline**:

| Pipeline | Required Gates | Optional Gates | Bypass |
|----------|---------------|----------------|--------|
| Command | All document validation gates | - | Never |
| Review | None | - | All validation gates |

## Document Validation Gates (Command Pipeline Only)

**Timeline Planning without Requirements**:
- Check: `requirements.md status = complete` in <tasks-file>
- Action: ⚠️ **WARNING** with guidance
- Message: "⚠️ No requirements found. Create requirements first with `/spec:requirements`, or describe what you want to implement?"
- Applies to: Command Pipeline only

## Gate Invocation Examples

```
Command Pipeline Example:
Input: "/spec:timeline"
Gates Applied: All document validation gates
Result: May warn if requirements incomplete

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
