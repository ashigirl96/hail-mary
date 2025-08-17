use super::{AnalyticsContext, AnalyticsMetric, MetricValue};
use crate::memory::repository::MemoryRepository;
use crate::utils::error::Result;
use clap::Args;
use std::collections::HashMap;
use std::convert::TryInto;

/// Memory analytics command for comprehensive memory type and confidence analysis
#[derive(Args)]
pub struct MemoryCommand {
    /// Show confidence score breakdown and distribution
    #[arg(long)]
    pub confidence_breakdown: bool,

    /// Show detailed memory type distribution analysis
    #[arg(long)]
    pub type_distribution: bool,

    /// Show content quality metrics and statistics
    #[arg(long)]
    pub quality_metrics: bool,

    /// Show detailed breakdown of all memory analytics
    #[arg(long)]
    pub detailed: bool,

    /// Number of top items to show (memories, types, etc.)
    #[arg(long, default_value = "10")]
    pub top_count: usize,
}

impl MemoryCommand {
    /// Execute memory analytics
    pub fn execute(&self, context: &AnalyticsContext) -> Result<Vec<AnalyticsMetric>> {
        let mut metrics = Vec::new();

        if !context.quiet {
            println!("📊 Analyzing memory types and distributions...");
        }

        // Core type distribution (always shown)
        let type_metrics = self.get_type_distribution_analysis(context)?;
        metrics.extend(type_metrics);

        // Confidence analysis
        if self.confidence_breakdown || self.detailed {
            let confidence_metrics = self.get_confidence_analysis(context)?;
            metrics.extend(confidence_metrics);
        }

        // Type distribution details
        if self.type_distribution || self.detailed {
            let detailed_type_metrics = self.get_detailed_type_analysis(context)?;
            metrics.extend(detailed_type_metrics);
        }

        // Quality metrics
        if self.quality_metrics || self.detailed {
            let quality_metrics = self.get_content_quality_analysis(context)?;
            metrics.extend(quality_metrics);
        }

        Ok(metrics)
    }

    /// Get basic type distribution analysis
    fn get_type_distribution_analysis(
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

        // Count memories by type
        let mut type_counts = HashMap::new();
        let mut type_confidence_sums = HashMap::new();
        let mut type_reference_sums = HashMap::new();

        for memory in &working_memories {
            let type_name = memory.memory_type.to_string();
            *type_counts.entry(type_name.clone()).or_insert(0) += 1;
            *type_confidence_sums.entry(type_name.clone()).or_insert(0.0) +=
                memory.confidence as f64;
            *type_reference_sums.entry(type_name.clone()).or_insert(0i64) +=
                memory.reference_count as i64;
        }

        let total_memories = working_memories.len();

        // Generate type distribution metrics
        for (memory_type, count) in type_counts.iter() {
            let percentage = (*count as f64 / total_memories as f64) * 100.0;
            let avg_confidence =
                type_confidence_sums.get(memory_type).unwrap_or(&0.0) / *count as f64;
            let avg_references = type_reference_sums.get(memory_type).unwrap_or(&0) / *count as i64;

            metrics.push(AnalyticsMetric {
                name: format!("{} Memories", memory_type.to_uppercase()),
                value: MetricValue::Integer((*count).into()),
                description: Some(format!("{:.1}% of total memories", percentage)),
                unit: Some("count".to_string()),
            });

            metrics.push(AnalyticsMetric {
                name: format!("{} Avg Confidence", memory_type.to_uppercase()),
                value: MetricValue::Float(avg_confidence),
                description: Some(format!("Average confidence for {} memories", memory_type)),
                unit: Some("score".to_string()),
            });

            metrics.push(AnalyticsMetric {
                name: format!("{} Avg References", memory_type.to_uppercase()),
                value: MetricValue::Integer(avg_references),
                description: Some(format!("Average references for {} memories", memory_type)),
                unit: Some("count".to_string()),
            });
        }

        Ok(metrics)
    }

