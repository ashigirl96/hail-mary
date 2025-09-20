# Steering Remind Lightweight Design

## Overview

To improve the performance of the `hail-mary steering remind` command, we will implement a lightweight mode that doesn't use the API by default, and switch to a 2-mode configuration that performs AI analysis only when necessary.

## Problem Definition

### Current Issues
- Slow response (1-3 seconds) due to calling Anthropic API (Claude 3.5 Haiku) every time
- API costs incurred even for simple steering type listing
- Delays in Git hook mode can harm user experience
- Excessive processing for daily use

### Expected Benefits
- Speed up default operation to under 50ms (20-60x improvement)
- Reduce API costs by over 90%
- Eliminate delays in Git hook execution
- Use AI analysis only when needed

## Architecture Design

### 1. Two-Mode Configuration

#### Light Mode (Default)
```
Processing Flow:
1. Load steering types from .kiro/config.toml
2. Check existence of .kiro/steering/{name}.md for each type
3. Display list of existing steering types
```

**Characteristics:**
- Execution time: <50ms
- API calls: None
- Cost: $0
- Use cases: Daily reference, Git hooks, quick reference

#### AI Analysis Mode (Optional)
```
Processing Flow:
1. Load all steering file contents
2. Analyze relevance with Anthropic API
3. Filter by confidence (threshold: 0.7)
4. Sort by relevance
```

**Characteristics:**
- Execution time: 1-3 seconds
- API calls: Claude 3.5 Haiku
- Cost: ~$0.0001/call
- Use cases: Context-dependent suggestions, problem-solving assistance

### 2. CLI Interface Changes

```rust
// cli/args.rs
#[derive(Args)]
pub struct SteeringRemindArgs {
    /// Input to analyze for steering relevance
    #[arg(conflicts_with = "hook")]
    pub input: Option<String>,

    /// Run in hook mode (read from stdin)
    #[arg(long, conflicts_with = "input")]
    pub hook: bool,

    /// Enable AI-powered relevance analysis
    #[arg(long, alias = "haiku")]
    pub analyze: bool,

    /// Output format
    #[arg(long, value_enum, default_value = "text")]
    pub format: OutputFormat,
}
```

**Command Examples:**
```bash
# Light mode (default)
hail-mary steering remind

# AI analysis mode
hail-mary steering remind --analyze "how to run tests"
hail-mary steering remind --haiku "performance optimization"

# Hook mode (automatically uses light mode)
echo "commit message" | hail-mary steering remind --hook
```

### 3. Use Case Layer Implementation

```rust
// application/use_cases/remind_steering.rs

pub enum SteeringRemindResult {
    Light {
        reminders: Vec<SteeringReminder>,
        mode: &'static str,
    },
    Analyzed {
        reminders: Vec<SteeringReminder>,
        analysis_time_ms: u64,
        model_used: String,
    },
}

pub async fn remind_steering(
    user_input: &str,
    steering_repo: &impl SteeringRepositoryInterface,
    anthropic_repo: &impl AnthropicRepositoryInterface,
    config_repo: &impl ConfigRepositoryInterface,
    options: SteeringRemindOptions,
) -> Result<SteeringRemindResult> {

    // Analyze mode requires input
    if options.analyze_mode && !user_input.trim().is_empty() {
        remind_steering_with_ai(user_input, steering_repo, anthropic_repo).await
    } else {
        remind_steering_light(config_repo, steering_repo).await
    }
}

// New: Light mode implementation
async fn remind_steering_light(
    config_repo: &impl ConfigRepositoryInterface,
    steering_repo: &impl SteeringRepositoryInterface,
) -> Result<SteeringRemindResult> {
    let config = config_repo.load_steering_config()?;
    let mut reminders = Vec::new();

    for steering_type in &config.types {
        // Check file existence
        let file_path = steering_repo.get_steering_path(&steering_type.name)?;
        if file_path.exists() {
            reminders.push(SteeringReminder {
                steering_name: steering_type.name.clone(),
                relevant_sections: vec![],  // Empty in light mode
                reasoning: steering_type.purpose.clone(),
                confidence: 1.0,  // Always max in light mode
            });
        }
    }

    Ok(SteeringRemindResult::Light {
        reminders,
        mode: "light"
    })
}

// Existing: AI analysis mode (minimal changes)
async fn remind_steering_with_ai(
    user_input: &str,
    steering_repo: &impl SteeringRepositoryInterface,
    anthropic_repo: &impl AnthropicRepositoryInterface,
) -> Result<SteeringRemindResult> {
    let start = std::time::Instant::now();

    // Maintain existing implementation
    let steering_files = steering_repo.list_steering_files()?;
    let mut steering_contents = HashMap::new();

    for file_path in steering_files {
        if let Some(file_name) = file_path.file_stem() {
            let name = file_name.to_string_lossy().to_string();
            if let Ok(content) = fs::read_to_string(&file_path) {
                steering_contents.insert(name, content);
            }
        }
    }

    let mut reminders = anthropic_repo
        .analyze_steering_relevance(user_input, steering_contents)
        .await?;

    // Threshold filtering
    let threshold = env::var("STEERING_CONFIDENCE_THRESHOLD")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.7);

    reminders.retain(|r| r.meets_threshold(threshold));
    reminders.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

    Ok(SteeringRemindResult::Analyzed {
        reminders,
        analysis_time_ms: start.elapsed().as_millis() as u64,
        model_used: env::var("STEERING_MODEL")
            .unwrap_or_else(|_| "claude-3-5-haiku-20241022".to_string()),
    })
}
```

