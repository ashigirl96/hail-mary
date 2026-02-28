use anyhow::Result;

use crate::application::use_cases::{initialize_project, launch_claude_with_spec};
use crate::cli::formatters::format_error;
use crate::infrastructure::filesystem::path_manager::PathManager;
use crate::infrastructure::repositories::spec::SpecRepository;

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
        let path_manager = match PathManager::discover() {
            Ok(pm) => pm,
            Err(_) => {
                let current_dir = std::env::current_dir()?;
                PathManager::new(current_dir)
            }
        };

        let spec_repo = SpecRepository::new(path_manager.clone());

        initialize_project(path_manager.project_root())?;

        match launch_claude_with_spec(&spec_repo, self.no_danger, self.continue_conversation) {
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
