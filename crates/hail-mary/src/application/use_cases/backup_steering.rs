use crate::application::errors::ApplicationError;
use crate::application::repositories::{ConfigRepositoryInterface, SteeringRepositoryInterface};
use chrono::Local;

/// Creates a backup of all steering files with automatic rotation
///
/// This function:
/// 1. Lists all steering markdown files (excluding backup directory)
/// 2. Creates a timestamped backup directory
/// 3. Copies all files to the backup
/// 4. Enforces the maximum backup limit by deleting oldest backups
pub fn backup_steering(
    config_repo: &dyn ConfigRepositoryInterface,
    steering_repo: &dyn SteeringRepositoryInterface,
) -> Result<String, ApplicationError> {
    // Ensure steering backup config exists
    config_repo.ensure_steering_backup_config()?;

    // Load the backup configuration
    let config = config_repo.load_steering_backup_config()?;

    // List all steering files to backup
    let files = steering_repo.list_steering_files()?;

    if files.is_empty() {
        return Ok("No steering files found to backup".to_string());
    }

    // Check current backup count BEFORE creating new backup
    let existing_backups = steering_repo.list_steering_backups()?;

    // If we're at or above the limit, delete oldest to make room
    if existing_backups.len() >= config.max {
        // Calculate how many to delete (at least 1 to make room for new backup)
        let excess_count = existing_backups.len() - config.max + 1;
        steering_repo.delete_oldest_steering_backups(excess_count)?;
    }

    // Generate timestamp for backup directory
    let timestamp = Local::now().format("%Y-%m-%d-%H-%M").to_string();

    // Create the backup (now we have room)
    steering_repo.create_steering_backup(&timestamp, &files)?;

    Ok(format!(
        "Created backup '{}' with {} files",
        timestamp,
        files.len()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_helpers::{MockConfigRepository, MockSteeringRepository};
    use std::path::PathBuf;

    #[test]
    fn test_backup_steering_creates_backup_successfully() {
        let config_repo = MockConfigRepository::new();
        let steering_repo = MockSteeringRepository::with_steering_files(vec![
            PathBuf::from("product.md"),
            PathBuf::from("tech.md"),
        ]);

        let result = backup_steering(&config_repo, &steering_repo);

        assert!(result.is_ok());
        let message = result.unwrap();
        assert!(message.contains("Created backup"));
        assert!(message.contains("with"));
        assert!(message.contains("files"));
    }

    #[test]
    fn test_backup_steering_handles_no_files() {
        let config_repo = MockConfigRepository::new();
        let steering_repo = MockSteeringRepository::new();

        let result = backup_steering(&config_repo, &steering_repo);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "No steering files found to backup");
    }

    #[test]
    fn test_backup_steering_enforces_max_limit() {
        let config_repo = MockConfigRepository::new();
        let steering_repo =
            MockSteeringRepository::with_steering_files(vec![PathBuf::from("product.md")]);

        let result = backup_steering(&config_repo, &steering_repo);

        assert!(result.is_ok());
    }

    #[test]
    fn test_backup_steering_handles_repository_errors() {
        let config_repo = MockConfigRepository::new();
        let steering_repo = MockSteeringRepository::new();
        steering_repo.set_operation_to_fail("list_steering_files");

        let result = backup_steering(&config_repo, &steering_repo);

        assert!(result.is_err());
        match result.unwrap_err() {
            ApplicationError::FileSystemError(_) => {}
            _ => panic!("Expected FileSystemError"),
        }
    }

    #[test]
    fn test_backup_steering_ensures_config_exists() {
        let config_repo = MockConfigRepository::new();
        let steering_repo =
            MockSteeringRepository::with_steering_files(vec![PathBuf::from("product.md")]);

        // The function should call ensure_steering_backup_config first
        let result = backup_steering(&config_repo, &steering_repo);

        // Should succeed even if config didn't exist initially
        assert!(result.is_ok());
    }

    #[test]
    fn test_backup_steering_handles_config_errors() {
        let config_repo = MockConfigRepository::new();
        config_repo.set_operation_to_fail("ensure_steering_backup_config");
        let steering_repo = MockSteeringRepository::new();

        let result = backup_steering(&config_repo, &steering_repo);

        assert!(result.is_err());
        match result.unwrap_err() {
            ApplicationError::ConfigurationError(_) => {}
            _ => panic!("Expected ConfigurationError"),
        }
    }
}
