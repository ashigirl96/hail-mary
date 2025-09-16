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
- **Steering Guidance**: Embedded steering → investigation focus → targeted search patterns
- **Depth Detection**: Simple question → standard depth | Complex/multi-system → deep investigation
- **Source Priority**: codebase search → Context7 docs → web (steering guides but not evidence)
- **Format Detection**: Code → Technical Pattern | System → Architecture Flow | Issue → Problem Analysis
- **Confidence Scoring**: Source trust × content match × recency = confidence level
- **Interactive Loop**: Investigate → Present → Refine → Document

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

## Document Template

### Investigation Structure
````markdown
# Investigation - [Spec Name]

## Topic: [Auto-generated Topic Title]
**Confidence**: [level] ([percentage]%)
**Primary Sources**: src/path/* ([%]), Context7:lib ([%]), web ([%])
**Guided by**: steering:[type-name] (patterns and criteria)

### Summary
[1-2 line executive summary of findings]

### Root Cause / Core Finding
[Main discovery - flexible format based on content type]
- Architecture diagrams (mermaid)
- Code implementations
- System designs
- Data flows

### Evidence
[Source-based evidence with attribution]

**From Codebase (path/file.ts:lines)**:
```language
// Actual implementation code
```

**From Context7 (library docs)**:
- [Official patterns and best practices]

**From Web (as last resort)**:
- [Recent developments or community solutions]

### Recommendations
1. [Actionable recommendation]
2. [Implementation approach]
3. [Consideration or trade-off]

### Investigation Notes
- **Update [time]**: [Additional findings or corrections]
- **Correction**: [Fixed understanding or updated information]
- **Note**: [Important observations or caveats]
- **Steering Applied**: Used [type-name] patterns to guide investigation focus
````

## Behavioral Flow

1. **Initialize**: Parse arguments and load existing investigation
   - Check for existing <kiro_investigation_path>
   - Determine mode: standalone or linked (`--for`)
   - If `--topic <name>`: search for matching section to resume/update
   - If no `--topic`: prepare for new investigation
   - Load existing topics for reference

2. **Topic Gathering**: Determine investigation topic
   - If `--topic <name>` provided: Resume specified existing topic
   - Otherwise, ask for new topic:
   ```
   > 🔍 What would you like to investigate?
   > [Provide specific technical question or area]
   ```

   **[STOP HERE AND WAIT FOR USER INPUT - DO NOT PROCEED]**

   - Auto-generate concise title (2-4 words) from user input
   - Create new section for this investigation

3. **Parallel Investigation**: Launch Task agents with plan display
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

4. **Progressive Documentation**: Save findings immediately
   - Auto-select format based on content type
   - Append to <kiro_investigation_path>
   - Display save confirmation

   ```
   > 📝 Investigation saved to <kiro_investigation_path>
   > Topic: "[Title]" (Section #[n])
   > Confidence: [level] ([percentage]%)
   ```

5. **Interactive Continuation**: Topic refinement loop
   ```
   > 🔄 Continue investigating "[Topic]"?
   > - [Y/Enter]: Deepen current topic
   > - [n/done]: Finish investigation
   >
   > Or provide specific follow-up question:
   ```

   **[STOP HERE AND WAIT FOR USER RESPONSE - DO NOT PROCEED]**

   - Y/follow-up → Update same section with new findings (return to step 3)
   - n/done → Proceed to step 6

6. **Finalization**: Link to other documents if `--for` present
   - If `--for` requirements: Extract relevant → Update <kiro_requirements_path>
   - If `--for` design: Extract architectural → Update <kiro_design_path>

   ```
   > ✅ Investigation complete
   > • Topics investigated: [count]
   > • Overall confidence: [level] ([percentage]%)
   > [if --for] • Updated: [document].md
   ```

Key behaviors:
- **Steering as Guide**: Use embedded steering to focus investigation, not as evidence source
- **Source Priority**: Code > docs > web for evidence (steering informs search patterns)
- **Parallel Execution**: Multiple Task agents investigate simultaneously
- **Progressive Save**: Write to <kiro_investigation_path> after each round, not just at end
- **Session Scope**: Each command invocation handles one topic (with deepening)
- **Topic Management**: `--topic <name>` resumes existing, no flag creates new
- **Section Management**: Same topic updates section, new command creates new section
- **Confidence Tracking**: Calculate and display trust level for all findings
- **History Preservation**: Maintain investigation notes and corrections within topic

## Examples

### Example 1: New Topic Investigation
```
/hm:investigate

> 🔍 What would you like to investigate?
> [Provide specific technical question or area]

[STOP AND WAIT]

User: "JWT authentication implementation"

> 🚀 Investigation Plan for "JWT Authentication":
> [Parallel agents launch...]

> 📝 Investigation saved to <kiro_investigation_path>
> Topic: "JWT Authentication" (Section #1)
> Confidence: High (90%)

> 🔄 Continue investigating "JWT Authentication"?

User: Y, what about refresh token rotation?

> 📝 Updated investigation for "JWT Authentication"
> Added findings about refresh token rotation
> Confidence: High (92%)

> 🔄 Continue investigating "JWT Authentication"?

User: done

> ✅ Investigation complete
> • Topics investigated: 1
> • Overall confidence: High (92%)
```

### Example 2: Resume Existing Topic
```
/hm:investigate --topic "JWT Authentication" --for requirements

> 📝 Resuming existing topic "JWT Authentication" from investigation.md
> Previous confidence: High (92%)

> 🚀 Continuing investigation for "JWT Authentication":
> [Parallel agents focus on gaps/updates...]

> 📝 Updated investigation for "JWT Authentication"
> Topic: "JWT Authentication" (Section #1 - Updated)
> Confidence: High (95%)

> 🔄 Continue investigating "JWT Authentication"?

User: done

> ✅ Investigation complete
> • Topics investigated: 1 (resumed)
> • Overall confidence: High (95%)
> • Updated: requirements.md (Technical Requirements section)
```

### Example 3: Multiple Separate Investigations
```
# First investigation
/hm:investigate

> 🔍 What would you like to investigate?

User: "API rate limiting"

[Investigation process...]

> 🔄 Continue investigating "API Rate Limiting"?

User: done

> ✅ Investigation complete
> • Topics investigated: 1
> • Overall confidence: High (85%)

# Second investigation (new command)
/hm:investigate

> 🔍 What would you like to investigate?

User: "caching strategy"

> 🚀 Investigation Plan for "Caching Strategy":
> [New parallel investigation...]

> 📝 Investigation saved to <kiro_investigation_path>
> Topic: "Caching Strategy" (Section #2)
> Confidence: Medium (75%)

> 🔄 Continue investigating "Caching Strategy"?

User: done

> ✅ Investigation complete
> • Topics investigated: 1
> • Overall confidence: Medium (75%)

# Third investigation (resuming first topic)
/hm:investigate --topic "API Rate Limiting"

> 📝 Resuming existing topic "API Rate Limiting" from investigation.md
> Previous confidence: High (85%)

[Continue investigation with new angle...]
```

### Example 4: Investigation Guided by Steering
```
/hm:investigate

> 🔍 What would you like to investigate?

User: "How to query production database for users table id=123 name field?"

> 🚀 Investigation Plan for "Production Database Query":
>
> Using steering guidance from `bigquery` type:
> - Focus on EXTERNAL_QUERY patterns for Cloud SQL
> - Check security best practices
> - Look for performance optimization patterns
>
> [Parallel agents launch with steering-informed focus...]

> 📝 Investigation saved to investigation.md
> Topic: "Production Database Query" (Section #1)
> Confidence: High (92%)
> **Guided by**: steering:bigquery (EXTERNAL_QUERY patterns)
>
> ### Summary
> Found optimal approach using BigQuery EXTERNAL_QUERY for Cloud SQL production access
>
> ### Evidence
> **From Codebase (src/services/data_access.ts:89)**:
> ```typescript
> // Existing pattern for production queries
> const query = `SELECT name FROM
>   EXTERNAL_QUERY('projects/prod-project/locations/asia-northeast1/connections/prod-mysql',
>   'SELECT name FROM users WHERE id = 123')`;
> ```
>
> **From Context7 (BigQuery documentation)**:
> - EXTERNAL_QUERY syntax for Cloud SQL connections
> - Security: Always use read-only credentials for production
> - Performance: Batch queries to minimize round-trips
>
> ### Recommendations
> 1. Use EXTERNAL_QUERY pattern from BigQuery to access production Cloud SQL
> 2. Ensure read-only access credentials are configured
> 3. Follow existing implementation pattern in data_access.ts
>
> ### Investigation Notes
> - **Steering Applied**: Used `bigquery` type patterns to identify EXTERNAL_QUERY as the appropriate solution

> 🔄 Continue investigating "Production Database Query"?

User: done

> ✅ Investigation complete
```
