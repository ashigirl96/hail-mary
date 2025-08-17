use crate::commands::memory::common::{FilterCriteria, FilterEngine, FilterStats};
use crate::mcp::server::get_default_db_path;
use crate::memory::{models::MemoryType, repository::SqliteMemoryRepository};
use crate::utils::error::Result;
use clap::Args;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Instant;

/// Bulk update memory properties with comprehensive filtering and atomic operations
#[derive(Args)]
pub struct BulkUpdateCommand {
    /// Path to the database file (defaults to ~/.local/share/hail-mary/memory.db)
    #[arg(long, value_name = "PATH")]
    pub db_path: Option<PathBuf>,

    // Filtering options (same as bulk delete for consistency)
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

    // Update operations (can specify multiple)
    /// Add tags to selected memories (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub add_tags: Option<Vec<String>>,

    /// Remove tags from selected memories (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub remove_tags: Option<Vec<String>>,

    /// Set confidence score for selected memories (0.0 to 1.0)
    #[arg(long, value_name = "SCORE")]
    pub set_confidence: Option<f32>,

    /// Change memory type for selected memories
    #[arg(long, value_enum)]
    pub set_type: Option<MemoryType>,

    /// Set source for selected memories
    #[arg(long, value_name = "SOURCE")]
    pub set_source: Option<String>,

    // Safety options
    /// Dry run - show what would be updated without making changes
    #[arg(long)]
    pub dry_run: bool,

    /// Skip confirmation prompt
    #[arg(long, short)]
    pub yes: bool,

    /// Create backup before updates
    #[arg(long)]
    pub backup: bool,

    /// Backup file path (used with --backup)
    #[arg(long, value_name = "PATH")]
    pub backup_path: Option<PathBuf>,

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

/// Summary of update operations to be performed
#[derive(Debug, Default)]
pub struct UpdateOperations {
    pub add_tags: Option<Vec<String>>,
    pub remove_tags: Option<Vec<String>>,
    pub set_confidence: Option<f32>,
    pub set_type: Option<MemoryType>,
    pub set_source: Option<String>,
}

impl UpdateOperations {
    /// Check if any update operations are specified
    pub fn has_operations(&self) -> bool {
        self.add_tags.is_some()
            || self.remove_tags.is_some()
            || self.set_confidence.is_some()
            || self.set_type.is_some()
            || self.set_source.is_some()
    }

