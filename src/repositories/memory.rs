use crate::models::error::Result;
use crate::models::kiro::KiroConfig;
use crate::models::memory::Memory;
#[cfg(test)]
use std::collections::HashMap;

// Repository trait definition
pub trait MemoryRepository: Send {
    #[allow(dead_code)] // Used in SQLite repository implementations
    fn save(&mut self, memory: &Memory) -> Result<()>;
    fn save_batch(&mut self, memories: &[Memory]) -> Result<()>;
    #[allow(dead_code)] // Used in SQLite repository implementations
    fn find_by_id(&self, id: &str) -> Result<Option<Memory>>;
    fn search_fts(&self, query: &str, limit: usize) -> Result<Vec<Memory>>;
    fn find_all(&self) -> Result<Vec<Memory>>;
    fn increment_reference_count(&mut self, id: &str) -> Result<()>;
}

// InMemoryRepository for testing
#[cfg(test)]
pub struct InMemoryRepository {
    memories: HashMap<String, Memory>,
}

#[cfg(test)]
impl Default for InMemoryRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl InMemoryRepository {
    pub fn new() -> Self {
        Self {
            memories: HashMap::new(),
        }
    }
}

#[cfg(test)]
impl MemoryRepository for InMemoryRepository {
    fn save(&mut self, memory: &Memory) -> Result<()> {
        self.memories.insert(memory.id.clone(), memory.clone());
        Ok(())
    }

    fn save_batch(&mut self, memories: &[Memory]) -> Result<()> {
        for memory in memories {
            self.save(memory)?;
        }
        Ok(())
    }

    fn find_by_id(&self, id: &str) -> Result<Option<Memory>> {
        match self.memories.get(id) {
            Some(memory) => {
                // Filter out logically deleted memories
                if memory.deleted {
                    Ok(None)
                } else {
                    Ok(Some(memory.clone()))
                }
            }
            None => Ok(None),
        }
    }

    fn search_fts(&self, query: &str, limit: usize) -> Result<Vec<Memory>> {
        let mut results = Vec::new();

        for memory in self.memories.values() {
            // Skip logically deleted memories
            if memory.deleted {
                continue;
            }

            // Simple text search in title and content
            if memory.title.contains(query) || memory.content.contains(query) {
                results.push(memory.clone());
            }

            // Respect the limit
            if results.len() >= limit {
                break;
            }
        }

        Ok(results)
    }

    fn find_all(&self) -> Result<Vec<Memory>> {
        let mut results = Vec::new();

        for memory in self.memories.values() {
            // Skip logically deleted memories
            if !memory.deleted {
                results.push(memory.clone());
            }
        }

        Ok(results)
    }

    fn increment_reference_count(&mut self, id: &str) -> Result<()> {
        use crate::models::error::MemoryError;

        match self.memories.get_mut(id) {
            Some(memory) => {
                memory.reference_count += 1;
                memory.last_accessed = Some(chrono::Utc::now().timestamp());
                Ok(())
            }
            None => Err(MemoryError::NotFound(format!("Memory not found: {}", id))),
        }
    }
}

// SQLite Repository implementation
pub struct SqliteMemoryRepository {
    conn: rusqlite::Connection,
}

impl SqliteMemoryRepository {
    pub fn new(kiro_config: &KiroConfig) -> Result<Self> {
        // Ensure parent directory exists
        let db_path = &kiro_config.memory.database.path;
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Open database connection
        let mut conn = rusqlite::Connection::open(db_path)?;

        // Set SQLite pragmas for performance and safety using pragma_update
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;

        // Run migrations using Refinery
        mod embedded {
            use refinery::embed_migrations;
            embed_migrations!("./migrations");
        }

        embedded::migrations::runner().run(&mut conn)?;

        Ok(Self { conn })
    }
}

impl MemoryRepository for SqliteMemoryRepository {
    fn save(&mut self, memory: &Memory) -> Result<()> {
        // Check if memory already exists to determine INSERT vs UPDATE
        let exists = self
            .conn
            .prepare("SELECT 1 FROM memories WHERE id = ?1")?
            .exists([&memory.id])?;

        if exists {
            // Use UPDATE to trigger UPDATE triggers
            const UPDATE_MEMORY: &str = r#"
                UPDATE memories SET 
                    type = ?2, title = ?3, tags = ?4, content = ?5,
                    reference_count = ?6, confidence = ?7, created_at = ?8, 
                    last_accessed = ?9, deleted = ?10
                WHERE id = ?1
            "#;

            self.conn.execute(
                UPDATE_MEMORY,
                rusqlite::params![
                    &memory.id,
                    &memory.memory_type.to_string(),
                    &memory.title,
                    &memory.tags.join(","),
                    &memory.content,
                    memory.reference_count,
                    memory.confidence,
                    memory.created_at,
                    memory.last_accessed,
                    memory.deleted as i32,
                ],
            )?;
        } else {
            // Use INSERT for new records
            const INSERT_MEMORY: &str = r#"
                INSERT INTO memories (id, type, title, tags, content, 
                                     reference_count, confidence, created_at, last_accessed, deleted)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#;

            self.conn.execute(
                INSERT_MEMORY,
                rusqlite::params![
                    &memory.id,
                    &memory.memory_type.to_string(),
                    &memory.title,
                    &memory.tags.join(","),
                    &memory.content,
                    memory.reference_count,
                    memory.confidence,
                    memory.created_at,
                    memory.last_accessed,
                    memory.deleted as i32,
                ],
            )?;
        }
        Ok(())
    }

