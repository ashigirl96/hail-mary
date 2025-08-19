use crate::models::error::{MemoryError, Result};
use crate::models::kiro::KiroConfig;
use crate::models::kiro_feature::KiroFeature;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Repository trait for project-related persistence operations
///
/// This trait abstracts the storage layer for project features and configuration,
/// allowing for different implementations (file system, in-memory, etc.)
pub trait ProjectRepository {
    /// Initialize the project structure (.kiro directory and subdirectories)
    ///
    /// # Arguments
    /// * `config` - The configuration to use for initialization
    ///
    /// # Returns
    /// * `Result<()>` - Success or error during initialization
    fn initialize_structure(&self, config: &KiroConfig) -> Result<()>;

    /// Save a new feature to the repository
    ///
    /// # Arguments
    /// * `feature` - The feature to save
    ///
    /// # Returns
    /// * `Result<PathBuf>` - Path where the feature was saved
    fn save_feature(&self, feature: &KiroFeature) -> Result<PathBuf>;

    /// Find a feature by its name
    ///
    /// # Arguments
    /// * `name` - The name of the feature to find
    ///
    /// # Returns
    /// * `Result<Option<KiroFeature>>` - The feature if found, None otherwise
    #[allow(dead_code)] // Used in service layer and future features
    fn find_feature_by_name(&self, name: &str) -> Result<Option<KiroFeature>>;

    /// List all features in the repository
    ///
    /// # Returns
    /// * `Result<Vec<KiroFeature>>` - All features, sorted by name
    fn list_all_features(&self) -> Result<Vec<KiroFeature>>;

    /// Save the configuration to disk
    ///
    /// # Arguments
    /// * `config` - The configuration to save
    ///
    /// # Returns
    /// * `Result<()>` - Success or error during save
    #[allow(dead_code)] // Used in configuration management and future features
    fn save_config(&self, config: &KiroConfig) -> Result<()>;

    /// Load the configuration from disk
    ///
    /// # Returns
    /// * `Result<KiroConfig>` - The loaded configuration or default if not found
    fn load_config(&self) -> Result<KiroConfig>;

    /// Find the .kiro root directory starting from the current directory
    ///
    /// # Returns
    /// * `Result<PathBuf>` - Path to the .kiro directory
    fn find_kiro_root(&self) -> Result<PathBuf>;

    /// Update .gitignore with the provided entries
    ///
    /// # Arguments
    /// * `entries` - The entries to add to .gitignore
    ///
    /// # Returns
    /// * `Result<()>` - Success or error during update
    fn update_gitignore(&self, entries: &[String]) -> Result<()>;
}

// ========================================
// File System Implementation
// ========================================

/// File system based implementation of ProjectRepository
///
/// This implementation directly uses std::fs for all file operations,
/// without any abstraction layer (no FileSystem trait).
pub struct FileProjectRepository {
    base_path: PathBuf,
}

impl FileProjectRepository {
    /// Create a new FileProjectRepository with default base path
    pub fn new() -> Self {
        Self {
            base_path: PathBuf::from(".kiro"),
        }
    }

    /// Create a new FileProjectRepository with custom base path
    #[allow(dead_code)] // Used in testing and configuration
    pub fn with_base_path(base_path: PathBuf) -> Self {
        Self { base_path }
    }
}

impl Default for FileProjectRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectRepository for FileProjectRepository {
    fn initialize_structure(&self, config: &KiroConfig) -> Result<()> {
        // Create all required directories
        for dir in config.required_directories() {
            fs::create_dir_all(&dir).map_err(MemoryError::Io)?;
        }

        // Create config.toml by serializing the actual config
        let config_path = config.root_dir.join("config.toml");
        let config_content = toml::to_string_pretty(config)?;
        fs::write(&config_path, config_content).map_err(MemoryError::Io)?;

        // Update .gitignore with default entries
        let entries: Vec<String> = KiroConfig::default_gitignore_entries()
            .iter()
            .map(|s| s.to_string())
            .collect();
        self.update_gitignore(&entries)?;

        Ok(())
    }

    fn save_feature(&self, feature: &KiroFeature) -> Result<PathBuf> {
        let feature_path = self.base_path.join("specs").join(&feature.directory_name);

        // Create feature directory
        fs::create_dir_all(&feature_path).map_err(MemoryError::Io)?;

        // Create all required feature files
        for filename in KiroConfig::default_feature_files() {
            let file_path = feature_path.join(filename);
            let content = match filename {
                "spec.json" => {
                    // Serialize the feature to JSON for spec.json
                    serde_json::to_string_pretty(feature)?
                }
                _ => {
                    // Other files start empty
                    String::new()
                }
            };
            fs::write(&file_path, content).map_err(MemoryError::Io)?;
        }

        Ok(feature_path)
    }

    fn find_feature_by_name(&self, name: &str) -> Result<Option<KiroFeature>> {
        let features = self.list_all_features()?;
        Ok(features.into_iter().find(|f| f.name == name))
    }

