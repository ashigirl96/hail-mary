use anyhow::Result;
use chrono::Utc;
use rusqlite::{Connection, params};
use std::fs;
use std::path::{Path, PathBuf};

use crate::memory::{
    embeddings::EmbeddingService,
    models::{Memory, MemoryType},
    repository::{MemoryRepository, SqliteMemoryRepository},
};

/// Configuration for reindex operation
pub struct ReindexConfig {
    /// Similarity threshold for considering memories as duplicates (0.0 to 1.0)
    pub similarity_threshold: f32,
    /// Whether to backup the database before reindexing
    pub backup_enabled: bool,
    /// Path to store backups
    pub backup_dir: PathBuf,
    /// Whether to show progress
    pub verbose: bool,
    /// Whether to generate embeddings during reindex
    #[allow(dead_code)] // Reserved for future embedding generation during reindex
    pub generate_embeddings: bool,
    /// Batch size for embedding generation
    pub embedding_batch_size: usize,
    /// Force regenerate embeddings even if they exist
    pub force_regenerate_embeddings: bool,
}

impl Default for ReindexConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.85,
            backup_enabled: true,
            backup_dir: dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("hail-mary")
                .join("backups"),
            verbose: false,
            generate_embeddings: false,
            embedding_batch_size: 32,
            force_regenerate_embeddings: false,
        }
    }
}

/// Service for reindexing and deduplicating memories
pub struct ReindexService {
    embedding_service: EmbeddingService,
    config: ReindexConfig,
}

impl ReindexService {
    /// Create a new reindex service
    pub fn new(config: ReindexConfig) -> Result<Self> {
        let embedding_service = EmbeddingService::new()?;
        Ok(Self {
            embedding_service,
            config,
        })
    }

    /// Perform complete reindex of the database
    pub async fn reindex(&self, db_path: &Path) -> Result<ReindexResult> {
        let start_time = Utc::now();

        // Step 1: Backup current database
        let backup_path = if self.config.backup_enabled {
            Some(self.backup_database(db_path)?)
        } else {
            None
        };

        if self.config.verbose {
            println!("🔄 Starting reindex process...");
            if let Some(ref path) = backup_path {
                println!("📦 Database backed up to: {}", path.display());
            }
        }

        // Step 2: Load all memories
        let repository = SqliteMemoryRepository::new(db_path)?;
        let all_memories = self.load_all_memories(&repository)?;
        let total_memories = all_memories.len();

        if self.config.verbose {
            println!("📊 Loaded {} memories for processing", total_memories);
        }

        // Step 3: Generate embeddings for all memories
        let embeddings = self.generate_embeddings(&all_memories).await?;

        if self.config.verbose {
            println!("🧮 Generated embeddings for all memories");
        }

        // Step 4: Find duplicates
        let duplicates = self.find_duplicates(&all_memories, &embeddings)?;
        let duplicates_found = duplicates.len();

        if self.config.verbose {
            println!("🔍 Found {} potential duplicate pairs", duplicates_found);
        }

        // Step 5: Merge duplicates
        let (merged_memories, duplicates_merged) =
            self.merge_duplicates(all_memories, duplicates)?;

        if self.config.verbose {
            println!("🔀 Merged {} duplicate memories", duplicates_merged);
        }

        // Step 6: Create new optimized database
        let temp_db_path = db_path.with_extension("reindex_temp");
        self.create_optimized_database(&temp_db_path, &merged_memories, &embeddings)?;

        // Step 7: Replace old database with new one
        let old_db_path = db_path.with_extension("old");
        fs::rename(db_path, &old_db_path)?;
        fs::rename(&temp_db_path, db_path)?;
        fs::remove_file(old_db_path)?;

        let end_time = Utc::now();
        let duration = end_time - start_time;

        if self.config.verbose {
            println!("✅ Reindex completed in {} seconds", duration.num_seconds());
        }

        Ok(ReindexResult {
            total_memories,
            duplicates_found,
            duplicates_merged,
            backup_path,
            duration_seconds: duration.num_seconds(),
        })
    }

    /// Backup the database
    fn backup_database(&self, db_path: &Path) -> Result<PathBuf> {
        fs::create_dir_all(&self.config.backup_dir)?;

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_filename = format!("memory_backup_{}.db", timestamp);
        let backup_path = self.config.backup_dir.join(backup_filename);

        fs::copy(db_path, &backup_path)?;
        Ok(backup_path)
    }

