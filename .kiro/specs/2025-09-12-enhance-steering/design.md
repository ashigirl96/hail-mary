# Design

## Overview
Enhanced steering command design following slash command best practices with interactive user flow and proper prompt engineering patterns.

## Enhanced Steering Command Markdown

````markdown
---
name: steering
description: "Verify and update steering documentation using parallel investigation agents"
category: utility
complexity: standard
mcp-servers: []
personas: [analyzer, architect]
allowed-tools: Bash(hail-mary:*), Read, Write, Edit, MultiEdit, Glob, Grep, Task
argument-hint: [--parallel] [--type <name>]
---

# /hm:steering - Steering Documentation Verification & Update System

## Triggers
- Steering documentation accuracy concerns and information drift prevention
- Periodic verification needs for project knowledge base maintenance
- Codebase changes that may have invalidated existing steering content
- Quality assurance requirements for steering file correctness

## Usage
```
/hm:steering [--parallel] [--type <name>]
```
- `--parallel`: Enable parallel investigation using Task agents (default)
- `--type <name>`: Focus on specific steering type

## Behavioral Flow

1. **Backup**: Execute `hail-mary steering backup` to create timestamped backup of current steering files
2. **Load**: Parse steering types from @.kiro/config.toml with criteria and purposes
3. **Investigate**: Launch Task agent to verify each steering type sequentially
4. **Aggregate**: Collect verification results as investigation completes
5. **Update**: Apply corrections and additions with user confirmation

Key behaviors:
- **Automatic backup**: Uses `hail-mary steering backup` to create timestamped backup before any modifications
- **Sequential investigation**: Task agent processes each type thoroughly one at a time
- **Correctness-first approach**: Prioritize fixing incorrect information over adding new content
- **Interactive confirmation**: User approves all changes before applying
- **Structure preservation**: Maintain existing file format and organization
- **Intelligent reporting**: Clear status indicators (❌ incorrect, ✅ verified, 🆕 new)

### Backup Phase

Execute backup command: !`hail-mary steering backup`

```
> 📦 Creating backup of current steering files...
> ✅ Created backup '2025-09-13-14-30' with 4 files
```

The `hail-mary steering backup` command creates a timestamped backup directory (e.g., `.kiro/steering/backup/2025-09-13-14-30/`) containing copies of all current steering files. This ensures we can restore the original state if needed.

### Sequential Investigation Phase

Launch Task agent to investigate all steering types:

```
> 🚀 Launching investigation for {n} steering types...
> 
> Types to investigate:
> • {type1.name} - {type1.purpose}
> • {type2.name} - {type2.purpose}
> • {type3.name} - {type3.purpose}
> 
> [Task agent processing each type sequentially...]
```

#### Task Agent Mission
The Task agent receives this structured investigation prompt:

```
Investigate all steering types from config.toml sequentially.

For each type:
1. Name: {type.name}
2. Purpose: {type.purpose}
3. Criteria: {type.criteria}

Your mission for EACH type:
1. READ the existing steering file: .kiro/steering/{type.name}.md
2. VERIFY each documented pattern against the actual codebase
3. IDENTIFY incorrect or outdated information
4. DISCOVER new patterns matching the criteria
5. COLLECT results for aggregation:
   - Incorrect items found (with corrections)
   - Outdated items needing updates
   - New discoveries to add
   - Validation status for each criterion

Process each type completely before moving to the next.

Use these tools:
- Read: Load the existing steering files
- Grep: Search for patterns in codebase
- Glob: Find relevant files
- Analyze patterns against the criteria

Focus on CORRECTNESS over completeness.
Return aggregated results for all types.
```

### Aggregation & Review Phase

After Task agent completes investigation of all types:

```
> 📊 Investigation Results:
> 
> {type1.name}:
>   ❌ Incorrect: {n} items need fixing
>   ⚠️ Outdated: {n} items need updating
>   ✅ Verified: {n} items are correct
>   🆕 New: {n} patterns discovered
> 
> {type2.name}:
>   ❌ Incorrect: {n} items need fixing
>   ✅ Verified: {n} items are correct
>   🆕 New: {n} patterns discovered
```

