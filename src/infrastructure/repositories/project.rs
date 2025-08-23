use crate::application::errors::ApplicationError;
use crate::application::repositories::ProjectRepository as ProjectRepositoryTrait;
use crate::domain::entities::memory::Memory;
use crate::domain::entities::project::{DocumentFormat, ProjectConfig};
use crate::infrastructure::filesystem::path_manager::PathManager;
use serde::{Deserialize, Serialize};
use std::fs;

// Type-safe TOML configuration structures
#[derive(Debug, Serialize, Deserialize)]
struct TomlConfig {
    memory: MemoryConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct MemoryConfig {
    types: Vec<String>,
    instructions: String,
    document: DocumentConfig,
    database: DatabaseConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct DocumentConfig {
    output_dir: String,
    format: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DatabaseConfig {
    path: String,
}

pub struct ProjectRepository {
    path_manager: PathManager,
}

impl ProjectRepository {
    pub fn new(path_manager: PathManager) -> Self {
        Self { path_manager }
    }
}

impl ProjectRepositoryTrait for ProjectRepository {
    fn initialize(&self) -> Result<(), ApplicationError> {
        // Create .kiro directory structure
        let kiro_dir = self.path_manager.kiro_dir(true);
        fs::create_dir_all(&kiro_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to create .kiro directory: {}", e))
        })?;

        // Create subdirectories
        let memory_dir = self.path_manager.memory_dir(true);
        fs::create_dir_all(&memory_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to create memory directory: {}", e))
        })?;

        let specs_dir = self.path_manager.specs_dir(true);
        fs::create_dir_all(&specs_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to create specs directory: {}", e))
        })?;

        Ok(())
    }

    fn exists(&self) -> Result<bool, ApplicationError> {
        Ok(self.path_manager.kiro_dir(true).exists())
    }

    fn save_config(&self, config: &ProjectConfig) -> Result<(), ApplicationError> {
        let config_path = self.path_manager.config_path(true);

        // Create type-safe TOML structure
        let toml_config = TomlConfig {
            memory: MemoryConfig {
                types: config.memory_types.clone(),
                instructions: config.instructions.clone(),
                document: DocumentConfig {
                    output_dir: self.path_manager.memory_dir(false).display().to_string(),
                    format: "markdown".to_string(),
                },
                database: DatabaseConfig {
                    path: self
                        .path_manager
                        .memory_db_path(false)
                        .display()
                        .to_string(),
                },
            },
        };

        // Serialize to TOML
        let config_content = toml::to_string_pretty(&toml_config).map_err(|e| {
            ApplicationError::ConfigurationError(format!("Failed to serialize config: {}", e))
        })?;

        fs::write(config_path, config_content).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to write config file: {}", e))
        })?;

        Ok(())
    }

    fn load_config(&self) -> Result<ProjectConfig, ApplicationError> {
        let config_path = self.path_manager.config_path(true);

        if !config_path.exists() {
            // Return default config if file doesn't exist
            return Ok(ProjectConfig::default_for_new_project());
        }

        let content = fs::read_to_string(config_path).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to read config file: {}", e))
        })?;

        // Type-safe deserialization
        let toml_config: TomlConfig = toml::from_str(&content).map_err(|e| {
            ApplicationError::ConfigurationError(format!("Failed to parse config: {}", e))
        })?;

        // Convert to domain entity
        Ok(ProjectConfig {
            memory_types: toml_config.memory.types,
            instructions: toml_config.memory.instructions,
            document_format: DocumentFormat::Markdown,
        })
    }

    fn update_gitignore(&self) -> Result<(), ApplicationError> {
        let gitignore_path = self.path_manager.project_root().join(".gitignore");

        let mut content = if gitignore_path.exists() {
            fs::read_to_string(&gitignore_path).map_err(|e| {
                ApplicationError::FileSystemError(format!("Failed to read .gitignore: {}", e))
            })?
        } else {
            String::new()
        };

        // Get relative paths for database files
        let db_path = self.path_manager.memory_db_path(false);
        let db_pattern = format!("{}", db_path.display());

        // Add .kiro database exclusions if not present
        if !content.contains(&db_pattern) {
            content.push_str("\n# Hail-Mary database\n");
            content.push_str(&format!("{}\n", db_pattern));

            // Add pattern for WAL and SHM files
            let memory_dir = self.path_manager.memory_dir(false);
            content.push_str(&format!("{}/*.sqlite3-*\n", memory_dir.display()));
        }

        fs::write(gitignore_path, content).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to write .gitignore: {}", e))
        })?;

        Ok(())
    }

    fn create_feature(&self, name: &str) -> Result<(), ApplicationError> {
        // Validate feature name (kebab-case)
        if !name
            .chars()
            .all(|c| c.is_lowercase() || c == '-' || c.is_numeric())
            || name.starts_with('-')
            || name.ends_with('-')
            || name.contains("--")
        {
            return Err(ApplicationError::InvalidFeatureName(name.to_string()));
        }

        // Generate directory name with date prefix
        let date = chrono::Utc::now().format("%Y-%m-%d");
        let feature_name = format!("{}-{}", date, name);
        let feature_dir = self.path_manager.specs_dir(true).join(&feature_name);

        // Check if feature already exists
        if feature_dir.exists() {
            return Err(ApplicationError::FeatureAlreadyExists(name.to_string()));
        }

        // Create feature directory
        fs::create_dir_all(&feature_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to create feature directory: {}", e))
        })?;

        // Create template files
        let requirements_content = format!(
            r#"# Requirements

## Overview
[Feature description for {}]

## User Stories
- As a [user type], I want to [action] so that [benefit]

## Acceptance Criteria
- [ ] Criterion 1
- [ ] Criterion 2

## Technical Requirements
- Database changes
- API endpoints
- UI components
"#,
            name
        );

        let design_content = format!(
            r#"# Design

## Architecture
[Architecture overview for {}]

## Components
[Component descriptions]

## Data Flow
[Data flow diagrams]
"#,
            name
        );

        let task_content = r#"# Tasks

