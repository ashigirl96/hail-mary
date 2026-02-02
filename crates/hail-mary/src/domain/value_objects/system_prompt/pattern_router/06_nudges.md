## Strategy-Specific Suggestions & Guidance

**Template Selection by Pipeline**:

| Pipeline | Template Category | Persistence | Timing |
|----------|------------------|-------------|--------|
| Command | State-based progress | Updates tasks.md | After operations |
| Review | Conversational | Ephemeral only | During review dialogue |
| Brainstorm | Exploratory dialogue | Updates brainstorming.md | During/After exploration |

## Command Pipeline Templates (State-Based)

### After Requirements Complete
<event id="requirements:nudge-next">
**If PBI type:**
- "SBI sections defined: [list of sbi-X-names with types]"
- "Next: `/pbi:decompose` to create individual SBI directories"
- "After decompose: Select SBI with `hail-mary code`, then work independently"
- "Each SBI follows its own lifecycle: requirements → investigate → design → timeline"

**If PRD/Bug/Tech type (Single Spec):**
- "Requirements complete. Ready to plan implementation with `/spec:timeline`?"
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
| Requirements complete | "Ready to plan implementation with `/spec:timeline`?" |
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

## Brainstorm Pipeline Templates (Exploratory Dialogue)

### During Conversation
<event id="brainstorm:nudge-conversation">
- "What problem does this feature solve for users?"
- "Who are your target users and their main workflows?"
- "What's your expected user volume and performance needs?"
- "Any existing systems to integrate with?"
- "What similar services could serve as references?"
- "What specific challenges are users facing?"
- "Current vs desired user experience?"
- "Security requirements and compliance needs?"
- "Timeline and resource constraints?"
</event>

### Before Save
<event id="brainstorm:nudge-save">
```
📝 Discussion converged

Issues identified:
• [Extracted issue 1]
• [Extracted issue 2]

Solutions explored:
• Option 1: [Approach 1]
• Option 2: [Approach 2]

Concerns raised:
• [Concern 1]

Save to brainstorming.md?
```
</event>

### After Save
<event id="brainstorm:nudge-next">
```
✅ Saved to brainstorming.md

During our discussion, these topics were identified:
• [topic-1] (Priority: High)
• [topic-2] (Priority: Medium)

Continue brainstorming with `/spec:brainstorm --topic [topic-name]` or start development with `/spec:requirements`?
```
</event>

## Template Selection Logic

```
Pipeline determines template category:
├─ Command → Use state-based templates
├─ Review → Use conversational templates
└─ Brainstorm → Use exploratory dialogue templates
```

## Key Principles

- **Pipeline-Aligned**: Each pipeline has appropriate template types
- **Context-Aware**: Templates match the interaction context
- **Non-Intrusive**: Review Pipeline uses lightweight conversational templates
- **Action-Oriented**: All templates guide toward next steps
