use crate::core::project::ProjectManager;
use crate::utils::error::Result;
use clap::Args;

#[derive(Args)]
pub struct NewCommand {
    /// Name of the new feature (must be in kebab-case)
    pub feature_name: String,
}

impl NewCommand {
    pub fn execute(&self) -> Result<()> {
        let project_manager = ProjectManager::new();

        println!("Creating new feature: {}", self.feature_name);

        let feature_path = project_manager.create_new_feature(&self.feature_name)?;

        println!("✅ Feature '{}' created successfully!", self.feature_name);
        println!("📁 Location: {}", feature_path.display());
        println!("📝 Files created:");
        println!("   - requirements.md");
        println!("   - design.md");
        println!("   - task.md");
        println!("   - spec.json");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    #[test]
    fn test_new_command_creation() {
        // Create temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let original_dir = env::current_dir().unwrap();

        // Change to temp directory
        env::set_current_dir(temp_dir.path()).unwrap();

        let command = NewCommand {
            feature_name: "test-feature".to_string(),
        };

        let result = command.execute();
        assert!(result.is_ok());

        // Check if files were created
        let spec_path = temp_dir.path().join(".kiro/specs");
        assert!(spec_path.exists());

        // Find the created directory (it will have today's date)
        let entries = std::fs::read_dir(&spec_path).unwrap();
        let mut found = false;
        for entry in entries {
            let entry = entry.unwrap();
            if entry
                .file_name()
                .to_str()
                .unwrap()
                .ends_with("-test-feature")
            {
                let feature_dir = entry.path();
                assert!(feature_dir.join("requirements.md").exists());
                assert!(feature_dir.join("design.md").exists());
                assert!(feature_dir.join("task.md").exists());
                assert!(feature_dir.join("spec.json").exists());
                found = true;
                break;
            }
        }
        assert!(found, "Feature directory not found");

        // Restore original directory
        env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_invalid_feature_name() {
        let command = NewCommand {
            feature_name: "Invalid_Name".to_string(),
        };

        let result = command.execute();
        assert!(result.is_err());
    }
}
