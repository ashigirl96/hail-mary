use crate::application::use_cases::create_feature;
use crate::cli::formatters::{format_error, format_success};
use crate::infrastructure::filesystem::path_manager::PathManager;
use crate::infrastructure::repositories::project::ProjectRepository;
use anyhow::Result;

pub struct NewCommand {
    name: String,
}

impl NewCommand {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn execute(&self) -> Result<()> {
        // Discover project root
        let path_manager = match PathManager::discover() {
            Ok(pm) => pm,
            Err(_) => {
                println!(
                    "{}",
                    format_error("Not in a project directory. Run 'hail-mary init' first.")
                );
                return Err(anyhow::anyhow!("Project not found"));
            }
        };

        // Create repository
        let project_repo = ProjectRepository::new(path_manager);

        // Execute use case function (validation is done inside)
        match create_feature(&project_repo, &self.name) {
            Ok(feature_path) => {
                println!(
                    "{}",
                    format_success(&format!(
                        "Feature '{}' created successfully at: {}",
                        self.name, feature_path
                    ))
                );
                Ok(())
            }
            Err(crate::application::errors::ApplicationError::InvalidFeatureName(name)) => {
                println!(
                    "{}",
                    format_error(&format!(
                        "Invalid feature name '{}'. Use kebab-case (lowercase letters, numbers, and hyphens only).",
                        name
                    ))
                );
                Err(anyhow::anyhow!("Invalid feature name"))
            }
            Err(crate::application::errors::ApplicationError::FeatureAlreadyExists(name)) => {
                println!(
                    "{}",
                    format_error(&format!("Feature '{}' already exists.", name))
                );
                Err(anyhow::anyhow!("Feature already exists"))
            }
            Err(e) => {
                println!("{}", format_error(&e.to_string()));
                Err(anyhow::anyhow!(e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_helpers::TestDirectory;
    use crate::cli::commands::init::InitCommand;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_new_command_new() {
        let cmd = NewCommand::new("test-feature".to_string());
        assert_eq!(cmd.name, "test-feature");
    }

    #[test]
    fn test_new_command_execute_success() {
        let _test_dir = TestDirectory::new();

        // Initialize project first
        let init_cmd = InitCommand::new(false);
        init_cmd.execute().unwrap();

        // Create new feature
        let cmd = NewCommand::new("user-authentication".to_string());
        let result = cmd.execute();

        assert!(result.is_ok());

        // Verify feature directory was created
        let date = chrono::Utc::now().format("%Y-%m-%d");
        let feature_dir = format!(".kiro/specs/{}-user-authentication", date);
        assert!(Path::new(&feature_dir).exists());
        assert!(Path::new(&format!("{}/requirements.md", feature_dir)).exists());
        assert!(Path::new(&format!("{}/design.md", feature_dir)).exists());
        assert!(Path::new(&format!("{}/tasks.md", feature_dir)).exists());
        assert!(Path::new(&format!("{}/spec.json", feature_dir)).exists());
    }

    #[test]
    fn test_new_command_execute_without_project() {
        let _test_dir = TestDirectory::new();

        // Try to create feature without initializing project
        let cmd = NewCommand::new("test-feature".to_string());
        let result = cmd.execute();

        assert!(result.is_err());
    }

    #[test]
    fn test_new_command_execute_invalid_name() {
        let _test_dir = TestDirectory::new();

        // Initialize project first
        let init_cmd = InitCommand::new(false);
        init_cmd.execute().unwrap();

        // Try invalid feature names
        let invalid_names = vec![
            "Invalid-Case",
            "invalid_underscore",
            "invalid space",
            "-invalid-start",
            "invalid-end-",
            "invalid--double",
        ];

        for name in invalid_names {
            let cmd = NewCommand::new(name.to_string());
            let result = cmd.execute();
            assert!(result.is_err(), "Name '{}' should be invalid", name);
        }
    }

    #[test]
    fn test_new_command_execute_valid_names() {
        let _test_dir = TestDirectory::new();

        // Initialize project first
        let init_cmd = InitCommand::new(false);
        init_cmd.execute().unwrap();

        let valid_names = vec![
            "api-endpoints",
            "database-migration",
            "feature-123",
            "simple",
            "a",
        ];

        for name in valid_names {
            let cmd = NewCommand::new(name.to_string());
            let result = cmd.execute();
            assert!(result.is_ok(), "Name '{}' should be valid", name);

            // Verify directory exists
            let date = chrono::Utc::now().format("%Y-%m-%d");
            let feature_dir = format!(".kiro/specs/{}-{}", date, name);
            assert!(
                Path::new(&feature_dir).exists(),
                "Feature directory should exist for '{}'",
                name
            );
        }
    }

    #[test]
    fn test_new_command_execute_duplicate_feature() {
        let _test_dir = TestDirectory::new();

        // Initialize project first
        let init_cmd = InitCommand::new(false);
        init_cmd.execute().unwrap();

        // Create feature first time
        let cmd1 = NewCommand::new("duplicate-test".to_string());
        let result1 = cmd1.execute();
        assert!(result1.is_ok());

        // Try to create same feature again
        let cmd2 = NewCommand::new("duplicate-test".to_string());
        let result2 = cmd2.execute();
        assert!(result2.is_err());
    }

    #[test]
    fn test_new_command_template_files_content() {
        let _test_dir = TestDirectory::new();

        // Initialize project first
        let init_cmd = InitCommand::new(false);
        init_cmd.execute().unwrap();

        // Create new feature
        let cmd = NewCommand::new("template-test".to_string());
        let result = cmd.execute();
        assert!(result.is_ok());

        // Check template content
        let date = chrono::Utc::now().format("%Y-%m-%d");
        let feature_dir = format!(".kiro/specs/{}-template-test", date);

        // Check requirements.md
        let requirements_path = format!("{}/requirements.md", feature_dir);
        let requirements_content = fs::read_to_string(&requirements_path).unwrap();
        assert!(requirements_content.contains("# Requirements"));
        assert!(requirements_content.contains("template-test"));
        assert!(requirements_content.contains("## Overview"));
        assert!(requirements_content.contains("## User Stories"));
        assert!(requirements_content.contains("## Acceptance Criteria"));

        // Check design.md
        let design_path = format!("{}/design.md", feature_dir);
        let design_content = fs::read_to_string(&design_path).unwrap();
        assert!(design_content.contains("# Design"));
        assert!(design_content.contains("template-test"));
        assert!(design_content.contains("## Architecture"));
        assert!(design_content.contains("## Components"));

        // Check tasks.md
        let tasks_path = format!("{}/tasks.md", feature_dir);
        let tasks_content = fs::read_to_string(&tasks_path).unwrap();
        assert!(tasks_content.contains("# Tasks"));
        assert!(tasks_content.contains("## Implementation Tasks"));
        assert!(tasks_content.contains("## Testing Tasks"));

        // Check spec.json
        let spec_path = format!("{}/spec.json", feature_dir);
        let spec_content = fs::read_to_string(&spec_path).unwrap();
        assert_eq!(spec_content, "{}");
    }

    #[test]
    fn test_new_command_feature_path_format() {
        let _test_dir = TestDirectory::new();

        // Initialize project first
        let init_cmd = InitCommand::new(false);
        init_cmd.execute().unwrap();

        // Create new feature
        let cmd = NewCommand::new("path-test".to_string());
        let result = cmd.execute();
        assert!(result.is_ok());

        // Verify path format
        let date = chrono::Utc::now().format("%Y-%m-%d");
        let expected_dir = format!(".kiro/specs/{}-path-test", date);
        assert!(Path::new(&expected_dir).exists());
        assert!(Path::new(&expected_dir).is_dir());
    }

    #[test]
    fn test_new_command_edge_cases() {
        let _test_dir = TestDirectory::new();

        // Initialize project first
        let init_cmd = InitCommand::new(false);
        init_cmd.execute().unwrap();

        // Single character (valid)
        let cmd = NewCommand::new("x".to_string());
        assert!(cmd.execute().is_ok());

        // Numbers only (valid)
        let cmd = NewCommand::new("123".to_string());
        assert!(cmd.execute().is_ok());

        // Multiple dashes (valid)
        let cmd = NewCommand::new("a-b-c-d".to_string());
        assert!(cmd.execute().is_ok());
    }
}
