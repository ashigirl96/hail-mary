use crate::mcp::server::get_default_db_path;
use crate::memory::{repository::SqliteMemoryRepository, service::MemoryService};
use crate::utils::error::Result;
use clap::Args;
use std::path::PathBuf;

/// Delete memories from the database
#[derive(Args)]
pub struct DeleteCommand {
    /// Memory ID to delete
    pub memory_id: String,

    /// Path to the database file (defaults to ~/.local/share/hail-mary/memory.db)
    #[arg(long, value_name = "PATH")]
    pub db_path: Option<PathBuf>,

    /// Actually delete the memory (default is dry run)
    #[arg(long)]
    pub confirm: bool,

    /// Enable verbose output
    #[arg(long, short)]
    pub verbose: bool,
}

impl DeleteCommand {
    /// Execute the delete command
    pub fn execute(self) -> Result<()> {
        // Determine database path
        let db_path = self
            .db_path
            .unwrap_or_else(|| get_default_db_path().expect("Failed to get default database path"));

        if !db_path.exists() {
            eprintln!("Database not found at {:?}", db_path);
            return Ok(());
        }

        // Create runtime for async operations
        let runtime = tokio::runtime::Runtime::new()?;

        runtime.block_on(async {
            // Initialize repository and service
            let repository = SqliteMemoryRepository::new(&db_path)?;
            let mut service = MemoryService::new(repository);

            // First, check if the memory exists
            if let Some(memory) = service.get_memory(&self.memory_id).await? {
                if self.verbose {
                    println!("Found memory:");
                    println!("  ID: {}", memory.id);
                    println!("  Type: {}", memory.memory_type);
                    println!("  Topic: {}", memory.topic);
                    println!("  Created: {}", memory.created_at);
                }

                if self.confirm {
                    // Actually delete the memory
                    service.delete_memory(&self.memory_id).await?;
                    println!(
                        "✅ Memory '{}' has been deleted (soft delete)",
                        self.memory_id
                    );
                } else {
                    println!("⚠️  Dry run mode - no changes made");
                    println!("To actually delete, run with --confirm flag:");
                    println!("  hail-mary memory delete {} --confirm", self.memory_id);
                }
            } else {
                eprintln!("❌ Memory with ID '{}' not found", self.memory_id);
            }

            Ok(())
        })
    }
}