    /// Get a human-readable description of the operations
    pub fn describe(&self) -> Vec<String> {
        let mut operations = Vec::new();

        if let Some(ref tags) = self.add_tags {
            operations.push(format!("Add tags: {}", tags.join(", ")));
        }

        if let Some(ref tags) = self.remove_tags {
            operations.push(format!("Remove tags: {}", tags.join(", ")));
        }

        if let Some(confidence) = self.set_confidence {
            operations.push(format!("Set confidence: {:.2}", confidence));
        }

        if let Some(ref memory_type) = self.set_type {
            operations.push(format!("Set type: {}", memory_type));
        }

        if let Some(ref source) = self.set_source {
            operations.push(format!("Set source: {}", source));
        }

        operations
    }
}

impl BulkUpdateCommand {
    /// Execute the bulk update command
    pub fn execute(self) -> Result<()> {
        // Build update operations
        let operations = self.build_update_operations()?;

        // Validate that at least one operation is specified
        if !operations.has_operations() {
            eprintln!("Error: No update operations specified.");
            eprintln!(
                "Use --add-tags, --remove-tags, --set-confidence, --set-type, or --set-source"
            );
            return Ok(());
        }

        // Build filter criteria from command arguments
        let criteria = self.build_filter_criteria()?;

        // Validate criteria
        criteria.validate()?;

        if self.verbose && !self.quiet {
            println!("🔄 Bulk Update Configuration:");
            println!("  Filters: {}", FilterEngine::describe_criteria(&criteria));
            println!("  Operations:");
            for op in operations.describe() {
                println!("    - {}", op);
            }
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

        // Dry run mode - show what would be updated
        if self.dry_run {
            self.show_dry_run_preview(&target_memories, &operations)?;
            return Ok(());
        }

        // Create backup if requested
        if self.backup {
            self.create_backup(&target_memories)?;
        }

        // Safety confirmation (unless --yes flag is used)
        if !self.yes && !self.quiet {
            self.confirm_updates(&target_memories, &operations)?;
        }

        // Perform the actual updates
        let mut repository = SqliteMemoryRepository::new(&db_path)?;
        self.perform_updates(&mut repository, &target_memories, &operations)?;

        Ok(())
    }

    /// Build UpdateOperations from command arguments
    fn build_update_operations(&self) -> Result<UpdateOperations> {
        let mut operations = UpdateOperations::default();

        if let Some(ref tags) = self.add_tags {
            operations.add_tags = Some(tags.clone());
        }

        if let Some(ref tags) = self.remove_tags {
            operations.remove_tags = Some(tags.clone());
        }

        if let Some(confidence) = self.set_confidence {
            if !(0.0..=1.0).contains(&confidence) {
                return Err(crate::utils::error::HailMaryError::General(
                    anyhow::anyhow!("Confidence score must be between 0.0 and 1.0"),
                ));
            }
            operations.set_confidence = Some(confidence);
        }

        if let Some(ref memory_type) = self.set_type {
            operations.set_type = Some(memory_type.clone());
        }

        if let Some(ref source) = self.set_source {
            operations.set_source = Some(source.clone());
        }

        Ok(operations)
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
    fn show_dry_run_preview(
        &self,
        memories: &[crate::memory::models::Memory],
        operations: &UpdateOperations,
    ) -> Result<()> {
        if !self.quiet {
            println!("🔍 Dry Run - No changes will be made");
            println!("Would update {} memories:", memories.len());
            println!();

            println!("📝 Planned Operations:");
            for op in operations.describe() {
                println!("  - {}", op);
            }
            println!();

            if self.verbose && memories.len() <= 20 {
                for (i, memory) in memories.iter().enumerate() {
                    println!("{}. {} [{}]", i + 1, memory.topic, memory.memory_type);
                    if !memory.tags.is_empty() {
                        println!("   Current tags: {}", memory.tags.join(", "));
                    }
                    println!("   Current confidence: {:.2}", memory.confidence);
                    if let Some(ref source) = memory.source {
                        println!("   Current source: {}", source);
                    }
                    println!("   ID: {}", memory.id);

                    // Show what the updates would result in
                    self.show_preview_changes(memory, operations)?;
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

            if self.stats {
                let stats = FilterStats::from_memories(memories);
                println!();
                stats.display();
            }
        }

        Ok(())
    }

    /// Show preview of changes for a single memory
    fn show_preview_changes(
        &self,
        memory: &crate::memory::models::Memory,
        operations: &UpdateOperations,
    ) -> Result<()> {
        if let Some(ref add_tags) = operations.add_tags {
            let mut new_tags = memory.tags.clone();
            for tag in add_tags {
                if !new_tags.contains(tag) {
                    new_tags.push(tag.clone());
                }
            }
            println!("   → Tags after add: {}", new_tags.join(", "));
        }

        if let Some(ref remove_tags) = operations.remove_tags {
            let new_tags: Vec<String> = memory
                .tags
                .iter()
                .filter(|tag| !remove_tags.contains(tag))
                .cloned()
                .collect();
            println!("   → Tags after remove: {}", new_tags.join(", "));
        }

        if let Some(confidence) = operations.set_confidence {
            println!(
                "   → Confidence: {:.2} → {:.2}",
                memory.confidence, confidence
            );
        }

        if let Some(ref memory_type) = operations.set_type {
            println!("   → Type: {} → {}", memory.memory_type, memory_type);
        }

        if let Some(ref source) = operations.set_source {
            let current = memory.source.as_deref().unwrap_or("(none)");
            println!("   → Source: {} → {}", current, source);
        }

        Ok(())
    }

    /// Create backup before updates
    fn create_backup(&self, memories: &[crate::memory::models::Memory]) -> Result<()> {
        let backup_path = if let Some(ref path) = self.backup_path {
            path.clone()
        } else {
            PathBuf::from(format!(
                "memory_update_backup_{}.json",
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

    /// Confirm updates with user
    fn confirm_updates(
        &self,
        memories: &[crate::memory::models::Memory],
        operations: &UpdateOperations,
    ) -> Result<()> {
        println!(
            "⚠️  This will update {} memories with the following operations:",
            memories.len()
        );

        for op in operations.describe() {
            println!("  - {}", op);
        }

        print!("\nContinue? [y/N]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().to_lowercase().starts_with('y') {
            println!("Operation cancelled.");
            std::process::exit(0);
        }

        Ok(())
    }

    /// Perform the actual updates with atomic transactions
    fn perform_updates(
        &self,
        repository: &mut SqliteMemoryRepository,
        memories: &[crate::memory::models::Memory],
        operations: &UpdateOperations,
    ) -> Result<()> {
        let start_time = Instant::now();
        let memory_ids: Vec<String> = memories.iter().map(|m| m.id.clone()).collect();

        if !self.quiet {
            println!("🔄 Updating {} memories...", memories.len());
        }

        let mut total_affected = 0;
        let mut operation_count = 0;

        // Perform each update operation atomically
        if let Some(ref tags) = operations.add_tags {
            let affected = repository.bulk_add_tags(&memory_ids, tags)?;
            total_affected += affected;
            operation_count += 1;
            if self.verbose && !self.quiet {
                println!("  ✅ Added tags to {} memories", affected);
            }
        }

        if let Some(ref tags) = operations.remove_tags {
            let affected = repository.bulk_remove_tags(&memory_ids, tags)?;
            total_affected += affected;
            operation_count += 1;
            if self.verbose && !self.quiet {
                println!("  ✅ Removed tags from {} memories", affected);
            }
        }

        if let Some(confidence) = operations.set_confidence {
            let affected = repository.bulk_update_confidence(&memory_ids, confidence)?;
            total_affected += affected;
            operation_count += 1;
            if self.verbose && !self.quiet {
                println!("  ✅ Updated confidence for {} memories", affected);
            }
        }

        if let Some(ref memory_type) = operations.set_type {
            let affected = repository.bulk_update_type(&memory_ids, memory_type)?;
            total_affected += affected;
            operation_count += 1;
            if self.verbose && !self.quiet {
                println!("  ✅ Updated type for {} memories", affected);
            }
        }

        if let Some(ref source) = operations.set_source {
            let affected = repository.bulk_update_source(&memory_ids, source)?;
            total_affected += affected;
            operation_count += 1;
            if self.verbose && !self.quiet {
                println!("  ✅ Updated source for {} memories", affected);
            }
        }

        let elapsed = start_time.elapsed();

        if !self.quiet {
            println!(
                "✅ Successfully completed {} operations affecting {} memories in {:?}",
                operation_count,
                memories.len(),
                elapsed
            );
        }

        if self.verbose && !self.quiet {
            println!("📈 Performance:");
            println!("  Memories processed: {}", memories.len());
            println!("  Operations performed: {}", operation_count);
            println!("  Total effects: {}", total_affected);
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
    fn test_build_update_operations() {
        let cmd = BulkUpdateCommand {
            db_path: None,
            r#type: None,
            tags: None,
            min_confidence: None,
            max_age_days: None,
            query: None,
            regex: false,
            case_sensitive: false,
            topic_only: false,
            content_only: false,
            include_deleted: false,
            add_tags: Some(vec!["new".to_string(), "test".to_string()]),
            remove_tags: Some(vec!["old".to_string()]),
            set_confidence: Some(0.9),
            set_type: Some(MemoryType::Domain),
            set_source: Some("test-source".to_string()),
            dry_run: true,
            yes: false,
            backup: false,
            backup_path: None,
            batch_size: 100,
            verbose: false,
            quiet: false,
            stats: false,
        };

        let operations = cmd.build_update_operations().unwrap();

        assert!(operations.has_operations());
        assert_eq!(
            operations.add_tags,
            Some(vec!["new".to_string(), "test".to_string()])
        );
        assert_eq!(operations.remove_tags, Some(vec!["old".to_string()]));
        assert_eq!(operations.set_confidence, Some(0.9));
        assert_eq!(operations.set_type, Some(MemoryType::Domain));
        assert_eq!(operations.set_source, Some("test-source".to_string()));

        let description = operations.describe();
        assert_eq!(description.len(), 5);
        assert!(description[0].contains("Add tags: new, test"));
        assert!(description[1].contains("Remove tags: old"));
        assert!(description[2].contains("Set confidence: 0.90"));
        assert!(description[3].contains("Set type: domain"));
        assert!(description[4].contains("Set source: test-source"));
    }

    #[test]
    fn test_no_operations() {
        let cmd = BulkUpdateCommand {
            db_path: None,
            r#type: None,
            tags: None,
            min_confidence: None,
            max_age_days: None,
            query: None,
            regex: false,
            case_sensitive: false,
            topic_only: false,
            content_only: false,
            include_deleted: false,
            add_tags: None,
            remove_tags: None,
            set_confidence: None,
            set_type: None,
            set_source: None,
            dry_run: true,
            yes: false,
            backup: false,
            backup_path: None,
            batch_size: 100,
            verbose: false,
            quiet: false,
            stats: false,
        };

        let operations = cmd.build_update_operations().unwrap();
        assert!(!operations.has_operations());
        assert!(operations.describe().is_empty());
    }

    #[test]
    fn test_invalid_confidence() {
        let cmd = BulkUpdateCommand {
            db_path: None,
            r#type: None,
            tags: None,
            min_confidence: None,
            max_age_days: None,
            query: None,
            regex: false,
            case_sensitive: false,
            topic_only: false,
            content_only: false,
            include_deleted: false,
            add_tags: None,
            remove_tags: None,
            set_confidence: Some(1.5), // Invalid
            set_type: None,
            set_source: None,
            dry_run: true,
            yes: false,
            backup: false,
            backup_path: None,
            batch_size: 100,
            verbose: false,
            quiet: false,
            stats: false,
        };

        assert!(cmd.build_update_operations().is_err());
    }
}
