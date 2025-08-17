use crate::memory::migration::initialize_database;
use crate::memory::models::{Memory, MemoryType};
use anyhow::Result;
use rusqlite::{Connection, params};
use std::path::{Path, PathBuf};

// SQLクエリを定数化（型安全性の補完）
const INSERT_MEMORY: &str = r#"
    INSERT INTO memories (id, type, topic, tags, content, examples, 
                         reference_count, confidence, created_at, 
                         last_accessed, source, deleted)
    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
"#;

const UPDATE_MEMORY: &str = r#"
    UPDATE memories 
    SET type = ?2, topic = ?3, tags = ?4, content = ?5, examples = ?6,
        confidence = ?7, last_accessed = ?8, source = ?9
    WHERE id = ?1 AND deleted = 0
"#;

const SELECT_BY_ID: &str = r#"
    SELECT id, type, topic, tags, content, examples, reference_count,
           confidence, created_at, last_accessed, source, deleted
    FROM memories
    WHERE id = ?1 AND deleted = 0
"#;

const SELECT_BY_TOPIC: &str = r#"
    SELECT id, type, topic, tags, content, examples, reference_count,
           confidence, created_at, last_accessed, source, deleted
    FROM memories
    WHERE topic = ?1 AND type = ?2 AND deleted = 0
"#;

const SEARCH_MEMORIES_FTS: &str = r#"
    SELECT m.id, m.type, m.topic, m.tags, m.content, m.examples, 
           m.reference_count, m.confidence, m.created_at, 
           m.last_accessed, m.source, m.deleted
    FROM memories m
    JOIN memories_fts f ON m.id = f.memory_id
    WHERE f.memories_fts MATCH ?1
    AND m.deleted = 0
    ORDER BY rank
    LIMIT ?2
"#;

const SEARCH_MEMORIES_FTS_WITH_TYPE: &str = r#"
    SELECT m.id, m.type, m.topic, m.tags, m.content, m.examples, 
           m.reference_count, m.confidence, m.created_at, 
           m.last_accessed, m.source, m.deleted
    FROM memories m
    JOIN memories_fts f ON m.id = f.memory_id
    WHERE f.memories_fts MATCH ?1
    AND m.type = ?2
    AND m.deleted = 0
    ORDER BY rank
    LIMIT ?3
"#;

const BROWSE_BY_TYPE: &str = r#"
    SELECT id, type, topic, tags, content, examples, reference_count,
           confidence, created_at, last_accessed, source, deleted
    FROM memories
    WHERE type = ?1 AND deleted = 0
    ORDER BY confidence DESC, reference_count DESC
"#;

const BROWSE_BY_TYPE_LIMIT: &str = r#"
    SELECT id, type, topic, tags, content, examples, reference_count,
           confidence, created_at, last_accessed, source, deleted
    FROM memories
    WHERE type = ?1 AND deleted = 0
    ORDER BY confidence DESC, reference_count DESC
    LIMIT ?2
"#;

const UPDATE_REFERENCE_COUNT: &str = r#"
    UPDATE memories 
    SET reference_count = reference_count + 1,
        last_accessed = unixepoch()
    WHERE id = ?1 AND deleted = 0
"#;

const UPDATE_LAST_ACCESSED: &str = r#"
    UPDATE memories 
    SET last_accessed = unixepoch()
    WHERE id = ?1 AND deleted = 0
"#;

const SOFT_DELETE: &str = r#"
    UPDATE memories 
    SET deleted = 1
    WHERE id = ?1
"#;

const SEARCH_MEMORIES_FTS_ALL: &str = r#"
    SELECT m.id, m.type, m.topic, m.tags, m.content, m.examples, 
           m.reference_count, m.confidence, m.created_at, 
           m.last_accessed, m.source, m.deleted
    FROM memories m
    JOIN memories_fts f ON m.id = f.memory_id
    WHERE f.memories_fts MATCH ?1
    ORDER BY rank
    LIMIT ?2
"#;

const BROWSE_ALL: &str = r#"
    SELECT id, type, topic, tags, content, examples, reference_count,
           confidence, created_at, last_accessed, source, deleted
    FROM memories
    ORDER BY confidence DESC, reference_count DESC
    LIMIT ?1
