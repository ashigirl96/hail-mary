use crate::application::errors::ApplicationError;
use crate::domain::entities::memory::Memory;
use uuid::Uuid;

pub trait MemoryRepository: Send + Sync {
    fn save(&mut self, memory: &Memory) -> Result<(), ApplicationError>;
    fn save_batch(&mut self, memories: &[Memory]) -> Result<(), ApplicationError>;
    fn find_by_id(&mut self, id: &Uuid) -> Result<Option<Memory>, ApplicationError>;
    fn search_fts(&mut self, query: &str, limit: usize) -> Result<Vec<Memory>, ApplicationError>;
    fn find_by_type(&mut self, memory_type: &str) -> Result<Vec<Memory>, ApplicationError>;
    fn find_all(&mut self) -> Result<Vec<Memory>, ApplicationError>;
    fn increment_reference_count(&mut self, id: &Uuid) -> Result<(), ApplicationError>;
    fn cleanup_deleted(&mut self) -> Result<usize, ApplicationError>;
    fn rebuild_fts_index(&mut self) -> Result<(), ApplicationError>;
    fn vacuum(&mut self) -> Result<(), ApplicationError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::confidence::Confidence;
    use std::collections::HashMap;

    // Mock implementation for testing
    #[derive(Debug, Default)]
    struct MockMemoryRepository {
        memories: HashMap<Uuid, Memory>,
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
        should_fail_next_save: bool,
    }

    impl MemoryRepository for MockMemoryRepository {
        fn save(&mut self, memory: &Memory) -> Result<(), ApplicationError> {
            self.save_calls += 1;
            if self.should_fail_next_save {
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
            for memory in memories {
                self.memories.insert(memory.id, memory.clone());
            }
            Ok(())
        }

        fn find_by_id(&mut self, id: &Uuid) -> Result<Option<Memory>, ApplicationError> {
            self.find_by_id_calls += 1;
            Ok(self.memories.get(id).cloned())
        }

        fn search_fts(
            &mut self,
            query: &str,
            limit: usize,
        ) -> Result<Vec<Memory>, ApplicationError> {
            self.search_fts_calls += 1;
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
            let initial_count = self.memories.len();
            self.memories.retain(|_, memory| !memory.deleted);
            Ok(initial_count - self.memories.len())
        }

        fn rebuild_fts_index(&mut self) -> Result<(), ApplicationError> {
            self.rebuild_fts_calls += 1;
            Ok(())
        }

        fn vacuum(&mut self) -> Result<(), ApplicationError> {
            self.vacuum_calls += 1;
            Ok(())
        }
    }

    impl MockMemoryRepository {
        fn new() -> Self {
            Self::default()
        }

        fn set_next_save_to_fail(&mut self) {
            self.should_fail_next_save = true;
        }

        fn get_save_calls(&self) -> u32 {
            self.save_calls
        }

        fn get_save_batch_calls(&self) -> u32 {
            self.save_batch_calls
        }

        fn memory_count(&self) -> usize {
            self.memories.len()
        }
    }

    #[test]
    fn test_memory_repository_save() {
        let mut repo = MockMemoryRepository::new();
        let memory = Memory::new(
            "tech".to_string(),
            "Test Memory".to_string(),
            "Test content".to_string(),
        );

        let result = repo.save(&memory);
        assert!(result.is_ok());
        assert_eq!(repo.get_save_calls(), 1);

        let found = repo.find_by_id(&memory.id).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Test Memory");
    }

    #[test]
    fn test_memory_repository_save_batch() {
        let mut repo = MockMemoryRepository::new();
        let memories = vec![
            Memory::new(
                "tech".to_string(),
                "Memory 1".to_string(),
                "Content 1".to_string(),
            ),
            Memory::new(
                "domain".to_string(),
                "Memory 2".to_string(),
                "Content 2".to_string(),
            ),
        ];

        let result = repo.save_batch(&memories);
        assert!(result.is_ok());
        assert_eq!(repo.get_save_batch_calls(), 1);

        let all_memories = repo.find_all().unwrap();
        assert_eq!(all_memories.len(), 2);
    }

