---
name: steering
description: "Verify and update steering documentation using parallel investigation agents"
category: utility
complexity: standard
mcp-servers: []
personas: [analyzer, architect]
allowed-tools: Bash(hail-mary:*), Read, Write, Edit, MultiEdit, Glob, Grep, Task
argument-hint: [--type <name>]
---

# /hm:steering - Steering Documentation Verification & Update System

## Triggers
- Steering documentation accuracy concerns and information drift prevention
- Periodic verification needs for project knowledge base maintenance
- Codebase changes that may have invalidated existing steering content
- Quality assurance requirements for steering file correctness

## Usage
```
/hm:steering [--type <name>]
```
- `--type <name>`: Focus on specific steering type

## Behavioral Flow

1. **Backup**: Execute !`hail-mary steering backup` to create timestamped backup of current steering files
2. **Load**: Parse steering types from @.kiro/config.toml with criteria and purposes
3. **Investigate**: Launch parallel Task agents to verify each steering type independently
4. **Aggregate**: Collect verification results as investigation completes
5. **Update**: Apply all corrections and additions with single batch confirmation

Key behaviors:
- **Automatic backup**: Uses `hail-mary steering backup` to create timestamped backup before any modifications
- **Parallel investigation**: Multiple Task agents process each type independently and concurrently
- **Correctness-first approach**: Prioritize fixing incorrect information over adding new content
- **Batch confirmation**: User approves all changes at once before applying
- **Structure preservation**: Maintain existing file format and organization
- **Intelligent reporting**: Clear status indicators (❌ incorrect, ✅ verified, 🆕 new)

### Backup Phase

Execute backup command: !`hail-mary steering backup`

```
> 📦 Creating backup of current steering files...
> ✅ Created backup '2025-09-13-14-30' with 4 files
```

The `hail-mary steering backup` command creates a timestamped backup directory (e.g., `.kiro/steering/backup/2025-09-13-14-30/`) containing copies of all current steering files. This ensures we can restore the original state if needed.

### Parallel Investigation Phase

Launch parallel Task agents for each steering type:

```
> 🚀 Launching parallel investigation for {n} steering types...
>
> Spawning investigation agents:
> • [Agent 1] {type1.name} - {type1.purpose}
> • [Agent 2] {type2.name} - {type2.purpose}
> • [Agent 3] {type3.name} - {type3.purpose}
> • [Agent n] {typeN.name} - {typeN.purpose}
>
> [Parallel Task agents processing independently...]
```

#### Parallel Task Agent Execution
Launch multiple Task agents in a single message for concurrent investigation:

**[The implementation will send multiple Task tool calls in one response]**
- Task 1: Investigate {type1.name} steering type
- Task 2: Investigate {type2.name} steering type
- Task 3: Investigate {type3.name} steering type
- Task n: Investigate {typeN.name} steering type

Each agent receives an independent mission:

```
Investigate the "{type.name}" steering type.

Purpose: {type.purpose}
Criteria: {type.criteria}

Your mission:
1. READ the existing steering file: .kiro/steering/{type.name}.md
2. VERIFY each documented pattern against the actual codebase
3. IDENTIFY incorrect or outdated information
4. DISCOVER new patterns matching the criteria
5. RETURN structured results:
   - Incorrect items found (with corrections)
   - Outdated items needing updates
   - New discoveries to add
   - Validation status for each criterion

Use these tools:
- Read: Load the existing steering file
- Grep: Search for patterns in codebase
- Glob: Find relevant files
- Analyze patterns against the criteria

Focus on CORRECTNESS over completeness.
Return your findings for aggregation.
```

### Aggregation & Review Phase

After Task agent completes investigation of all types, show detailed results with all changes:

```
> 📊 Investigation Results & Changes
>
> ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
> 📁 bigquery.md
> ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
> Status: ❌ 2 incorrect | ✅ 8 verified | 🆕 3 new
>
> 🔧 Corrections to apply:
> • OLD: "EXTERNAL_QUERY uses MySQL syntax"
>   NEW: "EXTERNAL_QUERY uses PostgreSQL syntax"
> • OLD: "Partitioning by DATE field"
>   NEW: "Partitioning by _PARTITIONDATE pseudo column"
>
> 🆕 New patterns found:
> • BigQuery ML patterns in ml/models/
> • Cost optimization with clustering
> • Materialized view strategies
>
> ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
> 📁 security.md
> ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
> Status: ❌ 1 incorrect | ✅ 12 verified | 🆕 5 new
>
> 🔧 Corrections to apply:
> • OLD: "JWT tokens expire after 24 hours"
>   NEW: "JWT tokens expire after 1 hour with 7-day refresh token"
>
> 🆕 New patterns found:
> • OAuth2 implementation in auth/oauth.ts
> • Rate limiting in middleware/rateLimit.ts
> • CSRF protection in middleware/csrf.ts
> • API key rotation in services/apiKeys.ts
> • Audit logging in services/audit.ts
>
> ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
> 📁 api-patterns.md
> ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
> Status: ✅ 15 verified | 🆕 2 new
>
> 🆕 New patterns found:
> • GraphQL subscription patterns in api/subscriptions/
> • REST endpoint versioning in api/v2/
>
> ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
>
> 🔄 Apply ALL changes listed above? [Y/n]:
```

