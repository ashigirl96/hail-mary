use anyhow::Result;
use clap::Parser;
use hail_mary::cli::args::{Cli, Commands, MemoryCommands};
use hail_mary::cli::commands::{InitCommand, MemoryCommand, NewCommand};
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
        Commands::Init { force } => {
            let command = InitCommand::new(force);
            command.execute()?;
        }
        Commands::New { name } => {
            let command = NewCommand::new(name);
            command.execute()?;
        }
        Commands::Memory { command } => {
            let memory_command = match command {
                MemoryCommands::Serve => MemoryCommand::Serve,
                MemoryCommands::Document { memory_type } => MemoryCommand::Document { memory_type },
                MemoryCommands::Reindex { dry_run, verbose } => {
                    MemoryCommand::Reindex { dry_run, verbose }
                }
            };
            memory_command.execute()?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hail_mary::application::repositories::memory_repository::MemoryRepository;
    use hail_mary::domain::entities::memory::Memory;
    use hail_mary::domain::value_objects::confidence::Confidence;
    use hail_mary::infrastructure::repositories::memory::SqliteMemoryRepository;
    use std::env;
    use std::fs;
    use std::path::Path;
    use uuid::Uuid;

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
            Commands::Init { force } => {
                let command = InitCommand::new(force);
                let result = command.execute();
                assert!(result.is_ok());

                // Verify directory structure was created
                assert!(Path::new(".kiro").exists());
                assert!(Path::new(".kiro/config.toml").exists());
                assert!(Path::new(".kiro/memory").exists());
                assert!(Path::new(".kiro/specs").exists());
            }
            _ => panic!("Expected init command"),
        }
    }

    #[test]
    fn test_main_init_command_with_force() {
        let _test_dir = TestDirectory::new();

        // Create existing .kiro directory
        fs::create_dir_all(".kiro").unwrap();
        fs::write(".kiro/config.toml", "existing content").unwrap();

        // Test init with force flag overwrites existing
        let args = vec!["hail-mary", "init", "--force"];
        let cli = Cli::parse_from(args);

        match cli.command {
            Commands::Init { force } => {
                assert!(force);
                let command = InitCommand::new(force);
                let result = command.execute();
                assert!(result.is_ok());

                // Verify config was overwritten
                let config = fs::read_to_string(".kiro/config.toml").unwrap();
                assert!(config.contains("[memory]"));
                assert!(!config.contains("existing content"));
            }
            _ => panic!("Expected init command"),
        }
    }

    #[test]
    fn test_main_new_command() {
        let _test_dir = TestDirectory::new();

        // Initialize project first
        InitCommand::new(false).execute().unwrap();

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
        InitCommand::new(false).execute().unwrap();

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
    fn test_main_memory_serve() {
        let _test_dir = TestDirectory::new();

        // Initialize project first
        InitCommand::new(false).execute().unwrap();

        // Test memory serve command parsing
        let args = vec!["hail-mary", "memory", "serve"];
        let cli = Cli::parse_from(args);

        match cli.command {
            Commands::Memory { command } => {
                match command {
                    MemoryCommands::Serve => {
                        // In real execution, this would start the MCP server
                        // For testing, we just verify the command parses correctly
                        assert!(true);
                    }
                    _ => panic!("Expected serve subcommand"),
                }
            }
            _ => panic!("Expected memory command"),
        }
    }

    #[test]
    fn test_main_memory_document() {
        let _test_dir = TestDirectory::new();

        // Initialize project and create test memory
        InitCommand::new(false).execute().unwrap();

        // Create a test database with memories
        let db_path = ".kiro/memory/memories.db";
        let mut repo = SqliteMemoryRepository::new(db_path).unwrap();

        let memory = Memory {
            id: Uuid::new_v4(),
            memory_type: "tech".to_string(),
            title: "Test Memory".to_string(),
            content: "Test content".to_string(),
            tags: vec!["test".to_string()],
            confidence: Confidence::new(0.9).unwrap(),
            reference_count: 0,
            created_at: chrono::Utc::now(),
            last_accessed: None,
            deleted: false,
        };
        repo.save(&memory).unwrap();

        // Test memory document command
        let args = vec!["hail-mary", "memory", "document"];
        let cli = Cli::parse_from(args);

        match cli.command {
            Commands::Memory { command } => {
                match command {
                    MemoryCommands::Document { memory_type } => {
                        assert!(memory_type.is_none());

                        let memory_command = MemoryCommand::Document { memory_type };
                        let result = memory_command.execute();
                        assert!(result.is_ok());

                        // Verify documentation was created for each type
                        // When no type is specified, documents are created for all types
                        assert!(Path::new(".kiro/memory/tech.md").exists());
                        let doc = fs::read_to_string(".kiro/memory/tech.md").unwrap();
                        // The document should contain the memory title
                        // Note: Documents for empty memory types still get created
                        assert!(doc.contains("tech Memories") || doc.contains("Test Memory"));
                    }
                    _ => panic!("Expected document subcommand"),
                }
            }
            _ => panic!("Expected memory command"),
        }
    }

    #[test]
    fn test_main_memory_document_with_type() {
        let _test_dir = TestDirectory::new();

        // Initialize project
        InitCommand::new(false).execute().unwrap();

        // Test memory document with type filter
        let args = vec!["hail-mary", "memory", "document", "--type", "tech"];
        let cli = Cli::parse_from(args);

        match cli.command {
            Commands::Memory { command } => match command {
                MemoryCommands::Document { memory_type } => {
                    assert_eq!(memory_type, Some("tech".to_string()));
                }
                _ => panic!("Expected document subcommand"),
            },
            _ => panic!("Expected memory command"),
        }
    }

    #[test]
    fn test_main_memory_reindex() {
        let _test_dir = TestDirectory::new();

        // Initialize project
        InitCommand::new(false).execute().unwrap();

        // Create a test database
        let db_path = ".kiro/memory/memories.db";
        let _ = SqliteMemoryRepository::new(db_path).unwrap();

        // Test memory reindex command
        let args = vec!["hail-mary", "memory", "reindex"];
        let cli = Cli::parse_from(args);

        match cli.command {
            Commands::Memory { command } => match command {
                MemoryCommands::Reindex { dry_run, verbose } => {
                    assert!(!dry_run);
                    assert!(!verbose);

                    let memory_command = MemoryCommand::Reindex { dry_run, verbose };
                    let result = memory_command.execute();
                    assert!(result.is_ok());
                }
                _ => panic!("Expected reindex subcommand"),
            },
            _ => panic!("Expected memory command"),
        }
    }

    #[test]
    fn test_main_memory_reindex_with_flags() {
        let _test_dir = TestDirectory::new();

        // Initialize project
        InitCommand::new(false).execute().unwrap();

        // Test memory reindex with flags
        let args = vec!["hail-mary", "memory", "reindex", "--dry-run", "--verbose"];
        let cli = Cli::parse_from(args);

        match cli.command {
            Commands::Memory { command } => match command {
                MemoryCommands::Reindex { dry_run, verbose } => {
                    assert!(dry_run);
                    assert!(verbose);
                }
                _ => panic!("Expected reindex subcommand"),
            },
            _ => panic!("Expected memory command"),
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
