# Design: Interactive Steering Reminder System

## Overview
A Claude Code hook-integrated system that analyzes user input and automatically reminds users of relevant steering sections using AI-powered semantic understanding. The system follows clean architecture principles, maintaining clear separation between domain logic, application orchestration, and infrastructure concerns.

## Architecture

### System Components

```
┌─────────────────────────────────────────────────────┐
│                  Claude Code                         │
│  ┌────────────────────────────────────────────┐     │
│  │          UserPromptSubmit Hook             │     │
│  └────────────────┬───────────────────────────┘     │
│                   │                                  │
│                   ▼                                  │
│  ┌────────────────────────────────────────────┐     │
│  │    hail-mary steering remind --stdin       │     │
│  └────────────────┬───────────────────────────┘     │
└───────────────────┼──────────────────────────────────┘
                    │
┌───────────────────▼──────────────────────────────────┐
│                 Hail-Mary CLI                        │
├───────────────────────────────────────────────────────┤
│              CLI Layer (commands)                     │
│  ┌────────────────────────────────────────────┐     │
│  │        SteeringCommand::Remind             │     │
│  └────────────────┬───────────────────────────┘     │
├───────────────────┼───────────────────────────────────┤
│           Application Layer (use cases)               │
│  ┌────────────────▼───────────────────────────┐     │
│  │        remind_steering()                   │     │
│  │  - Load steering files                     │     │
│  │  - Call Anthropic API                      │     │
│  │  - Filter by confidence                    │     │
│  └────┬─────────────────────┬─────────────────┘     │
├───────┼─────────────────────┼─────────────────────────┤
│       ▼                     ▼                        │
│  Repository Interfaces (traits)                      │
│  ┌──────────────┐  ┌──────────────────┐            │
│  │SteeringRepo  │  │AnthropicRepo     │            │
│  └──────┬───────┘  └────────┬─────────┘            │
├─────────┼───────────────────┼─────────────────────────┤
│         ▼                   ▼                        │
│    Infrastructure Layer (implementations)            │
│  ┌──────────────┐  ┌──────────────────┐            │
│  │FileSystem    │  │AnthropicClient   │            │
│  │SteeringRepo  │  │Repository        │            │
│  └──────────────┘  └────────┬─────────┘            │
└─────────────────────────────┼─────────────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │ anthropic-client │
                    │   (Pure API)      │
                    └──────────────────┘
```

## Component Design

### 1. Anthropic Client Enhancement
**Location**: `crates/anthropic-client/src/lib.rs`
**Responsibility**: Pure API communication without domain knowledge

```rust
pub struct Message {
    pub role: String,
    pub content: String,
}

pub async fn complete_with_system(
    model: &str,
    system_prompts: Vec<String>,
    messages: Vec<Message>,
    auth: &mut OAuthAuth,
) -> Result<String>
```

**Design Decisions**:
- Remove hardcoded system prompts
- Accept system prompts as parameters
- Support multiple message exchanges
- Maintain OAuth token management

### 2. Domain Models
**Location**: `crates/hail-mary/src/domain/entities/steering_reminder.rs`
**Purpose**: Core domain entity for steering reminders

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteeringReminder {
    pub steering_name: String,
    pub relevant_sections: Vec<String>,
    pub reasoning: String,
    pub confidence: f64,
}
```

**Location**: `crates/hail-mary/src/domain/value_objects/steering_analysis_prompt.rs`
**Purpose**: Value object for constructing steering analysis prompts

```rust
pub struct SteeringAnalysisPrompt {
    steering_contents: HashMap<String, String>,
}

impl SteeringAnalysisPrompt {
    pub fn new(steering_contents: HashMap<String, String>) -> Self {
        Self { steering_contents }
    }

