use crate::mcp::server::get_default_db_path;
use crate::memory::{
    embeddings::EmbeddingService,
    models::{Memory, MemoryType},
    repository::{MemoryRepository, SqliteMemoryRepository},
};
use crate::utils::error::{HailMaryError, Result};
use clap::Args;
use rusqlite::{Connection, params};
use std::collections::HashMap;
use std::path::PathBuf;

/// Manage embedding indices for performance optimization
#[derive(Args)]
pub struct IndexCommand {
    /// Path to the database file (defaults to ~/.local/share/hail-mary/memory.db)
    #[arg(long, value_name = "PATH")]
    pub db_path: Option<PathBuf>,

    /// Index operation to perform
    #[arg(long, value_enum, default_value = "build")]
    pub operation: IndexOperation,

    /// Memory type to index (if not specified, all types)
    #[arg(long, value_enum)]
    pub r#type: Option<MemoryType>,

    /// Batch size for index building
    #[arg(long, default_value = "100")]
    pub batch_size: usize,

    /// Enable quantization for reduced memory usage
    #[arg(long)]
    pub quantize: bool,

    /// Clear cache entries older than N days
    #[arg(long)]
    pub clear_cache_days: Option<i64>,

    /// Show detailed output
    #[arg(long, short)]
    pub verbose: bool,

    /// Force rebuild even if index exists
    #[arg(long)]
    pub force: bool,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum IndexOperation {
    /// Build or rebuild the embedding index
    Build,
    /// Update index for new or modified memories
    Update,
    /// Show index statistics
    Stats,
    /// Clear the embedding cache
    ClearCache,
    /// Optimize indices and vacuum database
    Optimize,
    /// Verify index integrity
    Verify,
}

impl IndexCommand {
    /// Execute the index command
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

        // Execute operation
        match self.operation {
            IndexOperation::Build => self.build_index(&db_path)?,
            IndexOperation::Update => self.update_index(&db_path)?,
            IndexOperation::Stats => self.show_stats(&db_path)?,
            IndexOperation::ClearCache => self.clear_cache(&db_path)?,
            IndexOperation::Optimize => self.optimize_database(&db_path)?,
            IndexOperation::Verify => self.verify_index(&db_path)?,
        }

        Ok(())
    }

