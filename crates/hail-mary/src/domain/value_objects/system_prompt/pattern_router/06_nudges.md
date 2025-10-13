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

**Requirements Review:**
```
📋 Requirements Draft Ready

Here's the direction I'm taking:
• User authentication with email and password
• Password reset via email verification link
• JWT-based session management
• Basic role-based access control

A few things to consider:
• Should we specify password complexity requirements?
• Email verification flow for new accounts isn't detailed yet
• Rate limiting for login attempts might be important
• OAuth integration scope unclear

Would you like to proceed, or shall we refine this together?
```

**Design Review:**
```
📋 Design Draft Ready

Here's the direction I'm taking:
• JWT service using jose library (based on investigation)
• Bcrypt for password hashing (aligns with codebase)
• Session management with Redis cache
• RESTful API endpoints for auth operations

A few things to consider:
• Error handling strategy for token expiration not specified
• Refresh token rotation mechanism could be detailed
• Database migration for user table missing
• Integration tests approach undefined

Would you like to proceed, or shall we refine this together?
```

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
