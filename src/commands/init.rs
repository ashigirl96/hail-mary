use anyhow::{Context, Result};
use clap::Args;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

#[derive(Args)]
pub struct InitCommand {
    /// Force overwrite existing configuration
    #[arg(long)]
    pub force: bool,
}

impl InitCommand {
    pub fn execute(&self) -> Result<()> {
        let kiro_dir = Path::new(".kiro");
        let config_path = kiro_dir.join("config.toml");
        let memory_dir = kiro_dir.join("memory");

        // Check if .kiro directory exists and handle accordingly
        if kiro_dir.exists() && !self.force {
            print!(".kiro directory already exists. Overwrite config.toml? (y/N): ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Initialization cancelled.");
                return Ok(());
            }
        }

        // Create directories
        fs::create_dir_all(kiro_dir).context("Failed to create .kiro directory")?;
        fs::create_dir_all(&memory_dir).context("Failed to create .kiro/memory directory")?;

        // Create config.toml template
        let config_template = r#"# .kiro/config.toml
# hail-mary Memory MCP project configuration

[memory]
# Memory types for categorization (customize for your project)
types = [
    "tech",           # General technical knowledge
    "project-tech",   # Project-specific technical details
    "domain",         # Business domain knowledge
    "workflow",       # Development workflows and processes
    "decision",       # Architecture decisions and rationale
]

# Instructions for MCP server
instructions = """
Available memory types:
- tech: General technical knowledge (languages, frameworks, algorithms)
- project-tech: This project's specific technical implementation
- domain: Business domain knowledge and requirements
- workflow: Development workflows and processes
- decision: Architecture decisions and their rationale
"""

# Document generation settings
[memory.document]
output_dir = ".kiro/memory"
format = "markdown"

# Database configuration
[memory.database]
path = ".kiro/memory/db.sqlite3"
"#;

        fs::write(&config_path, config_template).context("Failed to write config.toml")?;

        // Update .gitignore
        self.update_gitignore()?;

        println!("✅ Initialized .kiro directory structure:");
        println!("  - Created .kiro/");
        println!("  - Created .kiro/config.toml (configuration template)");
        println!("  - Created .kiro/memory/");
        println!("  - Updated .gitignore");
        println!();
        println!("You can now customize .kiro/config.toml for your project needs.");

        Ok(())
    }

    fn update_gitignore(&self) -> Result<()> {
        let gitignore_path = Path::new(".gitignore");

        if gitignore_path.exists() {
            let content = fs::read_to_string(gitignore_path)?;
            if !content.contains(".kiro/memory/db.sqlite3") {
                let mut file = fs::OpenOptions::new().append(true).open(gitignore_path)?;
                writeln!(file, "\n# hail-mary memory database")?;
                writeln!(file, ".kiro/memory/db.sqlite3")?;
                writeln!(file, ".kiro/memory/*.sqlite3-*")?;
            }
        } else {
            // Create new .gitignore if it doesn't exist
            let gitignore_content = r#"# hail-mary memory database
.kiro/memory/db.sqlite3
.kiro/memory/*.sqlite3-*
"#;
            fs::write(gitignore_path, gitignore_content)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::common::TestDirectory;

    #[test]
    fn test_init_creates_kiro_directory() {
        let _test_dir = TestDirectory::new();

        let cmd = InitCommand { force: false };
        let result = cmd.execute();

        assert!(result.is_ok(), "Init command should succeed");

        // Check that .kiro directory was created (relative to current directory)
        let kiro_dir = Path::new(".kiro");
        assert!(kiro_dir.exists(), ".kiro directory should be created");
        assert!(kiro_dir.is_dir(), ".kiro should be a directory");

        // Check that memory subdirectory was created
        let memory_dir = kiro_dir.join("memory");
        assert!(
            memory_dir.exists(),
            ".kiro/memory directory should be created"
        );
        assert!(memory_dir.is_dir(), ".kiro/memory should be a directory");
    }

    #[test]
    fn test_init_creates_config_toml() {
        let _test_dir = TestDirectory::new();

        let cmd = InitCommand { force: false };
        let result = cmd.execute();

        assert!(result.is_ok(), "Init command should succeed");

        // Check that config.toml was created (relative to current directory)
        let config_path = Path::new(".kiro/config.toml");
        assert!(config_path.exists(), "config.toml should be created");

        // Check content contains expected sections
        let content = fs::read_to_string(config_path).unwrap();
        assert!(
            content.contains("[memory]"),
            "Should contain [memory] section"
        );
        assert!(content.contains("types = ["), "Should contain types array");
        assert!(content.contains("tech"), "Should contain tech type");
        assert!(
            content.contains("project-tech"),
            "Should contain project-tech type"
        );
        assert!(content.contains("domain"), "Should contain domain type");
        assert!(
            content.contains("[memory.database]"),
            "Should contain database config"
        );
        assert!(
            content.contains(".kiro/memory/db.sqlite3"),
            "Should contain database path"
        );
    }

    #[test]
    fn test_init_updates_gitignore() {
        let _test_dir = TestDirectory::new();

        let cmd = InitCommand { force: false };
        let result = cmd.execute();

        assert!(result.is_ok(), "Init command should succeed");

        // Check that .gitignore was created (relative to current directory)
        let gitignore_path = Path::new(".gitignore");
        assert!(gitignore_path.exists(), ".gitignore should be created");

        // Check content contains database exclusions
        let content = fs::read_to_string(gitignore_path).unwrap();
        assert!(
            content.contains(".kiro/memory/db.sqlite3"),
            "Should exclude database file"
        );
        assert!(
            content.contains(".kiro/memory/*.sqlite3-*"),
            "Should exclude database temp files"
        );
        assert!(
            content.contains("hail-mary memory database"),
            "Should have comment"
        );
    }

    #[test]
    fn test_init_force_flag() {
        let _test_dir = TestDirectory::new();

        // Create existing .kiro directory (relative to current directory)
        let kiro_dir = Path::new(".kiro");
        fs::create_dir_all(kiro_dir).unwrap();

        // Write initial config
        let config_path = kiro_dir.join("config.toml");
        fs::write(&config_path, "# old config").unwrap();

        // Run init with force flag
        let cmd = InitCommand { force: true };
        let result = cmd.execute();

        assert!(result.is_ok(), "Init with force should succeed");

        // Check that config was overwritten
        let content = fs::read_to_string(config_path).unwrap();
        assert!(
            content.contains("[memory]"),
            "Should contain new config content"
        );
        assert!(
            !content.contains("# old config"),
            "Should not contain old content"
        );
    }

    #[test]
    fn test_init_with_existing_gitignore() {
        let _test_dir = TestDirectory::new();

        // Create existing .gitignore (relative to current directory)
        let gitignore_path = Path::new(".gitignore");
        fs::write(gitignore_path, "*.log\ntarget/\n").unwrap();

        let cmd = InitCommand { force: true };
        let result = cmd.execute();

        assert!(result.is_ok(), "Init command should succeed");

        // Check that .gitignore was updated, not replaced
        let content = fs::read_to_string(gitignore_path).unwrap();
        assert!(
            content.contains("*.log"),
            "Should preserve existing content"
        );
        assert!(
            content.contains("target/"),
            "Should preserve existing content"
        );
        assert!(
            content.contains(".kiro/memory/db.sqlite3"),
            "Should add database exclusions"
        );
    }
}
