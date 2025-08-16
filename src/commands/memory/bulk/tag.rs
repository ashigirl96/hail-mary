use crate::mcp::server::get_default_db_path;
use crate::memory::repository::SqliteMemoryRepository;
use crate::utils::error::Result;
use clap::Args;
use std::collections::HashSet;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Instant;

/// Bulk tag operations and management for comprehensive tag administration
#[derive(Args)]
pub struct BulkTagCommand {
    /// Path to the database file (defaults to ~/.local/share/hail-mary/memory.db)
    #[arg(long, value_name = "PATH")]
    pub db_path: Option<PathBuf>,

    // Tag operations (choose one)
    /// List all tags with usage statistics
    #[arg(long)]
    pub list: bool,

    /// Rename a tag across all memories
    #[arg(long, value_names = ["FROM", "TO"], num_args = 2)]
    pub rename: Option<Vec<String>>,

    /// Merge multiple tags into one (first tag is the target)
    #[arg(long, value_delimiter = ',', value_name = "TAGS")]
    pub merge: Option<Vec<String>>,

    /// Remove unused tags (tags not referenced by any memory)
    #[arg(long)]
    pub cleanup_unused: bool,

    /// Normalize tag formatting (trim whitespace, lowercase)
    #[arg(long)]
    pub normalize: bool,

    /// Find and suggest similar tags for merging
    #[arg(long)]
    pub find_similar: bool,

    /// Show detailed analytics about tag usage
    #[arg(long)]
    pub analytics: bool,

    // Filtering options for tag operations
    /// Minimum usage count for tags to be included in operations
    #[arg(long, value_name = "COUNT")]
    pub min_usage: Option<usize>,

    /// Maximum usage count for tags to be included in operations
    #[arg(long, value_name = "COUNT")]
    pub max_usage: Option<usize>,

    /// Filter tags by pattern (supports regex)
    #[arg(long, value_name = "PATTERN")]
    pub pattern: Option<String>,

    /// Use regex for pattern matching
    #[arg(long)]
    pub regex: bool,

    // Safety options
    /// Dry run - show what would be changed without making changes
    #[arg(long)]
    pub dry_run: bool,

    /// Skip confirmation prompt
    #[arg(long, short)]
    pub yes: bool,

    /// Create backup before operations
    #[arg(long)]
    pub backup: bool,

    /// Backup file path (used with --backup)
    #[arg(long, value_name = "PATH")]
    pub backup_path: Option<PathBuf>,

    // Output options
    /// Enable verbose output with detailed progress
    #[arg(long, short)]
    pub verbose: bool,

    /// Suppress all output except errors
    #[arg(long)]
    pub quiet: bool,

    /// Sort tags by usage count (descending)
    #[arg(long)]
    pub sort_by_usage: bool,

    /// Sort tags alphabetically
    #[arg(long)]
    pub sort_alphabetically: bool,

    /// Show only top N tags (for list operation)
    #[arg(long, value_name = "N")]
    pub top: Option<usize>,
}

/// Tag statistics and information
#[derive(Debug, Clone)]
pub struct TagInfo {
    pub name: String,
    pub usage_count: usize,
    pub similar_tags: Vec<String>,
}

impl TagInfo {
    pub fn new(name: String, usage_count: usize) -> Self {
        Self {
            name,
            usage_count,
            similar_tags: Vec::new(),
        }
    }

    pub fn with_similar(mut self, similar: Vec<String>) -> Self {
        self.similar_tags = similar;
        self
    }
}

/// Tag similarity analysis
pub struct TagSimilarityAnalyzer {
    threshold: f64,
}

impl TagSimilarityAnalyzer {
    pub fn new(threshold: f64) -> Self {
        Self { threshold }
    }

