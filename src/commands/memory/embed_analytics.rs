use crate::mcp::server::get_default_db_path;
use crate::memory::{
    embeddings::EmbeddingService,
    models::{Memory, MemoryType},
    repository::{MemoryRepository, SqliteMemoryRepository},
};
use crate::utils::error::Result;
use clap::Args;
use std::collections::HashMap;
use std::path::PathBuf;

/// Analyze embeddings and vector space characteristics
#[derive(Args)]
pub struct EmbedAnalyticsCommand {
    /// Path to the database file (defaults to ~/.local/share/hail-mary/memory.db)
    #[arg(long, value_name = "PATH")]
    pub db_path: Option<PathBuf>,

    /// Memory type to analyze (if not specified, all types)
    #[arg(long, value_enum)]
    pub r#type: Option<MemoryType>,

    /// Analysis type
    #[arg(long, value_enum, default_value = "overview")]
    pub analysis: AnalysisType,

    /// Number of memories to analyze
    #[arg(long, default_value = "1000")]
    pub limit: usize,

    /// Show detailed output
    #[arg(long, short)]
    pub verbose: bool,

    /// Export analysis to JSON file
    #[arg(long)]
    pub export: Option<PathBuf>,

    /// Show distribution histograms
    #[arg(long)]
    pub histogram: bool,

