use crate::application::use_cases::initialize_project;
use crate::cli::formatters::{format_error, format_list, format_success};
use crate::infrastructure::filesystem::path_manager::PathManager;
use crate::infrastructure::repositories::project::ProjectRepository;
use anyhow::Result;

pub struct InitCommand {
    force: bool,
}

impl InitCommand {
    pub fn new(force: bool) -> Self {
        Self { force }
    }

    pub fn execute(&self) -> Result<()> {
        // Use current directory as project root
        let current_dir = std::env::current_dir()?;
        let path_manager = PathManager::new(current_dir);

        // Create repository
        let project_repo = ProjectRepository::new(path_manager);

        // Execute use case function
        match initialize_project(&project_repo, self.force) {
            Ok(()) => {
                println!(
                    "{}",
                    format_success("Initialized .kiro directory structure:")
                );
                let items = vec![
                    "Created .kiro/".to_string(),
                    "Created .kiro/config.toml (configuration template)".to_string(),
                    "Created .kiro/specs/".to_string(),
                    "Updated .gitignore".to_string(),
                ];
                println!("{}", format_list(&items));
                Ok(())
            }
            Err(crate::application::errors::ApplicationError::ProjectAlreadyExists) => {
                println!(
                    "{}",
                    format_error("Project already initialized. Use --force to reinitialize.")
                );
                Err(anyhow::anyhow!("Project already exists"))
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
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_init_command_new() {
        let cmd = InitCommand::new(false);
        assert!(!cmd.force);

        let cmd_force = InitCommand::new(true);
        assert!(cmd_force.force);
    }

    #[test]
    fn test_init_command_execute_success() {
        let _test_dir = TestDirectory::new();

        let cmd = InitCommand::new(false);
        let result = cmd.execute();

        assert!(result.is_ok());

        // Verify directory structure was created
        assert!(Path::new(".kiro").exists());
        assert!(Path::new(".kiro/specs").exists());
        assert!(Path::new(".kiro/config.toml").exists());
        assert!(Path::new(".gitignore").exists());
    }

    #[test]
    fn test_init_command_execute_already_exists() {
        let _test_dir = TestDirectory::new();

        // First initialization
        let cmd1 = InitCommand::new(false);
        let result1 = cmd1.execute();
        assert!(result1.is_ok());

        // Second initialization without force
        let cmd2 = InitCommand::new(false);
        let result2 = cmd2.execute();
        assert!(result2.is_err());
    }

    #[test]
    fn test_init_command_execute_with_force() {
        let _test_dir = TestDirectory::new();

        // First initialization
        let cmd1 = InitCommand::new(false);
        let result1 = cmd1.execute();
        assert!(result1.is_ok());

        // Modify config to test force overwrites
        let config_path = Path::new(".kiro/config.toml");
        fs::write(config_path, "# Modified content").unwrap();

        // Second initialization with force
        let cmd2 = InitCommand::new(true);
        let result2 = cmd2.execute();
        assert!(result2.is_ok());

        // Verify config was updated with steering section (new behavior: add [steering] to existing config)
        let config_content = fs::read_to_string(config_path).unwrap();
        assert!(config_content.contains("# Modified content"));
        assert!(config_content.contains("[[steering.types]]"));
    }

    #[test]
    fn test_init_command_creates_gitignore() {
        let _test_dir = TestDirectory::new();

        let cmd = InitCommand::new(false);
        let result = cmd.execute();
        assert!(result.is_ok());

        let gitignore_path = Path::new(".gitignore");
        assert!(gitignore_path.exists());

        let content = fs::read_to_string(gitignore_path).unwrap();
        assert!(!content.is_empty());
    }

    #[test]
    fn test_init_command_appends_to_existing_gitignore() {
        let _test_dir = TestDirectory::new();

        // Create existing .gitignore
        let gitignore_path = Path::new(".gitignore");
        fs::write(gitignore_path, "# Existing content\nnode_modules/\n").unwrap();

        let cmd = InitCommand::new(false);
        let result = cmd.execute();
        assert!(result.is_ok());

        let content = fs::read_to_string(gitignore_path).unwrap();
        assert!(content.contains("# Existing content"));
        assert!(content.contains("node_modules/"));
    }

    #[test]
    fn test_init_command_directory_structure() {
        let _test_dir = TestDirectory::new();

        let cmd = InitCommand::new(false);
        let result = cmd.execute();
        assert!(result.is_ok());

        // Check all expected directories
        assert!(Path::new(".kiro").is_dir());
        assert!(Path::new(".kiro/specs").is_dir());

        // Check config file
        assert!(Path::new(".kiro/config.toml").is_file());
    }

    #[test]
    fn test_init_command_config_content() {
        let _test_dir = TestDirectory::new();

        let cmd = InitCommand::new(false);
        let result = cmd.execute();
        assert!(result.is_ok());

        let config_path = Path::new(".kiro/config.toml");
        let content = fs::read_to_string(config_path).unwrap();

        // Verify config contains expected sections
        assert!(content.contains("[steering]"));
        assert!(content.contains("domain"));
        assert!(content.contains("instructions ="));
    }
}
