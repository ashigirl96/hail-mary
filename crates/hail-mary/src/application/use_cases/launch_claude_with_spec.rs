use anyhow::Result;
use std::io::{self, Write};

use crate::application::errors::ApplicationError;
use crate::application::repositories::{
    ConfigRepositoryInterface, SpecRepositoryInterface,
    steering_repository::SteeringRepositoryInterface,
};
use crate::domain::entities::project::ProjectConfig;
use crate::domain::entities::steering::Steerings;
use crate::domain::value_objects::system_prompt::SystemPrompt;
use crate::infrastructure::process::claude_launcher::ClaudeProcessLauncher;
use crate::infrastructure::tui::spec_selector::{SpecSelectionResult, SpecSelectorTui};

pub fn launch_claude_with_spec(
    spec_repo: &dyn SpecRepositoryInterface,
    config_repo: &dyn ConfigRepositoryInterface,
    steering_repo: &dyn SteeringRepositoryInterface,
    no_danger: bool,
) -> Result<(), ApplicationError> {
    // 1. Get list of specifications
    let specs = spec_repo.list_spec_directories().map_err(|e| {
        ApplicationError::FileSystemError(format!("Failed to list specifications: {}", e))
    })?;

    // 2. Run TUI for spec selection (includes new spec option)
    let mut tui = SpecSelectorTui::new(specs);
    let selection_result = tui
        .run()
        .map_err(|e| ApplicationError::FileSystemError(format!("TUI error: {}", e)))?;

    let spec_name = match selection_result {
        SpecSelectionResult::Existing(name) => name,
        SpecSelectionResult::CreateNew => {
            // Prompt for name and create new spec
            let name = prompt_for_spec_name()?;
            ProjectConfig::validate_spec_name(&name)?;
            spec_repo.create_feature(&name)?;

            // Generate the actual directory name with date prefix (same logic as create_feature)
            let date = chrono::Utc::now().format("%Y-%m-%d");
            format!("{}-{}", date, name)
        }
        SpecSelectionResult::Cancelled => {
            return Ok(()); // User cancelled, exit gracefully
        }
    };

    // 3. Load project configuration and steering files
    let config = config_repo.load_config()?;
    let steering_files = steering_repo.load_steering_files(&config.steering)?;
    let steerings = Steerings(steering_files);

    // 4. Get spec path and generate system prompt with steering
    let spec_path = spec_repo.get_spec_path(&spec_name)?;
    let system_prompt = SystemPrompt::new(&spec_name, &spec_path, &steerings);

    // 5. Launch Claude with system prompt
    let launcher = ClaudeProcessLauncher::new();
    launcher
        .launch(system_prompt.as_str(), no_danger)
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
