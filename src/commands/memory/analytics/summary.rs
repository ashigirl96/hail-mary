use super::{AnalyticsContext, AnalyticsMetric, MetricValue};
use crate::memory::repository::MemoryRepository;
use crate::utils::error::Result;
use clap::Args;
use std::collections::HashMap;
use std::convert::TryInto;

/// Summary analytics command for overall database dashboard
#[derive(Args)]
pub struct SummaryCommand {
    /// Show detailed breakdown of all metrics
    #[arg(long)]
    pub detailed: bool,

    /// Include memory type distribution in summary
    #[arg(long)]
    pub include_types: bool,

    /// Include top tags in summary
    #[arg(long)]
    pub include_tags: bool,

    /// Number of top items to show (tags, types, etc.)
    #[arg(long, default_value = "10")]
    pub top_count: usize,
}

impl SummaryCommand {
    /// Execute summary analytics
    pub fn execute(&self, context: &AnalyticsContext) -> Result<Vec<AnalyticsMetric>> {
        let mut metrics = Vec::new();

        if !context.quiet {
            println!("📊 Generating database summary...");
        }

        // Core statistics
        let core_stats = self.get_core_statistics(context)?;
        metrics.extend(core_stats);

        // Memory type distribution
        if self.include_types || self.detailed {
            let type_stats = self.get_memory_type_distribution(context)?;
            metrics.extend(type_stats);
        }

        // Tag statistics
        if self.include_tags || self.detailed {
            let tag_stats = self.get_tag_statistics(context)?;
            metrics.extend(tag_stats);
        }

        // Quality indicators
        if self.detailed {
            let quality_stats = self.get_quality_indicators(context)?;
            metrics.extend(quality_stats);
        }

        // Recent activity
        let activity_stats = self.get_recent_activity(context)?;
        metrics.extend(activity_stats);

        Ok(metrics)
    }

    /// Get core database statistics
    fn get_core_statistics(&self, context: &AnalyticsContext) -> Result<Vec<AnalyticsMetric>> {
        let mut metrics = Vec::new();

        // Get all memories to calculate statistics
        let all_memories = context.repository.browse_all(1_000_000)?; // Use reasonable limit instead of usize::MAX

        let (active_memories, deleted_memories): (Vec<_>, Vec<_>) =
            all_memories.into_iter().partition(|m| !m.deleted);

        let total_memories = if context.include_deleted {
            active_memories.len() + deleted_memories.len()
        } else {
            active_memories.len()
        };

        metrics.push(AnalyticsMetric {
            name: "Total Memories".to_string(),
            value: MetricValue::Integer(total_memories.try_into().unwrap_or(0)),
            description: Some("Total number of memories in the database".to_string()),
            unit: Some("count".to_string()),
        });

        // Active vs deleted breakdown
        if context.include_deleted && !deleted_memories.is_empty() {
            metrics.push(AnalyticsMetric {
                name: "Active Memories".to_string(),
                value: MetricValue::Integer(active_memories.len().try_into().unwrap_or(0)),
                description: Some("Number of active (non-deleted) memories".to_string()),
                unit: Some("count".to_string()),
            });

            metrics.push(AnalyticsMetric {
                name: "Deleted Memories".to_string(),
                value: MetricValue::Integer(deleted_memories.len().try_into().unwrap_or(0)),
                description: Some("Number of soft-deleted memories".to_string()),
                unit: Some("count".to_string()),
            });
        }

        // Work with the relevant memory set
        let working_memories = if context.include_deleted {
            let mut all = active_memories.clone();
            all.extend(deleted_memories);
            all
        } else {
            active_memories
        };

        if !working_memories.is_empty() {
            // Average confidence score
            let total_confidence: f64 = working_memories.iter().map(|m| m.confidence as f64).sum();
            let avg_confidence = total_confidence / working_memories.len() as f64;

            metrics.push(AnalyticsMetric {
                name: "Average Confidence".to_string(),
                value: MetricValue::Float(avg_confidence),
                description: Some("Average confidence score across all memories".to_string()),
                unit: Some("score".to_string()),
            });

            // Total reference count
            let total_references: i64 = working_memories
                .iter()
                .map(|m| m.reference_count as i64)
                .sum();

            metrics.push(AnalyticsMetric {
                name: "Total References".to_string(),
                value: MetricValue::Integer(total_references),
                description: Some("Total number of memory references/accesses".to_string()),
                unit: Some("count".to_string()),
            });

            // Database age (oldest memory)
            if let Some(oldest_memory) = working_memories.iter().min_by_key(|m| m.created_at) {
                let current_time = chrono::Utc::now().timestamp();
                let age_days = (current_time - oldest_memory.created_at) / 86400; // Convert to days

                metrics.push(AnalyticsMetric {
                    name: "Database Age".to_string(),
                    value: MetricValue::Integer(age_days),
                    description: Some("Age of the oldest memory in the database".to_string()),
                    unit: Some("days".to_string()),
                });
            }
        }

        Ok(metrics)
    }

    /// Get memory type distribution
    fn get_memory_type_distribution(
        &self,
        context: &AnalyticsContext,
    ) -> Result<Vec<AnalyticsMetric>> {
        let mut metrics = Vec::new();

        // Get all memories and count by type
        let all_memories = context.repository.browse_all(1_000_000)?;
        let working_memories: Vec<_> = if context.include_deleted {
            all_memories
        } else {
            all_memories.into_iter().filter(|m| !m.deleted).collect()
        };

        // Count memories by type
        let mut type_counts = HashMap::new();
        for memory in working_memories {
            *type_counts
                .entry(memory.memory_type.to_string())
                .or_insert(0) += 1;
        }

        // Sort by count (descending)
        let mut sorted_types: Vec<_> = type_counts.into_iter().collect();
        sorted_types.sort_by(|a, b| b.1.cmp(&a.1));

        // Add top N types to metrics
        for (memory_type, count) in sorted_types.iter().take(self.top_count) {
            metrics.push(AnalyticsMetric {
                name: format!("Type: {}", memory_type),
                value: MetricValue::Integer((*count).into()),
                description: Some(format!("Number of {} memories", memory_type)),
                unit: Some("count".to_string()),
            });
        }

        Ok(metrics)
    }

