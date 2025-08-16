use crate::mcp::server::get_default_db_path;
use crate::memory::{
    embeddings::EmbeddingService,
    models::{Memory, MemoryType},
    repository::{MemoryRepository, SqliteMemoryRepository},
};
use crate::utils::error::Result;
use clap::Args;
use std::path::PathBuf;

/// Find and merge duplicate memories using semantic similarity
#[derive(Args)]
pub struct DedupCommand {
    /// Path to the database file (defaults to ~/.local/share/hail-mary/memory.db)
    #[arg(long, value_name = "PATH")]
    pub db_path: Option<PathBuf>,

    /// Similarity threshold for considering memories as duplicates (0.0 to 1.0, default: 0.85)
    #[arg(long, value_name = "THRESHOLD", default_value = "0.85")]
    pub similarity_threshold: f32,

    /// Memory type to deduplicate (if not specified, all types)
    #[arg(long, value_enum)]
    pub r#type: Option<MemoryType>,

    /// Maximum number of duplicates to process
    #[arg(long, default_value = "100")]
    pub limit: usize,

    /// Show detailed output
    #[arg(long, short)]
    pub verbose: bool,

    /// Dry run - show what would be merged without making changes
    #[arg(long)]
    pub dry_run: bool,

    /// Interactive mode - confirm each merge
    #[arg(long, short)]
    pub interactive: bool,

    /// Merge strategy
    #[arg(long, value_enum, default_value = "smart")]
    pub strategy: MergeStrategy,

    /// Backup database before deduplication
    #[arg(long, default_value = "true")]
    pub backup: bool,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum MergeStrategy {
    /// Keep the newer memory and merge content
    Newer,
    /// Keep the older memory and merge content
    Older,
    /// Keep the memory with higher confidence
    Confidence,
    /// Keep the memory with more references
    References,
    /// Smart merge based on multiple factors
    Smart,
}

impl DedupCommand {
    /// Execute the dedup command
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
            eprintln!("Please run 'hail-mary memory serve' first to create the database.");
            return Ok(());
        }

        // Create runtime for async operations
        let runtime = tokio::runtime::Runtime::new()?;

