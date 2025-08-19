use crate::models::error::{MemoryError, Result};
use crate::models::kiro::KiroConfig;
use crate::models::kiro_feature::KiroFeature;
use crate::repositories::project::ProjectRepository;
use std::path::PathBuf;

/// Project service that implements business use cases
///
/// This service layer orchestrates business operations using the repository
/// pattern for persistence and domain models for business rules.
pub struct ProjectService<R: ProjectRepository> {
    repository: R,
    config: KiroConfig,
}

impl<R: ProjectRepository> ProjectService<R> {
    /// Create a new ProjectService by loading configuration from repository
    pub fn new(repository: R) -> Result<Self> {
        let config = repository.load_config()?;
        Ok(Self { repository, config })
    }

    /// Create a new ProjectService with explicit configuration
    pub fn with_config(repository: R, config: KiroConfig) -> Self {
        Self { repository, config }
    }

    /// Initialize a new project structure
    ///
    /// Creates the .kiro directory structure, configuration file, and updates .gitignore.
    /// Use `force` to overwrite existing configuration.
    pub fn initialize_project(&self, force: bool) -> Result<()> {
        // Check for existing project
        if !force && self.repository.find_kiro_root().is_ok() {
            return Err(MemoryError::InvalidInput(
                ".kiro directory already exists. Use --force to overwrite.".to_string(),
            ));
        }

        // Initialize project structure
        self.repository.initialize_structure(&self.config)?;

        Ok(())
    }

    /// Create a new feature specification
    ///
    /// Validates the feature name, checks for duplicates, and creates
    /// the feature directory with all required files.
    pub fn create_new_feature(&self, name: &str) -> Result<PathBuf> {
        // Validate feature name (domain rule)
        if !KiroFeature::is_valid_name(name) {
            return Err(MemoryError::InvalidInput(format!(
                "Invalid feature name: {}. Must be kebab-case.",
                name
            )));
        }

        // Check for duplicate features (domain rule)
        let existing = self.repository.list_all_features()?;
        if !KiroFeature::can_create(name, &existing) {
            return Err(MemoryError::InvalidInput(format!(
                "Feature '{}' already exists",
                name
            )));
        }

        // Generate directory name using business rule
        let directory_name = self.config.generate_feature_dir_name(name);
        let feature = KiroFeature::new(name.to_string(), directory_name);

        // Persist the feature
        let feature_path = self.repository.save_feature(&feature)?;

        Ok(feature_path)
    }

    /// List all features in the project
    ///
    /// Returns a sorted list of all features.
    #[allow(dead_code)] // Used in CLI commands and future features
    pub fn list_features(&self) -> Result<Vec<KiroFeature>> {
        self.repository.list_all_features()
    }