    /// Calculate Levenshtein distance between two strings
    pub fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.len();
        let len2 = s2.len();

        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }

        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        for (i, c1) in s1.chars().enumerate() {
            for (j, c2) in s2.chars().enumerate() {
                let cost = if c1 == c2 { 0 } else { 1 };
                matrix[i + 1][j + 1] = (matrix[i][j + 1] + 1)
                    .min(matrix[i + 1][j] + 1)
                    .min(matrix[i][j] + cost);
            }
        }

        matrix[len1][len2]
    }

    /// Calculate similarity ratio (0.0 to 1.0)
    pub fn similarity_ratio(&self, s1: &str, s2: &str) -> f64 {
        let max_len = s1.len().max(s2.len());
        if max_len == 0 {
            return 1.0;
        }

        let distance = self.levenshtein_distance(s1, s2);
        1.0 - (distance as f64 / max_len as f64)
    }

    /// Find similar tags for a given tag
    pub fn find_similar_tags(&self, target: &str, tags: &[String]) -> Vec<String> {
        tags.iter()
            .filter(|tag| *tag != target)
            .filter(|tag| self.similarity_ratio(target, tag) >= self.threshold)
            .cloned()
            .collect()
    }

    /// Group similar tags together
    pub fn group_similar_tags(&self, tags: &[String]) -> Vec<Vec<String>> {
        let mut groups = Vec::new();
        let mut processed = HashSet::new();

        for tag in tags {
            if processed.contains(tag) {
                continue;
            }

            let similar = self.find_similar_tags(tag, tags);
            if !similar.is_empty() {
                let mut group = vec![tag.clone()];
                group.extend(similar.clone());

                // Mark all tags in this group as processed
                for t in &group {
                    processed.insert(t.clone());
                }

                groups.push(group);
            }
        }

        groups
    }
}

impl BulkTagCommand {
    /// Execute the bulk tag command
    pub fn execute(self) -> Result<()> {
        // Validate that exactly one operation is specified
        let operation_count = [
            self.list,
            self.rename.is_some(),
            self.merge.is_some(),
            self.cleanup_unused,
            self.normalize,
            self.find_similar,
            self.analytics,
        ]
        .iter()
        .filter(|&&op| op)
        .count();

        if operation_count == 0 {
            eprintln!("Error: No tag operation specified.");
            eprintln!(
                "Use one of: --list, --rename, --merge, --cleanup-unused, --normalize, --find-similar, --analytics"
            );
            return Ok(());
        }

        if operation_count > 1 {
            eprintln!("Error: Multiple tag operations specified. Please choose only one.");
            return Ok(());
        }

        if self.verbose && !self.quiet {
            println!("🏷️  Bulk Tag Management Configuration:");
            if self.list {
                println!("  Operation: List tags");
            }
            if self.rename.is_some() {
                println!("  Operation: Rename tag");
            }
            if self.merge.is_some() {
                println!("  Operation: Merge tags");
            }
            if self.cleanup_unused {
                println!("  Operation: Cleanup unused tags");
            }
            if self.normalize {
                println!("  Operation: Normalize tags");
            }
            if self.find_similar {
                println!("  Operation: Find similar tags");
            }
            if self.analytics {
                println!("  Operation: Tag analytics");
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

        let mut repository = SqliteMemoryRepository::new(&db_path)?;

        // Execute the appropriate operation
        if self.list {
            self.execute_list(&repository)?;
        } else if let Some(ref rename_args) = self.rename {
            self.execute_rename(&mut repository, &rename_args[0], &rename_args[1])?;
        } else if let Some(ref merge_tags) = self.merge {
            self.execute_merge(&mut repository, merge_tags)?;
        } else if self.cleanup_unused {
            self.execute_cleanup_unused(&mut repository)?;
        } else if self.normalize {
            self.execute_normalize(&mut repository)?;
        } else if self.find_similar {
            self.execute_find_similar(&repository)?;
        } else if self.analytics {
            self.execute_analytics(&repository)?;
        }

        Ok(())
    }

    /// Execute list tags operation
    fn execute_list(&self, repository: &SqliteMemoryRepository) -> Result<()> {
        let start_time = Instant::now();
        let tag_stats = repository.get_tag_stats()?;
        let load_time = start_time.elapsed();

        if tag_stats.is_empty() {
            if !self.quiet {
                println!("No tags found in the database.");
            }
            return Ok(());
        }

        let mut tags: Vec<(String, usize)> = tag_stats.into_iter().collect();

        // Apply filtering
        if let Some(min_usage) = self.min_usage {
            tags.retain(|(_, count)| *count >= min_usage);
        }
        if let Some(max_usage) = self.max_usage {
            tags.retain(|(_, count)| *count <= max_usage);
        }
        if let Some(ref pattern) = self.pattern {
            if self.regex {
                let regex = regex::Regex::new(pattern)?;
                tags.retain(|(tag, _)| regex.is_match(tag));
            } else {
                tags.retain(|(tag, _)| tag.contains(pattern));
            }
        }

        // Apply sorting
        if self.sort_by_usage {
            tags.sort_by(|a, b| b.1.cmp(&a.1)); // Descending by usage
        } else if self.sort_alphabetically {
            tags.sort_by(|a, b| a.0.cmp(&b.0)); // Ascending alphabetically
        } else {
            tags.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0))); // Default: usage desc, then name asc
        }

