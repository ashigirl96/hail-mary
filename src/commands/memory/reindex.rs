use crate::mcp::server::get_default_db_path;
use crate::memory::reindex::{ReindexConfig, ReindexService};
use crate::utils::error::Result;
use clap::Args;
use std::path::PathBuf;

/// Reindex and optimize the memory database
#[derive(Args)]
pub struct ReindexCommand {
    /// Path to the database file (defaults to ~/.local/share/hail-mary/memory.db)
    #[arg(long, value_name = "PATH")]
    pub db_path: Option<PathBuf>,

    /// Similarity threshold for detecting duplicates (0.0 to 1.0, default: 0.85)
    #[arg(long, value_name = "THRESHOLD", default_value = "0.85")]
    pub similarity_threshold: f32,

    /// Disable database backup before reindexing
    #[arg(long)]
    pub no_backup: bool,

    /// Custom backup directory (defaults to ~/.local/share/hail-mary/backups)
    #[arg(long, value_name = "DIR")]
    pub backup_dir: Option<PathBuf>,

    /// Enable verbose output
    #[arg(long, short)]
    pub verbose: bool,

    /// Dry run - analyze duplicates without making changes
    #[arg(long)]
    pub dry_run: bool,

    /// Generate embeddings for all memories during reindex
    #[arg(long)]
    pub generate_embeddings: bool,

    /// Batch size for embedding generation (default: 32)
    #[arg(long, default_value = "32")]
    pub batch_size: usize,

    /// Force regenerate embeddings even if they already exist
    #[arg(long)]
    pub force_regenerate: bool,
}

impl ReindexCommand {
    /// Execute the reindex command
    pub fn execute(self) -> Result<()> {
        // Validate similarity threshold
        if self.similarity_threshold < 0.0 || self.similarity_threshold > 1.0 {
            eprintln!("Error: Similarity threshold must be between 0.0 and 1.0");
            return Ok(());
        }

        // Determine database path
        let db_path = self
            .db_path
            .clone()
            .unwrap_or_else(|| get_default_db_path().expect("Failed to get default database path"));

        // Check if database exists
        if !db_path.exists() {
            eprintln!("Error: Database not found at {:?}", db_path);
            eprintln!(
                "Please run 'hail-mary memory serve' or 'hail-mary memory document' first to create the database."
            );
            return Ok(());
        }

        if self.verbose {
            println!("🔄 Reindex Configuration:");
            println!("  Database: {:?}", db_path);
            println!("  Similarity threshold: {:.2}", self.similarity_threshold);
            println!(
                "  Backup: {}",
                if self.no_backup {
                    "disabled"
                } else {
                    "enabled"
                }
            );
            if let Some(ref dir) = self.backup_dir {
                println!("  Backup directory: {:?}", dir);
            }
        }

        if self.dry_run {
            println!("🔍 Running in dry-run mode - no changes will be made");
            return self.run_dry_analysis(&db_path);
        }

        // Create runtime for async operations
        let runtime = tokio::runtime::Runtime::new()?;

        runtime.block_on(async {
            // Configure reindex service
            let mut config = ReindexConfig {
                similarity_threshold: self.similarity_threshold,
                backup_enabled: !self.no_backup,
                verbose: self.verbose,
                ..Default::default()
            };

            if let Some(backup_dir) = self.backup_dir {
                config.backup_dir = backup_dir;
            }

            // Create and run reindex service
            let service = ReindexService::new(config)?;
            let result = if self.generate_embeddings {
                println!("🧮 Generating embeddings for memories...");
                service.reindex_with_embeddings(&db_path).await?
            } else {
                service.reindex(&db_path).await?
            };

            // Print results
            println!("\n✅ Reindex completed successfully!");
            println!("📊 Statistics:");
            println!("  Total memories processed: {}", result.total_memories);
            println!("  Duplicate pairs found: {}", result.duplicates_found);
            println!("  Duplicates merged: {}", result.duplicates_merged);
            println!("  Duration: {} seconds", result.duration_seconds);

            if let Some(backup_path) = result.backup_path {
                println!("  Backup saved to: {}", backup_path.display());
            }

            if result.duplicates_merged > 0 {
                let reduction =
                    result.duplicates_merged as f64 / result.total_memories as f64 * 100.0;
                println!("  Database size reduction: {:.1}%", reduction);
            }

            Ok(())
        })
    }

    /// Run dry analysis to show what would be done
    fn run_dry_analysis(&self, db_path: &PathBuf) -> Result<()> {
        use crate::memory::{
            embeddings::EmbeddingService,
            models::MemoryType,
            repository::{MemoryRepository, SqliteMemoryRepository},
        };

        println!("\n🔍 Analyzing database for duplicates...");

        // Load all memories
        let repository = SqliteMemoryRepository::new(db_path)?;
        let mut all_memories = Vec::new();

        for memory_type in &[
            MemoryType::Tech,
            MemoryType::ProjectTech,
            MemoryType::Domain,
        ] {
            let memories = repository.browse_by_type(memory_type, usize::MAX)?;
            all_memories.extend(memories);
        }

        println!("📊 Loaded {} memories", all_memories.len());

        // Generate embeddings
        println!("🧮 Generating embeddings (this may take a moment)...");
        let embedding_service = EmbeddingService::new()?;
        let texts: Vec<String> = all_memories
            .iter()
            .map(|m| format!("{} {}", m.title, m.content))
            .collect();
        // Block on the async operation
        let rt = tokio::runtime::Runtime::new()?;
        let embeddings = rt.block_on(embedding_service.embed_texts(texts))?;

        // Find duplicates
        println!(
            "🔍 Searching for duplicates with threshold {:.2}...",
            self.similarity_threshold
        );
        let mut duplicate_count = 0;
        let mut duplicate_pairs = Vec::new();

        for i in 0..all_memories.len() {
            for j in (i + 1)..all_memories.len() {
                if all_memories[i].memory_type != all_memories[j].memory_type {
                    continue;
                }

                let similarity =
                    EmbeddingService::cosine_similarity(&embeddings[i], &embeddings[j]);

                if similarity >= self.similarity_threshold {
                    duplicate_count += 1;
                    duplicate_pairs.push((i, j, similarity));

                    if self.verbose && duplicate_pairs.len() <= 10 {
                        println!("\n  Found duplicate (similarity: {:.2}):", similarity);
                        println!("    Memory 1: {}", all_memories[i].title);
                        println!("    Memory 2: {}", all_memories[j].title);
                    }
                }
            }
        }

        println!("\n📈 Dry-run Analysis Results:");
        println!("  Total memories: {}", all_memories.len());
        println!("  Duplicate pairs found: {}", duplicate_count);

        if duplicate_count > 0 {
            let reduction = duplicate_count as f64 / all_memories.len() as f64 * 100.0;
            println!("  Potential size reduction: {:.1}%", reduction);
            println!("\nRun without --dry-run to perform the actual reindex and merge duplicates.");
        } else {
            println!("  No duplicates found with the current threshold.");
            println!("  Try lowering the similarity threshold to find more potential duplicates.");
        }

        Ok(())
    }
}
