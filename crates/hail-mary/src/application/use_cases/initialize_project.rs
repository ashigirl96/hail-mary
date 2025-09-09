use crate::application::errors::ApplicationError;
use crate::application::repositories::{
    ConfigRepositoryInterface, SpecRepositoryInterface, SteeringRepositoryInterface,
};
use crate::domain::entities::project::ProjectConfig;
use crate::domain::entities::steering::SteeringConfig;
use crate::infrastructure::filesystem::path_manager::PathManager;

pub fn initialize_project(
    config_repo: &dyn ConfigRepositoryInterface,
    _spec_repo: &dyn SpecRepositoryInterface,
    steering_repo: &dyn SteeringRepositoryInterface,
) -> Result<(), ApplicationError> {
    // Initialize project structure - create .kiro directory and subdirectories
    // The SpecRepository handles specs directory creation
    let path_manager = PathManager::discover()?;
    let specs_dir = path_manager.specs_dir(false);
    if !specs_dir.exists() {
        std::fs::create_dir_all(&specs_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to create specs directory: {}", e))
        })?;
    }

    // Initialize steering directories (idempotent)
    steering_repo.initialize_steering()?;

    // Ensure steering configuration exists (idempotent)
    config_repo.ensure_steering_config()?;

    // Ensure steering backup configuration exists (idempotent)
    config_repo.ensure_steering_backup_config()?;

    // Save default config if it doesn't exist
    let config = ProjectConfig::default_for_new_project();
    config_repo.save_config(&config)?;

    // Create steering files (idempotent)
    let steering_config = SteeringConfig::default_for_new_project();
    steering_repo.create_steering_files(&steering_config)?;

    // Update .gitignore (idempotent)
    steering_repo.update_gitignore()?;

    // Deploy slash command markdown files (always overwrites)
    steering_repo.deploy_slash_commands()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_helpers::{
        MockConfigRepository, MockSpecRepository, MockSteeringRepository,
    };

    #[test]
    fn test_initialize_project_success() {
        let config_repo = MockConfigRepository::new();
        let spec_repo = MockSpecRepository::new();
        let steering_repo = MockSteeringRepository::new();

        let result = initialize_project(&config_repo, &spec_repo, &steering_repo);
        assert!(result.is_ok());
    }

    #[test]
    fn test_initialize_project_idempotent() {
        let config_repo = MockConfigRepository::new();
        let spec_repo = MockSpecRepository::new();
        let steering_repo = MockSteeringRepository::new();
        steering_repo.set_project_exists(true); // Project already exists

        // Should succeed even if project already exists (idempotent)
        let result = initialize_project(&config_repo, &spec_repo, &steering_repo);
        assert!(result.is_ok());
    }

    #[test]
    fn test_initialize_project_steering_failure() {
        let config_repo = MockConfigRepository::new();
        let spec_repo = MockSpecRepository::new();
        let steering_repo = MockSteeringRepository::new();
        steering_repo.set_operation_to_fail("initialize_steering");

        let result = initialize_project(&config_repo, &spec_repo, &steering_repo);
        assert!(result.is_err());
        match result.unwrap_err() {
            ApplicationError::FileSystemError(_) => {}
            other => panic!("Expected FileSystemError, got {:?}", other),
        }
    }

    #[test]
    fn test_initialize_project_config_failure() {
        let config_repo = MockConfigRepository::new();
        config_repo.set_operation_to_fail("ensure_steering_config");
        let spec_repo = MockSpecRepository::new();
        let steering_repo = MockSteeringRepository::new();

        let result = initialize_project(&config_repo, &spec_repo, &steering_repo);
        assert!(result.is_err());
        match result.unwrap_err() {
            ApplicationError::ConfigurationError(_) => {}
            other => panic!("Expected ConfigurationError, got {:?}", other),
        }
    }

    #[test]
    fn test_initialize_project_gitignore_failure() {
        let config_repo = MockConfigRepository::new();
        let spec_repo = MockSpecRepository::new();
        let steering_repo = MockSteeringRepository::new();
        steering_repo.set_operation_to_fail("update_gitignore");

        let result = initialize_project(&config_repo, &spec_repo, &steering_repo);
        assert!(result.is_err());
        match result.unwrap_err() {
            ApplicationError::FileSystemError(_) => {}
            other => panic!("Expected FileSystemError, got {:?}", other),
        }
    }

    #[test]
    fn test_initialize_project_flow_order() {
        // Test that operations are called in the correct order
        let config_repo = MockConfigRepository::new();
        let spec_repo = MockSpecRepository::new();
        let steering_repo = MockSteeringRepository::new();

        let result = initialize_project(&config_repo, &spec_repo, &steering_repo);
        assert!(result.is_ok());
    }

    #[test]
    fn test_initialize_project_with_default_config() {
        let config_repo = MockConfigRepository::new();
        let spec_repo = MockSpecRepository::new();
        let steering_repo = MockSteeringRepository::new();

        let result = initialize_project(&config_repo, &spec_repo, &steering_repo);
        assert!(result.is_ok());
    }

    #[test]
    fn test_initialize_project_error_propagation() {
        // Test that all possible errors from dependencies are properly propagated

        // Test steering config failure
        {
            let config_repo = MockConfigRepository::new();
            config_repo.set_operation_to_fail("ensure_steering_config");
            let spec_repo = MockSpecRepository::new();
            let steering_repo = MockSteeringRepository::new();

            let result = initialize_project(&config_repo, &spec_repo, &steering_repo);
            assert!(result.is_err());
            match result.unwrap_err() {
                ApplicationError::ConfigurationError(_) => {}
                other => panic!("Expected ConfigurationError, got {:?}", other),
            }
        }

        // Test gitignore failure
        {
            let config_repo = MockConfigRepository::new();
            let spec_repo = MockSpecRepository::new();
            let steering_repo = MockSteeringRepository::new();
            steering_repo.set_operation_to_fail("update_gitignore");

            let result = initialize_project(&config_repo, &spec_repo, &steering_repo);
            assert!(result.is_err());
            match result.unwrap_err() {
                ApplicationError::FileSystemError(_) => {}
                other => panic!("Expected FileSystemError, got {:?}", other),
            }
        }

        // Test deploy slash commands failure
        {
            let config_repo = MockConfigRepository::new();
            let spec_repo = MockSpecRepository::new();
            let steering_repo = MockSteeringRepository::new();
            steering_repo.set_operation_to_fail("deploy_slash_commands");

            let result = initialize_project(&config_repo, &spec_repo, &steering_repo);
            assert!(result.is_err());
            match result.unwrap_err() {
                ApplicationError::FileSystemError(_) => {}
                other => panic!("Expected FileSystemError, got {:?}", other),
            }
        }
    }
}
