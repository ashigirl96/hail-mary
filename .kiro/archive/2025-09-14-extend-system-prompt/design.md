# Design

## Overview

Extend the system prompt generation to include steering information from the Kiro project configuration. This will embed project-specific context (product overview, technical stack, structure patterns, etc.) into Claude Code's system prompt, providing persistent contextual awareness across sessions.

## Architecture

### Component Overview

```
SystemPrompt Generation Flow:
config.toml → SteeringConfig → SteeringContent → SystemPromptTemplate → SystemPrompt
    |              |                |                      |
    v              v                v                      v
[steering]    Parse types     Read .md files       Embed in template
  types       & criteria       from disk            with formatting
```

### Key Components

1. **SteeringConfig Reader**
   - Parse `.kiro/config.toml` to extract steering types
   - Extract name, purpose, criteria for each steering type
   - Handle missing or malformed configuration gracefully

2. **SteeringContent Collector**
   - For each steering type, read corresponding `.kiro/steering/{name}.md`
   - Handle missing files gracefully (skip if not found)
   - Collect content with associated metadata

3. **SystemPrompt Enhancer**
   - Extend existing `SystemPrompt::new()` to accept optional steering content
   - Format steering information in XML-tagged structure
   - Maintain backward compatibility

## Technical Design

### Data Structures

```rust
// In domain/entities/steering.rs

// Simple structure to combine SteeringType with its file content
pub struct Steering {
    pub steering_type: SteeringType,  // Reuse existing type (has name, purpose, criteria)
    pub content: String,               // Content from .kiro/steering/{name}.md
}

impl fmt::Display for Steering {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let criteria_str = self.steering_type.criteria
            .iter()
            .map(|c| format!("- {}", c))  // Uses Criterion's Display impl
            .collect::<Vec<_>>()
            .join("\n");

        // Using named arguments for better readability
        write!(
            f,
            "name: {name}\ncriteria:\n{criteria}\ncontent:\n{content}",
            name = self.steering_type.name,
            criteria = criteria_str,
            content = self.content
        )
    }
}

// Wrapper for Vec<Steering> to provide Display implementation
pub struct Steerings(pub Vec<Steering>);

impl fmt::Display for Steerings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            return Ok(());  // Return empty string for empty steerings
        }

        let steerings_str = self.0
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join("\n-----\n");

        write!(f, "<steering>\n{}\n</steering>", steerings_str)
    }
}
```

### Benefits of This Approach

1. **Clean Separation of Concerns**:
   - `Steering`: Represents a single steering type with its content
   - `Steerings`: Handles formatting multiple steerings with XML tags
   - Display trait provides consistent, testable formatting

2. **Reuses Existing Infrastructure**:
   - `SteeringType`: Already has name, purpose, criteria
   - `Criterion::Display`: Already formats as "Name: Description"
   - `ProjectConfig.steering`: Already loaded from TOML

3. **Simple Integration**:
   - SystemPrompt just calls `Steerings(vec).to_string()`
   - No separate formatting function needed
   - Easy to test Display implementations independently

### Implementation Approach

1. **Extend SystemPrompt::new()**
   ```rust
   pub fn new(
       spec_name: &str,
       spec_path: &Path,
       steerings: &Steerings
   ) -> Self {
       let path_str = spec_path.display().to_string();

       // Format steering content using Display trait
       let steering_content = steerings.to_string();

       let content = SYSTEM_PROMPT_TEMPLATE
           .replace("{spec_name}", spec_name)
           .replace("{path_str}", &path_str)
           .replace("{steering_content}", &steering_content);

       Self { content }
   }
   ```

2. **Add steering loading in use cases**
   - Access `SteeringConfig` from `ProjectConfig.steering` (already loaded)
   - For each `SteeringType` in config.steering.types, read corresponding `.kiro/steering/{name}.md`
   - Create `Steering` struct combining the type info with file content
   - Create `Steerings` wrapper and pass reference to SystemPrompt constructor
   - If no steering files exist, pass `&Steerings(vec![])` (empty but valid)

3. **Template Enhancement**
   - Add new section in `system_prompt_template.md` for steering
   - Use XML tags for structured content embedding

### Template Integration

The steering section will be added to `system_prompt_template.md` file after the RULES section:

```markdown
# Kiro Specification Context

[... existing content ...]

## RULES

- **DO NOT read memo.md**: The memo.md file contains internal developer notes...

## Steering

The following <steering> section contains Kiro-specific context and knowledge that should guide your work:
- **PRD creation**: Product Requirements Document creation
- **Investigation**: Research and analysis phases
- **Planning**: Design and architecture decisions
- **Implementation**: Actual coding work

<steering>
{steering_content}
</steering>
```

Where `{steering_content}` will be replaced with formatted steering information:

```markdown
name: product
criteria:
- Product Overview: Brief description of what the product is
- Core Features: Bulleted list of main capabilities
- Target Use Case: Specific scenarios the product addresses
- Key Value Proposition: Unique benefits and differentiators
content:
[Content from .kiro/steering/product.md]

-----
name: tech
criteria:
- Architecture: High-level system design
- Frontend: Frameworks, libraries, build tools (if applicable)
[...]
content:
[Content from .kiro/steering/tech.md]

-----
[Additional steering types...]
```

## Implementation Strategy

### Phase 1: Core Implementation
1. Add `Steering` struct and `Steerings` wrapper with Display implementations in domain/entities/steering.rs
2. Add method to ProjectRepository trait for reading steering files
3. Implement steering file reader in FilesystemProjectRepository
4. Extend SystemPrompt::new() to accept `&Steerings` reference

### Phase 2: Integration
1. Modify launch_claude_with_spec use case to load steering files
2. Update system_prompt_template.md to include `{steering_content}` placeholder
3. Update SystemPrompt::new() to replace `{steering_content}` with formatted steering data
4. Add error handling for missing files

### Phase 3: Testing
1. Unit tests for steering readers
2. Integration tests for system prompt generation
3. End-to-end test for code command with steering

## Error Handling

- Missing `.kiro/config.toml`: Skip steering entirely (backward compatibility)
- Missing steering files: Skip individual file, continue with others
- Malformed TOML: Log warning, skip steering
- File read errors: Log error, skip affected file

## Backward Compatibility

- System remains functional without steering configuration
- Existing specs work without modification
- Optional parameter ensures no breaking changes

## Performance Considerations

- Lazy loading: Only read steering files when needed
- Caching: Consider caching parsed steering for session
- File size limits: Implement reasonable limits for steering files

