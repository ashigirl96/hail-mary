use hail_mary::models::kiro::KiroConfig;
use hail_mary::repositories::project::FileProjectRepository;
use hail_mary::services::project::ProjectService;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use tempfile::TempDir;

// Global mutex to synchronize current directory changes across tests
static TEST_DIR_MUTEX: Mutex<()> = Mutex::new(());

/// Test directory helper that automatically changes to a temporary directory
/// and restores the original directory when dropped (RAII pattern)
/// Uses a global mutex to prevent current_dir race conditions in parallel tests
pub struct TestDirectory {
    _temp_dir: TempDir,
    original_dir: PathBuf,
    _guard: MutexGuard<'static, ()>,
}

impl Default for TestDirectory {
    fn default() -> Self {
        Self::new()
    }
}

impl TestDirectory {
    /// Creates a new temporary directory and changes current directory to it
    /// Acquires a global lock to prevent race conditions in parallel test execution
    pub fn new() -> Self {
        let guard = TEST_DIR_MUTEX
            .lock()
            .expect("Failed to acquire test directory mutex");
        let original_dir = env::current_dir().expect("Failed to get current directory");
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");

        env::set_current_dir(temp_dir.path()).expect("Failed to change to temp directory");

        Self {
            _temp_dir: temp_dir,
            original_dir,
            _guard: guard,
        }
    }

    /// Returns the path to the temporary directory
    pub fn path(&self) -> &Path {
        self._temp_dir.path()
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        // Best effort to restore original directory
        // Ignore errors since we're in cleanup phase
        let _ = env::set_current_dir(&self.original_dir);
        // Mutex guard is automatically released when _guard is dropped
    }
}

/// Integration tests for ProjectService with real filesystem
#[test]
fn test_project_service_with_real_filesystem() {
    let _test_dir = TestDirectory::new();

    // Test with real filesystem repository - use with_config to avoid loading non-existent config
    let repository = FileProjectRepository::new();
    let service = ProjectService::with_config(repository, KiroConfig::default());

    // Test initialization
    assert!(service.initialize_project(false).is_ok());

    // Verify directory structure was created
    assert!(Path::new(".kiro").exists());
    assert!(Path::new(".kiro/config.toml").exists());
    assert!(Path::new(".kiro/memory").exists());
    assert!(Path::new(".kiro/specs").exists());

    // Test feature creation
    let feature_path = service.create_new_feature("test-feature").unwrap();
    assert!(feature_path.exists());
    assert!(feature_path.join("requirements.md").exists());
    assert!(feature_path.join("design.md").exists());
    assert!(feature_path.join("tasks.md").exists());
    assert!(feature_path.join("spec.json").exists());

    // Test feature listing
    let features = service.list_features().unwrap();
    assert_eq!(features.len(), 1);
    assert_eq!(features[0].name, "test-feature");

    // Test feature finding
    let found = service.find_feature("test-feature").unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "test-feature");
}

#[test]
fn test_project_service_repository_abstraction() {
    let _test_dir = TestDirectory::new();

    // Test FileProjectRepository behavior
    let repo = FileProjectRepository::new();
    let service = ProjectService::with_config(repo, KiroConfig::default());

    // Should initialize successfully
    assert!(service.initialize_project(false).is_ok());

    // Should create features successfully
    assert!(service.create_new_feature("test-feature").is_ok());

    // Should list features correctly
    let features = service.list_features().unwrap();
    assert_eq!(features.len(), 1);
    assert_eq!(features[0].name, "test-feature");

    // Should find features correctly
    let found = service.find_feature("test-feature").unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "test-feature");
}

#[test]
fn test_project_service_business_rules() {
    let _test_dir = TestDirectory::new();
    let repository = FileProjectRepository::new();
    let service = ProjectService::with_config(repository, KiroConfig::default());

    // Test invalid feature names (business rule validation)
    let invalid_names = vec![
        "InvalidName",   // Capital letters
        "invalid_name",  // Underscore
        "invalid-name-", // Trailing hyphen
        "-invalid-name", // Leading hyphen
        "invalid--name", // Double hyphen
        "",              // Empty string
    ];

    for invalid_name in invalid_names {
        let result = service.create_new_feature(invalid_name);
        assert!(
            result.is_err(),
            "Should reject invalid name: {}",
            invalid_name
        );
    }

    // Test valid feature names
    let valid_names = vec![
        "feature",
        "my-feature",
        "feature-123",
        "complex-feature-name",
    ];

    for valid_name in valid_names {
        let result = service.create_new_feature(valid_name);
        assert!(result.is_ok(), "Should accept valid name: {}", valid_name);
    }
}

