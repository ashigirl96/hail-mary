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
   ```
   > 📦 Creating backup of current steering files...
   > ✅ Created backup '2025-09-13-14-30' with 4 files
   ```
   The `hail-mary steering backup` command creates a timestamped backup directory (e.g., `.kiro/steering/backup/2025-09-13-14-30/`) containing copies of all current steering files. This ensures we can restore the original state if needed.

2. **Load**: Parse steering types from @.kiro/config.toml, filtering out types where `allowed_operations = []`
   ```
   > 🔍 Analyzing steering types...
   > • {type1.name}.md [{operations status}] - {action description}
   > • {type2.name}.md [{operations status}] - {action description}
   > • {type3.name}.md [{operations status}] - {action description}
   > • {typeN.name}.md [skipped - no operations allowed]
   ```

3. **Parallel Investigation**: Launch parallel Task agents to comprehensively verify each allowed type
   ```
   > 🚀 Launching specialized steering investigators for {n} types...
   >
   > Spawning steering-investigator agents:
   > • [Investigator 1] {type1.name} - {type1.purpose}
   > • [Investigator 2] {type2.name} - {type2.purpose}
   > • [Investigator 3] {type3.name} - {type3.purpose}
   > • [Investigator n] {typeN.name} - {typeN.purpose}
   >
   > [Parallel steering-investigator agents processing independently...]
   ```

   **[The implementation will send multiple Task tool calls with subagent_type="steering-investigator"]**

   Each steering-investigator receives type-specific context via prompt:
   ```python
   Task(
       subagent_type="steering-investigator",
       description="Verify {type.name} steering documentation",
       prompt="""
       Steering Type: {type.name}
       Purpose: {type.purpose}
       Criteria: {type.criteria}
       Allowed Operations: {allowed_operations}
       File Path: .kiro/steering/{type.name}.md

       Mission: Verify and update the steering documentation for this type.

       Instructions:
       1. LOAD the existing steering file
       2. VERIFY each documented pattern against codebase reality
       3. DISCOVER new patterns matching the criteria
       4. RESPECT allowed_operations when suggesting changes:
          - If "refresh" allowed: Report corrections for outdated info
          - If "discover" allowed: Report new pattern discoveries
          - If neither: Only report verification status

       Return structured findings for aggregation.
       """
   )
   ```

4. **Aggregation & Review**: Collect verification results and filter based on allowed_operations:
   ```
   > 📊 Investigation Results & Changes
   >
   > ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   > 📁 {type.name}.md [{allowed_operations status}]
   > ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   > Status: ❌ {incorrect count} | ✅ {verified count} | 🆕 {new count}
   >
   > [If "refresh" in allowed_operations:]
   > 🔧 Corrections to apply (refresh allowed):
   > • OLD: "{existing content}"
   >   NEW: "{corrected content}"
   >
   > [If "discover" in allowed_operations:]
   > 🆕 New patterns found (discover allowed):
   > • {new pattern description}
   >
   > [If allowed_operations is empty:]
   > ⏭️ Skipped - manual updates only
   >
   > ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   > [Repeat for each type...]
   >
   > 🔄 Apply ALL changes listed above? [Y/n]:
   ```

   **[STOP HERE AND WAIT FOR USER RESPONSE - DO NOT PROCEED]**

   After user responds:
   - Response = "Y" or Enter → Apply ALL corrections and updates with MultiEdit in batch
   - Response = "n" → Skip all updates

5. **Summary**: Apply all corrections and additions with single batch confirmation
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

Key behaviors:
- **Automatic backup**: Uses `hail-mary steering backup` to create timestamped backup before any modifications
- **Parallel investigation**: Multiple Task agents process each type independently and concurrently
- **Correctness-first approach**: Prioritize fixing incorrect information over adding new content
- **Batch confirmation**: User approves all changes at once before applying
- **Structure preservation**: Maintain existing file format and organization
- **Intelligent reporting**: Clear status indicators (❌ incorrect, ✅ verified, 🆕 new)

## Tool Coordination

- **@.kiro/config.toml**: Auto-loaded for configuration (no Read tool needed)
- **Task**: Spawn **parallel** steering-investigator subagents for each steering type
  - Multiple Task tools with `subagent_type="steering-investigator"` sent in single message
  - Each investigator receives type-specific context via prompt parameter
  - Subagents operate independently with specialized verification methodology
