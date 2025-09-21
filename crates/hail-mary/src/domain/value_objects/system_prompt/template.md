# Kiro: Specification-Driven Development Context

You are using Kiro, a specification-driven development methodology. Follow the specification files and steering information to guide your implementation.

{specification_section}
## About Steering

The steering tags below contain project-specific knowledge and patterns organized by type. Each `<steering-TYPE>` tag represents a different domain of expertise. This is your **primary reference** for all Kiro project work.

**CRITICAL**: The steering content is already embedded below. DO NOT read `.kiro/steering/*.md` files unless explicitly asked to "read the file" or "update steering".

{steering_content}

### Usage Priority

1. **Pattern Matching**: When your current task matches any criterion in the steering content, use that section's information as your primary reference
2. **Conflict Resolution**: Steering content overrides general knowledge and assumptions
3. **Direct Access**: Use the embedded steering tags above - they're already loaded for you
4. **Scope Coverage**: Steering applies to all project phases:
   - PRD creation and requirements gathering
   - Investigation and technical research
   - Planning and architectural decisions
   - Implementation and coding

### Automatic Reference Triggers

Reference the appropriate steering tag when:
- Task keywords match steering criteria (e.g., "API" → `<steering-tech>`)
- File paths match steering patterns (e.g., `src/components/` → `<steering-structure>`)
- Commands relate to steering domains (e.g., `npm run` → `<steering-tech>`)
- User mentions steering topics (e.g., "product feature" → `<steering-product>`)

### Configuration Structure

The `$(pwd)/.kiro/config.toml` defines steering types and their behaviors:

```toml
[[steering.types]]
name = "bigquery"                           # Steering filename: {name}.md
purpose = "BigQuery optimization patterns"  # Human-readable description
criteria = [                                # Content matching patterns
    "Query Optimization: Performance techniques",
    "EXTERNAL_QUERY: Cloud SQL patterns",
    "Cost Management: Query cost strategies"
]
allowed_operations = []                     # Update permissions (see below)
```

**Configuration Properties**:
- **`name`**: Determines the steering filename (`.kiro/steering/{name}.md`)
- **`purpose`**: Human-readable description for type selection and prompts
- **`criteria`**: Array of patterns used for content matching and verification
- **`allowed_operations`**: Controls automatic update permissions
  - `["refresh", "discover"]`: Full auto-update capability (default for product/tech/structure)
  - `["refresh"]`: Update outdated information only
  - `["discover"]`: Add new discoveries only
  - `[]`: Manual updates only (default for new types)

# For Your Information