    /// Get detailed confidence analysis
    fn get_confidence_analysis(&self, context: &AnalyticsContext) -> Result<Vec<AnalyticsMetric>> {
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

        // Confidence distribution analysis
        let high_confidence = working_memories
            .iter()
            .filter(|m| m.confidence > 0.8)
            .count();
        let medium_confidence = working_memories
            .iter()
            .filter(|m| m.confidence >= 0.5 && m.confidence <= 0.8)
            .count();
        let low_confidence = working_memories
            .iter()
            .filter(|m| m.confidence < 0.5)
            .count();

        let total = working_memories.len();

        metrics.push(AnalyticsMetric {
            name: "High Confidence (>0.8)".to_string(),
            value: MetricValue::Integer(high_confidence.try_into().unwrap_or(0)),
            description: Some(format!(
                "{:.1}% of memories",
                (high_confidence as f64 / total as f64) * 100.0
            )),
            unit: Some("count".to_string()),
        });

        metrics.push(AnalyticsMetric {
            name: "Medium Confidence (0.5-0.8)".to_string(),
            value: MetricValue::Integer(medium_confidence.try_into().unwrap_or(0)),
            description: Some(format!(
                "{:.1}% of memories",
                (medium_confidence as f64 / total as f64) * 100.0
            )),
            unit: Some("count".to_string()),
        });

        metrics.push(AnalyticsMetric {
            name: "Low Confidence (<0.5)".to_string(),
            value: MetricValue::Integer(low_confidence.try_into().unwrap_or(0)),
            description: Some(format!(
                "{:.1}% of memories",
                (low_confidence as f64 / total as f64) * 100.0
            )),
            unit: Some("count".to_string()),
        });

        // Confidence statistics
        let confidence_values: Vec<f64> = working_memories
            .iter()
            .map(|m| m.confidence as f64)
            .collect();
        let mean_confidence =
            confidence_values.iter().sum::<f64>() / confidence_values.len() as f64;
        let min_confidence = confidence_values
            .iter()
            .fold(f64::INFINITY, |a, &b| a.min(b));
        let max_confidence = confidence_values
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        metrics.push(AnalyticsMetric {
            name: "Confidence Mean".to_string(),
            value: MetricValue::Float(mean_confidence),
            description: Some("Average confidence across all memories".to_string()),
            unit: Some("score".to_string()),
        });

        metrics.push(AnalyticsMetric {
            name: "Confidence Range".to_string(),
            value: MetricValue::String(format!("{:.2} - {:.2}", min_confidence, max_confidence)),
            description: Some("Minimum and maximum confidence scores".to_string()),
            unit: Some("range".to_string()),
        });

        // Top confident memories
        let mut sorted_memories = working_memories.clone();
        sorted_memories.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        if let Some(top_memory) = sorted_memories.first() {
            metrics.push(AnalyticsMetric {
                name: "Most Confident Memory".to_string(),
                value: MetricValue::String(format!(
                    "{} ({:.2})",
                    top_memory.title, top_memory.confidence
                )),
                description: Some("Memory with highest confidence score".to_string()),
                unit: Some("confidence".to_string()),
            });
        }

        if let Some(lowest_memory) = sorted_memories.last() {
            metrics.push(AnalyticsMetric {
                name: "Least Confident Memory".to_string(),
                value: MetricValue::String(format!(
                    "{} ({:.2})",
                    lowest_memory.title, lowest_memory.confidence
                )),
                description: Some("Memory with lowest confidence score".to_string()),
                unit: Some("confidence".to_string()),
            });
        }

        Ok(metrics)
    }

    /// Get detailed type analysis
    fn get_detailed_type_analysis(
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

        // Group memories by type for detailed analysis
        let mut type_groups: HashMap<String, Vec<_>> = HashMap::new();
        for memory in &working_memories {
            type_groups
                .entry(memory.memory_type.to_string())
                .or_insert_with(Vec::new)
                .push(memory);
        }

        // Analyze each type in detail
        for (memory_type, memories) in type_groups.iter() {
            // Content length analysis
            let content_lengths: Vec<usize> = memories.iter().map(|m| m.content.len()).collect();
            let avg_content_length =
                content_lengths.iter().sum::<usize>() as f64 / content_lengths.len() as f64;
            let min_content_length = content_lengths.iter().min().unwrap_or(&0);
            let max_content_length = content_lengths.iter().max().unwrap_or(&0);

            metrics.push(AnalyticsMetric {
                name: format!("{} Avg Content Length", memory_type.to_uppercase()),
                value: MetricValue::Float(avg_content_length),
                description: Some(format!(
                    "Average content length for {} memories",
                    memory_type
                )),
                unit: Some("characters".to_string()),
            });

            metrics.push(AnalyticsMetric {
                name: format!("{} Content Range", memory_type.to_uppercase()),
                value: MetricValue::String(format!(
                    "{} - {} chars",
                    min_content_length, max_content_length
                )),
                description: Some(format!("Content length range for {} memories", memory_type)),
                unit: Some("range".to_string()),
            });

            // Tag coverage analysis
            let tagged_count = memories.iter().filter(|m| !m.tags.is_empty()).count();
            let tag_coverage = (tagged_count as f64 / memories.len() as f64) * 100.0;

            metrics.push(AnalyticsMetric {
                name: format!("{} Tag Coverage", memory_type.to_uppercase()),
                value: MetricValue::Float(tag_coverage),
                description: Some(format!("Percentage of {} memories with tags", memory_type)),
                unit: Some("percentage".to_string()),
            });

            // Examples coverage
            let with_examples_count = memories.iter().filter(|m| !m.examples.is_empty()).count();
            let examples_coverage = (with_examples_count as f64 / memories.len() as f64) * 100.0;

            metrics.push(AnalyticsMetric {
                name: format!("{} Examples Coverage", memory_type.to_uppercase()),
                value: MetricValue::Float(examples_coverage),
                description: Some(format!(
                    "Percentage of {} memories with examples",
                    memory_type
                )),
                unit: Some("percentage".to_string()),
            });
        }

        Ok(metrics)
    }

