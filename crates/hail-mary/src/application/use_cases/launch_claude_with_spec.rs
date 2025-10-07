use anyhow::Result;
use std::io::{self, Write};

use crate::application::errors::ApplicationError;
use crate::application::repositories::{
    ConfigRepositoryInterface, SpecRepositoryInterface,
    steering_repository::SteeringRepositoryInterface,
};
use crate::domain::value_objects::spec::SpecValidator;
use crate::domain::value_objects::steering::Steerings;
use crate::domain::value_objects::system_prompt::SystemPrompt;
use crate::infrastructure::process::claude_launcher::ClaudeProcessLauncher;
use crate::infrastructure::tui::spec_selector::{SpecSelectionResult, SpecSelectorTui};

pub fn launch_claude_with_spec(
    spec_repo: &dyn SpecRepositoryInterface,
    config_repo: &dyn ConfigRepositoryInterface,
    steering_repo: &dyn SteeringRepositoryInterface,
    no_danger: bool,
    continue_conversation: bool,
) -> Result<(), ApplicationError> {
    // 1. Get list of specifications
    let specs = spec_repo.list_spec_directories().map_err(|e| {
        ApplicationError::FileSystemError(format!("Failed to list specifications: {}", e))
    })?;

    // 2. Run TUI for spec selection (includes new spec and SBI options)
    let mut tui = SpecSelectorTui::new(specs, spec_repo);
    let selection_result = tui
        .run()
        .map_err(|e| ApplicationError::FileSystemError(format!("TUI error: {}", e)))?;

    let (spec_name, spec_path) = match selection_result {
        SpecSelectionResult::SingleSpec(name) => {
            let path = spec_repo.get_spec_path(&name)?;
            (Some(name), Some(path))
        }
        SpecSelectionResult::Pbi(name) => {
            let path = spec_repo.get_spec_path(&name)?;
            (Some(name), Some(path))
        }
        SpecSelectionResult::Sbi(pbi_name, sbi_name) => {
            let pbi_path = spec_repo.get_spec_path(&pbi_name)?;
            let sbi_path = pbi_path.join(&sbi_name);
            (Some(sbi_name), Some(sbi_path))
        }
        SpecSelectionResult::CreateNew => {
            // Prompt for name and create new spec
            let name = prompt_for_spec_name()?;
            SpecValidator::validate_spec_name(&name)?;
            spec_repo.create_spec(&name)?;

            // Generate the actual directory name with date prefix
            let date = chrono::Utc::now().format("%Y-%m-%d");
            let full_name = format!("{}-{}", date, name);
            let path = spec_repo.get_spec_path(&full_name)?;
            (Some(full_name), Some(path))
        }
        SpecSelectionResult::CreateNewSbi(pbi_name) => {
            // Prompt for SBI name
            let sbi_title = prompt_for_sbi_name()?;

            // Auto-number SBI
            let existing_sbis = spec_repo.list_sbis(&pbi_name)?;
            let next_number = existing_sbis.len() + 1;
            let sbi_name = format!("sbi-{}-{}", next_number, sbi_title);

            // Create SBI (generates tasks.md and memo.md only)
            spec_repo.create_sbi(&pbi_name, &sbi_name)?;

            // Get SBI path
            let pbi_path = spec_repo.get_spec_path(&pbi_name)?;
            let sbi_path = pbi_path.join(&sbi_name);
            (Some(sbi_name), Some(sbi_path))
        }
        SpecSelectionResult::NoSpec => (None, None),
        SpecSelectionResult::Cancelled => {
            return Ok(()); // User cancelled, exit gracefully
        }
    };

    // 3. Load project configuration and steering files
    let steering_config = config_repo.load_steering_config()?;
    let steering_files = steering_repo.load_steering_files(&steering_config)?;
    let steerings = Steerings(steering_files);

    // 4. Generate system prompt based on spec selection
    let system_prompt = if let (Some(name), Some(path)) = (spec_name, spec_path) {
        // With spec: generate full system prompt
        SystemPrompt::new(Some(&name), Some(&path), &steerings)
    } else {
        // Without spec: generate system prompt with only steering
        SystemPrompt::new(None, None, &steerings)
    };

    // 5. Launch Claude with system prompt
    let launcher = ClaudeProcessLauncher::new();
    launcher
        .launch(system_prompt.as_str(), no_danger, continue_conversation)
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

fn prompt_for_sbi_name() -> Result<String, ApplicationError> {
    print!("Enter SBI name (kebab-case): ");
    io::stdout()
        .flush()
        .map_err(|e| ApplicationError::FileSystemError(format!("Failed to flush stdout: {}", e)))?;

    let mut name = String::new();
    io::stdin()
        .read_line(&mut name)
        .map_err(|e| ApplicationError::FileSystemError(format!("Failed to read input: {}", e)))?;

    Ok(name.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_helpers::MockSpecRepository;

    #[test]
    fn test_launch_claude_with_existing_spec() {
        let mock_repo = MockSpecRepository::with_specs(vec!["2025-09-09-test-spec".to_string()]);

        // Test that the repository methods work correctly
        let result = mock_repo.get_spec_path("2025-09-09-test-spec");
        assert!(result.is_ok());

        let specs = mock_repo.list_spec_directories().unwrap();
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].0, "2025-09-09-test-spec");
        assert!(!specs[0].1); // not archived
    }

    #[test]
    fn test_launch_claude_with_nonexistent_spec() {
        let mock_repo = MockSpecRepository::new();

        let result = mock_repo.get_spec_path("nonexistent-spec");
        assert!(result.is_err());
        match result.unwrap_err() {
            ApplicationError::SpecNotFound(_) => {}
            _ => panic!("Expected SpecNotFound error"),
        }
    }
}