- **Grep**: Search for patterns matching criteria across codebase (used by subagents)
- **Glob**: Find files by type and pattern (used by subagents)
- **Read**: Load existing steering files for verification (used by subagents)
- **MultiEdit**: Batch corrections and updates efficiently
- **Write**: Create new steering files
- **Bash**: Execute `hail-mary steering backup` and check file existence

## Key Patterns
- **Specialized Investigation**: Config.toml types → **Parallel steering-investigator subagents** → evidence-based verification → aggregated results
- **Parameterized Subagent**: Single subagent type handles all steering types via prompt parameters
- **Multi-Hypothesis Verification**: Each investigator maintains 3-7 competing theories during verification
- **Evidence Chain Documentation**: Subagents document complete evidence trails for all findings
- **Batch Update Flow**: Collect all changes → display detailed summary → single confirmation → batch apply
- **Concurrent Execution**: Multiple Task tools in single message → independent processing → synchronized aggregation
- ultrathink

## Examples

### Example 1: All Types (Default)
```
/hm:steering

> 📦 Creating backup of current steering files...
> ✅ Created backup '2025-09-13-14-30' with 5 files
>
> 🔍 Analyzing steering types...
> • bigquery.md [refresh ✅, discover ✅] - Will check and update
> • security.md [refresh ✅, discover ✅] - Will check and update
> • api-patterns.md [discover ✅] - Will only add new content
> • principles.md [skipped - no operations allowed]
> • decisions.md [skipped - no operations allowed]
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
> 📁 bigquery.md [refresh ✅, discover ✅]
> ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
> Status: ❌ 2 incorrect | ✅ 8 verified | 🆕 3 new
>
> 🔧 Corrections to apply (refresh allowed):
> • OLD: "EXTERNAL_QUERY uses MySQL syntax"
>   NEW: "EXTERNAL_QUERY uses PostgreSQL syntax"
> • OLD: "Partitioning by DATE field"
>   NEW: "Partitioning by _PARTITIONDATE pseudo column"
>
> 🆕 New patterns found (discover allowed):
> • BigQuery ML patterns in ml/models/
> • Cost optimization with clustering
> • Materialized view strategies
>
> ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
> 📁 security.md [refresh ✅, discover ✅]
> ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
> Status: ❌ 1 incorrect | ✅ 12 verified | 🆕 5 new
>
> 🔧 Corrections to apply (refresh allowed):
> • OLD: "JWT tokens expire after 24 hours"
>   NEW: "JWT tokens expire after 1 hour with 7-day refresh token"
>
> 🆕 New patterns found (discover allowed):
> • OAuth2 implementation in auth/oauth.ts
> • Rate limiting in middleware/rateLimit.ts
> • CSRF protection in middleware/csrf.ts
> • API key rotation in services/apiKeys.ts
> • Audit logging in services/audit.ts
>
> ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
> 📁 decisions.md [discover ✅ only]
> ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
> Status: ❌ 1 incorrect | ✅ 15 verified | 🆕 2 new
>
> ⚠️ Found 1 incorrect item but refresh not allowed - skipping corrections
>
> 🆕 New patterns found (discover allowed):
> • GraphQL adoption decision in docs/adr/
> • Microservices migration strategy
>
> ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
> 📁 principles.md [no operations allowed]
> ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
> ⏭️ Skipped - manual updates only
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

### Example 2: Specific Type
```
/hm:steering --type tech

> 📦 Creating backup of current steering files...
> ✅ Created backup '2025-09-13-15-45' with 5 files
>
> 🔍 Analyzing single steering type...
> • tech.md [refresh ✅, discover ✅] - Will check and update
>
> 🚀 Launching investigation for 1 steering type...
>
> Spawning investigation agent:
> • [Agent 1] tech - Technical stack and development environment
>
> [Task agent processing...]
>
> 📊 Investigation Results & Changes
>
> ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
> 📁 tech.md [refresh ✅, discover ✅]
> ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
> Status: ❌ 2 incorrect | ✅ 14 verified | 🆕 3 new
>
> 🔧 Corrections to apply (refresh allowed):
> • OLD: "Node.js version 14"
>   NEW: "Node.js version 20 LTS"
> • OLD: "Python 3.8"
>   NEW: "Python 3.11+"
>
> 🆕 New patterns found (discover allowed):
> • Docker compose configuration in docker/
> • GitHub Actions workflows in .github/workflows/
> • Environment variables in .env.example
>
> ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
>
> 🔄 Apply ALL changes listed above? [Y/n]: Y
>
> ✅ Update Applied:
> • Fixed 2 incorrect items in tech.md
> • Added 3 new patterns to tech.md
> • Steering file updated successfully
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
