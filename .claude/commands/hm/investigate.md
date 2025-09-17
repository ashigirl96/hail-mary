---
name: investigate
description: "Comprehensive technical investigation with multi-source research and interactive refinement"
category: workflow
complexity: advanced
mcp-servers: [context7, sequential-thinking]
personas: [analyzer, architect]
allowed-tools: Read, Write, MultiEdit, Grep, Glob, Task, WebSearch, mcp__context7__*, mcp__sequential-thinking__*
argument-hint: "[--topic [name]] [--for requirements|design]"
---

# /hm:investigate - Technical Investigation Tool

## Triggers
- Technical research needed for requirements or design documents
- Deep dive into specific technical areas or problems
- Codebase exploration for implementation patterns
- Architecture and design decision investigation

## Usage
```
/hm:investigate [--topic <name>] [--for requirements|design]
```
- `--topic <name>`: Resume/update existing topic by name
- `--for`: Link investigation to <kiro_requirements_path> or <kiro_design_path>

## Key Patterns
- **Topic Analysis**: User input → title generation → scope determination
- **Steering Guidance**: Embedded `<steering>` → investigation focus → targeted search patterns
- **Depth Detection**: Simple question → standard depth | Complex/multi-system → deep investigation
- **Source Priority**: codebase search → Context7 docs → web (steering guides but not evidence)
- **Confidence Scoring**: Source trust × content match × recency = confidence level
- **Interactive Loop**: Investigate → Present → Refine → Document

## Boundaries
**Will:**
- Read steering information from embedded `<steering>` tag in Claude Code context
- Use steering patterns to guide and focus investigation approach
- Create new investigation section when no `--topic` flag provided
- Resume/update existing topic section when `--topic <name>` matches existing topic
- Link findings to <kiro_requirements_path> or <kiro_design_path> when `--for` flag present
- Think in English, but document in Japanese

**Will Not:**
- Read `.kiro/steering/*.md` files directly to retrieve steering information
- Ignore steering patterns and conduct unfocused investigation
- Create new section when `--topic <name>` is provided (always resume/update)
- Override automatic source prioritization (no manual source selection)
- Replace existing investigation sections (always append or update)
- Mix different topics in the same section
- Generate speculative technical details without evidence
- Continue to new topic within same command invocation
- Skip source verification or confidence scoring

## Tool Coordination
**Claude Code Tools:**
- **Read**: Load existing <kiro_investigation_path> for continuity
- **Write/MultiEdit**: Save investigation findings progressively
- **Grep/Glob**: Search codebase for patterns and implementations
- **Task**: Spawn parallel investigation agents

**MCP Integration:**
- **Context7**: Official documentation and best practices lookup
- **Sequential-thinking**: Complex analysis and systematic investigation
- **WebSearch**: Fallback for latest information and community solutions

## Behavioral Flow

1. **Initialize & Topic Gathering**: Parse arguments, load context, and determine investigation topic
   - Read existing <kiro_investigation_path>

   - **If `--for requirements` or `--for design` provided**:
     - Read corresponding <kiro_requirements_path> or <kiro_design_path>
     - Analyze document for incomplete technical sections (marked [TBD] or pending investigation)
     - Display: "📋 Found {document}.md with technical gaps"
     - Show list of sections needing investigation with brief descriptions
     - Suggest: "🔍 Suggested investigations based on {document}.md:"
       * List 2-3 specific technical areas that would complete the document
       * Include why each investigation would be valuable
     - Ask: "What would you like to investigate? (You can choose from suggestions above or provide your own topic):"

   - **If `--topic <name>` provided**:
     - Search existing <kiro_investigation_path> for matching topic section
     - Load and analyze previous investigation content
     - Display: "📋 Found existing investigation for '{topic}' (Section #{n})"
     - Show summary of what has been investigated so far
     - Suggest: "🔍 Additional investigations to deepen '{topic}':"
       * List 2-3 specific follow-up questions or unexplored areas
       * Include potential impact of each investigation
     - Ask: "What would you like to investigate? (You can explore suggestions above or provide your own focus):"

   - **If no flags provided**:
     - Ask: "🔍 What would you like to investigate?"
     - Sub-prompt: "[Provide specific technical question or area to explore]"

   **[STOP HERE AND WAIT FOR USER INPUT - DO NOT PROCEED]**

   - After user responds:
     - If `--topic <name>` provided: Continue with existing topic name
     - If no `--topic`: Auto-generate concise title (2-4 words) in English kebab-case from user input
     - Prepare for investigation phase

2. **Parallel Investigation**: Launch Task agents with plan display
   ```
   > 🚀 Investigation Plan for "[Topic]":
   >
   > Using steering patterns to guide investigation focus
   >
   > Launching parallel investigators:
   > • [Code Explorer] Search implementation in codebase
   > • [Docs Researcher] Query Context7 for best practices
   > • [Web Searcher] Find recent solutions and updates
   >
   > Priority: codebase > Context7 > web
   ```

   - Execute parallel Task agents
   - Aggregate findings with source priority
   - Calculate confidence scores

3. **Progressive Documentation**: Save findings immediately
   - **Document update strategy**:
     - If `--topic <name>` provided: Append to existing section (継続調査)
     - If no `--topic` flag: Create new section with auto-generated title (新規調査)
   - Write to <kiro_investigation_path>
   - Display save confirmation

   ```
   > 📝 Investigation saved to <kiro_investigation_path>
   > Topic: "[Title]" (Section #[n])
   > Mode: [Updated existing section | Created new section]
   > Confidence: [level] ([percentage]%)
   ```

4. **Interactive Continuation**: Topic refinement loop
   ```
   > 🔄 Continue investigating "[Topic]"?
   > - [Y/Enter]: Deepen current topic
   > - [n/done]: Finish investigation
   >
   > Or provide specific follow-up question:
   ```

   **[STOP HERE AND WAIT FOR USER RESPONSE - DO NOT PROCEED]**

   - Y/follow-up → Update same section with new findings (return to step 2)
   - n/done → Proceed to step 5

5. **Finalization**: Link to other documents if `--for` present
   - If `--for` requirements: Extract relevant → Update <kiro_requirements_path>
   - If `--for` design: Extract architectural → Update <kiro_design_path>

   ```
   > ✅ Investigation complete
   > • Topics investigated: [count]
   > • Overall confidence: [level] ([percentage]%)
   > [if --for] • Updated: [document].md
   ```

Key behaviors:
- **Steering as Guide**: Use embedded `<steering>` to focus investigation, not as evidence source
- **Source Priority**: Code > docs > web for evidence
- **Parallel Execution**: Multiple Task agents investigate simultaneously
- **Progressive Save**: Write to <kiro_investigation_path> after each round, not just at end
- **Session Scope**: Each command invocation handles one topic (with deepening)
- **Topic Management**: `--topic <name>` resumes existing, no flag creates new
- **Section Management**: Same topic updates section, new command creates new section
- **Confidence Tracking**: Calculate and display trust level for all findings
- **History Preservation**: Maintain investigation notes and corrections within topic
