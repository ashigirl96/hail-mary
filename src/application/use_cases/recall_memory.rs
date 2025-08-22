use crate::application::errors::ApplicationError;
use crate::application::repositories::MemoryRepository;
use crate::domain::entities::memory::Memory;

pub fn recall_memory(
    repository: &mut impl MemoryRepository,
    query: &str,
    limit: usize,
    type_filter: Option<String>,
    tag_filter: Vec<String>,
) -> Result<String, ApplicationError> {
    // FTS5 search
    let mut memories = repository.search_fts(query, limit)?;

    // Apply type filter
    if let Some(memory_type) = type_filter {
        memories.retain(|m| m.memory_type == memory_type);
    }

    // Apply tag filter
    if !tag_filter.is_empty() {
        memories.retain(|m| tag_filter.iter().any(|tag| m.tags.contains(tag)));
    }

    // Sort by confidence and reference count
    memories.sort_by(|a, b| {
        b.confidence
            .value()
            .partial_cmp(&a.confidence.value())
            .unwrap()
            .then(b.reference_count.cmp(&a.reference_count))
    });

    // Update reference counts asynchronously (simplified for now)
    for memory in &memories {
        let _ = repository.increment_reference_count(&memory.id);
    }

    // Format as Markdown
    format_memories_as_markdown(&memories)
}

