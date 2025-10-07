use anyhow::Result;

use crate::application::use_cases::{initialize_project, launch_claude_with_spec};
use crate::cli::formatters::format_error;
use crate::infrastructure::filesystem::path_manager::PathManager;
use crate::infrastructure::repositories::{
    config::ConfigRepository, spec::SpecRepository, steering::SteeringRepository,
};

pub struct CodeCommand {
    no_danger: bool,
    continue_conversation: bool,
}

impl CodeCommand {
    pub fn new(no_danger: bool, continue_conversation: bool) -> Self {
        Self {
            no_danger,
            continue_conversation,
        }
    }

    pub fn execute(&self) -> Result<()> {
        // Try to discover project root, or use current directory
        let path_manager = match PathManager::discover() {
            Ok(pm) => pm,
            Err(_) => {
                // If no .kiro directory found, use current directory for initialization
                let current_dir = std::env::current_dir()?;
                PathManager::new(current_dir)
            }
        };

        // Create repositories
        let spec_repo = SpecRepository::new(path_manager.clone());
        let config_repo = ConfigRepository::new(path_manager.clone());
        let steering_repo = SteeringRepository::new(path_manager.clone());

        // Initialize project if needed (this is idempotent)
        initialize_project(&config_repo, &spec_repo, &steering_repo)?;

        // Execute single use case
        match launch_claude_with_spec(
            &spec_repo,
            &config_repo,
            &steering_repo,
            self.no_danger,
            self.continue_conversation,
        ) {
            Ok(()) => Ok(()),
            Err(crate::application::errors::ApplicationError::ProcessLaunchError(msg)) => {
                println!("{}", format_error(&msg));
                Err(anyhow::anyhow!("Failed to launch Claude Code"))
            }
            Err(crate::application::errors::ApplicationError::SpecNotFound(spec)) => {
                println!(
                    "{}",
                    format_error(&format!("Specification '{}' not found", spec))
                );
                Err(anyhow::anyhow!("Spec not found"))
            }
            Err(crate::application::errors::ApplicationError::InvalidSpecName(name)) => {
                println!(
                    "{}",
                    format_error(&format!(
                        "Invalid specification name '{}'. Use kebab-case (lowercase letters, numbers, and hyphens only).",
                        name
                    ))
                );
                Err(anyhow::anyhow!("Invalid spec name"))
            }
            Err(e) => {
                println!("{}", format_error(&e.to_string()));
                Err(anyhow::anyhow!(e))
            }
        }
    }
}

impl Default for CodeCommand {
    fn default() -> Self {
        Self::new(false, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_command_new() {
        let command = CodeCommand::new(false, false);
        // Just ensure it can be created without panicking
        assert!(!command.no_danger);
        assert!(!command.continue_conversation);
    }

    #[test]
    fn test_code_command_new_with_no_danger() {
        let command = CodeCommand::new(true, false);
        assert!(command.no_danger);
        assert!(!command.continue_conversation);
    }

    #[test]
    fn test_code_command_default() {
        let command = CodeCommand::default();
        // Just ensure default works
        assert!(!command.no_danger);
        assert!(!command.continue_conversation);
    }

    // Note: execute() method testing is complex due to TUI and process launching
    // This should be tested in integration tests with proper mocking
}