### 4. Output Formats

#### Light Mode - Standard/Hook Output
```
📌 STEERING REMINDER

Available steering types for this project:
• <steering-product> - Product overview and requirements
• <steering-tech> - Technology stack and development environment
• <steering-structure> - Project structure and organization
• <steering-prompt-engineering> - Prompt engineering patterns
• <steering-documentation> - Documentation standards
• <steering-subagent> - Subagent patterns and best practices
• <steering-rust-dev> - Rust development tools and commands

💡 Use these tags to reference project context in Claude Code.
```

Note: In hook mode, this output appears before the original git message.

#### AI Analysis Mode - Standard Output
```
📌 STEERING REMINDER (AI Analysis)

Query: "how to run tests"

Highly relevant (85%+ confidence):
• <steering-tech>
  → Development Commands section
  → Testing Strategy section
  reasoning: User wants to know about test execution commands

Potentially relevant (70-84% confidence):
• <steering-rust-dev>
  → Test Organization section
  reasoning: Contains Rust-specific testing patterns

Analysis completed in 1.2s using claude-3-5-haiku-20241022
```

#### JSON Output Format
```json
{
  "mode": "light",
  "reminders": [
    {
      "steering_name": "tech",
      "tag": "<steering-tech>",
      "purpose": "Technology stack and development environment",
      "confidence": 1.0
    }
  ]
}
```

### 5. Performance Optimization

#### ConfigCache Implementation (Optional)
```rust
// infrastructure/caching/config_cache.rs
pub struct ConfigCache {
    config: Arc<RwLock<Option<(Config, Instant)>>>,
    ttl: Duration,
}

impl ConfigCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            config: Arc::new(RwLock::new(None)),
            ttl,
        }
    }

    pub fn get_or_load(&self, repo: &impl ConfigRepositoryInterface) -> Result<Config> {
        let now = Instant::now();

        // Check cache
        if let Some((config, loaded_at)) = &*self.config.read().unwrap() {
            if now.duration_since(*loaded_at) < self.ttl {
                return Ok(config.clone());
            }
        }

        // Load fresh
        let config = repo.load_config()?;
        *self.config.write().unwrap() = Some((config.clone(), now));
        Ok(config)
    }
}
```

### 6. Test Plan

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[tokio::test]
    async fn test_light_mode_performance() {
        // Setup
        let config_repo = MockConfigRepository::with_types(vec![
            ("tech", "Technology stack"),
            ("product", "Product overview"),
        ]);
        let steering_repo = MockSteeringRepository::with_files(vec![
            ".kiro/steering/tech.md",
            ".kiro/steering/product.md",
        ]);

        // Execute
        let start = Instant::now();
        let result = remind_steering_light(&config_repo, &steering_repo)
            .await
            .unwrap();

        // Verify: under 50ms
        assert!(start.elapsed() < Duration::from_millis(50));
        assert_eq!(result.reminders.len(), 2);
    }

    #[tokio::test]
    async fn test_mode_selection() {
        // analyze=false → light mode
        let options = SteeringRemindOptions {
            analyze_mode: false,
            is_hook: false,
        };
        // ... verify light mode is selected

        // analyze=true → AI analysis mode
        let options = SteeringRemindOptions {
            analyze_mode: true,
            is_hook: false,
        };
        // ... verify AI analysis mode is selected
    }

    #[tokio::test]
    async fn test_hook_mode_forces_light() {
        // hook=true → always light mode (ignores analyze flag)
        let options = SteeringRemindOptions {
            analyze_mode: true,  // Will be ignored
            is_hook: true,
        };
        // ... verify light mode is forced
    }

}
```

## Migration Plan

### Phase 1: Implementation (No Breaking Changes)
1. Implement `remind_steering_light()` function
2. Add `--analyze` flag (default: false)
3. Add tests

### Phase 2: Rollout
1. Update documentation
2. Release notes announcement
3. Usage monitoring (log mode in output)

### Phase 3: Optimization (After 2 weeks)
1. Analyze usage patterns
2. Consider cache implementation
3. Further optimizations

## Backward Compatibility

### Guaranteed Compatibility
- Existing Git hooks work without changes (automatically faster)
- `--analyze` flag maintains previous behavior
- Basic output format structure maintained

### Changes
- Default behavior changed to light mode
- AI analysis disabled in hook mode
- Significant reduction in execution time

## Metrics Comparison

| Item | Current | Light Mode | AI Analysis Mode |
|------|---------|------------|------------------|
| **Execution Time** | 1-3s | <50ms | 1-3s |
| **API Calls** | Every time | None | On demand |
| **Cost/Call** | $0.0001 | $0 | $0.0001 |
| **Accuracy** | High (relevance-sorted) | Show all | High (relevance-sorted) |
| **Hook Suitability** | △ | ◎ | × |

## Risks and Mitigations

### Risks
1. Users may not be satisfied with light mode output
2. Incorrect usage when AI analysis is needed

### Mitigations
1. Display hint: "--analyze for AI-powered suggestions" in output
2. Clear documentation explaining when to use each mode
3. Consider automatic switching based on usage patterns in the future

## Summary

This design provides:
- **20-60x speedup for 90%+ of use cases**
- **Significant API cost reduction**
- **Improved Git operation UX**
- **AI analysis still available when needed**

Following the existing clean architecture patterns, we achieve maximum effect with minimal changes.