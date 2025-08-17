use crate::commands::memory::common::{FilterCriteria, FilterEngine, FilterStats};
use crate::mcp::server::get_default_db_path;
use crate::memory::{models::MemoryType, repository::SqliteMemoryRepository};
use crate::utils::error::Result;
use clap::Args;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Instant;

/// Bulk delete memories with comprehensive filtering and safety features
#[derive(Args)]
pub struct BulkDeleteCommand {
    /// Path to the database file (defaults to ~/.local/share/hail-mary/memory.db)
    #[arg(long, value_name = "PATH")]
    pub db_path: Option<PathBuf>,

    // Filtering options (using FilterCriteria internally)
    /// Filter by memory type
    #[arg(long, value_enum)]
    pub r#type: Option<MemoryType>,

    /// Filter by tags (comma-separated, all must match)
    #[arg(long, value_delimiter = ',')]
    pub tags: Option<Vec<String>>,

    /// Minimum confidence score (0.0 to 1.0)
    #[arg(long, value_name = "SCORE")]
    pub min_confidence: Option<f32>,

    /// Maximum age in days
    #[arg(long, value_name = "DAYS")]
    pub max_age_days: Option<i64>,

    /// Search query for content/topic filtering
    #[arg(long, value_name = "QUERY")]
    pub query: Option<String>,

    /// Use regex search instead of FTS5
    #[arg(long)]
    pub regex: bool,

    /// Case-sensitive search (only with --regex)
    #[arg(long)]
    pub case_sensitive: bool,

    /// Search only in topic field
    #[arg(long)]
    pub topic_only: bool,

    /// Search only in content field
    #[arg(long)]
    pub content_only: bool,

    /// Include deleted memories in operation
    #[arg(long)]
    pub include_deleted: bool,

    // Safety options
    /// Dry run - show what would be deleted without making changes
    #[arg(long)]
    pub dry_run: bool,

    /// Skip confirmation prompt (dangerous!)
    #[arg(long, short)]
    pub yes: bool,

    /// Create backup before deletion
    #[arg(long)]
    pub backup: bool,

    /// Backup file path (used with --backup)
    #[arg(long, value_name = "PATH")]
    pub backup_path: Option<PathBuf>,

    // Delete options
    /// Permanently delete memories (hard delete vs soft delete)
    #[arg(long)]
    pub hard_delete: bool,

    /// Batch size for progress reporting
    #[arg(long, default_value = "100")]
    pub batch_size: usize,

    // Output options
    /// Enable verbose output with detailed progress
    #[arg(long, short)]
    pub verbose: bool,

    /// Suppress all output except errors
    #[arg(long)]
    pub quiet: bool,

    /// Show statistics about selected memories
    #[arg(long)]
    pub stats: bool,
}

impl BulkDeleteCommand {
    /// Execute the bulk delete command
    pub fn execute(self) -> Result<()> {
        // Build filter criteria from command arguments
        let criteria = self.build_filter_criteria()?;

        // Validate criteria
        criteria.validate()?;

        if self.verbose && !self.quiet {
            println!("🗑️  Bulk Delete Configuration:");
            println!("  Filters: {}", FilterEngine::describe_criteria(&criteria));
            println!("  Hard delete: {}", self.hard_delete);
            println!("  Dry run: {}", self.dry_run);
            println!("  Backup: {}", self.backup);
            println!();
        }

        // Determine database path
        let db_path = self
            .db_path
            .clone()
            .unwrap_or_else(|| get_default_db_path().expect("Failed to get default database path"));

        // Check if database exists
        if !db_path.exists() {
            eprintln!("Error: Database not found at {:?}", db_path);
            eprintln!("Please run 'hail-mary memory serve' first to create the database.");
            return Ok(());
        }

        let repository = SqliteMemoryRepository::new(&db_path)?;

        // Load memories that match the criteria
        let start_time = Instant::now();
        let target_memories = FilterEngine::load_memories(&repository, &criteria, usize::MAX)?;
        let load_time = start_time.elapsed();

        if target_memories.is_empty() {
            if !self.quiet {
                println!("No memories found matching the specified criteria.");
            }
            return Ok(());
        }

        if self.verbose && !self.quiet {
            println!(
                "📊 Found {} memories in {:?}",
                target_memories.len(),
                load_time
            );
        }

        // Show statistics if requested
        if self.stats && !self.quiet {
            let stats = FilterStats::from_memories(&target_memories);
            stats.display();
            println!();
        }

        // Dry run mode - show what would be deleted
        if self.dry_run {
            self.show_dry_run_preview(&target_memories)?;
            return Ok(());
        }

        // Create backup if requested
        if self.backup {
            self.create_backup(&target_memories)?;
        }

        // Safety confirmation (unless --yes flag is used)
        if !self.yes && !self.quiet {
            self.confirm_deletion(&target_memories)?;
        }

        // Perform the actual deletion
        let mut repository = SqliteMemoryRepository::new(&db_path)?;
        self.perform_deletion(&mut repository, &target_memories)?;

        Ok(())
    }