    /// Get content quality analysis
    fn get_content_quality_analysis(
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

        // Overall content quality metrics
        let total_memories = working_memories.len();
        let memories_with_tags = working_memories
            .iter()
            .filter(|m| !m.tags.is_empty())
            .count();
        let memories_with_examples = working_memories
            .iter()
            .filter(|m| !m.examples.is_empty())
            .count();
        let memories_with_source = working_memories.len();

        metrics.push(AnalyticsMetric {
            name: "Tagged Memories".to_string(),
            value: MetricValue::Integer(memories_with_tags.try_into().unwrap_or(0)),
            description: Some(format!(
                "{:.1}% have tags",
                (memories_with_tags as f64 / total_memories as f64) * 100.0
            )),
            unit: Some("count".to_string()),
        });

        metrics.push(AnalyticsMetric {
            name: "Memories with Examples".to_string(),
            value: MetricValue::Integer(memories_with_examples.try_into().unwrap_or(0)),
            description: Some(format!(
                "{:.1}% have examples",
                (memories_with_examples as f64 / total_memories as f64) * 100.0
            )),
            unit: Some("count".to_string()),
        });

        metrics.push(AnalyticsMetric {
            name: "Memories with Source".to_string(),
            value: MetricValue::Integer(memories_with_source.try_into().unwrap_or(0)),
            description: Some(format!(
                "{:.1}% have source attribution",
                (memories_with_source as f64 / total_memories as f64) * 100.0
            )),
            unit: Some("count".to_string()),
        });

        // Content length analysis
        let content_lengths: Vec<usize> =
            working_memories.iter().map(|m| m.content.len()).collect();
        let avg_content_length =
            content_lengths.iter().sum::<usize>() as f64 / content_lengths.len() as f64;
        let min_content_length = content_lengths.iter().min().unwrap_or(&0);
        let max_content_length = content_lengths.iter().max().unwrap_or(&0);

        metrics.push(AnalyticsMetric {
            name: "Average Content Length".to_string(),
            value: MetricValue::Float(avg_content_length),
            description: Some("Average number of characters per memory".to_string()),
            unit: Some("characters".to_string()),
        });

        metrics.push(AnalyticsMetric {
            name: "Content Length Range".to_string(),
            value: MetricValue::String(format!(
                "{} - {} chars",
                min_content_length, max_content_length
            )),
            description: Some("Minimum and maximum content lengths".to_string()),
            unit: Some("range".to_string()),
        });

        // Find shortest and longest memories
        let mut memories_by_length = working_memories.clone();
        memories_by_length.sort_by_key(|m| m.content.len());

        if let Some(shortest) = memories_by_length.first() {
            metrics.push(AnalyticsMetric {
                name: "Shortest Memory".to_string(),
                value: MetricValue::String(format!(
                    "{} ({} chars)",
                    shortest.title,
                    shortest.content.len()
                )),
                description: Some("Memory with the least content".to_string()),
                unit: Some("characters".to_string()),
            });
        }

        if let Some(longest) = memories_by_length.last() {
            metrics.push(AnalyticsMetric {
                name: "Longest Memory".to_string(),
                value: MetricValue::String(format!(
                    "{} ({} chars)",
                    longest.title,
                    longest.content.len()
                )),
                description: Some("Memory with the most content".to_string()),
                unit: Some("characters".to_string()),
            });
        }

        // Quality score calculation (composite metric)
        let quality_score = {
            let tag_score = (memories_with_tags as f64 / total_memories as f64) * 25.0;
            let example_score = (memories_with_examples as f64 / total_memories as f64) * 25.0;
            let source_score = (memories_with_source as f64 / total_memories as f64) * 25.0;
            let confidence_score = working_memories
                .iter()
                .map(|m| m.confidence as f64)
                .sum::<f64>()
                / total_memories as f64
                * 25.0;

            tag_score + example_score + source_score + confidence_score
        };

        metrics.push(AnalyticsMetric {
            name: "Overall Quality Score".to_string(),
            value: MetricValue::Float(quality_score),
            description: Some(
                "Composite quality score (0-100) based on tags, examples, sources, and confidence"
                    .to_string(),
            ),
            unit: Some("score".to_string()),
        });

        Ok(metrics)
    }
}
