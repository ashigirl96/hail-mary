use crate::mcp::server::get_default_db_path;
use crate::memory::{
    embeddings::EmbeddingService,
    models::{Memory, MemoryType},
    repository::{MemoryRepository, SqliteMemoryRepository},
};
use crate::utils::error::Result;
use clap::Args;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Cluster memories into groups based on semantic similarity
#[derive(Args)]
pub struct ClusterCommand {
    /// Path to the database file (defaults to ~/.local/share/hail-mary/memory.db)
    #[arg(long, value_name = "PATH")]
    pub db_path: Option<PathBuf>,

    /// Memory type to cluster (if not specified, all types)
    #[arg(long, value_enum)]
    pub r#type: Option<MemoryType>,

    /// Clustering algorithm
    #[arg(long, value_enum, default_value = "kmeans")]
    pub algorithm: ClusterAlgorithm,

    /// Number of clusters for k-means
    #[arg(long, default_value = "5")]
    pub num_clusters: usize,

    /// Minimum similarity for hierarchical clustering (0.0 to 1.0)
    #[arg(long, default_value = "0.7")]
    pub min_similarity: f32,

    /// Maximum memories to cluster
    #[arg(long, default_value = "1000")]
    pub limit: usize,

    /// Show detailed output
    #[arg(long, short)]
    pub verbose: bool,

    /// Output format
    #[arg(long, value_enum, default_value = "text")]
    pub format: OutputFormat,

    /// Export clusters to JSON file
    #[arg(long)]
    pub export: Option<PathBuf>,

    /// Show cluster statistics
    #[arg(long)]
    pub stats: bool,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum ClusterAlgorithm {
    /// K-means clustering
    Kmeans,
    /// Hierarchical clustering
    Hierarchical,
    /// DBSCAN clustering
    Dbscan,
    /// Topic-based clustering
    Topic,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    /// Plain text output
    Text,
    /// JSON output
    Json,
    /// Summary only
    Summary,
}

impl ClusterCommand {
    /// Execute the cluster command
    pub fn execute(self) -> Result<()> {
        // Validate parameters
        if self.min_similarity < 0.0 || self.min_similarity > 1.0 {
            eprintln!("Error: Minimum similarity must be between 0.0 and 1.0");
            return Ok(());
        }

        if self.num_clusters < 2 {
            eprintln!("Error: Number of clusters must be at least 2");
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
            let clusters = self.perform_clustering(&db_path).await?;

            if clusters.is_empty() {
                println!("No clusters found. Try adjusting parameters or adding more memories.");
                return Ok(());
            }

            match self.format {
                OutputFormat::Text => self.display_text(&clusters)?,
                OutputFormat::Json => self.display_json(&clusters)?,
                OutputFormat::Summary => self.display_summary(&clusters)?,
            }

            if let Some(ref export_path) = self.export {
                self.export_clusters(&clusters, export_path)?;
            }

            if self.stats {
                self.display_statistics(&clusters)?;
            }

            Ok(())
        })
    }

    /// Perform clustering based on selected algorithm
    async fn perform_clustering(&self, db_path: &PathBuf) -> Result<Vec<Cluster>> {
        let repository = SqliteMemoryRepository::new(db_path)?;

        // Load memories
        let memories = if let Some(ref memory_type) = self.r#type {
            repository.browse_by_type(memory_type, self.limit)?
        } else {
            repository.browse_all(self.limit)?
        };

        if memories.len() < 2 {
            return Ok(Vec::new());
        }

        if self.verbose {
            println!(
                "📊 Clustering {} memories using {:?} algorithm...",
                memories.len(),
                self.algorithm
            );
        }

        // Generate embeddings
        let embedding_service = EmbeddingService::new()?;
        let texts: Vec<String> = memories
            .iter()
            .map(|m| format!("{} {}", m.topic, m.content))
            .collect();

        let embeddings = embedding_service.embed_texts(texts).await?;

        // Perform clustering
        let clusters = match self.algorithm {
            ClusterAlgorithm::Kmeans => self.kmeans_clustering(&memories, &embeddings)?,
            ClusterAlgorithm::Hierarchical => {
                self.hierarchical_clustering(&memories, &embeddings)?
            }
            ClusterAlgorithm::Dbscan => self.dbscan_clustering(&memories, &embeddings)?,
            ClusterAlgorithm::Topic => self.topic_clustering(&memories)?,
        };

