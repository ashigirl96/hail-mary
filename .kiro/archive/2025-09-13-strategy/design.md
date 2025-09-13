# Design

## Overview
Implement an update strategy system for steering types to control how the `/hm:steering` command updates different types of steering files. This allows fine-grained control over which steering files can be automatically refreshed or extended with new discoveries.

## Architecture

### Core Concept: allowed_operations
Each steering type will have an `allowed_operations` property in config.toml that defines which automatic update operations are permitted:

- `["refresh", "discover"]` - Both update existing and add new information (default for product, tech, structure)
- `["refresh"]` - Only update out-of-date information
- `["discover"]` - Only add new discoveries
- `[]` - No automatic updates (manual updates only via `/hm:steering-remember`)

### Component Design

#### 1. Configuration Structure (TOML)
```toml
[steering.product]
allowed_operations = ["refresh", "discover"]

[steering.tech]
allowed_operations = ["refresh", "discover"]

[steering.structure]
allowed_operations = ["refresh", "discover"]

[steering.principles]
allowed_operations = []  # Manual updates only

[steering.decisions]
allowed_operations = ["discover"]  # Only add new, don't modify existing
```

#### 2. Domain Model Updates

##### File: `crates/hail-mary/src/domain/entities/steering.rs`
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SteeringType {
    pub name: String,
    pub purpose: String,
    pub criteria: Vec<Criterion>,
    #[serde(default = "default_allowed_operations")]
    pub allowed_operations: Vec<String>,  // ["refresh", "discover"]
}

fn default_allowed_operations() -> Vec<String> {
    vec![]  // Default to manual-only for safety
}

impl SteeringConfig {
    pub fn default_for_new_project() -> Self {
        Self {
            backup: SteeringBackupConfig::default(),
            types: vec![
                SteeringType {
                    name: "product".to_string(),
                    purpose: "Product overview and value proposition".to_string(),
                    criteria: vec![/* existing criteria */],
                    allowed_operations: vec!["refresh".to_string(), "discover".to_string()],
                },
                SteeringType {
                    name: "tech".to_string(),
                    purpose: "Technical stack and development environment".to_string(),
                    criteria: vec![/* existing criteria */],
                    allowed_operations: vec!["refresh".to_string(), "discover".to_string()],
                },
                SteeringType {
                    name: "structure".to_string(),
                    purpose: "Code organization and project structure patterns".to_string(),
                    criteria: vec![/* existing criteria */],
                    allowed_operations: vec!["refresh".to_string(), "discover".to_string()],
                },
            ],
        }
    }
}
```

##### File: `crates/hail-mary/src/infrastructure/repositories/config.rs`
```rust
#[derive(Debug, Serialize, Deserialize)]
struct SteeringTypeToml {
    name: String,
    purpose: String,
    criteria: Vec<String>,
    #[serde(default)]
    allowed_operations: Vec<String>,  // New field
}

impl ConfigRepository {
    fn parse_steering_config(steering_section: &SteeringSection) -> SteeringConfig {
        let types = steering_section
            .types
            .iter()
            .map(|t| SteeringType {
                name: t.name.clone(),
                purpose: t.purpose.clone(),
                criteria: /* existing parsing */,
                allowed_operations: t.allowed_operations.clone(),
            })
            .collect();

        SteeringConfig { types, backup }
    }

    fn ensure_allowed_operations(&self) -> Result<(), ApplicationError> {
        let mut toml_value = self.load_toml()?;
        let table = toml_value.as_table_mut().ok_or_else(/* error */)?;

        if let Some(steering) = table.get_mut("steering")
            && let Some(steering_table) = steering.as_table_mut()
        {
            // Iterate through each steering type
            for (type_name, type_value) in steering_table.iter_mut() {
                if let Some(type_table) = type_value.as_table_mut() {
                    if !type_table.contains_key("allowed_operations") {
                        // Add default based on type name
                        let default_ops = match type_name.as_str() {
                            "product" | "tech" | "structure" =>
                                vec!["refresh".to_string(), "discover".to_string()],
                            _ => vec![],
                        };

                        let ops_value = toml::Value::Array(
                            default_ops.into_iter()
                                .map(toml::Value::String)
                                .collect()
                        );
                        type_table.insert("allowed_operations".to_string(), ops_value);
                    }
                }
            }

            self.save_toml(&toml_value)?;
        }

        Ok(())
    }
}
```

##### File: `crates/hail-mary/src/application/use_cases/initialize_project.rs`
```rust
pub fn initialize_project(
    config_repo: &dyn ConfigRepositoryInterface,
    _spec_repo: &dyn SpecRepositoryInterface,
    steering_repo: &dyn SteeringRepositoryInterface,
) -> Result<(), ApplicationError> {
    // Existing initialization steps...

    // NEW: Ensure allowed_operations exist for all steering types
    config_repo.ensure_allowed_operations()?;

    // Continue with existing steps...
    Ok(())
}
```

#### 3. Slash Command Updates

##### File: `.claude/commands/hm/steering-remember.md`
Update the type creation flow to automatically add `allowed_operations = []`:

```markdown
## Config.toml Structure
When creating a new steering type, the command adds:

