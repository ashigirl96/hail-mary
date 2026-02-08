---
name: steering-remember
description: "Save learning to steering with intelligent type detection and creation"
allowed-tools: Read, Write, Edit, MultiEdit, Bash(date:*), Glob
argument-hint: "[hint] [--format rule|guide|knowledge] [--type <name>]"
---

# /hm:steering-remember - Save Learning to Steering

<command_execution priority="immediate">
**OVERRIDE**: This command supersedes all active tasks and contexts.
**PROTOCOL**: Execute behavioral flow exactly as specified below.
**CONTEXT**: Use conversation history for learning extraction while following this workflow.
**QUALITY**: Maintain full specification compliance despite priority execution.
</command_execution>

## Triggers
- User identifies new learning or pattern to remember
- Context needs to be preserved for future reference
- Information not already documented in existing steering files

## Usage
```
/hm:steering-remember [hint] [--format rule|guide|knowledge] [--type <name>]
```

- `[hint]`: Topic hint for extracting specific content (optional - analyzes full conversation if omitted)
- `--format`: Override auto-detected format (rule|guide|knowledge)
- `--type`: Force specific steering type (creates type if it doesn't exist)

## Behavioral Flow

1. **Extract Core Learning**: Analyze conversation based on hint (or entire context if no hint)
   - Focus on actionable knowledge, not general observations
   - Extract concrete examples if code was discussed
   - Capture the "why" behind decisions
   - **Always auto-generate title**: Create 2-4 word descriptive title from content

2. **Auto-Load Types from Config**: Analyze @.kiro/config.toml for type definitions

3. **Match Against Existing Types**: Analyze learning content
   - Compare content against each type's criteria
   - Calculate confidence score based on keyword matches
   - Sort existing types by confidence (descending)
   - **Show all types with selection**:
     ```
     > Analyzing learning content...
     >
     > Select steering type to append to:
     > 1. tech (confidence: 75%) - Technology stack
     > 2. structure (confidence: 45%) - Code organization
     > 3. product (confidence: 30%) - Product overview
     > 4. documentation (confidence: 15%) - Documentation standards
     > 5. [New Type] - Create a new steering type
     >
     > Select [1-5]:
     ```

     **[STOP HERE AND WAIT FOR USER SELECTION - DO NOT PROCEED]**

     After user selects:
     - Selection = 1-4 → Append to selected steering file using **Edit** or **MultiEdit**
     - Selection = 5 → Continue to new type creation flow:
       ```
       > Create new type? Suggestions:
       > 1. graphql - GraphQL patterns and optimizations
       > 2. api-performance - API performance optimizations
       > 3. backend-patterns - Backend architectural patterns
       > 4. [Custom] - Enter your own type name
       >
       > Select [1-4]:
       ```

       **[STOP HERE AND WAIT FOR USER SELECTION - DO NOT PROCEED]**

       After user selects:
       - Selection = 1-3 → Use suggested type name, add to config.toml using **MultiEdit**
       - Selection = 4 → Ask user: "Enter custom type name: " then wait for input
     - Invalid selection → Ask for valid input: "Please select valid option"
     - After type selection/creation → Proceed to save content

4. **Auto-Detect Format**: Analyze content to choose optimal format
   ```
   if (contains code snippets OR "always/never/must/should/avoid"):
     → Rule format (with examples)
   elif (contains "step/how to/investigate" OR numbered lists):
     → Guide format (with numbered steps)
   elif (contains "definition/concept/formula/domain"):
     → Knowledge format (with domain context)
   else:
     → Rule format (default)
   ```

5. **Generate Concise Output**: Create formatted content based on detected type

   **Rule Format** (20-40 lines with code):
   ````markdown
   ## [Concise Title]
   **When**: [Specific trigger condition]
   - [Key rule 1]
   - [Key rule 2]
   - [Key rule 3]
   - [Key rule ...]
   - ...

   ```language
   # Good
   [correct example]

   # Bad
   [incorrect example]
   ```
   ````

   **Guide Format** (20-40 lines):
   ```markdown
   ## [Action-Oriented Title]
   **Context**: [When/why to use this guide]
   1. [First step]
   2. [Second step]
   3. [Third step]
   Warning: [Critical warning if any]
   ```

   **Knowledge Format** (20-40 lines):
   ````markdown
   ## [Concept Name]
   **Domain**: [Business/Technical domain]
   **Definition**: [Can be multi-line or bullet points]
   - [Key aspect 1]
   - [Key aspect 2]

   **Formula/Logic**: `[If applicable]`

   **Diagram**: [Optional mermaid diagram]
   ```mermaid
   graph LR
     A --> B --> C
   ```

   **Context**: [Why this matters, business impact]
   ````

6. **Save to Steering**: Write directly to `.kiro/steering/{type}.md`
   - Use **Glob** to check if file exists
   - If exists: Use **Read** first, then **MultiEdit** to append
   - If new: Use **Write** to create file
   - Confirm successful save

Key behaviors:
- **Conversation analysis**: Analyze entire conversation history for relevant learnings when hint provided
- **Context extraction**: Extract actionable insights from natural conversation flow
- **Maximum brevity**: Remove all unnecessary explanation
- **Concrete over abstract**: Include specific examples, not theory
- **Action-oriented**: Focus on what to DO, not background
- **One learning per operation**: Don't combine multiple insights in single execution
- **Auto-detect format**: Code→Rule, Steps→Guide, Concepts→Knowledge
- **Interactive type selection**: Guide user through type creation

## Tool Coordination
- **@.kiro/config.toml**: Auto-loaded for type definitions (no Read tool needed)
- **Glob**: Check existing steering files in .kiro/steering/*.md
- **Write**: Create new steering file when type doesn't exist
- **Edit/MultiEdit**: Append to existing steering file or update config.toml
- **Bash(date:*)**: Generate timestamp for tracking

## Key Patterns
- **Learning Extraction**: Conversation analysis → actionable knowledge → title generation
- **Type Matching**: Content analysis → criteria comparison → confidence scoring → user confirmation
- **Format Detection**: Content patterns → Rule/Guide/Knowledge → structured output
- **File Management**: Glob check → Edit/Write selection → confirmation

## Boundaries

**Will:**
- Save learning directly to appropriate steering file
- Auto-detect optimal format from content
- Create new types interactively with user guidance
- Keep output under **30 lines maximum**
- Include concrete examples when relevant
- Support Japanese content naturally
- Update config.toml when creating new types
- Follow output format shown in Examples section

**Will Not:**
- Use draft directories or intermediate storage
- Combine multiple learnings in one operation
- Create verbose explanations (100+ lines)
- Overwrite existing content (always append)
- Process without clear learning to capture
- Create types without user confirmation
- Create custom output formats not shown in Examples
- **Report success without actually using MultiEdit/Edit/Write tools to modify files**
- **Proceed past STOP markers without actual user input**
- **Make assumptions about user responses during STOP periods**