### Correction Phase (Priority)

For each type with incorrect information:

```
> 🔧 Fixing incorrect information in {type.name}.md
> 
> Corrections to apply:
> • OLD: "Authentication uses JWT tokens"
>   NEW: "Authentication uses session cookies"
> • OLD: "Database queries use raw SQL"
>   NEW: "Database queries use ORM (Prisma)"
> 
> Apply corrections? [Y/n]: 
```

**[STOP HERE AND WAIT FOR USER RESPONSE - DO NOT PROCEED]**

After user responds:
- Response = "Y" or Enter → Apply corrections with MultiEdit
- Response = "n" → Skip corrections for this file

### Update Phase

For verified new discoveries:

```
> 📝 Adding new discoveries to {type.name}.md
> 
> New patterns found:
> • GraphQL subscription patterns in api/subscriptions/
> • WebSocket handling in realtime/events.ts
> • Rate limiting middleware in middleware/rateLimit.ts
> 
> Add new patterns? [Y/n]: 
```

**[STOP HERE AND WAIT FOR USER RESPONSE - DO NOT PROCEED]**

After user responds:
- Response = "Y" or Enter → Add patterns with Edit/MultiEdit
- Response = "n" → Skip additions

### Summary

```
> ✅ Steering verification complete:
> 
> Corrections Applied:
> • Fixed {n} incorrect items across {m} files
> • Updated {n} outdated patterns
> 
> New Discoveries:
> • Added {n} new patterns to documentation
> 
> Validation Status:
> • All steering files now verified against codebase
> • Last verification: {timestamp}
```

## Tool Coordination

- **@.kiro/config.toml**: Auto-loaded for configuration (no Read tool needed)
- **Task**: Spawn parallel investigation agents for each steering type
- **Grep**: Search for patterns matching criteria across codebase
- **Glob**: Find files by type and pattern
- **Read**: Load existing steering files for verification
- **MultiEdit**: Batch corrections and updates efficiently
- **Write**: Create new steering files
- **Bash**: Check file existence and timestamps

## Key Patterns
- **Parallel Investigation**: Config.toml types → Task agent spawning → concurrent verification → aggregated results
- **Verification Flow**: Read existing → compare with codebase → identify discrepancies → generate corrections
- **Correction Priority**: Incorrect fixes → outdated updates → new discoveries → user confirmation
- **Agent Communication**: Structured mission → independent investigation → status report → main aggregation

## Examples

### Example 1: Investigation with Corrections
```
/hm:steering

> 📦 Creating backup of current steering files...
> ✅ Created backup '2025-09-13-14-30' with 4 files
> 
> 🚀 Launching investigation for 3 steering types...
> 
> Types to investigate:
> • bigquery - BigQuery optimization patterns
> • security - Security patterns and vulnerabilities
> • api-patterns - API design and contracts
> 
> [Task agent processing each type sequentially...]
> 
> 📊 Investigation Results:
> 
> bigquery:
>   ❌ Incorrect: 2 items need fixing
>   ✅ Verified: 8 items are correct
>   🆕 New: 3 patterns discovered
> 
> security:
>   ⚠️ Outdated: 1 item needs updating
>   ✅ Verified: 12 items are correct
>   🆕 New: 5 patterns discovered
> 
> api-patterns:
>   ✅ Verified: 15 items are correct
>   🆕 New: 2 patterns discovered
> 
> 🔧 Fixing incorrect information in bigquery.md
> 
> Corrections to apply:
> • OLD: "EXTERNAL_QUERY uses MySQL syntax"
>   NEW: "EXTERNAL_QUERY uses PostgreSQL syntax"
> • OLD: "Partitioning by DATE field"
>   NEW: "Partitioning by _PARTITIONDATE pseudo column"
> 
> Apply corrections? [Y/n]: Y
> 
> ✅ Applied 2 corrections to bigquery.md
> 
> 📝 Adding new discoveries to security.md
> 
> New patterns found:
> • OAuth2 implementation in auth/oauth.ts
> • Rate limiting in middleware/rateLimit.ts
> • CSRF protection in middleware/csrf.ts
> • API key rotation in services/apiKeys.ts
> • Audit logging in services/audit.ts
> 
> Add new patterns? [Y/n]: Y
> 
> ✅ Added 5 new patterns to security.md
```

