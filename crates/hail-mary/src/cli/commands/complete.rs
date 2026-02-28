use anyhow::Result;

use crate::application::repositories::SpecRepositoryInterface;
use crate::application::use_cases::complete_specs;
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
        let path_manager = match PathManager::discover() {
            Ok(pm) => pm,
            Err(_) => {
                println!(
                    "{}",
                    format_error("Not in a project directory. Run 'hail-mary code' to initialize.")
                );
                return Err(anyhow::anyhow!("Project not found"));
            }
        };

        let spec_repo = SpecRepository::new(path_manager);

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
                format_error("No specifications found in .specs directory.")
            );
            return Ok(());
        }

        let selected_specs = match select_specs_for_completion(specs)? {
            Some(specs) => specs,
            None => {
                return Ok(());
            }
        };

        if selected_specs.is_empty() {
            return Ok(());
        }

        match complete_specs(&spec_repo, &selected_specs) {
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