        Ok(clusters)
    }

    /// K-means clustering
    fn kmeans_clustering(
        &self,
        memories: &[Memory],
        embeddings: &[Vec<f32>],
    ) -> Result<Vec<Cluster>> {
        let k = self.num_clusters.min(memories.len());

        // Initialize centroids randomly
        let mut centroids: Vec<Vec<f32>> = Vec::new();
        let mut used_indices = HashSet::new();

        while centroids.len() < k {
            let idx = rand::random::<usize>() % memories.len();
            if used_indices.insert(idx) {
                centroids.push(embeddings[idx].clone());
            }
        }

        // Iterate until convergence (max 100 iterations)
        for iteration in 0..100 {
            // Assign points to clusters
            let mut assignments = vec![0; memories.len()];
            for (i, embedding) in embeddings.iter().enumerate() {
                let mut min_dist = f32::MAX;
                let mut closest_cluster = 0;

                for (j, centroid) in centroids.iter().enumerate() {
                    let dist = 1.0 - EmbeddingService::cosine_similarity(embedding, centroid);
                    if dist < min_dist {
                        min_dist = dist;
                        closest_cluster = j;
                    }
                }
                assignments[i] = closest_cluster;
            }

            // Update centroids
            let mut new_centroids = vec![vec![0.0; embeddings[0].len()]; k];
            let mut counts = vec![0; k];

            for (i, &cluster_id) in assignments.iter().enumerate() {
                for (j, &val) in embeddings[i].iter().enumerate() {
                    new_centroids[cluster_id][j] += val;
                }
                counts[cluster_id] += 1;
            }

            // Average the centroids
            for (i, count) in counts.iter().enumerate() {
                if *count > 0 {
                    for j in 0..new_centroids[i].len() {
                        new_centroids[i][j] /= *count as f32;
                    }
                }
            }

            // Check convergence
            let mut converged = true;
            for (old, new) in centroids.iter().zip(&new_centroids) {
                if EmbeddingService::cosine_similarity(old, new) < 0.999 {
                    converged = false;
                    break;
                }
            }

            centroids = new_centroids;

            if converged {
                if self.verbose {
                    println!("  Converged after {} iterations", iteration + 1);
                }
                break;
            }
        }

        // Create final clusters
        let mut clusters: Vec<Cluster> = (0..k)
            .map(|i| Cluster::new(format!("Cluster {}", i + 1)))
            .collect();

        for (i, memory) in memories.iter().enumerate() {
            let mut min_dist = f32::MAX;
            let mut closest_cluster = 0;

            for (j, centroid) in centroids.iter().enumerate() {
                let dist = 1.0 - EmbeddingService::cosine_similarity(&embeddings[i], centroid);
                if dist < min_dist {
                    min_dist = dist;
                    closest_cluster = j;
                }
            }

            clusters[closest_cluster].members.push(memory.clone());
            clusters[closest_cluster]
                .embeddings
                .push(embeddings[i].clone());
        }

        // Remove empty clusters and calculate centroids
        clusters.retain(|c| !c.members.is_empty());
        for cluster in &mut clusters {
            cluster.calculate_centroid();
            cluster.calculate_coherence();
        }

        Ok(clusters)
    }

    /// Hierarchical clustering
    fn hierarchical_clustering(
        &self,
        memories: &[Memory],
        embeddings: &[Vec<f32>],
    ) -> Result<Vec<Cluster>> {
        let mut clusters: Vec<Cluster> = memories
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let mut cluster = Cluster::new(format!("Memory_{}", i));
                cluster.members.push(m.clone());
                cluster.embeddings.push(embeddings[i].clone());
                cluster
            })
            .collect();

        // Merge clusters until similarity threshold is not met
        loop {
            let mut best_pair = None;
            let mut best_similarity = 0.0;

            // Find most similar pair of clusters
            for i in 0..clusters.len() {
                for j in (i + 1)..clusters.len() {
                    let sim = self.cluster_similarity(&clusters[i], &clusters[j]);
                    if sim > best_similarity && sim >= self.min_similarity {
                        best_similarity = sim;
                        best_pair = Some((i, j));
                    }
                }
            }

            // Merge if found
            if let Some((i, j)) = best_pair {
                let cluster_j = clusters.remove(j);
                clusters[i].merge(cluster_j);
                clusters[i].name = format!("Cluster_{}", clusters.len());
            } else {
                break;
            }
        }

        // Rename clusters and calculate properties
        for (i, cluster) in clusters.iter_mut().enumerate() {
            cluster.name = format!("Cluster {}", i + 1);
            cluster.calculate_centroid();
            cluster.calculate_coherence();
        }

        Ok(clusters)
    }

    /// DBSCAN clustering
    fn dbscan_clustering(
        &self,
        memories: &[Memory],
        embeddings: &[Vec<f32>],
    ) -> Result<Vec<Cluster>> {
        let eps = 1.0 - self.min_similarity; // Convert similarity to distance
        let min_points = 2;

        let mut visited = vec![false; memories.len()];
        let mut cluster_assignments = vec![None; memories.len()];
        let mut clusters = Vec::new();

        for i in 0..memories.len() {
            if visited[i] {
                continue;
            }

            visited[i] = true;

            // Find neighbors
            let mut neighbors = Vec::new();
            for j in 0..memories.len() {
                if i != j {
                    let dist =
                        1.0 - EmbeddingService::cosine_similarity(&embeddings[i], &embeddings[j]);
                    if dist <= eps {
                        neighbors.push(j);
                    }
                }
            }

            if neighbors.len() >= min_points {
                // Create new cluster
                let cluster_id = clusters.len();
                let mut cluster = Cluster::new(format!("Cluster {}", cluster_id + 1));
                cluster.members.push(memories[i].clone());
                cluster.embeddings.push(embeddings[i].clone());
                cluster_assignments[i] = Some(cluster_id);

                // Expand cluster
                let mut neighbor_idx = 0;
                while neighbor_idx < neighbors.len() {
                    let n = neighbors[neighbor_idx];

                    if !visited[n] {
                        visited[n] = true;

                        // Find neighbors of neighbor
                        let mut n_neighbors = Vec::new();
                        for j in 0..memories.len() {
                            if n != j {
                                let dist = 1.0
                                    - EmbeddingService::cosine_similarity(
                                        &embeddings[n],
                                        &embeddings[j],
                                    );
                                if dist <= eps {
                                    n_neighbors.push(j);
                                }
                            }
                        }

                        if n_neighbors.len() >= min_points {
                            for nn in n_neighbors {
                                if !neighbors.contains(&nn) {
                                    neighbors.push(nn);
                                }
                            }
                        }
                    }

                    if cluster_assignments[n].is_none() {
                        cluster.members.push(memories[n].clone());
                        cluster.embeddings.push(embeddings[n].clone());
                        cluster_assignments[n] = Some(cluster_id);
                    }

                    neighbor_idx += 1;
                }

                cluster.calculate_centroid();
                cluster.calculate_coherence();
                clusters.push(cluster);
            }
        }

        // Add noise points as individual clusters if verbose
        if self.verbose {
            let noise_count = cluster_assignments.iter().filter(|a| a.is_none()).count();
            if noise_count > 0 {
                println!("  Found {} noise points (not in any cluster)", noise_count);
            }
        }

        Ok(clusters)
    }

    /// Topic-based clustering
    fn topic_clustering(&self, memories: &[Memory]) -> Result<Vec<Cluster>> {
        let mut topic_clusters: HashMap<String, Cluster> = HashMap::new();

        for memory in memories {
            // Extract main topic (first significant word)
            let topic_key = self.extract_topic_key(&memory.topic);

            let cluster = topic_clusters
                .entry(topic_key.clone())
                .or_insert_with(|| Cluster::new(topic_key));

            cluster.members.push(memory.clone());
        }

        let mut clusters: Vec<Cluster> = topic_clusters.into_values().collect();
        clusters.sort_by(|a, b| b.members.len().cmp(&a.members.len()));

        // Rename clusters
        for (i, cluster) in clusters.iter_mut().enumerate() {
            let topic_name = cluster.name.clone();
            cluster.name = format!("Topic: {} (Cluster {})", topic_name, i + 1);
        }

        Ok(clusters)
    }

    /// Calculate similarity between two clusters
    fn cluster_similarity(&self, c1: &Cluster, c2: &Cluster) -> f32 {
        if c1.embeddings.is_empty() || c2.embeddings.is_empty() {
            return 0.0;
        }

        // Average linkage
        let mut total_sim = 0.0;
        let mut count = 0;

        for e1 in &c1.embeddings {
            for e2 in &c2.embeddings {
                total_sim += EmbeddingService::cosine_similarity(e1, e2);
                count += 1;
            }
        }

        if count > 0 {
            total_sim / count as f32
        } else {
            0.0
        }
    }

    /// Extract topic key from topic string
    fn extract_topic_key(&self, topic: &str) -> String {
        let words: Vec<&str> = topic.split_whitespace().collect();

        // Find key technical terms
        for word in &words {
            let lower = word.to_lowercase();
            if lower.len() > 3
                && !["with", "from", "using", "about", "the"].contains(&lower.as_str())
            {
                return word.to_string();
            }
        }

        words.first().unwrap_or(&"Unknown").to_string()
    }

    /// Display clusters in text format
    fn display_text(&self, clusters: &[Cluster]) -> Result<()> {
        println!("📊 Found {} clusters", clusters.len());
        println!();

        for cluster in clusters {
            println!("🔹 {} ({} members)", cluster.name, cluster.members.len());

            if self.verbose {
                println!("  Coherence: {:.2}", cluster.coherence);

                // Show top topics
                let topics: Vec<String> = cluster
                    .members
                    .iter()
                    .take(5)
                    .map(|m| m.topic.clone())
                    .collect();

                for topic in topics {
                    println!("  - {}", topic);
                }

                if cluster.members.len() > 5 {
                    println!("  ... and {} more", cluster.members.len() - 5);
                }

                // Show common tags
                let mut tag_counts: HashMap<String, usize> = HashMap::new();
                for member in &cluster.members {
                    for tag in &member.tags {
                        *tag_counts.entry(tag.clone()).or_insert(0) += 1;
                    }
                }

                let mut common_tags: Vec<_> = tag_counts.into_iter().collect();
                common_tags.sort_by(|a, b| b.1.cmp(&a.1));

                if !common_tags.is_empty() {
                    let tags: Vec<String> = common_tags
                        .iter()
                        .take(3)
                        .map(|(tag, count)| format!("{} ({})", tag, count))
                        .collect();
                    println!("  Common tags: {}", tags.join(", "));
                }
            }

            println!();
        }

        Ok(())
    }

    /// Display clusters in JSON format
    fn display_json(&self, clusters: &[Cluster]) -> Result<()> {
        let json = serde_json::json!({
            "algorithm": format!("{:?}", self.algorithm),
            "total_clusters": clusters.len(),
            "clusters": clusters.iter().map(|c| {
                serde_json::json!({
                    "name": c.name,
                    "size": c.members.len(),
                    "coherence": c.coherence,
                    "members": c.members.iter().map(|m| {
                        serde_json::json!({
                            "id": m.id,
                            "topic": m.topic,
                            "type": m.memory_type.to_string(),
                            "confidence": m.confidence,
                        })
                    }).collect::<Vec<_>>()
                })
            }).collect::<Vec<_>>()
        });

        println!("{}", serde_json::to_string_pretty(&json)?);
        Ok(())
    }

    /// Display cluster summary
    fn display_summary(&self, clusters: &[Cluster]) -> Result<()> {
        println!("📈 Clustering Summary");
        println!("  Algorithm: {:?}", self.algorithm);
        println!("  Total clusters: {}", clusters.len());

        let total_memories: usize = clusters.iter().map(|c| c.members.len()).sum();
        println!("  Total memories: {}", total_memories);

        let avg_size = total_memories as f32 / clusters.len() as f32;
        println!("  Average cluster size: {:.1}", avg_size);

        let largest = clusters.iter().max_by_key(|c| c.members.len());
        let smallest = clusters.iter().min_by_key(|c| c.members.len());

        if let Some(largest) = largest {
            println!(
                "  Largest cluster: {} ({} members)",
                largest.name,
                largest.members.len()
            );
        }

        if let Some(smallest) = smallest {
            println!(
                "  Smallest cluster: {} ({} members)",
                smallest.name,
                smallest.members.len()
            );
        }

        let avg_coherence: f32 =
            clusters.iter().map(|c| c.coherence).sum::<f32>() / clusters.len() as f32;
        println!("  Average coherence: {:.2}", avg_coherence);

        Ok(())
    }

    /// Display detailed statistics
    fn display_statistics(&self, clusters: &[Cluster]) -> Result<()> {
        println!("\n📊 Detailed Statistics");

        // Size distribution
        println!("\nCluster Size Distribution:");
        let mut size_buckets = HashMap::new();
        for cluster in clusters {
            let bucket = match cluster.members.len() {
                1 => "1",
                2..=5 => "2-5",
                6..=10 => "6-10",
                11..=20 => "11-20",
                _ => "20+",
            };
            *size_buckets.entry(bucket).or_insert(0) += 1;
        }

        for (bucket, count) in size_buckets.iter() {
            println!("  {}: {} clusters", bucket, count);
        }

        // Type distribution
        println!("\nMemory Type Distribution:");
        let mut type_counts = HashMap::new();
        for cluster in clusters {
            for member in &cluster.members {
                *type_counts.entry(member.memory_type.clone()).or_insert(0) += 1;
            }
        }

        for (mem_type, count) in type_counts.iter() {
            println!("  {}: {} memories", mem_type, count);
        }

        // Confidence distribution
        let all_confidences: Vec<f32> = clusters
            .iter()
            .flat_map(|c| c.members.iter().map(|m| m.confidence))
            .collect();

        if !all_confidences.is_empty() {
            let avg_confidence = all_confidences.iter().sum::<f32>() / all_confidences.len() as f32;
            let min_confidence = all_confidences
                .iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();
            let max_confidence = all_confidences
                .iter()
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();

            println!("\nConfidence Statistics:");
            println!("  Average: {:.2}", avg_confidence);
            println!("  Min: {:.2}", min_confidence);
            println!("  Max: {:.2}", max_confidence);
        }

        Ok(())
    }

    /// Export clusters to file
    fn export_clusters(&self, clusters: &[Cluster], path: &PathBuf) -> Result<()> {
        let export_data = serde_json::json!({
            "metadata": {
                "algorithm": format!("{:?}", self.algorithm),
                "parameters": {
                    "num_clusters": self.num_clusters,
                    "min_similarity": self.min_similarity,
                },
                "timestamp": chrono::Utc::now().to_rfc3339(),
            },
            "clusters": clusters.iter().map(|c| c.to_json()).collect::<Vec<_>>(),
        });

        std::fs::write(path, serde_json::to_string_pretty(&export_data)?)?;
        println!("✅ Clusters exported to: {}", path.display());

        Ok(())
    }
}

