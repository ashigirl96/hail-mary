---
name: steering-remember
description: "Save learning to steering with intelligent type detection and creation"
category: utility
complexity: standard
mcp-servers: []
personas: []
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
     > 🔍 Analyzing learning content...
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
     → Rule format (with ✅/❌ examples)
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
   # ✅ Good
   [correct example]
   
   # ❌ Bad  
   [incorrect example]
   ```
   ````
   
   *Example output:*
   ````markdown
   ## Service Return Values
   **When**: Creating service objects in this codebase
   - Return plain hashes for performance
   - Wrap in transactions for consistency
   - Use Japanese error messages
   
   ```ruby
   # ✅ Good
   def call
     { success: true, data: @result }
   end
   
   # ❌ Bad
   OpenStruct.new(success: true)
   ```
   ````
   
   **Guide Format** (20-40 lines):
   ```markdown
   ## [Action-Oriented Title]
   **Context**: [When/why to use this guide]
   1. [First step]
   2. [Second step]
   3. [Third step]
   ⚠️ [Critical warning if any]
   ```
   
   *Example output:*
   ````markdown
   ## BigQuery Debug Process
   **Context**: Troubleshooting query failures
   1. Check Cloud Logging for errors
   2. Verify connection string format
   3. Run EXTERNAL_QUERY with minimal scope
   ⚠️ No Japanese comments in SQL files
   ````
   
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
   
   *Example output:*
   `````markdown
   ## Restaurant Reservation System
   **Domain**: Booking Management
   
   **Definition**: 
   - 空席管理と予約調整を行うシステム
   - リアルタイム在庫と予約状態の同期
   - キャンセル待ちリストの自動管理
   
   **Formula**: 
   ````
   予約可能数 = 総席数 - 既存予約 - バッファ
   キャンセル率 = 過去30日のキャンセル数 / 予約総数
   ````
   
   **Flow**:
   ````mermaid
   graph LR
     A[予約リクエスト] --> B{空席確認}
     B -->|あり| C[予約確定]
     B -->|なし| D[キャンセル待ち]
     D --> E[通知登録]
   ````
   
   **Context**: ダブルブッキング防止とキャンセル率を考慮した収益最適化
   `````

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

## Examples

### Example 1: Existing Type Selection
````
/hm:steering-remember "BigQueryで学んだこと"

> 🔍 Analyzing conversation for BigQuery-related learnings...
>
> Select steering type to append to:
> 1. bigquery (confidence: 85%) - BigQuery optimization patterns
> 2. tech (confidence: 45%) - Technology stack
> 3. structure (confidence: 20%) - Code organization
> 4. product (confidence: 10%) - Product overview
> 5. [New Type] - Create a new steering type
>
> Select [1-5]:

[STOP AND WAIT FOR USER SELECTION]

User: 1

> 📝 Added to bigquery.md:
> ## Query Constraints
> **When**: Using EXTERNAL_QUERY with Cloud SQL
> - Japanese comments cause encoding errors
> - Use English comments only in SQL files
> - Applies to all BigQuery external queries
````

### Example 2: Creating New Type
````
/hm:steering-remember "GraphQLの話"

> 🔍 Analyzing conversation for GraphQL-related learnings...
>
> Select steering type to append to:
> 1. tech (confidence: 40%) - Technology stack
> 2. structure (confidence: 25%) - Code organization
> 3. product (confidence: 10%) - Product overview
> 4. documentation (confidence: 5%) - Documentation standards
> 5. [New Type] - Create a new steering type
>
> Select [1-5]:

[STOP AND WAIT FOR USER SELECTION]

User: 5

> Create new type? Suggestions:
> 1. graphql - GraphQL patterns and optimizations
> 2. api-performance - API performance optimizations
> 3. backend-patterns - Backend architectural patterns
> 4. [Custom] - Enter your own type name
>
> Select [1-4]:

[STOP AND WAIT FOR USER SELECTION]

User: 1

> 📝 Creating new type 'graphql'...
> ✅ Added to config.toml:
> [[steering.types]]
> name = "graphql"
> purpose = "GraphQL patterns and optimizations"
> criteria = [
>     "Schema Design: Type definitions and relationships",
>     "Performance: N+1 prevention and query optimization",
>     "Security: Query depth limiting and validation",
>     "Best Practices: Naming conventions and patterns"
> ]
> allowed_operations = []
>
> ✅ Created graphql.md with your learning:
> ## N+1 Query Prevention
> **When**: Implementing GraphQL resolvers
> - Use DataLoader for batch loading
> - Cache results within request context
> - Avoid nested resolver queries
````

### Example 3: Custom Type Selection
````
/hm:steering-remember "独自の認証フロー"

> 🔍 Analyzing conversation for authentication-related learnings...
>
> Select steering type to append to:
> 1. tech (confidence: 50%) - Technology stack
> 2. structure (confidence: 35%) - Code organization
> 3. product (confidence: 15%) - Product overview
> 4. documentation (confidence: 5%) - Documentation standards
> 5. [New Type] - Create a new steering type
>
> Select [1-5]:

[STOP AND WAIT FOR USER SELECTION]

User: 5

> Create new type? Suggestions:
> 1. security - Security patterns and best practices
> 2. backend-patterns - Backend architectural patterns
> 3. api-patterns - API design and implementation
> 4. [Custom] - Enter your own type name
>
> Select [1-4]:

[STOP AND WAIT FOR USER SELECTION]

User: 4

> Enter custom type name:

[STOP AND WAIT FOR USER INPUT]

User: auth-flow

> 📝 Creating new type 'auth-flow'...
> ✅ Added to config.toml and created auth-flow.md
````

### Example 4: Manual Type Override
```
/hm:steering-remember --type security "JWT署名は必ず検証"
# Forces save to security.md even if other types might match
# Creates security type if it doesn't exist
```

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
