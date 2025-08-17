use super::{AnalyticsContext, AnalyticsMetric, MetricValue};
use crate::memory::repository::MemoryRepository;
use crate::utils::error::Result;
use clap::Args;
use std::collections::HashMap;
use std::convert::TryInto;

/// Usage analytics command for comprehensive access patterns and usage behavior analysis
#[derive(Args)]
pub struct UsageCommand {
    /// Show access pattern analysis and frequency distribution
    #[arg(long)]
    pub access_patterns: bool,

    /// Show reference count analysis and top accessed memories
    #[arg(long)]
    pub reference_analysis: bool,

    /// Show recency analysis and staleness patterns
    #[arg(long)]
    pub recency_analysis: bool,

    /// Show usage patterns by memory type
    #[arg(long)]
    pub type_usage: bool,

    /// Show detailed breakdown of all usage analytics
    #[arg(long)]
    pub detailed: bool,

    /// Number of top items to show (memories, etc.)
    #[arg(long, default_value = "10")]
    pub top_count: usize,
}

impl UsageCommand {
    /// Execute usage analytics
    pub fn execute(&self, context: &AnalyticsContext) -> Result<Vec<AnalyticsMetric>> {
        let mut metrics = Vec::new();

        if !context.quiet {
            println!("📊 Analyzing usage patterns and access behaviors...");
        }

        // Core access pattern analysis (always shown)
        let access_metrics = self.get_basic_access_analysis(context)?;
        metrics.extend(access_metrics);

        // Access patterns
        if self.access_patterns || self.detailed {
            let pattern_metrics = self.get_access_pattern_analysis(context)?;
            metrics.extend(pattern_metrics);
        }

        // Reference analysis
        if self.reference_analysis || self.detailed {
            let reference_metrics = self.get_reference_count_analysis(context)?;
            metrics.extend(reference_metrics);
        }

        // Recency analysis
        if self.recency_analysis || self.detailed {
            let recency_metrics = self.get_recency_analysis(context)?;
            metrics.extend(recency_metrics);
        }

        // Type-usage correlation
        if self.type_usage || self.detailed {
            let type_usage_metrics = self.get_type_usage_analysis(context)?;
            metrics.extend(type_usage_metrics);
        }

        Ok(metrics)
    }

    /// Get basic access analysis metrics
    fn get_basic_access_analysis(
        &self,
        context: &AnalyticsContext,
    ) -> Result<Vec<AnalyticsMetric>> {
        let mut metrics = Vec::new();

        // Get all memories and filter appropriately
        let all_memories = context.repository.browse_all(1_000_000)?;
        let working_memories: Vec<_> = if context.include_deleted {
            all_memories
        } else {
            all_memories.into_iter().filter(|m| !m.deleted).collect()
        };

        if working_memories.is_empty() {
            return Ok(metrics);
        }

        // Basic usage statistics
        let total_memories = working_memories.len();
        let never_accessed = working_memories
            .iter()
            .filter(|m| m.reference_count == 0)
            .count();
        let has_accesses = total_memories - never_accessed;

        let total_references: i64 = working_memories
            .iter()
            .map(|m| m.reference_count as i64)
            .sum();
        let avg_references = if total_memories > 0 {
            total_references as f64 / total_memories as f64
        } else {
            0.0
        };

        metrics.push(AnalyticsMetric {
            name: "Total Access Count".to_string(),
            value: MetricValue::Integer(total_references),
            description: Some("Total number of memory accesses across all memories".to_string()),
            unit: Some("accesses".to_string()),
        });

        metrics.push(AnalyticsMetric {
            name: "Average Accesses per Memory".to_string(),
            value: MetricValue::Float(avg_references),
            description: Some("Average number of accesses per memory".to_string()),
            unit: Some("accesses".to_string()),
        });

        metrics.push(AnalyticsMetric {
            name: "Never Accessed Memories".to_string(),
            value: MetricValue::Integer(never_accessed.try_into().unwrap_or(0)),
            description: Some(format!(
                "{:.1}% of memories",
                (never_accessed as f64 / total_memories as f64) * 100.0
            )),
            unit: Some("count".to_string()),
        });

        metrics.push(AnalyticsMetric {
            name: "Accessed Memories".to_string(),
            value: MetricValue::Integer(has_accesses.try_into().unwrap_or(0)),
            description: Some(format!(
                "{:.1}% of memories",
                (has_accesses as f64 / total_memories as f64) * 100.0
            )),
            unit: Some("count".to_string()),
        });

        Ok(metrics)
    }

