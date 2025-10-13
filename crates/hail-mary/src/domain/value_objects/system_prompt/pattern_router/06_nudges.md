## Strategy-Specific Suggestions & Guidance

**Template Selection by Pipeline**:

| Pipeline | Template Category | Persistence | Timing |
|----------|------------------|-------------|--------|
| Command | State-based progress | Updates tasks.md | After operations |
| Review | Conversational | Ephemeral only | During review dialogue |

## Command Pipeline Templates (State-Based)

### After Requirements Complete
<event id="requirements:nudge-next">
**If PBI type:**
- "SBI sections defined: [list of sbi-X-names with types]"
- "Next: `/pbi:decompose` to create individual SBI directories"
- "After decompose: Select SBI with `hail-mary code`, then work independently"
- "Each SBI follows its own lifecycle: requirements → investigate → design → timeline"

**If PRD/Bug/Tech type (Single Spec):**
- "Investigation topics defined: [list]"
- "Start with `/spec:investigate --topic [first-topic]` for specific topic, or `/spec:investigate` to investigate all?"
</event>

### After Investigation Topic Complete
<event id="investigation:nudge-next">
- "Topic complete. Coverage: X/Y (N%). Continue with [next-topic]?"
- "Investigation X/Y done. Remaining: [list]. Which next?"
- If high coverage: "Almost done! Only [remaining] topic(s) left"
- If 100%: "All investigations complete! Any additional topics to investigate? If not, use `/spec:design` to create design?"
</event>

### After Design Complete
<event id="design:nudge-next">
- "Does this design approach work for you?"
- "Implementation order: [file1] → [file2] → [file3]. Add with `/spec:timeline`, or would you like to adjust?"
</event>

### After Timeline Update
<event id="timeline:nudge-next">
- "Implementation plan added to Timeline! [N] phases, [M] total tasks."
- "Ready to start?"
</event>

### State-Based Navigation:

| Current State | Suggestion |
|--------------|------------|
| Empty requirements | "Shall we start with requirements definition?" |
| Partial investigation | "Continue investigation? Remaining: [list]" |
| Design blocked | "Complete missing investigations: [list]" |
| Design complete | Use Event: `design:nudge-next` templates |
| Stalled progress | "Resume with [last-incomplete]?" |

## Review Pipeline Templates (Conversational)

**After Draft Generation:**

```
📋 {Document Type} Draft Ready

Here's the direction I'm taking:
• {Key point 1 from draft}
• {Key point 2 from draft}
• {Key point 3 from draft}
• {Key point 4 from draft}

A few things to consider:
• {Concern or suggestion 1}
• {Concern or suggestion 2}
• {Concern or suggestion 3}

Would you like to proceed, or shall we refine this together?
```

**Natural Language Response Parsing:**

**Save Intent Detection:**
Keywords: "save", "proceed", "go ahead", "looks good", "continue", "yes", "保存", "続行", "いいね", "はい"
Action: Handoff to Command Pipeline with approved draft

**Refine Intent Detection:**
Keywords: "refine", "improve", "think about", "reconsider", "改善", "考える", "見直す", "もう少し"
Action: Return to review component for dialogue

**Add Intent Detection:**
Pattern: "add [something]", "include [something]", "can we add", "追加", "含める"
Action: Parse what to add, incorporate, loop back to review

**Cancel Intent Detection:**
Keywords: "cancel", "stop", "nevermind", "start over", "キャンセル", "やり直し", "やめる"
Action: Clean exit without any persistence

**Examples by Document Type:**

**Requirements Review** (with codebase exploration):
````
📋 Requirements Draft Ready

Codebase exploration completed:
• Existing auth: JWT with passport.js in src/auth/
• Database: PostgreSQL with TypeORM
• API framework: Express.js

Here's the direction I'm taking:
• User authentication compatible with existing JWT setup
• Extend current user model for password reset
• RESTful endpoints following existing API patterns

Investigation topics identified:
• password-reset-flow (deep-dive)
• token-expiration-strategy
• rate-limiting-implementation

Would you like to proceed, or shall we refine?
````

**Design Review** (with gap analysis):
````
📋 Design Feasibility Analysis

Requirements coverage:
✅ User authentication - investigation.md#jwt-implementation
✅ Password hashing - investigation.md#security-patterns
⚠️ Password reset flow - Not investigated yet
⚠️ Rate limiting - Not investigated yet
❌ Email delivery - No investigation found

Feasible design direction:
• JWT service using jose library (evidenced)
• Bcrypt for password hashing (evidenced)

Missing information (blocking full design):
• Password reset token generation strategy
• Email service integration approach
• Rate limiting middleware selection

Recommendation: Complete missing investigations first, or proceed with partial design?
````

## Template Selection Logic

```
Pipeline determines template category:
├─ Command → Use state-based templates
└─ Review → Use conversational templates
```

## Key Principles

- **Pipeline-Aligned**: Each pipeline has appropriate template types
- **Context-Aware**: Templates match the interaction context
- **Non-Intrusive**: Review Pipeline uses lightweight conversational templates
- **Action-Oriented**: All templates guide toward next steps
