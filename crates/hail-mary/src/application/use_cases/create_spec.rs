use crate::application::errors::ApplicationError;
use crate::application::repositories::SpecRepositoryInterface;

pub fn create_spec(
    spec_repo: &dyn SpecRepositoryInterface,
    name: &str,
) -> Result<String, ApplicationError> {
    // Validate spec name (must be kebab-case) at use case level for consistency
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_lowercase() || c == '-' || c.is_numeric())
        || name.starts_with('-')
        || name.ends_with('-')
        || name.contains("--")
    {
        return Err(ApplicationError::InvalidSpecName(name.to_string()));
    }

    // Create spec through repository
    spec_repo.create_spec(name)?;

    // Return spec path for user feedback
    let date = chrono::Utc::now().format("%Y-%m-%d");
    Ok(format!(".kiro/specs/{}-{}", date, name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_helpers::MockSpecRepository;

    #[test]
    fn test_create_spec_success() {
        let repo = MockSpecRepository::new();

        let result = create_spec(&repo, "user-authentication");
        assert!(result.is_ok());

        let spec_path = result.unwrap();
        assert!(spec_path.starts_with(".kiro/specs/"));
        assert!(spec_path.ends_with("-user-authentication"));
        assert!(spec_path.contains("2025")); // Should contain current year
    }

    #[test]
    fn test_create_spec_valid_names() {
        let repo = MockSpecRepository::new();

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
            let result = create_spec(&repo, name);
            assert!(result.is_ok(), "Feature name '{}' should be valid", name);

            let spec_path = result.unwrap();
            assert!(
                spec_path.contains(name),
                "Feature path should contain the name: {}",
                name
            );
        }
    }

    #[test]
    fn test_create_spec_invalid_names() {
        let repo = MockSpecRepository::new();

        let invalid_names = vec![
            "-invalid-start",     // starts with dash
            "invalid-end-",       // ends with dash
            "invalid--double",    // double dash
            "InvalidCase",        // uppercase
            "invalid_underscore", // underscore
            "invalid.dot",        // dot
            "invalid space",      // space
            "invalid@symbol",     // special character
            "",                   // empty string
            "-",                  // just dash
            "--",                 // just double dash
            "UPPERCASE",          // all uppercase
            "Mixed-Case",         // mixed case
        ];

        for name in invalid_names {
            let result = create_spec(&repo, name);
            assert!(result.is_err(), "Feature name '{}' should be invalid", name);

            match result.unwrap_err() {
                ApplicationError::InvalidSpecName(invalid_name) => {
                    assert_eq!(invalid_name, name);
                }
                other => panic!("Expected InvalidSpecName for '{}', got {:?}", name, other),
            }
        }
    }

    #[test]
    fn test_create_spec_repository_failure() {
        let repo = MockSpecRepository::new();
        repo.set_operation_to_fail("create_spec");

        let result = create_spec(&repo, "valid-feature");
        assert!(result.is_err());

        match result.unwrap_err() {
            ApplicationError::SpecCreationError(msg) => {
                assert!(msg.contains("valid-feature"));
                assert!(msg.contains("Mock creation failure"));
            }
            other => panic!("Expected SpecCreationError, got {:?}", other),
        }
    }

    #[test]
    fn test_create_spec_path_format() {
        let repo = MockSpecRepository::new();

        let result = create_spec(&repo, "test-feature");
        assert!(result.is_ok());

        let spec_path = result.unwrap();

        // Check path structure: .kiro/specs/YYYY-MM-DD-feature-name
        assert!(spec_path.starts_with(".kiro/specs/"));
        assert!(spec_path.ends_with("-test-feature"));

        // Extract date part
        let path_parts: Vec<&str> = spec_path.split('/').collect();
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
    fn test_create_spec_validation_edge_cases() {
        let repo = MockSpecRepository::new();

        // Test single character (valid)
        let result = create_spec(&repo, "a");
        assert!(result.is_ok());

        // Test numbers only (valid)
        let result = create_spec(&repo, "123");
        assert!(result.is_ok());

        // Test dash in middle (valid)
        let result = create_spec(&repo, "a-b");
        assert!(result.is_ok());

        // Test multiple dashes (valid)
        let result = create_spec(&repo, "a-b-c-d");
        assert!(result.is_ok());

        // Test numbers with dashes (valid)
        let result = create_spec(&repo, "feature-123-test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_spec_validation_before_repository_call() {
        // This test ensures validation happens before calling repository
        // by using an invalid name with a repo that would fail
        let repo = MockSpecRepository::new();
        repo.set_operation_to_fail("create_spec");

        let result = create_spec(&repo, "Invalid-Name");
        assert!(result.is_err());

        // Should get validation error, not repository error
        match result.unwrap_err() {
            ApplicationError::InvalidSpecName(_) => {
                // This is correct - validation happens first
            }
            ApplicationError::SpecCreationError(_) => {
                panic!("Should not reach repository with invalid name");
            }
            other => panic!("Unexpected error type: {:?}", other),
        }
    }

    #[test]
    fn test_create_spec_error_propagation() {
        let repo = MockSpecRepository::new();
        repo.set_operation_to_fail("create_spec");

        let result = create_spec(&repo, "valid-name");
        assert!(result.is_err());

        // Test that repository errors are properly propagated
        match result.unwrap_err() {
            ApplicationError::SpecCreationError(msg) => {
                assert!(msg.contains("valid-name"));
            }
            other => panic!("Expected SpecCreationError, got {:?}", other),
        }
    }

    #[test]
    fn test_create_spec_return_value_consistency() {
        let repo = MockSpecRepository::new();

        // Test multiple calls with same name should return same path format
        let result1 = create_spec(&repo, "consistent-test");
        let result2 = create_spec(&repo, "consistent-test");

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        let path1 = result1.unwrap();
        let path2 = result2.unwrap();

        // Paths should be identical (assuming called on same day)
        assert_eq!(path1, path2);
    }

    #[test]
    fn test_create_spec_special_characters_validation() {
        let repo = MockSpecRepository::new();

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
            let result = create_spec(&repo, name);
            assert!(
                result.is_err(),
                "Name with special character should be invalid: {}",
                name
            );
        }
    }
}