        runtime.block_on(async {
            if self.backup && !self.dry_run {
                self.backup_database(&db_path)?;
            }

            // Find duplicates
            let duplicates = self.find_duplicates(&db_path).await?;

            if duplicates.is_empty() {
                println!(
                    "✅ No duplicate memories found with similarity >= {:.2}",
                    self.similarity_threshold
                );
                return Ok(());
            }

            println!("🔍 Found {} duplicate groups", duplicates.len());

            if self.dry_run {
                self.show_dry_run_results(&duplicates)?;
                return Ok(());
            }

            if self.interactive {
                self.interactive_merge(&db_path, duplicates)?;
            } else {
                self.automatic_merge(&db_path, duplicates)?;
            }

            Ok(())
        })
    }

    /// Find duplicate memories using embeddings
    async fn find_duplicates(&self, db_path: &PathBuf) -> Result<Vec<DuplicateGroup>> {
        let repository = SqliteMemoryRepository::new(db_path)?;
        let embedding_service = EmbeddingService::new()?;

        // Get memories to check
        let memories = if let Some(ref memory_type) = self.r#type {
            repository.browse_by_type(memory_type, self.limit * 10)?
        } else {
            repository.browse_all(self.limit * 10)?
        };

        if self.verbose {
            println!("📊 Analyzing {} memories for duplicates...", memories.len());
        }

        // Generate embeddings for all memories
        let texts: Vec<String> = memories
            .iter()
            .map(|m| format!("{} {}", m.topic, m.content))
            .collect();

        let embeddings = embedding_service.embed_texts(texts).await?;

        // Find duplicate groups
        let mut duplicate_groups = Vec::new();
        let mut processed = std::collections::HashSet::new();

        for i in 0..memories.len() {
            if processed.contains(&i) || duplicate_groups.len() >= self.limit {
                continue;
            }

            let mut group = DuplicateGroup {
                primary_index: i,
                primary_memory: memories[i].clone(),
                duplicates: Vec::new(),
                similarities: Vec::new(),
            };

            for j in (i + 1)..memories.len() {
                if processed.contains(&j) {
                    continue;
                }

                // Only check within same memory type
                if memories[i].memory_type != memories[j].memory_type {
                    continue;
                }

                let similarity =
                    EmbeddingService::cosine_similarity(&embeddings[i], &embeddings[j]);

                if similarity >= self.similarity_threshold {
                    group.duplicates.push(memories[j].clone());
                    group.similarities.push(similarity);
                    processed.insert(j);
                }
            }

            if !group.duplicates.is_empty() {
                processed.insert(i);
                duplicate_groups.push(group);
            }
        }

        // Sort by average similarity
        duplicate_groups.sort_by(|a, b| {
            let avg_a: f32 = a.similarities.iter().sum::<f32>() / a.similarities.len() as f32;
            let avg_b: f32 = b.similarities.iter().sum::<f32>() / b.similarities.len() as f32;
            avg_b.partial_cmp(&avg_a).unwrap()
        });

        Ok(duplicate_groups)
    }

    /// Show dry run results
    fn show_dry_run_results(&self, groups: &[DuplicateGroup]) -> Result<()> {
        println!("\n📋 Dry Run Results:");
        println!("Would merge {} duplicate groups", groups.len());

        for (i, group) in groups.iter().enumerate() {
            println!(
                "\n{}. Duplicate Group [{}]",
                i + 1,
                group.primary_memory.memory_type
            );
            println!(
                "   Primary: {} (ID: {})",
                group.primary_memory.topic,
                &group.primary_memory.id[..8]
            );

            if self.verbose {
                println!(
                    "   Created: {}",
                    format_timestamp(group.primary_memory.created_at)
                );
                println!(
                    "   Confidence: {:.2}, References: {}",
                    group.primary_memory.confidence, group.primary_memory.reference_count
                );
            }

            for (j, (dup, sim)) in group.duplicates.iter().zip(&group.similarities).enumerate() {
                println!(
                    "   Duplicate {}: {} (Similarity: {:.2})",
                    j + 1,
                    dup.topic,
                    sim
                );
                if self.verbose {
                    println!("      ID: {}", &dup.id[..8]);
                    println!("      Created: {}", format_timestamp(dup.created_at));
                }
            }

            println!("   Merge Strategy: {:?}", self.strategy);
            let (keep, _) = self.determine_merge(&group.primary_memory, &group.duplicates[0]);
            println!(
                "   Would keep: {}",
                if keep == 0 { "Primary" } else { "Duplicate" }
            );
        }

        let total_reduction = groups.iter().map(|g| g.duplicates.len()).sum::<usize>();
        println!("\n📈 Summary:");
        println!("  Total memories that would be merged: {}", total_reduction);
        println!(
            "  Estimated database size reduction: {} memories",
            total_reduction
        );

        Ok(())
    }

    /// Interactive merge mode
    fn interactive_merge(&self, db_path: &PathBuf, groups: Vec<DuplicateGroup>) -> Result<()> {
        use std::io::{self, Write};

        let mut repository = SqliteMemoryRepository::new(db_path)?;
        let mut merged_count = 0;
        let mut skipped_count = 0;

        for (i, group) in groups.iter().enumerate() {
            println!(
                "\n{}/{} - Duplicate Group [{}]",
                i + 1,
                groups.len(),
                group.primary_memory.memory_type
            );

            println!("Primary Memory:");
            self.display_memory(&group.primary_memory, "  ");

            for (j, (dup, sim)) in group.duplicates.iter().zip(&group.similarities).enumerate() {
                println!("\nDuplicate {} (Similarity: {:.2}):", j + 1, sim);
                self.display_memory(dup, "  ");
            }

            print!("\nMerge these memories? [y/n/q] (n): ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim().to_lowercase();

            match input.as_str() {
                "y" | "yes" => {
                    self.merge_group(&mut repository, group)?;
                    merged_count += group.duplicates.len();
                    println!("✅ Merged {} memories", group.duplicates.len() + 1);
                }
                "q" | "quit" => {
                    println!("Exiting interactive merge...");
                    break;
                }
                _ => {
                    skipped_count += group.duplicates.len();
                    println!("⏭️  Skipped");
                }
            }
        }

        println!("\n📊 Interactive Merge Complete:");
        println!("  Memories merged: {}", merged_count);
        println!("  Memories skipped: {}", skipped_count);

        Ok(())
    }

    /// Automatic merge mode
    fn automatic_merge(&self, db_path: &PathBuf, groups: Vec<DuplicateGroup>) -> Result<()> {
        let mut repository = SqliteMemoryRepository::new(db_path)?;
        let mut total_merged = 0;

        println!("\n🔄 Starting automatic merge...");

        for (i, group) in groups.iter().enumerate() {
            if self.verbose {
                println!(
                    "  Merging group {}/{}: {}",
                    i + 1,
                    groups.len(),
                    group.primary_memory.topic
                );
            }

            self.merge_group(&mut repository, group)?;
            total_merged += group.duplicates.len();
        }

        println!("\n✅ Automatic Merge Complete:");
        println!("  Groups processed: {}", groups.len());
        println!("  Memories merged: {}", total_merged);

        Ok(())
    }

    /// Merge a duplicate group
    fn merge_group(
        &self,
        repository: &mut SqliteMemoryRepository,
        group: &DuplicateGroup,
    ) -> Result<()> {
        let mut merged_memory = group.primary_memory.clone();

        for duplicate in &group.duplicates {
            let (keep_idx, merged) = self.determine_merge(&merged_memory, duplicate);

            if keep_idx == 0 {
                // Keep primary, merge content from duplicate
                merged_memory = merged;
            } else {
                // Keep duplicate as base, merge content from primary
                merged_memory = merged;
                merged_memory.id = group.primary_memory.id.clone(); // Keep original ID
            }
        }

        // Update the primary memory with merged content
        repository.update(&merged_memory)?;

        // Delete the duplicates using soft delete
        for duplicate in &group.duplicates {
            repository.soft_delete(&duplicate.id)?;
        }

        Ok(())
    }

    /// Determine which memory to keep and how to merge
    pub fn determine_merge(&self, memory1: &Memory, memory2: &Memory) -> (usize, Memory) {
        let mut merged = memory1.clone();
        let keep_idx = match self.strategy {
            MergeStrategy::Newer => {
                if memory2.created_at > memory1.created_at {
                    1
                } else {
                    0
                }
            }
            MergeStrategy::Older => {
                if memory1.created_at > memory2.created_at {
                    1
                } else {
                    0
                }
            }
            MergeStrategy::Confidence => {
                if memory2.confidence > memory1.confidence {
                    1
                } else {
                    0
                }
            }
            MergeStrategy::References => {
                if memory2.reference_count > memory1.reference_count {
                    1
                } else {
                    0
                }
            }
            MergeStrategy::Smart => {
                // Smart strategy: weighted scoring
                let score1 = self.calculate_memory_score(memory1);
                let score2 = self.calculate_memory_score(memory2);
                if score2 > score1 { 1 } else { 0 }
            }
        };

        // Choose base memory
        if keep_idx == 1 {
            merged = memory2.clone();
        }

        // Merge content (always combine unique information)
        if !merged.content.contains(&memory2.content) && !memory2.content.contains(&merged.content)
        {
            merged.content = format!(
                "{}\n\n## Merged Content:\n{}",
                merged.content, memory2.content
            );
        }

        // Merge tags (unique)
        for tag in &memory2.tags {
            if !merged.tags.contains(tag) {
                merged.tags.push(tag.clone());
            }
        }

        // Merge examples (unique)
        for example in &memory2.examples {
            if !merged.examples.contains(example) {
                merged.examples.push(example.clone());
            }
        }

        // Combine metadata
        merged.reference_count = memory1.reference_count + memory2.reference_count;
        merged.confidence = (memory1.confidence + memory2.confidence) / 2.0;

        // Keep earliest creation date
        if memory2.created_at < merged.created_at {
            merged.created_at = memory2.created_at;
        }

        // Keep latest access date
        if let Some(access2) = memory2.last_accessed {
            if let Some(access1) = merged.last_accessed {
                if access2 > access1 {
                    merged.last_accessed = Some(access2);
                }
            } else {
                merged.last_accessed = Some(access2);
            }
        }

        (keep_idx, merged)
    }

    /// Calculate a smart score for a memory
    pub fn calculate_memory_score(&self, memory: &Memory) -> f32 {
        let mut score = 0.0;

        // Confidence weight (30%)
        score += memory.confidence * 0.3;

        // Reference count weight (25%)
        let ref_score = (memory.reference_count as f32 / 10.0).min(1.0);
        score += ref_score * 0.25;

        // Content length weight (15%)
        let content_score = (memory.content.len() as f32 / 500.0).min(1.0);
        score += content_score * 0.15;

        // Tag count weight (10%)
        let tag_score = (memory.tags.len() as f32 / 5.0).min(1.0);
        score += tag_score * 0.1;

        // Example count weight (10%)
        let example_score = (memory.examples.len() as f32 / 3.0).min(1.0);
        score += example_score * 0.1;

        // Recency weight (10%)
        let now = chrono::Utc::now().timestamp();
        let age_days = (now - memory.created_at) / (24 * 60 * 60);
        let recency_score = 1.0 / (1.0 + (age_days as f32 / 30.0)); // Decay over 30 days
        score += recency_score * 0.1;

        score
    }

    /// Display a memory
    fn display_memory(&self, memory: &Memory, indent: &str) {
        println!("{}Topic: {}", indent, memory.topic);
        println!("{}Type: {}", indent, memory.memory_type);
        println!("{}Created: {}", indent, format_timestamp(memory.created_at));
        println!(
            "{}Confidence: {:.2}, References: {}",
            indent, memory.confidence, memory.reference_count
        );

        if !memory.tags.is_empty() {
            println!("{}Tags: {}", indent, memory.tags.join(", "));
        }

        let content_preview = if memory.content.len() > 150 {
            format!("{}...", &memory.content[..150])
        } else {
            memory.content.clone()
        };
        println!("{}Content: {}", indent, content_preview);
    }

    /// Backup the database
    fn backup_database(&self, db_path: &PathBuf) -> Result<()> {
        let backup_dir = db_path.parent().unwrap().join("backups");
        std::fs::create_dir_all(&backup_dir)?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_filename = format!("memory_dedup_backup_{}.db", timestamp);
        let backup_path = backup_dir.join(backup_filename);

        std::fs::copy(db_path, &backup_path)?;
        println!("📦 Database backed up to: {}", backup_path.display());

        Ok(())
    }
}

