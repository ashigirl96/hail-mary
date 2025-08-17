use anyhow::Result;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Configuration for embedding service
#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
    /// Dimension of the embedding vectors
    pub dimension: usize,
    /// Model name for tracking
    pub model_name: String,
    /// Similarity threshold for duplicate detection
    #[allow(dead_code)] // Reserved for future duplicate detection functionality
    pub similarity_threshold: f32,
    /// Batch size for processing
    pub batch_size: usize,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            dimension: 384, // Standard dimension for small models
            model_name: "simple-tfidf-v1".to_string(),
            similarity_threshold: 0.9,
            batch_size: 32,
        }
    }
}

/// Service for generating text embeddings
/// Using an enhanced TF-IDF approach with vocabulary tracking
pub struct EmbeddingService {
    config: EmbeddingConfig,
    /// Vocabulary for consistent feature extraction
    vocabulary: Arc<RwLock<HashMap<String, usize>>>,
    /// IDF weights for terms
    idf_weights: Arc<RwLock<HashMap<String, f32>>>,
    /// Document count for IDF calculation
    doc_count: Arc<RwLock<usize>>,
}

impl EmbeddingService {
    /// Create a new embedding service with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(EmbeddingConfig::default())
    }

    /// Create a new embedding service with custom configuration
    pub fn with_config(config: EmbeddingConfig) -> Result<Self> {
        Ok(Self {
            config,
            vocabulary: Arc::new(RwLock::new(HashMap::new())),
            idf_weights: Arc::new(RwLock::new(HashMap::new())),
            doc_count: Arc::new(RwLock::new(0)),
        })
    }

    /// Get the configuration
    #[allow(dead_code)] // Reserved for future configuration access
    pub fn config(&self) -> &EmbeddingConfig {
        &self.config
    }

    /// Get the model name
    pub fn model_name(&self) -> &str {
        &self.config.model_name
    }

    /// Get the embedding dimension
    #[allow(dead_code)] // Reserved for future dimension queries
    pub fn dimension(&self) -> usize {
        self.config.dimension
    }

    /// Generate embeddings for a list of texts using an enhanced TF-IDF approach
    pub async fn embed_texts(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        // Process in batches for efficiency
        let mut embeddings = Vec::with_capacity(texts.len());

        for batch in texts.chunks(self.config.batch_size) {
            let batch_embeddings = self.process_batch(batch).await?;
            embeddings.extend(batch_embeddings);
        }

        Ok(embeddings)
    }

    /// Process a batch of texts
    async fn process_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::new();

        // Update vocabulary and IDF weights
        self.update_vocabulary(texts).await;

        for text in texts {
            let embedding = self.enhanced_embedding(text).await;
            embeddings.push(embedding);
        }

        Ok(embeddings)
    }

    /// Update vocabulary and IDF weights from new texts
    async fn update_vocabulary(&self, texts: &[String]) {
        let mut vocab = self.vocabulary.write().await;
        let mut idf = self.idf_weights.write().await;
        let mut doc_count = self.doc_count.write().await;

        for text in texts {
            *doc_count += 1;
            let tokens = self.tokenize(text);
            let mut seen = std::collections::HashSet::new();

            for token in tokens {
                if !seen.contains(&token) {
                    seen.insert(token.clone());
                    *idf.entry(token.clone()).or_insert(0.0) += 1.0;
                }

                if !vocab.contains_key(&token) {
                    let idx = vocab.len();
                    vocab.insert(token, idx);
                }
            }
        }

        // Update IDF weights
        let n = *doc_count as f32;
        for (_, weight) in idf.iter_mut() {
            *weight = (n / (*weight + 1.0)).ln();
        }
    }

    /// Generate embedding for a single text
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        Ok(self.enhanced_embedding(text).await)
    }

    /// Tokenize text into terms
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty() && s.len() > 2)
            .map(|s| s.to_string())
            .collect()
    }

    /// Enhanced embedding using TF-IDF with n-grams
    async fn enhanced_embedding(&self, text: &str) -> Vec<f32> {
        let mut embedding = vec![0.0; self.config.dimension];
        let tokens = self.tokenize(text);

        if tokens.is_empty() {
            return embedding;
        }

        let _vocab = self.vocabulary.read().await;
        let idf = self.idf_weights.read().await;

        // Calculate term frequencies
        let mut tf_map = HashMap::new();
        for token in &tokens {
            *tf_map.entry(token.clone()).or_insert(0.0) += 1.0;
        }

        // Apply TF-IDF weighting
        for (token, tf) in tf_map {
            let tf_normalized = tf / tokens.len() as f32;
            let idf_weight = idf.get(&token).unwrap_or(&1.0);
            let tfidf = tf_normalized * idf_weight;

            // Hash to multiple positions for better distribution
            for i in 0..3 {
                let hash = self.hash_string_with_seed(&token, i);
                let index = (hash as usize) % self.config.dimension;
                embedding[index] += tfidf;
            }

            // Add character n-grams for robustness
            if token.len() >= 3 {
                for window in token.chars().collect::<Vec<_>>().windows(3) {
                    let trigram: String = window.iter().collect();
                    let hash = self.hash_string(&trigram);
                    let index = (hash as usize) % self.config.dimension;
                    embedding[index] += tfidf * 0.5; // Lower weight for n-grams
                }
            }
        }

        // L2 normalization
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for value in &mut embedding {
                *value /= norm;
            }
        }

        embedding
    }

    /// Hash string with seed for better distribution
    fn hash_string_with_seed(&self, s: &str, seed: u64) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        seed.hash(&mut hasher);
        s.hash(&mut hasher);
        hasher.finish()
    }

    /// Simple hash function for strings
    fn hash_string(&self, s: &str) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }

    /// Find similar embeddings given a query embedding
    #[allow(dead_code)] // Reserved for future similarity search functionality
    pub fn find_similar(
        &self,
        query: &[f32],
        candidates: &[(String, Vec<f32>)],
        top_k: usize,
        min_similarity: f32,
    ) -> Vec<(String, f32)> {
        let mut similarities: Vec<(String, f32)> = candidates
            .iter()
            .map(|(id, embedding)| {
                let similarity = Self::cosine_similarity(query, embedding);
                (id.clone(), similarity)
            })
            .filter(|(_, sim)| *sim >= min_similarity)
            .collect();

        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        similarities.truncate(top_k);
        similarities
    }

    /// Check if two embeddings are potential duplicates
    #[allow(dead_code)] // Reserved for future duplicate detection functionality
    pub fn are_duplicates(&self, a: &[f32], b: &[f32]) -> bool {
        Self::cosine_similarity(a, b) >= self.config.similarity_threshold
    }

    /// Calculate cosine similarity between two embeddings
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }

    /// Convert embedding to bytes for storage
    pub fn embedding_to_bytes(embedding: &[f32]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(embedding.len() * 4);
        for &value in embedding {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes
    }

    /// Convert bytes back to embedding
    pub fn bytes_to_embedding(bytes: &[u8]) -> Result<Vec<f32>> {
        if bytes.len() % 4 != 0 {
            anyhow::bail!("Invalid embedding bytes length");
        }

        let mut embedding = Vec::with_capacity(bytes.len() / 4);
        for chunk in bytes.chunks_exact(4) {
            let array: [u8; 4] = chunk.try_into()?;
            embedding.push(f32::from_le_bytes(array));
        }

        Ok(embedding)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((EmbeddingService::cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0, 0.0];
        assert!((EmbeddingService::cosine_similarity(&a, &c) - 0.0).abs() < 0.001);

        let d = vec![-1.0, 0.0, 0.0];
        assert!((EmbeddingService::cosine_similarity(&a, &d) - -1.0).abs() < 0.001);
    }

    #[test]
    fn test_embedding_bytes_conversion() {
        let embedding = vec![1.0, 2.0, 3.0, 4.0];
        let bytes = EmbeddingService::embedding_to_bytes(&embedding);
        let recovered = EmbeddingService::bytes_to_embedding(&bytes).unwrap();

        assert_eq!(embedding.len(), recovered.len());
        for (original, recovered) in embedding.iter().zip(recovered.iter()) {
            assert!((original - recovered).abs() < 0.001);
        }
    }
}
