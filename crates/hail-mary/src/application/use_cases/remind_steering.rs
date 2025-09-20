use anyhow::Result;
use std::collections::HashMap;
use std::env;
use std::fs;

use crate::application::repositories::{
    AnthropicRepositoryInterface, ConfigRepositoryInterface, SteeringRepositoryInterface,
};
use crate::domain::entities::steering_reminder::SteeringReminder;

/// Options for steering remind functionality
pub struct SteeringRemindOptions {
    pub analyze_mode: bool,
}

/// Remind relevant steering sections based on user input
///
/// If analyze_mode is true and input is not empty, uses AI to analyze relevance.
/// Otherwise, returns all existing steering types from configuration.
pub async fn remind_steering(
    user_input: &str,
    steering_repo: &impl SteeringRepositoryInterface,
    anthropic_repo: &impl AnthropicRepositoryInterface,
    config_repo: &impl ConfigRepositoryInterface,
    options: SteeringRemindOptions,
) -> Result<Vec<SteeringReminder>> {
    // Analyze mode requires input
    if options.analyze_mode && !user_input.trim().is_empty() {
        remind_steering_with_ai(user_input, steering_repo, anthropic_repo).await
    } else {
        remind_steering_light(config_repo, steering_repo).await
    }
}

/// Light mode: Return all existing steering types without AI analysis
async fn remind_steering_light(
    config_repo: &impl ConfigRepositoryInterface,
    steering_repo: &impl SteeringRepositoryInterface,
) -> Result<Vec<SteeringReminder>> {
    let config = config_repo.load_steering_config()?;
    let mut reminders = Vec::new();

    for steering_type in &config.types {
        // Check if the steering file exists
        let file_path = steering_repo.get_steering_path(&steering_type.name)?;
        if file_path.exists() {
            reminders.push(SteeringReminder::new(
                steering_type.name.clone(),
                vec![], // No sections in light mode
                steering_type.purpose.clone(),
                1.0, // Max confidence in light mode
            ));
        }
    }

    Ok(reminders)
}