/// A group of duplicate memories
struct DuplicateGroup {
    primary_index: usize,
    primary_memory: Memory,
    duplicates: Vec<Memory>,
    similarities: Vec<f32>,
}

fn format_timestamp(timestamp: i64) -> String {
    match chrono::DateTime::from_timestamp(timestamp, 0) {
        Some(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        None => "Invalid timestamp".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_strategy() {
        use crate::memory::models::Memory;

        let mut memory1 = Memory::new(
            MemoryType::Tech,
            "Test Memory 1".to_string(),
            "Content 1".to_string(),
        );
        memory1.confidence = 0.8;
        memory1.reference_count = 5;
        memory1.created_at = 1000;

        let mut memory2 = Memory::new(
            MemoryType::Tech,
            "Test Memory 2".to_string(),
            "Content 2".to_string(),
        );
        memory2.confidence = 0.9;
        memory2.reference_count = 3;
        memory2.created_at = 2000;

        let cmd = DedupCommand {
            db_path: None,
            similarity_threshold: 0.85,
            r#type: None,
            limit: 100,
            verbose: false,
            dry_run: true,
            interactive: false,
            strategy: MergeStrategy::Confidence,
            backup: true,
        };

        let (keep_idx, merged) = cmd.determine_merge(&memory1, &memory2);
        assert_eq!(keep_idx, 1); // Should keep memory2 (higher confidence)
        assert!(merged.content.contains("Content 2"));
    }

    #[test]
    fn test_memory_score_calculation() {
        use crate::memory::models::Memory;

        let mut memory = Memory::new(
            MemoryType::Tech,
            "Test Memory".to_string(),
            "A".repeat(300), // 300 chars
        );
        memory.confidence = 0.9;
        memory.reference_count = 8;
        memory.tags = vec!["tag1".to_string(), "tag2".to_string()];
        memory.examples = vec!["ex1".to_string()];

        let cmd = DedupCommand {
            db_path: None,
            similarity_threshold: 0.85,
            r#type: None,
            limit: 100,
            verbose: false,
            dry_run: true,
            interactive: false,
            strategy: MergeStrategy::Smart,
            backup: true,
        };

        let score = cmd.calculate_memory_score(&memory);
        assert!(score > 0.0 && score <= 1.0);
    }
}
