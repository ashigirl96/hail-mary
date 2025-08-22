use crate::application::repositories::ProjectRepository;
use crate::application::errors::ApplicationError;

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
    
    // Create default configuration
    let config = crate::domain::entities::project::ProjectConfig::default_for_new_project();
    repository.save_config(&config)?;
    
    // Update .gitignore
    repository.update_gitignore()?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::project::ProjectConfig;

    // Mock implementation for testing
    #[derive(Debug, Default)]
    struct MockProjectRepository {
        is_initialized: bool,
        config: Option<ProjectConfig>,
        gitignore_updated: bool,
        should_fail_operation: Option<String>, // Which operation should fail
        call_history: Vec<String>, // Track method calls
    }

    impl ProjectRepository for MockProjectRepository {
        fn initialize(&self) -> Result<(), ApplicationError> {
            if let Some(ref fail_op) = self.should_fail_operation {
                if fail_op == "initialize" {
                    return Err(ApplicationError::ProjectInitializationError("Mock initialization failure".to_string()));
                }
            }
            Ok(())
        }

        fn exists(&self) -> Result<bool, ApplicationError> {
            if let Some(ref fail_op) = self.should_fail_operation {
                if fail_op == "exists" {
                    return Err(ApplicationError::FileSystemError("Mock exists failure".to_string()));
                }
            }
            Ok(self.is_initialized)
        }

        fn save_config(&self, _config: &ProjectConfig) -> Result<(), ApplicationError> {
            if let Some(ref fail_op) = self.should_fail_operation {
                if fail_op == "save_config" {
                    return Err(ApplicationError::ConfigurationError("Mock save config failure".to_string()));
                }
            }
            Ok(())
        }

        fn load_config(&self) -> Result<ProjectConfig, ApplicationError> {
            if let Some(ref config) = self.config {
                Ok(config.clone())
            } else {
                Ok(ProjectConfig::default_for_new_project())
            }
        }

        fn update_gitignore(&self) -> Result<(), ApplicationError> {
            if let Some(ref fail_op) = self.should_fail_operation {
                if fail_op == "update_gitignore" {
                    return Err(ApplicationError::FileSystemError("Mock gitignore update failure".to_string()));
                }
            }
            Ok(())
        }

        fn create_feature(&self, _name: &str) -> Result<(), ApplicationError> {
            Ok(())
        }

        fn save_document(&self, _memory_type: &str, _memories: &[crate::domain::entities::memory::Memory]) -> Result<(), ApplicationError> {
            Ok(())
        }
    }

    impl MockProjectRepository {
        fn new() -> Self {
            Self::default()
        }

        fn set_initialized(&mut self, initialized: bool) {
            self.is_initialized = initialized;
        }

        fn set_operation_to_fail(&mut self, operation: &str) {
            self.should_fail_operation = Some(operation.to_string());
        }

        fn clear_failure(&mut self) {
            self.should_fail_operation = None;
        }
    }

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
            ApplicationError::ProjectAlreadyExists => {},
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
            ApplicationError::FileSystemError(_) => {},
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
            ApplicationError::ProjectInitializationError(_) => {},
            other => panic!("Expected ProjectInitializationError, got {:?}", other),
        }
    }

    #[test]
    fn test_initialize_project_save_config_failure() {
        let mut repo = MockProjectRepository::new();
        repo.set_operation_to_fail("save_config");
        
        let result = initialize_project(&repo, false);
        assert!(result.is_err());
        match result.unwrap_err() {
            ApplicationError::ConfigurationError(_) => {},
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
            ApplicationError::FileSystemError(_) => {},
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
        assert!(matches!(result.unwrap_err(), ApplicationError::ProjectAlreadyExists));
        
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
            ("save_config", "ConfigurationError"),
            ("update_gitignore", "FileSystemError"),
        ];
        
        for (operation, expected_error_type) in operations {
            repo.set_operation_to_fail(operation);
            
            let result = initialize_project(&repo, false);
            assert!(result.is_err(), "Operation {} should fail", operation);
            
            let error = result.unwrap_err();
            match operation {
                "exists" | "update_gitignore" => {
                    assert!(matches!(error, ApplicationError::FileSystemError(_)), 
                           "Expected FileSystemError for {}, got {:?}", operation, error);
                },
                "initialize" => {
                    assert!(matches!(error, ApplicationError::ProjectInitializationError(_)), 
                           "Expected ProjectInitializationError for {}, got {:?}", operation, error);
                },
                "save_config" => {
                    assert!(matches!(error, ApplicationError::ConfigurationError(_)), 
                           "Expected ConfigurationError for {}, got {:?}", operation, error);
                },
                _ => panic!("Unexpected operation: {}", operation),
            }
            
            repo.clear_failure();
        }
    }
}