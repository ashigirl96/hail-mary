use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};

use crate::utils::error::{HailMaryError, Result};
use crate::utils::validator::validate_kebab_case;

pub struct ProjectManager {
    base_path: PathBuf,
}

impl Default for ProjectManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectManager {
    pub fn new() -> Self {
        Self {
            base_path: PathBuf::from(".kiro/specs"),
        }
    }

    /// Create a new feature specification directory and files
    pub fn create_new_feature(&self, feature_name: &str) -> Result<PathBuf> {
        // Validate feature name
        validate_kebab_case(feature_name)?;

        // Generate directory name with current date
        let dir_name = self.generate_feature_dir_name(feature_name);
        let feature_path = self.base_path.join(&dir_name);

        // Check if feature already exists
        if self.feature_exists(feature_name) {
            return Err(HailMaryError::FeatureAlreadyExists(
                feature_name.to_string(),
            ));
        }

        // Create .kiro/specs directory if it doesn't exist
        self.ensure_base_directory()?;

        // Create feature directory
        fs::create_dir_all(&feature_path).map_err(HailMaryError::Io)?;

        // Create required files
        self.create_spec_files(&feature_path)?;

        Ok(feature_path)
    }

    /// Generate directory name in YYYY-MM-dd-[feature-name] format
    fn generate_feature_dir_name(&self, feature_name: &str) -> String {
        let today = Utc::now().format("%Y-%m-%d");
        format!("{}-{}", today, feature_name)
    }

    /// Ensure the base .kiro/specs directory exists
    fn ensure_base_directory(&self) -> Result<()> {
        if !self.base_path.exists() {
            fs::create_dir_all(&self.base_path).map_err(HailMaryError::Io)?;
        }
        Ok(())
    }

    /// Create the required specification files
    fn create_spec_files(&self, feature_path: &Path) -> Result<()> {
        let files = [
            ("requirements.md", ""),
            ("design.md", ""),
            ("task.md", ""),
            ("spec.json", "{}"),
        ];

        for (filename, content) in &files {
            let file_path = feature_path.join(filename);
            fs::write(&file_path, content).map_err(HailMaryError::Io)?;
        }

        Ok(())
    }

    /// Check if a feature already exists
    pub fn feature_exists(&self, feature_name: &str) -> bool {
        if let Ok(features) = self.list_features() {
            features.contains(&feature_name.to_string())
        } else {
            false
        }
    }

    /// List existing features
    pub fn list_features(&self) -> Result<Vec<String>> {
        if !self.base_path.exists() {
            return Ok(vec![]);
        }

        let mut features = Vec::new();

        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir()
                && let Some(name) = entry.file_name().to_str()
            {
                // Extract feature name from YYYY-MM-dd-[feature-name] format
                if let Some(feature_name) = self.extract_feature_name(name) {
                    features.push(feature_name);
                }
            }
        }

        features.sort();
        Ok(features)
    }

    /// Extract feature name from directory name (YYYY-MM-dd-[feature-name])
    fn extract_feature_name(&self, dir_name: &str) -> Option<String> {
        // Split by '-' and take everything after the date part (first 3 parts)
        let parts: Vec<&str> = dir_name.split('-').collect();
        if parts.len() >= 4 {
            // Rejoin the feature name parts
            Some(parts[3..].join("-"))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_project() -> (ProjectManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let mut project = ProjectManager::new();
        project.base_path = temp_dir.path().join(".kiro/specs");
        (project, temp_dir)
    }

    #[test]
    fn test_generate_feature_dir_name() {
        let project = ProjectManager::new();
        let dir_name = project.generate_feature_dir_name("my-feature");

        // Should match YYYY-MM-dd-my-feature pattern
        assert!(dir_name.ends_with("-my-feature"));
        assert!(dir_name.len() >= 21); // "2024-01-01-my-feature" = 21 chars
    }

    #[test]
    fn test_create_new_feature_success() {
        let (project, _temp_dir) = setup_test_project();

        let result = project.create_new_feature("test-feature");
        assert!(result.is_ok());

        let feature_path = result.unwrap();
        assert!(feature_path.exists());
        assert!(feature_path.join("requirements.md").exists());
        assert!(feature_path.join("design.md").exists());
        assert!(feature_path.join("task.md").exists());
        assert!(feature_path.join("spec.json").exists());
    }

    #[test]
    fn test_create_duplicate_feature() {
        let (project, _temp_dir) = setup_test_project();

        // Create first feature
        assert!(project.create_new_feature("test-feature").is_ok());

        // Try to create duplicate
        let result = project.create_new_feature("test-feature");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HailMaryError::FeatureAlreadyExists(_)
        ));
    }

    #[test]
    fn test_invalid_feature_name() {
        let (project, _temp_dir) = setup_test_project();

        let result = project.create_new_feature("Invalid_Name");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HailMaryError::InvalidFeatureName(_)
        ));
    }

    #[test]
    fn test_extract_feature_name() {
        let project = ProjectManager::new();

        assert_eq!(
            project.extract_feature_name("2024-01-01-my-feature"),
            Some("my-feature".to_string())
        );

        assert_eq!(
            project.extract_feature_name("2024-12-25-complex-feature-name"),
            Some("complex-feature-name".to_string())
        );

        assert_eq!(project.extract_feature_name("invalid-format"), None);
    }
}