**[STOP HERE AND WAIT FOR USER RESPONSE - DO NOT PROCEED]**

After user responds:
- Response = "Y" or Enter → Apply ALL corrections and updates with MultiEdit in batch
- Response = "n" → Skip all updates

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
- **Task**: Spawn **parallel** investigation agents for each steering type
  - Multiple Task tools sent in single message for concurrent execution
  - Each agent operates independently with its own context
- **Grep**: Search for patterns matching criteria across codebase
- **Glob**: Find files by type and pattern
- **Read**: Load existing steering files for verification
- **MultiEdit**: Batch corrections and updates efficiently
- **Write**: Create new steering files
- **Bash**: Execute `hail-mary steering backup` and check file existence

## Key Patterns
- **Parallel Investigation**: Config.toml types → **Parallel Task agent spawning** → concurrent verification → aggregated results
- **Verification Flow**: Read existing → compare with codebase → identify discrepancies → generate corrections
- **Batch Update Flow**: Collect all changes → display detailed summary → single confirmation → batch apply
- **Agent Communication**: Structured mission → **independent parallel investigation** → status reports → main aggregation
- **Concurrent Execution**: Multiple Task tools in single message → independent processing → synchronized aggregation

## Examples

### Example 1: Batch Mode Update
```
/hm:steering

> 📦 Creating backup of current steering files...
> ✅ Created backup '2025-09-13-14-30' with 4 files
>
> 🚀 Launching parallel investigation for 3 steering types...
>
> Spawning investigation agents:
> • [Agent 1] bigquery - BigQuery optimization patterns
> • [Agent 2] security - Security patterns and vulnerabilities
> • [Agent 3] api-patterns - API design and contracts
>
> [Parallel Task agents processing independently...]
>
> 📊 Investigation Results & Changes
>
> ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
> 📁 bigquery.md
> ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
> Status: ❌ 2 incorrect | ✅ 8 verified | 🆕 3 new
>
> 🔧 Corrections to apply:
> • OLD: "EXTERNAL_QUERY uses MySQL syntax"
>   NEW: "EXTERNAL_QUERY uses PostgreSQL syntax"
> • OLD: "Partitioning by DATE field"
>   NEW: "Partitioning by _PARTITIONDATE pseudo column"
>
> 🆕 New patterns found:
> • BigQuery ML patterns in ml/models/
> • Cost optimization with clustering
> • Materialized view strategies
>
> ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
> 📁 security.md
> ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
> Status: ❌ 1 incorrect | ✅ 12 verified | 🆕 5 new
>
> 🔧 Corrections to apply:
> • OLD: "JWT tokens expire after 24 hours"
>   NEW: "JWT tokens expire after 1 hour with 7-day refresh token"
>
> 🆕 New patterns found:
> • OAuth2 implementation in auth/oauth.ts
> • Rate limiting in middleware/rateLimit.ts
> • CSRF protection in middleware/csrf.ts
> • API key rotation in services/apiKeys.ts
> • Audit logging in services/audit.ts
>
> ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
> 📁 api-patterns.md
> ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
> Status: ✅ 15 verified | 🆕 2 new
>
> 🆕 New patterns found:
> • GraphQL subscription patterns in api/subscriptions/
> • REST endpoint versioning in api/v2/
>
> ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
>
> 🔄 Apply ALL changes listed above? [Y/n]: Y
>
> ✅ Batch Update Applied:
> • Fixed 3 incorrect items across 2 files
> • Added 10 new patterns across 3 files
> • All steering files updated successfully
```

### Example 2: Skip All Changes
```
/hm:steering

> 📦 Creating backup of current steering files...
> ✅ Created backup '2025-09-13-14-32' with 4 files
>
> [Investigation phase completed...]
>
> 📊 Investigation Results & Changes
>
> ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
> 📁 security.md
> ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
> Status: ❌ 2 incorrect | ✅ 14 verified | 🆕 4 new
>
> 🔧 Corrections to apply:
> • OLD: "JWT tokens expire after 24 hours"
>   NEW: "JWT tokens expire after 1 hour with 7-day refresh token"
> • OLD: "Password hashing uses MD5"
>   NEW: "Password hashing uses bcrypt with salt rounds 10"
>
> 🆕 New patterns found:
> • WebSocket authentication in ws/auth.ts
> • Session management in services/session.ts
> • Two-factor authentication in auth/2fa.ts
> • Security headers middleware in middleware/security.ts
>
> ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
>
> 🔄 Apply ALL changes listed above? [Y/n]: n
>
> ⏭️ Skipped all updates
>
> ✅ Steering verification complete (no changes applied)
```

## Boundaries

### Will
- **Verify correctness first** - Priority on fixing incorrect information
- **Use parallel Task agents** - Investigate each type independently and concurrently
- **Provide clear investigation reports** - Show what's correct, incorrect, and new
- **Batch updates** - User confirms all changes at once before applying
- **Preserve existing file structure** - Maintain current format and organization of steering files
- **Create backups** using `hail-mary steering backup` before modifying existing files
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