        // Apply top limit
        if let Some(top_n) = self.top {
            tags.truncate(top_n);
        }

        if !self.quiet {
            println!("📊 Found {} tags in {:?}", tags.len(), load_time);
            println!();

            if tags.is_empty() {
                println!("No tags match the specified criteria.");
                return Ok(());
            }

            let total_usage: usize = tags.iter().map(|(_, count)| count).sum();
            let max_tag_length = tags.iter().map(|(tag, _)| tag.len()).max().unwrap_or(0);

            println!("📝 Tag List (Total usage: {}):", total_usage);
            println!(
                "{:width$} | Usage | Percentage",
                "Tag",
                width = max_tag_length
            );
            println!("{}", "-".repeat(max_tag_length + 20));

            for (tag, count) in &tags {
                let percentage = (*count as f64 / total_usage as f64) * 100.0;
                println!(
                    "{:width$} | {:5} | {:6.1}%",
                    tag,
                    count,
                    percentage,
                    width = max_tag_length
                );
            }

            if self.verbose {
                println!();
                println!("📈 Summary Statistics:");
                println!("  Total unique tags: {}", tags.len());
                println!("  Total tag usages: {}", total_usage);
                println!(
                    "  Average usage per tag: {:.1}",
                    total_usage as f64 / tags.len() as f64
                );
                println!("  Most used tag: {} ({} uses)", tags[0].0, tags[0].1);
                if tags.len() > 1 {
                    println!(
                        "  Least used tag: {} ({} uses)",
                        tags.last().unwrap().0,
                        tags.last().unwrap().1
                    );
                }
            }
        }

