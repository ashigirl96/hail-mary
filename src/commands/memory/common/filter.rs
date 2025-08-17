use crate::memory::{
    models::{Memory, MemoryType},
    repository::{MemoryRepository, SqliteMemoryRepository},
};
use crate::utils::error::Result;
use regex::Regex;
use std::collections::HashMap;

/// Shared filtering criteria for search, export, and bulk operations
#[derive(Debug, Clone, Default)]
pub struct FilterCriteria {
    /// Filter by memory type
    pub memory_type: Option<MemoryType>,

    /// Filter by tags (all must match)
    pub tags: Option<Vec<String>>,

    /// Minimum confidence score (0.0 to 1.0)
    pub min_confidence: Option<f32>,

    /// Maximum age in days
    pub max_age_days: Option<i64>,

    /// Include deleted memories in results
    pub include_deleted: bool,

    /// Search query for content/topic filtering
    pub query: Option<String>,

    /// Use regex search instead of FTS5
    pub regex: bool,

    /// Case-sensitive search (only with regex)
    pub case_sensitive: bool,

    /// Search only in title field
    pub title_only: bool,

    /// Search only in content field
    pub content_only: bool,
}

impl FilterCriteria {
    /// Create new empty filter criteria
    pub fn new() -> Self {
        Self::default()
    }

    /// Add memory type filter
    pub fn with_type(mut self, memory_type: MemoryType) -> Self {
        self.memory_type = Some(memory_type);
        self
    }

    /// Add tags filter
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// Add confidence filter
    pub fn with_min_confidence(mut self, confidence: f32) -> Self {
        self.min_confidence = Some(confidence);
        self
    }

    /// Add age filter
    pub fn with_max_age_days(mut self, days: i64) -> Self {
        self.max_age_days = Some(days);
        self
    }

    /// Include deleted memories
    pub fn include_deleted(mut self) -> Self {
        self.include_deleted = true;
        self
    }

    /// Add query filter
    pub fn with_query(mut self, query: String, regex: bool, case_sensitive: bool) -> Self {
        self.query = Some(query);
        self.regex = regex;
        self.case_sensitive = case_sensitive;
        self
    }

    /// Validate the filter criteria
    pub fn validate(&self) -> Result<()> {
        if let Some(confidence) = self.min_confidence
            && (!(0.0..=1.0).contains(&confidence))
        {
            return Err(crate::utils::error::HailMaryError::General(
                anyhow::anyhow!("Confidence score must be between 0.0 and 1.0"),
            ));
        }

        if self.title_only && self.content_only {
            return Err(crate::utils::error::HailMaryError::General(
                anyhow::anyhow!("Cannot specify both topic-only and content-only"),
            ));
        }

        // Validate regex if provided
        if let Some(ref query) = self.query
            && self.regex
        {
            let regex_pattern = if self.case_sensitive {
                query.clone()
            } else {
                format!("(?i){}", query)
            };

            if let Err(e) = Regex::new(&regex_pattern) {
                return Err(crate::utils::error::HailMaryError::General(
                    anyhow::anyhow!("Invalid regex pattern: {}", e),
                ));
            }
        }

        Ok(())
    }
}

/// Filter engine for applying criteria to memory collections
pub struct FilterEngine;

impl FilterEngine {
    /// Apply filters to a collection of memories
    pub fn apply_filters(
        criteria: &FilterCriteria,
        mut memories: Vec<Memory>,
    ) -> Result<Vec<Memory>> {
        // Filter by type
        if let Some(ref filter_type) = criteria.memory_type {
            memories.retain(|m| &m.memory_type == filter_type);
        }

        // Filter by tags
        if let Some(ref filter_tags) = criteria.tags {
            memories.retain(|m| {
                filter_tags.iter().all(|tag| {
                    m.tags
                        .iter()
                        .any(|mem_tag| mem_tag.to_lowercase().contains(&tag.to_lowercase()))
                })
            });
        }

        // Filter by confidence
        if let Some(min_conf) = criteria.min_confidence {
            memories.retain(|m| m.confidence >= min_conf);
        }

        // Filter by age
        if let Some(max_age) = criteria.max_age_days {
            let cutoff_time = chrono::Utc::now().timestamp() - (max_age * 24 * 60 * 60);
            memories.retain(|m| m.created_at >= cutoff_time);
        }

        // Filter by deleted status
        if !criteria.include_deleted {
            memories.retain(|m| !m.deleted);
        }

        // Apply query filter if provided
        if let Some(ref query) = criteria.query
            && criteria.regex
        {
            memories = Self::apply_regex_filter(criteria, memories, query)?;
        }
        // Note: FTS5 filtering would be handled at repository level

        Ok(memories)
    }

