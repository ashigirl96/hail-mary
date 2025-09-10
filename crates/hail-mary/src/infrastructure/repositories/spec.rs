use crate::application::errors::ApplicationError;
use crate::application::repositories::SpecRepositoryInterface;
use crate::infrastructure::filesystem::path_manager::PathManager;
use std::fs;
use std::path::PathBuf;

pub struct SpecRepository {
    path_manager: PathManager,
}

impl SpecRepository {
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
        feature_dir: &std::path::Path,
        name: &str,
    ) -> Result<(), ApplicationError> {
        // Create template files
        let requirements_content = format!(
            r#"# Requirements

## Overview
[Feature description for {}]

## Purpose
[Why are we building this feature?]

## User Stories
[List of user stories]

## Acceptance Criteria
[Define clear criteria for feature completion]

## Technical Requirements
[Technical details and constraints]

## Priority

P0: [Must have]
P1: [Should have]
P2: [Could have]
P3: [Won't have this time]

## Risk and Mitigation
[Potential risks and how to mitigate them]
"#,
            name
        );

        let design_content = format!(
            r#"# Design

## Overview
[High-level architecture for {}]

"#,
            name
        );

        let task_content = format!(
            r#"# Tasks: {}
## Overview
[Overview of tasks]

## Phase 1: [Description]
[List of tasks]
"#,
            name
        );

        let memo_content = format!(
            r#"# Memo: {}


"#,
            name
        );

        let investigation_content = format!(
            r#"# Investigation

## Research for {}

## Current State Analysis
[What exists today]

## Problem Space
[What problems are we solving]

## Existing Solutions
### Option 1
- Pros:
- Cons:

### Option 2
- Pros:
- Cons:

## Technical Research
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

impl SpecRepositoryInterface for SpecRepository {
    fn create_feature(&self, name: &str) -> Result<(), ApplicationError> {
        // Validate feature name
        self.validate_feature_name(name)?;

        // Ensure specs directory exists (like ConfigRepository pattern)
        let specs_dir = self.path_manager.specs_dir(true);
        fs::create_dir_all(&specs_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to create specs directory: {}", e))
        })?;

        // Generate directory name with date prefix
        let date = chrono::Utc::now().format("%Y-%m-%d");
        let feature_name = format!("{}-{}", date, name);
        let feature_dir = specs_dir.join(&feature_name);

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

    fn get_spec_path(&self, name: &str) -> Result<PathBuf, ApplicationError> {
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
                let name = entry.file_name().to_string_lossy().to_string();
                specs.push(name);
            }
        }

        // Sort by name in reverse order (newer dates first)
        specs.sort_by(|a, b| b.cmp(a));
        Ok(specs)
    }
}