    fn list_all_features(&self) -> Result<Vec<KiroFeature>> {
        let specs_dir = self.base_path.join("specs");

        // Return empty list if specs directory doesn't exist
        if !specs_dir.exists() {
            return Ok(vec![]);
        }

        let mut features = Vec::new();

        // Read all directories in specs/
        for entry in fs::read_dir(&specs_dir).map_err(MemoryError::Io)? {
            let entry = entry.map_err(MemoryError::Io)?;
            let path = entry.path();

            if path.is_dir() {
                // Try to read spec.json from the directory
                let spec_file = path.join("spec.json");
                if spec_file.exists() {
                    let content = fs::read_to_string(&spec_file).map_err(MemoryError::Io)?;
                    // Parse the feature from spec.json
                    if let Ok(mut feature) = serde_json::from_str::<KiroFeature>(&content) {
                        // Set the path to the actual directory
                        feature.path = Some(path);
                        features.push(feature);
                    }
                }
            }
        }

        // Sort features by name for consistent ordering
        features.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(features)
    }

    fn save_config(&self, config: &KiroConfig) -> Result<()> {
        let config_path = config.root_dir.join("config.toml");
        let content = toml::to_string_pretty(config)
            .map_err(|e| MemoryError::InvalidInput(format!("Failed to serialize config: {}", e)))?;
        fs::write(&config_path, content).map_err(MemoryError::Io)?;
        Ok(())
    }

    fn load_config(&self) -> Result<KiroConfig> {
        let kiro_root = self.find_kiro_root()?;
        KiroConfig::load_from_root(&kiro_root)
    }

    fn find_kiro_root(&self) -> Result<PathBuf> {
        let mut current_dir = std::env::current_dir().map_err(MemoryError::Io)?;

        loop {
            let kiro_dir = current_dir.join(".kiro");
            if kiro_dir.exists() && kiro_dir.is_dir() {
                return Ok(kiro_dir);
            }

            // Move up to parent directory
            if let Some(parent) = current_dir.parent() {
                current_dir = parent.to_path_buf();
            } else {
                // Reached root without finding .kiro
                return Err(MemoryError::NotFound(
                    ".kiro directory not found".to_string(),
                ));
            }
        }
    }

    fn update_gitignore(&self, entries: &[String]) -> Result<()> {
        let gitignore_path = Path::new(".gitignore");

        if gitignore_path.exists() {
            // Read existing content
            let content = fs::read_to_string(gitignore_path).map_err(MemoryError::Io)?;

            // Check which entries need to be added
            let mut needs_update = false;
            let mut new_entries = Vec::new();

            for entry in entries {
                if !content.contains(entry) {
                    needs_update = true;
                    new_entries.push(entry.clone());
                }
            }

            // Append new entries if needed
            if needs_update {
                use std::fs::OpenOptions;

                let mut file = OpenOptions::new()
                    .append(true)
                    .open(gitignore_path)
                    .map_err(MemoryError::Io)?;

                // Add a newline before our entries if file doesn't end with one
                if !content.ends_with('\n') {
                    writeln!(file)?;
                }

                // Write new entries
                for entry in new_entries {
                    writeln!(file, "{}", entry)?;
                }
            }
        } else {
            // Create new .gitignore with our entries
            let content = entries.join("\n") + "\n";
            fs::write(gitignore_path, content).map_err(MemoryError::Io)?;
        }

        Ok(())
    }
}

// Note: InMemoryProjectRepository has been removed as TestDirectory provides
// sufficient isolated testing with real file system operations in temporary directories