"#;

/// メモリリポジトリのトレイト
pub trait MemoryRepository {
    fn save(&mut self, memory: &Memory) -> Result<()>;
    fn update(&mut self, memory: &Memory) -> Result<()>;
    fn find_by_id(&self, id: &str) -> Result<Option<Memory>>;
    fn find_by_topic(&self, topic: &str, memory_type: &MemoryType) -> Result<Option<Memory>>;
    fn search(&self, query: &str, limit: usize) -> Result<Vec<Memory>>;
    fn search_with_type(
        &self,
        query: &str,
        memory_type: &MemoryType,
        limit: usize,
    ) -> Result<Vec<Memory>>;
    fn search_all(&self, query: &str, limit: usize) -> Result<Vec<Memory>>;
    fn browse_by_type(&self, memory_type: &MemoryType, limit: usize) -> Result<Vec<Memory>>;
    fn browse_all(&self, limit: usize) -> Result<Vec<Memory>>;
    fn update_reference_count(&mut self, id: &str) -> Result<()>;
    fn update_last_accessed(&mut self, id: &str) -> Result<()>;
    fn soft_delete(&mut self, id: &str) -> Result<()>;

    // Embedding-related methods
    fn store_embedding(
        &mut self,
        memory_id: &str,
        embedding: &[f32],
        model_name: &str,
    ) -> Result<()>;
    fn get_embedding(&self, memory_id: &str) -> Result<Option<Vec<f32>>>;
    #[allow(dead_code)] // Reserved for future batch embedding operations
    fn get_embeddings_batch(
        &self,
        memory_ids: &[String],
    ) -> Result<Vec<(String, Option<Vec<f32>>)>>;
    fn search_similar(
        &self,
        embedding: &[f32],
        limit: usize,
        min_similarity: f32,
    ) -> Result<Vec<(Memory, f32)>>;
    #[allow(dead_code)] // Reserved for future duplicate detection functionality
    fn find_duplicates(&self, similarity_threshold: f32) -> Result<Vec<(String, String, f32)>>;
    fn get_memories_without_embeddings(&self, limit: usize) -> Result<Vec<Memory>>;
}

/// SQLite実装のメモリリポジトリ
#[derive(Debug)]
pub struct SqliteMemoryRepository {
    conn: Connection,
    pub db_path: PathBuf,
}

impl SqliteMemoryRepository {
    /// 新しいリポジトリを作成
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let db_path = db_path.as_ref().to_path_buf();
        let mut conn = Connection::open(&db_path)?;
        initialize_database(&mut conn)?;
        Ok(Self { conn, db_path })
    }

    /// インメモリデータベースで作成（テスト用）
    #[cfg(test)]
    pub fn new_in_memory() -> Result<Self> {
        let mut conn = Connection::open_in_memory()?;
        initialize_database(&mut conn)?;
        Ok(Self {
            conn,
            db_path: PathBuf::from(":memory:"),
        })
    }
}

impl MemoryRepository for SqliteMemoryRepository {
    fn save(&mut self, memory: &Memory) -> Result<()> {
        let examples_json = serde_json::to_string(&memory.examples)?;
        let tags_str = memory.tags.join(",");

        self.conn.execute(
            INSERT_MEMORY,
            params![
                &memory.id,
                &memory.memory_type.to_string(),
                &memory.topic,
                &tags_str,
                &memory.content,
                &examples_json,
                memory.reference_count,
                memory.confidence,
                memory.created_at,
                memory.last_accessed,
                &memory.source,
                memory.deleted as i32,
            ],
        )?;
        Ok(())
    }

    fn update(&mut self, memory: &Memory) -> Result<()> {
        let examples_json = serde_json::to_string(&memory.examples)?;
        let tags_str = memory.tags.join(",");

        self.conn.execute(
            UPDATE_MEMORY,
            params![
                &memory.id,
                &memory.memory_type.to_string(),
                &memory.topic,
                &tags_str,
                &memory.content,
                &examples_json,
                memory.confidence,
                memory.last_accessed,
                &memory.source,
            ],
        )?;
        Ok(())
    }