    /// Number of bins for histograms
    #[arg(long, default_value = "10")]
    pub bins: usize,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum AnalysisType {
    /// General overview of embeddings
    Overview,
    /// Similarity distribution analysis
    Similarity,
    /// Density and coverage analysis
    Density,
    /// Outlier detection
    Outliers,
    /// Vocabulary analysis
    Vocabulary,
    /// Temporal analysis
    Temporal,
}

impl EmbedAnalyticsCommand {
    /// Execute the embed-analytics command
    pub fn execute(self) -> Result<()> {
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
            let analysis = self.perform_analysis(&db_path).await?;

            // Display results
            self.display_analysis(&analysis)?;

            // Export if requested
            if let Some(ref export_path) = self.export {
                self.export_analysis(&analysis, export_path)?;
            }

            Ok(())
        })
    }

    /// Perform the requested analysis
    async fn perform_analysis(&self, db_path: &PathBuf) -> Result<AnalysisResult> {
        let repository = SqliteMemoryRepository::new(db_path)?;

        // Load memories
        let memories = if let Some(ref memory_type) = self.r#type {
            repository.browse_by_type(memory_type, self.limit)?
        } else {
            repository.browse_all(self.limit)?
        };

        if memories.is_empty() {
            return Ok(AnalysisResult::default());
        }

        // Get embeddings (check if they exist in DB first)
        let memories_with_embeddings = self.get_embeddings(&repository, &memories).await?;

        if self.verbose {
            println!(
                "📊 Analyzing {} memories with embeddings...",
                memories_with_embeddings.len()
            );
        }

        // Perform selected analysis
        let result = match self.analysis {
            AnalysisType::Overview => self.analyze_overview(&memories_with_embeddings)?,
            AnalysisType::Similarity => self.analyze_similarity(&memories_with_embeddings)?,
            AnalysisType::Density => self.analyze_density(&memories_with_embeddings)?,
            AnalysisType::Outliers => self.analyze_outliers(&memories_with_embeddings)?,
            AnalysisType::Vocabulary => self.analyze_vocabulary(&memories)?,
            AnalysisType::Temporal => self.analyze_temporal(&memories_with_embeddings)?,
        };

        Ok(result)
    }

    /// Get embeddings for memories
    async fn get_embeddings(
        &self,
        repository: &SqliteMemoryRepository,
        memories: &[Memory],
    ) -> Result<Vec<(Memory, Vec<f32>)>> {
        let mut memories_with_embeddings = Vec::new();
        let embedding_service = EmbeddingService::new()?;

        for memory in memories {
            // Try to get from database first
            if let Ok(Some(embedding)) = repository.get_embedding(&memory.id) {
                memories_with_embeddings.push((memory.clone(), embedding));
            } else if self.verbose {
                // Generate embedding if not found
                let text = format!("{} {}", memory.title, memory.content);
                let embeddings = embedding_service.embed_texts(vec![text]).await?;
                if let Some(embedding) = embeddings.into_iter().next() {
                    memories_with_embeddings.push((memory.clone(), embedding));
                }
            }
        }

        Ok(memories_with_embeddings)
    }

    /// Analyze overview of embeddings
    fn analyze_overview(
        &self,
        memories_with_embeddings: &[(Memory, Vec<f32>)],
    ) -> Result<AnalysisResult> {
        let mut result = AnalysisResult {
            analysis_type: "Overview".to_string(),
            total_memories: memories_with_embeddings.len(),
            ..Default::default()
        };

        if memories_with_embeddings.is_empty() {
            return Ok(result);
        }

        // Dimension analysis
        result.embedding_dimension = memories_with_embeddings[0].1.len();

        // Calculate average magnitude
        let magnitudes: Vec<f32> = memories_with_embeddings
            .iter()
            .map(|(_, emb)| emb.iter().map(|x| x * x).sum::<f32>().sqrt())
            .collect();

        result.avg_magnitude = magnitudes.iter().sum::<f32>() / magnitudes.len() as f32;
        result.min_magnitude = magnitudes.iter().cloned().fold(f32::INFINITY, f32::min);
        result.max_magnitude = magnitudes.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

        // Calculate centroid
        let dim = result.embedding_dimension;
        let mut centroid = vec![0.0; dim];
        for (_, embedding) in memories_with_embeddings {
            for (i, val) in embedding.iter().enumerate() {
                centroid[i] += val;
            }
        }
        for val in &mut centroid {
            *val /= memories_with_embeddings.len() as f32;
        }

        // Calculate spread (average distance from centroid)
        let distances: Vec<f32> = memories_with_embeddings
            .iter()
            .map(|(_, emb)| 1.0 - EmbeddingService::cosine_similarity(emb, &centroid))
            .collect();

        result.avg_spread = distances.iter().sum::<f32>() / distances.len() as f32;

        // Memory type distribution
        let mut type_counts = HashMap::new();
        for (memory, _) in memories_with_embeddings {
            *type_counts.entry(memory.memory_type.clone()).or_insert(0) += 1;
        }
        result.type_distribution = type_counts;

        // Coverage percentage
        result.coverage_percentage =
            (memories_with_embeddings.len() as f32 / result.total_memories as f32) * 100.0;

        Ok(result)
    }

    /// Analyze similarity distribution
    fn analyze_similarity(
        &self,
        memories_with_embeddings: &[(Memory, Vec<f32>)],
    ) -> Result<AnalysisResult> {
        let mut result = AnalysisResult {
            analysis_type: "Similarity Distribution".to_string(),
            total_memories: memories_with_embeddings.len(),
            ..Default::default()
        };

        if memories_with_embeddings.len() < 2 {
            return Ok(result);
        }

        // Calculate all pairwise similarities
        let mut similarities = Vec::new();
        for i in 0..memories_with_embeddings.len() {
            for j in (i + 1)..memories_with_embeddings.len() {
                let sim = EmbeddingService::cosine_similarity(
                    &memories_with_embeddings[i].1,
                    &memories_with_embeddings[j].1,
                );
                similarities.push(sim);
            }
        }

        if similarities.is_empty() {
            return Ok(result);
        }

        // Calculate statistics
        similarities.sort_by(|a, b| a.partial_cmp(b).unwrap());
        result.avg_similarity = similarities.iter().sum::<f32>() / similarities.len() as f32;
        result.min_similarity = *similarities.first().unwrap();
        result.max_similarity = *similarities.last().unwrap();
        result.median_similarity = similarities[similarities.len() / 2];

        // Calculate percentiles
        result.percentile_25 = similarities[similarities.len() / 4];
        result.percentile_75 = similarities[similarities.len() * 3 / 4];

        // Create histogram
        if self.histogram {
            result.similarity_histogram = self.create_histogram(&similarities, self.bins);
        }

        // Find most similar pairs
        let mut pairs: Vec<(usize, usize, f32)> = Vec::new();
        for i in 0..memories_with_embeddings.len() {
            for j in (i + 1)..memories_with_embeddings.len() {
                let sim = EmbeddingService::cosine_similarity(
                    &memories_with_embeddings[i].1,
                    &memories_with_embeddings[j].1,
                );
                pairs.push((i, j, sim));
            }
        }
        pairs.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

        // Store top similar pairs
        for (i, j, sim) in pairs.iter().take(5) {
            result.top_similar_pairs.push((
                memories_with_embeddings[*i].0.title.clone(),
                memories_with_embeddings[*j].0.title.clone(),
                *sim,
            ));
        }

        Ok(result)
    }

    /// Analyze density and coverage
    fn analyze_density(
        &self,
        memories_with_embeddings: &[(Memory, Vec<f32>)],
    ) -> Result<AnalysisResult> {
        let mut result = AnalysisResult {
            analysis_type: "Density Analysis".to_string(),
            total_memories: memories_with_embeddings.len(),
            ..Default::default()
        };

        if memories_with_embeddings.is_empty() {
            return Ok(result);
        }

        // Calculate k-nearest neighbor distances
        let k = 5.min(memories_with_embeddings.len() - 1);
        let mut avg_knn_distances = Vec::new();

        for i in 0..memories_with_embeddings.len() {
            let mut distances = Vec::new();
            for j in 0..memories_with_embeddings.len() {
                if i != j {
                    let dist = 1.0
                        - EmbeddingService::cosine_similarity(
                            &memories_with_embeddings[i].1,
                            &memories_with_embeddings[j].1,
                        );
                    distances.push(dist);
                }
            }
            distances.sort_by(|a, b| a.partial_cmp(b).unwrap());

            if distances.len() >= k {
                let avg_knn = distances[..k].iter().sum::<f32>() / k as f32;
                avg_knn_distances.push(avg_knn);
            }
        }

        if !avg_knn_distances.is_empty() {
            result.avg_density =
                1.0 / (avg_knn_distances.iter().sum::<f32>() / avg_knn_distances.len() as f32);

            // Find dense and sparse regions
            let mut density_scores: Vec<(usize, f32)> = avg_knn_distances
                .iter()
                .enumerate()
                .map(|(i, d)| (i, 1.0 / d))
                .collect();

            density_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

            // Dense regions (top 10%)
            let dense_count = (memories_with_embeddings.len() as f32 * 0.1).ceil() as usize;
            for &(idx, _score) in density_scores
                .iter()
                .take(dense_count.min(density_scores.len()))
            {
                result
                    .dense_regions
                    .push(memories_with_embeddings[idx].0.title.clone());
            }

            // Sparse regions (bottom 10%)
            for &(idx, _score) in density_scores
                .iter()
                .skip(density_scores.len().saturating_sub(dense_count))
            {
                result
                    .sparse_regions
                    .push(memories_with_embeddings[idx].0.title.clone());
            }
        }

        Ok(result)
    }

    /// Detect outliers
    fn analyze_outliers(
        &self,
        memories_with_embeddings: &[(Memory, Vec<f32>)],
    ) -> Result<AnalysisResult> {
        let mut result = AnalysisResult {
            analysis_type: "Outlier Detection".to_string(),
            total_memories: memories_with_embeddings.len(),
            ..Default::default()
        };

        if memories_with_embeddings.len() < 3 {
            return Ok(result);
        }

        // Calculate average distance to all other points for each memory
        let mut avg_distances = Vec::new();
        for i in 0..memories_with_embeddings.len() {
            let mut distances = Vec::new();
            for j in 0..memories_with_embeddings.len() {
                if i != j {
                    let dist = 1.0
                        - EmbeddingService::cosine_similarity(
                            &memories_with_embeddings[i].1,
                            &memories_with_embeddings[j].1,
                        );
                    distances.push(dist);
                }
            }
            let avg_dist = distances.iter().sum::<f32>() / distances.len() as f32;
            avg_distances.push((i, avg_dist));
        }

        // Calculate statistics
        let distances_only: Vec<f32> = avg_distances.iter().map(|(_, d)| *d).collect();
        let mean = distances_only.iter().sum::<f32>() / distances_only.len() as f32;
        let variance = distances_only
            .iter()
            .map(|d| (d - mean).powi(2))
            .sum::<f32>()
            / distances_only.len() as f32;
        let std_dev = variance.sqrt();

        // Find outliers (> 2 standard deviations from mean)
        let threshold = mean + 2.0 * std_dev;

        for (i, avg_dist) in avg_distances {
            if avg_dist > threshold {
                result.outliers.push(OutlierInfo {
                    memory_id: memories_with_embeddings[i].0.id.clone(),
                    title: memories_with_embeddings[i].0.title.clone(),
                    distance_score: avg_dist,
                    deviation: (avg_dist - mean) / std_dev,
                });
            }
        }

        result
            .outliers
            .sort_by(|a, b| b.deviation.partial_cmp(&a.deviation).unwrap());
        result.outlier_count = result.outliers.len();
        result.outlier_percentage =
            (result.outlier_count as f32 / memories_with_embeddings.len() as f32) * 100.0;

        Ok(result)
    }

    /// Analyze vocabulary
    fn analyze_vocabulary(&self, memories: &[Memory]) -> Result<AnalysisResult> {
        let mut result = AnalysisResult {
            analysis_type: "Vocabulary Analysis".to_string(),
            total_memories: memories.len(),
            ..Default::default()
        };

        // Count word frequencies
        let mut word_counts = HashMap::new();
        let mut total_words = 0;

        for memory in memories {
            let text = format!("{} {}", memory.title, memory.content);
            for word in text.split_whitespace() {
                let word = word
                    .to_lowercase()
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_string();
                if word.len() > 2 {
                    *word_counts.entry(word).or_insert(0) += 1;
                    total_words += 1;
                }
            }
        }

        result.unique_words = word_counts.len();
        result.total_words = total_words;
        result.vocabulary_richness = result.unique_words as f32 / result.total_words as f32;

        // Find most common words
        let mut word_freq: Vec<_> = word_counts.into_iter().collect();
        word_freq.sort_by(|a, b| b.1.cmp(&a.1));

        result.top_words = word_freq
            .iter()
            .take(20)
            .map(|(word, count)| (word.clone(), *count))
            .collect();

        // Tag analysis
        let mut tag_counts = HashMap::new();
        for memory in memories {
            for tag in &memory.tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }

        let mut tag_freq: Vec<_> = tag_counts.into_iter().collect();
        tag_freq.sort_by(|a, b| b.1.cmp(&a.1));

        result.top_tags = tag_freq
            .iter()
            .take(10)
            .map(|(tag, count)| (tag.clone(), *count))
            .collect();

        Ok(result)
    }

    /// Analyze temporal patterns
    fn analyze_temporal(
        &self,
        memories_with_embeddings: &[(Memory, Vec<f32>)],
    ) -> Result<AnalysisResult> {
        let mut result = AnalysisResult {
            analysis_type: "Temporal Analysis".to_string(),
            total_memories: memories_with_embeddings.len(),
            ..Default::default()
        };

        if memories_with_embeddings.is_empty() {
            return Ok(result);
        }

        // Sort by creation time
        let mut temporal_data = memories_with_embeddings.to_vec();
        temporal_data.sort_by_key(|(m, _)| m.created_at);

        // Analyze embedding drift over time
        let window_size = 10.min(temporal_data.len());
        let mut drift_scores = Vec::new();

        for i in window_size..temporal_data.len() {
            let current = &temporal_data[i].1;
            let previous = &temporal_data[i - window_size].1;
            let drift = 1.0 - EmbeddingService::cosine_similarity(current, previous);
            drift_scores.push(drift);
        }

        if !drift_scores.is_empty() {
            result.avg_temporal_drift =
                drift_scores.iter().sum::<f32>() / drift_scores.len() as f32;
            result.max_temporal_drift = drift_scores
                .iter()
                .cloned()
                .fold(f32::NEG_INFINITY, f32::max);
        }

        // Time-based clustering
        let day_seconds = 24 * 60 * 60;
        let mut daily_groups = HashMap::new();

        for (memory, _) in &temporal_data {
            let day = memory.created_at / day_seconds;
            daily_groups
                .entry(day)
                .or_insert_with(Vec::new)
                .push(memory.clone());
        }

        result.temporal_clusters = daily_groups.len();

        // Find periods of high activity
        let mut daily_counts: Vec<_> = daily_groups
            .iter()
            .map(|(day, memories)| (*day, memories.len()))
            .collect();
        daily_counts.sort_by_key(|(_, count)| *count);

        if !daily_counts.is_empty() {
            let high_activity_threshold = daily_counts[daily_counts.len() * 3 / 4].1;
            result.high_activity_periods = daily_counts
                .iter()
                .filter(|(_, count)| *count >= high_activity_threshold)
                .map(|(day, count)| format!("Day {}: {} memories", day, count))
                .collect();
        }

        Ok(result)
    }

    /// Create histogram from data
    fn create_histogram(&self, data: &[f32], bins: usize) -> Vec<(f32, f32, usize)> {
        if data.is_empty() || bins == 0 {
            return Vec::new();
        }

        let min = *data
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let max = *data
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let range = max - min;
        let bin_width = range / bins as f32;

        let mut histogram = vec![(0.0, 0.0, 0); bins];

        // Index is used for calculations and boundary checks, so range loop is appropriate
        #[allow(clippy::needless_range_loop)]
        for i in 0..bins {
            let start = min + i as f32 * bin_width;
            let end = if i == bins - 1 {
                max
            } else {
                start + bin_width
            };
            histogram[i].0 = start;
            histogram[i].1 = end;
        }

        for value in data {
            let bin_idx = ((value - min) / bin_width).floor() as usize;
            let bin_idx = bin_idx.min(bins - 1);
            histogram[bin_idx].2 += 1;
        }

        histogram
    }

    /// Display analysis results
    fn display_analysis(&self, analysis: &AnalysisResult) -> Result<()> {
        println!("📊 Embedding Analytics: {}", analysis.analysis_type);
        println!();

        match self.analysis {
            AnalysisType::Overview => self.display_overview(analysis)?,
            AnalysisType::Similarity => self.display_similarity(analysis)?,
            AnalysisType::Density => self.display_density(analysis)?,
            AnalysisType::Outliers => self.display_outliers(analysis)?,
            AnalysisType::Vocabulary => self.display_vocabulary(analysis)?,
            AnalysisType::Temporal => self.display_temporal(analysis)?,
        }

        Ok(())
    }

    /// Display overview results
    fn display_overview(&self, analysis: &AnalysisResult) -> Result<()> {
        println!("📈 General Statistics:");
        println!("  Total memories: {}", analysis.total_memories);
        println!("  Embedding dimension: {}", analysis.embedding_dimension);
        println!(
            "  Coverage: {:.1}% have embeddings",
            analysis.coverage_percentage
        );
        println!();

        println!("📐 Vector Space Metrics:");
        println!("  Average magnitude: {:.3}", analysis.avg_magnitude);
        println!(
            "  Magnitude range: {:.3} - {:.3}",
            analysis.min_magnitude, analysis.max_magnitude
        );
        println!("  Average spread: {:.3}", analysis.avg_spread);
        println!();

        if !analysis.type_distribution.is_empty() {
            println!("📂 Type Distribution:");
            for (mem_type, count) in &analysis.type_distribution {
                println!("  {}: {} memories", mem_type, count);
            }
        }

        Ok(())
    }

    /// Display similarity results
    fn display_similarity(&self, analysis: &AnalysisResult) -> Result<()> {
        println!("📊 Similarity Statistics:");
        println!("  Average similarity: {:.3}", analysis.avg_similarity);
        println!(
            "  Similarity range: {:.3} - {:.3}",
            analysis.min_similarity, analysis.max_similarity
        );
        println!("  Median similarity: {:.3}", analysis.median_similarity);
        println!("  25th percentile: {:.3}", analysis.percentile_25);
        println!("  75th percentile: {:.3}", analysis.percentile_75);
        println!();

        if !analysis.top_similar_pairs.is_empty() {
            println!("🔗 Most Similar Pairs:");
            for (topic1, topic2, sim) in &analysis.top_similar_pairs {
                println!("  {:.3}: {} ↔ {}", sim, topic1, topic2);
            }
            println!();
        }

        if self.histogram && !analysis.similarity_histogram.is_empty() {
            println!("📊 Similarity Distribution:");
            for (start, end, count) in &analysis.similarity_histogram {
                let bar =
                    "█".repeat((*count as f32 / analysis.total_memories as f32 * 50.0) as usize);
                println!("  [{:.2}-{:.2}): {} ({})", start, end, bar, count);
            }
        }

        Ok(())
    }

    /// Display density results
    fn display_density(&self, analysis: &AnalysisResult) -> Result<()> {
        println!("🗺️  Density Analysis:");
        println!("  Average density score: {:.3}", analysis.avg_density);
        println!();

        if !analysis.dense_regions.is_empty() {
            println!("🔴 Dense Regions (top 10%):");
            for topic in &analysis.dense_regions {
                println!("  - {}", topic);
            }
            println!();
        }

        if !analysis.sparse_regions.is_empty() {
            println!("⚪ Sparse Regions (bottom 10%):");
            for topic in &analysis.sparse_regions {
                println!("  - {}", topic);
            }
        }

        Ok(())
    }

    /// Display outlier results
    fn display_outliers(&self, analysis: &AnalysisResult) -> Result<()> {
        println!("🎯 Outlier Detection:");
        println!(
            "  Outliers found: {} ({:.1}%)",
            analysis.outlier_count, analysis.outlier_percentage
        );
        println!();

        if !analysis.outliers.is_empty() {
            println!("📍 Top Outliers:");
            for outlier in analysis.outliers.iter().take(10) {
                println!(
                    "  {:.1}σ: {} (distance: {:.3})",
                    outlier.deviation, outlier.title, outlier.distance_score
                );
                if self.verbose {
                    println!("    ID: {}", &outlier.memory_id[..8]);
                }
            }
        }

        Ok(())
    }

    /// Display vocabulary results
    fn display_vocabulary(&self, analysis: &AnalysisResult) -> Result<()> {
        println!("📚 Vocabulary Statistics:");
        println!("  Unique words: {}", analysis.unique_words);
        println!("  Total words: {}", analysis.total_words);
        println!("  Vocabulary richness: {:.3}", analysis.vocabulary_richness);
        println!();

        if !analysis.top_words.is_empty() {
            println!("🔤 Most Common Words:");
            for (i, (word, count)) in analysis.top_words.iter().enumerate().take(10) {
                println!("  {}. {} ({})", i + 1, word, count);
            }
            println!();
        }

        if !analysis.top_tags.is_empty() {
            println!("🏷️  Most Common Tags:");
            for (tag, count) in &analysis.top_tags {
                println!("  {} ({})", tag, count);
            }
        }

        Ok(())
    }

    /// Display temporal results
    fn display_temporal(&self, analysis: &AnalysisResult) -> Result<()> {
        println!("⏰ Temporal Analysis:");
        println!("  Temporal clusters: {}", analysis.temporal_clusters);
        println!("  Average drift: {:.3}", analysis.avg_temporal_drift);
        println!("  Maximum drift: {:.3}", analysis.max_temporal_drift);
        println!();

        if !analysis.high_activity_periods.is_empty() {
            println!("📈 High Activity Periods:");
            for period in &analysis.high_activity_periods {
                println!("  - {}", period);
            }
        }

        Ok(())
    }

    /// Export analysis to JSON
    fn export_analysis(&self, analysis: &AnalysisResult, path: &PathBuf) -> Result<()> {
        let json = serde_json::to_string_pretty(analysis)?;
        std::fs::write(path, json)?;
        println!("\n✅ Analysis exported to: {}", path.display());
        Ok(())
    }
}