    /// Load all memories from the database
    fn load_all_memories(&self, repository: &SqliteMemoryRepository) -> Result<Vec<Memory>> {
        let mut all_memories = Vec::new();

        for memory_type in &[
            MemoryType::Tech,
            MemoryType::ProjectTech,
            MemoryType::Domain,
        ] {
            let memories = repository.browse_by_type(memory_type, usize::MAX)?;
            all_memories.extend(memories);
        }

        Ok(all_memories)
    }

    /// Generate embeddings for all memories
    async fn generate_embeddings(&self, memories: &[Memory]) -> Result<Vec<Vec<f32>>> {
        let texts: Vec<String> = memories
            .iter()
            .map(|m| format!("{} {}", m.topic, m.content))
            .collect();

        self.embedding_service.embed_texts(texts).await
    }

    /// Find duplicate memories based on embedding similarity
    fn find_duplicates(
        &self,
        memories: &[Memory],
        embeddings: &[Vec<f32>],
    ) -> Result<Vec<DuplicatePair>> {
        let mut duplicates = Vec::new();

        for i in 0..memories.len() {
            for j in (i + 1)..memories.len() {
                // Only check duplicates within the same memory type
                if memories[i].memory_type != memories[j].memory_type {
                    continue;
                }

                let similarity =
                    EmbeddingService::cosine_similarity(&embeddings[i], &embeddings[j]);

                if similarity >= self.config.similarity_threshold {
                    duplicates.push(DuplicatePair {
                        index1: i,
                        index2: j,
                        similarity,
                    });
                }
            }
        }

        Ok(duplicates)
    }

    /// Merge duplicate memories
    fn merge_duplicates(
        &self,
        memories: Vec<Memory>,
        duplicates: Vec<DuplicatePair>,
    ) -> Result<(Vec<Memory>, usize)> {
        // Sort duplicates by similarity (highest first)
        let mut duplicates = duplicates;
        duplicates.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());

        // Track which memories have been merged
        let mut merged_indices = std::collections::HashSet::new();
        let mut merge_count = 0;

        // Create a new vector with merged memories
        let mut merged_memories = memories.clone();

        for dup in duplicates {
            // Skip if either memory has already been merged
            if merged_indices.contains(&dup.index1) || merged_indices.contains(&dup.index2) {
                continue;
            }

            // Get the two memories to merge
            let memory2 = merged_memories[dup.index2].clone();
            let memory1 = &mut merged_memories[dup.index1];

            // Combine content
            if !memory1.content.contains(&memory2.content) {
                memory1.content = format!("{}\n\n{}", memory1.content, memory2.content);
            }

            // Merge tags (unique)
            for tag in &memory2.tags {
                if !memory1.tags.contains(tag) {
                    memory1.tags.push(tag.clone());
                }
            }

            // Merge examples (unique)
            for example in &memory2.examples {
                if !memory1.examples.contains(example) {
                    memory1.examples.push(example.clone());
                }
            }

            // Update reference count and confidence
            memory1.reference_count += memory2.reference_count;
            memory1.confidence = (memory1.confidence + memory2.confidence) / 2.0;

            // Keep the earlier creation date
            if memory2.created_at < memory1.created_at {
                memory1.created_at = memory2.created_at;
            }

            // Mark memory2 as merged
            merged_indices.insert(dup.index2);
            merge_count += 1;
        }

        // Filter out merged memories
        let final_memories: Vec<Memory> = merged_memories
            .into_iter()
            .enumerate()
            .filter(|(i, _)| !merged_indices.contains(i))
            .map(|(_, m)| m)
            .collect();