    /// Get detailed access pattern analysis
    fn get_access_pattern_analysis(
        &self,
        context: &AnalyticsContext,
    ) -> Result<Vec<AnalyticsMetric>> {
        let mut metrics = Vec::new();

        // Get all memories and filter appropriately
        let all_memories = context.repository.browse_all(1_000_000)?;
        let working_memories: Vec<_> = if context.include_deleted {
            all_memories
        } else {
            all_memories.into_iter().filter(|m| !m.deleted).collect()
        };

        if working_memories.is_empty() {
            return Ok(metrics);
        }

        // Classify memories by access frequency
        let never_accessed = working_memories
            .iter()
            .filter(|m| m.reference_count == 0)
            .count();
        let rarely_accessed = working_memories
            .iter()
            .filter(|m| m.reference_count >= 1 && m.reference_count <= 5)
            .count();
        let moderately_accessed = working_memories
            .iter()
            .filter(|m| m.reference_count >= 6 && m.reference_count <= 20)
            .count();
        let heavily_accessed = working_memories
            .iter()
            .filter(|m| m.reference_count > 20)
            .count();

        let total = working_memories.len();

        metrics.push(AnalyticsMetric {
            name: "Never Accessed (0)".to_string(),
            value: MetricValue::Integer(never_accessed.try_into().unwrap_or(0)),
            description: Some(format!(
                "{:.1}% of memories",
                (never_accessed as f64 / total as f64) * 100.0
            )),
            unit: Some("count".to_string()),
        });

        metrics.push(AnalyticsMetric {
            name: "Rarely Accessed (1-5)".to_string(),
            value: MetricValue::Integer(rarely_accessed.try_into().unwrap_or(0)),
            description: Some(format!(
                "{:.1}% of memories",
                (rarely_accessed as f64 / total as f64) * 100.0
            )),
            unit: Some("count".to_string()),
        });

        metrics.push(AnalyticsMetric {
            name: "Moderately Accessed (6-20)".to_string(),
            value: MetricValue::Integer(moderately_accessed.try_into().unwrap_or(0)),
            description: Some(format!(
                "{:.1}% of memories",
                (moderately_accessed as f64 / total as f64) * 100.0
            )),
            unit: Some("count".to_string()),
        });

        metrics.push(AnalyticsMetric {
            name: "Heavily Accessed (>20)".to_string(),
            value: MetricValue::Integer(heavily_accessed.try_into().unwrap_or(0)),
            description: Some(format!(
                "{:.1}% of memories",
                (heavily_accessed as f64 / total as f64) * 100.0
            )),
            unit: Some("count".to_string()),
        });

        // Access efficiency analysis
        let accessed_memories: Vec<_> = working_memories
            .iter()
            .filter(|m| m.reference_count > 0)
            .collect();
        if !accessed_memories.is_empty() {
            let avg_efficiency = accessed_memories
                .iter()
                .map(|m| (m.reference_count as f64) * m.confidence as f64)
                .sum::<f64>()
                / accessed_memories.len() as f64;

            metrics.push(AnalyticsMetric {
                name: "Access Efficiency Score".to_string(),
                value: MetricValue::Float(avg_efficiency),
                description: Some(
                    "Average of (reference_count × confidence) for accessed memories".to_string(),
                ),
                unit: Some("score".to_string()),
            });
        }

        Ok(metrics)
    }