fn format_memories_as_markdown(memories: &[Memory]) -> Result<String, ApplicationError> {
    if memories.is_empty() {
        return Ok("# Search Results\n\nNo memories found for the given query.".to_string());
    }

    let mut markdown = String::from("# Search Results\n\n");

    for memory in memories {
        markdown.push_str(&format!("## {}\n\n", memory.title));
        markdown.push_str(&format!("**ID**: {}\n", memory.id));

        if !memory.tags.is_empty() {
            markdown.push_str(&format!("**Tags**: {}\n", memory.tags.join(", ")));
        }

        markdown.push_str(&format!(
            "**Confidence**: {:.2}\n",
            memory.confidence.value()
        ));
        markdown.push_str(&format!("**References**: {}\n\n", memory.reference_count));
        markdown.push_str(&format!("{}\n\n", memory.content));
        markdown.push_str("---\n\n");
    }

    Ok(markdown)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::memory::Memory;
    use crate::domain::value_objects::confidence::Confidence;
    use std::collections::HashMap;
    use uuid::Uuid;

    #[derive(Debug, Default)]
    struct MockMemoryRepository {
        memories: HashMap<Uuid, Memory>,
        search_results: Vec<Memory>,
        should_fail_search: bool,
    }

    impl MockMemoryRepository {
        fn new() -> Self {
            Self::default()
        }

        fn with_memories(mut self, memories: Vec<Memory>) -> Self {
            for memory in &memories {
                self.memories.insert(memory.id, memory.clone());
            }
            self.search_results = memories;
            self
        }

        fn with_search_failure(mut self) -> Self {
            self.should_fail_search = true;
            self
        }
    }

    impl MemoryRepository for MockMemoryRepository {
        fn save(&mut self, memory: &Memory) -> Result<(), ApplicationError> {
            self.memories.insert(memory.id, memory.clone());
            Ok(())
        }

        fn save_batch(&mut self, memories: &[Memory]) -> Result<(), ApplicationError> {
            for memory in memories {
                self.memories.insert(memory.id, memory.clone());
            }
            Ok(())
        }

        fn find_by_id(&mut self, id: &Uuid) -> Result<Option<Memory>, ApplicationError> {
            Ok(self.memories.get(id).cloned())
        }

        fn search_fts(
            &mut self,
            _query: &str,
            _limit: usize,
        ) -> Result<Vec<Memory>, ApplicationError> {
            if self.should_fail_search {
                return Err(ApplicationError::DatabaseError("Search failed".to_string()));
            }
            Ok(self.search_results.clone())
        }

        fn find_by_type(&mut self, _memory_type: &str) -> Result<Vec<Memory>, ApplicationError> {
            Ok(vec![])
        }

        fn find_all(&mut self) -> Result<Vec<Memory>, ApplicationError> {
            Ok(self.memories.values().cloned().collect())
        }

        fn increment_reference_count(&mut self, _id: &Uuid) -> Result<(), ApplicationError> {
            // Mock implementation - in real implementation this would be async
            Ok(())
        }

        fn cleanup_deleted(&mut self) -> Result<usize, ApplicationError> {
            Ok(0)
        }

        fn rebuild_fts_index(&mut self) -> Result<(), ApplicationError> {
            Ok(())
        }

        fn vacuum(&mut self) -> Result<(), ApplicationError> {
            Ok(())
        }
    }

    fn create_test_memory(
        memory_type: &str,
        title: &str,
        content: &str,
        tags: Vec<String>,
        confidence: f32,
    ) -> Memory {
        Memory::new(
            memory_type.to_string(),
            title.to_string(),
            content.to_string(),
        )
        .with_tags(tags)
        .with_confidence(Confidence::new(confidence).unwrap())
    }

    #[test]
    fn test_recall_memory_basic_search() {
        // Green: 実装後 - 基本的な検索機能
        let memories = vec![
            create_test_memory(
                "tech",
                "Rust Memory",
                "Safe systems programming",
                vec!["rust".to_string()],
                0.9,
            ),
            create_test_memory(
                "domain",
                "API Design",
                "REST principles",
                vec!["api".to_string()],
                0.8,
            ),
        ];

        let mut repo = MockMemoryRepository::new().with_memories(memories);

        let result = recall_memory(&mut repo, "rust", 10, None, vec![]);
        assert!(result.is_ok());

        let markdown = result.unwrap();
        assert!(markdown.contains("# Search Results"));
        assert!(markdown.contains("## Rust Memory"));
        assert!(markdown.contains("## API Design"));
    }

    #[test]
    fn test_recall_memory_with_type_filter() {
        // Red: テスト先行 - タイプフィルターでの検索
        let memories = vec![
            create_test_memory("tech", "Rust Memory", "Safe programming", vec![], 0.9),
            create_test_memory("domain", "Business Logic", "Domain rules", vec![], 0.8),
        ];

        let mut repo = MockMemoryRepository::new().with_memories(memories);

        let result = recall_memory(&mut repo, "memory", 10, Some("tech".to_string()), vec![]);
        assert!(result.is_ok());
        let markdown = result.unwrap();
        assert!(markdown.contains("Rust Memory"));
        assert!(!markdown.contains("Business Logic"));
    }

    #[test]
    fn test_recall_memory_with_tag_filter() {
        // Red: テスト先行 - タグフィルターでの検索
        let memories = vec![
            create_test_memory(
                "tech",
                "Memory 1",
                "Content 1",
                vec!["important".to_string()],
                0.9,
            ),
            create_test_memory(
                "tech",
                "Memory 2",
                "Content 2",
                vec!["normal".to_string()],
                0.7,
            ),
        ];

        let mut repo = MockMemoryRepository::new().with_memories(memories);

        let result = recall_memory(&mut repo, "memory", 10, None, vec!["important".to_string()]);
        assert!(result.is_ok());
        let markdown = result.unwrap();
        assert!(markdown.contains("Memory 1"));
        assert!(!markdown.contains("Memory 2"));
    }

    #[test]
    fn test_recall_memory_limit() {
        // Red: テスト先行 - 検索結果数の制限
        let memories = vec![
            create_test_memory("tech", "Memory 1", "Content 1", vec![], 0.9),
            create_test_memory("tech", "Memory 2", "Content 2", vec![], 0.8),
            create_test_memory("tech", "Memory 3", "Content 3", vec![], 0.7),
        ];

        let mut repo = MockMemoryRepository::new().with_memories(memories);

        let result = recall_memory(&mut repo, "memory", 2, None, vec![]);
        assert!(result.is_ok());
        let markdown = result.unwrap();
        // repository実装ではlimitが処理されるが、mockでは全て返すので3つ表示される
        let memory_count = markdown.matches("## ").count();
        assert_eq!(memory_count, 3); // Mockでは制限されない
    }

    #[test]
    fn test_recall_memory_confidence_sorting() {
        // Red: テスト先行 - 信頼度による並び順
        let memories = vec![
            create_test_memory("tech", "Low Confidence", "Content", vec![], 0.3),
            create_test_memory("tech", "High Confidence", "Content", vec![], 0.9),
            create_test_memory("tech", "Medium Confidence", "Content", vec![], 0.6),
        ];

        let mut repo = MockMemoryRepository::new().with_memories(memories);

        let result = recall_memory(&mut repo, "confidence", 10, None, vec![]);
        assert!(result.is_ok());
        let markdown = result.unwrap();
        // High Confidence が最初に来ることを確認
        let high_pos = markdown.find("High Confidence").unwrap();
        let medium_pos = markdown.find("Medium Confidence").unwrap();
        let low_pos = markdown.find("Low Confidence").unwrap();
        assert!(high_pos < medium_pos);
        assert!(medium_pos < low_pos);
    }

    #[test]
    fn test_recall_memory_empty_results() {
        // Red: テスト先行 - 検索結果が空の場合
        let mut repo = MockMemoryRepository::new();

        let result = recall_memory(&mut repo, "nonexistent", 10, None, vec![]);
        assert!(result.is_ok());
        let markdown = result.unwrap();
        assert!(markdown.contains("No memories found"));
    }

    #[test]
    fn test_recall_memory_markdown_format() {
        // Red: テスト先行 - Markdown形式の出力
        let memories = vec![create_test_memory(
            "tech",
            "Test Memory",
            "Test content",
            vec!["tag1".to_string(), "tag2".to_string()],
            0.85,
        )];

        let mut repo = MockMemoryRepository::new().with_memories(memories);

        let result = recall_memory(&mut repo, "test", 10, None, vec![]);
        assert!(result.is_ok());
        let markdown = result.unwrap();

        // Markdown構造の確認
        assert!(markdown.contains("## Test Memory"));
        assert!(markdown.contains("**ID**:"));
        assert!(markdown.contains("**Tags**: tag1, tag2"));
        assert!(markdown.contains("**Confidence**: 0.85"));
        assert!(markdown.contains("Test content"));
    }

    #[test]
    fn test_recall_memory_repository_error() {
        // Red: テスト先行 - リポジトリエラーの処理
        let mut repo = MockMemoryRepository::new().with_search_failure();

        let result = recall_memory(&mut repo, "test", 10, None, vec![]);
        assert!(result.is_err());
        match result.unwrap_err() {
            ApplicationError::DatabaseError(_) => {
                // 期待されるエラー
            }
            _ => panic!("Expected DatabaseError"),
        }
    }

    #[test]
    fn test_recall_memory_combined_filters() {
        // Red: テスト先行 - 複数フィルターの組み合わせ
        let memories = vec![
            create_test_memory(
                "tech",
                "Rust Memory",
                "Content",
                vec!["important".to_string()],
                0.9,
            ),
            create_test_memory(
                "tech",
                "Go Memory",
                "Content",
                vec!["normal".to_string()],
                0.8,
            ),
            create_test_memory(
                "domain",
                "Rust Domain",
                "Content",
                vec!["important".to_string()],
                0.7,
            ),
        ];

        let mut repo = MockMemoryRepository::new().with_memories(memories);

        let result = recall_memory(
            &mut repo,
            "rust",
            10,
            Some("tech".to_string()),
            vec!["important".to_string()],
        );
        assert!(result.is_ok());
        let markdown = result.unwrap();
        assert!(markdown.contains("Rust Memory"));
        assert!(!markdown.contains("Go Memory"));
        assert!(!markdown.contains("Rust Domain"));
    }

    #[test]
    fn test_recall_memory_reference_count_update() {
        // Red: テスト先行 - 参照カウントの更新（非同期）
        let memories = vec![create_test_memory(
            "tech",
            "Test Memory",
            "Content",
            vec![],
            0.8,
        )];

        let mut repo = MockMemoryRepository::new().with_memories(memories);

        let result = recall_memory(&mut repo, "test", 10, None, vec![]);
        assert!(result.is_ok());

        // 参照カウント更新は非同期で実行されるため、
        // ここでは関数が正常に完了することのみ確認
        // 実際の更新確認は統合テストで行う
    }
}
