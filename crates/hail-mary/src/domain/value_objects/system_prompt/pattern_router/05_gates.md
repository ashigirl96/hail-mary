## Strategy-Specific Validation Gates

**Gate Application by Pipeline**:

| Pipeline | Required Gates | Optional Gates | Bypass |
|----------|---------------|----------------|--------|
| Command | All document validation gates | - | Never |
| Suggestion | Confidence gate only | Cooldown gate | All document gates |
| Diagnostic | None | - | All gates |
| Recovery | None | Emergency override | All validation gates |

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

## Suggestion Gates (Suggestion Pipeline Only)

**Suggestion Confidence Gate**:
- Check: Accumulated confidence score (in-memory)
- Threshold: 0.7 for proactive suggestion
- Action: ✅ **ALLOW** suggestion generation
- Below threshold: Continue accumulating silently
- Applies to: Suggestion Pipeline only

**Suggestion Cooldown Gate**:
- Check: Previous suggestion status (ephemeral state)
- If dismissed: Block same suggestion for current topic
- Action: ⚠️ **SUPPRESS** repeated suggestion
- Reset: On topic change or explicit command
- Applies to: Suggestion Pipeline only

## Emergency Override (Recovery Pipeline Only)

**Critical Operation Override**:
- Check: Emergency pattern detected
- Action: ⚦ **BYPASS** all standard gates
- Message: "⚠️ Emergency mode: Bypassing validation for immediate response"
- Applies to: Recovery Pipeline only

## Gate Invocation Examples

```
Command Pipeline Example:
Input: "/spec:design"
Gates Applied: All document validation gates
Result: May block if requirements/investigation incomplete

Suggestion Pipeline Example:
Input: "Users need login" (implicit)
Gates Applied: Confidence gate only
Result: Suggest if confidence > 0.7, no document validation

Diagnostic Pipeline Example:
Input: "What's the status?"
Gates Applied: None
Result: Direct read and report

Recovery Pipeline Example:
Input: "Error: can't create design"
Gates Applied: Emergency override
Result: Bypass all gates, immediate assistance

Timeline Planning Example:
Input: "/spec:timeline"
Gates Applied: Timeline Planning without Design
Result: Warning if no design.md, but proceed if user provides details
```

## Key Principles

- **Pipeline-Specific**: Each pipeline has its own set of applicable gates
- **Efficiency**: Lightweight pipelines skip heavy validation
- **Safety**: Command pipeline maintains full validation
- **Flexibility**: Emergency situations can bypass normal gates
- **Isolation**: Gates only invoked when specified by pipeline strategy