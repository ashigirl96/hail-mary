use crate::application::errors::ApplicationError;
use crate::application::repositories::MemoryRepository;
use crate::domain::entities::memory::Memory;
use std::collections::HashMap;
use uuid::Uuid;

/// Mock implementation of MemoryRepository for testing
/// Combines features from all existing mock implementations
#[derive(Debug, Default)]
pub struct MockMemoryRepository {
    memories: HashMap<Uuid, Memory>,
    // Call tracking
    save_calls: u32,
    save_batch_calls: u32,
    find_by_id_calls: u32,
    search_fts_calls: u32,
    find_by_type_calls: u32,
    find_all_calls: u32,
    increment_ref_calls: u32,
    cleanup_deleted_calls: u32,
    rebuild_fts_calls: u32,
    vacuum_calls: u32,
    // Failure simulation
    should_fail_next_save: bool,
    should_fail: bool,
    fail_on_batch: bool,
    fail_cleanup: bool,
    // Custom behavior
    deleted_count: usize,
    cleanup_called: bool,
    rebuild_called: bool,
    vacuum_called: bool,
    search_results: Vec<Memory>,
    should_fail_search: bool,
}

impl MockMemoryRepository {
    pub fn new() -> Self {
        Self::default()
    }

    // Call tracking methods
    pub fn get_save_calls(&self) -> u32 {
        self.save_calls
    }

    pub fn get_save_batch_calls(&self) -> u32 {
        self.save_batch_calls
    }

    pub fn memory_count(&self) -> usize {
        self.memories.len()
    }

    // Failure simulation
    pub fn set_next_save_to_fail(&mut self) {
        self.should_fail_next_save = true;
    }

    pub fn set_should_fail(&mut self, should_fail: bool) {
        self.should_fail = should_fail;
    }

    pub fn with_batch_failure(mut self) -> Self {
        self.fail_on_batch = true;
        self
    }

    pub fn set_fail_cleanup(&mut self, fail: bool) {
        self.fail_cleanup = fail;
    }

    // Custom behavior
    pub fn set_deleted_count(&mut self, count: usize) {
        self.deleted_count = count;
    }

    pub fn add_memory(&mut self, memory: Memory) {
        self.memories.insert(memory.id, memory);
    }

    pub fn is_cleanup_called(&self) -> bool {
        self.cleanup_called
    }

    pub fn is_rebuild_called(&self) -> bool {
        self.rebuild_called
    }

    pub fn is_vacuum_called(&self) -> bool {
        self.vacuum_called
    }

    pub fn with_memories(mut self, memories: Vec<Memory>) -> Self {
        for memory in &memories {
            self.memories.insert(memory.id, memory.clone());
        }
        self.search_results = memories;
        self
    }

    pub fn with_search_failure(mut self) -> Self {
        self.should_fail_search = true;
        self
    }

    pub fn add_memory_by_type(&mut self, _memory_type: &str, memory: Memory) {
        self.memories.insert(memory.id, memory);
    }
}

impl MemoryRepository for MockMemoryRepository {
    fn save(&mut self, memory: &Memory) -> Result<(), ApplicationError> {
        self.save_calls += 1;
        if self.should_fail || self.should_fail_next_save {
            self.should_fail_next_save = false;
            return Err(ApplicationError::DatabaseError(
                "Simulated save failure".to_string(),
            ));
        }
        self.memories.insert(memory.id, memory.clone());
        Ok(())
    }

    fn save_batch(&mut self, memories: &[Memory]) -> Result<(), ApplicationError> {
        self.save_batch_calls += 1;
        if self.fail_on_batch {
            return Err(ApplicationError::DatabaseError(
                "Batch save failed".to_string(),
            ));
        }
        for memory in memories {
            self.memories.insert(memory.id, memory.clone());
        }
        Ok(())
    }

    fn find_by_id(&mut self, id: &Uuid) -> Result<Option<Memory>, ApplicationError> {
        self.find_by_id_calls += 1;
        Ok(self.memories.get(id).cloned())
    }

    fn search_fts(&mut self, query: &str, limit: usize) -> Result<Vec<Memory>, ApplicationError> {
        self.search_fts_calls += 1;
        if self.should_fail_search {
            return Err(ApplicationError::DatabaseError("Search failed".to_string()));
        }

        if !self.search_results.is_empty() {
            return Ok(self.search_results.clone());
        }

        let mut results: Vec<Memory> = self
            .memories
            .values()
            .filter(|m| {
                !m.deleted
                    && (m.title.contains(query)
                        || m.content.contains(query)
                        || m.tags.iter().any(|tag| tag.contains(query)))
            })
            .cloned()
            .collect();

        // Sort by confidence and reference count (descending)
        results.sort_by(|a, b| {
            b.confidence
                .value()
                .partial_cmp(&a.confidence.value())
                .unwrap()
                .then(b.reference_count.cmp(&a.reference_count))
        });

        results.truncate(limit);
        Ok(results)
    }

    fn find_by_type(&mut self, memory_type: &str) -> Result<Vec<Memory>, ApplicationError> {
        self.find_by_type_calls += 1;
        Ok(self
            .memories
            .values()
            .filter(|m| m.memory_type == memory_type && !m.deleted)
            .cloned()
            .collect())
    }

    fn find_all(&mut self) -> Result<Vec<Memory>, ApplicationError> {
        self.find_all_calls += 1;
        Ok(self
            .memories
            .values()
            .filter(|m| !m.deleted)
            .cloned()
            .collect())
    }

    fn increment_reference_count(&mut self, id: &Uuid) -> Result<(), ApplicationError> {
        self.increment_ref_calls += 1;
        if let Some(memory) = self.memories.get_mut(id) {
            memory.reference_count += 1;
            memory.last_accessed = Some(chrono::Utc::now());
        }
        Ok(())
    }

    fn cleanup_deleted(&mut self) -> Result<usize, ApplicationError> {
        self.cleanup_deleted_calls += 1;
        self.cleanup_called = true;
        if self.fail_cleanup {
            return Err(ApplicationError::DatabaseError(
                "Cleanup failed".to_string(),
            ));
        }
        if self.deleted_count > 0 {
            let initial_count = self.memories.len();
            self.memories.retain(|_, memory| !memory.deleted);
            let removed = initial_count - self.memories.len();
            if removed > 0 {
                return Ok(removed);
            }
        }
        Ok(self.deleted_count)
    }

    fn rebuild_fts_index(&mut self) -> Result<(), ApplicationError> {
        self.rebuild_fts_calls += 1;
        self.rebuild_called = true;
        Ok(())
    }

    fn vacuum(&mut self) -> Result<(), ApplicationError> {
        self.vacuum_calls += 1;
        self.vacuum_called = true;
        Ok(())
    }
}