        Ok((final_memories, merge_count))
    }

    /// Create optimized database with deduplicated memories
    fn create_optimized_database(
        &self,
        db_path: &Path,
        memories: &[Memory],
        embeddings: &[Vec<f32>],
    ) -> Result<()> {
        // Create new database
        let mut conn = Connection::open(db_path)?;

        // Initialize schema
        crate::memory::migration::initialize_database(&mut conn)?;

        // Insert all memories
        let tx = conn.transaction()?;
        {
            for (i, memory) in memories.iter().enumerate() {
                // Insert memory
                tx.execute(
                    "INSERT INTO memories (id, type, topic, tags, content, examples, 
                     reference_count, confidence, created_at, last_accessed, source, deleted)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                    params![
                        &memory.id,
                        &memory.memory_type.to_string(),
                        &memory.topic,
                        &memory.tags.join(","),
                        &memory.content,
                        &serde_json::to_string(&memory.examples)?,
                        memory.reference_count,
                        memory.confidence,
                        memory.created_at,
                        memory.last_accessed,
                        &memory.source,
                        0, // Always set deleted to false in optimized database
                    ],
                )?;

                // Insert embedding
                if i < embeddings.len() {
                    let embedding_bytes = EmbeddingService::embedding_to_bytes(&embeddings[i]);
                    tx.execute(
                        "INSERT INTO memory_embeddings (memory_id, embedding, embedding_model)
                         VALUES (?1, ?2, ?3)",
                        params![&memory.id, &embedding_bytes, "BAAI/bge-small-en-v1.5",],
                    )?;
                }
            }
        }
        tx.commit()?;

        Ok(())
    }

    /// Run reindex with embedding generation
    pub async fn reindex_with_embeddings(&self, db_path: &Path) -> Result<ReindexResult> {
        let start_time = Utc::now();

        // First run normal reindex
        let mut result = self.reindex(db_path).await?;

        // Then generate embeddings for memories without them
        let mut repo = SqliteMemoryRepository::new(db_path)?;

        // Get memories without embeddings
        let memories_without_embeddings = if self.config.force_regenerate_embeddings {
            // Get all memories if force regenerate
            repo.browse_all(1_000_000)?
        } else {
            // Get only memories without embeddings
            repo.get_memories_without_embeddings(1_000_000)?
        };

        if self.config.verbose {
            println!(
                "📊 Found {} memories needing embeddings",
                memories_without_embeddings.len()
            );
        }

        // Generate embeddings in batches
        let total = memories_without_embeddings.len();
        let mut processed = 0;

        for batch in memories_without_embeddings.chunks(self.config.embedding_batch_size) {
            // Generate embeddings for batch
            let texts: Vec<String> = batch
                .iter()
                .map(|m| format!("{} {}", m.topic, m.content))
                .collect();

            // Use Handle::current() to get the current runtime if in async context
            let embeddings = if tokio::runtime::Handle::try_current().is_ok() {
                // We're already in an async context
                self.embedding_service.embed_texts(texts).await?
            } else {
                // We need to create a runtime
                let rt = tokio::runtime::Runtime::new()?;
                rt.block_on(self.embedding_service.embed_texts(texts))?
            };

            // Store embeddings
            for (memory, embedding) in batch.iter().zip(embeddings.iter()) {
                repo.store_embedding(&memory.id, embedding, self.embedding_service.model_name())?;
            }

            processed += batch.len();

            if self.config.verbose {
                println!(
                    "  Generated embeddings: {}/{} ({:.1}%)",
                    processed,
                    total,
                    (processed as f32 / total as f32) * 100.0
                );
            }
        }

        // Update result with embedding statistics
        result.duration_seconds = (Utc::now() - start_time).num_seconds();

        if self.config.verbose {
            println!("✅ Generated embeddings for {} memories", processed);
        }

        Ok(result)
    }
}

/// Result of a reindex operation
#[derive(Debug)]
pub struct ReindexResult {
    pub total_memories: usize,
    pub duplicates_found: usize,
    pub duplicates_merged: usize,
    pub backup_path: Option<PathBuf>,
    pub duration_seconds: i64,
}

/// A pair of duplicate memories
#[derive(Debug)]
struct DuplicatePair {
    index1: usize,
    index2: usize,
    similarity: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_reindex_empty_database() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create empty database
        let _repo = SqliteMemoryRepository::new(&db_path).unwrap();

        let config = ReindexConfig {
            backup_enabled: false,
            ..Default::default()
        };

        let service = ReindexService::new(config).unwrap();
        let result = service.reindex(&db_path).await.unwrap();

        assert_eq!(result.total_memories, 0);
        assert_eq!(result.duplicates_found, 0);
        assert_eq!(result.duplicates_merged, 0);
    }
}