    /// Apply regex filtering
    fn apply_regex_filter(
        criteria: &FilterCriteria,
        memories: Vec<Memory>,
        query: &str,
    ) -> Result<Vec<Memory>> {
        let regex_pattern = if criteria.case_sensitive {
            query.to_string()
        } else {
            format!("(?i){}", query)
        };

        let regex = Regex::new(&regex_pattern)?;

        let filtered: Vec<Memory> = memories
            .into_iter()
            .filter(|memory| {
                if criteria.title_only {
                    regex.is_match(&memory.title)
                } else if criteria.content_only {
                    regex.is_match(&memory.content)
                } else {
                    regex.is_match(&memory.title)
                        || regex.is_match(&memory.content)
                        || memory.tags.iter().any(|tag| regex.is_match(tag))
                }
            })
            .collect();

        Ok(filtered)
    }

    /// Load memories from repository based on criteria
    pub fn load_memories(
        repository: &SqliteMemoryRepository,
        criteria: &FilterCriteria,
        limit: usize,
    ) -> Result<Vec<Memory>> {
        let memories = if let Some(ref query) = criteria.query {
            if !criteria.regex {
                // Use FTS5 search
                if criteria.include_deleted {
                    repository.search_all(query, limit)?
                } else {
                    repository.search(query, limit)?
                }
            } else {
                // Load all for regex filtering
                if criteria.include_deleted {
                    repository.browse_all(limit)?
                } else {
                    Self::load_all_non_deleted(repository, limit)?
                }
            }
        } else if let Some(ref memory_type) = criteria.memory_type {
            // Load specific type
            repository.browse_by_type(memory_type, limit)?
        } else {
            // Load all
            if criteria.include_deleted {
                repository.browse_all(limit)?
            } else {
                Self::load_all_non_deleted(repository, limit)?
            }
        };

        // Apply additional filters
        Self::apply_filters(criteria, memories)
    }

    /// Load all non-deleted memories across all types
    fn load_all_non_deleted(
        repository: &SqliteMemoryRepository,
        limit: usize,
    ) -> Result<Vec<Memory>> {
        let mut all = Vec::new();
        for memory_type in &[
            MemoryType::Tech,
            MemoryType::ProjectTech,
            MemoryType::Domain,
        ] {
            let memories = repository.browse_by_type(memory_type, limit)?;
            all.extend(memories);
        }
        Ok(all)
    }

    /// Generate a human-readable description of the filter criteria
    pub fn describe_criteria(criteria: &FilterCriteria) -> String {
        let mut parts = Vec::new();

        if let Some(ref memory_type) = criteria.memory_type {
            parts.push(format!("type: {}", memory_type));
        }

        if let Some(ref tags) = criteria.tags {
            parts.push(format!("tags: {}", tags.join(", ")));
        }

        if let Some(confidence) = criteria.min_confidence {
            parts.push(format!("confidence >= {:.2}", confidence));
        }

        if let Some(age) = criteria.max_age_days {
            parts.push(format!("age <= {} days", age));
        }

        if criteria.include_deleted {
            parts.push("including deleted".to_string());
        }

        if let Some(ref query) = criteria.query {
            let search_type = if criteria.regex { "regex" } else { "FTS5" };
            parts.push(format!("query: '{}' ({})", query, search_type));
        }

        if parts.is_empty() {
            "no filters".to_string()
        } else {
            parts.join(", ")
        }
    }

    /// Count memories that would match the criteria
    #[allow(dead_code)] // Reserved for future bulk operation previews
    pub fn count_matches(
        repository: &SqliteMemoryRepository,
        criteria: &FilterCriteria,
    ) -> Result<usize> {
        let memories = Self::load_memories(repository, criteria, usize::MAX)?;
        Ok(memories.len())
    }

    /// Get a preview of memories that would be affected
    #[allow(dead_code)] // Reserved for future bulk operation previews
    pub fn preview_matches(
        repository: &SqliteMemoryRepository,
        criteria: &FilterCriteria,
        preview_limit: usize,
    ) -> Result<Vec<Memory>> {
        let memories = Self::load_memories(repository, criteria, preview_limit)?;
        Ok(memories)
    }
}

