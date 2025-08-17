use crate::mcp::server::get_default_db_path;
use crate::memory::{
    models::MemoryType, repository::SqliteMemoryRepository, service::MemoryService,
};
use crate::utils::error::Result;
use clap::Args;
use std::path::PathBuf;

/// List memories from the database
#[derive(Args)]
pub struct ListCommand {
    /// Filter by memory type
    #[arg(long, value_name = "TYPE")]
    pub r#type: Option<String>,

    /// Show deleted memories
    #[arg(long)]
    pub deleted: bool,

    /// Path to the database file (defaults to .kiro/memory/memory.db)
    #[arg(long, value_name = "PATH")]
    pub db_path: Option<PathBuf>,

    /// Limit number of results
    #[arg(long, short, default_value = "20")]
    pub limit: usize,

    /// Enable verbose output
    #[arg(long, short)]
    pub verbose: bool,
}

impl ListCommand {
    /// Execute the list command
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
            let service = MemoryService::new(repository);

            let memories = if let Some(type_str) = self.r#type {
                // Filter by type
                match MemoryType::from_str(&type_str) {
                    Some(memory_type) => service.get_all_by_type(&memory_type).await?,
                    None => {
                        eprintln!(
                            "Invalid memory type: {}. Valid types are: tech, project-tech, domain",
                            type_str
                        );
                        return Ok(());
                    }
                }
            } else {
                // Get all memories
                let mut all_memories = Vec::new();
                for memory_type in &[
                    MemoryType::Tech,
                    MemoryType::ProjectTech,
                    MemoryType::Domain,
                ] {
                    let memories = service.get_all_by_type(memory_type).await?;
                    all_memories.extend(memories);
                }
                all_memories
            };

            // Filter deleted if not requested
            let memories: Vec<_> = if self.deleted {
                // TODO: Need to implement get_deleted_memories in service
                println!("⚠️  Showing deleted memories is not yet implemented");
                return Ok(());
            } else {
                memories.into_iter().filter(|m| !m.deleted).collect()
            };

            // Apply limit
            let memories: Vec<_> = memories.into_iter().take(self.limit).collect();

            // Display results
            if memories.is_empty() {
                println!("No memories found");
            } else {
                println!("Found {} memories:", memories.len());
                println!();

                for memory in memories {
                    println!("📝 {}", memory.title);
                    println!("   ID: {}", memory.id);
                    println!("   Type: {}", memory.memory_type);
                    println!("   References: {}", memory.reference_count);
                    println!("   Confidence: {:.2}", memory.confidence);

                    if self.verbose {
                        println!("   Tags: {}", memory.tags.join(", "));
                        println!("   Created: {}", memory.created_at);
                        if let Some(last_accessed) = memory.last_accessed {
                            println!("   Last accessed: {}", last_accessed);
                        }
                        if !memory.content.is_empty() {
                            let preview = if memory.content.len() > 100 {
                                format!("{}...", &memory.content[..100])
                            } else {
                                memory.content.clone()
                            };
                            println!("   Content: {}", preview);
                        }
                    }
                    println!();
                }
            }

            Ok(())
        })
    }
}
