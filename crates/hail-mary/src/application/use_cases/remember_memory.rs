use crate::application::errors::ApplicationError;
use crate::application::repositories::MemoryRepository;
use crate::domain::entities::memory::Memory;
use crate::domain::entities::project::ProjectConfig;
use crate::domain::value_objects::confidence::Confidence;

pub struct RememberRequest {
    pub memory_type: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub confidence: Option<f32>,
}

pub fn remember_memory(
    repository: &mut (impl MemoryRepository + ?Sized),
    config: &ProjectConfig,
    requests: Vec<RememberRequest>,
) -> Result<Vec<Memory>, ApplicationError> {
    let mut memories = Vec::new();

    for request in requests {
        // Validate memory type against configuration
        if !config.validate_memory_type(&request.memory_type) {
            return Err(ApplicationError::InvalidMemoryType(request.memory_type));
        }

        // Create memory entity
        let confidence = match request.confidence {
            Some(c) => Confidence::new(c)?,
            None => Confidence::default(),
        };

        let memory = Memory::new(request.memory_type, request.title, request.content)
            .with_tags(request.tags)
            .with_confidence(confidence);

        memories.push(memory);
    }

    // Batch save to repository
    repository.save_batch(&memories)?;

    Ok(memories)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_helpers::MockMemoryRepository;

    fn create_test_config() -> ProjectConfig {
        ProjectConfig::default_for_new_project()
    }

    #[test]
    fn test_remember_memory_single_valid_request() {
        // Green: 実装後 - 単一の有効なリクエスト
        let mut repo = MockMemoryRepository::new();
        let config = create_test_config();

        let request = RememberRequest {
            memory_type: "tech".to_string(),
            title: "Test Memory".to_string(),
            content: "Test content".to_string(),
            tags: vec!["test".to_string()],
            confidence: Some(0.8),
        };

        let result = remember_memory(&mut repo, &config, vec![request]);
        assert!(result.is_ok());

        let memories = result.unwrap();
        assert_eq!(memories.len(), 1);

        let memory = &memories[0];
        assert_eq!(memory.memory_type, "tech");
        assert_eq!(memory.title, "Test Memory");
        assert_eq!(memory.content, "Test content");
        assert_eq!(memory.tags, vec!["test"]);
        assert_eq!(memory.confidence.value(), 0.8);
    }

    #[test]
    fn test_remember_memory_multiple_requests() {
        // Red: テスト先行 - 複数のリクエストのバッチ処理
        let mut repo = MockMemoryRepository::new();
        let config = create_test_config();

        let requests = vec![
            RememberRequest {
                memory_type: "tech".to_string(),
                title: "Memory 1".to_string(),
                content: "Content 1".to_string(),
                tags: vec!["tag1".to_string()],
                confidence: Some(0.9),
            },
            RememberRequest {
                memory_type: "domain".to_string(),
                title: "Memory 2".to_string(),
                content: "Content 2".to_string(),
                tags: vec!["tag2".to_string()],
                confidence: Some(0.7),
            },
        ];

        let result = remember_memory(&mut repo, &config, requests);
        assert!(result.is_ok());
        let memories = result.unwrap();
        assert_eq!(memories.len(), 2);
    }

    #[test]
    fn test_remember_memory_validates_memory_type() {
        // Red: テスト先行 - メモリタイプの検証
        let mut repo = MockMemoryRepository::new();
        let config = create_test_config();

        let request = RememberRequest {
            memory_type: "invalid-type".to_string(),
            title: "Test Memory".to_string(),
            content: "Test content".to_string(),
            tags: vec![],
            confidence: None,
        };

        let result = remember_memory(&mut repo, &config, vec![request]);
        assert!(result.is_err());
        match result.unwrap_err() {
            ApplicationError::InvalidMemoryType(memory_type) => {
                assert_eq!(memory_type, "invalid-type");
            }
            _ => panic!("Expected InvalidMemoryType error"),
        }
    }

    #[test]
    fn test_remember_memory_confidence_validation() {
        // Red: テスト先行 - 信頼度の検証
        let mut repo = MockMemoryRepository::new();
        let config = create_test_config();

        let request = RememberRequest {
            memory_type: "tech".to_string(),
            title: "Test Memory".to_string(),
            content: "Test content".to_string(),
            tags: vec![],
            confidence: Some(1.5), // 無効な信頼度
        };

        let result = remember_memory(&mut repo, &config, vec![request]);
        assert!(result.is_err());
        // DomainErrorからApplicationErrorに変換されることを確認
        match result.unwrap_err() {
            ApplicationError::DomainError(_) => {
                // 期待されるエラー（信頼度検証失敗）
            }
            _ => panic!("Expected DomainError for invalid confidence"),
        }
    }

    #[test]
    fn test_remember_memory_default_confidence() {
        // Red: テスト先行 - デフォルト信頼度の使用
        let mut repo = MockMemoryRepository::new();
        let config = create_test_config();

        let request = RememberRequest {
            memory_type: "tech".to_string(),
            title: "Test Memory".to_string(),
            content: "Test content".to_string(),
            tags: vec![],
            confidence: None, // デフォルト信頼度を使用
        };

        let result = remember_memory(&mut repo, &config, vec![request]);
        assert!(result.is_ok());
        let memories = result.unwrap();
        assert_eq!(memories[0].confidence.value(), 1.0); // デフォルトは1.0
    }

    #[test]
    fn test_remember_memory_repository_error() {
        // Red: テスト先行 - リポジトリエラーの処理
        let mut repo = MockMemoryRepository::new().with_batch_failure();
        let config = create_test_config();

        let request = RememberRequest {
            memory_type: "tech".to_string(),
            title: "Test Memory".to_string(),
            content: "Test content".to_string(),
            tags: vec![],
            confidence: Some(0.8),
        };

        let result = remember_memory(&mut repo, &config, vec![request]);
        assert!(result.is_err());
        match result.unwrap_err() {
            ApplicationError::DatabaseError(_) => {
                // 期待されるエラー
            }
            _ => panic!("Expected DatabaseError"),
        }
    }

    #[test]
    fn test_remember_memory_empty_requests() {
        // Red: テスト先行 - 空のリクエストリストの処理
        let mut repo = MockMemoryRepository::new();
        let config = create_test_config();

        let result = remember_memory(&mut repo, &config, vec![]);
        assert!(result.is_ok());
        let memories = result.unwrap();
        assert!(memories.is_empty());
    }

    #[test]
    fn test_remember_memory_preserves_request_data() {
        // Red: テスト先行 - リクエストデータの保持確認
        let mut repo = MockMemoryRepository::new();
        let config = create_test_config();

        let request = RememberRequest {
            memory_type: "project-tech".to_string(),
            title: "Important Memory".to_string(),
            content: "Detailed content here".to_string(),
            tags: vec!["important".to_string(), "urgent".to_string()],
            confidence: Some(0.95),
        };

        let result = remember_memory(&mut repo, &config, vec![request]);
        assert!(result.is_ok());
        let memories = result.unwrap();
        let memory = &memories[0];

        assert_eq!(memory.memory_type, "project-tech");
        assert_eq!(memory.title, "Important Memory");
        assert_eq!(memory.content, "Detailed content here");
        assert_eq!(memory.tags, vec!["important", "urgent"]);
        assert_eq!(memory.confidence.value(), 0.95);
    }

    #[test]
    fn test_remember_memory_generates_unique_ids() {
        // Red: テスト先行 - ユニークIDの生成確認
        let mut repo = MockMemoryRepository::new();
        let config = create_test_config();

        let requests = vec![
            RememberRequest {
                memory_type: "tech".to_string(),
                title: "Memory 1".to_string(),
                content: "Content 1".to_string(),
                tags: vec![],
                confidence: Some(0.8),
            },
            RememberRequest {
                memory_type: "tech".to_string(),
                title: "Memory 2".to_string(),
                content: "Content 2".to_string(),
                tags: vec![],
                confidence: Some(0.8),
            },
        ];

        let result = remember_memory(&mut repo, &config, requests);
        assert!(result.is_ok());
        let memories = result.unwrap();
        assert_ne!(memories[0].id, memories[1].id);
    }

    #[test]
    fn test_remember_memory_sets_timestamps() {
        // Red: テスト先行 - タイムスタンプの設定確認
        let mut repo = MockMemoryRepository::new();
        let config = create_test_config();

        let request = RememberRequest {
            memory_type: "tech".to_string(),
            title: "Test Memory".to_string(),
            content: "Test content".to_string(),
            tags: vec![],
            confidence: Some(0.8),
        };

        let result = remember_memory(&mut repo, &config, vec![request]);
        assert!(result.is_ok());
        let memories = result.unwrap();
        let memory = &memories[0];

        // created_atが設定されていることを確認
        use chrono::DateTime;
        let epoch = DateTime::from_timestamp(0, 0).unwrap();
        assert!(memory.created_at > epoch);
        assert_eq!(memory.last_accessed, None); // 新規作成時はNone
        assert_eq!(memory.reference_count, 0); // 新規作成時は0
        assert!(!memory.deleted); // 新規作成時はfalse
    }
}
