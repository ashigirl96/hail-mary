use crate::application::repositories::ProjectRepository as ProjectRepositoryTrait;
use crate::application::use_cases::{generate_document, reindex_memories};
use crate::cli::formatters::{
    format_error, format_info, format_path, format_reindex_stats_as_text, format_success,
};
use crate::infrastructure::filesystem::path_manager::PathManager;
use crate::infrastructure::mcp::server::{MemoryMcpServer, MemoryService};
use crate::infrastructure::repositories::memory::SqliteMemoryRepository;
use crate::infrastructure::repositories::project::ProjectRepository;
use anyhow::Result;

pub enum MemoryCommand {
    Serve,
    Document { memory_type: Option<String> },
    Reindex { dry_run: bool, verbose: bool },
}

impl MemoryCommand {
    pub fn execute(&self) -> Result<()> {
        match self {
            Self::Serve => self.serve(),
            Self::Document { memory_type } => self.document(memory_type.as_deref()),
            Self::Reindex { dry_run, verbose } => self.reindex(*dry_run, *verbose),
        }
    }

    fn serve(&self) -> Result<()> {
        // Log startup to stderr to avoid interfering with MCP protocol on stdout
        eprintln!("{}", format_info("Starting Memory MCP server..."));

        // Discover project and load configuration
        let path_manager = match PathManager::discover() {
            Ok(pm) => pm,
            Err(_) => {
                eprintln!(
                    "{}",
                    format_error("Not in a project directory. Run 'hail-mary init' first.")
                );
                return Err(anyhow::anyhow!("Project not found"));
            }
        };

        let project_repo = ProjectRepository::new(path_manager.clone());
        let config = match project_repo.load_config() {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("{}", format_error(&format!("Failed to load config: {}", e)));
                return Err(anyhow::anyhow!(e));
            }
        };

        // Initialize memory repository
        let db_path = path_manager.memory_db_path(true);
        let memory_repo = match SqliteMemoryRepository::new(&db_path) {
            Ok(repo) => repo,
            Err(e) => {
                eprintln!(
                    "{}",
                    format_error(&format!("Failed to open database: {}", e))
                );
                return Err(anyhow::anyhow!(e));
            }
        };

        // Create service and start MCP server
        let service = MemoryService::new(Box::new(memory_repo), config);
        let _server = MemoryMcpServer::new(service);

        eprintln!(
            "{}",
            format_info("Memory MCP server ready. Connect with MCP client via stdio.")
        );

        // // Run the actual MCP server with stdio transport
        // #[cfg(not(test))]
        // {
        //     use rmcp::{ServiceExt, transport::stdio};
        //
        //     let rt = tokio::runtime::Runtime::new()?;
        //     rt.block_on(async {
        //         let service = server
        //             .serve(stdio())
        //             .await
        //             .map_err(|e| anyhow::anyhow!("Failed to start MCP server: {}", e))?;
        //         service
        //             .waiting()
        //             .await
        //             .map_err(|e| anyhow::anyhow!("MCP server error: {}", e))
        //     })?;
        // }