[steering.{type_name}]
name = "{type_name}"
purpose = "{purpose}"
criteria = [...]
allowed_operations = []  # New types default to manual-only

## Behavioral Flow
Step 3: When creating new type:
- Automatically sets allowed_operations = [] for safety
- Documents in user feedback that automatic updates are disabled
- User can manually edit config.toml later to enable operations
```

##### File: `.claude/commands/hm/steering.md`
Update the investigation phase to respect `allowed_operations`:

```markdown
## Config.toml Structure
Each steering type can control automatic updates via allowed_operations:
- ["refresh", "discover"] - Both update and add (default for product/tech/structure)
- ["refresh"] - Only update existing information
- ["discover"] - Only add new discoveries
- [] - Skip automatic updates (manual-only via steering-remember)

## Behavioral Flow
Step 3: Investigation Phase
- Load allowed_operations for each type from config.toml
- Skip types with empty allowed_operations []
- For types with operations:
  - If "refresh" in allowed_operations: Check for outdated information
  - If "discover" in allowed_operations: Look for new patterns

Example output:
> Analyzing steering types...
> • product.md [refresh ✅, discover ✅] - Will check and update
> • tech.md [refresh ✅, discover ✅] - Will check and update
> • principles.md [skipped - no operations allowed]
> • decisions.md [discover ✅] - Will only add new content
```

#### 4. Command Behavior

##### `/hm:steering-remember` Command
- Always allows manual additions to any steering type (ignores `allowed_operations`)
- When creating new steering types, automatically sets `allowed_operations = []`
- Appends to existing files using Edit/MultiEdit tools
- Creates new files with Write tool when new type is created

##### `/hm:steering` Command
- Reads `allowed_operations` for each steering type from config.toml
- Skips types where `allowed_operations = []`
- For each type with operations:
  - If "refresh" in `allowed_operations`: Update out-of-date information
  - If "discover" in `allowed_operations`: Add new discoveries
- Reports which types were processed and which were skipped

##### `hail-mary init` Command
- When initializing a project:
  1. Creates default steering types if not present
  2. Ensures all steering types have `allowed_operations` property
  3. Sets defaults based on type name:
     - product, tech, structure → `["refresh", "discover"]`
     - All other types → `[]`
  4. Never overwrites existing `allowed_operations` values

### Implementation Strategy

#### Phase 1: Documentation Updates
1. **Update `.claude/commands/hm/steering-remember.md`**:
   - Add Config.toml Structure section explaining `allowed_operations` property
   - Document that new types default to `allowed_operations = []`
   - Update type creation flow to include the new property

2. **Update `.claude/commands/hm/steering.md`**:
   - Add Config.toml Structure section explaining `allowed_operations` property
   - Document the investigation phase changes to check `allowed_operations`
   - Add example output showing skipped types

#### Phase 2: Code Implementation

1. **Update domain model** (`crates/hail-mary/src/domain/entities/steering.rs`):
   - Add `allowed_operations: Vec<String>` field to `SteeringType` struct
   - Add `#[serde(default = "default_allowed_operations")]` attribute
   - Update `default_for_new_project()` to include allowed_operations for default types

2. **Update infrastructure** (`crates/hail-mary/src/infrastructure/repositories/config.rs`):
   - Add `allowed_operations` field to `SteeringTypeToml` struct
   - Update `parse_steering_config()` to handle the new field
   - Add `ensure_allowed_operations()` method to add missing properties
   - Update serialization logic to include allowed_operations

3. **Update init use case** (`crates/hail-mary/src/application/use_cases/initialize_project.rs`):
   - Call `ensure_allowed_operations()` after config initialization
   - Ensure idempotent behavior

4. **Update repository interface** (`crates/hail-mary/src/application/repositories/config_repository.rs`):
   - Add `ensure_allowed_operations()` to the trait

#### Phase 3: Testing
1. **Domain tests**:
   - Test SteeringType serialization/deserialization with allowed_operations
   - Test default values for allowed_operations

2. **Infrastructure tests**:
   - Test parsing config with and without allowed_operations
   - Test ensure_allowed_operations() adds correct defaults
   - Test idempotent behavior

3. **Integration tests**:
   - Test `hail-mary init` adds allowed_operations to existing configs
   - Test new projects get correct defaults

### Migration Path
For existing projects without `allowed_operations`:
1. `hail-mary init` will detect missing properties and add them
2. Preserves existing steering configuration while adding new properties
3. Non-destructive update that maintains backward compatibility

### Benefits
1. **Fine-grained control**: Each steering type can have different update policies
2. **Safe defaults**: New types default to manual-only updates
3. **Clear intent**: Configuration explicitly states what's allowed
4. **Preservation of critical content**: Principles, decisions, etc. can be protected
5. **Flexibility**: Easy to adjust behavior per type as needs evolve