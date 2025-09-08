---
description: Update all steering documents based on config.toml criteria
allowed-tools: Bash, Read, Write, Edit, MultiEdit, Glob, Grep, LS
---

# Kiro Steering Management

Update steering documents in `.kiro/steering/` based on types defined in @.kiro/config.toml. This command analyzes the project against each type's criteria and maintains accurate project knowledge.

## Configuration Check

### Current config.toml status
- Config file: !`[ -f ".kiro/config.toml" ] && echo "✅ Config exists" || echo "❌ Config missing - run 'hail-mary init' first"`
- Steering types defined: !`if [ -f ".kiro/config.toml" ]; then grep -c "^\[\[steering.types\]\]" .kiro/config.toml 2>/dev/null || echo "0"; else echo "Config not found"; fi`

### Current steering files
- Existing files: !`if [ -d ".kiro/steering" ]; then ls -1 .kiro/steering/*.md 2>/dev/null | xargs -I {} basename {} | tr '\n' ' ' || echo "No steering files yet"; else echo "Steering directory not found"; fi`

## Project Analysis

### Current Project State
- Project files: !`find . -path ./node_modules -prune -o -path ./.git -prune -o -path ./dist -prune -o -type f \( -name "*.py" -o -name "*.js" -o -name "*.ts" -o -name "*.jsx" -o -name "*.tsx" -o -name "*.java" -o -name "*.go" -o -name "*.rs" \) -print 2>/dev/null | head -20 || echo "No source files found"`
- Configuration files: !`find . -maxdepth 3 \( -name "package.json" -o -name "requirements.txt" -o -name "pom.xml" -o -name "Cargo.toml" -o -name "go.mod" -o -name "pyproject.toml" -o -name "tsconfig.json" \) 2>/dev/null || echo "No config files found"`
- Documentation: !`find . -maxdepth 3 -path ./node_modules -prune -o -path ./.git -prune -o -path ./.kiro -prune -o \( -name "README*" -o -name "CHANGELOG*" -o -name "LICENSE*" -o -name "*.md" \) -print 2>/dev/null | head -10 || echo "No documentation files found"`

### Recent Changes (if updating)
- Last steering update: !`git log -1 --oneline -- .kiro/steering/ 2>/dev/null || echo "No previous steering commits"`
- Recent commits: !`git log --oneline -10 2>/dev/null || echo "Not a git repository"`
- Working tree status: !`git status --porcelain 2>/dev/null | head -10 || echo "Not a git repository"`

### Existing Documentation References
- Main README: @README.md
- Package configuration: @package.json
- Cargo configuration: @Cargo.toml
- Project documentation: @docs/

## Config.toml Structure

This command reads all steering types from @.kiro/config.toml:

```toml
[[steering.types]]
name = "bigquery"                           # Filename: bigquery.md
purpose = "BigQuery optimization patterns"  # Description for user prompts
criteria = [                                # Analysis patterns for this type
    "Query Optimization: Performance techniques",
    "EXTERNAL_QUERY: Cloud SQL patterns",
    "Cost Management: Query cost strategies",
    "Common Pitfalls: Known issues and solutions"
]
```

### Property Details
- **`name`**: Determines the steering filename (`{name}.md`)
- **`purpose`**: Human-readable description of the type's focus area
- **`criteria`**: List of patterns used to analyze and categorize project content

## Task: Update All Steering Types from Config

### 1. Load Types from Config
Use **Read** tool to load @.kiro/config.toml and process all `[[steering.types]]` entries.

### 2. For Each Type in Config

Process each type defined in config.toml:

#### Analysis Phase
- Use **Grep** to search for patterns matching the type's criteria
- Use **Glob** to find relevant files based on type context
- Use **Bash** to check file existence and git history

#### Update Phase
For each {type.name}.md file:

**If NEW file:**
- Use **Write** to create comprehensive initial content
- Include all criteria as section headers
- Generate content based on project analysis

**If EXISTING file:**
- Use **Read** to load current content
- Use **MultiEdit** to update content
- Backup preserved in .kiro/steering/backup/ before modification

### 3. Detect Uncategorized Patterns

Analyze project for patterns not matching any existing criteria:
- Use **Glob** and **Grep** to find unmatched patterns
- If significant patterns found, suggest new types:
  ```
  > 🔍 Found patterns not matching existing types:
  > - SQL queries and database migrations
  > - Docker configuration files
  > 
  > Add new types? Suggestions:
  > 1. database - Database schemas and queries
  > 2. deployment - Docker and CI/CD configuration
  > 3. Skip for now
  > 
  > Select [1-3]: 
  ```
- If user selects a new type:
  - Use **MultiEdit** to add `[[steering.types]]` to config.toml
  - Use **Write** to create new steering file

## Tool Usage

- **Read**: Load @.kiro/config.toml and existing steering files
- **Glob**: Find project files for analysis (*.py, *.js, *.ts, *.sql, etc.)
- **Grep**: Search for patterns matching criteria
- **Write**: Create new steering files for new types
- **MultiEdit**: Update multiple sections in existing files efficiently
- **Edit**: Make targeted updates to specific sections
- **Bash**: Check file existence, git history, and directory structure
- **LS**: List files in directories for analysis

## Update Strategy

### Smart Content Updates
1. **Create backup** - Copy existing files to .kiro/steering/backup/
2. **Update factual information** - Dependencies, file structures, commands
3. **Add new sections** - Only if significant new capabilities exist
4. **Replace outdated content** - Remove obsolete information
5. **Maintain clear structure** - Use consistent markdown formatting

### Example Type Processing

The command processes each type from config.toml systematically:
1. Search for patterns matching the type's criteria using **Grep**
2. Check if {type.name}.md exists using **Bash**
3. Either create new content or update existing file
4. Structure content around the defined criteria
5. Include concrete examples from the codebase

## Important Principles

### Security Guidelines
- **Never include sensitive data**: No API keys, passwords, database credentials, or personal information
- **Review before commit**: Always review steering content before version control
- **Team sharing consideration**: Remember steering files are shared with all project collaborators

### Content Quality Guidelines
- **Single domain focus**: Each steering file covers one specific area defined by its type
- **Clear, descriptive content**: Provide concrete examples and rationale for decisions
- **Regular maintenance**: Review and update steering files after major project changes
- **Actionable guidance**: Write specific, implementable guidelines rather than abstract principles

### Backup Strategy
- **Pre-update backup**: All existing files copied to .kiro/steering/backup/
- **Recovery option**: Users can restore from backup if needed
- **Git integration**: Changes are trackable through version control

### Update Philosophy
- **Fresh content**: Generate current, relevant information
- **Clear communication**: Use straightforward language
- **Factual accuracy**: Document what actually exists in the project

## Features

- **Dynamic type loading**: All types loaded from @.kiro/config.toml
- **Configuration-driven**: All types processed based on config.toml definitions
- **New type detection**: Automatically suggests new types based on uncategorized patterns
- **Backup-based safety**: Original content preserved in backup directory
- **Incremental updates**: Only updates what has changed
- **Git-aware**: Uses git history to understand project evolution

The goal is to maintain living documentation that adapts to your project's growth while preserving your customizations and insights.