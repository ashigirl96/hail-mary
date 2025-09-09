use crate::application::use_cases::backup_steering;
use crate::cli::formatters::{format_error, format_success};
use crate::infrastructure::filesystem::path_manager::PathManager;
use crate::infrastructure::repositories::{config::ConfigRepository, steering::SteeringRepository};
use anyhow::Result;

pub struct SteeringBackupCommand;

impl Default for SteeringBackupCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl SteeringBackupCommand {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self) -> Result<()> {
        // Use current directory as project root
        let current_dir = std::env::current_dir()?;
        let path_manager = PathManager::new(current_dir);

        // Create repositories
        let config_repo = ConfigRepository::new(path_manager.clone());
        let steering_repo = SteeringRepository::new(path_manager);

        // Execute backup use case
        match backup_steering(&config_repo, &steering_repo) {
            Ok(message) => {
                println!("{}", format_success(&message));
                Ok(())
            }
            Err(e) => {
                println!("{}", format_error(&e.to_string()));
                Err(anyhow::anyhow!(e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_helpers::TestDirectory;
    use std::fs;

    #[test]
    fn test_steering_backup_command_new() {
        let command = SteeringBackupCommand::new();
        assert!(matches!(command, SteeringBackupCommand));
    }

    #[test]
    fn test_steering_backup_command_default() {
        let command = SteeringBackupCommand;
        assert!(matches!(command, SteeringBackupCommand));
    }

    #[test]
    fn test_steering_backup_command_execute_with_project() {
        let _test_dir = TestDirectory::new();

        // Initialize a mock project with steering directory
        let kiro_dir = _test_dir.path().join(".kiro");
        let steering_dir = kiro_dir.join("steering");
        fs::create_dir_all(&steering_dir).unwrap();

        // Create some steering files
        fs::write(steering_dir.join("product.md"), "# Product").unwrap();
        fs::write(steering_dir.join("tech.md"), "# Tech").unwrap();

        // Create config.toml with steering configuration
        let config_content = r#"
[[steering.types]]
name = "product"
purpose = "Product overview"
criteria = ["Overview: Description"]

[steering.backup]
max = 10
"#;
        fs::write(kiro_dir.join("config.toml"), config_content).unwrap();

        let command = SteeringBackupCommand::new();
        let result = command.execute();

        // Should succeed
        assert!(result.is_ok());

        // Verify backup was created
        let backup_dir = steering_dir.join("backup");
        assert!(backup_dir.exists());

        // Should contain at least one backup directory
        let entries: Vec<_> = fs::read_dir(&backup_dir)
            .unwrap()
            .map(|e| e.unwrap())
            .collect();
        assert!(!entries.is_empty());
    }

    #[test]
    fn test_steering_backup_command_execute_without_project() {
        let _test_dir = TestDirectory::new();

        // No .kiro directory exists
        let command = SteeringBackupCommand::new();
        let result = command.execute();

        // Should succeed but report no files (graceful handling)
        assert!(result.is_ok());
    }

    #[test]
    fn test_steering_backup_command_execute_with_no_steering_files() {
        let _test_dir = TestDirectory::new();

        // Create .kiro directory but no steering files
        let kiro_dir = _test_dir.path().join(".kiro");
        let steering_dir = kiro_dir.join("steering");
        fs::create_dir_all(&steering_dir).unwrap();

        // Create config.toml
        let config_content = r#"
[steering.backup]
max = 10
"#;
        fs::write(kiro_dir.join("config.toml"), config_content).unwrap();

        let command = SteeringBackupCommand::new();
        let result = command.execute();

        // Should succeed but report no files
        assert!(result.is_ok());
    }
}
