use crate::application::errors::ApplicationError;
use crate::application::repositories::ProjectRepository as ProjectRepositoryTrait;
use crate::domain::entities::project::ProjectConfig;
use crate::domain::entities::steering::SteeringConfig;
use crate::infrastructure::filesystem::path_manager::PathManager;
use serde::{Deserialize, Serialize};
use std::fs;

// Type-safe TOML configuration structures
#[derive(Debug, Serialize, Deserialize)]
struct TomlConfig {
    steering: Option<SteeringSection>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SteeringTomlConfig {
    steering: SteeringSection,
}

#[derive(Debug, Serialize, Deserialize)]
struct SteeringSection {
    types: Vec<SteeringTypeToml>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SteeringTypeToml {
    name: String,
    purpose: String,
    criteria: Vec<String>,
}

pub struct ProjectRepository {
    path_manager: PathManager,
}

impl ProjectRepository {
    pub fn new(path_manager: PathManager) -> Self {
        Self { path_manager }
    }

    fn steering_dir(&self) -> std::path::PathBuf {
        self.path_manager.kiro_dir(true).join("steering")
    }

    fn draft_dir(&self) -> std::path::PathBuf {
        self.steering_dir().join("draft")
    }
}

/// Helper function to generate steering section TOML
fn generate_steering_section(config: &SteeringConfig) -> Result<String, ApplicationError> {
    let steering_toml = SteeringTomlConfig {
        steering: SteeringSection {
            types: config
                .types
                .iter()
                .map(|steering_type| SteeringTypeToml {
                    name: steering_type.name.clone(),
                    purpose: steering_type.purpose.clone(),
                    criteria: steering_type
                        .criteria
                        .iter()
                        .map(|c| format!("{}: {}", c.name, c.description))
                        .collect(),
                })
                .collect(),
        },
    };

    toml::to_string_pretty(&steering_toml).map_err(|e| {
        ApplicationError::ConfigurationError(format!("Failed to serialize steering section: {}", e))
    })
}

impl ProjectRepositoryTrait for ProjectRepository {
    fn initialize(&self) -> Result<(), ApplicationError> {
        // Create .kiro directory structure
        let kiro_dir = self.path_manager.kiro_dir(true);
        fs::create_dir_all(&kiro_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to create .kiro directory: {}", e))
        })?;

        // Create subdirectories
        let specs_dir = self.path_manager.specs_dir(true);
        fs::create_dir_all(&specs_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to create specs directory: {}", e))
        })?;

        Ok(())
    }

    fn exists(&self) -> Result<bool, ApplicationError> {
        Ok(self.path_manager.kiro_dir(true).exists())
    }