    /// Get tag statistics
    fn get_tag_statistics(&self, context: &AnalyticsContext) -> Result<Vec<AnalyticsMetric>> {
        let tag_stats = context.repository.get_tag_stats()?;
        let mut sorted_tags: Vec<_> = tag_stats.into_iter().collect();
        sorted_tags.sort_by(|a, b| b.1.cmp(&a.1));

        let mut metrics = Vec::new();

        // Total unique tags
        metrics.push(AnalyticsMetric {
            name: "Unique Tags".to_string(),
            value: MetricValue::Integer(sorted_tags.len().try_into().unwrap_or(0)),
            description: Some("Total number of unique tags".to_string()),
            unit: Some("count".to_string()),
        });

        // Top tags
        for (i, (tag, count)) in sorted_tags.iter().take(self.top_count).enumerate() {
            metrics.push(AnalyticsMetric {
                name: format!("Top Tag #{}", i + 1),
                value: MetricValue::String(format!("{} ({})", tag, count)),
                description: Some(format!("Tag '{}' used {} times", tag, count)),
                unit: Some("usage".to_string()),
            });
        }

        Ok(metrics)
    }

    /// Get quality indicators
    fn get_quality_indicators(&self, context: &AnalyticsContext) -> Result<Vec<AnalyticsMetric>> {
        let mut metrics = Vec::new();

        // Get all memories and filter appropriately
        let all_memories = context.repository.browse_all(1_000_000)?;
        let working_memories: Vec<_> = if context.include_deleted {
            all_memories
        } else {
            all_memories.into_iter().filter(|m| !m.deleted).collect()
        };

        // Low confidence memories (< 0.5)
        let low_confidence_count = working_memories
            .iter()
            .filter(|m| m.confidence < 0.5)
            .count();

        metrics.push(AnalyticsMetric {
            name: "Low Confidence Memories".to_string(),
            value: MetricValue::Integer(low_confidence_count.try_into().unwrap_or(0)),
            description: Some("Memories with confidence score below 0.5".to_string()),
            unit: Some("count".to_string()),
        });

        // Memories without tags
        let no_tags_count = working_memories
            .iter()
            .filter(|m| m.tags.is_empty())
            .count();

        metrics.push(AnalyticsMetric {
            name: "Untagged Memories".to_string(),
            value: MetricValue::Integer(no_tags_count.try_into().unwrap_or(0)),
            description: Some("Memories without any tags".to_string()),
            unit: Some("count".to_string()),
        });

        // Never accessed memories
        let never_accessed_count = working_memories
            .iter()
            .filter(|m| m.reference_count == 0)
            .count();

        metrics.push(AnalyticsMetric {
            name: "Never Accessed".to_string(),
            value: MetricValue::Integer(never_accessed_count.try_into().unwrap_or(0)),
            description: Some("Memories that have never been accessed".to_string()),
            unit: Some("count".to_string()),
        });

        Ok(metrics)
    }

    /// Get recent activity statistics
    fn get_recent_activity(&self, context: &AnalyticsContext) -> Result<Vec<AnalyticsMetric>> {
        let mut metrics = Vec::new();
        let current_time = chrono::Utc::now().timestamp();
        let week_ago = current_time - (7 * 24 * 60 * 60);

        // Get all memories and filter appropriately
        let all_memories = context.repository.browse_all(1_000_000)?;
        let working_memories: Vec<_> = if context.include_deleted {
            all_memories
        } else {
            all_memories.into_iter().filter(|m| !m.deleted).collect()
        };

        // Memories created in last 7 days
        let recent_created_count = working_memories
            .iter()
            .filter(|m| m.created_at > week_ago)
            .count();

        metrics.push(AnalyticsMetric {
            name: "Created Last 7 Days".to_string(),
            value: MetricValue::Integer(recent_created_count.try_into().unwrap_or(0)),
            description: Some("Memories created in the last 7 days".to_string()),
            unit: Some("count".to_string()),
        });

        // Memories accessed in last 7 days
        let recent_accessed_count = working_memories
            .iter()
            .filter(|m| m.last_accessed.unwrap_or(0) > week_ago)
            .count();

        metrics.push(AnalyticsMetric {
            name: "Accessed Last 7 Days".to_string(),
            value: MetricValue::Integer(recent_accessed_count.try_into().unwrap_or(0)),
            description: Some("Memories accessed in the last 7 days".to_string()),
            unit: Some("count".to_string()),
        });

        // Most accessed memory
        if let Some(most_accessed) = working_memories.iter().max_by_key(|m| m.reference_count) {
            metrics.push(AnalyticsMetric {
                name: "Most Accessed Memory".to_string(),
                value: MetricValue::String(format!(
                    "{} ({} accesses)",
                    most_accessed.topic, most_accessed.reference_count
                )),
                description: Some("Memory with the highest reference count".to_string()),
                unit: Some("accesses".to_string()),
            });
        }

        Ok(metrics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Unused imports removed for lint compliance

    #[test]
    fn test_summary_command_creation() {
        let cmd = SummaryCommand {
            detailed: true,
            include_types: true,
            include_tags: true,
            top_count: 5,
        };

        assert!(cmd.detailed);
        assert!(cmd.include_types);
        assert!(cmd.include_tags);
        assert_eq!(cmd.top_count, 5);
    }
}