    #[test]
    fn test_memory_repository_find_by_id() {
        let mut repo = MockMemoryRepository::new();
        let memory = Memory::new(
            "tech".to_string(),
            "Test Memory".to_string(),
            "Test content".to_string(),
        );
        let memory_id = memory.id;

        repo.save(&memory).unwrap();

        let found = repo.find_by_id(&memory_id).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, memory_id);

        let not_found = repo.find_by_id(&Uuid::new_v4()).unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_memory_repository_search_fts() {
        let mut repo = MockMemoryRepository::new();
        let memory1 = Memory::new(
            "tech".to_string(),
            "Rust Programming".to_string(),
            "Rust is a systems programming language".to_string(),
        );
        let memory2 = Memory::new(
            "tech".to_string(),
            "Python Basics".to_string(),
            "Python is a high-level language".to_string(),
        );

        repo.save(&memory1).unwrap();
        repo.save(&memory2).unwrap();

        let results = repo.search_fts("Rust", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust Programming");

        let results = repo.search_fts("language", 10).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_memory_repository_find_by_type() {
        let mut repo = MockMemoryRepository::new();
        let tech_memory = Memory::new(
            "tech".to_string(),
            "Tech Memory".to_string(),
            "Tech content".to_string(),
        );
        let domain_memory = Memory::new(
            "domain".to_string(),
            "Domain Memory".to_string(),
            "Domain content".to_string(),
        );

        repo.save(&tech_memory).unwrap();
        repo.save(&domain_memory).unwrap();

        let tech_memories = repo.find_by_type("tech").unwrap();
        assert_eq!(tech_memories.len(), 1);
        assert_eq!(tech_memories[0].memory_type, "tech");

        let domain_memories = repo.find_by_type("domain").unwrap();
        assert_eq!(domain_memories.len(), 1);
        assert_eq!(domain_memories[0].memory_type, "domain");
    }

    #[test]
    fn test_memory_repository_increment_reference_count() {
        let mut repo = MockMemoryRepository::new();
        let memory = Memory::new(
            "tech".to_string(),
            "Test Memory".to_string(),
            "Test content".to_string(),
        );
        let memory_id = memory.id;

        repo.save(&memory).unwrap();
        repo.increment_reference_count(&memory_id).unwrap();

        let updated = repo.find_by_id(&memory_id).unwrap().unwrap();
        assert_eq!(updated.reference_count, 1);
    }

    #[test]
    fn test_memory_repository_cleanup_deleted() {
        let mut repo = MockMemoryRepository::new();
        let mut memory1 = Memory::new(
            "tech".to_string(),
            "Memory 1".to_string(),
            "Content 1".to_string(),
        );
        let memory2 = Memory::new(
            "tech".to_string(),
            "Memory 2".to_string(),
            "Content 2".to_string(),
        );

        memory1.deleted = true;
        repo.save(&memory1).unwrap();
        repo.save(&memory2).unwrap();

        let deleted_count = repo.cleanup_deleted().unwrap();
        assert_eq!(deleted_count, 1);

        let remaining = repo.find_all().unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].title, "Memory 2");
    }

    #[test]
    fn test_memory_repository_maintenance_operations() {
        let mut repo = MockMemoryRepository::new();

        let rebuild_result = repo.rebuild_fts_index();
        assert!(rebuild_result.is_ok());

        let vacuum_result = repo.vacuum();
        assert!(vacuum_result.is_ok());
    }

    #[test]
    fn test_search_fts_sorting_by_confidence() {
        let mut repo = MockMemoryRepository::new();
        let high_confidence = Memory::new(
            "tech".to_string(),
            "High confidence".to_string(),
            "Content with keyword".to_string(),
        )
        .with_confidence(Confidence::new(0.9).unwrap());

        let low_confidence = Memory::new(
            "tech".to_string(),
            "Low confidence".to_string(),
            "Content with keyword".to_string(),
        )
        .with_confidence(Confidence::new(0.3).unwrap());

        repo.save(&low_confidence).unwrap();
        repo.save(&high_confidence).unwrap();

        let results = repo.search_fts("keyword", 10).unwrap();
        assert_eq!(results.len(), 2);
        // High confidence should come first
        assert_eq!(results[0].title, "High confidence");
        assert_eq!(results[1].title, "Low confidence");
    }

