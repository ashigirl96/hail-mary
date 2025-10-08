use crate::application::use_cases::initialize_project;
use crate::cli::formatters::{format_error, format_success};
use crate::infrastructure::filesystem::path_manager::PathManager;
use crate::infrastructure::repositories::{
    config::ConfigRepository, spec::SpecRepository, steering::SteeringRepository,
};
use anyhow::Result;

pub struct InitCommand;

impl Default for InitCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl InitCommand {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self) -> Result<()> {
        // Use current directory as project root
        let current_dir = std::env::current_dir()?;
        let path_manager = PathManager::new(current_dir);

        // Create repositories
        let config_repo = ConfigRepository::new(path_manager.clone());
        let spec_repo = SpecRepository::new(path_manager.clone());
        let steering_repo = SteeringRepository::new(path_manager);

        // Execute use case function (now idempotent)
        match initialize_project(&config_repo, &spec_repo, &steering_repo) {
            Ok(()) => {
                println!("{}", format_success("Initialization complete."));
                Ok(())
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
        let _cmd = InitCommand::new();
        // No force field to test anymore
    }

    #[test]
    fn test_init_command_execute_success() {
        let _test_dir = TestDirectory::new();

        let cmd = InitCommand::new();
        let result = cmd.execute();

        assert!(result.is_ok());

        // Verify directory structure was created
        assert!(Path::new(".kiro").exists());
        assert!(Path::new(".kiro/specs").exists());
        assert!(Path::new(".kiro/config.toml").exists());
        assert!(Path::new(".gitignore").exists());
    }

    #[test]
    fn test_init_command_is_idempotent() {
        let _test_dir = TestDirectory::new();

        // First initialization
        let cmd1 = InitCommand::new();
        let result1 = cmd1.execute();
        assert!(result1.is_ok());

        // Second initialization (should succeed, not error)
        let cmd2 = InitCommand::new();
        let result2 = cmd2.execute();
        assert!(result2.is_ok());

        // Third initialization (should also succeed)
        let cmd3 = InitCommand::new();
        let result3 = cmd3.execute();
        assert!(result3.is_ok());
    }

    #[test]
    fn test_init_command_partial_initialization() {
        let _test_dir = TestDirectory::new();

        // Create only .kiro directory (partial initialization)
        fs::create_dir(".kiro").unwrap();

        // Run init command (should complete the initialization)
        let cmd = InitCommand::new();
        let result = cmd.execute();
        assert!(result.is_ok());

        // Verify all expected components were created
        assert!(Path::new(".kiro").exists());
        assert!(Path::new(".kiro/specs").exists());
        assert!(Path::new(".kiro/config.toml").exists());
        assert!(Path::new(".kiro/steering").exists());
        assert!(Path::new(".kiro/steering/product.md").exists());
        assert!(Path::new(".kiro/steering/tech.md").exists());
        assert!(Path::new(".kiro/steering/structure.md").exists());
    }

    #[test]
    fn test_init_command_creates_gitignore() {
        let _test_dir = TestDirectory::new();

        let cmd = InitCommand::new();
        let result = cmd.execute();
        assert!(result.is_ok());

        let gitignore_path = Path::new(".gitignore");
        assert!(gitignore_path.exists());

        // With file-based steering system, gitignore may be empty
        let _ = fs::read_to_string(gitignore_path).unwrap();
    }

    #[test]
    fn test_init_command_appends_to_existing_gitignore() {
        let _test_dir = TestDirectory::new();

        // Create existing .gitignore
        let gitignore_path = Path::new(".gitignore");
        fs::write(gitignore_path, "# Existing content\nnode_modules/\n").unwrap();

        let cmd = InitCommand::new();
        let result = cmd.execute();
        assert!(result.is_ok());

        let content = fs::read_to_string(gitignore_path).unwrap();
        assert!(content.contains("# Existing content"));
        assert!(content.contains("node_modules/"));
    }

    #[test]
    fn test_init_command_directory_structure() {
        let _test_dir = TestDirectory::new();

        let cmd = InitCommand::new();
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

        let cmd = InitCommand::new();
        let result = cmd.execute();
        assert!(result.is_ok());

        let config_path = Path::new(".kiro/config.toml");
        let content = fs::read_to_string(config_path).unwrap();

        // Verify config contains expected sections
        assert!(content.contains("[[steering.types]]"));
        assert!(content.contains("name ="));
        assert!(content.contains("purpose ="));
    }

    #[test]
    fn test_init_command_deploys_slash_commands() {
        let _test_dir = TestDirectory::new();

        let cmd = InitCommand::new();
        let result = cmd.execute();
        assert!(result.is_ok());

        // Verify hm commands were deployed
        let hm_dir = Path::new(".claude/commands/hm");
        assert!(hm_dir.exists(), ".claude/commands/hm should exist");

        let expected_hm_commands = ["steering-remember.md", "steering.md"];
        for file in &expected_hm_commands {
            let file_path = hm_dir.join(file);
            assert!(file_path.exists(), "HM command file {} should exist", file);

            // Verify content
            let content = fs::read_to_string(&file_path).unwrap();
            assert!(!content.is_empty(), "File {} should have content", file);
        }

        // Verify spec commands were deployed
        let spec_dir = Path::new(".claude/commands/spec");
        assert!(spec_dir.exists(), ".claude/commands/spec should exist");

        let expected_spec_commands = [
            "requirements.md",
            "investigate.md",
            "design.md",
            "timeline.md",
        ];
        for file in &expected_spec_commands {
            let file_path = spec_dir.join(file);
            assert!(
                file_path.exists(),
                "Spec command file {} should exist",
                file
            );

            // Verify content
            let content = fs::read_to_string(&file_path).unwrap();
            assert!(!content.is_empty(), "File {} should have content", file);
        }

        // Verify agents were deployed
        let agents_dir = Path::new(".claude/agents");
        assert!(agents_dir.exists(), ".claude/agents should exist");

        // Check all expected agent files
        let expected_agents = [
            "steering-investigator.md",
            "root-cause-investigator.md",
            "backend-architect.md",
            "frontend-architect.md",
            "system-architect.md",
        ];
        for file in &expected_agents {
            let file_path = agents_dir.join(file);
            assert!(file_path.exists(), "Agent file {} should exist", file);

            // Verify content
            let content = fs::read_to_string(&file_path).unwrap();
            assert!(!content.is_empty(), "File {} should have content", file);
        }
    }

    #[test]
    fn test_init_command_overwrites_slash_commands() {
        let _test_dir = TestDirectory::new();

        // Create .claude/commands/hm with old content
        let hm_dir = Path::new(".claude/commands/hm");
        fs::create_dir_all(hm_dir).unwrap();
        fs::write(hm_dir.join("steering.md"), "OLD CONTENT").unwrap();

        let cmd = InitCommand::new();
        let result = cmd.execute();
        assert!(result.is_ok());

        // Verify file was overwritten with new content
        let content = fs::read_to_string(hm_dir.join("steering.md")).unwrap();
        assert!(!content.contains("OLD CONTENT"));
        assert!(content.contains("/hm:steering - Steering Documentation")); // Expected content from embedded file
    }
}
