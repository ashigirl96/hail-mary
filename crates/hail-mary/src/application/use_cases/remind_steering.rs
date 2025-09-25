use anyhow::Result;

use crate::application::repositories::{ConfigRepositoryInterface, SteeringRepositoryInterface};
use crate::domain::value_objects::steering_reminder::SteeringReminder;

/// Remind relevant steering sections based on user input
///
/// Returns all existing steering types from configuration.
pub async fn remind_steering(
    steering_repo: &impl SteeringRepositoryInterface,
    config_repo: &impl ConfigRepositoryInterface,
) -> Result<Vec<SteeringReminder>> {
    remind_steering_light(config_repo, steering_repo).await
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_helpers::{MockConfigRepository, MockSteeringRepository};
    use crate::domain::value_objects::steering::{SteeringConfig, SteeringType};
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_remind_steering_light_mode() {
        // Setup mocks
        let config_repo = MockConfigRepository::new();
        config_repo.set_steering_config(SteeringConfig {
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
        });

        let steering_repo = MockSteeringRepository::with_steering_files(vec![
            PathBuf::from(".kiro/steering/tech.md"),
            PathBuf::from(".kiro/steering/product.md"),
        ]);

        // Execute
        let reminders = remind_steering(&steering_repo, &config_repo).await.unwrap();

        // Verify all existing types are returned
        assert_eq!(reminders.len(), 2);
        assert_eq!(reminders[0].steering_name, "tech");
        assert_eq!(reminders[0].confidence, 1.0);
        assert_eq!(reminders[1].steering_name, "product");
        assert_eq!(reminders[1].confidence, 1.0);
    }

    #[tokio::test]
    async fn test_remind_steering_always_light_mode() {
        // Setup mocks
        let config_repo = MockConfigRepository::new();
        config_repo.set_steering_config(SteeringConfig {
            types: vec![SteeringType {
                name: "tech".to_string(),
                purpose: "Technology stack".to_string(),
                criteria: vec![],
                allowed_operations: vec![],
            }],
            backup: Default::default(),
        });

        let steering_repo = MockSteeringRepository::with_steering_files(vec![PathBuf::from(
            ".kiro/steering/tech.md",
        )]);

        // Execute
        let reminders = remind_steering(&steering_repo, &config_repo).await.unwrap();

        // Should return light mode results
        assert_eq!(reminders.len(), 1);
        assert_eq!(reminders[0].steering_name, "tech");
        assert_eq!(reminders[0].confidence, 1.0);
    }

    #[tokio::test]
    async fn test_remind_steering_filters_non_existent_files() {
        // Setup mocks
        let config_repo = MockConfigRepository::new();
        config_repo.set_steering_config(SteeringConfig {
            types: vec![
                SteeringType {
                    name: "tech".to_string(),
                    purpose: "Technology stack".to_string(),
                    criteria: vec![],
                    allowed_operations: vec![],
                },
                SteeringType {
                    name: "missing".to_string(),
                    purpose: "Missing file".to_string(),
                    criteria: vec![],
                    allowed_operations: vec![],
                },
            ],
            backup: Default::default(),
        });

        // Only tech.md exists, missing.md does not
        let steering_repo = MockSteeringRepository::with_steering_files(vec![PathBuf::from(
            ".kiro/steering/tech.md",
        )]);

        // Execute
        let reminders = remind_steering(&steering_repo, &config_repo).await.unwrap();

        // Should only return tech, not missing
        assert_eq!(reminders.len(), 1);
        assert_eq!(reminders[0].steering_name, "tech");
    }

    #[tokio::test]
    async fn test_remind_steering_with_empty_config() {
        let config_repo = MockConfigRepository::new();
        config_repo.set_steering_config(SteeringConfig {
            types: vec![],
            backup: Default::default(),
        });

        let steering_repo = MockSteeringRepository::new();

        // Execute with empty config
        let reminders = remind_steering(&steering_repo, &config_repo).await.unwrap();

        // Should return empty list
        assert_eq!(reminders.len(), 0);
    }
}