    /// Build the embedding index
    fn build_index(&self, db_path: &PathBuf) -> Result<()> {
        println!("🔨 Building embedding index...");

        let runtime =
            tokio::runtime::Runtime::new().map_err(|e| HailMaryError::General(e.into()))?;
        runtime.block_on(async {
            let mut repository =
                SqliteMemoryRepository::new(db_path).map_err(HailMaryError::General)?;
            let embedding_service =
                EmbeddingService::new().map_err(HailMaryError::General)?;

            // Load memories
            let memories = if let Some(ref memory_type) = self.r#type {
                repository
                    .browse_by_type(memory_type, 100000)
                    .map_err(HailMaryError::General)?
            } else {
                repository
                    .browse_all(100000)
                    .map_err(HailMaryError::General)?
            };

            if memories.is_empty() {
                println!("No memories found to index.");
                return Ok(());
            }

            println!("📊 Processing {} memories...", memories.len());

            // Process in batches
            let mut indexed_count = 0;
            let mut cache_hits = 0;
            let mut cache_misses = 0;

            for batch in memories.chunks(self.batch_size) {
                let mut batch_embeddings = Vec::new();

                for memory in batch {
                    // Check if embedding already exists
                    if !self.force
                        && let Ok(Some(_)) = repository.get_embedding(&memory.id) {
                            cache_hits += 1;
                            continue;
                        }

                    cache_misses += 1;

                    // Generate embedding
                    let text = format!("{} {}", memory.topic, memory.content);
                    let embeddings = embedding_service
                        .embed_texts(vec![text.clone()])
                        .await
                        .map_err(HailMaryError::General)?;

                    if let Some(embedding) = embeddings.into_iter().next() {
                        batch_embeddings.push((memory.id.clone(), embedding, text));
                    }
                }

                // Store embeddings and update index
                self.store_batch_index(&mut repository, &batch_embeddings)
                    .map_err(|e| HailMaryError::General(e.into()))?;
                indexed_count += batch_embeddings.len();

                if self.verbose {
                    println!("  Indexed {}/{} memories", indexed_count, memories.len());
                }
            }

            // Update statistics
            self.update_vocabulary(&mut repository, &memories)
                .map_err(|e| HailMaryError::General(e.into()))?;

            println!("✅ Index built successfully!");
            println!("  Memories indexed: {}", indexed_count);
            println!("  Cache hits: {}", cache_hits);
            println!("  Cache misses: {}", cache_misses);

            if self.quantize {
                println!("\n🔢 Applying quantization...");
                self.quantize_embeddings(&mut repository)
                    .map_err(|e| HailMaryError::General(e.into()))?;
                println!("✅ Quantization complete!");
            }

            Ok(())
        })
    }

    /// Update the index for new or modified memories
    fn update_index(&self, db_path: &PathBuf) -> Result<()> {
        println!("🔄 Updating embedding index...");

        let runtime =
            tokio::runtime::Runtime::new().map_err(|e| HailMaryError::General(e.into()))?;
        runtime.block_on(async {
            let mut repository =
                SqliteMemoryRepository::new(db_path).map_err(HailMaryError::General)?;
            let conn = Connection::open(db_path).map_err(|e| HailMaryError::General(e.into()))?;

            // Find memories without index entries
            let mut stmt = conn
                .prepare(
                    "SELECT m.* FROM memories m 
                 LEFT JOIN embedding_index ei ON m.id = ei.memory_id 
                 WHERE ei.memory_id IS NULL AND m.deleted = 0",
                )
                .map_err(|e| HailMaryError::General(e.into()))?;

            let memory_iter = stmt
                .query_map([], |row| {
                    Ok(Memory {
                        id: row.get(0)?,
                        memory_type: MemoryType::from_str(&row.get::<_, String>(1)?).unwrap(),
                        topic: row.get(2)?,
                        tags: row
                            .get::<_, String>(3)?
                            .split(',')
                            .filter(|s| !s.is_empty())
                            .map(String::from)
                            .collect(),
                        content: row.get(4)?,
                        examples: serde_json::from_str(&row.get::<_, String>(5)?)
                            .unwrap_or_default(),
                        reference_count: row.get(6)?,
                        confidence: row.get(7)?,
                        created_at: row.get(8)?,
                        last_accessed: row.get(9)?,
                        source: row.get(10).ok(),
                        deleted: row.get(11)?,
                    })
                })
                .map_err(|e| HailMaryError::General(e.into()))?;

            let mut memories_to_index = Vec::new();
            for memory in memory_iter {
                memories_to_index.push(memory.map_err(|e| HailMaryError::General(e.into()))?);
            }

            if memories_to_index.is_empty() {
                println!("✅ Index is up to date!");
                return Ok(());
            }

            println!("📊 Found {} memories to index", memories_to_index.len());

            // Generate and store embeddings
            let embedding_service =
                EmbeddingService::new().map_err(HailMaryError::General)?;
            let mut indexed_count = 0;

            for batch in memories_to_index.chunks(self.batch_size) {
                let mut batch_embeddings = Vec::new();

                for memory in batch {
                    let text = format!("{} {}", memory.topic, memory.content);
                    let embeddings = embedding_service
                        .embed_texts(vec![text.clone()])
                        .await
                        .map_err(HailMaryError::General)?;

                    if let Some(embedding) = embeddings.into_iter().next() {
                        // Store in repository
                        repository
                            .store_embedding(&memory.id, &embedding, embedding_service.model_name())
                            .map_err(HailMaryError::General)?;
                        batch_embeddings.push((memory.id.clone(), embedding, text));
                    }
                }

                // Update index
                self.store_batch_index(&mut repository, &batch_embeddings)
                    .map_err(|e| HailMaryError::General(e.into()))?;
                indexed_count += batch_embeddings.len();

                if self.verbose {
                    println!(
                        "  Indexed {}/{} memories",
                        indexed_count,
                        memories_to_index.len()
                    );
                }
            }

            println!("✅ Index updated successfully!");
            println!("  New memories indexed: {}", indexed_count);

            Ok(())
        })
    }

    /// Show index statistics
    fn show_stats(&self, db_path: &PathBuf) -> Result<()> {
        let conn = Connection::open(db_path).map_err(|e| HailMaryError::General(e.into()))?;

        println!("📊 Embedding Index Statistics");
        println!();

        // Total memories
        let total_memories: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM memories WHERE deleted = 0",
                [],
                |row| row.get(0),
            )
            .map_err(|e| HailMaryError::General(e.into()))?;

        // Indexed memories
        let indexed_memories: i64 = conn
            .query_row("SELECT COUNT(*) FROM embedding_index", [], |row| row.get(0))
            .map_err(|e| HailMaryError::General(e.into()))?;

        // Embeddings count
        let embeddings_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM memory_embeddings", [], |row| {
                row.get(0)
            })
            .map_err(|e| HailMaryError::General(e.into()))?;

        // Cache statistics
        let cache_entries: i64 = conn
            .query_row("SELECT COUNT(*) FROM embedding_cache", [], |row| row.get(0))
            .unwrap_or(0);

        let total_cache_hits: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(access_count), 0) FROM embedding_cache",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Quantized embeddings
        let quantized_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM embeddings_quantized", [], |row| {
                row.get(0)
            })
            .unwrap_or(0);

        // Vocabulary size
        let vocab_size: i64 = conn
            .query_row("SELECT COUNT(*) FROM vocabulary", [], |row| row.get(0))
            .unwrap_or(0);

        // Queue statistics
        let pending_queue: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM embedding_queue WHERE status = 'pending'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let failed_queue: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM embedding_queue WHERE status = 'failed'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Display statistics
        println!("📈 Memory Statistics:");
        println!("  Total memories: {}", total_memories);
        println!(
            "  Indexed memories: {} ({:.1}%)",
            indexed_memories,
            (indexed_memories as f64 / total_memories as f64) * 100.0
        );
        println!("  Embeddings stored: {}", embeddings_count);
        println!();

        println!("💾 Cache Statistics:");
        println!("  Cache entries: {}", cache_entries);
        println!("  Total cache hits: {}", total_cache_hits);
        if cache_entries > 0 {
            println!(
                "  Average hits per entry: {:.1}",
                total_cache_hits as f64 / cache_entries as f64
            );
        }
        println!();

        println!("🔢 Optimization Statistics:");
        println!("  Quantized embeddings: {}", quantized_count);
        println!("  Vocabulary size: {} words", vocab_size);
        println!();

        if pending_queue > 0 || failed_queue > 0 {
            println!("📋 Queue Statistics:");
            println!("  Pending: {}", pending_queue);
            println!("  Failed: {}", failed_queue);
            println!();
        }

        // Database size
        let page_count: i64 = conn
            .query_row("PRAGMA page_count", [], |row| row.get(0))
            .map_err(|e| HailMaryError::General(e.into()))?;
        let page_size: i64 = conn
            .query_row("PRAGMA page_size", [], |row| row.get(0))
            .map_err(|e| HailMaryError::General(e.into()))?;
        let db_size_mb = (page_count * page_size) as f64 / (1024.0 * 1024.0);

        println!("💿 Database Size: {:.2} MB", db_size_mb);

        Ok(())
    }

    /// Clear the embedding cache
    fn clear_cache(&self, db_path: &PathBuf) -> Result<()> {
        let conn = Connection::open(db_path).map_err(|e| HailMaryError::General(e.into()))?;

        if let Some(days) = self.clear_cache_days {
            let cutoff_time = chrono::Utc::now().timestamp() - (days * 24 * 60 * 60);

            let deleted_count = conn
                .execute(
                    "DELETE FROM embedding_cache WHERE accessed_at < ?1",
                    params![cutoff_time],
                )
                .map_err(|e| HailMaryError::General(e.into()))?;

            println!(
                "✅ Cleared {} cache entries older than {} days",
                deleted_count, days
            );
        } else {
            // Clear all cache
            let deleted_count = conn
                .execute("DELETE FROM embedding_cache", [])
                .map_err(|e| HailMaryError::General(e.into()))?;
            println!("✅ Cleared all {} cache entries", deleted_count);
        }

        Ok(())
    }

    /// Optimize database
    fn optimize_database(&self, db_path: &PathBuf) -> Result<()> {
        println!("🔧 Optimizing database...");

        let conn = Connection::open(db_path).map_err(|e| HailMaryError::General(e.into()))?;

        // Analyze tables for query optimization
        conn.execute("ANALYZE", [])
            .map_err(|e| HailMaryError::General(e.into()))?;

        // Vacuum to reclaim space
        conn.execute("VACUUM", [])
            .map_err(|e| HailMaryError::General(e.into()))?;

        // Reindex for better performance
        conn.execute("REINDEX", [])
            .map_err(|e| HailMaryError::General(e.into()))?;

        println!("✅ Database optimized successfully!");

        Ok(())
    }

    /// Verify index integrity
    fn verify_index(&self, db_path: &PathBuf) -> Result<()> {
        println!("🔍 Verifying index integrity...");

        let conn = Connection::open(db_path).map_err(|e| HailMaryError::General(e.into()))?;
        let mut issues = Vec::new();

        // Check for orphaned index entries
        let orphaned_index: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM embedding_index ei 
             LEFT JOIN memories m ON ei.memory_id = m.id 
             WHERE m.id IS NULL",
                [],
                |row| row.get(0),
            )
            .map_err(|e| HailMaryError::General(e.into()))?;

        if orphaned_index > 0 {
            issues.push(format!("{} orphaned index entries", orphaned_index));
        }

        // Check for missing embeddings
        let missing_embeddings: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM embedding_index ei 
             LEFT JOIN memory_embeddings me ON ei.memory_id = me.memory_id 
             WHERE me.memory_id IS NULL",
                [],
                |row| row.get(0),
            )
            .map_err(|e| HailMaryError::General(e.into()))?;

        if missing_embeddings > 0 {
            issues.push(format!(
                "{} index entries without embeddings",
                missing_embeddings
            ));
        }

        // Check for duplicate entries
        let duplicates: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM (
                SELECT memory_id, COUNT(*) as cnt 
                FROM embedding_index 
                GROUP BY memory_id 
                HAVING cnt > 1
            )",
                [],
                |row| row.get(0),
            )
            .map_err(|e| HailMaryError::General(e.into()))?;

        if duplicates > 0 {
            issues.push(format!("{} duplicate index entries", duplicates));
        }

        // Display results
        if issues.is_empty() {
            println!("✅ Index integrity verified - no issues found!");
        } else {
            println!("⚠️  Index integrity issues found:");
            for issue in issues {
                println!("  - {}", issue);
            }
            println!(
                "\nRun 'hail-mary memory index --operation build --force' to rebuild the index."
            );
        }

        Ok(())
    }

    /// Store batch of embeddings in index
    fn store_batch_index(
        &self,
        repository: &mut SqliteMemoryRepository,
        batch: &[(String, Vec<f32>, String)],
    ) -> Result<()> {
        let mut conn =
            Connection::open(&repository.db_path).map_err(|e| HailMaryError::General(e.into()))?;
        let tx = conn
            .transaction()
            .map_err(|e| HailMaryError::General(e.into()))?;

        for (memory_id, embedding, text) in batch {
            // Calculate embedding hash for cache validation
            let embedding_hash = self.calculate_hash(embedding);

            // Calculate magnitude
            let magnitude = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();

            // Store in index
            tx.execute(
                "INSERT OR REPLACE INTO embedding_index 
                 (memory_id, embedding_hash, magnitude, created_at, updated_at) 
                 VALUES (?1, ?2, ?3, ?4, ?4)",
                params![
                    memory_id,
                    embedding_hash,
                    magnitude,
                    chrono::Utc::now().timestamp()
                ],
            )
            .map_err(|e| HailMaryError::General(e.into()))?;

            // Store in cache
            let cache_key = self.calculate_hash_str(text);
            tx.execute(
                "INSERT OR REPLACE INTO embedding_cache 
                 (cache_key, embedding, model_name, dimension, created_at, accessed_at, access_count)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?5, 
                        COALESCE((SELECT access_count + 1 FROM embedding_cache WHERE cache_key = ?1), 1))",
                params![
                    cache_key,
                    EmbeddingService::embedding_to_bytes(embedding),
                    "tfidf",
                    embedding.len(),
                    chrono::Utc::now().timestamp(),
                ],
            ).map_err(|e| HailMaryError::General(e.into()))?;
        }

        tx.commit().map_err(|e| HailMaryError::General(e.into()))?;
        Ok(())
    }

    /// Update vocabulary for TF-IDF
    fn update_vocabulary(
        &self,
        repository: &mut SqliteMemoryRepository,
        memories: &[Memory],
    ) -> Result<()> {
        let mut conn =
            Connection::open(&repository.db_path).map_err(|e| HailMaryError::General(e.into()))?;
        let total_docs = memories.len() as f32;

        // Count word frequencies
        let mut word_doc_counts = HashMap::new();
        let mut word_total_counts = HashMap::new();

        for memory in memories {
            let text = format!("{} {}", memory.topic, memory.content);
            let mut seen_words = std::collections::HashSet::new();

            for word in text.split_whitespace() {
                let word = word
                    .to_lowercase()
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_string();

                if word.len() > 2 {
                    *word_total_counts.entry(word.clone()).or_insert(0) += 1;

                    if seen_words.insert(word.clone()) {
                        *word_doc_counts.entry(word).or_insert(0) += 1;
                    }
                }
            }
        }

        // Update vocabulary table
        let tx = conn
            .transaction()
            .map_err(|e| HailMaryError::General(e.into()))?;

        for (word, doc_count) in word_doc_counts {
            let total_count = word_total_counts.get(&word).unwrap_or(&0);
            let idf_score = (total_docs / doc_count as f32).ln();

            tx.execute(
                "INSERT OR REPLACE INTO vocabulary 
                 (word, idf_score, document_count, total_count, updated_at) 
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    word,
                    idf_score,
                    doc_count,
                    total_count,
                    chrono::Utc::now().timestamp()
                ],
            )
            .map_err(|e| HailMaryError::General(e.into()))?;
        }

        tx.commit().map_err(|e| HailMaryError::General(e.into()))?;
        Ok(())
    }

    /// Quantize embeddings for reduced memory usage
    fn quantize_embeddings(&self, repository: &mut SqliteMemoryRepository) -> Result<()> {
        let mut conn =
            Connection::open(&repository.db_path).map_err(|e| HailMaryError::General(e.into()))?;

        // Get all embeddings first
        let embeddings_to_quantize: Vec<(String, Vec<u8>)> = {
            let mut stmt = conn
                .prepare("SELECT memory_id, embedding FROM memory_embeddings")
                .map_err(|e| HailMaryError::General(e.into()))?;
            let embedding_iter = stmt
                .query_map([], |row| {
                    let memory_id: String = row.get(0)?;
                    let embedding_bytes: Vec<u8> = row.get(1)?;
                    Ok((memory_id, embedding_bytes))
                })
                .map_err(|e| HailMaryError::General(e.into()))?;

            let mut result = Vec::new();
            for item in embedding_iter {
                result.push(item.map_err(|e| HailMaryError::General(e.into()))?);
            }
            result
        };

        let tx = conn
            .transaction()
            .map_err(|e| HailMaryError::General(e.into()))?;
        let mut quantized_count = 0;

        for (memory_id, embedding_bytes) in embeddings_to_quantize {
            let embedding = EmbeddingService::bytes_to_embedding(&embedding_bytes)
                .map_err(HailMaryError::General)?;

            // Quantize to 8-bit
            let (quantized, scale, offset) = self.quantize_vector(&embedding);
            let original_norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();

            tx.execute(
                "INSERT OR REPLACE INTO embeddings_quantized 
                 (memory_id, embedding_q8, scale, offset, original_norm, created_at) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    memory_id,
                    quantized,
                    scale,
                    offset,
                    original_norm,
                    chrono::Utc::now().timestamp(),
                ],
            )
            .map_err(|e| HailMaryError::General(e.into()))?;

            quantized_count += 1;
        }

        tx.commit().map_err(|e| HailMaryError::General(e.into()))?;

        if self.verbose {
            println!("  Quantized {} embeddings", quantized_count);
        }

        Ok(())
    }

    /// Quantize a vector to 8-bit
    fn quantize_vector(&self, vector: &[f32]) -> (Vec<u8>, f32, f32) {
        let min = vector.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = vector.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

        let range = max - min;
        let scale = range / 255.0;
        let offset = min;

        let quantized: Vec<u8> = vector
            .iter()
            .map(|&v| ((v - offset) / scale).round() as u8)
            .collect();

        (quantized, scale, offset)
    }

    /// Calculate hash of embedding
    fn calculate_hash(&self, embedding: &[f32]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        for val in embedding {
            val.to_bits().hash(&mut hasher);
        }
        format!("{:x}", hasher.finish())
    }

    /// Calculate hash of string
    fn calculate_hash_str(&self, text: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantization() {
        let cmd = IndexCommand {
            db_path: None,
            operation: IndexOperation::Build,
            r#type: None,
            batch_size: 100,
            quantize: false,
            clear_cache_days: None,
            verbose: false,
            force: false,
        };

        let vector = vec![0.0, 0.5, 1.0, -0.5, -1.0];
        let (quantized, scale, offset) = cmd.quantize_vector(&vector);

        // Verify quantization
        assert_eq!(quantized.len(), vector.len());

        // Reconstruct and verify
        let reconstructed: Vec<f32> = quantized
            .iter()
            .map(|&q| q as f32 * scale + offset)
            .collect();

        for (original, reconstructed) in vector.iter().zip(reconstructed.iter()) {
            assert!((original - reconstructed).abs() < 0.01);
        }
    }
}