    /// Get reference count analysis
    fn get_reference_count_analysis(
        &self,
        context: &AnalyticsContext,
    ) -> Result<Vec<AnalyticsMetric>> {
        let mut metrics = Vec::new();

        // Get all memories and filter appropriately
        let all_memories = context.repository.browse_all(1_000_000)?;
        let working_memories: Vec<_> = if context.include_deleted {
            all_memories
        } else {
            all_memories.into_iter().filter(|m| !m.deleted).collect()
        };

        if working_memories.is_empty() {
            return Ok(metrics);
        }

        // Reference count statistics
        let reference_counts: Vec<u32> =
            working_memories.iter().map(|m| m.reference_count).collect();
        let total_references: i64 = reference_counts.iter().map(|&r| r as i64).sum();
        let avg_references = total_references as f64 / reference_counts.len() as f64;
        let min_references = reference_counts.iter().min().unwrap_or(&0);
        let max_references = reference_counts.iter().max().unwrap_or(&0);

        metrics.push(AnalyticsMetric {
            name: "Reference Count Average".to_string(),
            value: MetricValue::Float(avg_references),
            description: Some("Average number of references per memory".to_string()),
            unit: Some("references".to_string()),
        });

        metrics.push(AnalyticsMetric {
            name: "Reference Count Range".to_string(),
            value: MetricValue::String(format!("{} - {}", min_references, max_references)),
            description: Some("Minimum and maximum reference counts".to_string()),
            unit: Some("range".to_string()),
        });

        // Top accessed memories
        let mut sorted_by_references = working_memories.clone();
        sorted_by_references.sort_by(|a, b| b.reference_count.cmp(&a.reference_count));

        for (i, memory) in sorted_by_references
            .iter()
            .take(self.top_count.min(5))
            .enumerate()
        {
            metrics.push(AnalyticsMetric {
                name: format!("Top {} Most Accessed", i + 1),
                value: MetricValue::String(format!(
                    "{} ({} refs)",
                    memory.title, memory.reference_count
                )),
                description: Some(format!("{}th most accessed memory", i + 1)),
                unit: Some("references".to_string()),
            });
        }

        // Reference distribution analysis
        let single_access = reference_counts.iter().filter(|&&r| r == 1).count();
        let multiple_access = reference_counts
            .iter()
            .filter(|&&r| r > 1 && r <= 10)
            .count();
        let frequent_access = reference_counts.iter().filter(|&&r| r > 10).count();

        let total = reference_counts.len();

        metrics.push(AnalyticsMetric {
            name: "Single Access Memories".to_string(),
            value: MetricValue::Integer(single_access.try_into().unwrap_or(0)),
            description: Some(format!(
                "{:.1}% accessed exactly once",
                (single_access as f64 / total as f64) * 100.0
            )),
            unit: Some("count".to_string()),
        });

        metrics.push(AnalyticsMetric {
            name: "Multiple Access Memories".to_string(),
            value: MetricValue::Integer(multiple_access.try_into().unwrap_or(0)),
            description: Some(format!(
                "{:.1}% accessed 2-10 times",
                (multiple_access as f64 / total as f64) * 100.0
            )),
            unit: Some("count".to_string()),
        });

        metrics.push(AnalyticsMetric {
            name: "Frequently Accessed Memories".to_string(),
            value: MetricValue::Integer(frequent_access.try_into().unwrap_or(0)),
            description: Some(format!(
                "{:.1}% accessed >10 times",
                (frequent_access as f64 / total as f64) * 100.0
            )),
            unit: Some("count".to_string()),
        });

        Ok(metrics)
    }