/// A cluster of memories
struct Cluster {
    name: String,
    members: Vec<Memory>,
    embeddings: Vec<Vec<f32>>,
    centroid: Option<Vec<f32>>,
    coherence: f32,
}

impl Cluster {
    fn new(name: String) -> Self {
        Self {
            name,
            members: Vec::new(),
            embeddings: Vec::new(),
            centroid: None,
            coherence: 0.0,
        }
    }

    fn merge(&mut self, other: Cluster) {
        self.members.extend(other.members);
        self.embeddings.extend(other.embeddings);
        self.centroid = None;
        self.coherence = 0.0;
    }

    fn calculate_centroid(&mut self) {
        if self.embeddings.is_empty() {
            return;
        }

        let dim = self.embeddings[0].len();
        let mut centroid = vec![0.0; dim];

        for embedding in &self.embeddings {
            for (i, val) in embedding.iter().enumerate() {
                centroid[i] += val;
            }
        }

        for val in &mut centroid {
            *val /= self.embeddings.len() as f32;
        }

        self.centroid = Some(centroid);
    }

    fn calculate_coherence(&mut self) {
        if self.embeddings.len() < 2 {
            self.coherence = 1.0;
            return;
        }

        let mut total_sim = 0.0;
        let mut count = 0;

        for i in 0..self.embeddings.len() {
            for j in (i + 1)..self.embeddings.len() {
                total_sim +=
                    EmbeddingService::cosine_similarity(&self.embeddings[i], &self.embeddings[j]);
                count += 1;
            }
        }

        self.coherence = if count > 0 {
            total_sim / count as f32
        } else {
            0.0
        };
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "size": self.members.len(),
            "coherence": self.coherence,
            "members": self.members.iter().map(|m| {
                serde_json::json!({
                    "id": m.id,
                    "topic": m.topic,
                    "type": m.memory_type.to_string(),
                    "tags": m.tags,
                    "confidence": m.confidence,
                    "references": m.reference_count,
                })
            }).collect::<Vec<_>>(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_topic_key() {
        let cmd = ClusterCommand {
            db_path: None,
            r#type: None,
            algorithm: ClusterAlgorithm::Topic,
            num_clusters: 5,
            min_similarity: 0.7,
            limit: 100,
            verbose: false,
            format: OutputFormat::Text,
            export: None,
            stats: false,
        };

        assert_eq!(cmd.extract_topic_key("Rust programming language"), "Rust");
        assert_eq!(cmd.extract_topic_key("The Python ecosystem"), "Python");
        assert_eq!(cmd.extract_topic_key("Using Docker containers"), "Docker");
    }
}
