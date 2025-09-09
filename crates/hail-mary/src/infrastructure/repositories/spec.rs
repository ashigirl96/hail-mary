use crate::application::errors::ApplicationError;
use crate::application::repositories::SpecRepository;
use crate::infrastructure::filesystem::path_manager::PathManager;
use std::fs;
use std::path::Path;

pub struct SpecRepositoryImpl {
    path_manager: PathManager,
}

impl SpecRepositoryImpl {
    pub fn new(path_manager: PathManager) -> Self {
        Self { path_manager }
    }

    fn validate_feature_name(&self, name: &str) -> Result<(), ApplicationError> {
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
        Ok(())
    }

    fn create_template_files(
        &self,
        feature_dir: &Path,
        name: &str,
    ) -> Result<(), ApplicationError> {
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
}

impl SpecRepository for SpecRepositoryImpl {
    fn create_feature(&self, name: &str) -> Result<(), ApplicationError> {
        // Validate feature name
        self.validate_feature_name(name)?;

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
        self.create_template_files(&feature_dir, name)?;

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

    fn list_archived_specs(&self) -> Result<Vec<String>, ApplicationError> {
        let archive_dir = self.path_manager.archive_dir(true);
        let mut specs = Vec::new();

        if !archive_dir.exists() {
            return Ok(specs);
        }

        let entries = fs::read_dir(archive_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to read archive directory: {}", e))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                ApplicationError::FileSystemError(format!("Failed to read directory entry: {}", e))
            })?;

            let file_type = entry.file_type().map_err(|e| {
                ApplicationError::FileSystemError(format!("Failed to get file type: {}", e))
            })?;

            if file_type.is_dir() {
                specs.push(entry.file_name().to_string_lossy().to_string());
            }
        }

        // Sort by name in reverse order (newer dates first)
        specs.sort_by(|a, b| b.cmp(a));
        Ok(specs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_helpers::TestDirectory;

    fn create_test_repository() -> (SpecRepositoryImpl, TestDirectory) {
        let test_dir = TestDirectory::new_no_cd();
        let path_manager = PathManager::new(test_dir.path().to_path_buf());
        let repository = SpecRepositoryImpl::new(path_manager);
        (repository, test_dir)
    }

    #[test]
    fn test_create_feature_with_valid_name() {
        let (repository, test_dir) = create_test_repository();

        // Create specs directory
        fs::create_dir_all(test_dir.path().join(".kiro/specs")).unwrap();

        let result = repository.create_feature("test-feature");
        assert!(result.is_ok());

        // Verify feature directory was created
        let specs_dir = test_dir.path().join(".kiro/specs");
        let entries: Vec<_> = fs::read_dir(specs_dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();

        assert_eq!(entries.len(), 1);
        assert!(entries[0].ends_with("-test-feature"));
    }

    #[test]
    fn test_create_feature_with_invalid_name() {
        let (repository, test_dir) = create_test_repository();

        // Create specs directory
        fs::create_dir_all(test_dir.path().join(".kiro/specs")).unwrap();

        let invalid_names = vec![
            "-invalid-start",
            "invalid-end-",
            "invalid--double",
            "InvalidCase",
            "invalid_underscore",
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
    fn test_list_spec_directories() {
        let (repository, test_dir) = create_test_repository();

        // Create specs directory with some specs
        let specs_dir = test_dir.path().join(".kiro/specs");
        fs::create_dir_all(&specs_dir).unwrap();
        fs::create_dir(specs_dir.join("2025-01-01-feature1")).unwrap();
        fs::create_dir(specs_dir.join("2025-01-02-feature2")).unwrap();

        let specs = repository.list_spec_directories().unwrap();

        assert_eq!(specs.len(), 2);
        // Should be sorted in reverse order (newer first)
        assert_eq!(specs[0].0, "2025-01-02-feature2");
        assert_eq!(specs[1].0, "2025-01-01-feature1");
        // All should be not archived
        assert!(!specs[0].1);
        assert!(!specs[1].1);
    }

    #[test]
    fn test_mark_spec_complete() {
        let (repository, test_dir) = create_test_repository();

        // Create specs and archive directories
        let specs_dir = test_dir.path().join(".kiro/specs");
        let archive_dir = test_dir.path().join(".kiro/archive");
        fs::create_dir_all(&specs_dir).unwrap();
        fs::create_dir_all(&archive_dir).unwrap();

        // Create a spec
        let spec_name = "2025-01-01-feature";
        fs::create_dir(specs_dir.join(spec_name)).unwrap();

        // Mark as complete
        let result = repository.mark_spec_complete(spec_name);
        assert!(result.is_ok());

        // Verify moved to archive
        assert!(!specs_dir.join(spec_name).exists());
        assert!(archive_dir.join(spec_name).exists());
    }

    #[test]
    fn test_get_spec_path() {
        let (repository, test_dir) = create_test_repository();

        // Create specs directory with a spec
        let specs_dir = test_dir.path().join(".kiro/specs");
        fs::create_dir_all(&specs_dir).unwrap();
        let spec_name = "2025-01-01-feature";
        fs::create_dir(specs_dir.join(spec_name)).unwrap();

        // Get spec path
        let path = repository.get_spec_path(spec_name).unwrap();
        assert_eq!(path, specs_dir.join(spec_name));

        // Try non-existent spec
        let result = repository.get_spec_path("non-existent");
        assert!(result.is_err());
        match result.unwrap_err() {
            ApplicationError::SpecNotFound(_) => {}
            _ => panic!("Expected SpecNotFound"),
        }
    }
}