    /// Get recency analysis
    fn get_recency_analysis(&self, context: &AnalyticsContext) -> Result<Vec<AnalyticsMetric>> {
        let mut metrics = Vec::new();

        // Get all memories and filter appropriately
        let all_memories = context.repository.browse_all(1_000_000)?;
        let working_memories: Vec<_> = if context.include_deleted {
            all_memories
        } else {
            all_memories.into_iter().filter(|m| !m.deleted).collect()
        };

        if working_memories.is_empty() {
            return Ok(metrics);
        }

        let current_time = chrono::Utc::now().timestamp();
        let day_ago = current_time - (24 * 60 * 60);
        let week_ago = current_time - (7 * 24 * 60 * 60);
        let month_ago = current_time - (30 * 24 * 60 * 60);

        // Categorize by recency
        let accessed_last_24h = working_memories
            .iter()
            .filter(|m| m.last_accessed.unwrap_or(0) > day_ago)
            .count();

        let accessed_last_week = working_memories
            .iter()
            .filter(|m| {
                m.last_accessed.unwrap_or(0) > week_ago && m.last_accessed.unwrap_or(0) <= day_ago
            })
            .count();

        let accessed_last_month = working_memories
            .iter()
            .filter(|m| {
                m.last_accessed.unwrap_or(0) > month_ago && m.last_accessed.unwrap_or(0) <= week_ago
            })
            .count();

        let stale_memories = working_memories
            .iter()
            .filter(|m| m.last_accessed.unwrap_or(0) <= month_ago && m.last_accessed.is_some())
            .count();

        let never_accessed = working_memories
            .iter()
            .filter(|m| m.last_accessed.is_none())
            .count();

        let total = working_memories.len();

        metrics.push(AnalyticsMetric {
            name: "Accessed Last 24h".to_string(),
            value: MetricValue::Integer(accessed_last_24h.try_into().unwrap_or(0)),
            description: Some(format!(
                "{:.1}% of memories",
                (accessed_last_24h as f64 / total as f64) * 100.0
            )),
            unit: Some("count".to_string()),
        });

        metrics.push(AnalyticsMetric {
            name: "Accessed Last Week".to_string(),
            value: MetricValue::Integer(accessed_last_week.try_into().unwrap_or(0)),
            description: Some(format!(
                "{:.1}% of memories",
                (accessed_last_week as f64 / total as f64) * 100.0
            )),
            unit: Some("count".to_string()),
        });

        metrics.push(AnalyticsMetric {
            name: "Accessed Last Month".to_string(),
            value: MetricValue::Integer(accessed_last_month.try_into().unwrap_or(0)),
            description: Some(format!(
                "{:.1}% of memories",
                (accessed_last_month as f64 / total as f64) * 100.0
            )),
            unit: Some("count".to_string()),
        });

        metrics.push(AnalyticsMetric {
            name: "Stale Memories (>30d)".to_string(),
            value: MetricValue::Integer(stale_memories.try_into().unwrap_or(0)),
            description: Some(format!(
                "{:.1}% not accessed in 30+ days",
                (stale_memories as f64 / total as f64) * 100.0
            )),
            unit: Some("count".to_string()),
        });

        metrics.push(AnalyticsMetric {
            name: "Never Accessed".to_string(),
            value: MetricValue::Integer(never_accessed.try_into().unwrap_or(0)),
            description: Some(format!(
                "{:.1}% never accessed",
                (never_accessed as f64 / total as f64) * 100.0
            )),
            unit: Some("count".to_string()),
        });

        // Most and least recently accessed
        let accessed_memories: Vec<_> = working_memories
            .iter()
            .filter(|m| m.last_accessed.is_some())
            .collect();

        if !accessed_memories.is_empty() {
            let mut sorted_by_recency = accessed_memories.clone();
            sorted_by_recency.sort_by(|a, b| {
                b.last_accessed
                    .unwrap_or(0)
                    .cmp(&a.last_accessed.unwrap_or(0))
            });

            if let Some(most_recent) = sorted_by_recency.first() {
                let days_ago = (current_time - most_recent.last_accessed.unwrap_or(0)) / 86400;
                metrics.push(AnalyticsMetric {
                    name: "Most Recently Accessed".to_string(),
                    value: MetricValue::String(format!(
                        "{} ({} days ago)",
                        most_recent.title, days_ago
                    )),
                    description: Some("Memory accessed most recently".to_string()),
                    unit: Some("days".to_string()),
                });
            }

            if let Some(least_recent) = sorted_by_recency.last() {
                let days_ago = (current_time - least_recent.last_accessed.unwrap_or(0)) / 86400;
                metrics.push(AnalyticsMetric {
                    name: "Least Recently Accessed".to_string(),
                    value: MetricValue::String(format!(
                        "{} ({} days ago)",
                        least_recent.title, days_ago
                    )),
                    description: Some("Memory accessed least recently".to_string()),
                    unit: Some("days".to_string()),
                });
            }
        }

        Ok(metrics)
    }