### Example 2: Single Type Verification
```
/hm:steering --type security

> 📦 Creating backup of current steering files...
> ✅ Created backup '2025-09-13-14-31' with 4 files
> 
> 🚀 Launching investigation for 1 steering type...
> 
> Type to investigate:
> • security - Security patterns and vulnerabilities
> 
> [Task agent processing...]
> 
> 📊 Investigation Results:
> 
> security:
>   ❌ Incorrect: 1 item needs fixing
>   ✅ Verified: 14 items are correct
>   🆕 New: 2 patterns discovered
> 
> 🔧 Fixing incorrect information in security.md
> 
> Corrections to apply:
> • OLD: "JWT tokens expire after 24 hours"
>   NEW: "JWT tokens expire after 1 hour with 7-day refresh token"
> 
> Apply corrections? [Y/n]: Y
> 
> ✅ Steering verification complete
```


## Boundaries

### Will
- **Verify correctness first** - Priority on fixing incorrect information
- **Use parallel Task agents** - Investigate each type independently
- **Provide clear investigation reports** - Show what's correct, incorrect, and new
- **Interactive corrections** - User confirms all fixes before applying
- **Preserve existing file structure** - Maintain current format and organization of steering files
- **Create backups** before modifying existing files
- **Use proper @ prefix** for auto-loading configuration
- **Focus on accuracy** over comprehensiveness

### Will Not
- Process without valid config.toml (assumes hail-mary init was run)
- Change or impose new structure on existing steering files
- Overwrite correct information without verification
- Delete or remove existing patterns from files
- Modify files outside of .kiro/steering/ directory
- Make changes without using actual Write/Edit/MultiEdit tools
- Claim success without verifying file operations
- Include sensitive information (API keys, passwords) in steering files

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

Each steering type in config.toml defines:
1. The filename for the steering documentation
2. The purpose shown to users during investigation
3. The criteria patterns used to search and categorize project content
````

## 解説（日本語）

### 設計の要点

#### 1. 並列Task実行パターン
- **Task Tool による並列調査**: 各 steering type に対して独立したTask agentを起動
- **構造化されたミッション**: 各agentに明確な調査手順を与える
- **並列実行の表現**: slash command内でTask toolを使った並列処理を実現

```javascript
// コンセプトの実装
function behavioralFlow() {
  const types = loadTypesFromConfig()
  // Task agents を並列起動
  Promise.all(types.map(type => investigateAndUpdateByType(type)))
}
```

#### 2. 情報の劣化防止に注力
- **検証ファースト**: 既存のsteering内容の正確性を最優先で確認
- **誤り修正**: 間違った情報は即座に修正提案
- **既存構造の維持**: steering fileの現在の構造を尊重

#### 3. Task Agent のミッション設計
各Agentが受け取る構造化されたプロンプト：
1. 既存ファイルを読む
2. コードベースと照合して検証
3. 誤りや古い情報を特定
4. 新しいパターンを発見
5. 構造化レポートを返す

#### 4. シンプルな設計
- **Config.toml Structure**: steering-remember.mdと同一のセクションでDRY原則を遵守
- **不要な機能を削除**: --fix-onlyオプションを削除し、本質的な機能に集中
- **既存ファイル構造の尊重**: Content Structureで新しいテンプレートを強制しない

#### 5. 集約とレビュー
- **並列結果の集約**: 全agentの結果を統合表示
- **視覚的なステータス**: ❌誤り、⚠️古い、✅正しい、🆕新規
- **優先順位付け**: 誤り修正 → 更新 → 新規追加

この設計により：
- **効率的**: 並列調査で高速化
- **正確**: 情報の劣化防止を最優先
- **シンプル**: 必要最小限の機能に集中
- **一貫性**: 他のslash commandとの統一性