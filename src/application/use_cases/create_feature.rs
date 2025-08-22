use crate::application::repositories::ProjectRepository;
use crate::application::errors::ApplicationError;

pub fn create_feature(
    repository: &impl ProjectRepository,
    name: &str,
) -> Result<String, ApplicationError> {
    // Validate feature name (must be kebab-case)
    if name.is_empty()
        || !name.chars().all(|c| c.is_lowercase() || c == '-' || c.is_numeric())
        || name.starts_with('-')
        || name.ends_with('-')
        || name.contains("--") {
        return Err(ApplicationError::InvalidFeatureName(name.to_string()));
    }
    
    // Create feature through repository
    repository.create_feature(name)?;
    
    // Return feature path for user feedback
    let date = chrono::Utc::now().format("%Y-%m-%d");
    Ok(format!(".kiro/specs/{}-{}", date, name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::project::ProjectConfig;
    use crate::domain::entities::memory::Memory;

    // Mock implementation for testing
    #[derive(Debug, Default)]
    struct MockProjectRepository {
        should_fail_operation: Option<String>,
    }

    impl ProjectRepository for MockProjectRepository {
        fn initialize(&self) -> Result<(), ApplicationError> {
            Ok(())
        }

        fn exists(&self) -> Result<bool, ApplicationError> {
            Ok(false)
        }

        fn save_config(&self, _config: &ProjectConfig) -> Result<(), ApplicationError> {
            Ok(())
        }

        fn load_config(&self) -> Result<ProjectConfig, ApplicationError> {
            Ok(ProjectConfig::default_for_new_project())
        }

        fn update_gitignore(&self) -> Result<(), ApplicationError> {
            Ok(())
        }

        fn create_feature(&self, name: &str) -> Result<(), ApplicationError> {
            if let Some(ref fail_op) = self.should_fail_operation {
                if fail_op == "create_feature" {
                    return Err(ApplicationError::FeatureCreationError(format!("Mock creation failure for: {}", name)));
                }
            }
            Ok(())
        }

        fn save_document(&self, _memory_type: &str, _memories: &[Memory]) -> Result<(), ApplicationError> {
            Ok(())
        }
    }

    impl MockProjectRepository {
        fn new() -> Self {
            Self::default()
        }

        fn set_operation_to_fail(&mut self, operation: &str) {
            self.should_fail_operation = Some(operation.to_string());
        }
    }

    #[test]
    fn test_create_feature_success() {
        let repo = MockProjectRepository::new();
        
        let result = create_feature(&repo, "user-authentication");
        assert!(result.is_ok());
        
        let feature_path = result.unwrap();
        assert!(feature_path.starts_with(".kiro/specs/"));
        assert!(feature_path.ends_with("-user-authentication"));
        assert!(feature_path.contains("2025")); // Should contain current year
    }

    #[test]
    fn test_create_feature_valid_names() {
        let repo = MockProjectRepository::new();
        
        let valid_names = vec![
            "user-authentication",
            "api-endpoints", 
            "database-migration",
            "feature-123",
            "simple",
            "a",
            "test-feature-with-numbers-123",
        ];
        
        for name in valid_names {
            let result = create_feature(&repo, name);
            assert!(result.is_ok(), "Feature name '{}' should be valid", name);
            
            let feature_path = result.unwrap();
            assert!(feature_path.contains(name), "Feature path should contain the name: {}", name);
        }
    }

    #[test]
    fn test_create_feature_invalid_names() {
        let repo = MockProjectRepository::new();
        
        let invalid_names = vec![
            "-invalid-start",      // starts with dash
            "invalid-end-",        // ends with dash
            "invalid--double",     // double dash
            "InvalidCase",         // uppercase
            "invalid_underscore",  // underscore
            "invalid.dot",         // dot
            "invalid space",       // space
            "invalid@symbol",      // special character
            "",                    // empty string
            "-",                   // just dash
            "--",                  // just double dash
            "UPPERCASE",           // all uppercase
            "Mixed-Case",          // mixed case
        ];
        
        for name in invalid_names {
            let result = create_feature(&repo, name);
            assert!(result.is_err(), "Feature name '{}' should be invalid", name);
            
            match result.unwrap_err() {
                ApplicationError::InvalidFeatureName(invalid_name) => {
                    assert_eq!(invalid_name, name);
                },
                other => panic!("Expected InvalidFeatureName for '{}', got {:?}", name, other),
            }
        }
    }

    #[test]
    fn test_create_feature_repository_failure() {
        let mut repo = MockProjectRepository::new();
        repo.set_operation_to_fail("create_feature");
        
        let result = create_feature(&repo, "valid-feature");
        assert!(result.is_err());
        
        match result.unwrap_err() {
            ApplicationError::FeatureCreationError(msg) => {
                assert!(msg.contains("valid-feature"));
                assert!(msg.contains("Mock creation failure"));
            },
            other => panic!("Expected FeatureCreationError, got {:?}", other),
        }
    }

    #[test]
    fn test_create_feature_path_format() {
        let repo = MockProjectRepository::new();
        
        let result = create_feature(&repo, "test-feature");
        assert!(result.is_ok());
        
        let feature_path = result.unwrap();
        
        // Check path structure: .kiro/specs/YYYY-MM-DD-feature-name
        assert!(feature_path.starts_with(".kiro/specs/"));
        assert!(feature_path.ends_with("-test-feature"));
        
        // Extract date part
        let path_parts: Vec<&str> = feature_path.split('/').collect();
        assert_eq!(path_parts[0], ".kiro");
        assert_eq!(path_parts[1], "specs");
        
        let date_and_name = path_parts[2];
        let date_part = &date_and_name[0..10]; // YYYY-MM-DD is 10 characters
        
        // Verify date format (basic check)
        assert_eq!(date_part.len(), 10);
        assert_eq!(date_part.chars().nth(4).unwrap(), '-');
        assert_eq!(date_part.chars().nth(7).unwrap(), '-');
        
        // Verify it's today's date
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        assert_eq!(date_part, today);
    }

    #[test]
    fn test_create_feature_validation_edge_cases() {
        let repo = MockProjectRepository::new();
        
        // Test single character (valid)
        let result = create_feature(&repo, "a");
        assert!(result.is_ok());
        
        // Test numbers only (valid)
        let result = create_feature(&repo, "123");
        assert!(result.is_ok());
        
        // Test dash in middle (valid)
        let result = create_feature(&repo, "a-b");
        assert!(result.is_ok());
        
        // Test multiple dashes (valid)
        let result = create_feature(&repo, "a-b-c-d");
        assert!(result.is_ok());
        
        // Test numbers with dashes (valid)
        let result = create_feature(&repo, "feature-123-test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_feature_validation_before_repository_call() {
        // This test ensures validation happens before calling repository
        // by using an invalid name with a repo that would fail
        let mut repo = MockProjectRepository::new();
        repo.set_operation_to_fail("create_feature");
        
        let result = create_feature(&repo, "Invalid-Name");
        assert!(result.is_err());
        
        // Should get validation error, not repository error
        match result.unwrap_err() {
            ApplicationError::InvalidFeatureName(_) => {
                // This is correct - validation happens first
            },
            ApplicationError::FeatureCreationError(_) => {
                panic!("Should not reach repository with invalid name");
            },
            other => panic!("Unexpected error type: {:?}", other),
        }
    }

    #[test]
    fn test_create_feature_error_propagation() {
        let mut repo = MockProjectRepository::new();
        repo.set_operation_to_fail("create_feature");
        
        let result = create_feature(&repo, "valid-name");
        assert!(result.is_err());
        
        // Test that repository errors are properly propagated
        match result.unwrap_err() {
            ApplicationError::FeatureCreationError(msg) => {
                assert!(msg.contains("valid-name"));
            },
            other => panic!("Expected FeatureCreationError, got {:?}", other),
        }
    }

    #[test]
    fn test_create_feature_return_value_consistency() {
        let repo = MockProjectRepository::new();
        
        // Test multiple calls with same name should return same path format
        let result1 = create_feature(&repo, "consistent-test");
        let result2 = create_feature(&repo, "consistent-test");
        
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        
        let path1 = result1.unwrap();
        let path2 = result2.unwrap();
        
        // Paths should be identical (assuming called on same day)
        assert_eq!(path1, path2);
    }

    #[test]
    fn test_create_feature_special_characters_validation() {
        let repo = MockProjectRepository::new();
        
        let special_chars = vec![
            "test@feature",
            "test#feature", 
            "test$feature",
            "test%feature",
            "test^feature",
            "test&feature",
            "test*feature",
            "test(feature",
            "test)feature",
            "test+feature",
            "test=feature",
            "test[feature",
            "test]feature",
            "test{feature",
            "test}feature",
            "test|feature",
            "test\\feature",
            "test:feature",
            "test;feature",
            "test\"feature",
            "test'feature",
            "test<feature",
            "test>feature",
            "test,feature",
            "test?feature",
            "test/feature",
            "test~feature",
            "test`feature",
        ];
        
        for name in special_chars {
            let result = create_feature(&repo, name);
            assert!(result.is_err(), "Name with special character should be invalid: {}", name);
        }
    }
}