use anyhow::Result;

use crate::application::repositories::SpecRepositoryInterface;
use crate::application::use_cases::complete_features;
use crate::cli::formatters::{format_error, format_success};
use crate::infrastructure::filesystem::path_manager::PathManager;
use crate::infrastructure::repositories::spec::SpecRepository;
use crate::infrastructure::tui::completion_ui::select_specs_for_completion;

pub struct CompleteCommand;

impl Default for CompleteCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl CompleteCommand {
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

        // Create spec repository
        let spec_repo = SpecRepository::new(path_manager);

        // Get list of specifications
        let specs = match spec_repo.list_spec_directories() {
            Ok(specs) => specs,
            Err(e) => {
                println!("{}", format_error(&e.to_string()));
                return Err(anyhow::anyhow!(e));
            }
        };

        if specs.is_empty() {
            println!(
                "{}",
                format_error("No specifications found in .kiro/specs directory.")
            );
            return Ok(());
        }

        // Show TUI for spec selection
        let selected_specs = match select_specs_for_completion(specs)? {
            Some(specs) => specs,
            None => {
                // User cancelled
                return Ok(());
            }
        };

        if selected_specs.is_empty() {
            return Ok(());
        }

        // Execute complete features
        match complete_features(&spec_repo, &selected_specs) {
            Ok(()) => {
                println!(
                    "{}",
                    format_success(&format!(
                        "{} specification(s) moved to archive successfully.",
                        selected_specs.len()
                    ))
                );
            }
            Err(e) => {
                println!("{}", format_error(&e.to_string()));
                return Err(anyhow::anyhow!(e));
            }
        }

        Ok(())
    }
}