    #[test]
    fn test_search_fts_excludes_deleted() {
        let mut repo = MockMemoryRepository::new();
        let active_memory = Memory::new(
            "tech".to_string(),
            "Active memory".to_string(),
            "This contains keyword".to_string(),
        );
        let mut deleted_memory = Memory::new(
            "tech".to_string(),
            "Deleted memory".to_string(),
            "This also contains keyword".to_string(),
        );
        deleted_memory.deleted = true;

        repo.save(&active_memory).unwrap();
        repo.save(&deleted_memory).unwrap();

        let results = repo.search_fts("keyword", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Active memory");
    }

    #[test]
    fn test_search_fts_tags() {
        let mut repo = MockMemoryRepository::new();
        let memory_with_tags = Memory::new(
            "tech".to_string(),
            "Tagged memory".to_string(),
            "Some content".to_string(),
        )
        .with_tags(vec!["rust".to_string(), "programming".to_string()]);

        repo.save(&memory_with_tags).unwrap();

        let results = repo.search_fts("rust", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Tagged memory");
    }

    #[test]
    fn test_increment_reference_count_updates_last_accessed() {
        let mut repo = MockMemoryRepository::new();
        let memory = Memory::new(
            "tech".to_string(),
            "Test Memory".to_string(),
            "Test content".to_string(),
        );
        let memory_id = memory.id;

        repo.save(&memory).unwrap();
        assert!(
            repo.find_by_id(&memory_id)
                .unwrap()
                .unwrap()
                .last_accessed
                .is_none()
        );

        repo.increment_reference_count(&memory_id).unwrap();

        let updated = repo.find_by_id(&memory_id).unwrap().unwrap();
        assert_eq!(updated.reference_count, 1);
        assert!(updated.last_accessed.is_some());
    }

    #[test]
    fn test_save_failure_error_handling() {
        let mut repo = MockMemoryRepository::new();
        repo.set_next_save_to_fail();

        let memory = Memory::new(
            "tech".to_string(),
            "Test Memory".to_string(),
            "Test content".to_string(),
        );

        let result = repo.save(&memory);
        assert!(result.is_err());
        assert_eq!(repo.memory_count(), 0);

        // Next save should succeed
        let result2 = repo.save(&memory);
        assert!(result2.is_ok());
        assert_eq!(repo.memory_count(), 1);
    }

    #[test]
    fn test_find_all_excludes_deleted() {
        let mut repo = MockMemoryRepository::new();
        let active_memory = Memory::new(
            "tech".to_string(),
            "Active".to_string(),
            "Active content".to_string(),
        );
        let mut deleted_memory = Memory::new(
            "tech".to_string(),
            "Deleted".to_string(),
            "Deleted content".to_string(),
        );
        deleted_memory.deleted = true;

        repo.save(&active_memory).unwrap();
        repo.save(&deleted_memory).unwrap();

        let all_memories = repo.find_all().unwrap();
        assert_eq!(all_memories.len(), 1);
        assert_eq!(all_memories[0].title, "Active");
    }

    #[test]
    fn test_find_by_type_excludes_deleted() {
        let mut repo = MockMemoryRepository::new();
        let active_memory = Memory::new(
            "tech".to_string(),
            "Active Tech".to_string(),
            "Active content".to_string(),
        );
        let mut deleted_memory = Memory::new(
            "tech".to_string(),
            "Deleted Tech".to_string(),
            "Deleted content".to_string(),
        );
        deleted_memory.deleted = true;

        repo.save(&active_memory).unwrap();
        repo.save(&deleted_memory).unwrap();

        let tech_memories = repo.find_by_type("tech").unwrap();
        assert_eq!(tech_memories.len(), 1);
        assert_eq!(tech_memories[0].title, "Active Tech");
    }

    #[test]
    fn test_search_fts_limit_behavior() {
        let mut repo = MockMemoryRepository::new();
        for i in 1..=5 {
            let memory = Memory::new(
                "tech".to_string(),
                format!("Title {}", i),
                "Common keyword content".to_string(),
            );
            repo.save(&memory).unwrap();
        }

        let results = repo.search_fts("keyword", 3).unwrap();
        assert_eq!(results.len(), 3);

        let results = repo.search_fts("keyword", 10).unwrap();
        assert_eq!(results.len(), 5);
    }
}