/// Analysis result structure
#[derive(Default, serde::Serialize)]
struct AnalysisResult {
    analysis_type: String,
    total_memories: usize,

    // Overview metrics
    embedding_dimension: usize,
    coverage_percentage: f32,
    avg_magnitude: f32,
    min_magnitude: f32,
    max_magnitude: f32,
    avg_spread: f32,
    type_distribution: HashMap<MemoryType, usize>,

    // Similarity metrics
    avg_similarity: f32,
    min_similarity: f32,
    max_similarity: f32,
    median_similarity: f32,
    percentile_25: f32,
    percentile_75: f32,
    similarity_histogram: Vec<(f32, f32, usize)>,
    top_similar_pairs: Vec<(String, String, f32)>,

    // Density metrics
    avg_density: f32,
    dense_regions: Vec<String>,
    sparse_regions: Vec<String>,

    // Outlier metrics
    outlier_count: usize,
    outlier_percentage: f32,
    outliers: Vec<OutlierInfo>,

    // Vocabulary metrics
    unique_words: usize,
    total_words: usize,
    vocabulary_richness: f32,
    top_words: Vec<(String, usize)>,
    top_tags: Vec<(String, usize)>,

    // Temporal metrics
    temporal_clusters: usize,
    avg_temporal_drift: f32,
    max_temporal_drift: f32,
    high_activity_periods: Vec<String>,
}

#[derive(serde::Serialize)]
struct OutlierInfo {
    memory_id: String,
    title: String,
    distance_score: f32,
    deviation: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_histogram_creation() {
        let cmd = EmbedAnalyticsCommand {
            db_path: None,
            r#type: None,
            analysis: AnalysisType::Overview,
            limit: 100,
            verbose: false,
            export: None,
            histogram: true,
            bins: 5,
        };

        let data = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
        let histogram = cmd.create_histogram(&data, 5);

        assert_eq!(histogram.len(), 5);
        assert_eq!(
            histogram.iter().map(|(_, _, c)| c).sum::<usize>(),
            data.len()
        );
    }
}
