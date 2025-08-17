use crate::mcp::server::get_default_db_path;
use crate::memory::{
    models::{Memory, MemoryType},
    repository::{MemoryRepository, SqliteMemoryRepository},
};
use crate::utils::error::Result;
use clap::Args;
use regex::Regex;
use std::path::PathBuf;

/// Advanced search for memories with filtering and sorting options
#[derive(Args)]
pub struct SearchCommand {
    /// Search query (supports FTS5 syntax and regex with --regex flag)
    pub query: String,

    /// Path to the database file (defaults to ~/.local/share/hail-mary/memory.db)
    #[arg(long, value_name = "PATH")]
    pub db_path: Option<PathBuf>,

    /// Filter by memory type
    #[arg(long, value_enum)]
    pub r#type: Option<MemoryType>,

    /// Filter by tags (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub tags: Option<Vec<String>>,

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

    /// Minimum confidence score (0.0 to 1.0)
    #[arg(long, value_name = "SCORE")]
    pub min_confidence: Option<f32>,

    /// Maximum age in days
    #[arg(long, value_name = "DAYS")]
    pub max_age_days: Option<i64>,

    /// Sort by field
    #[arg(long, value_enum, default_value = "relevance")]
    pub sort_by: SortBy,

    /// Sort order
    #[arg(long, value_enum, default_value = "desc")]
    pub sort_order: SortOrder,

    /// Limit number of results
    #[arg(long, short, default_value = "20")]
    pub limit: usize,

    /// Enable verbose output with match details
    #[arg(long, short)]
    pub verbose: bool,

    /// Show only matching snippets (with context)
    #[arg(long)]
    pub snippets: bool,

    /// Include deleted memories in search
    #[arg(long)]
    pub include_deleted: bool,

    /// Use semantic search with embeddings
    #[arg(long)]
    pub semantic: bool,

    /// Minimum similarity score for semantic search (0.0 to 1.0, default 0.7)
    #[arg(long, default_value = "0.7")]
    pub min_similarity: f32,

