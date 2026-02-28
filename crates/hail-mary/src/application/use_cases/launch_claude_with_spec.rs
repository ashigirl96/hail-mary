use std::io::{self, Write};

use crate::application::errors::ApplicationError;
use crate::application::repositories::SpecRepositoryInterface;
use crate::domain::value_objects::spec::SpecValidator;
use crate::infrastructure::process::claude_launcher::ClaudeProcessLauncher;
use crate::infrastructure::tui::spec_selector::{SpecSelectionResult, SpecSelectorTui};

pub fn launch_claude_with_spec(
    spec_repo: &dyn SpecRepositoryInterface,
    no_danger: bool,
    continue_conversation: bool,
) -> Result<(), ApplicationError> {
    // 1. Get list of specifications
    let specs = spec_repo.list_spec_directories().map_err(|e| {
        ApplicationError::FileSystemError(format!("Failed to list specifications: {}", e))
    })?;

    // 2. Run TUI for spec selection
    let mut tui = SpecSelectorTui::new(specs);
    let selection_result = tui
        .run()
        .map_err(|e| ApplicationError::FileSystemError(format!("TUI error: {}", e)))?;

    match selection_result {
        SpecSelectionResult::SingleSpec(name) => {
            let _path = spec_repo.get_spec_path(&name)?;
        }
        SpecSelectionResult::CreateNew => {
            let name = prompt_for_spec_name()?;
            SpecValidator::validate_spec_name(&name)?;
            spec_repo.create_spec(&name)?;
        }
        SpecSelectionResult::NoSpec => {}
        SpecSelectionResult::Cancelled => {
            return Ok(());
        }
    };

    // 3. Launch Claude
    let launcher = ClaudeProcessLauncher::new();
    launcher
        .launch(no_danger, continue_conversation)
        .map_err(|e| ApplicationError::ProcessLaunchError(e.to_string()))?;

    Ok(())
}

fn prompt_for_spec_name() -> Result<String, ApplicationError> {
    print!("Enter specification name: ");
    io::stdout()
        .flush()
        .map_err(|e| ApplicationError::FileSystemError(format!("Failed to flush stdout: {}", e)))?;

    let mut name = String::new();
    io::stdin()
        .read_line(&mut name)
        .map_err(|e| ApplicationError::FileSystemError(format!("Failed to read input: {}", e)))?;

    Ok(name.trim().to_string())
}
