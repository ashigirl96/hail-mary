use crate::application::errors::ApplicationError;
use crate::domain::entities::memory::Memory;
use crate::domain::entities::project::ProjectConfig;

pub trait ProjectRepository: Send + Sync {
    fn initialize(&self) -> Result<(), ApplicationError>;
    fn exists(&self) -> Result<bool, ApplicationError>;
    fn save_config(&self, config: &ProjectConfig) -> Result<(), ApplicationError>;
    fn load_config(&self) -> Result<ProjectConfig, ApplicationError>;
    fn update_gitignore(&self) -> Result<(), ApplicationError>;
    fn create_feature(&self, name: &str) -> Result<(), ApplicationError>;
    fn save_document(&self, memory_type: &str, memories: &[Memory])
    -> Result<(), ApplicationError>;

    /// List all specification directories in .kiro/specs
    /// Returns a vector of (name, is_archived) tuples
    fn list_spec_directories(&self) -> Result<Vec<(String, bool)>, ApplicationError>;

    /// Mark a specification as complete by moving it to archive
    fn mark_spec_complete(&self, name: &str) -> Result<(), ApplicationError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_helpers::MockProjectRepository;

    #[test]
    fn test_project_repository_initialize() {
        let repo = MockProjectRepository::new();
        let result = repo.initialize();
        assert!(result.is_ok());
    }

    #[test]
    fn test_project_repository_initialize_failure() {
        let mut repo = MockProjectRepository::new();
        repo.set_next_operation_to_fail();

        let result = repo.initialize();
        assert!(result.is_err());
        match result.unwrap_err() {
            ApplicationError::ProjectInitializationError(_) => {}
            _ => panic!("Expected ProjectInitializationError"),
        }
    }

    #[test]
    fn test_project_repository_exists() {
        let mut repo = MockProjectRepository::new();

        // Initially not initialized
        let exists = repo.exists().unwrap();
        assert!(!exists);

        // After setting as initialized
        repo.set_initialized(true);
        let exists = repo.exists().unwrap();
        assert!(exists);
    }

