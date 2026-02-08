---
name: steering
description: "Verify and update steering documentation using parallel investigation agents"
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
   > Creating backup of current steering files...
   > Created backup '2025-09-13-14-30' with 4 files
   ```
   The `hail-mary steering backup` command creates a timestamped backup directory (e.g., `.kiro/steering/backup/2025-09-13-14-30/`) containing copies of all current steering files. This ensures we can restore the original state if needed.

2. **Load**: Parse steering types from @.kiro/config.toml, filtering out types where `allowed_operations = []`
   ```
   > Analyzing steering types...
   > {type1.name}.md [{operations status}] - {action description}
   > {type2.name}.md [{operations status}] - {action description}
   > {type3.name}.md [{operations status}] - {action description}
   > {typeN.name}.md [skipped - no operations allowed]
   ```

3. **Parallel Investigation**: Launch parallel Task agents to comprehensively verify each allowed type
   ```
   > Launching specialized steering investigators for {n} types...
   >
   > Spawning Explore agents:
   > [Investigator 1] {type1.name} - {type1.purpose}
   > [Investigator 2] {type2.name} - {type2.purpose}
   > [Investigator 3] {type3.name} - {type3.purpose}
   > [Investigator n] {typeN.name} - {typeN.purpose}
   >
   > [Parallel Explore agents processing independently...]
   ```

   **[The implementation will send multiple Task tool calls with subagent_type="Explore"]**

   Each Explore agent receives type-specific context via prompt:
   ```python
   Task(
       subagent_type="Explore",
       description="Verify {type.name} steering documentation",
       prompt="""
       Steering Type: {type.name}
       Purpose: {type.purpose}
       Criteria: {type.criteria}
       Allowed Operations: {allowed_operations}
       File Path: .kiro/steering/{type.name}.md

       Mission: Verify and update the steering documentation for this type. --ultrathink

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
   > Investigation Results & Changes
   >
   > {type.name}.md [{allowed_operations status}]
   > Status: {incorrect count} incorrect | {verified count} verified | {new count} new
   >
   > [If "refresh" in allowed_operations:]
   > Corrections to apply (refresh allowed):
   > OLD: "{existing content}"
   >   NEW: "{corrected content}"
   >
   > [If "discover" in allowed_operations:]
   > New patterns found (discover allowed):
   > {new pattern description}
   >
   > [If allowed_operations is empty:]
   > Skipped - manual updates only
   >
   > [Repeat for each type...]
   >
   > Apply ALL changes listed above? [Y/n]:
   ```

   **[STOP HERE AND WAIT FOR USER RESPONSE - DO NOT PROCEED]**

   After user responds:
   - Response = "Y" or Enter → Read all files then apply ALL corrections and updates with MultiEdit in batch
   - Response = "n" → Skip all updates

5. **Update Phase**: Read all files then apply corrections with MultiEdit
   ```
   > Reading all steering files for update...
   > Reading {type1.name}.md...
   > Reading {type2.name}.md...
   > Reading {typeN.name}.md...
   >
   > Applying batch updates with MultiEdit...
   > Updating {n} files with corrections and discoveries
   >
   > Steering verification complete:
   >
   > Corrections Applied:
   > Fixed {n} incorrect items across {m} files
   > Updated {n} outdated patterns
   >
   > New Discoveries:
   > Added {n} new patterns to documentation
   >
   > Validation Status:
   > All steering files now verified against codebase
   > Last verification: {timestamp}
   ```

Key behaviors:
- **Automatic backup**: Uses `hail-mary steering backup` to create timestamped backup before any modifications
- **Parallel investigation**: Multiple Task agents process each type independently and concurrently
- **Correctness-first approach**: Prioritize fixing incorrect information over adding new content
- **Batch confirmation**: User approves all changes at once before applying
- **Read-before-edit**: Read ALL steering files first before applying MultiEdit updates
- **Structure preservation**: Maintain existing file format and organization
- **Intelligent reporting**: Clear status indicators (incorrect, verified, new)

## Tool Coordination

- **@.kiro/config.toml**: Auto-loaded for configuration (no Read tool needed)
- **Task**: Spawn **parallel** Explore subagents for each steering type
  - Multiple Task tools with `subagent_type="Explore"` sent in single message
  - Each investigator receives type-specific context via prompt parameter
  - Subagents operate independently with read-only codebase exploration
- **Grep**: Search for patterns matching criteria across codebase (used by subagents)
- **Glob**: Find files by type and pattern (used by subagents)
- **Read**: Load ALL steering files before applying updates (batch read operation)
- **MultiEdit**: Batch corrections and updates efficiently after reading all files
- **Write**: Create new steering files
- **Bash**: Execute `hail-mary steering backup` and check file existence

## Key Patterns
- **Specialized Investigation**: Config.toml types → **Parallel Explore subagents** → evidence-based verification → aggregated results
- **Parameterized Subagent**: Single subagent type handles all steering types via prompt parameters
- **Multi-Hypothesis Verification**: Each investigator maintains 3-7 competing theories during verification
- **Evidence Chain Documentation**: Subagents document complete evidence trails for all findings
- **Batch Update Flow**: Collect all changes → display detailed summary → single confirmation → batch apply
- **Concurrent Execution**: Multiple Task tools in single message → independent processing → synchronized aggregation
- ultrathink

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
