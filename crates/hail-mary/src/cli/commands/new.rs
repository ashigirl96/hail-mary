use crate::application::use_cases::create_spec;
use crate::cli::formatters::{format_error, format_success};
use crate::infrastructure::filesystem::path_manager::PathManager;
use crate::infrastructure::repositories::spec::SpecRepository;
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
        let spec_repo = SpecRepository::new(path_manager);

        // Execute use case function (validation is done inside)
        match create_spec(&spec_repo, &self.name) {
            Ok(spec_path) => {
                println!(
                    "{}",
                    format_success(&format!(
                        "Spec '{}' created successfully at: {}",
                        self.name, spec_path
                    ))
                );
                Ok(())
            }
            Err(crate::application::errors::ApplicationError::InvalidSpecName(name)) => {
                println!(
                    "{}",
                    format_error(&format!(
                        "Invalid spec name '{}'. Use kebab-case (lowercase letters, numbers, and hyphens only).",
                        name
                    ))
                );
                Err(anyhow::anyhow!("Invalid spec name"))
            }
            Err(crate::application::errors::ApplicationError::SpecAlreadyExists(name)) => {
                println!(
                    "{}",
                    format_error(&format!("Spec '{}' already exists.", name))
                );
                Err(anyhow::anyhow!("Spec already exists"))
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
        let init_cmd = InitCommand::new();
        init_cmd.execute().unwrap();

        // Create new feature
        let cmd = NewCommand::new("user-authentication".to_string());
        let result = cmd.execute();

        assert!(result.is_ok());

        // Verify feature directory was created with only essential files
        let date = chrono::Utc::now().format("%Y-%m-%d");
        let spec_dir = format!(".kiro/specs/{}-user-authentication", date);
        assert!(Path::new(&spec_dir).exists());
        // Essential files should be created
        // TODO: design.md and investigation.md are temporarily disabled
        // assert!(Path::new(&format!("{}/design.md", spec_dir)).exists());
        // assert!(Path::new(&format!("{}/investigation.md", spec_dir)).exists());
        assert!(Path::new(&format!("{}/memo.md", spec_dir)).exists());
        assert!(Path::new(&format!("{}/tasks.md", spec_dir)).exists()); // tasks.md is now created
        // These files should NOT be created automatically
        assert!(!Path::new(&format!("{}/requirements.md", spec_dir)).exists());
        assert!(!Path::new(&format!("{}/spec.json", spec_dir)).exists());
        // Verify design.md and investigation.md are NOT created (temporarily disabled)
        assert!(!Path::new(&format!("{}/design.md", spec_dir)).exists());
        assert!(!Path::new(&format!("{}/investigation.md", spec_dir)).exists());
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
        let init_cmd = InitCommand::new();
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
        let init_cmd = InitCommand::new();
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
            let spec_dir = format!(".kiro/specs/{}-{}", date, name);
            assert!(
                Path::new(&spec_dir).exists(),
                "Feature directory should exist for '{}'",
                name
            );
        }
    }

    #[test]
    fn test_new_command_execute_duplicate_feature() {
        let _test_dir = TestDirectory::new();

        // Initialize project first
        let init_cmd = InitCommand::new();
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
    fn test_new_command_spec_path_format() {
        let _test_dir = TestDirectory::new();

        // Initialize project first
        let init_cmd = InitCommand::new();
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
        let init_cmd = InitCommand::new();
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