    fn save_config(&self) -> Result<(), ApplicationError> {
        let config_path = self.path_manager.config_path(true);

        // Never overwrite existing config.toml (even with --force)
        if config_path.exists() {
            return Ok(());
        }

        // Create type-safe TOML structure
        let steering_config = SteeringConfig::default_for_new_project();
        let toml_config = TomlConfig {
            steering: Some(SteeringSection {
                types: steering_config
                    .types
                    .iter()
                    .map(|steering_type| SteeringTypeToml {
                        name: steering_type.name.clone(),
                        purpose: steering_type.purpose.clone(),
                        criteria: steering_type
                            .criteria
                            .iter()
                            .map(|c| format!("{}: {}", c.name, c.description))
                            .collect(),
                    })
                    .collect(),
            }),
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
        // Return default config for steering system
        // TODO: Update to load only steering configuration
        Ok(ProjectConfig::default_for_new_project())
    }

    fn update_gitignore(&self) -> Result<(), ApplicationError> {
        let gitignore_path = self.path_manager.project_root().join(".gitignore");

        let content = if gitignore_path.exists() {
            fs::read_to_string(&gitignore_path).map_err(|e| {
                ApplicationError::FileSystemError(format!("Failed to read .gitignore: {}", e))
            })?
        } else {
            String::new()
        };

        // No database files to exclude since we're using file-based steering system

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

## Data Flow
[Data flow diagrams]

## ...
"#,
            name
        );

        let task_content = r#"# Tasks

## References
...

## Phase 1: ...
- [ ] Task 1
- [ ] Task 2
- [ ] Task 3
"#;

        let memo_content = format!(
            r#"# Memo: {}

"#,
            name
        );

        let investigation_content = format!(
            r#"# Investigation: {}

## Research Notes
[Research findings and exploration notes]

## Key Findings
- Finding 1
- Finding 2

## Technical Considerations
[Technical details discovered during investigation]

## Questions & Uncertainties
- [ ] Question 1
- [ ] Question 2

## Resources & References
- Resource 1
- Resource 2
"#,
            name
        );

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

        fs::write(feature_dir.join("memo.md"), memo_content).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to write memo.md: {}", e))
        })?;

        fs::write(feature_dir.join("investigation.md"), investigation_content).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to write investigation.md: {}", e))
        })?;

        fs::write(feature_dir.join("spec.json"), "{}").map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to write spec.json: {}", e))
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

        // If already exists in archive, remove it first to allow overwriting
        if dest_path.exists() {
            fs::remove_dir_all(&dest_path).map_err(|e| {
                ApplicationError::FileSystemError(format!(
                    "Failed to remove existing archive: {}",
                    e
                ))
            })?;
        }

        // Move directory to archive
        fs::rename(&source_path, &dest_path).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to move spec to archive: {}", e))
        })?;

        Ok(())
    }

    fn get_spec_path(&self, name: &str) -> Result<std::path::PathBuf, ApplicationError> {
        let specs_dir = self.path_manager.specs_dir(true);
        let spec_path = specs_dir.join(name);

        if !spec_path.exists() {
            return Err(ApplicationError::SpecNotFound(name.to_string()));
        }

        if !spec_path.is_dir() {
            return Err(ApplicationError::InvalidSpecDirectory(name.to_string()));
        }

        Ok(spec_path)
    }

    fn initialize_steering(&self) -> Result<(), ApplicationError> {
        // Create .kiro/steering directory
        let steering_dir = self.steering_dir();
        fs::create_dir_all(&steering_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to create steering directory: {}", e))
        })?;

        // Create .kiro/steering/draft directory
        let draft_dir = self.draft_dir();
        fs::create_dir_all(&draft_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to create draft directory: {}", e))
        })?;

        Ok(())
    }

    fn create_steering_files(&self, config: &SteeringConfig) -> Result<(), ApplicationError> {
        for steering_type in &config.types {
            let file_path = self
                .steering_dir()
                .join(format!("{}.md", steering_type.name));

            // Never overwrite existing files
            if file_path.exists() {
                continue;
            }

            // Generate simple markdown header only
            let content = format!("# {}\n\n", steering_type.name);

            fs::write(file_path, content).map_err(|e| {
                ApplicationError::FileSystemError(format!(
                    "Failed to write steering file {}: {}",
                    steering_type.name, e
                ))
            })?;
        }
        Ok(())
    }

    fn ensure_steering_config(&self) -> Result<(), ApplicationError> {
        let config_path = self.path_manager.config_path(true);

        if !config_path.exists() {
            // Create new config with steering section
            self.save_config()?;
            return Ok(());
        }

        // Read existing config content
        let existing_content = fs::read_to_string(&config_path).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to read existing config: {}", e))
        })?;

        // Check if steering section already exists
        if existing_content.contains("[steering]")
            || existing_content.contains("[[steering.types]]")
        {
            // Steering section already exists, nothing to do
            return Ok(());
        }

        // Generate steering section to append
        let steering_config = SteeringConfig::default_for_new_project();
        let steering_section = generate_steering_section(&steering_config)?;

        // Append steering section to existing content
        let new_content = format!("{}\n{}", existing_content.trim(), steering_section);

        fs::write(&config_path, new_content).map_err(|e| {
            ApplicationError::FileSystemError(format!(
                "Failed to update config with steering section: {}",
                e
            ))
        })?;

        Ok(())
    }

    fn deploy_slash_commands(&self) -> Result<(), ApplicationError> {
        use crate::infrastructure::embedded_resources::EmbeddedSlashCommands;

        // Create .claude/commands/hm directory
        let claude_dir = self.path_manager.project_root().join(".claude");
        let commands_dir = claude_dir.join("commands");
        let hm_dir = commands_dir.join("hm");

        // Create directory structure
        fs::create_dir_all(&hm_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!(
                "Failed to create .claude/commands/hm directory: {}",
                e
            ))
        })?;

        // Deploy all embedded markdown files (force overwrite)
        for (filename, content) in EmbeddedSlashCommands::get_all() {
            let file_path = hm_dir.join(filename);
            fs::write(&file_path, content).map_err(|e| {
                ApplicationError::FileSystemError(format!(
                    "Failed to write slash command file {}: {}",
                    filename, e
                ))
            })?;
        }

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
        // Memory directory no longer created
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

        // Save config (always saves default)
        let save_result = repository.save_config();
        assert!(save_result.is_ok());

        // Load config
        let loaded_config = repository.load_config().unwrap();

        // Verify it returns default config
        let default_config = ProjectConfig::default_for_new_project();
        assert_eq!(loaded_config.instructions, default_config.instructions);
        assert_eq!(
            loaded_config.document_format,
            default_config.document_format
        );
    }

    #[test]
    fn test_load_config_returns_default_when_missing() {
        let (repository, _temp_dir) = create_test_repository();

        let config = repository.load_config().unwrap();

        // Should return default config
        assert!(!config.instructions.is_empty());
    }

    #[test]
    fn test_update_gitignore_creates_file() {
        let (repository, _temp_dir) = create_test_repository();

        let result = repository.update_gitignore();
        assert!(result.is_ok());

        // Verify gitignore was created
        let gitignore_path = repository.path_manager.project_root().join(".gitignore");
        assert!(gitignore_path.exists());

        // With file-based steering system, no database patterns are added
        let content = fs::read_to_string(gitignore_path).unwrap();
        // File should exist but be empty or minimal
        assert!(content.is_empty() || content.trim().is_empty());
    }

    #[test]
    fn test_update_gitignore_does_not_duplicate_patterns() {
        let (repository, _temp_dir) = create_test_repository();

        // Run twice
        repository.update_gitignore().unwrap();
        repository.update_gitignore().unwrap();

        let gitignore_path = repository.path_manager.project_root().join(".gitignore");
        let content = fs::read_to_string(gitignore_path).unwrap();

        // No database patterns with file-based steering system
        let pattern_count = content.matches("# Hail-Mary database").count();
        assert_eq!(pattern_count, 0);
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
        assert!(feature_path.join("memo.md").exists());
        assert!(feature_path.join("investigation.md").exists());
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

        // Create directory first
        fs::create_dir_all(repository.path_manager.kiro_dir(true)).unwrap();

        // Ensure config doesn't exist yet
        let config_path = repository.path_manager.config_path(true);
        if config_path.exists() {
            fs::remove_file(&config_path).unwrap();
        }

        repository.save_config().unwrap();

        // Verify TOML format
        let content = fs::read_to_string(&config_path).unwrap();

        // Should contain TOML structure for steering
        assert!(content.contains("[[steering.types]]"));
        assert!(content.contains("name ="));
        assert!(content.contains("purpose ="));
    }

    #[test]
    fn test_deploy_slash_commands() {
        let (repository, _temp_dir) = create_test_repository();

        // Deploy slash commands
        let result = repository.deploy_slash_commands();
        assert!(result.is_ok());

        // Verify files were created
        let hm_dir = repository
            .path_manager
            .project_root()
            .join(".claude/commands/hm");
        assert!(
            hm_dir.exists(),
            ".claude/commands/hm directory should exist"
        );

        // Check that all expected files exist
        let expected_files = ["steering-remember.md", "steering.md", "steering-merge.md"];
        for file in &expected_files {
            let file_path = hm_dir.join(file);
            assert!(file_path.exists(), "File {} should exist", file);

            // Verify content is not empty
            let content = fs::read_to_string(&file_path).unwrap();
            assert!(!content.is_empty(), "File {} should not be empty", file);
        }
    }

    #[test]
    fn test_deploy_slash_commands_overwrites_existing() {
        let (repository, _temp_dir) = create_test_repository();

        // Create .claude/commands/hm directory with a test file
        let hm_dir = repository
            .path_manager
            .project_root()
            .join(".claude/commands/hm");
        fs::create_dir_all(&hm_dir).unwrap();

        // Write a test file that should be overwritten
        let test_file = hm_dir.join("steering.md");
        fs::write(&test_file, "OLD CONTENT").unwrap();

        // Deploy slash commands
        let result = repository.deploy_slash_commands();
        assert!(result.is_ok());

        // Verify the file was overwritten
        let content = fs::read_to_string(&test_file).unwrap();
        assert!(
            !content.contains("OLD CONTENT"),
            "File should be overwritten"
        );
        assert!(
            content.contains("Kiro Steering Management"),
            "File should contain new content"
        );
    }
}
