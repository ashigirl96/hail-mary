use crate::mcp::server::get_default_db_path;
use crate::memory::{
    repository::{MemoryRepository, SqliteMemoryRepository},
    service::MemoryService,
};
use crate::utils::error::Result;
use clap::Args;
use std::path::PathBuf;

/// Find memories related to a specific memory using semantic similarity
#[derive(Args)]
pub struct RelatedCommand {
    /// Memory ID to find related memories for
    pub memory_id: String,

    /// Path to the database file (defaults to ~/.local/share/hail-mary/memory.db)
    #[arg(long, value_name = "PATH")]
    pub db_path: Option<PathBuf>,

    /// Number of related memories to return
    #[arg(long, short, default_value = "5")]
    pub limit: usize,

    /// Minimum similarity score (0.0 to 1.0, default 0.5)
    #[arg(long, default_value = "0.5")]
    pub min_similarity: f32,

    /// Show detailed output
    #[arg(long, short)]
    pub verbose: bool,

    /// Include the similarity score in output
    #[arg(long)]
    pub show_scores: bool,
}

impl RelatedCommand {
    /// Execute the related command
    pub fn execute(self) -> Result<()> {
        // Validate similarity threshold
        if self.min_similarity < 0.0 || self.min_similarity > 1.0 {
            eprintln!("Error: Similarity score must be between 0.0 and 1.0");
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
            // Create service with embeddings
            let repository = SqliteMemoryRepository::new(&db_path)?;
            let mut service = MemoryService::with_embeddings(repository)?;

            // Get the source memory first to display it
            let source_memory = {
                let repo = SqliteMemoryRepository::new(&db_path)?;
                match repo.find_by_id(&self.memory_id) {
                    Ok(Some(memory)) => memory,
                    Ok(None) => {
                        eprintln!("Error: Memory with ID '{}' not found", self.memory_id);
                        return Ok(());
                    }
                    Err(e) => {
                        eprintln!("Error retrieving memory: {}", e);
                        return Ok(());
                    }
                }
            };

            if self.verbose {
                println!("🔍 Finding memories related to:");
                println!("  Topic: {}", source_memory.title);
                println!("  Type: {}", source_memory.memory_type);
                if !source_memory.tags.is_empty() {
                    println!("  Tags: {}", source_memory.tags.join(", "));
                }
                println!();
            }

            // Find related memories
            let related = service
                .find_related(&self.memory_id, self.limit, self.min_similarity)
                .await?;

            if related.is_empty() {
                println!(
                    "No related memories found with similarity >= {:.2}",
                    self.min_similarity
                );
                println!("Try lowering the --min-similarity threshold to find more matches.");
                return Ok(());
            }

            // Display results
            println!(
                "📊 Found {} related memor{}:",
                related.len(),
                if related.len() == 1 { "y" } else { "ies" }
            );
            println!();

            for (i, (memory, similarity)) in related.iter().enumerate() {
                let score_str = if self.show_scores {
                    format!(" [Similarity: {:.2}]", similarity)
                } else {
                    String::new()
                };

                println!(
                    "{}. {} [{}]{}",
                    i + 1,
                    memory.title,
                    memory.memory_type,
                    score_str
                );

                if self.verbose {
                    println!("   ID: {}", memory.id);
                    println!("   Created: {}", format_timestamp(memory.created_at));
                    if let Some(accessed) = memory.last_accessed {
                        println!("   Last accessed: {}", format_timestamp(accessed));
                    }
                    println!(
                        "   References: {}, Confidence: {:.2}",
                        memory.reference_count, memory.confidence
                    );
                }

                if !memory.tags.is_empty() {
                    println!("   Tags: {}", memory.tags.join(", "));
                }

                // Show content preview
                let content = if self.verbose && memory.content.len() > 200 {
                    format!("{}...", &memory.content[..200])
                } else if !self.verbose && memory.content.len() > 100 {
                    format!("{}...", &memory.content[..100])
                } else {
                    memory.content.clone()
                };

                let label = if self.verbose { "Content" } else { "Preview" };
                println!("   {}: {}", label, content);

                if !memory.examples.is_empty() && self.verbose {
                    println!("   Examples: {}", memory.examples.len());
                }

                println!();
            }

            // Summary
            if related.len() > 1 {
                let avg_similarity: f32 =
                    related.iter().map(|(_, s)| *s).sum::<f32>() / related.len() as f32;
                let avg_confidence: f32 =
                    related.iter().map(|(m, _)| m.confidence).sum::<f32>() / related.len() as f32;

                println!("📈 Summary:");
                println!("  Average similarity: {:.2}", avg_similarity);
                println!("  Average confidence: {:.2}", avg_confidence);

                // Type distribution
                let mut type_counts = std::collections::HashMap::new();
                for (memory, _) in &related {
                    *type_counts.entry(&memory.memory_type).or_insert(0) += 1;
                }

                if type_counts.len() > 1 {
                    println!("  Type distribution:");
                    for (mem_type, count) in &type_counts {
                        println!("    {}: {}", mem_type, count);
                    }
                }

                // Common tags
                let mut tag_counts = std::collections::HashMap::new();
                for (memory, _) in &related {
                    for tag in &memory.tags {
                        *tag_counts.entry(tag.clone()).or_insert(0) += 1;
                    }
                }

                if !tag_counts.is_empty() {
                    let mut common_tags: Vec<_> = tag_counts
                        .into_iter()
                        .filter(|(_, count)| *count > 1)
                        .collect();
                    common_tags.sort_by(|a, b| b.1.cmp(&a.1));

                    if !common_tags.is_empty() {
                        println!("  Common tags:");
                        for (tag, count) in common_tags.iter().take(5) {
                            println!("    {}: {} occurrences", tag, count);
                        }
                    }
                }
            }

            Ok(())
        })
    }
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
    fn test_related_command_validation() {
        let cmd = RelatedCommand {
            memory_id: "test-id".to_string(),
            db_path: None,
            limit: 5,
            min_similarity: 1.5, // Invalid
            verbose: false,
            show_scores: true,
        };

        // Should handle invalid similarity gracefully
        assert!(cmd.execute().is_ok());
    }
}