/// AI analysis mode: Use Anthropic API to analyze relevance
async fn remind_steering_with_ai(
    user_input: &str,
    steering_repo: &impl SteeringRepositoryInterface,
    anthropic_repo: &impl AnthropicRepositoryInterface,
) -> Result<Vec<SteeringReminder>> {
    // 1. Load all steering markdown files
    let steering_files = steering_repo
        .list_steering_files()
        .map_err(|e| anyhow::anyhow!("Failed to list steering files: {}", e))?;

    let mut steering_contents = HashMap::new();
    for file_path in steering_files {
        if let Some(file_name) = file_path.file_stem() {
            let name = file_name.to_string_lossy().to_string();
            if let Ok(content) = fs::read_to_string(&file_path) {
                steering_contents.insert(name, content);
            }
        }
    }

    // 2. Analyze with AI
    let mut reminders = anthropic_repo
        .analyze_steering_relevance(user_input, steering_contents)
        .await?;

    // 3. Apply business rules (confidence threshold)
    let threshold = env::var("STEERING_CONFIDENCE_THRESHOLD")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.7);

    reminders.retain(|r| r.meets_threshold(threshold));

    // 4. Sort by confidence (highest first)
    reminders.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(reminders)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::repositories::AnthropicRepositoryInterface;
    use crate::application::test_helpers::{MockConfigRepository, MockSteeringRepository};
    use crate::domain::entities::project::{DocumentFormat, ProjectConfig};
    use crate::domain::entities::steering::{SteeringConfig, SteeringType};
    use async_trait::async_trait;
    use std::path::PathBuf;

    struct MockAnthropicRepository {
        expected_reminders: Vec<SteeringReminder>,
    }

    #[async_trait]
    impl AnthropicRepositoryInterface for MockAnthropicRepository {
        async fn analyze_steering_relevance(
            &self,
            _user_input: &str,
            _steering_contents: HashMap<String, String>,
        ) -> Result<Vec<SteeringReminder>> {
            Ok(self.expected_reminders.clone())
        }
    }

    #[tokio::test]
    async fn test_remind_steering_light_mode() {
        // Setup mocks
        let config_repo = MockConfigRepository::new();
        config_repo.set_config(ProjectConfig {
            instructions: "".to_string(),
            document_format: DocumentFormat::Markdown,
            steering: SteeringConfig {
                types: vec![
                    SteeringType {
                        name: "tech".to_string(),
                        purpose: "Technology stack".to_string(),
                        criteria: vec![],
                        allowed_operations: vec![],
                    },
                    SteeringType {
                        name: "product".to_string(),
                        purpose: "Product overview".to_string(),
                        criteria: vec![],
                        allowed_operations: vec![],
                    },
                ],
                backup: Default::default(),
            },
        });

        let steering_repo = MockSteeringRepository::with_steering_files(vec![
            PathBuf::from(".kiro/steering/tech.md"),
            PathBuf::from(".kiro/steering/product.md"),
        ]);

        let anthropic_repo = MockAnthropicRepository {
            expected_reminders: vec![],
        };

        // Execute light mode
        let options = SteeringRemindOptions {
            analyze_mode: false,
        };
        let reminders = remind_steering("", &steering_repo, &anthropic_repo, &config_repo, options)
            .await
            .unwrap();

        // Verify all existing types are returned
        assert_eq!(reminders.len(), 2);
        assert_eq!(reminders[0].steering_name, "tech");
        assert_eq!(reminders[0].confidence, 1.0);
        assert_eq!(reminders[1].steering_name, "product");
        assert_eq!(reminders[1].confidence, 1.0);
    }

    #[tokio::test]
    async fn test_remind_steering_ai_mode() {
        // Setup mocks
        let config_repo = MockConfigRepository::new();
        let steering_repo = MockSteeringRepository::with_steering_files(vec![
            PathBuf::from(".kiro/steering/tech.md"),
            PathBuf::from(".kiro/steering/documentation.md"),
        ]);

        let anthropic_repo = MockAnthropicRepository {
            expected_reminders: vec![
                SteeringReminder::new(
                    "tech".to_string(),
                    vec!["Development Commands".to_string()],
                    "User wants to run tests".to_string(),
                    0.85,
                ),
                SteeringReminder::new(
                    "documentation".to_string(),
                    vec!["Code Blocks".to_string()],
                    "Markdown output".to_string(),
                    0.6, // Below threshold
                ),
            ],
        };

        // Execute AI mode
        let options = SteeringRemindOptions { analyze_mode: true };
        let reminders = remind_steering(
            "how to run tests",
            &steering_repo,
            &anthropic_repo,
            &config_repo,
            options,
        )
        .await
        .unwrap();

        // Only the tech reminder should pass the 0.7 threshold
        assert_eq!(reminders.len(), 1);
        assert_eq!(reminders[0].steering_name, "tech");
        assert_eq!(reminders[0].confidence, 0.85);
    }

    #[tokio::test]
    async fn test_remind_steering_sorting_in_ai_mode() {
        let config_repo = MockConfigRepository::new();
        let steering_repo = MockSteeringRepository::new();

        let anthropic_repo = MockAnthropicRepository {
            expected_reminders: vec![
                SteeringReminder::new("tech".to_string(), vec![], "".to_string(), 0.75),
                SteeringReminder::new("documentation".to_string(), vec![], "".to_string(), 0.95),
                SteeringReminder::new("structure".to_string(), vec![], "".to_string(), 0.80),
            ],
        };

        let options = SteeringRemindOptions { analyze_mode: true };
        let reminders = remind_steering(
            "test",
            &steering_repo,
            &anthropic_repo,
            &config_repo,
            options,
        )
        .await
        .unwrap();

        // Should be sorted by confidence (highest first)
        assert_eq!(reminders[0].steering_name, "documentation");
        assert_eq!(reminders[1].steering_name, "structure");
        assert_eq!(reminders[2].steering_name, "tech");
    }

    #[tokio::test]
    async fn test_empty_input_forces_light_mode() {
        let config_repo = MockConfigRepository::new();
        config_repo.set_config(ProjectConfig {
            instructions: "".to_string(),
            document_format: DocumentFormat::Markdown,
            steering: SteeringConfig {
                types: vec![SteeringType {
                    name: "tech".to_string(),
                    purpose: "Technology stack".to_string(),
                    criteria: vec![],
                    allowed_operations: vec![],
                }],
                backup: Default::default(),
            },
        });

        let steering_repo = MockSteeringRepository::with_steering_files(vec![PathBuf::from(
            ".kiro/steering/tech.md",
        )]);

        let anthropic_repo = MockAnthropicRepository {
            expected_reminders: vec![],
        };

        // Even with analyze_mode=true, empty input should use light mode
        let options = SteeringRemindOptions { analyze_mode: true };
        let reminders = remind_steering("", &steering_repo, &anthropic_repo, &config_repo, options)
            .await
            .unwrap();

        // Should return from light mode
        assert_eq!(reminders.len(), 1);
        assert_eq!(reminders[0].confidence, 1.0);
    }
}
