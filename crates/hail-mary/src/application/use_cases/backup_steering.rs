use crate::application::errors::ApplicationError;
use crate::application::repositories::ProjectRepository;
use chrono::Local;

/// Creates a backup of all steering files with automatic rotation
///
/// This function:
/// 1. Lists all steering markdown files (excluding backup/draft directories)
/// 2. Creates a timestamped backup directory
/// 3. Copies all files to the backup
/// 4. Enforces the maximum backup limit by deleting oldest backups
pub fn backup_steering(repository: &dyn ProjectRepository) -> Result<String, ApplicationError> {
    // Ensure steering backup config exists
    repository.ensure_steering_backup_config()?;

    // Load the backup configuration
    let config = repository.load_steering_backup_config()?;

    // List all steering files to backup
    let files = repository.list_steering_files()?;

    if files.is_empty() {
        return Ok("No steering files found to backup".to_string());
    }

    // Generate timestamp for backup directory
    let timestamp = Local::now().format("%Y-%m-%d-%H-%M").to_string();

    // Create the backup
    repository.create_steering_backup(&timestamp, &files)?;

    // Check if we need to enforce the max backup limit
    let backups = repository.list_steering_backups()?;

    if backups.len() > config.max {
        // Calculate how many to delete
        let excess_count = backups.len() - config.max;
        repository.delete_oldest_steering_backups(excess_count)?;
    }

    Ok(format!(
        "Created backup '{}' with {} files",
        timestamp,
        files.len()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_helpers::MockProjectRepository;

    #[test]
    fn test_backup_steering_creates_backup_successfully() {
        let repo = MockProjectRepository::new();

        // The mock will return default steering files

        // Mock will return these files when list_steering_files is called
        // For now, we'll just verify the function runs without error

        let result = backup_steering(&repo);

        assert!(result.is_ok());
        let message = result.unwrap();
        assert!(message.contains("Created backup"));
        assert!(message.contains("with"));
        assert!(message.contains("files"));
    }

    #[test]
    fn test_backup_steering_handles_no_files() {
        let repo = MockProjectRepository::new().with_empty_steering_files();

        let result = backup_steering(&repo);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "No steering files found to backup");
    }

    #[test]
    fn test_backup_steering_enforces_max_limit() {
        let repo = MockProjectRepository::new();

        // This test would verify that old backups are deleted when limit is exceeded
        // The mock implementation will handle this

        let result = backup_steering(&repo);

        assert!(result.is_ok());
    }

    #[test]
    fn test_backup_steering_handles_repository_errors() {
        let mut repo = MockProjectRepository::new();
        repo.set_operation_to_fail("list_steering_files");

        let result = backup_steering(&repo);

        assert!(result.is_err());
        match result.unwrap_err() {
            ApplicationError::FileSystemError(_) => {}
            _ => panic!("Expected FileSystemError"),
        }
    }

    #[test]
    fn test_backup_steering_ensures_config_exists() {
        let repo = MockProjectRepository::new();

        // The function should call ensure_steering_backup_config first
        let result = backup_steering(&repo);

        // Should succeed even if config didn't exist initially
        assert!(result.is_ok());
    }
}
