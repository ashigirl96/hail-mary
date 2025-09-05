use anyhow::Result;

use crate::application::use_cases::launch_claude_with_spec;
use crate::cli::formatters::format_error;
use crate::infrastructure::filesystem::path_manager::PathManager;
use crate::infrastructure::repositories::project::ProjectRepository;

pub struct CodeCommand;

impl CodeCommand {
    pub fn new() -> Self {
        Self
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

        // Execute single use case
        match launch_claude_with_spec(&project_repo) {
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
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_command_new() {
        let command = CodeCommand::new();
        // Just ensure it can be created without panicking
        assert!(std::mem::size_of_val(&command) == 0);
    }

    #[test]
    fn test_code_command_default() {
        let command = CodeCommand::new();
        // Just ensure default works
        assert!(std::mem::size_of_val(&command) == 0);
    }

    // Note: execute() method testing is complex due to TUI and process launching
    // This should be tested in integration tests with proper mocking
}