/// Statistics about filtered memories
#[derive(Debug)]
pub struct FilterStats {
    pub total_count: usize,
    pub type_distribution: HashMap<MemoryType, usize>,
    pub avg_confidence: f32,
    pub total_references: u32,
    pub tag_distribution: HashMap<String, usize>,
}

impl FilterStats {
    /// Generate statistics from a collection of memories
    pub fn from_memories(memories: &[Memory]) -> Self {
        let total_count = memories.len();

        // Type distribution
        let mut type_distribution = HashMap::new();
        for memory in memories {
            *type_distribution
                .entry(memory.memory_type.clone())
                .or_insert(0) += 1;
        }

        // Average confidence
        let avg_confidence = if total_count > 0 {
            memories.iter().map(|m| m.confidence).sum::<f32>() / total_count as f32
        } else {
            0.0
        };

        // Total references
        let total_references = memories.iter().map(|m| m.reference_count).sum();

        // Tag distribution
        let mut tag_distribution = HashMap::new();
        for memory in memories {
            for tag in &memory.tags {
                *tag_distribution.entry(tag.clone()).or_insert(0) += 1;
            }
        }

        Self {
            total_count,
            type_distribution,
            avg_confidence,
            total_references,
            tag_distribution,
        }
    }

    /// Display statistics in a formatted way
    pub fn display(&self) {
        println!("📊 Filter Statistics:");
        println!("  Total memories: {}", self.total_count);

        if self.total_count > 0 {
            println!("  Average confidence: {:.2}", self.avg_confidence);
            println!("  Total references: {}", self.total_references);

            println!("  Type distribution:");
            for (memory_type, count) in &self.type_distribution {
                println!("    {}: {}", memory_type, count);
            }

            if !self.tag_distribution.is_empty() {
                println!("  Most common tags:");
                let mut tag_vec: Vec<_> = self.tag_distribution.iter().collect();
                tag_vec.sort_by(|a, b| b.1.cmp(a.1));
                for (tag, count) in tag_vec.iter().take(10) {
                    println!("    {}: {}", tag, count);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::models::Memory;

    #[test]
    fn test_filter_criteria_builder() {
        let criteria = FilterCriteria::new()
            .with_type(MemoryType::Tech)
            .with_tags(vec!["rust".to_string(), "async".to_string()])
            .with_min_confidence(0.8)
            .include_deleted();

        assert_eq!(criteria.memory_type, Some(MemoryType::Tech));
        assert_eq!(
            criteria.tags,
            Some(vec!["rust".to_string(), "async".to_string()])
        );
        assert_eq!(criteria.min_confidence, Some(0.8));
        assert!(criteria.include_deleted);
    }

    #[test]
    fn test_filter_validation() {
        let mut criteria = FilterCriteria::new();
        criteria.min_confidence = Some(1.5); // Invalid
        assert!(criteria.validate().is_err());

        criteria.min_confidence = Some(0.8); // Valid
        assert!(criteria.validate().is_ok());

        criteria.title_only = true;
        criteria.content_only = true; // Invalid combination
        assert!(criteria.validate().is_err());
    }

    #[test]
    fn test_filter_engine_type_filtering() {
        let memories = vec![
            Memory::new(MemoryType::Tech, "Rust".to_string(), "Content".to_string()),
            Memory::new(
                MemoryType::Domain,
                "Business".to_string(),
                "Content".to_string(),
            ),
        ];

        let criteria = FilterCriteria::new().with_type(MemoryType::Tech);
        let filtered = FilterEngine::apply_filters(&criteria, memories).unwrap();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].memory_type, MemoryType::Tech);
    }

    #[test]
    fn test_filter_stats() {
        let memories = vec![
            Memory::with_tags(
                MemoryType::Tech,
                "Topic1".to_string(),
                "Content".to_string(),
                vec!["rust".to_string()],
            ),
            Memory::with_tags(
                MemoryType::Tech,
                "Topic2".to_string(),
                "Content".to_string(),
                vec!["async".to_string()],
            ),
            Memory::with_tags(
                MemoryType::Domain,
                "Topic3".to_string(),
                "Content".to_string(),
                vec!["rust".to_string()],
            ),
        ];

        let stats = FilterStats::from_memories(&memories);
        assert_eq!(stats.total_count, 3);
        assert_eq!(stats.type_distribution.get(&MemoryType::Tech), Some(&2));
        assert_eq!(stats.type_distribution.get(&MemoryType::Domain), Some(&1));
        assert_eq!(stats.tag_distribution.get("rust"), Some(&2));
        assert_eq!(stats.tag_distribution.get("async"), Some(&1));
    }
}