    fn find_by_id(&self, id: &str) -> Result<Option<Memory>> {
        let mut stmt = self.conn.prepare(SELECT_BY_ID)?;
        let mut rows = stmt.query_map(params![id], Memory::from_row)?;

        match rows.next() {
            Some(result) => Ok(Some(result?)),
            None => Ok(None),
        }
    }

    fn find_by_topic(&self, topic: &str, memory_type: &MemoryType) -> Result<Option<Memory>> {
        let mut stmt = self.conn.prepare(SELECT_BY_TOPIC)?;
        let mut rows = stmt.query_map(params![topic, memory_type.to_string()], |row| {
            Memory::from_row(row)
        })?;

        match rows.next() {
            Some(result) => Ok(Some(result?)),
            None => Ok(None),
        }
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<Memory>> {
        let mut stmt = self.conn.prepare(SEARCH_MEMORIES_FTS)?;
        let memory_iter = stmt.query_map(params![query, limit], Memory::from_row)?;

        let mut memories = Vec::new();
        for memory in memory_iter {
            memories.push(memory?);
        }
        Ok(memories)
    }

    fn search_all(&self, query: &str, limit: usize) -> Result<Vec<Memory>> {
        let mut stmt = self.conn.prepare(SEARCH_MEMORIES_FTS_ALL)?;
        let memory_iter = stmt.query_map(params![query, limit], Memory::from_row)?;

        let mut memories = Vec::new();
        for memory in memory_iter {
            memories.push(memory?);
        }
        Ok(memories)
    }

    fn search_with_type(
        &self,
        query: &str,
        memory_type: &MemoryType,
        limit: usize,
    ) -> Result<Vec<Memory>> {
        let mut stmt = self.conn.prepare(SEARCH_MEMORIES_FTS_WITH_TYPE)?;
        let memory_iter = stmt.query_map(
            params![query, memory_type.to_string(), limit],
            Memory::from_row,
        )?;

        let mut memories = Vec::new();
        for memory in memory_iter {
            memories.push(memory?);
        }
        Ok(memories)
    }

    fn browse_by_type(&self, memory_type: &MemoryType, limit: usize) -> Result<Vec<Memory>> {
        let mut memories = Vec::new();

        if limit == usize::MAX || limit > 10000 {
            // No limit or very large limit - use query without LIMIT
            let mut stmt = self.conn.prepare(BROWSE_BY_TYPE)?;
            let memory_iter = stmt.query_map(params![memory_type.to_string()], |row| {
                Memory::from_row(row)
            })?;
            for memory in memory_iter {
                memories.push(memory?);
            }
        } else {
            // Use query with LIMIT
            let mut stmt = self.conn.prepare(BROWSE_BY_TYPE_LIMIT)?;
            let memory_iter = stmt.query_map(params![memory_type.to_string(), limit], |row| {
                Memory::from_row(row)
            })?;
            for memory in memory_iter {
                memories.push(memory?);
            }
        }

        Ok(memories)
    }

    fn update_reference_count(&mut self, id: &str) -> Result<()> {
        self.conn.execute(UPDATE_REFERENCE_COUNT, params![id])?;
        Ok(())
    }

    fn update_last_accessed(&mut self, id: &str) -> Result<()> {
        self.conn.execute(UPDATE_LAST_ACCESSED, params![id])?;
        Ok(())
    }

    fn browse_all(&self, limit: usize) -> Result<Vec<Memory>> {
        let mut stmt = self.conn.prepare(BROWSE_ALL)?;
        let memory_iter = stmt.query_map(params![limit], Memory::from_row)?;

        let mut memories = Vec::new();
        for memory in memory_iter {
            memories.push(memory?);
        }
        Ok(memories)
    }

    fn soft_delete(&mut self, id: &str) -> Result<()> {
        self.conn.execute(SOFT_DELETE, params![id])?;
        Ok(())
    }

    // =========================================================================
    // Embedding Operations
    // =========================================================================

    fn store_embedding(
        &mut self,
        memory_id: &str,
        embedding: &[f32],
        model_name: &str,
    ) -> Result<()> {
        use crate::memory::embeddings::EmbeddingService;

        let embedding_bytes = EmbeddingService::embedding_to_bytes(embedding);

        // First, check if an embedding already exists
        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM memory_embeddings WHERE memory_id = ?",
                params![memory_id],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if exists {
            // Update existing embedding
            self.conn.execute(
                "UPDATE memory_embeddings SET embedding = ?, embedding_model = ?, created_at = unixepoch() WHERE memory_id = ?",
                params![&embedding_bytes, model_name, memory_id]
            )?;
        } else {
            // Insert new embedding
            self.conn.execute(
                "INSERT INTO memory_embeddings (memory_id, embedding, embedding_model) VALUES (?, ?, ?)",
                params![memory_id, &embedding_bytes, model_name]
            )?;
        }

        Ok(())
    }

    fn get_embedding(&self, memory_id: &str) -> Result<Option<Vec<f32>>> {
        use crate::memory::embeddings::EmbeddingService;

        let result = self.conn.query_row(
            "SELECT embedding FROM memory_embeddings WHERE memory_id = ?",
            params![memory_id],
            |row| {
                let bytes: Vec<u8> = row.get(0)?;
                Ok(bytes)
            },
        );

        match result {
            Ok(bytes) => Ok(Some(EmbeddingService::bytes_to_embedding(&bytes)?)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn get_embeddings_batch(
        &self,
        memory_ids: &[String],
    ) -> Result<Vec<(String, Option<Vec<f32>>)>> {
        use crate::memory::embeddings::EmbeddingService;

        let mut results = Vec::new();

        for id in memory_ids {
            let embedding = self.conn.query_row(
                "SELECT embedding FROM memory_embeddings WHERE memory_id = ?",
                params![id],
                |row| {
                    let bytes: Vec<u8> = row.get(0)?;
                    Ok(bytes)
                },
            );

            let embedding_vec = match embedding {
                Ok(bytes) => Some(EmbeddingService::bytes_to_embedding(&bytes)?),
                Err(_) => None,
            };

            results.push((id.clone(), embedding_vec));
        }

        Ok(results)
    }

    fn search_similar(
        &self,
        embedding: &[f32],
        limit: usize,
        min_similarity: f32,
    ) -> Result<Vec<(Memory, f32)>> {
        use crate::memory::embeddings::EmbeddingService;

        // Get all memories with embeddings
        let mut stmt = self.conn.prepare(
            "SELECT m.*, e.embedding 
             FROM memories m 
             JOIN memory_embeddings e ON m.id = e.memory_id 
             WHERE m.deleted = 0",
        )?;

        let rows = stmt.query_map([], |row| {
            let memory = Memory::from_row(row)?;
            let embedding_bytes: Vec<u8> = row.get(12)?; // Assuming embedding is after all memory fields
            Ok((memory, embedding_bytes))
        })?;

        let mut similarities = Vec::new();

        for row in rows {
            let (memory, embedding_bytes) = row?;
            let stored_embedding = EmbeddingService::bytes_to_embedding(&embedding_bytes)?;
            let similarity = EmbeddingService::cosine_similarity(embedding, &stored_embedding);

            if similarity >= min_similarity {
                similarities.push((memory, similarity));
            }
        }

        // Sort by similarity (highest first)
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        similarities.truncate(limit);

        Ok(similarities)
    }

    fn find_duplicates(&self, similarity_threshold: f32) -> Result<Vec<(String, String, f32)>> {
        use crate::memory::embeddings::EmbeddingService;

        // Get all embeddings
        let mut stmt = self
            .conn
            .prepare("SELECT memory_id, embedding FROM memory_embeddings")?;

        let embeddings: Vec<(String, Vec<f32>)> = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let bytes: Vec<u8> = row.get(1)?;
                Ok((id, bytes))
            })?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .filter_map(|(id, bytes)| {
                EmbeddingService::bytes_to_embedding(&bytes)
                    .ok()
                    .map(|embedding| (id, embedding))
            })
            .collect();

        let mut duplicates = Vec::new();

        // Compare all pairs
        for i in 0..embeddings.len() {
            for j in (i + 1)..embeddings.len() {
                let similarity =
                    EmbeddingService::cosine_similarity(&embeddings[i].1, &embeddings[j].1);

                if similarity >= similarity_threshold {
                    duplicates.push((embeddings[i].0.clone(), embeddings[j].0.clone(), similarity));
                }
            }
        }

        // Sort by similarity (highest first)
        duplicates.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

        Ok(duplicates)
    }

    fn get_memories_without_embeddings(&self, limit: usize) -> Result<Vec<Memory>> {
        let mut stmt = self.conn.prepare(
            "SELECT m.* FROM memories m 
             LEFT JOIN memory_embeddings e ON m.id = e.memory_id 
             WHERE e.memory_id IS NULL AND m.deleted = 0 
             LIMIT ?",
        )?;

        let memory_iter = stmt.query_map(params![limit], Memory::from_row)?;

        let mut memories = Vec::new();
        for memory in memory_iter {
            memories.push(memory?);
        }

        Ok(memories)
    }
}

impl SqliteMemoryRepository {
    /// Convenience method for import command - alias for find_by_id
    pub fn get_by_id(&self, id: &str) -> Result<Option<Memory>> {
        self.find_by_id(id)
    }

    /// Convenience method for import command - alias for update  
    pub fn update_memory(&mut self, memory: &Memory) -> Result<()> {
        self.update(memory)
    }

    /// Convenience method for import command - alias for save
    pub fn create_memory(&mut self, memory: &Memory) -> Result<()> {
        self.save(memory)
    }

    // =========================================================================
    // Bulk Operations
    // =========================================================================

    /// Execute a bulk operation with transaction safety
    pub fn bulk_operation<F, R>(&mut self, operation: F) -> Result<R>
    where
        F: FnOnce(&mut rusqlite::Transaction) -> Result<R>,
    {
        let mut tx = self.conn.transaction()?;
        let result = operation(&mut tx)?;
        tx.commit()?;
        Ok(result)
    }

    /// Bulk soft delete multiple memories by IDs
    pub fn bulk_soft_delete(&mut self, memory_ids: &[String]) -> Result<usize> {
        if memory_ids.is_empty() {
            return Ok(0);
        }

        self.bulk_operation(|tx| {
            let placeholders = memory_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let sql = format!(
                "UPDATE memories SET deleted = 1 WHERE id IN ({})",
                placeholders
            );

            let params: Vec<&dyn rusqlite::ToSql> = memory_ids
                .iter()
                .map(|id| id as &dyn rusqlite::ToSql)
                .collect();
            let affected = tx.execute(&sql, &params[..])?;
            Ok(affected)
        })
    }

    /// Bulk hard delete multiple memories by IDs (permanent deletion)
    pub fn bulk_hard_delete(&mut self, memory_ids: &[String]) -> Result<usize> {
        if memory_ids.is_empty() {
            return Ok(0);
        }

        self.bulk_operation(|tx| {
            let placeholders = memory_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let sql = format!("DELETE FROM memories WHERE id IN ({})", placeholders);

            let params: Vec<&dyn rusqlite::ToSql> = memory_ids
                .iter()
                .map(|id| id as &dyn rusqlite::ToSql)
                .collect();
            let affected = tx.execute(&sql, &params[..])?;
            Ok(affected)
        })
    }

    /// Bulk add tags to multiple memories
    pub fn bulk_add_tags(&mut self, memory_ids: &[String], tags: &[String]) -> Result<usize> {
        if memory_ids.is_empty() || tags.is_empty() {
            return Ok(0);
        }

        let new_tags = tags.join(",");

        self.bulk_operation(|tx| {
            let mut affected = 0;
            for memory_id in memory_ids {
                // Get current tags
                let current_tags: String = tx.query_row(
                    "SELECT tags FROM memories WHERE id = ? AND deleted = 0",
                    params![memory_id],
                    |row| row.get(0),
                )?;

                // Merge tags
                let updated_tags = if current_tags.is_empty() {
                    new_tags.clone()
                } else {
                    format!("{},{}", current_tags, new_tags)
                };

                // Update memory
                let rows_affected = tx.execute(
                    "UPDATE memories SET tags = ? WHERE id = ? AND deleted = 0",
                    params![updated_tags, memory_id],
                )?;
                affected += rows_affected;
            }
            Ok(affected)
        })
    }

    /// Bulk remove tags from multiple memories
    pub fn bulk_remove_tags(&mut self, memory_ids: &[String], tags: &[String]) -> Result<usize> {
        if memory_ids.is_empty() || tags.is_empty() {
            return Ok(0);
        }

        self.bulk_operation(|tx| {
            let mut affected = 0;
            for memory_id in memory_ids {
                // Get current tags
                let current_tags: String = tx.query_row(
                    "SELECT tags FROM memories WHERE id = ? AND deleted = 0",
                    params![memory_id],
                    |row| row.get(0),
                )?;

                if !current_tags.is_empty() {
                    // Remove specified tags
                    let mut remaining_tags: Vec<String> = current_tags
                        .split(',')
                        .filter(|tag| !tags.contains(&tag.to_string()))
                        .map(|s| s.to_string())
                        .collect();

                    remaining_tags.dedup();
                    let updated_tags = remaining_tags.join(",");

                    // Update memory
                    let rows_affected = tx.execute(
                        "UPDATE memories SET tags = ? WHERE id = ? AND deleted = 0",
                        params![updated_tags, memory_id],
                    )?;
                    affected += rows_affected;
                }
            }
            Ok(affected)
        })
    }

    /// Bulk update confidence for multiple memories
    pub fn bulk_update_confidence(
        &mut self,
        memory_ids: &[String],
        confidence: f32,
    ) -> Result<usize> {
        if memory_ids.is_empty() {
            return Ok(0);
        }

        self.bulk_operation(|tx| {
            let placeholders = memory_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let sql = format!(
                "UPDATE memories SET confidence = ? WHERE id IN ({}) AND deleted = 0",
                placeholders
            );

            let mut params: Vec<&dyn rusqlite::ToSql> = vec![&confidence];
            params.extend(memory_ids.iter().map(|id| id as &dyn rusqlite::ToSql));

            let affected = tx.execute(&sql, &params[..])?;
            Ok(affected)
        })
    }

    /// Bulk update memory type for multiple memories
    pub fn bulk_update_type(
        &mut self,
        memory_ids: &[String],
        memory_type: &MemoryType,
    ) -> Result<usize> {
        if memory_ids.is_empty() {
            return Ok(0);
        }

        let type_str = memory_type.to_string();

        self.bulk_operation(|tx| {
            let placeholders = memory_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let sql = format!(
                "UPDATE memories SET type = ? WHERE id IN ({}) AND deleted = 0",
                placeholders
            );

            let mut params: Vec<&dyn rusqlite::ToSql> = vec![&type_str];
            params.extend(memory_ids.iter().map(|id| id as &dyn rusqlite::ToSql));

            let affected = tx.execute(&sql, &params[..])?;
            Ok(affected)
        })
    }

    /// Bulk update source for multiple memories
    pub fn bulk_update_source(&mut self, memory_ids: &[String], source: &str) -> Result<usize> {
        if memory_ids.is_empty() {
            return Ok(0);
        }

        self.bulk_operation(|tx| {
            let placeholders = memory_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let sql = format!(
                "UPDATE memories SET source = ? WHERE id IN ({}) AND deleted = 0",
                placeholders
            );

            let mut params: Vec<&dyn rusqlite::ToSql> = vec![&source];
            params.extend(memory_ids.iter().map(|id| id as &dyn rusqlite::ToSql));

            let affected = tx.execute(&sql, &params[..])?;
            Ok(affected)
        })
    }

    /// Rename a tag across all memories
    #[allow(dead_code)] // Reserved for future tag management functionality
    pub fn rename_tag(&mut self, from: &str, to: &str) -> Result<usize> {
        if from == to {
            return Ok(0);
        }

        self.bulk_operation(|tx| {
            let affected = tx.execute(
                "UPDATE memories SET tags = REPLACE(tags, ?, ?) WHERE tags LIKE '%' || ? || '%'",
                params![from, to, from],
            )?;
            Ok(affected)
        })
    }

    /// Get all unique tags across all memories
    #[allow(dead_code)] // Reserved for future tag management functionality
    pub fn get_all_tags(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT tags FROM memories WHERE deleted = 0 AND tags != ''")?;
        let rows = stmt.query_map([], |row| {
            let tags_str: String = row.get(0)?;
            Ok(tags_str)
        })?;

        let mut all_tags = std::collections::HashSet::new();
        for row in rows {
            let tags_str = row?;
            for tag in tags_str.split(',') {
                if !tag.trim().is_empty() {
                    all_tags.insert(tag.trim().to_string());
                }
            }
        }

        let mut tags: Vec<String> = all_tags.into_iter().collect();
        tags.sort();
        Ok(tags)
    }

    /// Remove unused tags (implementation depends on requirements)
    #[allow(dead_code)] // Reserved for future tag cleanup functionality
    pub fn remove_unused_tags(&mut self) -> Result<usize> {
        // This is a placeholder - actual implementation would depend on
        // whether we store tags separately or inline in the memories table
        // For now, this is a no-op since tags are stored inline
        Ok(0)
    }

    /// Get tag usage statistics
    pub fn get_tag_stats(&self) -> Result<std::collections::HashMap<String, usize>> {
        let mut stmt = self
            .conn
            .prepare("SELECT tags FROM memories WHERE deleted = 0 AND tags != ''")?;
        let rows = stmt.query_map([], |row| {
            let tags_str: String = row.get(0)?;
            Ok(tags_str)
        })?;

        let mut tag_counts = std::collections::HashMap::new();
        for row in rows {
            let tags_str = row?;
            for tag in tags_str.split(',') {
                let tag = tag.trim();
                if !tag.is_empty() {
                    *tag_counts.entry(tag.to_string()).or_insert(0) += 1;
                }
            }
        }

        Ok(tag_counts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_find() {
        let mut repo = SqliteMemoryRepository::new_in_memory().unwrap();

        let memory = Memory::new(
            MemoryType::Tech,
            "Test Topic".to_string(),
            "Test Content".to_string(),
        );

        repo.save(&memory).unwrap();

        let found = repo.find_by_id(&memory.id).unwrap();
        assert!(found.is_some());

        let found_memory = found.unwrap();
        assert_eq!(found_memory.id, memory.id);
        assert_eq!(found_memory.topic, "Test Topic");
        assert_eq!(found_memory.content, "Test Content");
    }

    #[test]
    fn test_search() {
        let mut repo = SqliteMemoryRepository::new_in_memory().unwrap();

        let memory1 = Memory::with_tags(
            MemoryType::Tech,
            "Rust async programming".to_string(),
            "Rust uses async/await for asynchronous programming".to_string(),
            vec!["rust".to_string(), "async".to_string()],
        );

        let memory2 = Memory::with_tags(
            MemoryType::Tech,
            "Python decorators".to_string(),
            "Python decorators are functions that modify other functions".to_string(),
            vec!["python".to_string(), "decorator".to_string()],
        );

        repo.save(&memory1).unwrap();
        repo.save(&memory2).unwrap();

        let results = repo.search("rust async", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].topic, "Rust async programming");

        let results = repo.search("python", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].topic, "Python decorators");
    }

    #[test]
    fn test_update_reference_count() {
        let mut repo = SqliteMemoryRepository::new_in_memory().unwrap();

        let memory = Memory::new(
            MemoryType::Tech,
            "Test Topic".to_string(),
            "Test Content".to_string(),
        );

        repo.save(&memory).unwrap();
        repo.update_reference_count(&memory.id).unwrap();

        let found = repo.find_by_id(&memory.id).unwrap().unwrap();
        assert_eq!(found.reference_count, 1);

        repo.update_reference_count(&memory.id).unwrap();
        let found = repo.find_by_id(&memory.id).unwrap().unwrap();
        assert_eq!(found.reference_count, 2);
    }

    #[test]
    fn test_soft_delete() {
        let mut repo = SqliteMemoryRepository::new_in_memory().unwrap();

        let memory = Memory::new(
            MemoryType::Tech,
            "Test Topic".to_string(),
            "Test Content".to_string(),
        );

        repo.save(&memory).unwrap();
        repo.soft_delete(&memory.id).unwrap();

        let found = repo.find_by_id(&memory.id).unwrap();
        assert!(found.is_none());
    }
}
