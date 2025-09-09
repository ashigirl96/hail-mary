use crate::application::errors::ApplicationError;
use crate::application::repositories::ConfigRepository;
use crate::domain::entities::project::ProjectConfig;
use crate::domain::entities::steering::{SteeringBackupConfig, SteeringConfig};
use crate::infrastructure::filesystem::path_manager::PathManager;
use std::fs;

pub struct ConfigRepositoryImpl {
    path_manager: PathManager,
}

impl ConfigRepositoryImpl {
    pub fn new(path_manager: PathManager) -> Self {
        Self { path_manager }
    }
}

impl ConfigRepository for ConfigRepositoryImpl {
    fn load_config(&self) -> Result<ProjectConfig, ApplicationError> {
        let config_path = self.path_manager.config_path(true);

        if !config_path.exists() {
            // Return default config if file doesn't exist
            return Ok(ProjectConfig::default_for_new_project());
        }

        let content = fs::read_to_string(&config_path).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to read config file: {}", e))
        })?;

        // Parse TOML directly into ProjectConfig
        toml::from_str(&content).map_err(|e| {
            ApplicationError::ConfigurationError(format!("Failed to parse config: {}", e))
        })
    }

    fn save_config(&self, config: &ProjectConfig) -> Result<(), ApplicationError> {
        let config_path = self.path_manager.config_path(true);

        // Never overwrite existing config.toml (even with --force)
        if config_path.exists() {
            return Ok(());
        }

        // Serialize ProjectConfig to TOML
        let content = toml::to_string_pretty(config).map_err(|e| {
            ApplicationError::ConfigurationError(format!("Failed to serialize config: {}", e))
        })?;

        fs::write(config_path, content).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to write config file: {}", e))
        })?;

        Ok(())
    }

    fn load_steering_config(&self) -> Result<SteeringConfig, ApplicationError> {
        let config = self.load_config()?;
        Ok(config.steering)
    }

    fn load_steering_backup_config(&self) -> Result<SteeringBackupConfig, ApplicationError> {
        let config = self.load_config()?;
        Ok(config.steering.backup)
    }

    fn ensure_steering_config(&self) -> Result<(), ApplicationError> {
        let config_path = self.path_manager.config_path(true);

        if !config_path.exists() {
            // Create new config with steering section
            let config = ProjectConfig::default_for_new_project();
            return self.save_full_config(&config);
        }

        // Load existing config
        let mut config = self.load_config()?;

        // Check if steering types are empty (indicating missing steering section)
        if config.steering.types.is_empty() {
            config.steering = SteeringConfig::default_for_new_project();
            return self.save_full_config(&config);
        }

        Ok(())
    }

    fn ensure_steering_backup_config(&self) -> Result<(), ApplicationError> {
        let config_path = self.path_manager.config_path(true);

        if !config_path.exists() {
            // Config doesn't exist, will be created by save_config with defaults
            return Ok(());
        }

        // For now, the backup config is always included through serde defaults
        // so we don't need to do anything special here
        Ok(())
    }
}

impl ConfigRepositoryImpl {
    /// Save the full configuration (used internally for updates)
    fn save_full_config(&self, config: &ProjectConfig) -> Result<(), ApplicationError> {
        let config_path = self.path_manager.config_path(true);

        let content = toml::to_string_pretty(config).map_err(|e| {
            ApplicationError::ConfigurationError(format!("Failed to serialize config: {}", e))
        })?;

        fs::write(config_path, content).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to write config file: {}", e))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_helpers::TestDirectory;

    fn create_test_repository() -> (ConfigRepositoryImpl, TestDirectory) {
        let test_dir = TestDirectory::new_no_cd();
        let path_manager = PathManager::new(test_dir.path().to_path_buf());
        let repository = ConfigRepositoryImpl::new(path_manager);
        (repository, test_dir)
    }

    #[test]
    fn test_load_config_returns_default_when_missing() {
        let (repository, _test_dir) = create_test_repository();

        let config = repository.load_config().unwrap();

        // Should return default config
        assert!(!config.instructions.is_empty());
        assert!(!config.steering.types.is_empty());
    }

    #[test]
    fn test_save_and_load_config_roundtrip() {
        let (repository, test_dir) = create_test_repository();

        // Create .kiro directory
        fs::create_dir_all(test_dir.path().join(".kiro")).unwrap();

        // Save config
        let config = ProjectConfig::default_for_new_project();
        repository.save_config(&config).unwrap();

        // Load config
        let loaded_config = repository.load_config().unwrap();

        // Verify roundtrip
        assert_eq!(loaded_config.instructions, config.instructions);
        assert_eq!(
            loaded_config.steering.types.len(),
            config.steering.types.len()
        );
    }

    #[test]
    fn test_save_config_does_not_overwrite_existing() {
        let (repository, test_dir) = create_test_repository();

        // Create .kiro directory and config file
        let kiro_dir = test_dir.path().join(".kiro");
        fs::create_dir_all(&kiro_dir).unwrap();

        let config_path = kiro_dir.join("config.toml");
        fs::write(&config_path, "# Custom config\ninstructions = \"Custom\"").unwrap();

        // Try to save config
        let config = ProjectConfig::default_for_new_project();
        repository.save_config(&config).unwrap();

        // Verify original content is preserved
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("Custom"));
    }

    #[test]
    fn test_load_steering_config() {
        let (repository, _test_dir) = create_test_repository();

        let steering_config = repository.load_steering_config().unwrap();

        // Should have default steering types
        assert_eq!(steering_config.types.len(), 3);
        assert_eq!(steering_config.types[0].name, "product");
    }

    #[test]
    fn test_load_steering_backup_config() {
        let (repository, _test_dir) = create_test_repository();

        let backup_config = repository.load_steering_backup_config().unwrap();

        // Should have default max value
        assert_eq!(backup_config.max, 10);
    }

    #[test]
    fn test_ensure_steering_config_adds_missing_section() {
        let (repository, test_dir) = create_test_repository();

        // Create .kiro directory and config without steering
        let kiro_dir = test_dir.path().join(".kiro");
        fs::create_dir_all(&kiro_dir).unwrap();

        let config_path = kiro_dir.join("config.toml");
        fs::write(&config_path, "instructions = \"Test\"").unwrap();

        // Ensure steering config
        repository.ensure_steering_config().unwrap();

        // Load and verify
        let config = repository.load_config().unwrap();
        assert!(!config.steering.types.is_empty());
    }
}
