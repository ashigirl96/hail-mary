use anyhow::Result;
use clap::Parser;
use hail_mary::cli::args::{Cli, Commands};
use hail_mary::cli::commands::{CodeCommand, CompleteCommand, InitCommand, NewCommand, completion};
use hail_mary::cli::formatters::format_error;
use std::process;

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", format_error(&format!("{:#}", e)));
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            let command = InitCommand::new();
            command.execute()?;
        }
        Commands::New { name } => {
            let command = NewCommand::new(name);
            command.execute()?;
        }
        Commands::Completion { shell } => {
            completion::handle_completion(&shell)?;
        }
        Commands::Complete => {
            let command = CompleteCommand::new();
            command.execute()?;
        }
        Commands::Code { no_danger } => {
            let command = CodeCommand::new(no_danger);
            command.execute()?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::path::Path;

    use std::sync::{Mutex, MutexGuard};

    // Global mutex to synchronize current directory changes across tests
    static TEST_DIR_MUTEX: Mutex<()> = Mutex::new(());

    // Test helper for managing temporary directories
    // Uses the same thread-safe pattern as the one in application::test_helpers
    struct TestDirectory {
        original_dir: std::path::PathBuf,
        _temp_dir: tempfile::TempDir,
        _guard: MutexGuard<'static, ()>,
    }

    impl TestDirectory {
        fn new() -> Self {
            // Acquire global lock to prevent concurrent directory changes
            let guard = TEST_DIR_MUTEX
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            let original_dir = env::current_dir().unwrap();
            let temp_dir = tempfile::tempdir().unwrap();
            env::set_current_dir(temp_dir.path()).unwrap();
            Self {
                original_dir,
                _temp_dir: temp_dir,
                _guard: guard,
            }
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = env::set_current_dir(&self.original_dir);
            // Mutex guard is automatically released when _guard is dropped
        }
    }

    #[test]
    fn test_main_init_command() {
        let _test_dir = TestDirectory::new();

        // Test init command creates .kiro directory structure
        let args = vec!["hail-mary", "init"];
        let cli = Cli::parse_from(args);

        match cli.command {
            Commands::Init => {
                let command = InitCommand::new();
                let result = command.execute();
                assert!(result.is_ok());

                // Verify directory structure was created
                assert!(Path::new(".kiro").exists());
                assert!(Path::new(".kiro/config.toml").exists());
                assert!(Path::new(".kiro/specs").exists());
            }
            _ => panic!("Expected init command"),
        }
    }

    #[test]
    fn test_main_init_command_idempotent() {
        let _test_dir = TestDirectory::new();

        // Create existing .kiro directory with config
        fs::create_dir_all(".kiro").unwrap();
        fs::write(".kiro/config.toml", "existing content").unwrap();

        // Test init is idempotent (no force flag needed)
        let args = vec!["hail-mary", "init"];
        let cli = Cli::parse_from(args);

        match cli.command {
            Commands::Init => {
                let command = InitCommand::new();
                let result = command.execute();
                assert!(result.is_ok());

                // Verify existing config is preserved and steering section is added
                let config = fs::read_to_string(".kiro/config.toml").unwrap();
                assert!(config.contains("existing content"));
                assert!(config.contains("[[steering.types]]"));
            }
            _ => panic!("Expected init command"),
        }
    }

    #[test]
    fn test_main_new_command() {
        let _test_dir = TestDirectory::new();

        // Initialize project first
        InitCommand::new().execute().unwrap();

        // Test new feature command
        let args = vec!["hail-mary", "new", "test-feature"];
        let cli = Cli::parse_from(args);

        match cli.command {
            Commands::New { name } => {
                assert_eq!(name, "test-feature");
                let command = NewCommand::new(name);
                let result = command.execute();
                assert!(result.is_ok());

                // Verify feature directory was created with date prefix
                let specs_dir = fs::read_dir(".kiro/specs").unwrap();
                let feature_dirs: Vec<_> = specs_dir
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| {
                        entry
                            .file_name()
                            .to_string_lossy()
                            .ends_with("-test-feature")
                    })
                    .collect();
                assert_eq!(feature_dirs.len(), 1);
            }
            _ => panic!("Expected new command"),
        }
    }

    #[test]
    fn test_main_new_command_invalid_name() {
        let _test_dir = TestDirectory::new();

        // Initialize project first
        InitCommand::new().execute().unwrap();

        // Test new feature with invalid name
        let args = vec!["hail-mary", "new", "Invalid Name"];
        let cli = Cli::parse_from(args);

        match cli.command {
            Commands::New { name } => {
                let command = NewCommand::new(name);
                let result = command.execute();
                assert!(result.is_err());

                let err_msg = result.unwrap_err().to_string();
                // Check that the error message mentions invalid feature name
                // The actual error shown in the output is:
                // "Invalid feature name 'Invalid Name'. Use kebab-case (lowercase letters, numbers, and hyphens only)."
                // But when we get the error through anyhow, it might be different
                assert!(
                    err_msg.contains("Invalid")
                        || err_msg.contains("invalid")
                        || err_msg.contains("kebab")
                );
            }
            _ => panic!("Expected new command"),
        }
    }

    #[test]
    fn test_run_function_error_handling() {
        let _test_dir = TestDirectory::new();

        // Test that run function returns error for uninitialized project
        let args = vec!["hail-mary", "new", "test-feature"];
        let cli = Cli::parse_from(args);

        // This should fail because project is not initialized
        let result = match cli.command {
            Commands::New { name } => {
                let command = NewCommand::new(name);
                command.execute()
            }
            _ => panic!("Expected new command"),
        };

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not found") || err.to_string().contains("initialize"));
    }
}