    /// Get type-usage correlation analysis
    fn get_type_usage_analysis(&self, context: &AnalyticsContext) -> Result<Vec<AnalyticsMetric>> {
        let mut metrics = Vec::new();

        // Get all memories and filter appropriately
        let all_memories = context.repository.browse_all(1_000_000)?;
        let working_memories: Vec<_> = if context.include_deleted {
            all_memories
        } else {
            all_memories.into_iter().filter(|m| !m.deleted).collect()
        };

        if working_memories.is_empty() {
            return Ok(metrics);
        }

        // Group by memory type for usage analysis
        let mut type_usage: HashMap<String, Vec<_>> = HashMap::new();
        for memory in &working_memories {
            type_usage
                .entry(memory.memory_type.to_string())
                .or_insert_with(Vec::new)
                .push(memory);
        }

        // Analyze usage patterns by type
        for (memory_type, memories) in type_usage.iter() {
            let total_refs: i64 = memories.iter().map(|m| m.reference_count as i64).sum();
            let avg_refs = total_refs as f64 / memories.len() as f64;
            let never_accessed = memories.iter().filter(|m| m.reference_count == 0).count();
            let usage_rate =
                ((memories.len() - never_accessed) as f64 / memories.len() as f64) * 100.0;

            metrics.push(AnalyticsMetric {
                name: format!("{} Avg Usage", memory_type.to_uppercase()),
                value: MetricValue::Float(avg_refs),
                description: Some(format!("Average references for {} memories", memory_type)),
                unit: Some("references".to_string()),
            });

            metrics.push(AnalyticsMetric {
                name: format!("{} Usage Rate", memory_type.to_uppercase()),
                value: MetricValue::Float(usage_rate),
                description: Some(format!(
                    "Percentage of {} memories that have been accessed",
                    memory_type
                )),
                unit: Some("percentage".to_string()),
            });

            // Usage efficiency (references per confidence point)
            let accessed_memories: Vec<_> =
                memories.iter().filter(|m| m.reference_count > 0).collect();
            if !accessed_memories.is_empty() {
                let avg_efficiency = accessed_memories
                    .iter()
                    .map(|m| m.reference_count as f64 / m.confidence.max(0.1) as f64)
                    .sum::<f64>()
                    / accessed_memories.len() as f64;

                metrics.push(AnalyticsMetric {
                    name: format!("{} Usage Efficiency", memory_type.to_uppercase()),
                    value: MetricValue::Float(avg_efficiency),
                    description: Some(format!(
                        "Average references per confidence point for {} memories",
                        memory_type
                    )),
                    unit: Some("efficiency".to_string()),
                });
            }
        }

        // Cross-type comparison
        let type_averages: Vec<_> = type_usage
            .iter()
            .map(|(t, memories)| {
                let avg = memories
                    .iter()
                    .map(|m| m.reference_count as f64)
                    .sum::<f64>()
                    / memories.len() as f64;
                (t.clone(), avg)
            })
            .collect();

        let mut sorted_types = type_averages.clone();
        sorted_types.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        if let Some((most_used_type, avg_usage)) = sorted_types.first() {
            metrics.push(AnalyticsMetric {
                name: "Most Used Type".to_string(),
                value: MetricValue::String(format!(
                    "{} ({:.1} avg refs)",
                    most_used_type.to_uppercase(),
                    avg_usage
                )),
                description: Some("Memory type with highest average usage".to_string()),
                unit: Some("type".to_string()),
            });
        }

        if let Some((least_used_type, avg_usage)) = sorted_types.last() {
            metrics.push(AnalyticsMetric {
                name: "Least Used Type".to_string(),
                value: MetricValue::String(format!(
                    "{} ({:.1} avg refs)",
                    least_used_type.to_uppercase(),
                    avg_usage
                )),
                description: Some("Memory type with lowest average usage".to_string()),
                unit: Some("type".to_string()),
            });
        }

        Ok(metrics)
    }
}
