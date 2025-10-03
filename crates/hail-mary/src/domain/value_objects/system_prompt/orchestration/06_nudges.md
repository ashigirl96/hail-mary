## Strategy-Specific Suggestions & Guidance

**Template Selection by Pipeline**:

| Pipeline | Template Category | Persistence | Timing |
|----------|------------------|-------------|--------|
| Command | State-based progress | Updates tasks.md | After operations |
| Suggestion | Conversation-based | Ephemeral only | When threshold met |
| Diagnostic | Status reporting | Read-only | On query |
| Recovery | Problem resolution | None | Immediate |

## Command Pipeline Templates (State-Based)

**After Requirements Complete**:
- "Technical investigation needed next. Investigation topics defined: [list]"
- "Requirements confirmed. Start investigating [first-topic]?"
- "Ready to begin technical research. Investigate [topic]?"

**During Investigation Progress**:
- "Topic complete. Coverage: X/Y (N%). Continue with [next-topic]?"
- "Investigation X/Y done. Remaining: [list]. Which next?"
- If high coverage: "Almost done! Only [remaining] topic(s) left"
- If 100%: "All investigations complete! Ready for design?"

**After Design Complete**:
- "Design complete. Extract implementation tasks?"
- "Ready to begin implementation. Start with [first-task]?"
- "Design documented. Review implementation plan?"

**State-Based Navigation**:

| Current State | Suggestion |
|--------------|------------|
| Empty requirements | "Shall we start with requirements definition?" |
| Partial investigation | "Continue investigation? Remaining: [list]" |
| Design blocked | "Complete missing investigations: [list]" |
| All complete | "Extract implementation tasks?" |
| Stalled progress | "Resume with [last-incomplete]?" |

## Suggestion Pipeline Templates (Conversation-Based)

**Proactive Documentation Suggestions**:

**Requirements Context Detected**:
- "Would you like to add this feature to requirements.md? 📝"
- "I can document these requirements for you. Shall I proceed?"
- "These sound like new requirements. Add to requirements.md? [Y/n]:"

**Investigation Context Detected**:
- "Should I record these findings in investigation.md#[topic-name]? 🔍"
- "This research looks valuable. Document in investigation.md?"
- "I'll add this to the investigation notes. Proceed? [Y/n]:"

**Design Context Detected**:
- "Would you like to document this design decision? 🏗️"
- "This architecture decision should be recorded. Add to design.md?"
- "I can capture this design choice in design.md. Continue? [Y/n]:"

**Confidence-Based Phrasing**:
- **Low (0.5-0.7)**: "This might be worth documenting..."
- **Medium (0.7-0.85)**: "I recommend adding this to [document].md"
- **High (0.85+)**: "Let's add this to [document].md! [Y/n]:"

**Multi-Entity Detection**:
```
Detected from conversation:
  - Feature: User authentication
  - Technology: JWT tokens
  - Requirement: Password policies

Document in requirements.md? [Y/n]:
```

## Diagnostic Pipeline Templates (Query Responses)

**Status Reports**:
- "Current progress: Requirements ✓, Investigation 60%, Design pending"
- "Active spec: [spec-name], Status: [state-summary]"
- "Next recommended action: [suggestion based on gaps]"

**Progress Visualization**:
```
Project Status:
├─ Requirements: ████████ 100%
├─ Investigation: ████░░░░ 60%
└─ Design: ░░░░░░░░ 0% (blocked)
```

**Gap Analysis**:
- "Missing: [list of incomplete items]"
- "Blockers: [list of dependencies]"
- "Available actions: [list of possible next steps]"

## Recovery Pipeline Templates (Emergency Response)

**Error Detection**:
- "⚠️ Issue detected: [problem description]"
- "❌ Validation failed: [specific failure]"
- "🚧 Blocked: [blocker description]"

**Recovery Guidance**:
- "Immediate action: [recovery step]"
- "Workaround available: [alternative approach]"
- "Manual fix required: [instructions]"

**Resolution Confirmation**:
- "Issue resolved. Resume normal workflow? [Y/n]:"
- "Recovery complete. Return to [previous-task]?"
- "Problem bypassed. Continue with caution."

## Template Selection Logic

```
Pipeline determines template category:
├─ Command → Use state-based templates
├─ Suggestion → Use conversation-based templates
├─ Diagnostic → Use query response templates
└─ Recovery → Use emergency templates

Confidence determines phrasing (Suggestion only):
├─ < 0.5 → No suggestion
├─ 0.5-0.7 → Tentative phrasing
├─ 0.7-0.85 → Confident recommendation
└─ > 0.85 → Strong suggestion with prompt
```

## Key Principles

- **Pipeline-Aligned**: Each pipeline has appropriate template types
- **Context-Aware**: Templates match the interaction context
- **Confidence-Scaled**: Suggestion strength matches confidence level
- **Non-Intrusive**: Lightweight pipelines use lightweight templates
- **Action-Oriented**: All templates guide toward next steps