## Implementation Tasks
- [ ] Task 1
- [ ] Task 2
- [ ] Task 3

## Testing Tasks
- [ ] Unit tests
- [ ] Integration tests
- [ ] E2E tests
"#;

        // Write template files
        fs::write(feature_dir.join("requirements.md"), requirements_content).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to write requirements.md: {}", e))
        })?;

        fs::write(feature_dir.join("design.md"), design_content).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to write design.md: {}", e))
        })?;

        fs::write(feature_dir.join("tasks.md"), task_content).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to write tasks.md: {}", e))
        })?;

        fs::write(feature_dir.join("spec.json"), "{}").map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to write spec.json: {}", e))
        })?;

        Ok(())
    }

    fn save_document(
        &self,
        memory_type: &str,
        memories: &[Memory],
    ) -> Result<(), ApplicationError> {
        let memory_dir = self.path_manager.memory_dir(true);
        let doc_path = memory_dir.join(format!("{}.md", memory_type));

        // Generate markdown content
        let mut content = format!("# {} Memories\n\n", memory_type);

        for memory in memories {
            content.push_str(&format!("## {}\n", memory.title));
            content.push_str(&format!("**ID**: {}\n", memory.id));
            content.push_str(&format!("**Tags**: {}\n", memory.tags.join(", ")));
            content.push_str(&format!(
                "**Confidence**: {:.2}\n",
                memory.confidence.value()
            ));
            content.push_str(&format!("**References**: {}\n", memory.reference_count));
            content.push('\n');
            content.push_str(&memory.content);
            content.push_str("\n\n---\n\n");
        }

        fs::write(doc_path, content).map_err(|e| {
            ApplicationError::DocumentGenerationError(format!(
                "Failed to write document for {}: {}",
                memory_type, e
            ))
        })?;

        Ok(())
    }

    fn list_spec_directories(&self) -> Result<Vec<(String, bool)>, ApplicationError> {
        let specs_dir = self.path_manager.specs_dir(true);
        let mut specs = Vec::new();

        if !specs_dir.exists() {
            return Ok(specs);
        }

        let entries = fs::read_dir(specs_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to read specs directory: {}", e))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                ApplicationError::FileSystemError(format!("Failed to read directory entry: {}", e))
            })?;

            let file_type = entry.file_type().map_err(|e| {
                ApplicationError::FileSystemError(format!("Failed to get file type: {}", e))
            })?;

            if file_type.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                // For now, all specs in the specs directory are considered not archived
                specs.push((name, false));
            }
        }

        // Sort by name in reverse order (newer dates first)
        specs.sort_by(|a, b| b.0.cmp(&a.0));
        Ok(specs)
    }

    fn mark_spec_complete(&self, name: &str) -> Result<(), ApplicationError> {
        let source_path = self.path_manager.specs_dir(true).join(name);

        if !source_path.exists() {
            return Err(ApplicationError::SpecNotFound(name.to_string()));
        }

        if !source_path.is_dir() {
            return Err(ApplicationError::InvalidSpecDirectory(name.to_string()));
        }

        // Create archive directory
        let archive_dir = self.path_manager.archive_dir(true);
        fs::create_dir_all(&archive_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to create archive directory: {}", e))
        })?;

        let dest_path = archive_dir.join(name);

        // Check if already exists in archive
        if dest_path.exists() {
            return Err(ApplicationError::ArchiveAlreadyExists(name.to_string()));
        }

        // Move directory to archive
        fs::rename(&source_path, &dest_path).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to move spec to archive: {}", e))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_helpers::TestDirectory;

    fn create_test_repository() -> (ProjectRepository, TestDirectory) {
        let test_dir = TestDirectory::new_no_cd();
        let path_manager = PathManager::new(test_dir.path().to_path_buf());
        let repository = ProjectRepository::new(path_manager);
        (repository, test_dir)
    }

    #[test]
    fn test_initialize_creates_directory_structure() {
        let (repository, _test_dir) = create_test_repository();

        let result = repository.initialize();
        assert!(result.is_ok());

        // Verify directories were created
        assert!(repository.path_manager.kiro_dir(true).exists());
        assert!(repository.path_manager.memory_dir(true).exists());
        assert!(repository.path_manager.specs_dir(true).exists());
    }

    #[test]
    fn test_exists_returns_true_when_initialized() {
        let (repository, _test_dir) = create_test_repository();

        // Initially should not exist
        let exists = repository.exists().unwrap();
        assert!(!exists);

        // After initialization should exist
        repository.initialize().unwrap();
        let exists = repository.exists().unwrap();
        assert!(exists);
    }

    #[test]
    fn test_save_and_load_config_roundtrip() {
        let (repository, _test_dir) = create_test_repository();

        // Initialize to create directories
        repository.initialize().unwrap();

        // Create custom config
        let original_config = ProjectConfig {
            memory_types: vec!["custom".to_string(), "test".to_string()],
            instructions: "Custom instructions for testing".to_string(),
            document_format: DocumentFormat::Markdown,
        };

        // Save config
        let save_result = repository.save_config(&original_config);
        assert!(save_result.is_ok());

        // Load config
        let loaded_config = repository.load_config().unwrap();

        // Verify roundtrip
        assert_eq!(loaded_config.memory_types, original_config.memory_types);
        assert_eq!(loaded_config.instructions, original_config.instructions);
        assert_eq!(
            loaded_config.document_format,
            original_config.document_format
        );
    }

    #[test]
    fn test_load_config_returns_default_when_missing() {
        let (repository, _temp_dir) = create_test_repository();

        let config = repository.load_config().unwrap();

        // Should return default config
        assert_eq!(config.memory_types.len(), 5);
        assert!(config.memory_types.contains(&"tech".to_string()));
        assert!(config.memory_types.contains(&"project-tech".to_string()));
        assert!(config.memory_types.contains(&"domain".to_string()));
        assert!(config.memory_types.contains(&"workflow".to_string()));
        assert!(config.memory_types.contains(&"decision".to_string()));
    }

    #[test]
    fn test_update_gitignore_adds_database_patterns() {
        let (repository, _temp_dir) = create_test_repository();

        let result = repository.update_gitignore();
        assert!(result.is_ok());

        // Verify gitignore was created and contains patterns
        let gitignore_path = repository.path_manager.project_root().join(".gitignore");
        assert!(gitignore_path.exists());

        let content = fs::read_to_string(gitignore_path).unwrap();
        assert!(content.contains("# Hail-Mary database"));
        assert!(content.contains(".kiro/memory/db.sqlite3"));
        assert!(content.contains(".kiro/memory/*.sqlite3-*"));
    }

    #[test]
    fn test_update_gitignore_does_not_duplicate_patterns() {
        let (repository, _temp_dir) = create_test_repository();

        // Run twice
        repository.update_gitignore().unwrap();
        repository.update_gitignore().unwrap();

        let gitignore_path = repository.path_manager.project_root().join(".gitignore");
        let content = fs::read_to_string(gitignore_path).unwrap();

        // Should only appear once
        let pattern_count = content.matches("# Hail-Mary database").count();
        assert_eq!(pattern_count, 1);
    }

    #[test]
    fn test_create_feature_with_date_prefix() {
        let (repository, _temp_dir) = create_test_repository();

        repository.initialize().unwrap();

        let result = repository.create_feature("user-authentication");
        assert!(result.is_ok());

        // Verify feature directory was created with date prefix
        let specs_dir = repository.path_manager.specs_dir(true);
        let entries: Vec<_> = fs::read_dir(specs_dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();

        assert_eq!(entries.len(), 1);
        let feature_dir_name = &entries[0];
        assert!(feature_dir_name.ends_with("-user-authentication"));
        assert!(feature_dir_name.starts_with(&chrono::Utc::now().format("%Y-%m-%d").to_string()));

        // Verify template files were created
        let feature_path = repository
            .path_manager
            .specs_dir(true)
            .join(feature_dir_name);
        assert!(feature_path.join("requirements.md").exists());
        assert!(feature_path.join("design.md").exists());
        assert!(feature_path.join("tasks.md").exists());
        assert!(feature_path.join("spec.json").exists());

        // Verify content contains feature name
        let requirements_content =
            fs::read_to_string(feature_path.join("requirements.md")).unwrap();
        assert!(requirements_content.contains("user-authentication"));
    }

    #[test]
    fn test_invalid_feature_names() {
        let (repository, _temp_dir) = create_test_repository();

        repository.initialize().unwrap();

        let invalid_names = vec![
            "-invalid-start",
            "invalid-end-",
            "invalid--double",
            "InvalidCase",
            "invalid_underscore",
            "invalid.dot",
        ];

        for name in invalid_names {
            let result = repository.create_feature(name);
            assert!(result.is_err());
            match result.unwrap_err() {
                ApplicationError::InvalidFeatureName(_) => {}
                _ => panic!("Expected InvalidFeatureName for '{}'", name),
            }
        }
    }

    #[test]
    fn test_create_feature_duplicate_names() {
        let (repository, _temp_dir) = create_test_repository();

        repository.initialize().unwrap();

        // Create feature first time
        let result1 = repository.create_feature("test-feature");
        assert!(result1.is_ok());

        // Try to create same feature again
        let result2 = repository.create_feature("test-feature");
        assert!(result2.is_err());
        match result2.unwrap_err() {
            ApplicationError::FeatureAlreadyExists(_) => {}
            _ => panic!("Expected FeatureAlreadyExists"),
        }
    }

    #[test]
    fn test_save_document_generates_markdown() {
        let (repository, _temp_dir) = create_test_repository();

        repository.initialize().unwrap();

        // Create test memories
        let memory1 = Memory::new(
            "tech".to_string(),
            "Test Memory 1".to_string(),
            "This is test content 1".to_string(),
        )
        .with_tags(vec!["rust".to_string(), "testing".to_string()]);

        let memory2 = Memory::new(
            "tech".to_string(),
            "Test Memory 2".to_string(),
            "This is test content 2".to_string(),
        )
        .with_tags(vec!["programming".to_string()]);

        let memories = vec![memory1, memory2];

        let result = repository.save_document("tech", &memories);
        assert!(result.is_ok());

        // Verify file was created
        let doc_path = repository.path_manager.memory_dir(true).join("tech.md");
        assert!(doc_path.exists());

        // Verify content
        let content = fs::read_to_string(doc_path).unwrap();
        assert!(content.contains("# tech Memories"));
        assert!(content.contains("## Test Memory 1"));
        assert!(content.contains("## Test Memory 2"));
        assert!(content.contains("This is test content 1"));
        assert!(content.contains("This is test content 2"));
        assert!(content.contains("**Tags**: rust, testing"));
        assert!(content.contains("**Tags**: programming"));
        assert!(content.contains("**Confidence**:"));
        assert!(content.contains("**References**:"));
    }

    #[test]
    fn test_error_handling_scenarios() {
        let test_dir = TestDirectory::new_no_cd();

        // Test with invalid path (read-only)
        let readonly_path = test_dir.path().join("readonly");
        fs::create_dir(&readonly_path).unwrap();

        // Make directory read-only on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&readonly_path).unwrap().permissions();
            perms.set_mode(0o444); // read-only
            fs::set_permissions(&readonly_path, perms).unwrap();
        }

        let path_manager = PathManager::new(readonly_path);
        let repository = ProjectRepository::new(path_manager);

        // This should fail on Unix systems
        #[cfg(unix)]
        {
            let result = repository.initialize();
            assert!(result.is_err());
        }

        // Reset permissions for cleanup
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(test_dir.path().join("readonly"))
                .unwrap()
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(test_dir.path().join("readonly"), perms).unwrap();
        }
    }

    #[test]
    fn test_config_serialization_format() {
        let (repository, _temp_dir) = create_test_repository();

        repository.initialize().unwrap();

        let config = ProjectConfig::default_for_new_project();
        repository.save_config(&config).unwrap();

        // Verify TOML format
        let config_path = repository.path_manager.config_path(true);
        let content = fs::read_to_string(config_path).unwrap();

        // Should contain TOML structure
        assert!(content.contains("[memory]"));
        assert!(content.contains("types ="));
        assert!(content.contains("instructions ="));
        assert!(content.contains("[memory.document]"));
        assert!(content.contains("[memory.database]"));
        assert!(content.contains("output_dir ="));
        assert!(content.contains("path ="));
    }
}
