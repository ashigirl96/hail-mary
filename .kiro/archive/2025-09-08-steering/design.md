# Design

## Overview

Two-command steering system with intelligent type detection and automatic configuration management through slash commands.

## Architecture

### Command Structure
```
.claude/commands/hm/
├── steering-remember.md   # Learning capture with type management
└── steering.md            # Comprehensive project analysis
```

### Key Features
- Dynamic type creation based on learning content
- Automatic config.toml management
- Interactive type matching and selection
- Criteria-based project analysis

## Slash Command Specifications

### `/hm:steering-remember`

#### Specification
```markdown
---
description: Save learning to steering with intelligent type detection and creation
allowed-tools: Read, Write, Edit, MultiEdit, Bash(date:*), Glob
argument-hint: [learning content] [--format rule|guide|knowledge] [--type <name>]
---

## Behavioral Flow
1. Extract Core Learning from conversation
2. Load types from @.kiro/config.toml using **Read** tool
3. Match learning against existing types
   - Match found (>70% confidence): 
     → Confirm with user
     → Append to existing file using **Edit** or **MultiEdit**
   - No match:
     → Suggest new type candidates
     → User selects or creates custom
     → Add type to config.toml using **MultiEdit**
     → Create new file using **Write**
4. Auto-detect content format (Rule/Guide/Knowledge)
5. Save to .kiro/steering/{type}.md

## Tool Usage
- **Read**: Load @.kiro/config.toml for type definitions
- **Glob**: Check existing steering files in .kiro/steering/*.md
- **Write**: Create new steering file when type doesn't exist
- **Edit/MultiEdit**: Append to existing steering file or update config.toml
- **Bash(date:*)**: Generate timestamp for tracking

## Interactive Prompts

### When Type Matches
> 🔍 Analyzing learning content...
> ✅ Found match: 'bigquery' type (confidence: 85%)
> 
> Append to bigquery.md? [Y/n]: 

### When Creating New Type
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

### Type Successfully Created
> 📝 Creating new type 'graphql'...
> ✅ Added to config.toml:
> [[steering.types]]
> name = "graphql"
> purpose = "GraphQL patterns and optimizations"
> criteria = [...]
> 
> ✅ Created graphql.md with your learning
```

### `/hm:steering`

#### Specification
````markdown
---
description: Update all steering documents based on config.toml criteria
allowed-tools: Bash, Read, Write, Edit, MultiEdit, Glob, Grep, LS
---

## Task: Update All Steering Types from Config

1. **Load all types from @.kiro/config.toml**
   - Use **Read** to load config.toml
   - Parse all [[steering.types]] entries
   
2. **For each type in config:**
   - Use **Grep** to find patterns matching type.criteria
   - Use **Read** to load existing {type.name}.md
   - Use **MultiEdit** to update content while preserving custom sections
   - Apply deprecation markers for outdated content

3. **Detect uncategorized patterns:**
   - Use **Glob** and **Grep** to find unmatched patterns
   - Suggest: "Found SQL patterns. Add 'database' type? [Y/n]"
   - Use **MultiEdit** to add new type to config.toml
   - Use **Write** to create new steering file

## Tool Usage
- **Read**: Load @.kiro/config.toml and existing steering files
- **Glob**: Find project files for analysis (*.py, *.js, *.ts, etc.)
- **Grep**: Search for patterns matching criteria
- **Write**: Create new steering files for new types
- **MultiEdit**: Update multiple sections in existing files
- **Bash**: Check file existence and git history

## Features
- Dynamic type loading from @.kiro/config.toml
- All types processed equally (no hardcoded assumptions)
- Automatic new type detection and suggestion
- Preservation of user customizations (CUSTOM:START/END blocks)
````

## File Structure

### Commands
```
.claude/commands/hm/
├── steering-remember.md   # Learning capture with type management
└── steering.md            # Project-wide analysis and updates
```

### Storage
```
.kiro/
├── config.toml           # Type definitions and criteria
└── steering/
    ├── product.md       # Core type
    ├── tech.md          # Core type
    ├── structure.md     # Core type
    └── {dynamic}.md     # User-created types
```

## Config.toml Structure

### Overview
The `.kiro/config.toml` file defines all steering types and their categorization criteria. Both commands interact with this configuration to manage knowledge systematically.

### Structure
```toml
[[steering.types]]
name = "product"                                    # Filename: product.md
purpose = "Product overview and value proposition"  # Human-readable description
criteria = [
    "Product Overview: What the product is",       # Matching patterns for categorization
    "Core Features: Main capabilities",            # Used by steering-remember for type detection
    "Target Use Case: Scenarios addressed",        # Used by steering for content analysis
    "Key Value Proposition: Unique benefits"       # Each criterion guides what content belongs
]
```

### Property Meanings

| Property | Purpose | Used By | Example |
|----------|---------|---------|---------|
| `name` | Determines steering filename (`{name}.md`) | Both commands | `"bigquery"` → `bigquery.md` |
| `purpose` | Human-readable type description | User selection prompts | `"BigQuery optimization patterns"` |
| `criteria` | List of categorization patterns | Type matching & analysis | `["Query Optimization: ...", "EXTERNAL_QUERY: ..."]` |

### How Commands Use Config.toml

#### `/hm:steering-remember`
1. **Read** tool loads @.kiro/config.toml
2. **Match** learning content against `criteria` arrays
3. **MultiEdit** tool adds new `[[steering.types]]` when creating types
4. **Write** tool creates new {name}.md file

```toml
# When user selects "Create new type: graphql"
[[steering.types]]
name = "graphql"        # → Creates graphql.md
purpose = "GraphQL..."  # → Shows in prompts
criteria = [...]        # → For future matching
```

#### `/hm:steering`
1. **Read** tool loads @.kiro/config.toml
2. **Grep** tool searches project using each type's `criteria`
3. **MultiEdit** tool updates {name}.md preserving custom sections
4. **Write** tool creates new files if new types detected

### Example: Complete Config.toml
```toml
# Core types (initialized by hail-mary init)
[[steering.types]]
name = "product"
purpose = "Product overview and value proposition"
criteria = [
    "Product Overview: Brief description of what the product is",
    "Core Features: Bulleted list of main capabilities",
    "Target Use Case: Specific scenarios the product addresses",
    "Key Value Proposition: Unique benefits and differentiators"
]

[[steering.types]]
name = "tech"
purpose = "Technical stack and development environment"
criteria = [
    "Architecture: High-level system design",
    "Frontend: Frameworks, libraries, build tools",
    "Backend: Language, framework, server technology",
    "Development Environment: Required tools and setup",
    "Common Commands: Frequently used development commands"
]

[[steering.types]]
name = "structure"
purpose = "Code organization and project structure patterns"
criteria = [
    "Root Directory Organization: Top-level structure",
    "Code Organization Patterns: How code is structured",
    "File Naming Conventions: Standards for naming",
    "Key Architectural Principles: Core design decisions"
]

# Dynamic types (added by steering-remember)
[[steering.types]]
name = "bigquery"
purpose = "BigQuery optimization and best practices"
criteria = [
    "Query Optimization: Performance tuning techniques",
    "EXTERNAL_QUERY: Cloud SQL connection patterns",
    "Cost Management: Query cost reduction strategies",
    "Common Pitfalls: Known issues and workarounds"
]
```

## Benefits of This Design

1. **Simpler Mental Model**: remember → save, steering → update
2. **No Intermediate State**: No draft accumulation to manage
3. **Immediate Persistence**: Learning available instantly
4. **Natural Growth**: Types expand through normal usage
5. **Git-Friendly**: Each learning is immediately committable