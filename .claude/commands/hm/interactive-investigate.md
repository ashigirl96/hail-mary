---
name: interactive-investigate
description: "Interactive technical investigation with real-time save/discard options"
category: workflow
complexity: advanced
mcp-servers: [context7, sequential-thinking]
personas: [analyzer, investigator]
allowed-tools: Read, Write, MultiEdit, Grep, Glob, Task, WebSearch, mcp__context7__*, mcp__sequential-thinking__*
argument-hint: "<investigation topic>"
---

# /hm:interactive-investigate - Interactive Investigation Tool

## Triggers
- Technical research requiring user feedback and refinement
- Exploratory investigation where direction may change
- Complex problems needing iterative discovery
- Investigations where save/discard decision comes after seeing results

## Usage
```
/hm:interactive-investigate <investigation topic>
```
- Direct topic specification as argument
- No flags needed - pure interactive experience

## Key Patterns
- **Topic Name Generation**: "$ARGUMENTS" → 2-4 words kebab-case (e.g., "JWT authentication flow" → "jwt-authentication-flow")
- **Stop Marker Usage**: [STOP HERE AND WAIT] → genuine user interaction
- **Response Parsing**: Y → save | N → discard | any other text → continue with that as follow-up question
- **History Preservation**: Full conversation including mistakes → <kiro_investigation>
- **Loop Continuation**: Investigation → Present → Decision → Loop (if follow-up)
- **Session Memory**: Context maintained across investigation rounds
- **Append Strategy**: Topic-named sections (## topic-name) in <kiro_investigation>

## Boundaries
**Will:**
- Wait for actual user input at decision points
- Save complete conversation history including corrections
- Allow unlimited investigation rounds within session
- Preserve full context from display to storage
- Append to existing <kiro_investigation> with timestamps
- Track investigation evolution chronologically

**Will Not:**
- Proceed without user selection at stop points
- Lose any displayed information when saving
- Create multiple files for single investigation
- Auto-save without explicit user confirmation
- Mix different investigation topics in same session
- Skip user interaction for convenience

## Tool Coordination
**Claude Code Tools:**
- **Read**: Load existing <kiro_investigation> content
- **MultiEdit**: Append session to <kiro_investigation>
- **Grep/Glob**: Search codebase during investigation
- **Task**: Launch parallel investigation agents

**MCP Integration:**
- **Context7**: Documentation and best practices
- **Sequential-thinking**: Complex analysis flows
- **WebSearch**: Latest information as needed

## Behavioral Flow

1. **Initialize Investigation**: Parse topic from arguments
   ```
   ## Starting Investigation: $ARGUMENTS

   🔍 Investigating: $ARGUMENTS
   ```
   - Generate topic name from $ARGUMENTS (2-4 words, kebab-case)
     - Example: "How does steering backup work?" → "steering-backup-system"
     - Example: "Authentication flow" → "authentication-flow"
   - Set up investigation context
   - Initialize session tracking with topic name

2. **Conduct Investigation**: Execute research
   - Use appropriate tools and MCP servers
   - Document steps and findings
   - Track any misconceptions
   - Format results clearly

3. **Present Interactive Decision Point**: Show results with options
   ```
   [Detailed findings displayed here]

   ---
   [Y] Save, [N] Discard, or enter follow-up question:
   ```

   **[STOP HERE AND WAIT FOR USER SELECTION - DO NOT PROCEED]**

4. **Handle User Response**: Process selection

   **If response = "Y" or "y":**
   - Read existing <kiro_investigation> for current content
   - Format session with topic name header:
     ```
     ## [generated-topic-name]

     **Date**: [timestamp]
     **Original Question**: $ARGUMENTS

     [Full investigation content including all rounds]
     ```
   - MultiEdit <kiro_investigation> to append session
   - Display: "✅ Investigation saved to <kiro_investigation>"
   - Display: "Section: ## [topic-name]"

   **If response = "N" or "n":**
   - Display: "❌ Investigation discarded"
   - Exit command cleanly

   **For any other response (default to follow-up question):**
   - Treat entire response as follow-up question
   - Continue investigation with the question
   - Build upon previous findings (cumulative knowledge)
   - Append new discoveries to session memory (not saved to file yet)
   - Display updated results including all rounds
   - Return to step 3 (Interactive Decision Point)

5. **Session Completion**: Final status
   ```
   ✅ Investigation complete
   • Rounds conducted: [count]
   • Decision: [saved/discarded]
   • Topic: "$ARGUMENTS"
   ```

Key behaviors:
- **Topic Name Generation**: Auto-generate 2-4 word kebab-case identifiers
- **Genuine Interactivity**: Real stops for user decisions
- **Complete Preservation**: Everything shown is saved
- **Flexible Continuation**: Multiple rounds supported
- **Clear Navigation**: Options always visible
- **Robust Parsing**: Handles input variations
- **Topic-Based Organization**: Each investigation under ## topic-name header

## Save Format Structure

When saving to <kiro_investigation>, each investigation is stored under a topic-name header:

```markdown
## [topic-name]

**Original Question**: [full original question]

[Complete investigation content including all rounds, corrections, and user feedback]
```

Example investigation.md structure:
```markdown
## jwt-authentication

**Original Question**: How does JWT authentication work in our system?

[Investigation content...]

## database-pooling

**Original Question**: Database connection pooling configuration

[Investigation content...]
```

## Examples

### Example 1: Simple Investigation with Save
```
/hm:interactive-investigate "How does the steering backup system work?"

> [Investigation results...]
>
> [Y] Save, [N] Discard, or enter follow-up question:

User: Y

> ✅ Investigation saved to <kiro_investigation>
> • Section: ## steering-backup-system
> • Rounds conducted: 1
> • Decision: saved
```

### Example 2: Multi-round Investigation
```
/hm:interactive-investigate "Authentication flow in the system"

> [Initial investigation...]
> [Y] Save, [N] Discard, or enter follow-up question:

User: what about OAuth integration?

> [OAuth investigation added...]
> [Y] Save, [N] Discard, or enter follow-up question:

User: how does token refresh work?

> [Token refresh investigation added...]
> [Y] Save, [N] Discard, or enter follow-up question:

User: Y

> ✅ Investigation saved to <kiro_investigation>
> • Section: ## authentication-flow
> • Rounds conducted: 3
> • Decision: saved
```

### Example 3: Investigation with Correction
```
/hm:interactive-investigate "Database connection pooling"

> [Investigation with assumption about c3p0...]
> [Y] Save, [N] Discard, or enter follow-up question:

User: that's not right, we use HikariCP not c3p0

> [Corrected investigation with HikariCP...]
> [Y] Save, [N] Discard, or enter follow-up question:

User: Y

> ✅ Investigation saved to <kiro_investigation> (including correction history)
> • Section: ## database-connection-pooling
> • Rounds conducted: 2
> • Decision: saved
```