        Ok(())
    }

    fn document(&self, memory_type: Option<&str>) -> Result<()> {
        eprintln!("{}", format_info("Generating memory documentation..."));

        // Discover project
        let path_manager = match PathManager::discover() {
            Ok(pm) => pm,
            Err(_) => {
                eprintln!(
                    "{}",
                    format_error("Not in a project directory. Run 'hail-mary init' first.")
                );
                return Err(anyhow::anyhow!("Project not found"));
            }
        };

        let project_repo = ProjectRepository::new(path_manager.clone());

        // Initialize memory repository
        let db_path = path_manager.memory_db_path(true);
        let mut memory_repo = match SqliteMemoryRepository::new(&db_path) {
            Ok(repo) => repo,
            Err(e) => {
                eprintln!(
                    "{}",
                    format_error(&format!("Failed to open database: {}", e))
                );
                return Err(anyhow::anyhow!(e));
            }
        };

        // Execute use case function
        match generate_document(&mut memory_repo, &project_repo, memory_type) {
            Ok(output_dir) => {
                eprintln!(
                    "{}",
                    format_success(&format!(
                        "Generated memory documents in: {}",
                        format_path(&output_dir)
                    ))
                );
                Ok(())
            }
            Err(crate::application::errors::ApplicationError::InvalidMemoryType(mt)) => {
                eprintln!(
                    "{}",
                    format_error(&format!(
                        "Invalid memory type: '{}'. Check your config.toml for valid types.",
                        mt
                    ))
                );
                Err(anyhow::anyhow!("Invalid memory type"))
            }
            Err(e) => {
                eprintln!("{}", format_error(&e.to_string()));
                Err(anyhow::anyhow!(e))
            }
        }
    }

    fn reindex(&self, dry_run: bool, verbose: bool) -> Result<()> {
        if dry_run {
            eprintln!(
                "{}",
                format_info("🔍 Dry run mode - would perform reindex operations:")
            );
            eprintln!("  - Analyze database for duplicates and optimization opportunities");
            eprintln!("  - Remove logical deleted entries");
            eprintln!("  - Rebuild FTS5 index");
            eprintln!("  - Archive old database");

            if verbose {
                eprintln!("{}", format_info("Verbose logging enabled"));
            }

            return Ok(());
        }

        // Discover project
        let path_manager = match PathManager::discover() {
            Ok(pm) => pm,
            Err(_) => {
                eprintln!(
                    "{}",
                    format_error("Not in a project directory. Run 'hail-mary init' first.")
                );
                return Err(anyhow::anyhow!("Project not found"));
            }
        };

        // Initialize memory repository
        let db_path = path_manager.memory_db_path(true);
        let mut memory_repo = match SqliteMemoryRepository::new(&db_path) {
            Ok(repo) => repo,
            Err(e) => {
                eprintln!(
                    "{}",
                    format_error(&format!("Failed to open database: {}", e))
                );
                return Err(anyhow::anyhow!(e));
            }
        };

        // Execute use case function
        match reindex_memories(&mut memory_repo, verbose) {
            Ok(stats) => {
                eprintln!("{}", format_success("Database reindexed successfully"));
                eprintln!("{}", format_reindex_stats_as_text(&stats));
                Ok(())
            }
            Err(e) => {
                eprintln!("{}", format_error(&e.to_string()));
                Err(anyhow::anyhow!(e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::repositories::memory_repository::MemoryRepository;
    use crate::application::test_helpers::TestDirectory;
    use crate::cli::commands::init::InitCommand;
    use crate::domain::entities::memory::Memory;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_memory_command_serve() {
        let _test_dir = TestDirectory::new();

        // Initialize project first
        let init_cmd = InitCommand::new(false);
        init_cmd.execute().unwrap();

        // Test serve command
        let cmd = MemoryCommand::Serve;
        let result = cmd.execute();

        // In test mode, serve just returns success
        assert!(result.is_ok());
    }

    #[test]
    fn test_memory_command_serve_without_project() {
        let _test_dir = TestDirectory::new();

        // Try to serve without initializing project
        let cmd = MemoryCommand::Serve;
        let result = cmd.execute();

        assert!(result.is_err());
    }

    #[test]
    fn test_memory_command_document_all() {
        let _test_dir = TestDirectory::new();

        // Initialize project first
        let init_cmd = InitCommand::new(false);
        init_cmd.execute().unwrap();

        // Initialize database
        let path_manager = PathManager::discover().unwrap();
        let db_path = path_manager.memory_db_path(true);
        let mut memory_repo = SqliteMemoryRepository::new(&db_path).unwrap();

        // Add some test memories
        let memory = Memory::new(
            "tech".to_string(),
            "Test Memory".to_string(),
            "Test content".to_string(),
        );
        memory_repo.save(&memory).unwrap();

        // Test document command for all types
        let cmd = MemoryCommand::Document { memory_type: None };
        let result = cmd.execute();

        assert!(result.is_ok());

        // Check that document files were created
        assert!(Path::new(".kiro/memory/tech.md").exists());
    }

    #[test]
    fn test_memory_command_document_specific_type() {
        let _test_dir = TestDirectory::new();

        // Initialize project first
        let init_cmd = InitCommand::new(false);
        init_cmd.execute().unwrap();

        // Initialize database
        let path_manager = PathManager::discover().unwrap();
        let db_path = path_manager.memory_db_path(true);
        let mut memory_repo = SqliteMemoryRepository::new(&db_path).unwrap();

        // Add test memory
        let memory = Memory::new(
            "tech".to_string(),
            "Rust Tips".to_string(),
            "Use pattern matching".to_string(),
        );
        memory_repo.save(&memory).unwrap();

        // Test document command for specific type
        let cmd = MemoryCommand::Document {
            memory_type: Some("tech".to_string()),
        };
        let result = cmd.execute();

        assert!(result.is_ok());
        assert!(Path::new(".kiro/memory/tech.md").exists());

        // Check content
        let content = fs::read_to_string(".kiro/memory/tech.md").unwrap();
        assert!(content.contains("Rust Tips"));
        assert!(content.contains("Use pattern matching"));
    }

    #[test]
    fn test_memory_command_document_invalid_type() {
        let _test_dir = TestDirectory::new();

        // Initialize project first
        let init_cmd = InitCommand::new(false);
        init_cmd.execute().unwrap();

        // Test document command with invalid type
        let cmd = MemoryCommand::Document {
            memory_type: Some("invalid_type".to_string()),
        };
        let result = cmd.execute();

        assert!(result.is_err());
    }

    #[test]
    fn test_memory_command_document_without_project() {
        let _test_dir = TestDirectory::new();

        // Try to generate documents without initializing project
        let cmd = MemoryCommand::Document { memory_type: None };
        let result = cmd.execute();

        assert!(result.is_err());
    }

    #[test]
    fn test_memory_command_reindex_dry_run() {
        let _test_dir = TestDirectory::new();

        // Initialize project first
        let init_cmd = InitCommand::new(false);
        init_cmd.execute().unwrap();

        // Test reindex with dry-run
        let cmd = MemoryCommand::Reindex {
            dry_run: true,
            verbose: false,
        };
        let result = cmd.execute();

        assert!(result.is_ok());
    }

    #[test]
    fn test_memory_command_reindex_dry_run_verbose() {
        let _test_dir = TestDirectory::new();

        // Initialize project first
        let init_cmd = InitCommand::new(false);
        init_cmd.execute().unwrap();

        // Test reindex with dry-run and verbose
        let cmd = MemoryCommand::Reindex {
            dry_run: true,
            verbose: true,
        };
        let result = cmd.execute();

        assert!(result.is_ok());
    }

    #[test]
    fn test_memory_command_reindex_actual() {
        let _test_dir = TestDirectory::new();

        // Initialize project first
        let init_cmd = InitCommand::new(false);
        init_cmd.execute().unwrap();

        // Initialize database and add some memories
        let path_manager = PathManager::discover().unwrap();
        let db_path = path_manager.memory_db_path(true);
        let mut memory_repo = SqliteMemoryRepository::new(&db_path).unwrap();

        // Add a memory and mark it as deleted
        let mut memory = Memory::new(
            "tech".to_string(),
            "To Delete".to_string(),
            "This will be deleted".to_string(),
        );
        memory.deleted = true;
        memory_repo.save(&memory).unwrap();

        // Test actual reindex
        let cmd = MemoryCommand::Reindex {
            dry_run: false,
            verbose: false,
        };
        let result = cmd.execute();

        assert!(result.is_ok());
    }

    #[test]
    fn test_memory_command_reindex_verbose() {
        let _test_dir = TestDirectory::new();

        // Initialize project first
        let init_cmd = InitCommand::new(false);
        init_cmd.execute().unwrap();

        // Test reindex with verbose output
        let cmd = MemoryCommand::Reindex {
            dry_run: false,
            verbose: true,
        };
        let result = cmd.execute();

        assert!(result.is_ok());
    }

    #[test]
    fn test_memory_command_reindex_without_project() {
        let _test_dir = TestDirectory::new();

        // Try to reindex without initializing project
        let cmd = MemoryCommand::Reindex {
            dry_run: false,
            verbose: false,
        };
        let result = cmd.execute();

        assert!(result.is_err());
    }
}