    /// Find a specific feature by name
    ///
    /// Returns None if the feature doesn't exist.
    #[allow(dead_code)] // Used in CLI commands and future features
    pub fn find_feature(&self, name: &str) -> Result<Option<KiroFeature>> {
        self.repository.find_feature_by_name(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::project::FileProjectRepository;
    use crate::tests::common::TestDirectory;

    #[test]
    fn test_create_feature_success() {
        let _test_dir = TestDirectory::new();
        let repository = FileProjectRepository::new();
        let config = KiroConfig::default();
        let service = ProjectService::with_config(repository, config);

        // Create a feature
        let result = service.create_new_feature("test-feature");
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("test-feature"));
    }

    #[test]
    fn test_create_feature_invalid_name() {
        let _test_dir = TestDirectory::new();
        let repository = FileProjectRepository::new();
        let config = KiroConfig::default();
        let service = ProjectService::with_config(repository, config);

        // Test various invalid names
        let long_name = "a".repeat(51);
        let invalid_names = vec![
            "InvalidName",      // Capital letters
            "invalid_name",     // Underscore
            "invalid-name-",    // Trailing hyphen
            "-invalid-name",    // Leading hyphen
            "invalid--name",    // Double hyphen
            "",                 // Empty string
            long_name.as_str(), // Too long
        ];

        for invalid_name in invalid_names {
            let result = service.create_new_feature(invalid_name);
            assert!(result.is_err());

            if let Err(MemoryError::InvalidInput(msg)) = result {
                assert!(msg.contains("Invalid feature name"));
            } else {
                panic!("Expected InvalidInput error for name: {}", invalid_name);
            }
        }
    }

    #[test]
    fn test_duplicate_feature_fails() {
        let _test_dir = TestDirectory::new();
        let repository = FileProjectRepository::new();
        let config = KiroConfig::default();
        let service = ProjectService::with_config(repository, config);

        // Create first feature
        assert!(service.create_new_feature("test-feature").is_ok());

        // Try to create duplicate
        let result = service.create_new_feature("test-feature");
        assert!(result.is_err());

        if let Err(MemoryError::InvalidInput(msg)) = result {
            assert!(msg.contains("already exists"));
        } else {
            panic!("Expected InvalidInput error for duplicate feature");
        }
    }

    #[test]
    fn test_list_features_empty() {
        let _test_dir = TestDirectory::new();
        let repository = FileProjectRepository::new();
        let config = KiroConfig::default();
        let service = ProjectService::with_config(repository, config);

        let features = service.list_features().unwrap();
        assert_eq!(features.len(), 0);
    }

    #[test]
    fn test_list_features_multiple() {
        let _test_dir = TestDirectory::new();
        let repository = FileProjectRepository::new();
        let config = KiroConfig::default();
        let service = ProjectService::with_config(repository, config);

        // Create multiple features
        service.create_new_feature("feature-a").unwrap();
        service.create_new_feature("feature-b").unwrap();
        service.create_new_feature("feature-c").unwrap();

        let features = service.list_features().unwrap();
        assert_eq!(features.len(), 3);

        // Check they are sorted by name
        assert_eq!(features[0].name, "feature-a");
        assert_eq!(features[1].name, "feature-b");
        assert_eq!(features[2].name, "feature-c");
    }

    #[test]
    fn test_find_feature_exists() {
        let _test_dir = TestDirectory::new();
        let repository = FileProjectRepository::new();
        let config = KiroConfig::default();
        let service = ProjectService::with_config(repository, config);

        // Create a feature
        service.create_new_feature("test-feature").unwrap();

        // Find it
        let found = service.find_feature("test-feature").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "test-feature");
    }

    #[test]
    fn test_find_feature_not_exists() {
        let _test_dir = TestDirectory::new();
        let repository = FileProjectRepository::new();
        let config = KiroConfig::default();
        let service = ProjectService::with_config(repository, config);

        let found = service.find_feature("non-existent").unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_initialize_project() {
        let _test_dir = TestDirectory::new();
        let repository = FileProjectRepository::new();
        let config = KiroConfig::default();
        let service = ProjectService::with_config(repository, config);

        // Initialize should succeed
        let result = service.initialize_project(false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_initialize_project_already_exists() {
        let _test_dir = TestDirectory::new();
        let repository = FileProjectRepository::new();
        let config = KiroConfig::default();

        // Pre-initialize the repository
        repository.initialize_structure(&config).unwrap();

        let service = ProjectService::with_config(repository, config);

        // Initialize without force should fail
        let result = service.initialize_project(false);
        assert!(result.is_err());

        if let Err(MemoryError::InvalidInput(msg)) = result {
            assert!(msg.contains("already exists"));
        } else {
            panic!("Expected InvalidInput error for existing project");
        }
    }

    #[test]
    fn test_initialize_project_with_force() {
        let _test_dir = TestDirectory::new();
        let repository = FileProjectRepository::new();
        let config = KiroConfig::default();

        // Pre-initialize the repository
        repository.initialize_structure(&config).unwrap();

        let service = ProjectService::with_config(repository, config);

        // Initialize with force should succeed
        let result = service.initialize_project(true);
        assert!(result.is_ok());
    }
}
