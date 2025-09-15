use anyhow::Result;

use crate::application::use_cases::launch_claude_with_spec;
use crate::cli::formatters::format_error;
use crate::infrastructure::filesystem::path_manager::PathManager;
use crate::infrastructure::repositories::{
    config::ConfigRepository, spec::SpecRepository, steering::SteeringRepository,
};

pub struct CodeCommand {
    no_danger: bool,
}

impl CodeCommand {
    pub fn new(no_danger: bool) -> Self {
        Self { no_danger }
    }

    fn project_not_found_error(&self) -> anyhow::Error {
        println!(
            "{}",
            format_error("Not in a project directory. Run 'hail-mary init' first.")
        );
        anyhow::anyhow!("Project not found")
    }

    pub fn execute(&self) -> Result<()> {
        // Discover project root
        let path_manager = PathManager::discover().map_err(|_| self.project_not_found_error())?;

        // Create repositories
        let spec_repo = SpecRepository::new(path_manager.clone());
        let config_repo = ConfigRepository::new(path_manager.clone());
        let steering_repo = SteeringRepository::new(path_manager);

        // Execute single use case
        match launch_claude_with_spec(&spec_repo, &config_repo, &steering_repo, self.no_danger) {
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
            Err(crate::application::errors::ApplicationError::InvalidFeatureName(name)) => {
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
        Self::new(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_command_new() {
        let command = CodeCommand::new(false);
        // Just ensure it can be created without panicking
        assert!(!command.no_danger);
    }

    #[test]
    fn test_code_command_new_with_no_danger() {
        let command = CodeCommand::new(true);
        assert!(command.no_danger);
    }

    #[test]
    fn test_code_command_default() {
        let command = CodeCommand::default();
        // Just ensure default works
        assert!(!command.no_danger);
    }

    // Note: execute() method testing is complex due to TUI and process launching
    // This should be tested in integration tests with proper mocking
}