    pub fn build_system_prompt(&self) -> String {
        let steering_section = self.steering_contents
            .iter()
            .map(|(name, content)| format!("=== STEERING: {} ===\n{}\n", name, content))
            .collect::<Vec<_>>()
            .join("\n");

        format!(r#"You are a steering analyzer for the hail-mary project.
Analyze user input and determine which steering sections are relevant.

Available steering documents:
{}

For each relevant steering (confidence > 0.7), output:
Remember: <steering-NAME>
sections: relevant section headers from the steering document
reasoning: why this steering is relevant and what specific knowledge to apply"#, steering_section)
    }
}
```

### 3. Repository Pattern Implementation
**Location**: `crates/hail-mary/src/application/repositories/`
**Purpose**: Define contracts for external service interactions

```rust
// anthropic_repository.rs
use crate::domain::entities::SteeringReminder;

#[async_trait]
pub trait AnthropicRepositoryInterface {
    async fn analyze_steering_relevance(
        &self,
        user_input: &str,
        steering_contents: HashMap<String, String>,
    ) -> Result<Vec<SteeringReminder>>;
}

```

**Location**: `crates/hail-mary/src/infrastructure/repositories/anthropic.rs`
**Purpose**: Concrete implementation of Anthropic integration

```rust
use crate::domain::value_objects::SteeringAnalysisPrompt;
use crate::domain::entities::SteeringReminder;

pub struct AnthropicRepository {
    client: anthropic_client::Client,
}

impl AnthropicRepository {
    async fn analyze_steering_relevance(
        &self,
        user_input: &str,
        steering_contents: HashMap<String, String>,
    ) -> Result<Vec<SteeringReminder>> {
        // Use value object to build prompt
        let prompt_builder = SteeringAnalysisPrompt::new(steering_contents);
        let system_prompt = prompt_builder.build_system_prompt();

        // Call anthropic-client with system prompt
        let response = self.client.complete_with_system(
            "claude-3-haiku-20240307",
            vec![system_prompt],
            vec![Message { role: "user".to_string(), content: user_input.to_string() }],
        ).await?;

        // Parse response into domain entities
        self.parse_response(&response)
    }

    fn parse_response(&self, response: &str) -> Result<Vec<SteeringReminder>> {
        // Parse structured output format into SteeringReminder entities
    }
}
```

### 4. Use Case Layer
**Location**: `crates/hail-mary/src/application/use_cases/remind_steering.rs`
**Purpose**: Orchestrate business logic

```rust
use crate::domain::entities::SteeringReminder;
use crate::application::repositories::{
    SteeringRepositoryInterface,
    AnthropicRepositoryInterface,
};

pub async fn remind_steering(
    user_input: &str,
    steering_repo: &impl SteeringRepositoryInterface,
    anthropic_repo: &impl AnthropicRepositoryInterface,
) -> Result<Vec<SteeringReminder>> {
    // 1. Load all steering markdown files
    let steering_contents = steering_repo.load_all_steering_files()?;

    // 2. Analyze with AI
    let recommendations = anthropic_repo
        .analyze_steering_relevance(user_input, steering_contents)
        .await?;

    // 3. Apply business rules (confidence threshold)
    Ok(recommendations
        .into_iter()
        .filter(|r| r.confidence > 0.7)
        .collect())
}
```

### 4. CLI Command Structure
**Location**: `crates/hail-mary/src/cli/commands/steering.rs`
**Purpose**: User interface for steering operations

```rust
#[derive(Subcommand)]
pub enum SteeringCommands {
    /// Remind relevant steering sections based on input
    Remind {
        /// Input text to analyze
        #[arg(value_name = "INPUT")]
        input: Option<String>,

        /// Read input from stdin
        #[arg(long, short)]
        stdin: bool,

        /// Output format (text, json)
        #[arg(long, default_value = "text")]
        format: OutputFormat,
    },