    /// Build FilterCriteria from command arguments
    fn build_filter_criteria(&self) -> Result<FilterCriteria> {
        let mut criteria = FilterCriteria::new();

        if let Some(ref memory_type) = self.r#type {
            criteria = criteria.with_type(memory_type.clone());
        }

        if let Some(ref tags) = self.tags {
            criteria = criteria.with_tags(tags.clone());
        }

        if let Some(confidence) = self.min_confidence {
            criteria = criteria.with_min_confidence(confidence);
        }

        if let Some(age_days) = self.max_age_days {
            criteria = criteria.with_max_age_days(age_days);
        }

        if self.include_deleted {
            criteria = criteria.include_deleted();
        }

        if let Some(ref query) = self.query {
            criteria = criteria.with_query(query.clone(), self.regex, self.case_sensitive);
        }

        criteria.topic_only = self.topic_only;
        criteria.content_only = self.content_only;

        Ok(criteria)
    }

    /// Show dry run preview
    fn show_dry_run_preview(&self, memories: &[crate::memory::models::Memory]) -> Result<()> {
        if !self.quiet {
            println!("🔍 Dry Run - No changes will be made");
            println!("Would delete {} memories:", memories.len());
            println!();

            if self.verbose && memories.len() <= 20 {
                for (i, memory) in memories.iter().enumerate() {
                    println!("{}. {} [{}]", i + 1, memory.topic, memory.memory_type);
                    if !memory.tags.is_empty() {
                        println!("   Tags: {}", memory.tags.join(", "));
                    }
                    println!("   ID: {}", memory.id);
                    if memory.deleted {
                        println!("   Status: Already deleted");
                    }
                    println!();
                }
            } else if memories.len() > 20 {
                // Show first 10 and last 10
                for (i, memory) in memories.iter().take(10).enumerate() {
                    println!("{}. {} [{}]", i + 1, memory.topic, memory.memory_type);
                }

                if memories.len() > 20 {
                    println!("   ... {} more memories ...", memories.len() - 20);
                }

                for (i, memory) in memories.iter().skip(memories.len() - 10).enumerate() {
                    println!(
                        "{}. {} [{}]",
                        memories.len() - 10 + i + 1,
                        memory.topic,
                        memory.memory_type
                    );
                }
            }

            println!(
                "Operation: {} delete",
                if self.hard_delete { "Hard" } else { "Soft" }
            );

            if self.stats {
                let stats = FilterStats::from_memories(memories);
                println!();
                stats.display();
            }
        }

        Ok(())
    }

    /// Create backup before deletion
    fn create_backup(&self, memories: &[crate::memory::models::Memory]) -> Result<()> {
        let backup_path = if let Some(ref path) = self.backup_path {
            path.clone()
        } else {
            PathBuf::from(format!(
                "memory_backup_{}.json",
                chrono::Utc::now().format("%Y%m%d_%H%M%S")
            ))
        };

        if self.verbose && !self.quiet {
            println!("📦 Creating backup: {:?}", backup_path);
        }

        let backup_data = serde_json::to_string_pretty(memories)?;
        std::fs::write(&backup_path, backup_data)?;

        if !self.quiet {
            println!("✅ Backup created: {:?}", backup_path);
        }

        Ok(())
    }

    /// Confirm deletion with user
    fn confirm_deletion(&self, memories: &[crate::memory::models::Memory]) -> Result<()> {
        let operation = if self.hard_delete {
            "permanently delete"
        } else {
            "soft delete (mark as deleted)"
        };

        println!(
            "⚠️  WARNING: This will {} {} memories.",
            operation,
            memories.len()
        );

        if self.hard_delete {
            println!("   Hard deletion is IRREVERSIBLE!");
        }

        print!("Continue? [y/N]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().to_lowercase().starts_with('y') {
            println!("Operation cancelled.");
            std::process::exit(0);
        }

        Ok(())
    }

    /// Perform the actual deletion
    fn perform_deletion(
        &self,
        repository: &mut SqliteMemoryRepository,
        memories: &[crate::memory::models::Memory],
    ) -> Result<()> {
        let start_time = Instant::now();
        let memory_ids: Vec<String> = memories.iter().map(|m| m.id.clone()).collect();

        if !self.quiet {
            println!("🗑️  Deleting {} memories...", memories.len());
        }

        let affected = if self.hard_delete {
            repository.bulk_hard_delete(&memory_ids)?
        } else {
            repository.bulk_soft_delete(&memory_ids)?
        };

        let elapsed = start_time.elapsed();

        if !self.quiet {
            println!(
                "✅ Successfully deleted {} memories in {:?}",
                affected, elapsed
            );

            if affected != memories.len() {
                println!(
                    "⚠️  Note: {} memories were not affected (they may have been already deleted or modified)",
                    memories.len() - affected
                );
            }
        }

        if self.verbose && !self.quiet {
            println!("📈 Performance:");
            println!("  Memories processed: {}", memories.len());
            println!("  Memories affected: {}", affected);
            println!("  Processing time: {:?}", elapsed);
            println!(
                "  Rate: {:.1} memories/second",
                memories.len() as f64 / elapsed.as_secs_f64()
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::models::MemoryType;

    #[test]
    fn test_build_filter_criteria() {
        let cmd = BulkDeleteCommand {
            db_path: None,
            r#type: Some(MemoryType::Tech),
            tags: Some(vec!["rust".to_string(), "async".to_string()]),
            min_confidence: Some(0.8),
            max_age_days: Some(30),
            query: Some("test".to_string()),
            regex: true,
            case_sensitive: false,
            topic_only: false,
            content_only: true,
            include_deleted: true,
            dry_run: true,
            yes: false,
            backup: false,
            backup_path: None,
            hard_delete: false,
            batch_size: 100,
            verbose: false,
            quiet: false,
            stats: false,
        };

        let criteria = cmd.build_filter_criteria().unwrap();

        assert_eq!(criteria.memory_type, Some(MemoryType::Tech));
        assert_eq!(
            criteria.tags,
            Some(vec!["rust".to_string(), "async".to_string()])
        );
        assert_eq!(criteria.min_confidence, Some(0.8));
        assert_eq!(criteria.max_age_days, Some(30));
        assert_eq!(criteria.query, Some("test".to_string()));
        assert!(criteria.regex);
        assert!(!criteria.case_sensitive);
        assert!(!criteria.topic_only);
        assert!(criteria.content_only);
        assert!(criteria.include_deleted);
    }

    #[test]
    fn test_validation() {
        let cmd = BulkDeleteCommand {
            db_path: None,
            r#type: None,
            tags: None,
            min_confidence: Some(1.5), // Invalid
            max_age_days: None,
            query: None,
            regex: false,
            case_sensitive: false,
            topic_only: true,
            content_only: true, // Invalid combination
            include_deleted: false,
            dry_run: true,
            yes: false,
            backup: false,
            backup_path: None,
            hard_delete: false,
            batch_size: 100,
            verbose: false,
            quiet: false,
            stats: false,
        };

        let criteria = cmd.build_filter_criteria().unwrap();
        assert!(criteria.validate().is_err());
    }
}