    #[test]
    fn test_project_repository_save_config() {
        let repo = MockProjectRepository::new();
        let config = ProjectConfig::default_for_new_project();

        let result = repo.save_config(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_project_repository_save_config_failure() {
        let mut repo = MockProjectRepository::new();
        repo.set_next_operation_to_fail();
        let config = ProjectConfig::default_for_new_project();

        let result = repo.save_config(&config);
        assert!(result.is_err());
        match result.unwrap_err() {
            ApplicationError::ConfigurationError(_) => {}
            _ => panic!("Expected ConfigurationError"),
        }
    }

    #[test]
    fn test_project_repository_load_config() {
        let mut repo = MockProjectRepository::new();

        // Load default config when none exists
        let config = repo.load_config().unwrap();
        assert_eq!(config.memory_types.len(), 5); // Default types
        assert!(config.memory_types.contains(&"tech".to_string()));
        assert!(config.memory_types.contains(&"project-tech".to_string()));
        assert!(config.memory_types.contains(&"domain".to_string()));

        // Load custom config
        let custom_config = ProjectConfig {
            memory_types: vec!["custom".to_string()],
            instructions: "Custom instructions".to_string(),
            document_format: crate::domain::entities::project::DocumentFormat::Markdown,
        };
        repo.set_config(custom_config.clone());

        let loaded_config = repo.load_config().unwrap();
        assert_eq!(loaded_config.memory_types, vec!["custom".to_string()]);
        assert_eq!(loaded_config.instructions, "Custom instructions");
    }

    #[test]
    fn test_project_repository_update_gitignore() {
        let repo = MockProjectRepository::new();

        let result = repo.update_gitignore();
        assert!(result.is_ok());
    }

    #[test]
    fn test_project_repository_update_gitignore_failure() {
        let mut repo = MockProjectRepository::new();
        repo.set_next_operation_to_fail();

        let result = repo.update_gitignore();
        assert!(result.is_err());
        match result.unwrap_err() {
            ApplicationError::FileSystemError(_) => {}
            _ => panic!("Expected FileSystemError"),
        }
    }

    #[test]
    fn test_project_repository_create_feature_valid_names() {
        let repo = MockProjectRepository::new();

        // Valid feature names
        let valid_names = vec![
            "user-authentication",
            "api-endpoints",
            "database-migration",
            "feature-123",
            "simple",
        ];

        for name in valid_names {
            let result = repo.create_feature(name);
            assert!(result.is_ok(), "Feature name '{}' should be valid", name);
        }
    }

    #[test]
    fn test_project_repository_create_feature_invalid_names() {
        let repo = MockProjectRepository::new();

        // Invalid feature names
        let invalid_names = vec![
            "-invalid-start",     // starts with dash
            "invalid-end-",       // ends with dash
            "invalid--double",    // double dash
            "InvalidCase",        // uppercase
            "invalid_underscore", // underscore
            "invalid.dot",        // dot
        ];

        for name in invalid_names {
            let result = repo.create_feature(name);
            assert!(result.is_err(), "Feature name '{}' should be invalid", name);
            match result.unwrap_err() {
                ApplicationError::InvalidFeatureName(_) => {}
                _ => panic!("Expected InvalidFeatureName for '{}'", name),
            }
        }
    }

    #[test]
    fn test_project_repository_create_feature_failure() {
        let mut repo = MockProjectRepository::new();
        repo.set_next_operation_to_fail();

        let result = repo.create_feature("valid-name");
        assert!(result.is_err());
        match result.unwrap_err() {
            ApplicationError::FeatureCreationError(_) => {}
            _ => panic!("Expected FeatureCreationError"),
        }
    }

    #[test]
    fn test_project_repository_save_document() {
        let repo = MockProjectRepository::new();
        let memories = vec![crate::domain::entities::memory::Memory::new(
            "tech".to_string(),
            "Test Memory".to_string(),
            "Test content".to_string(),
        )];

        let result = repo.save_document("tech", &memories);
        assert!(result.is_ok());
    }

    #[test]
    fn test_project_repository_save_document_failure() {
        let mut repo = MockProjectRepository::new();
        repo.set_next_operation_to_fail();
        let memories = vec![];

        let result = repo.save_document("tech", &memories);
        assert!(result.is_err());
        match result.unwrap_err() {
            ApplicationError::DocumentGenerationError(_) => {}
            _ => panic!("Expected DocumentGenerationError"),
        }
    }

    #[test]
    fn test_project_config_validate_memory_type() {
        let config = ProjectConfig::default_for_new_project();

        // Valid types
        assert!(config.validate_memory_type("tech"));
        assert!(config.validate_memory_type("project-tech"));
        assert!(config.validate_memory_type("domain"));
        assert!(config.validate_memory_type("workflow"));
        assert!(config.validate_memory_type("decision"));

        // Invalid types
        assert!(!config.validate_memory_type("invalid"));
        assert!(!config.validate_memory_type(""));
        assert!(!config.validate_memory_type("TECH"));
    }

    #[test]
    fn test_mock_repository_helper_methods() {
        let mut repo = MockProjectRepository::new();

        // Test feature tracking
        repo.add_created_feature("feature1");
        repo.add_created_feature("feature2");
        assert_eq!(repo.get_created_features().len(), 2);
        assert_eq!(repo.get_created_features()[0], "feature1");

        // Test document tracking
        repo.add_saved_document("tech", 5);
        repo.add_saved_document("domain", 3);
        assert_eq!(repo.get_saved_documents().len(), 2);
        assert_eq!(*repo.get_saved_documents().get("tech").unwrap(), 5);
        assert_eq!(*repo.get_saved_documents().get("domain").unwrap(), 3);

        // Test failure flag
        repo.set_next_operation_to_fail();
        // Can't access private field directly in the shared mock
        // The behavior is tested implicitly through the API
        repo.reset_failure_flag();
        // Ensure the flag was reset by testing an operation that would fail
        let result = repo.initialize();
        assert!(result.is_ok());
    }
}