    /// Combine semantic and keyword search results
    #[arg(long)]
    pub hybrid: bool,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum SortBy {
    Relevance,
    Created,
    Modified,
    Confidence,
    References,
    Topic,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl SearchCommand {
    /// Execute the search command
    pub fn execute(self) -> Result<()> {
        // Validate input
        if let Some(confidence) = self.min_confidence
            && (!(0.0..=1.0).contains(&confidence))
        {
            eprintln!("Error: Confidence score must be between 0.0 and 1.0");
            return Ok(());
        }

        if self.min_similarity < 0.0 || self.min_similarity > 1.0 {
            eprintln!("Error: Similarity score must be between 0.0 and 1.0");
            return Ok(());
        }

        if self.case_sensitive && !self.regex {
            eprintln!("Warning: --case-sensitive has no effect without --regex");
        }

        if self.topic_only && self.content_only {
            eprintln!("Error: Cannot specify both --topic-only and --content-only");
            return Ok(());
        }

        if self.semantic && self.regex {
            eprintln!("Error: Cannot use both --semantic and --regex modes");
            return Ok(());
        }

        if self.hybrid && !self.semantic {
            eprintln!("Warning: --hybrid requires --semantic to be enabled");
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

        if self.verbose {
            println!("🔍 Search Configuration:");
            println!("  Query: '{}'", self.query);
            let mode = if self.semantic {
                if self.hybrid {
                    "Hybrid (Semantic + Keyword)"
                } else {
                    "Semantic"
                }
            } else if self.regex {
                "Regex"
            } else {
                "FTS5"
            };
            println!("  Mode: {}", mode);
            if self.semantic {
                println!("  Min similarity: {:.2}", self.min_similarity);
            }
            if let Some(ref t) = self.r#type {
                println!("  Type filter: {:?}", t);
            }
            if let Some(ref tags) = self.tags {
                println!("  Tag filter: {}", tags.join(", "));
            }
            println!("  Sort: {:?} {:?}", self.sort_by, self.sort_order);
            println!("  Limit: {}", self.limit);
            println!();
        }

        // Perform search based on mode
        if self.semantic {
            self.execute_semantic_search(&db_path)
        } else {
            self.execute_keyword_search(&db_path)
        }
    }

    /// Execute keyword-based search (FTS5 or regex)
    fn execute_keyword_search(&self, db_path: &std::path::Path) -> Result<()> {
        let repository = SqliteMemoryRepository::new(db_path)?;

        // Perform search
        let results = if self.regex {
            self.regex_search(&repository)?
        } else {
            self.fts_search(&repository)?
        };

        // Filter results
        let filtered_results = self.apply_filters(results)?;

        // Sort results
        let sorted_results = self.sort_results(filtered_results);

        // Apply limit
        let final_results: Vec<_> = sorted_results.into_iter().take(self.limit).collect();

        // Display results
        self.display_results(&final_results)?;

        Ok(())
    }

    /// Execute semantic search using embeddings
    fn execute_semantic_search(&self, db_path: &std::path::Path) -> Result<()> {
        use crate::memory::service::MemoryService;

        // Create runtime for async operations
        let runtime = tokio::runtime::Runtime::new()?;

        runtime.block_on(async {
            let repository = SqliteMemoryRepository::new(db_path)?;
            let mut service = MemoryService::with_embeddings(repository)?;

            // Perform semantic search
            let semantic_results = service
                .recall_semantic(
                    &self.query,
                    self.limit * 2, // Get more results for filtering
                    self.min_similarity,
                )
                .await?;

            // Convert to format with optional scores
            let mut results_with_scores: Vec<(Memory, Option<f32>)> = semantic_results
                .into_iter()
                .map(|(m, s)| (m, Some(s)))
                .collect();

            // If hybrid mode, also get keyword results and combine
            if self.hybrid {
                let keyword_repo = SqliteMemoryRepository::new(db_path)?;
                let keyword_results = if self.include_deleted {
                    keyword_repo.search_all(&self.query, self.limit * 2)?
                } else {
                    keyword_repo.search(&self.query, self.limit * 2)?
                };

                // Combine results, prioritizing semantic matches
                let mut seen_ids = std::collections::HashSet::new();

                // Mark semantic results as seen
                for (memory, _) in &results_with_scores {
                    seen_ids.insert(memory.id.clone());
                }

                // Add keyword results that weren't in semantic results
                for memory in keyword_results {
                    if seen_ids.insert(memory.id.clone()) {
                        results_with_scores.push((memory, None));
                    }
                }
            }

            // Extract memories for filtering
            let memories: Vec<Memory> =
                results_with_scores.iter().map(|(m, _)| m.clone()).collect();

            // Apply filters
            let filtered_memories = self.apply_filters(memories)?;

            // Rebuild results with scores
            let mut filtered_results = Vec::new();
            for memory in filtered_memories {
                if let Some((_, score)) =
                    results_with_scores.iter().find(|(m, _)| m.id == memory.id)
                {
                    filtered_results.push((memory, *score));
                }
            }

            // Sort results (semantic search already provides relevance via similarity)
            let sorted_results = if matches!(self.sort_by, SortBy::Relevance) {
                // For relevance, sort by similarity score
                let mut results = filtered_results;
                results.sort_by(|a, b| {
                    let score_a = a.1.unwrap_or(0.0);
                    let score_b = b.1.unwrap_or(0.0);
                    score_b
                        .partial_cmp(&score_a)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                results
            } else {
                // Use regular sorting for other criteria
                let memories = filtered_results.iter().map(|(m, _)| m.clone()).collect();
                let sorted_memories = self.sort_results(memories);
                sorted_memories
                    .into_iter()
                    .map(|m| {
                        let score = filtered_results
                            .iter()
                            .find(|(mem, _)| mem.id == m.id)
                            .and_then(|(_, s)| *s);
                        (m, score)
                    })
                    .collect()
            };

            // Apply limit
            let final_results: Vec<_> = sorted_results.into_iter().take(self.limit).collect();

            // Display results with similarity scores
            self.display_semantic_results(&final_results)?;

            Ok(())
        })
    }

    fn fts_search(&self, repository: &SqliteMemoryRepository) -> Result<Vec<Memory>> {
        // Use FTS5 search
        let memories = if self.include_deleted {
            repository.search_all(&self.query, usize::MAX)?
        } else {
            repository.search(&self.query, usize::MAX)?
        };

        Ok(memories)
    }

    fn regex_search(&self, repository: &SqliteMemoryRepository) -> Result<Vec<Memory>> {
        // Compile regex
        let regex = if self.case_sensitive {
            Regex::new(&self.query)
        } else {
            Regex::new(&format!("(?i){}", self.query))
        };

        let regex = match regex {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error: Invalid regex pattern: {}", e);
                return Ok(Vec::new());
            }
        };

        // Get all memories
        let all_memories = if self.include_deleted {
            repository.browse_all(usize::MAX)?
        } else {
            let mut all = Vec::new();
            for memory_type in &[
                MemoryType::Tech,
                MemoryType::ProjectTech,
                MemoryType::Domain,
            ] {
                let memories = repository.browse_by_type(memory_type, usize::MAX)?;
                all.extend(memories);
            }
            all
        };

        // Filter by regex
        let mut matching = Vec::new();
        for memory in all_memories {
            let matches = if self.topic_only {
                regex.is_match(&memory.topic)
            } else if self.content_only {
                regex.is_match(&memory.content)
            } else {
                regex.is_match(&memory.topic)
                    || regex.is_match(&memory.content)
                    || memory.tags.iter().any(|tag| regex.is_match(tag))
            };

            if matches {
                matching.push(memory);
            }
        }

        Ok(matching)
    }

    fn apply_filters(&self, mut memories: Vec<Memory>) -> Result<Vec<Memory>> {
        // Filter by type
        if let Some(ref filter_type) = self.r#type {
            memories.retain(|m| &m.memory_type == filter_type);
        }

        // Filter by tags
        if let Some(ref filter_tags) = self.tags {
            memories.retain(|m| {
                filter_tags.iter().all(|tag| {
                    m.tags
                        .iter()
                        .any(|mem_tag| mem_tag.to_lowercase().contains(&tag.to_lowercase()))
                })
            });
        }

        // Filter by confidence
        if let Some(min_conf) = self.min_confidence {
            memories.retain(|m| m.confidence >= min_conf);
        }

        // Filter by age
        if let Some(max_age) = self.max_age_days {
            let cutoff_time = chrono::Utc::now().timestamp() - (max_age * 24 * 60 * 60);
            memories.retain(|m| m.created_at >= cutoff_time);
        }

        Ok(memories)
    }

    fn sort_results(&self, mut memories: Vec<Memory>) -> Vec<Memory> {
        memories.sort_by(|a, b| {
            let ordering = match self.sort_by {
                SortBy::Relevance => b.reference_count.cmp(&a.reference_count).then_with(|| {
                    b.confidence
                        .partial_cmp(&a.confidence)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }),
                SortBy::Created => b.created_at.cmp(&a.created_at),
                SortBy::Modified => {
                    let a_modified = a.last_accessed.unwrap_or(a.created_at);
                    let b_modified = b.last_accessed.unwrap_or(b.created_at);
                    b_modified.cmp(&a_modified)
                }
                SortBy::Confidence => b
                    .confidence
                    .partial_cmp(&a.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal),
                SortBy::References => b.reference_count.cmp(&a.reference_count),
                SortBy::Topic => a.topic.cmp(&b.topic),
            };

            match self.sort_order {
                SortOrder::Asc => ordering.reverse(),
                SortOrder::Desc => ordering,
            }
        });

        memories
    }

    fn display_results(&self, memories: &[Memory]) -> Result<()> {
        if memories.is_empty() {
            println!("No memories found matching your search criteria.");
            return Ok(());
        }

        println!(
            "📊 Found {} result{}",
            memories.len(),
            if memories.len() == 1 { "" } else { "s" }
        );
        println!();

        for (i, memory) in memories.iter().enumerate() {
            println!("{}. {} [{}]", i + 1, memory.topic, memory.memory_type);

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

            if self.snippets {
                // Show matching snippets with context
                self.show_snippets(memory)?;
            } else if self.verbose {
                // Show full content in verbose mode
                let content = if memory.content.len() > 200 {
                    format!("{}...", &memory.content[..200])
                } else {
                    memory.content.clone()
                };
                println!("   Content: {}", content);
            } else {
                // Show short preview
                let content = if memory.content.len() > 100 {
                    format!("{}...", &memory.content[..100])
                } else {
                    memory.content.clone()
                };
                println!("   Preview: {}", content);
            }

            if !memory.examples.is_empty() && self.verbose {
                println!("   Examples: {}", memory.examples.len());
            }

            println!();
        }

        // Summary
        if memories.len() > 1 {
            let avg_confidence: f32 =
                memories.iter().map(|m| m.confidence).sum::<f32>() / memories.len() as f32;
            let total_refs: u32 = memories.iter().map(|m| m.reference_count).sum();

            println!("📈 Summary:");
            println!("  Average confidence: {:.2}", avg_confidence);
            println!("  Total references: {}", total_refs);

            // Type distribution
            let mut type_counts = std::collections::HashMap::new();
            for memory in memories {
                *type_counts.entry(&memory.memory_type).or_insert(0) += 1;
            }
            println!("  Type distribution:");
            for (mem_type, count) in &type_counts {
                println!("    {}: {}", mem_type, count);
            }
        }

        Ok(())
    }

    /// Display semantic search results with similarity scores
    fn display_semantic_results(&self, results: &[(Memory, Option<f32>)]) -> Result<()> {
        if results.is_empty() {
            println!("No memories found matching your search criteria.");
            return Ok(());
        }

        let mode_str = if self.hybrid { "Hybrid" } else { "Semantic" };
        println!(
            "📊 {} Search: Found {} result{}",
            mode_str,
            results.len(),
            if results.len() == 1 { "" } else { "s" }
        );
        println!();

        for (i, (memory, score)) in results.iter().enumerate() {
            let similarity_str = if let Some(s) = score {
                format!(" [Similarity: {:.2}]", s)
            } else {
                " [Keyword match]".to_string()
            };

            println!(
                "{}. {} [{}]{}",
                i + 1,
                memory.topic,
                memory.memory_type,
                similarity_str
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

            if self.snippets {
                // Show matching snippets with context
                self.show_snippets(memory)?;
            } else if self.verbose {
                // Show full content in verbose mode
                let content = if memory.content.len() > 200 {
                    format!("{}...", &memory.content[..200])
                } else {
                    memory.content.clone()
                };
                println!("   Content: {}", content);
            } else {
                // Show short preview
                let content = if memory.content.len() > 100 {
                    format!("{}...", &memory.content[..100])
                } else {
                    memory.content.clone()
                };
                println!("   Preview: {}", content);
            }

            if !memory.examples.is_empty() && self.verbose {
                println!("   Examples: {}", memory.examples.len());
            }

            println!();
        }

        // Summary with similarity statistics
        if results.len() > 1 {
            let avg_confidence: f32 =
                results.iter().map(|(m, _)| m.confidence).sum::<f32>() / results.len() as f32;
            let total_refs: u32 = results.iter().map(|(m, _)| m.reference_count).sum();

            // Calculate average similarity for semantic results
            let semantic_results: Vec<f32> = results.iter().filter_map(|(_, s)| *s).collect();

            println!("📈 Summary:");
            if !semantic_results.is_empty() {
                let avg_similarity =
                    semantic_results.iter().sum::<f32>() / semantic_results.len() as f32;
                println!("  Average similarity: {:.2}", avg_similarity);
                println!("  Semantic matches: {}", semantic_results.len());
                if self.hybrid {
                    println!(
                        "  Keyword-only matches: {}",
                        results.len() - semantic_results.len()
                    );
                }
            }
            println!("  Average confidence: {:.2}", avg_confidence);
            println!("  Total references: {}", total_refs);

            // Type distribution
            let mut type_counts = std::collections::HashMap::new();
            for (memory, _) in results {
                *type_counts.entry(&memory.memory_type).or_insert(0) += 1;
            }
            println!("  Type distribution:");
            for (mem_type, count) in &type_counts {
                println!("    {}: {}", mem_type, count);
            }
        }

        Ok(())
    }

    fn show_snippets(&self, memory: &Memory) -> Result<()> {
        if self.regex {
            self.show_regex_snippets(memory)
        } else {
            // For FTS5, just show content preview since we don't have match positions
            let content = if memory.content.len() > 150 {
                format!("{}...", &memory.content[..150])
            } else {
                memory.content.clone()
            };
            println!("   Snippet: {}", content);
            Ok(())
        }
    }

    fn show_regex_snippets(&self, memory: &Memory) -> Result<()> {
        let regex = if self.case_sensitive {
            Regex::new(&self.query)
        } else {
            Regex::new(&format!("(?i){}", self.query))
        };

        let regex = match regex {
            Ok(r) => r,
            Err(_) => return Ok(()), // Skip if regex is invalid
        };

        let context_chars = 50;
        let mut shown_snippets = 0;
        const MAX_SNIPPETS: usize = 3;

        // Check topic
        if regex.is_match(&memory.topic) && !self.content_only {
            println!("   Topic match: {}", memory.topic);
            shown_snippets += 1;
        }

        // Check content
        if !self.topic_only && shown_snippets < MAX_SNIPPETS {
            for mat in regex.find_iter(&memory.content) {
                if shown_snippets >= MAX_SNIPPETS {
                    break;
                }

                let start = mat.start().saturating_sub(context_chars);
                let end = std::cmp::min(mat.end() + context_chars, memory.content.len());
                let snippet = &memory.content[start..end];

                let prefix = if start > 0 { "..." } else { "" };
                let suffix = if end < memory.content.len() {
                    "..."
                } else {
                    ""
                };

                println!("   Content match: {}{}{}", prefix, snippet, suffix);
                shown_snippets += 1;
            }
        }

        // Check tags
        if !self.topic_only && !self.content_only && shown_snippets < MAX_SNIPPETS {
            for tag in &memory.tags {
                if regex.is_match(tag) {
                    println!("   Tag match: {}", tag);
                    shown_snippets += 1;
                    if shown_snippets >= MAX_SNIPPETS {
                        break;
                    }
                }
            }
        }

        Ok(())
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
    // tempdir removed - not used in tests

    #[test]
    fn test_search_command_validation() {
        let cmd = SearchCommand {
            query: "test".to_string(),
            db_path: None,
            r#type: None,
            tags: None,
            regex: false,
            case_sensitive: false,
            topic_only: false,
            content_only: false,
            min_confidence: Some(1.5), // Invalid
            max_age_days: None,
            sort_by: SortBy::Relevance,
            sort_order: SortOrder::Desc,
            limit: 10,
            verbose: false,
            snippets: false,
            include_deleted: false,
            semantic: false,
            min_similarity: 0.7,
            hybrid: false,
        };

        // Should handle invalid confidence gracefully
        assert!(cmd.execute().is_ok());
    }

    #[test]
    fn test_sort_by_enum() {
        let sort_options = [
            SortBy::Relevance,
            SortBy::Created,
            SortBy::Modified,
            SortBy::Confidence,
            SortBy::References,
            SortBy::Topic,
        ];

        // Just ensure all enum variants are valid
        assert_eq!(sort_options.len(), 6);
    }
}
