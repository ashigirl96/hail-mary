---
description: Save learning to steering with intelligent type detection and creation
allowed-tools: Read, Write, Edit, MultiEdit, Bash(date:*), Glob
argument-hint: [hint] [--format rule|guide|knowledge] [--type <name>]
---

## Triggers
- User identifies new learning or pattern to remember
- Context needs to be preserved for future reference  
- Information not already documented in existing steering files

## Usage
```
/hm:steering-remember [hint] [--format rule|guide|knowledge] [--type <name>]
```

### Examples
```bash
# With topic hint
/hm:steering-remember "BigQueryについて学んだこと"
/hm:steering-remember "認証の話" --type security

# No hint - extract from entire conversation
/hm:steering-remember

# Format override
/hm:steering-remember "デバッグ手順" --format guide
```

## Config.toml Structure

This command reads steering type definitions from @.kiro/config.toml:

```toml
[[steering.types]]
name = "bigquery"                           # Filename: bigquery.md
purpose = "BigQuery optimization patterns"  # Description shown in prompts
criteria = [                                # Patterns for type matching
    "Query Optimization: Performance techniques",
    "EXTERNAL_QUERY: Cloud SQL patterns",
    "Cost Management: Query cost strategies"
]
```

### Property Details
- **`name`**: Determines the steering filename (`{name}.md`)
- **`purpose`**: Human-readable description shown during type selection
- **`criteria`**: Array of patterns used for automatic type matching

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
   - **If match found (>60% confidence)**:
     ```
     > 🔍 Analyzing learning content...
     > ✅ Found match: 'bigquery' type (confidence: 85%)
     > 
     > Append to bigquery.md? [Y/n]: 
     ```
     
     **[STOP HERE AND WAIT FOR USER RESPONSE - DO NOT PROCEED]**
     
     After user responds:
     - Response = "Y" or "y" or Enter → Append to existing steering file using **Edit** or **MultiEdit**
     - Response = "n" or "N" → Skip appending and continue
     - Any other response → Ask for clarification: "Please enter Y or n"
     
   - **If no match**:
     ```
     > 🔍 Analyzing learning content...
     > 🤔 No existing type matches this content
     > 
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
     - Invalid selection → Ask for valid input: "Please select 1-4"
     - After type creation → Create new steering file using **Write**

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
   - If exists: Use **Edit** or **MultiEdit** to append
   - If new: Use **Write** to create file
   - Confirm successful save

## Tool Usage
- **Auto-loaded**: @.kiro/config.toml for type definitions
- **Glob**: Check existing steering files in .kiro/steering/*.md
- **Write**: Create new steering file when type doesn't exist
- **Edit/MultiEdit**: Append to existing steering file or update config.toml
- **Bash(date:*)**: Generate timestamp for tracking

## Key Behaviors
- **Conversation analysis**: Analyze entire conversation history for relevant learnings when hint provided
- **Context extraction**: Extract actionable insights from natural conversation flow
- **Maximum brevity**: Remove all unnecessary explanation
- **Concrete over abstract**: Include specific examples, not theory
- **Action-oriented**: Focus on what to DO, not background
- **One learning per operation**: Don't combine multiple insights in single execution
- **Auto-detect format**: Code→Rule, Steps→Guide, Concepts→Knowledge
- **Interactive type selection**: Guide user through type creation

## Examples

### Example 1: Existing Type Match
````
/hm:steering-remember "BigQueryで学んだこと"

> 🔍 Analyzing conversation for BigQuery-related learnings...
> ✅ Found match: 'bigquery' type (confidence: 85%)
> 
> Append to bigquery.md? [Y/n]: 

[STOP AND WAIT FOR USER INPUT]

User: Y

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
> 🤔 No existing type matches this content
> 
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
> 🤔 No existing type matches this content
> 
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

**Will Not:**
- Use draft directories or intermediate storage
- Combine multiple learnings in one operation
- Create verbose explanations (100+ lines)
- Overwrite existing content (always append)
- Process without clear learning to capture
- Create types without user confirmation
- **Report success without actually using MultiEdit/Edit/Write tools to modify files**