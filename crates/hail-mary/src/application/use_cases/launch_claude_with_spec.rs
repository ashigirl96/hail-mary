use anyhow::Result;
use std::io::{self, Write};

use crate::application::errors::ApplicationError;
use crate::application::repositories::ProjectRepository;
use crate::domain::entities::project::ProjectConfig;
use crate::domain::value_objects::system_prompt::SystemPrompt;
use crate::infrastructure::process::claude_launcher::ClaudeProcessLauncher;
use crate::infrastructure::tui::spec_selector::{SpecSelectionResult, SpecSelectorTui};

pub fn launch_claude_with_spec(
    project_repo: &dyn ProjectRepository,
    no_danger: bool,
) -> Result<(), ApplicationError> {
    // 1. Get list of specifications
    let specs = project_repo.list_spec_directories().map_err(|e| {
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
            project_repo.create_feature(&name)?;

            // Generate the actual directory name with date prefix (same logic as create_feature)
            let date = chrono::Utc::now().format("%Y-%m-%d");
            format!("{}-{}", date, name)
        }
        SpecSelectionResult::Cancelled => {
            return Ok(()); // User cancelled, exit gracefully
        }
    };

    // 3. Get spec path and generate system prompt
    let spec_path = project_repo.get_spec_path(&spec_name)?;
    let system_prompt = SystemPrompt::new(&spec_name, &spec_path);

    // 4. Launch Claude with system prompt
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
    use crate::application::test_helpers::MockProjectRepository;

    #[test]
    fn test_launch_claude_with_existing_spec() {
        let mut mock_repo = MockProjectRepository::new();
        mock_repo.add_created_feature("test-spec");

        // Mock the list_spec_directories to return our test spec
        let _specs = [("test-spec".to_string(), false)];
        // Note: This test would need a mock TUI to fully test the workflow
        // For now, we just test that the repository methods work correctly

        let result = mock_repo.get_spec_path("test-spec");
        assert!(result.is_ok());
    }

    #[test]
    fn test_launch_claude_with_nonexistent_spec() {
        let mock_repo = MockProjectRepository::new();

        let result = mock_repo.get_spec_path("nonexistent-spec");
        assert!(result.is_err());
        match result.unwrap_err() {
            ApplicationError::SpecNotFound(_) => {}
            _ => panic!("Expected SpecNotFound error"),
        }
    }
}