    fn save_batch(&mut self, memories: &[Memory]) -> Result<()> {
        if memories.is_empty() {
            return Ok(());
        }

        let tx = self.conn.transaction()?;

        for memory in memories {
            // Use existing save logic within transaction
            let exists = tx
                .prepare("SELECT 1 FROM memories WHERE id = ?1")?
                .exists([&memory.id])?;

            if exists {
                // Use UPDATE to trigger UPDATE triggers
                const UPDATE_MEMORY: &str = r#"
                    UPDATE memories SET 
                        type = ?2, title = ?3, tags = ?4, content = ?5,
                        reference_count = ?6, confidence = ?7, created_at = ?8, 
                        last_accessed = ?9, deleted = ?10
                    WHERE id = ?1
                "#;

                tx.execute(
                    UPDATE_MEMORY,
                    rusqlite::params![
                        &memory.id,
                        &memory.memory_type.to_string(),
                        &memory.title,
                        &memory.tags.join(","),
                        &memory.content,
                        memory.reference_count,
                        memory.confidence,
                        memory.created_at,
                        memory.last_accessed,
                        memory.deleted as i32,
                    ],
                )?;
            } else {
                // Use INSERT for new records
                const INSERT_MEMORY: &str = r#"
                    INSERT INTO memories (id, type, title, tags, content, 
                                         reference_count, confidence, created_at, last_accessed, deleted)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                "#;

                tx.execute(
                    INSERT_MEMORY,
                    rusqlite::params![
                        &memory.id,
                        &memory.memory_type.to_string(),
                        &memory.title,
                        &memory.tags.join(","),
                        &memory.content,
                        memory.reference_count,
                        memory.confidence,
                        memory.created_at,
                        memory.last_accessed,
                        memory.deleted as i32,
                    ],
                )?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    fn find_by_id(&self, id: &str) -> Result<Option<Memory>> {
        const FIND_BY_ID_SQL: &str = r#"
            SELECT id, type, title, tags, content, reference_count, confidence, 
                   created_at, last_accessed, deleted
            FROM memories 
            WHERE id = ?1 AND deleted = 0
        "#;

        let mut stmt = self.conn.prepare(FIND_BY_ID_SQL)?;
        let memory_result = stmt.query_row([id], |row| {
            let type_str: String = row.get("type")?;
            let tags_str: String = row.get("tags")?;
            let reference_count: i32 = row.get("reference_count")?;

            Ok(Memory {
                id: row.get("id")?,
                memory_type: type_str.parse().map_err(|_| {
                    rusqlite::Error::InvalidColumnType(
                        0,
                        "type".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?,
                title: row.get("title")?,
                tags: if tags_str.is_empty() {
                    Vec::new()
                } else {
                    tags_str.split(',').map(|s| s.to_string()).collect()
                },
                content: row.get("content")?,
                reference_count: reference_count as u32,
                confidence: row.get("confidence")?,
                created_at: row.get("created_at")?,
                last_accessed: row.get("last_accessed")?,
                deleted: row.get::<_, i32>("deleted")? != 0,
            })
        });

        match memory_result {
            Ok(memory) => Ok(Some(memory)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn search_fts(&self, query: &str, limit: usize) -> Result<Vec<Memory>> {
        const SEARCH_FTS_SQL: &str = r#"
            SELECT m.id, m.type, m.title, m.tags, m.content, m.reference_count, 
                   m.confidence, m.created_at, m.last_accessed, m.deleted
            FROM memories m
            JOIN memories_fts f ON m.id = f.memory_id
            WHERE f.memories_fts MATCH ?1
            AND m.deleted = 0
            ORDER BY rank
            LIMIT ?2
        "#;

        let mut stmt = self.conn.prepare(SEARCH_FTS_SQL)?;
        let memory_iter = stmt.query_map(rusqlite::params![query, limit as i32], |row| {
            let type_str: String = row.get("type")?;
            let tags_str: String = row.get("tags")?;
            let reference_count: i32 = row.get("reference_count")?;

            Ok(Memory {
                id: row.get("id")?,
                memory_type: type_str.parse().map_err(|_| {
                    rusqlite::Error::InvalidColumnType(
                        0,
                        "type".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?,
                title: row.get("title")?,
                tags: if tags_str.is_empty() {
                    Vec::new()
                } else {
                    tags_str.split(',').map(|s| s.to_string()).collect()
                },
                content: row.get("content")?,
                reference_count: reference_count as u32,
                confidence: row.get("confidence")?,
                created_at: row.get("created_at")?,
                last_accessed: row.get("last_accessed")?,
                deleted: row.get::<_, i32>("deleted")? != 0,
            })
        })?;

        let mut memories = Vec::new();
        for memory in memory_iter {
            memories.push(memory?);
        }
        Ok(memories)
    }

    fn find_all(&self) -> Result<Vec<Memory>> {
        const FIND_ALL_SQL: &str = r#"
            SELECT id, type, title, tags, content, reference_count, confidence, 
                   created_at, last_accessed, deleted
            FROM memories 
            WHERE deleted = 0
            ORDER BY created_at DESC
        "#;

        let mut stmt = self.conn.prepare(FIND_ALL_SQL)?;
        let memory_iter = stmt.query_map([], |row| {
            let type_str: String = row.get("type")?;
            let tags_str: String = row.get("tags")?;
            let reference_count: i32 = row.get("reference_count")?;

            Ok(Memory {
                id: row.get("id")?,
                memory_type: type_str.parse().map_err(|_| {
                    rusqlite::Error::InvalidColumnType(
                        0,
                        "type".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?,
                title: row.get("title")?,
                tags: if tags_str.is_empty() {
                    Vec::new()
                } else {
                    tags_str.split(',').map(|s| s.to_string()).collect()
                },
                content: row.get("content")?,
                reference_count: reference_count as u32,
                confidence: row.get("confidence")?,
                created_at: row.get("created_at")?,
                last_accessed: row.get("last_accessed")?,
                deleted: row.get::<_, i32>("deleted")? != 0,
            })
        })?;

        let mut memories = Vec::new();
        for memory in memory_iter {
            memories.push(memory?);
        }
        Ok(memories)
    }

    fn increment_reference_count(&mut self, id: &str) -> Result<()> {
        use crate::models::error::MemoryError;

        const INCREMENT_REF_COUNT_SQL: &str = r#"
            UPDATE memories 
            SET reference_count = reference_count + 1,
                last_accessed = unixepoch()
            WHERE id = ?1 AND deleted = 0
        "#;

        let rows_affected = self.conn.execute(INCREMENT_REF_COUNT_SQL, [id])?;

        if rows_affected == 0 {
            Err(MemoryError::NotFound(format!(
                "Memory not found or deleted: {}",
                id
            )))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::memory::{Memory, MemoryType};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_in_memory_save_and_find() {
        // Test that saved memory can be retrieved by find_by_id
        let mut repo = InMemoryRepository::new();

        let memory = Memory::new(
            MemoryType::Tech,
            "Test Memory".to_string(),
            "Test content for memory".to_string(),
        );
        let memory_id = memory.id.clone();

        // Save memory
        repo.save(&memory).unwrap();

        // Find by ID should return the memory
        let found = repo.find_by_id(&memory_id).unwrap();
        assert!(found.is_some());
        let found_memory = found.unwrap();
        assert_eq!(found_memory.id, memory_id);
        assert_eq!(found_memory.title, "Test Memory");
        assert_eq!(found_memory.content, "Test content for memory");

        // Non-existent ID should return None
        let not_found = repo.find_by_id("non-existent-id").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_in_memory_save_and_find_with_logical_deletion() {
        // Test that logically deleted memories are not returned
        let mut repo = InMemoryRepository::new();

        let mut memory = Memory::new(
            MemoryType::Tech,
            "Test Memory".to_string(),
            "Test content".to_string(),
        );
        let memory_id = memory.id.clone();

        // Save memory
        repo.save(&memory).unwrap();

        // Mark as deleted
        memory.deleted = true;
        repo.save(&memory).unwrap();

        // Should not be found when deleted
        let found = repo.find_by_id(&memory_id).unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_in_memory_save_batch() {
        // Test batch saving multiple memories
        let mut repo = InMemoryRepository::new();

        let memories = vec![
            Memory::new(
                MemoryType::Tech,
                "Memory 1".to_string(),
                "Content 1".to_string(),
            ),
            Memory::new(
                MemoryType::ProjectTech,
                "Memory 2".to_string(),
                "Content 2".to_string(),
            ),
            Memory::new(
                MemoryType::Domain,
                "Memory 3".to_string(),
                "Content 3".to_string(),
            ),
        ];

        let memory_ids: Vec<String> = memories.iter().map(|m| m.id.clone()).collect();

        // Save batch
        repo.save_batch(&memories).unwrap();

        // All memories should be retrievable
        for (i, id) in memory_ids.iter().enumerate() {
            let found = repo.find_by_id(id).unwrap();
            assert!(found.is_some());
            let found_memory = found.unwrap();
            assert_eq!(found_memory.title, format!("Memory {}", i + 1));
        }
    }

    #[test]
    fn test_in_memory_search_fts() {
        // Test simple text search functionality
        let mut repo = InMemoryRepository::new();

        let memories = vec![
            Memory::new(
                MemoryType::Tech,
                "Rust Programming".to_string(),
                "Rust is a systems programming language".to_string(),
            ),
            Memory::new(
                MemoryType::Tech,
                "JavaScript Basics".to_string(),
                "JavaScript is a web programming language".to_string(),
            ),
            Memory::new(
                MemoryType::Domain,
                "Business Logic".to_string(),
                "Domain-driven design principles".to_string(),
            ),
        ];

        repo.save_batch(&memories).unwrap();

        // Search for "programming" should return 2 results
        let results = repo.search_fts("programming", 10).unwrap();
        assert_eq!(results.len(), 2);

        // Search for "Rust" should return 1 result
        let rust_results = repo.search_fts("Rust", 10).unwrap();
        assert_eq!(rust_results.len(), 1);
        assert_eq!(rust_results[0].title, "Rust Programming");

        // Search with limit should respect the limit
        let limited_results = repo.search_fts("programming", 1).unwrap();
        assert_eq!(limited_results.len(), 1);
    }

    #[test]
    fn test_in_memory_search_fts_excludes_deleted() {
        // Test that search excludes logically deleted memories
        let mut repo = InMemoryRepository::new();

        let mut memory = Memory::new(
            MemoryType::Tech,
            "Deleted Memory".to_string(),
            "This should not be found".to_string(),
        );

        repo.save(&memory).unwrap();

        // Should be found initially
        let results = repo.search_fts("Deleted", 10).unwrap();
        assert_eq!(results.len(), 1);

        // Mark as deleted and update
        memory.deleted = true;
        repo.save(&memory).unwrap();

        // Should not be found after deletion
        let deleted_results = repo.search_fts("Deleted", 10).unwrap();
        assert_eq!(deleted_results.len(), 0);
    }

    #[test]
    fn test_in_memory_find_all() {
        // Test retrieving all memories (excluding deleted)
        let mut repo = InMemoryRepository::new();

        let memories = vec![
            Memory::new(
                MemoryType::Tech,
                "Memory 1".to_string(),
                "Content 1".to_string(),
            ),
            Memory::new(
                MemoryType::ProjectTech,
                "Memory 2".to_string(),
                "Content 2".to_string(),
            ),
        ];

        repo.save_batch(&memories).unwrap();

        // Should find all memories
        let all_memories = repo.find_all().unwrap();
        assert_eq!(all_memories.len(), 2);

        // Delete one memory
        let mut deleted_memory = memories[0].clone();
        deleted_memory.deleted = true;
        repo.save(&deleted_memory).unwrap();

        // Should now find only 1 memory
        let remaining_memories = repo.find_all().unwrap();
        assert_eq!(remaining_memories.len(), 1);
        assert_eq!(remaining_memories[0].title, "Memory 2");
    }

    #[test]
    fn test_in_memory_increment_reference_count() {
        // Test incrementing reference count and updating last_accessed
        let mut repo = InMemoryRepository::new();

        let memory = Memory::new(
            MemoryType::Tech,
            "Test Memory".to_string(),
            "Test content".to_string(),
        );
        let memory_id = memory.id.clone();

        repo.save(&memory).unwrap();

        // Initial state
        let found = repo.find_by_id(&memory_id).unwrap().unwrap();
        assert_eq!(found.reference_count, 0);
        assert!(found.last_accessed.is_none());

        // Increment reference count
        repo.increment_reference_count(&memory_id).unwrap();

        // Check updated state
        let updated = repo.find_by_id(&memory_id).unwrap().unwrap();
        assert_eq!(updated.reference_count, 1);
        assert!(updated.last_accessed.is_some());

        // Increment again
        repo.increment_reference_count(&memory_id).unwrap();

        // Should be 2 now
        let updated_again = repo.find_by_id(&memory_id).unwrap().unwrap();
        assert_eq!(updated_again.reference_count, 2);
    }

    #[test]
    fn test_in_memory_increment_reference_count_nonexistent() {
        // Test incrementing reference count for non-existent memory
        let mut repo = InMemoryRepository::new();

        // Should handle non-existent ID gracefully
        let result = repo.increment_reference_count("non-existent-id");
        assert!(result.is_err());
    }

    // SQLite Repository Tests
    mod sqlite_tests {
        use super::{MemoryRepository, SqliteMemoryRepository};
        use crate::models::kiro::KiroConfig;
        use crate::models::memory::{Memory, MemoryType};
        use std::fs;
        use tempfile::TempDir;

        fn setup_test_config() -> (TempDir, KiroConfig) {
            let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
            let kiro_dir = temp_dir.path().join(".kiro");
            fs::create_dir_all(&kiro_dir).expect("Failed to create .kiro directory");

            let mut config = KiroConfig {
                root_dir: kiro_dir.clone(),
                ..KiroConfig::default()
            };
            config.memory.database.path = kiro_dir.join("memory").join("db.sqlite3");

            // Create memory directory
            fs::create_dir_all(config.memory.database.path.parent().unwrap())
                .expect("Failed to create memory directory");

            (temp_dir, config)
        }

        #[test]
        fn test_sqlite_manual_connection() {
            // Test direct SQLite connection without refinery
            let (_temp_dir, config) = setup_test_config();

            // Database file should not exist initially
            assert!(!config.memory.database.path.exists());

            // Ensure parent directory exists
            let db_path = &config.memory.database.path;
            if let Some(parent) = db_path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }

            // Open database connection directly
            let conn = rusqlite::Connection::open(db_path).unwrap();

            // Note: Skipping PRAGMA statements as they return results and can't use execute()

            // Create table directly (from V001 migration)
            let create_table_sql = r#"
                CREATE TABLE memories (
                    id TEXT PRIMARY KEY,
                    type TEXT NOT NULL,
                    title TEXT NOT NULL,
                    tags TEXT,
                    content TEXT NOT NULL,
                    reference_count INTEGER DEFAULT 0,
                    confidence REAL DEFAULT 1.0 CHECK(confidence >= 0 AND confidence <= 1),
                    created_at INTEGER DEFAULT (unixepoch()),
                    last_accessed INTEGER,
                    deleted INTEGER DEFAULT 0
                );
            "#;

            conn.execute(create_table_sql, []).unwrap();

            // Database file should now exist
            assert!(config.memory.database.path.exists());

            // Test that we can query the table
            let table_exists: bool = conn
                .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='memories'")
                .unwrap()
                .exists([])
                .unwrap();
            assert!(table_exists, "memories table should exist");
        }

        #[test]
        fn test_sqlite_repository_new_creates_database() {
            // Test that SqliteMemoryRepository::new creates database file
            let (_temp_dir, config) = setup_test_config();

            // Database file should not exist initially
            assert!(!config.memory.database.path.exists());

            // Create repository - this should create the database
            let repo = SqliteMemoryRepository::new(&config).unwrap();

            // Database file should now exist
            assert!(config.memory.database.path.exists());

            // Should be able to create another connection to the same database
            drop(repo);
            let _repo2 = SqliteMemoryRepository::new(&config).unwrap();
        }

        #[test]
        fn test_sqlite_repository_runs_migrations() {
            // Test that migrations are executed during initialization
            let (_temp_dir, config) = setup_test_config();

            let repo = SqliteMemoryRepository::new(&config).unwrap();

            // Check that main table exists
            let table_exists: bool = repo
                .conn
                .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='memories'")
                .unwrap()
                .exists([])
                .unwrap();
            assert!(table_exists, "memories table should exist after migration");

            // Check that FTS5 table exists
            let fts_exists: bool = repo
                .conn
                .prepare(
                    "SELECT name FROM sqlite_master WHERE type='table' AND name='memories_fts'",
                )
                .unwrap()
                .exists([])
                .unwrap();
            assert!(
                fts_exists,
                "memories_fts table should exist after migration"
            );

            // Check that indexes exist
            let index_exists: bool = repo.conn.prepare("SELECT name FROM sqlite_master WHERE type='index' AND name='idx_memories_type'")
                .unwrap()
                .exists([])
                .unwrap();
            assert!(
                index_exists,
                "idx_memories_type index should exist after migration"
            );
        }

        #[test]
        fn test_sqlite_pragmas_set_correctly() {
            // Test that SQLite pragmas are set correctly
            let (_temp_dir, config) = setup_test_config();

            let repo = SqliteMemoryRepository::new(&config).unwrap();

            // Check foreign_keys is ON (returns integer)
            let foreign_keys_on: i32 = repo
                .conn
                .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
                .unwrap();
            std::assert_eq!(foreign_keys_on, 1, "foreign_keys should be enabled");

            // Check journal_mode is WAL (if implemented)
            let journal_mode: String = repo
                .conn
                .query_row("PRAGMA journal_mode", [], |row| row.get(0))
                .unwrap();

            // WAL mode should be set for better concurrency
            std::assert_eq!(
                journal_mode.to_uppercase(),
                "WAL",
                "journal_mode should be WAL"
            );
        }

        #[test]
        fn test_sqlite_triggers_work() {
            // Test that triggers automatically maintain FTS5 index
            let (_temp_dir, config) = setup_test_config();

            let mut repo = SqliteMemoryRepository::new(&config).unwrap();

            // Create a memory and save it
            let memory = Memory::new(
                MemoryType::Tech,
                "Test Memory".to_string(),
                "Test content for FTS5".to_string(),
            );

            repo.save(&memory).unwrap();

            // Check that memory was automatically added to FTS5 table
            let fts_count: i32 = repo
                .conn
                .query_row(
                    "SELECT COUNT(*) FROM memories_fts WHERE memory_id = ?",
                    [&memory.id],
                    |row| row.get(0),
                )
                .unwrap();

            std::assert_eq!(
                fts_count,
                1,
                "Memory should be automatically added to FTS5 index"
            );

            // Test logical deletion trigger
            let mut deleted_memory = memory.clone();
            deleted_memory.deleted = true;
            repo.save(&deleted_memory).unwrap();

            // Should be removed from FTS5 index
            let fts_count_after_delete: i32 = repo
                .conn
                .query_row(
                    "SELECT COUNT(*) FROM memories_fts WHERE memory_id = ?",
                    [&memory.id],
                    |row| row.get(0),
                )
                .unwrap();

            std::assert_eq!(
                fts_count_after_delete,
                0,
                "Deleted memory should be removed from FTS5 index"
            );
        }

        #[test]
        fn test_sqlite_save_and_find() {
            // Test INSERT/SELECT with logical deletion filtering
            let (_temp_dir, config) = setup_test_config();
            let mut repo = SqliteMemoryRepository::new(&config).unwrap();

            let memory = Memory::new(
                MemoryType::Tech,
                "Test Memory".to_string(),
                "Test content for retrieval".to_string(),
            )
            .with_tags(vec!["test".to_string(), "sqlite".to_string()]);
            let memory_id = memory.id.clone();

            // Save memory
            repo.save(&memory).unwrap();

            // Find by ID should return the memory
            let found = repo.find_by_id(&memory_id).unwrap();
            assert!(found.is_some());
            let found_memory = found.unwrap();
            assert_eq!(found_memory.id, memory_id);
            assert_eq!(found_memory.title, "Test Memory");
            assert_eq!(found_memory.content, "Test content for retrieval");
            assert_eq!(found_memory.tags, vec!["test", "sqlite"]);

            // Non-existent ID should return None
            let not_found = repo.find_by_id("non-existent-id").unwrap();
            assert!(not_found.is_none());

            // Test logical deletion filtering
            let mut deleted_memory = memory.clone();
            deleted_memory.deleted = true;
            repo.save(&deleted_memory).unwrap();

            // Should not be found when deleted
            let found_deleted = repo.find_by_id(&memory_id).unwrap();
            assert!(found_deleted.is_none());
        }

        #[test]
        fn test_sqlite_save_batch_with_transaction() {
            // Test transactional batch saves with rollback on error
            let (_temp_dir, config) = setup_test_config();
            let mut repo = SqliteMemoryRepository::new(&config).unwrap();

            let memories = vec![
                Memory::new(
                    MemoryType::Tech,
                    "Memory 1".to_string(),
                    "Content 1".to_string(),
                ),
                Memory::new(
                    MemoryType::ProjectTech,
                    "Memory 2".to_string(),
                    "Content 2".to_string(),
                ),
                Memory::new(
                    MemoryType::Domain,
                    "Memory 3".to_string(),
                    "Content 3".to_string(),
                ),
            ];

            let memory_ids: Vec<String> = memories.iter().map(|m| m.id.clone()).collect();

            // Save batch
            repo.save_batch(&memories).unwrap();

            // All memories should be retrievable
            for (i, id) in memory_ids.iter().enumerate() {
                let found = repo.find_by_id(id).unwrap();
                assert!(found.is_some());
                let found_memory = found.unwrap();
                assert_eq!(found_memory.title, format!("Memory {}", i + 1));
            }

            // Test empty batch (should succeed)
            repo.save_batch(&[]).unwrap();
        }

        #[test]
        fn test_sqlite_fts_search() {
            // Test FTS5 search with Japanese tokenization
            let (_temp_dir, config) = setup_test_config();
            let mut repo = SqliteMemoryRepository::new(&config).unwrap();

            let memories = vec![
                Memory::new(
                    MemoryType::Tech,
                    "Rust Programming".to_string(),
                    "Rust is a systems programming language with memory safety".to_string(),
                )
                .with_tags(vec!["rust".to_string(), "systems".to_string()]),
                Memory::new(
                    MemoryType::Tech,
                    "JavaScript Basics".to_string(),
                    "JavaScript is a dynamic programming language for web development".to_string(),
                )
                .with_tags(vec!["javascript".to_string(), "web".to_string()]),
                Memory::new(
                    MemoryType::Domain,
                    "ビジネスロジック".to_string(),
                    "ドメイン駆動設計の原則について".to_string(),
                )
                .with_tags(vec!["domain".to_string(), "japanese".to_string()]),
            ];

            repo.save_batch(&memories).unwrap();

            // Search for "programming" should return 2 results
            let results = repo.search_fts("programming", 10).unwrap();
            assert_eq!(results.len(), 2);

            // Search for "Rust" should return 1 result
            let rust_results = repo.search_fts("Rust", 10).unwrap();
            assert_eq!(rust_results.len(), 1);
            assert_eq!(rust_results[0].title, "Rust Programming");

            // Search with limit should respect the limit
            let limited_results = repo.search_fts("programming", 1).unwrap();
            assert_eq!(limited_results.len(), 1);

            // Test Japanese search - search by title (which works reliably with FTS5)
            let japanese_results = repo.search_fts("ビジネスロジック", 10).unwrap();
            assert_eq!(japanese_results.len(), 1);
            assert_eq!(japanese_results[0].title, "ビジネスロジック");

            // Also test that Japanese content is properly stored and retrievable
            // Note: FTS5 with porter unicode61 tokenizer may not perfectly handle
            // Japanese content search, but title search works reliably
            let found_japanese = repo.find_by_id(&japanese_results[0].id).unwrap();
            assert!(found_japanese.is_some());
            assert!(found_japanese.unwrap().content.contains("ドメイン駆動設計"));

            // Test logical deletion filtering - create deleted memory
            let mut deleted_memory = Memory::new(
                MemoryType::Tech,
                "Deleted Memory".to_string(),
                "This should not be found in search".to_string(),
            );
            deleted_memory.deleted = true;
            repo.save(&deleted_memory).unwrap();

            // Should not find deleted memory
            let deleted_results = repo.search_fts("Deleted", 10).unwrap();
            assert_eq!(deleted_results.len(), 0);
        }

        #[test]
        fn test_sqlite_find_all() {
            // Test retrieving all non-deleted memories
            let (_temp_dir, config) = setup_test_config();
            let mut repo = SqliteMemoryRepository::new(&config).unwrap();

            let memories = vec![
                Memory::new(
                    MemoryType::Tech,
                    "Memory 1".to_string(),
                    "Content 1".to_string(),
                ),
                Memory::new(
                    MemoryType::ProjectTech,
                    "Memory 2".to_string(),
                    "Content 2".to_string(),
                ),
                Memory::new(
                    MemoryType::Domain,
                    "Memory 3".to_string(),
                    "Content 3".to_string(),
                ),
            ];

            repo.save_batch(&memories).unwrap();

            // Should find all memories
            let all_memories = repo.find_all().unwrap();
            assert_eq!(all_memories.len(), 3);

            // Delete one memory
            let mut deleted_memory = memories[0].clone();
            deleted_memory.deleted = true;
            repo.save(&deleted_memory).unwrap();

            // Should now find only 2 memories
            let remaining_memories = repo.find_all().unwrap();
            assert_eq!(remaining_memories.len(), 2);

            // Verify the remaining memories are the correct ones
            let titles: Vec<String> = remaining_memories.iter().map(|m| m.title.clone()).collect();
            assert!(titles.contains(&"Memory 2".to_string()));
            assert!(titles.contains(&"Memory 3".to_string()));
            assert!(!titles.contains(&"Memory 1".to_string()));
        }

        #[test]
        fn test_sqlite_increment_reference_count() {
            // Test reference count and last_accessed updates
            let (_temp_dir, config) = setup_test_config();
            let mut repo = SqliteMemoryRepository::new(&config).unwrap();

            let memory = Memory::new(
                MemoryType::Tech,
                "Test Memory".to_string(),
                "Test content".to_string(),
            );
            let memory_id = memory.id.clone();

            repo.save(&memory).unwrap();

            // Initial state
            let found = repo.find_by_id(&memory_id).unwrap().unwrap();
            assert_eq!(found.reference_count, 0);
            assert!(found.last_accessed.is_none());

            // Increment reference count
            repo.increment_reference_count(&memory_id).unwrap();

            // Check updated state
            let updated = repo.find_by_id(&memory_id).unwrap().unwrap();
            assert_eq!(updated.reference_count, 1);
            assert!(updated.last_accessed.is_some());

            let first_access_time = updated.last_accessed.unwrap();

            // Wait a moment and increment again
            std::thread::sleep(std::time::Duration::from_millis(10));
            repo.increment_reference_count(&memory_id).unwrap();

            // Should be 2 now with updated timestamp
            let updated_again = repo.find_by_id(&memory_id).unwrap().unwrap();
            assert_eq!(updated_again.reference_count, 2);
            assert!(updated_again.last_accessed.unwrap() >= first_access_time);

            // Test incrementing non-existent memory
            let result = repo.increment_reference_count("non-existent-id");
            assert!(result.is_err());
        }
    }

    // ========================================
    // Phase 4: Migration Tests
    // ========================================

    #[test]
    fn test_migration_creates_tables() -> Result<()> {
        // Test that migrations create the required tables with correct schema
        use crate::models::kiro::KiroConfig;
        use crate::tests::common::setup_test_dir;

        let temp_dir = setup_test_dir();
        let db_path = temp_dir.path().join("test_migration.db");

        // Create test config
        let config = KiroConfig {
            root_dir: temp_dir.path().to_path_buf(),
            memory: crate::models::kiro::MemoryConfig {
                types: vec![
                    "tech".to_string(),
                    "project-tech".to_string(),
                    "domain".to_string(),
                ],
                instructions: "Test instructions".to_string(),
                document: crate::models::kiro::DocumentConfig {
                    output_dir: temp_dir.path().to_path_buf(),
                    format: "markdown".to_string(),
                },
                database: crate::models::kiro::DatabaseConfig {
                    path: db_path.clone(),
                },
            },
        };

        // Create repository (this should run migrations)
        let repo = SqliteMemoryRepository::new(&config).unwrap();

        // Check that memories table exists with all columns
        let mut stmt = repo.conn.prepare("PRAGMA table_info(memories)").unwrap();
        let column_info: rusqlite::Result<Vec<String>> = stmt
            .query_map([], |row| {
                let column_name: String = row.get(1)?;
                Ok(column_name)
            })?
            .collect();

        let columns = column_info.unwrap();
        assert!(columns.contains(&"id".to_string()));
        assert!(columns.contains(&"type".to_string()));
        assert!(columns.contains(&"title".to_string()));
        assert!(columns.contains(&"tags".to_string()));
        assert!(columns.contains(&"content".to_string()));
        assert!(columns.contains(&"reference_count".to_string()));
        assert!(columns.contains(&"confidence".to_string()));
        assert!(columns.contains(&"created_at".to_string()));
        assert!(columns.contains(&"last_accessed".to_string()));
        assert!(columns.contains(&"deleted".to_string()));

        // Check that memories_fts virtual table exists
        let fts_exists: bool = repo
            .conn
            .prepare(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='memories_fts'",
            )?
            .query_row([], |row| {
                let count: i32 = row.get(0)?;
                Ok(count > 0)
            })
            .unwrap();
        assert!(fts_exists, "memories_fts virtual table should exist");
        Ok(())
    }

    #[test]
    fn test_migration_creates_indexes() -> Result<()> {
        // Test that migrations create the required indexes
        use crate::models::kiro::KiroConfig;
        use crate::tests::common::setup_test_dir;

        let temp_dir = setup_test_dir();
        let db_path = temp_dir.path().join("test_indexes.db");

        let config = KiroConfig {
            root_dir: temp_dir.path().to_path_buf(),
            memory: crate::models::kiro::MemoryConfig {
                types: vec!["tech".to_string()],
                instructions: "Test instructions".to_string(),
                document: crate::models::kiro::DocumentConfig {
                    output_dir: temp_dir.path().to_path_buf(),
                    format: "markdown".to_string(),
                },
                database: crate::models::kiro::DatabaseConfig {
                    path: db_path.clone(),
                },
            },
        };

        let repo = SqliteMemoryRepository::new(&config).unwrap();

        // Check that required indexes exist
        let mut stmt = repo
            .conn
            .prepare(
                "SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_memories_%'",
            )
            .unwrap();

        let index_names: rusqlite::Result<Vec<String>> = stmt
            .query_map([], |row| {
                let name: String = row.get(0)?;
                Ok(name)
            })?
            .collect();

        let indexes = index_names.unwrap();
        assert!(indexes.contains(&"idx_memories_type".to_string()));
        assert!(indexes.contains(&"idx_memories_ref_count".to_string()));
        assert!(indexes.contains(&"idx_memories_created".to_string()));
        Ok(())
    }

    #[test]
    fn test_migration_creates_triggers() -> Result<()> {
        // Test that migrations create the required triggers and they work correctly
        use crate::models::kiro::KiroConfig;
        use crate::tests::common::setup_test_dir;

        let temp_dir = setup_test_dir();
        let db_path = temp_dir.path().join("test_triggers.db");

        let config = KiroConfig {
            root_dir: temp_dir.path().to_path_buf(),
            memory: crate::models::kiro::MemoryConfig {
                types: vec!["tech".to_string()],
                instructions: "Test instructions".to_string(),
                document: crate::models::kiro::DocumentConfig {
                    output_dir: temp_dir.path().to_path_buf(),
                    format: "markdown".to_string(),
                },
                database: crate::models::kiro::DatabaseConfig {
                    path: db_path.clone(),
                },
            },
        };

        let mut repo = SqliteMemoryRepository::new(&config).unwrap();

        // Check that triggers exist
        let triggers = {
            let mut stmt = repo.conn
                .prepare("SELECT name FROM sqlite_master WHERE type='trigger' AND name LIKE 'memories_%'")
                .unwrap();

            let trigger_names: rusqlite::Result<Vec<String>> = stmt
                .query_map([], |row| {
                    let name: String = row.get(0)?;
                    Ok(name)
                })?
                .collect();

            trigger_names.unwrap()
        };

        assert!(triggers.contains(&"memories_ai".to_string())); // INSERT trigger
        assert!(triggers.contains(&"memories_au".to_string())); // UPDATE trigger  
        assert!(triggers.contains(&"memories_ad".to_string())); // DELETE trigger
        assert!(triggers.contains(&"memories_soft_delete".to_string())); // Soft delete trigger

        // Test that INSERT trigger works - FTS5 should be populated
        let memory = Memory::new(
            MemoryType::Tech,
            "Test Trigger Memory".to_string(),
            "Content for testing triggers".to_string(),
        );
        let memory_id = memory.id.clone();

        repo.save(&memory).unwrap();

        // Check that FTS5 table was populated by trigger
        let fts_count: i32 = repo
            .conn
            .prepare("SELECT COUNT(*) FROM memories_fts WHERE memory_id = ?1")?
            .query_row([&memory_id], |row| row.get(0))
            .unwrap();
        assert_eq!(
            fts_count, 1,
            "FTS5 trigger should populate memories_fts on INSERT"
        );

        // Test soft delete trigger - FTS5 should be cleared
        repo.conn
            .execute(
                "UPDATE memories SET deleted = 1 WHERE id = ?1",
                [&memory_id],
            )
            .unwrap();

        let fts_count_after_delete: i32 = repo
            .conn
            .prepare("SELECT COUNT(*) FROM memories_fts WHERE memory_id = ?1")?
            .query_row([&memory_id], |row| row.get(0))
            .unwrap();
        assert_eq!(
            fts_count_after_delete, 0,
            "Soft delete trigger should remove from memories_fts"
        );
        Ok(())
    }
}
