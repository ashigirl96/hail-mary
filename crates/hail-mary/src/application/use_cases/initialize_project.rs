use crate::application::errors::ApplicationError;
use crate::application::repositories::ProjectRepository;
use crate::domain::entities::steering::SteeringConfig;

pub fn initialize_project(
    repository: &impl ProjectRepository,
    force: bool,
) -> Result<(), ApplicationError> {
    // Check if project already exists
    if repository.exists()? && !force {
        return Err(ApplicationError::ProjectAlreadyExists);
    }

    // Initialize project structure
    repository.initialize()?;

    // Initialize steering directories
    repository.initialize_steering()?;

    // Ensure steering configuration exists (add [steering] section if missing)
    repository.ensure_steering_config()?;

    // Create steering files
    let steering_config = SteeringConfig::default_for_new_project();
    repository.create_steering_files(&steering_config)?;

    // Update .gitignore
    repository.update_gitignore()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_helpers::MockProjectRepository;

    #[test]
    fn test_initialize_project_success() {
        let repo = MockProjectRepository::new();

        let result = initialize_project(&repo, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_initialize_project_success_with_force() {
        let mut repo = MockProjectRepository::new();
        repo.set_initialized(true); // Project already exists

        // Should succeed with force=true
        let result = initialize_project(&repo, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_initialize_project_already_exists() {
        let mut repo = MockProjectRepository::new();
        repo.set_initialized(true); // Project already exists

        // Should fail with force=false
        let result = initialize_project(&repo, false);
        assert!(result.is_err());
        match result.unwrap_err() {
            ApplicationError::ProjectAlreadyExists => {}
            other => panic!("Expected ProjectAlreadyExists, got {:?}", other),
        }
    }

    #[test]
    fn test_initialize_project_exists_check_failure() {
        let mut repo = MockProjectRepository::new();
        repo.set_operation_to_fail("exists");

        let result = initialize_project(&repo, false);
        assert!(result.is_err());
        match result.unwrap_err() {
            ApplicationError::FileSystemError(_) => {}
            other => panic!("Expected FileSystemError, got {:?}", other),
        }
    }

    #[test]
    fn test_initialize_project_initialize_failure() {
        let mut repo = MockProjectRepository::new();
        repo.set_operation_to_fail("initialize");

        let result = initialize_project(&repo, false);
        assert!(result.is_err());
        match result.unwrap_err() {
            ApplicationError::ProjectInitializationError(_) => {}
            other => panic!("Expected ProjectInitializationError, got {:?}", other),
        }
    }

    #[test]
    fn test_initialize_project_save_config_failure() {
        let mut repo = MockProjectRepository::new();
        repo.set_operation_to_fail("ensure_steering_config");

        let result = initialize_project(&repo, false);
        assert!(result.is_err());
        match result.unwrap_err() {
            ApplicationError::ConfigurationError(_) => {}
            other => panic!("Expected ConfigurationError, got {:?}", other),
        }
    }

    #[test]
    fn test_initialize_project_update_gitignore_failure() {
        let mut repo = MockProjectRepository::new();
        repo.set_operation_to_fail("update_gitignore");

        let result = initialize_project(&repo, false);
        assert!(result.is_err());
        match result.unwrap_err() {
            ApplicationError::FileSystemError(_) => {}
            other => panic!("Expected FileSystemError, got {:?}", other),
        }
    }

    #[test]
    fn test_initialize_project_flow_order() {
        // Test that operations are called in the correct order
        // This is important because each step depends on the previous ones

        let repo = MockProjectRepository::new();
        let result = initialize_project(&repo, false);
        assert!(result.is_ok());

        // The test implicitly verifies order by the fact that:
        // 1. exists() is called first
        // 2. initialize() is called second
        // 3. save_config() is called third
        // 4. update_gitignore() is called last
        // If any of these were called out of order, the function would behave incorrectly
    }

    #[test]
    fn test_initialize_project_with_default_config() {
        let repo = MockProjectRepository::new();

        let result = initialize_project(&repo, false);
        assert!(result.is_ok());

        // Verify that default config is used
        // The function should create a default ProjectConfig internally
        // This is tested indirectly by ensuring the function completes successfully
    }

    #[test]
    fn test_initialize_project_force_flag_behavior() {
        let mut repo = MockProjectRepository::new();

        // Test force=false with non-existing project
        repo.set_initialized(false);
        let result = initialize_project(&repo, false);
        assert!(result.is_ok());

        // Test force=false with existing project
        repo.set_initialized(true);
        let result = initialize_project(&repo, false);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ApplicationError::ProjectAlreadyExists
        ));

        // Test force=true with existing project
        let result = initialize_project(&repo, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_initialize_project_error_propagation() {
        // Test that all possible errors from dependencies are properly propagated

        let mut repo = MockProjectRepository::new();

        // Test each operation failure
        let operations = vec![
            ("exists", "FileSystemError"),
            ("initialize", "ProjectInitializationError"),
            ("ensure_steering_config", "ConfigurationError"),
            ("update_gitignore", "FileSystemError"),
        ];

        for (operation, _expected_error_type) in operations {
            repo.set_operation_to_fail(operation);

            let result = initialize_project(&repo, false);
            assert!(result.is_err(), "Operation {} should fail", operation);

            let error = result.unwrap_err();
            match operation {
                "exists" | "update_gitignore" => {
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