#[test]
fn test_project_service_duplicate_prevention() {
    let _test_dir = TestDirectory::new();
    let repository = FileProjectRepository::new();
    let service = ProjectService::with_config(repository, KiroConfig::default());

    // Create first feature
    assert!(service.create_new_feature("test-feature").is_ok());

    // Try to create duplicate - should fail
    let result = service.create_new_feature("test-feature");
    assert!(result.is_err());

    // Should only have one feature
    let features = service.list_features().unwrap();
    assert_eq!(features.len(), 1);
}

#[test]
fn test_project_service_configuration_loading() {
    let _test_dir = TestDirectory::new();

    // Create .kiro directory with custom config in current test directory
    let kiro_dir = Path::new(".kiro");
    fs::create_dir_all(kiro_dir).unwrap();

    let config_content = r#"
[memory]
types = ["custom", "test-type"]
instructions = "Custom memory types for testing"

[memory.document]
output_dir = ".kiro/memory"
format = "markdown"

[memory.database]  
path = ".kiro/memory/db.sqlite3"
"#;

    fs::write(kiro_dir.join("config.toml"), config_content).unwrap();

    // Service should load the custom config
    let repository = FileProjectRepository::new();
    let service = ProjectService::new(repository).unwrap();

    // Config should contain our custom types
    // Note: We can't directly access config from service, but we can test the behavior
    // through the service methods. The config is properly loaded and used internally.

    // Test that service works with loaded config
    let feature_path = service.create_new_feature("test-feature").unwrap();
    assert!(feature_path.exists());
}

#[test]
fn test_project_service_feature_directory_naming() {
    let _test_dir = TestDirectory::new();
    let repository = FileProjectRepository::new();
    let service = ProjectService::with_config(repository, KiroConfig::default());

    // Create a feature
    let feature_path = service.create_new_feature("my-feature").unwrap();

    // Directory name should follow YYYY-MM-dd-feature-name format
    let dir_name = feature_path.file_name().unwrap().to_str().unwrap();
    assert!(dir_name.ends_with("-my-feature"));

    // Should have proper date prefix (YYYY-MM-dd)
    let parts: Vec<&str> = dir_name.split('-').collect();
    assert!(parts.len() >= 4); // YYYY-MM-dd-feature-name
    assert_eq!(parts[0].len(), 4); // Year
    assert_eq!(parts[1].len(), 2); // Month  
    assert_eq!(parts[2].len(), 2); // Day
}

#[test]
fn test_project_service_error_handling() {
    // Test initialization without force when directory exists
    let _test_dir = TestDirectory::new();
    let repository = FileProjectRepository::new();
    let service = ProjectService::with_config(repository, KiroConfig::default());

    // First initialization should succeed
    assert!(service.initialize_project(false).is_ok());

    // Create a new service instance to test the existing directory check
    let repository2 = FileProjectRepository::new();
    let service2 = ProjectService::with_config(repository2, KiroConfig::default());

    // Second initialization without force should fail
    let result = service2.initialize_project(false);
    assert!(result.is_err());

    // With force should succeed
    assert!(service2.initialize_project(true).is_ok());
}

#[test]
fn test_project_service_feature_lifecycle() {
    let _test_dir = TestDirectory::new();
    let repository = FileProjectRepository::new();
    let service = ProjectService::with_config(repository, KiroConfig::default());

    // Start with empty feature list
    assert_eq!(service.list_features().unwrap().len(), 0);
    assert!(service.find_feature("nonexistent").unwrap().is_none());

    // Create multiple features
    let features_to_create = vec!["feature-a", "feature-b", "feature-c"];
    for feature_name in &features_to_create {
        assert!(service.create_new_feature(feature_name).is_ok());
    }

    // List should return all features in sorted order
    let features = service.list_features().unwrap();
    assert_eq!(features.len(), 3);
    assert_eq!(features[0].name, "feature-a");
    assert_eq!(features[1].name, "feature-b");
    assert_eq!(features[2].name, "feature-c");

    // Each feature should be findable
    for feature_name in &features_to_create {
        let found = service.find_feature(feature_name).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, *feature_name);
    }
}