// ========================================
// Tests
// ========================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::common::TestDirectory;
    use chrono::Utc;

    // ========================================
    // FileProjectRepository Tests
    // ========================================

    #[test]
    fn test_file_repository_initialize_structure() {
        let _test_dir = TestDirectory::new();
        let repo = FileProjectRepository::new();
        let config = KiroConfig::default();

        let result = repo.initialize_structure(&config);
        assert!(result.is_ok());

        // Check directories were created
        assert!(Path::new(".kiro").exists());
        assert!(Path::new(".kiro/memory").exists());
        assert!(Path::new(".kiro/specs").exists());

        // Check config.toml was created
        assert!(Path::new(".kiro/config.toml").exists());

        // Check .gitignore was updated
        assert!(Path::new(".gitignore").exists());
        let gitignore_content = fs::read_to_string(".gitignore").unwrap();
        assert!(gitignore_content.contains(".kiro/memory/db.sqlite3"));
    }

    #[test]
    fn test_file_repository_save_feature() {
        let _test_dir = TestDirectory::new();

        // Create .kiro/specs directory first
        fs::create_dir_all(".kiro/specs").unwrap();

        let repo = FileProjectRepository::new();
        let feature = KiroFeature::new(
            "test-feature".to_string(),
            format!("{}-test-feature", Utc::now().format("%Y-%m-%d")),
        );

        let result = repo.save_feature(&feature);
        assert!(result.is_ok());

        let feature_path = result.unwrap();
        assert!(feature_path.exists());

        // Check all files were created
        assert!(feature_path.join("requirements.md").exists());
        assert!(feature_path.join("design.md").exists());
        assert!(feature_path.join("tasks.md").exists());
        assert!(feature_path.join("spec.json").exists());

        // Check spec.json contains the feature data
        let spec_content = fs::read_to_string(feature_path.join("spec.json")).unwrap();
        let loaded_feature: KiroFeature = serde_json::from_str(&spec_content).unwrap();
        assert_eq!(loaded_feature.name, "test-feature");
    }

    #[test]
    fn test_file_repository_list_all_features() {
        let _test_dir = TestDirectory::new();
        let repo = FileProjectRepository::new();

        // Create .kiro/specs directory
        fs::create_dir_all(".kiro/specs").unwrap();

        // Create two test features
        let feature1 = KiroFeature::new(
            "feature-alpha".to_string(),
            "2024-01-01-feature-alpha".to_string(),
        );
        let feature2 = KiroFeature::new(
            "feature-beta".to_string(),
            "2024-01-02-feature-beta".to_string(),
        );

        repo.save_feature(&feature1).unwrap();
        repo.save_feature(&feature2).unwrap();

        // List all features
        let features = repo.list_all_features().unwrap();

        assert_eq!(features.len(), 2);
        // Should be sorted by name
        assert_eq!(features[0].name, "feature-alpha");
        assert_eq!(features[1].name, "feature-beta");
        // Path should be set
        assert!(features[0].path.is_some());
        assert!(features[1].path.is_some());
    }

    #[test]
    fn test_file_repository_find_feature_by_name() {
        let _test_dir = TestDirectory::new();
        let repo = FileProjectRepository::new();

        // Create .kiro/specs directory
        fs::create_dir_all(".kiro/specs").unwrap();

        let feature = KiroFeature::new(
            "test-feature".to_string(),
            "2024-01-01-test-feature".to_string(),
        );
        repo.save_feature(&feature).unwrap();

        // Find existing feature
        let found = repo.find_feature_by_name("test-feature").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "test-feature");

        // Try to find non-existent feature
        let not_found = repo.find_feature_by_name("non-existent").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_file_repository_save_and_load_config() {
        let _test_dir = TestDirectory::new();

        // Create .kiro directory
        fs::create_dir_all(".kiro").unwrap();

        let repo = FileProjectRepository::new();
        let mut config = KiroConfig::default();
        config.memory.types.push("custom-type".to_string());

        // Save config
        repo.save_config(&config).unwrap();

        // Load config
        let loaded = repo.load_config().unwrap();
        assert_eq!(loaded.memory.types.len(), 4); // 3 default + 1 custom
        assert!(loaded.memory.types.contains(&"custom-type".to_string()));
    }

    #[test]
    fn test_file_repository_find_kiro_root() {
        let _test_dir = TestDirectory::new();

        // Create .kiro directory
        fs::create_dir_all(".kiro").unwrap();

        let repo = FileProjectRepository::new();
        let root = repo.find_kiro_root();

        assert!(root.is_ok());
        assert!(root.unwrap().ends_with(".kiro"));
    }

    #[test]
    fn test_file_repository_find_kiro_root_not_found() {
        let _test_dir = TestDirectory::new();

        // Don't create .kiro directory
        let repo = FileProjectRepository::new();
        let root = repo.find_kiro_root();

        assert!(root.is_err());
        match root.unwrap_err() {
            MemoryError::NotFound(msg) => assert!(msg.contains(".kiro")),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_file_repository_update_gitignore_new_file() {
        let _test_dir = TestDirectory::new();
        let repo = FileProjectRepository::new();

        let entries = vec!["test-entry-1".to_string(), "test-entry-2".to_string()];

        repo.update_gitignore(&entries).unwrap();

        let content = fs::read_to_string(".gitignore").unwrap();
        assert!(content.contains("test-entry-1"));
        assert!(content.contains("test-entry-2"));
    }

    #[test]
    fn test_file_repository_update_gitignore_existing_file() {
        let _test_dir = TestDirectory::new();

        // Create existing .gitignore
        fs::write(".gitignore", "existing-entry\n").unwrap();

        let repo = FileProjectRepository::new();
        let entries = vec!["new-entry".to_string()];

        repo.update_gitignore(&entries).unwrap();

        let content = fs::read_to_string(".gitignore").unwrap();
        assert!(content.contains("existing-entry"));
        assert!(content.contains("new-entry"));
    }

    #[test]
    fn test_file_repository_update_gitignore_duplicate_entry() {
        let _test_dir = TestDirectory::new();

        // Create .gitignore with existing entry
        fs::write(".gitignore", "duplicate-entry\n").unwrap();

        let repo = FileProjectRepository::new();
        let entries = vec!["duplicate-entry".to_string(), "new-entry".to_string()];

        repo.update_gitignore(&entries).unwrap();

        let content = fs::read_to_string(".gitignore").unwrap();
        // Should only appear once
        let count = content.matches("duplicate-entry").count();
        assert_eq!(count, 1);
        assert!(content.contains("new-entry"));
    }

    // Note: InMemoryProjectRepository tests have been removed as TestDirectory
    // provides sufficient isolated testing with real file system operations
}
