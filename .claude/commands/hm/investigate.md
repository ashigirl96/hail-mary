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
- **Depth Detection**: Simple question → direct investigation | Complex/multi-system → parallel Task agents
- **Interactive Loop**: Plan → Investigate → Present → Refine → Document
- **Agent Selection**: Topic complexity → agent specialization → parallel execution strategy

## Boundaries
**Will:**
- Read steering information from embedded `<steering>` tag in Claude Code context
- Use steering patterns to guide and focus investigation approach
- Create new investigation section when no `--topic` flag provided
- Resume/update existing topic section when `--topic <name>` matches existing topic
- Link findings to <kiro_requirements_path> or <kiro_design_path> when `--for` flag present

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

2. **Investigation Planning & Approval**: Analyze approach and get user confirmation
   - Think about investigation strategy based on topic and context
   - Determine optimal agent configuration and specialization
   - Present investigation plan:

   ```
   > 🔍 Investigation Strategy for "[Topic]":
   >
   > **Approach**: [Describe investigation coordination strategy]
   > **Agent Configuration**: [N] specialized agents for parallel execution
   >
   > **Parallel Investigation Agents**:
   > • [Agent 1: root-cause-investigator] - Evidence-based systematic investigation
   >   Mission: Apply systematic investigation methodology with source prioritization
   > • [Agent 2: analyzer] - Codebase pattern analysis
   >   Mission: Search implementation patterns, identify correlations
   > • [Agent 3: architect] - Architecture and design investigation
   >   Mission: Dependency analysis, scalability assessment
   > [Additional agents based on topic: security-engineer, performance-engineer, etc.]
   >
   > **Execution**: All agents will run **concurrently** for efficiency
   >
   > Proceed with this investigation plan? [Y/n]:
   ```

   **[STOP HERE AND WAIT FOR USER RESPONSE - DO NOT PROCEED]**

   - After user responds:
     - Response = "Y" or "y" or Enter → Execute parallel Task agents
     - Response = "n" or custom requirements → Rethink strategy based on feedback:
       * Analyze user's alternative approach
       * Reconfigure agent specialization
       * Present revised plan (return to start of step 2)
     - Invalid response → Ask: "Please enter Y to proceed or n to revise the plan"

   - **Execute Parallel Investigation**: Launch approved Task agents
     - **[The implementation will send multiple Task tool calls in one response]**
     - Each agent operates **independently** with its own investigation context
     - Agents process **concurrently** without dependencies

   - Aggregate findings with source priority and confidence scoring

3. **Progressive Documentation**: Review and save findings with user confirmation
   - **Document update strategy**:
     - If `--topic <name>` provided: Append to existing section (continue investigation)
     - If no `--topic` flag: Create new section with auto-generated title (new investigation)

   - Present findings for review:

   ```
   > 📝 Investigation Results for "[Topic]":
   >
   > **Summary**: [Brief overview of key findings]
   > **Confidence**: [level] ([percentage]%)
   > **Sources**: [Number] codebase references, [Number] docs, [Number] web sources
   >
   > --- Content to Save ---
   > ## [Topic Title]
   > **Confidence**: [percentage]%
   >
   > [Investigation findings formatted in markdown]
   > [Key discoveries, patterns, implementations]
   > [Code examples if relevant]
   > [Recommendations or next steps]
   > --- End of Content ---
   >
   > Save this investigation to <kiro_investigation_path>? [Y/n]:
   ```

   **[STOP HERE AND WAIT FOR USER RESPONSE - DO NOT PROCEED]**

   - After user responds:
     - Response = "Y"  → Write to <kiro_investigation_path>
       * Display: "✅ Investigation saved (Section #[n], Mode: [new|append])"
     - Response = "n" or additional requirements → Refine content:
       * Examples: "add performance metrics", "include error handling", "simplify format"
       * Reformat/enhance based on feedback
       * Present updated content (return to start of step 3)
     - Invalid response → Ask: "Please enter Y to save or provide refinement instructions"

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
- **Steering as Guide**: Use embedded `<steering>` to focus investigation direction
- **Task Agent Usage**:
  - **Simple investigations**: Direct investigation without Task agents (e.g., "What does this function do?" → use Read/Grep directly)
  - **Complex investigations**: Use parallel Task agents with specialized subagents (e.g., multi-component analysis, root cause investigation → spawn multiple agents)
- **Parallel Execution**: Multiple Task agents investigate simultaneously using specialized subagents:
  ```
  Task(
      subagent_type="root-cause-investigator",
      description="Systematic evidence-based investigation",
      prompt=f"""
      Topic: {topic_name}
      Scope: {investigation_scope}
      Context: {existing_findings}

      Your mission: Investigate systematically with evidence-based analysis
      Focus: {specific_focus_area}
      """
  )
  ```
- **Progressive Save**: Write to <kiro_investigation_path> after each round, not just at end
- **Session Scope**: Each command invocation handles one topic (with deepening)
- **Topic Management**: `--topic <name>` resumes existing, no flag creates new
- **Section Management**: Same topic updates section, new command creates new section
- **Confidence Tracking**: Calculate and display trust level for all findings
- **History Preservation**: Maintain investigation notes and corrections within topic