        Ok(())
    }

    /// Execute rename tag operation
    fn execute_rename(
        &self,
        repository: &mut SqliteMemoryRepository,
        from: &str,
        to: &str,
    ) -> Result<()> {
        if from == to {
            if !self.quiet {
                println!("Source and target tags are the same. Nothing to do.");
            }
            return Ok(());
        }

        // Check if source tag exists
        let tag_stats = repository.get_tag_stats()?;
        let usage_count = tag_stats.get(from).copied().unwrap_or(0);

        if usage_count == 0 {
            if !self.quiet {
                println!("Tag '{}' not found in the database.", from);
            }
            return Ok(());
        }

        if !self.quiet {
            println!(
                "🔄 Renaming tag '{}' to '{}' ({} usages)",
                from, to, usage_count
            );
        }

        // Dry run mode
        if self.dry_run {
            if !self.quiet {
                println!("🔍 Dry Run - No changes will be made");
                println!(
                    "Would rename tag '{}' to '{}' in {} memories",
                    from, to, usage_count
                );
            }
            return Ok(());
        }

        // Create backup if requested
        if self.backup {
            self.create_tag_backup(repository)?;
        }

        // Safety confirmation (unless --yes flag is used)
        if !self.yes && !self.quiet {
            print!(
                "⚠️  This will rename tag '{}' to '{}' in {} memories. Continue? [y/N]: ",
                from, to, usage_count
            );
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if !input.trim().to_lowercase().starts_with('y') {
                println!("Operation cancelled.");
                return Ok(());
            }
        }

        // Perform the rename
        let start_time = Instant::now();
        let affected = repository.rename_tag(from, to)?;
        let elapsed = start_time.elapsed();

        if !self.quiet {
            println!(
                "✅ Successfully renamed tag '{}' to '{}' affecting {} memories in {:?}",
                from, to, affected, elapsed
            );
        }

        Ok(())
    }

    /// Execute merge tags operation
    fn execute_merge(
        &self,
        repository: &mut SqliteMemoryRepository,
        merge_tags: &[String],
    ) -> Result<()> {
        if merge_tags.len() < 2 {
            eprintln!("Error: Merge operation requires at least 2 tags.");
            return Ok(());
        }

        let target_tag = &merge_tags[0];
        let source_tags = &merge_tags[1..];

        if !self.quiet {
            println!("🔗 Merging tags into '{}':", target_tag);
            for tag in source_tags {
                println!("  - {}", tag);
            }
        }

        // Dry run mode
        if self.dry_run {
            if !self.quiet {
                println!("🔍 Dry Run - No changes will be made");
                println!(
                    "Would merge {} tags into '{}'",
                    source_tags.len(),
                    target_tag
                );
            }
            return Ok(());
        }

        // Create backup if requested
        if self.backup {
            self.create_tag_backup(repository)?;
        }

        // Safety confirmation
        if !self.yes && !self.quiet {
            print!(
                "⚠️  This will merge {} tags into '{}'. Continue? [y/N]: ",
                source_tags.len(),
                target_tag
            );
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if !input.trim().to_lowercase().starts_with('y') {
                println!("Operation cancelled.");
                return Ok(());
            }
        }

        // Perform the merges
        let start_time = Instant::now();
        let mut total_affected = 0;

        for source_tag in source_tags {
            if source_tag != target_tag {
                let affected = repository.rename_tag(source_tag, target_tag)?;
                total_affected += affected;
                if self.verbose && !self.quiet {
                    println!(
                        "  ✅ Merged '{}' into '{}' ({} memories)",
                        source_tag, target_tag, affected
                    );
                }
            }
        }

        let elapsed = start_time.elapsed();

        if !self.quiet {
            println!(
                "✅ Successfully merged {} tags into '{}' affecting {} memories in {:?}",
                source_tags.len(),
                target_tag,
                total_affected,
                elapsed
            );
        }

        Ok(())
    }

    /// Execute cleanup unused tags operation
    fn execute_cleanup_unused(&self, repository: &mut SqliteMemoryRepository) -> Result<()> {
        if !self.quiet {
            println!("🧹 Cleaning up unused tags...");
        }

        // Dry run mode
        if self.dry_run {
            if !self.quiet {
                println!("🔍 Dry Run - No changes will be made");
                println!("Would remove unused tags from the database");
            }
            return Ok(());
        }

        // Create backup if requested
        if self.backup {
            self.create_tag_backup(repository)?;
        }

        // Safety confirmation
        if !self.yes && !self.quiet {
            print!("⚠️  This will remove unused tags. Continue? [y/N]: ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if !input.trim().to_lowercase().starts_with('y') {
                println!("Operation cancelled.");
                return Ok(());
            }
        }

        // Perform cleanup
        let start_time = Instant::now();
        let affected = repository.remove_unused_tags()?;
        let elapsed = start_time.elapsed();

        if !self.quiet {
            println!("✅ Cleaned up {} unused tags in {:?}", affected, elapsed);
        }

        Ok(())
    }

    /// Execute normalize tags operation
    fn execute_normalize(&self, _repository: &mut SqliteMemoryRepository) -> Result<()> {
        if !self.quiet {
            println!("📝 Tag normalization is not yet implemented.");
            println!("This operation would:");
            println!("  - Trim whitespace from tags");
            println!("  - Convert to consistent case");
            println!("  - Remove empty tags");
            println!("  - Deduplicate tags within memories");
        }

        Ok(())
    }

    /// Execute find similar tags operation
    fn execute_find_similar(&self, repository: &SqliteMemoryRepository) -> Result<()> {
        let start_time = Instant::now();
        let tag_stats = repository.get_tag_stats()?;
        let tags: Vec<String> = tag_stats.keys().cloned().collect();
        let load_time = start_time.elapsed();

        if tags.is_empty() {
            if !self.quiet {
                println!("No tags found in the database.");
            }
            return Ok(());
        }

        if !self.quiet {
            println!(
                "🔍 Analyzing {} tags for similarities in {:?}...",
                tags.len(),
                load_time
            );
            println!();
        }

        let analyzer = TagSimilarityAnalyzer::new(0.7); // 70% similarity threshold
        let groups = analyzer.group_similar_tags(&tags);

        if groups.is_empty() {
            if !self.quiet {
                println!("No similar tags found.");
            }
            return Ok(());
        }

        if !self.quiet {
            println!("📊 Found {} groups of similar tags:", groups.len());
            println!();

            for (i, group) in groups.iter().enumerate() {
                println!("Group {}: ({} tags)", i + 1, group.len());
                for tag in group {
                    let usage = tag_stats.get(tag).unwrap_or(&0);
                    println!("  - {} (used {} times)", tag, usage);
                }

                if self.verbose {
                    // Show suggested merge command
                    let merge_command =
                        format!("hail-mary memory bulk-tag --merge {}", group.join(","));
                    println!("  Suggested merge: {}", merge_command);
                }
                println!();
            }

            println!(
                "💡 To merge similar tags, use: hail-mary memory bulk-tag --merge tag1,tag2,tag3"
            );
        }

        Ok(())
    }

    /// Execute analytics operation
    fn execute_analytics(&self, repository: &SqliteMemoryRepository) -> Result<()> {
        let start_time = Instant::now();
        let tag_stats = repository.get_tag_stats()?;
        let load_time = start_time.elapsed();

        if tag_stats.is_empty() {
            if !self.quiet {
                println!("No tags found in the database.");
            }
            return Ok(());
        }

        let total_tags = tag_stats.len();
        let total_usage: usize = tag_stats.values().sum();
        let avg_usage = total_usage as f64 / total_tags as f64;

        let mut usage_counts: Vec<usize> = tag_stats.values().copied().collect();
        usage_counts.sort_by(|a, b| b.cmp(a));

        let median_usage = if usage_counts.len() % 2 == 0 {
            (usage_counts[usage_counts.len() / 2 - 1] + usage_counts[usage_counts.len() / 2]) as f64
                / 2.0
        } else {
            usage_counts[usage_counts.len() / 2] as f64
        };

        let max_usage = usage_counts.first().copied().unwrap_or(0);
        let min_usage = usage_counts.last().copied().unwrap_or(0);

        // Usage distribution analysis
        let single_use_tags = usage_counts.iter().filter(|&&count| count == 1).count();
        let low_use_tags = usage_counts.iter().filter(|&&count| count <= 3).count();
        let high_use_tags = usage_counts.iter().filter(|&&count| count >= 10).count();

        if !self.quiet {
            println!("📊 Tag Analytics Report (Generated in {:?})", load_time);
            println!("{}", "═".repeat(50));
            println!();

            println!("📈 Overview:");
            println!("  Total unique tags: {}", total_tags);
            println!("  Total tag usages: {}", total_usage);
            println!("  Average usage per tag: {:.2}", avg_usage);
            println!("  Median usage: {:.1}", median_usage);
            println!("  Most used tag usage: {}", max_usage);
            println!("  Least used tag usage: {}", min_usage);
            println!();

            println!("📊 Usage Distribution:");
            println!(
                "  Single-use tags: {} ({:.1}%)",
                single_use_tags,
                single_use_tags as f64 / total_tags as f64 * 100.0
            );
            println!(
                "  Low-use tags (≤3): {} ({:.1}%)",
                low_use_tags,
                low_use_tags as f64 / total_tags as f64 * 100.0
            );
            println!(
                "  High-use tags (≥10): {} ({:.1}%)",
                high_use_tags,
                high_use_tags as f64 / total_tags as f64 * 100.0
            );
            println!();

            println!("🏆 Top 10 Most Used Tags:");
            let mut sorted_tags: Vec<_> = tag_stats.iter().collect();
            sorted_tags.sort_by(|a, b| b.1.cmp(a.1));

            for (i, (tag, count)) in sorted_tags.iter().take(10).enumerate() {
                let percentage = **count as f64 / total_usage as f64 * 100.0;
                println!("  {}. {} ({} uses, {:.1}%)", i + 1, tag, count, percentage);
            }

            if self.verbose && sorted_tags.len() > 10 {
                println!();
                println!("🔽 Least Used Tags:");
                for (tag, count) in sorted_tags.iter().rev().take(10) {
                    println!("  {} ({} uses)", tag, count);
                }
            }

            println!();
            println!("💡 Recommendations:");
            if single_use_tags > total_tags / 4 {
                println!("  - Consider reviewing single-use tags for potential merging or removal");
            }
            if high_use_tags < total_tags / 10 {
                println!("  - Consider creating more specific tags for better organization");
            }
            println!("  - Use --find-similar to identify tags that could be merged");
            println!("  - Use --cleanup-unused to remove orphaned tags");
        }

        Ok(())
    }

    /// Create backup of tag data
    fn create_tag_backup(&self, repository: &SqliteMemoryRepository) -> Result<()> {
        let backup_path = if let Some(ref path) = self.backup_path {
            path.clone()
        } else {
            PathBuf::from(format!(
                "tag_backup_{}.json",
                chrono::Utc::now().format("%Y%m%d_%H%M%S")
            ))
        };

        if self.verbose && !self.quiet {
            println!("📦 Creating tag backup: {:?}", backup_path);
        }

        let tag_stats = repository.get_tag_stats()?;
        let backup_data = serde_json::to_string_pretty(&tag_stats)?;
        std::fs::write(&backup_path, backup_data)?;

        if !self.quiet {
            println!("✅ Tag backup created: {:?}", backup_path);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_similarity_analyzer() {
        let analyzer = TagSimilarityAnalyzer::new(0.7);

        // Test exact match
        assert_eq!(analyzer.similarity_ratio("rust", "rust"), 1.0);

        // Test similar strings
        let ratio = analyzer.similarity_ratio("rust", "rust-lang");
        assert!(ratio > 0.5);

        // Test very different strings
        let ratio = analyzer.similarity_ratio("rust", "python");
        assert!(ratio < 0.5);
    }

    #[test]
    fn test_levenshtein_distance() {
        let analyzer = TagSimilarityAnalyzer::new(0.7);

        assert_eq!(analyzer.levenshtein_distance("", ""), 0);
        assert_eq!(analyzer.levenshtein_distance("a", ""), 1);
        assert_eq!(analyzer.levenshtein_distance("", "a"), 1);
        assert_eq!(analyzer.levenshtein_distance("abc", "abc"), 0);
        assert_eq!(analyzer.levenshtein_distance("abc", "ab"), 1);
        assert_eq!(analyzer.levenshtein_distance("abc", "def"), 3);
    }

    #[test]
    fn test_find_similar_tags() {
        let analyzer = TagSimilarityAnalyzer::new(0.7);
        let tags = vec![
            "rust".to_string(),
            "rust-lang".to_string(),
            "python".to_string(),
            "javascript".to_string(),
            "js".to_string(),
        ];

        let similar = analyzer.find_similar_tags("rust", &tags);
        // Should find "rust-lang" as similar to "rust"
        assert!(!similar.is_empty());
    }

    #[test]
    fn test_tag_info() {
        let tag_info = TagInfo::new("rust".to_string(), 5);
        assert_eq!(tag_info.name, "rust");
        assert_eq!(tag_info.usage_count, 5);
        assert!(tag_info.similar_tags.is_empty());

        let tag_info_with_similar = tag_info.with_similar(vec!["rust-lang".to_string()]);
        assert_eq!(tag_info_with_similar.similar_tags.len(), 1);
    }
}
