use crate::application::errors::ApplicationError;
use crate::application::repositories::ProjectRepository;
use crate::domain::entities::steering::SteeringConfig;

pub fn initialize_project(repository: &impl ProjectRepository) -> Result<(), ApplicationError> {
    // Initialize project structure (idempotent)
    repository.initialize()?;

    // Initialize steering directories (idempotent)
    repository.initialize_steering()?;

    // Ensure steering configuration exists (idempotent)
    repository.ensure_steering_config()?;

    // Ensure steering backup configuration exists (idempotent)
    repository.ensure_steering_backup_config()?;

    // Create steering files (idempotent)
    let steering_config = SteeringConfig::default_for_new_project();
    repository.create_steering_files(&steering_config)?;

    // Update .gitignore (idempotent)
    repository.update_gitignore()?;

    // Deploy slash command markdown files (always overwrites)
    repository.deploy_slash_commands()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_helpers::MockProjectRepository;

    #[test]
    fn test_initialize_project_success() {
        let repo = MockProjectRepository::new();

        let result = initialize_project(&repo);
        assert!(result.is_ok());
    }

    #[test]
    fn test_initialize_project_idempotent() {
        let mut repo = MockProjectRepository::new();
        repo.set_initialized(true); // Project already exists

        // Should succeed even if project already exists (idempotent)
        let result = initialize_project(&repo);
        assert!(result.is_ok());
    }

    #[test]
    fn test_initialize_project_initialize_failure() {
        let mut repo = MockProjectRepository::new();
        repo.set_operation_to_fail("initialize");

        let result = initialize_project(&repo);
        assert!(result.is_err());
        match result.unwrap_err() {
            ApplicationError::ProjectInitializationError(_) => {}
            other => panic!("Expected ProjectInitializationError, got {:?}", other),
        }
    }

    #[test]
    fn test_initialize_project_config_failure() {
        let mut repo = MockProjectRepository::new();
        repo.set_operation_to_fail("ensure_steering_config");

        let result = initialize_project(&repo);
        assert!(result.is_err());
        match result.unwrap_err() {
            ApplicationError::ConfigurationError(_) => {}
            other => panic!("Expected ConfigurationError, got {:?}", other),
        }
    }

    #[test]
    fn test_initialize_project_gitignore_failure() {
        let mut repo = MockProjectRepository::new();
        repo.set_operation_to_fail("update_gitignore");

        let result = initialize_project(&repo);
        assert!(result.is_err());
        match result.unwrap_err() {
            ApplicationError::FileSystemError(_) => {}
            other => panic!("Expected FileSystemError, got {:?}", other),
        }
    }

    #[test]
    fn test_initialize_project_flow_order() {
        // Test that operations are called in the correct order
        let repo = MockProjectRepository::new();
        let result = initialize_project(&repo);
        assert!(result.is_ok());
    }

    #[test]
    fn test_initialize_project_with_default_config() {
        let repo = MockProjectRepository::new();
        let result = initialize_project(&repo);
        assert!(result.is_ok());
    }

    #[test]
    fn test_initialize_project_error_propagation() {
        // Test that all possible errors from dependencies are properly propagated
        let mut repo = MockProjectRepository::new();

        // Test each operation failure
        let operations = vec![
            ("initialize", "ProjectInitializationError"),
            ("ensure_steering_config", "ConfigurationError"),
            ("update_gitignore", "FileSystemError"),
            ("deploy_slash_commands", "FileSystemError"),
        ];

        for (operation, _expected_error_type) in operations {
            repo.set_operation_to_fail(operation);

            let result = initialize_project(&repo);
            assert!(result.is_err(), "Operation {} should fail", operation);

            let error = result.unwrap_err();
            match operation {
                "update_gitignore" | "deploy_slash_commands" => {
                    assert!(
                        matches!(error, ApplicationError::FileSystemError(_)),
                        "Expected FileSystemError for {}, got {:?}",
                        operation,
                        error
                    );
                }
                "initialize" => {
                    assert!(
                        matches!(error, ApplicationError::ProjectInitializationError(_)),
                        "Expected ProjectInitializationError for {}, got {:?}",
                        operation,
                        error
                    );
                }
                "ensure_steering_config" => {
                    assert!(
                        matches!(error, ApplicationError::ConfigurationError(_)),
                        "Expected ConfigurationError for {}, got {:?}",
                        operation,
                        error
                    );
                }
                _ => panic!("Unexpected operation: {}", operation),
            }

            repo.clear_failure();
        }
    }
}