    /// Existing backup command
    Backup {
        #[arg(long)]
        max_backups: Option<usize>,
    },
}
```

## Data Flow

### Request Flow
1. User types in Claude Code
2. UserPromptSubmit hook intercepts
3. Hook calls `hail-mary steering remind --stdin`
4. CLI command invokes use case
5. Use case loads steering files via SteeringRepository
6. Use case calls AnthropicRepository for analysis
7. Haiku analyzes with full markdown context
8. Response parsed into SteeringReminder objects
9. Filtered reminders returned to hook
10. Hook outputs formatted reminders to Claude Code context

### Output Format
```
Remember: <steering-tech>
sections: Development Commands, Common Commands
reasoning: User query about testing requires 'just test' command pattern

Remember: <steering-documentation>
sections: Code Block Formatting
reasoning: Output will be in markdown requiring 4 backticks for nesting
```

## Configuration

### Hook Configuration
**File**: `.claude/config.json`
```json
{
  "hooks": {
    "UserPromptSubmit": [{
      "type": "command",
      "command": "hail-mary steering remind --stdin --format text"
    }]
  }
}
```

### Environment Variables
```bash
# Optional: Override model selection
STEERING_MODEL=claude-3-haiku-20240307

# Optional: Confidence threshold
STEERING_CONFIDENCE_THRESHOLD=0.7
```

## Technical Decisions

### 1. Full Markdown Transmission
**Decision**: Send complete steering markdown content to Haiku
**Rationale**:
- Higher accuracy with full context
- Simple implementation without preprocessing
- Token usage within Haiku limits (15K tokens << 200K limit)
- Cost acceptable (~$0.006 per request)

### 2. Repository Pattern for External Services
**Decision**: Create AnthropicRepository in infrastructure layer
**Rationale**:
- Maintains clean architecture boundaries
- Enables testing with mock implementations
- Consistent with existing project patterns
- Allows future AI service substitution

### 3. Use Case Function vs Service Class
**Decision**: Use function-based use case pattern
**Rationale**:
- Consistent with existing codebase (initialize_project, create_feature)
- Simpler than class-based services
- Easier dependency injection for testing

### 4. Synchronous Hook Execution
**Decision**: Hook waits for AI response before proceeding
**Rationale**:
- Ensures steering reminder appears before Claude processes input
- Acceptable latency (100-300ms) for improved accuracy
- Prevents race conditions

## Testing Strategy

### Unit Tests
```rust
// Test use case with mock repositories
#[cfg(test)]
mod tests {
    use crate::domain::entities::SteeringReminder;

    struct MockAnthropicRepository {
        expected_reminders: Vec<SteeringReminder>,
    }

    #[async_trait]
    impl AnthropicRepositoryInterface for MockAnthropicRepository {
        async fn analyze_steering_relevance(...) -> Result<Vec<SteeringReminder>> {
            Ok(self.expected_reminders.clone())
        }
    }
}
```

### Integration Tests
```rust
// Test actual Anthropic API calls
#[tokio::test]
#[ignore] // Run with --ignored flag
async fn test_anthropic_steering_analysis() {
    let repo = AnthropicRepository::new();
    let result = repo.analyze_steering_relevance(
        "how to run tests",
        load_test_steering_files(),
    ).await;

    assert!(result.unwrap().iter().any(|r| r.steering_name == "tech"));
}
```

### Manual Testing
```bash
# Test CLI directly
echo "implement new feature" | hail-mary steering remind --stdin

# Test with explicit input
hail-mary steering remind "how to run tests"

# Test JSON output
echo "test query" | hail-mary steering remind --stdin --format json
```

## Performance Considerations

### Latency
- Hook execution: ~100-300ms (acceptable for interactive use)
- Haiku API call: ~100-200ms
- File loading: <10ms
- Total overhead: ~300ms worst case

### Token Usage
- Input: ~15,000 tokens (all steering files)
- Output: ~200 tokens
- Cost: ~$0.006 per invocation

### Caching Strategy
- No caching initially (steering files may change)
- Future: Cache steering file contents for session duration
- Future: Cache common query patterns

## Security Considerations

### API Key Management
- Use existing anthropic-client OAuth token management
- No API keys in code or configuration
- Tokens refreshed automatically

### Input Validation
- Sanitize user input before sending to API
- Limit input length to prevent abuse
- No execution of user-provided code

## Future Enhancements

### Phase 1 (Current)
- Basic steering reminder via Haiku
- Text output format
- Manual testing

### Phase 2
- JSON output for programmatic use
- Caching for performance
- Metrics collection

### Phase 3
- Embedding-based pre-filtering with fastembed-rs
- Multiple AI model support
- Learning from user feedback

## Migration Path

### Step 1: Anthropic Client Update
1. Add new `complete_with_system` function
2. Maintain backward compatibility
3. Update tests

### Step 2: Repository Implementation
1. Create repository interfaces
2. Implement AnthropicRepository
3. Add to dependency injection

### Step 3: CLI Command
1. Add steering remind subcommand
2. Wire up to use case
3. Add output formatting

### Step 4: Hook Integration
1. Create shell script for hook
2. Test with Claude Code
3. Document configuration

