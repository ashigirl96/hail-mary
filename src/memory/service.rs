use crate::memory::{
    embeddings::EmbeddingService,
    models::{Memory, MemoryType, RecallParams, RecallResponse, RememberParams, RememberResponse},
    repository::MemoryRepository,
};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

/// メモリサービス（ビジネスロジック層）
pub struct MemoryService<R: MemoryRepository> {
    repository: R,
    embedding_service: Option<Arc<RwLock<EmbeddingService>>>,
    auto_generate_embeddings: bool,
}

impl<R: MemoryRepository> MemoryService<R> {
    /// 新しいサービスを作成
    pub fn new(repository: R) -> Self {
        Self {
            repository,
            embedding_service: None,
            auto_generate_embeddings: false,
        }
    }

    /// 新しいサービスを作成（埋め込み生成付き）
    pub fn with_embeddings(repository: R) -> Result<Self> {
        let embedding_service = EmbeddingService::new()?;
        Ok(Self {
            repository,
            embedding_service: Some(Arc::new(RwLock::new(embedding_service))),
            auto_generate_embeddings: true,
        })
    }

    /// 埋め込み自動生成を有効/無効にする
    pub fn set_auto_embeddings(&mut self, enabled: bool) {
        self.auto_generate_embeddings = enabled;
    }

    /// メモリの埋め込みを生成・保存
    async fn generate_and_store_embedding(&mut self, memory: &Memory) -> Result<()> {
        if let Some(service) = &self.embedding_service {
            let text = format!("{} {}", memory.topic, memory.content);
            let service = service.read().await;
            let embedding = service.embed_text(&text).await?;
            let model_name = service.model_name();
            self.repository
                .store_embedding(&memory.id, &embedding, model_name)?;
        }
        Ok(())
    }

    /// 記憶を保存
    pub async fn remember(&mut self, params: RememberParams) -> Result<RememberResponse> {
        // ビジネスロジック: 重複チェック
        if let Some(mut existing) = self
            .repository
            .find_by_topic(&params.topic, &params.memory_type)?
        {
            // 既存の記憶を更新
            self.repository.update_reference_count(&existing.id)?;

            // タグとコンテンツも更新
            if let Some(tags) = params.tags {
                existing.tags = tags;
            }
            if let Some(examples) = params.examples {
                existing.examples = examples;
            }
            if let Some(source) = params.source {
                existing.source = Some(source);
            }

            // 内容も更新
            existing.content = params.content;
            existing.last_accessed = Some(chrono::Utc::now().timestamp());

            self.repository.update(&existing)?;

            // Update embedding if content changed and embeddings are enabled
            if self.auto_generate_embeddings {
                self.generate_and_store_embedding(&existing).await?;
            }

            return Ok(RememberResponse {
                memory_id: existing.id,
                action: "updated".to_string(),
                similar_count: None,
            });
        }

        // 新規作成
        let mut memory = Memory::new(params.memory_type, params.topic, params.content);

        if let Some(tags) = params.tags {
            memory.tags = tags;
        }
        if let Some(examples) = params.examples {
            memory.examples = examples;
        }
        if let Some(source) = params.source {
            memory.source = Some(source);
        }

        self.repository.save(&memory)?;

        // Generate embedding if enabled
        if self.auto_generate_embeddings {
            self.generate_and_store_embedding(&memory).await?;
        }

        Ok(RememberResponse {
            memory_id: memory.id,
            action: "created".to_string(),
            similar_count: None,
        })
    }

    /// セマンティック検索を実行
    pub async fn recall_semantic(
        &mut self,
        query: &str,
        limit: usize,
        min_similarity: f32,
    ) -> Result<Vec<(Memory, f32)>> {
        if let Some(service) = &self.embedding_service {
            let service = service.read().await;
            let query_embedding = service.embed_text(query).await?;
            self.repository
                .search_similar(&query_embedding, limit, min_similarity)
        } else {
            Ok(Vec::new())
        }
    }

    /// 関連メモリを検索
    pub async fn find_related(
        &mut self,
        memory_id: &str,
        limit: usize,
        min_similarity: f32,
    ) -> Result<Vec<(Memory, f32)>> {
        if let Some(embedding) = self.repository.get_embedding(memory_id)? {
            self.repository
                .search_similar(&embedding, limit + 1, min_similarity)
                .map(|mut results| {
                    // Remove the query memory itself
                    results.retain(|(m, _)| m.id != memory_id);
                    results.truncate(limit);
                    results
                })
        } else {
            Ok(Vec::new())
        }
    }

    /// 記憶を検索
    pub async fn recall(&mut self, params: RecallParams) -> Result<RecallResponse> {
        let limit = params.limit.unwrap_or(10);

        // 検索戦略の選択
        let mut memories = if params.query.is_empty() {
            // クエリがない場合はタイプでブラウズ
            if let Some(memory_type) = params.memory_type {
                self.repository.browse_by_type(&memory_type, limit)?
            } else {
                // デフォルトはTechタイプをブラウズ
                self.repository.browse_by_type(&MemoryType::Tech, limit)?
            }
        } else {
            // FTS5検索を実行
            let mut results = self.repository.search(&params.query, limit)?;

            // タイプフィルタを適用
            if let Some(memory_type) = params.memory_type {
                results.retain(|m| m.memory_type == memory_type);
            }

            // タグフィルタを適用
            if let Some(tags) = params.tags {
                results.retain(|m| tags.iter().any(|tag| m.tags.contains(tag)));
            }

            results
        };

        // 最終アクセス時刻を更新
        for memory in &memories {
            self.repository.update_last_accessed(&memory.id)?;
        }

        // ビジネスロジック: 信頼度でソート
        memories.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap()
                .then(b.reference_count.cmp(&a.reference_count))
        });

        let total_count = memories.len();

        Ok(RecallResponse {
            memories,
            total_count,
        })
    }

    /// ID で記憶を取得
    pub async fn get_memory(&self, id: &str) -> Result<Option<Memory>> {
        self.repository.find_by_id(id)
    }

    /// 記憶を削除（論理削除）
    pub async fn delete_memory(&mut self, id: &str) -> Result<()> {
        self.repository.soft_delete(id)
    }

    /// タイプごとに全ての記憶を取得（ドキュメント生成用）
    pub async fn get_all_by_type(&self, memory_type: &MemoryType) -> Result<Vec<Memory>> {
        // 制限なしで全件取得（10000件を上限とする）
        self.repository.browse_by_type(memory_type, 10000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::repository::SqliteMemoryRepository;

    #[tokio::test]
    async fn test_remember_new() {
        let repo = SqliteMemoryRepository::new_in_memory().unwrap();
        let mut service = MemoryService::new(repo);

        let params = RememberParams {
            memory_type: MemoryType::Tech,
            topic: "Test Topic".to_string(),
            content: "Test Content".to_string(),
            tags: Some(vec!["test".to_string()]),
            examples: None,
            source: None,
        };

        let response = service.remember(params).await.unwrap();
        assert_eq!(response.action, "created");
        assert!(!response.memory_id.is_empty());
    }

    #[tokio::test]
    async fn test_remember_duplicate() {
        let repo = SqliteMemoryRepository::new_in_memory().unwrap();
        let mut service = MemoryService::new(repo);

        let params1 = RememberParams {
            memory_type: MemoryType::Tech,
            topic: "Test Topic".to_string(),
            content: "Test Content 1".to_string(),
            tags: None,
            examples: None,
            source: None,
        };

        let response1 = service.remember(params1).await.unwrap();
        assert_eq!(response1.action, "created");

        let params2 = RememberParams {
            memory_type: MemoryType::Tech,
            topic: "Test Topic".to_string(),
            content: "Test Content 2".to_string(),
            tags: None,
            examples: None,
            source: None,
        };

        let response2 = service.remember(params2).await.unwrap();
        assert_eq!(response2.action, "updated");
        assert_eq!(response1.memory_id, response2.memory_id);
    }

    #[tokio::test]
    async fn test_recall() {
        let repo = SqliteMemoryRepository::new_in_memory().unwrap();
        let mut service = MemoryService::new(repo);

        // いくつか記憶を追加
        let params1 = RememberParams {
            memory_type: MemoryType::Tech,
            topic: "Rust async".to_string(),
            content: "Rust async/await programming".to_string(),
            tags: Some(vec!["rust".to_string(), "async".to_string()]),
            examples: None,
            source: None,
        };
        service.remember(params1).await.unwrap();

        let params2 = RememberParams {
            memory_type: MemoryType::Tech,
            topic: "Python decorators".to_string(),
            content: "Python decorator patterns".to_string(),
            tags: Some(vec!["python".to_string()]),
            examples: None,
            source: None,
        };
        service.remember(params2).await.unwrap();

        // 検索
        let recall_params = RecallParams {
            query: "rust".to_string(),
            memory_type: None,
            tags: None,
            limit: Some(10),
        };

        let response = service.recall(recall_params).await.unwrap();
        assert_eq!(response.total_count, 1);
        assert_eq!(response.memories[0].topic, "Rust async");
    }

    #[tokio::test]
    async fn test_recall_by_type() {
        let repo = SqliteMemoryRepository::new_in_memory().unwrap();
        let mut service = MemoryService::new(repo);

        // Tech タイプを追加
        let params1 = RememberParams {
            memory_type: MemoryType::Tech,
            topic: "Tech Topic".to_string(),
            content: "Tech Content".to_string(),
            tags: None,
            examples: None,
            source: None,
        };
        service.remember(params1).await.unwrap();

        // Domain タイプを追加
        let params2 = RememberParams {
            memory_type: MemoryType::Domain,
            topic: "Domain Topic".to_string(),
            content: "Domain Content".to_string(),
            tags: None,
            examples: None,
            source: None,
        };
        service.remember(params2).await.unwrap();

        // Tech タイプで検索
        let recall_params = RecallParams {
            query: "".to_string(), // 空のクエリでブラウズ
            memory_type: Some(MemoryType::Tech),
            tags: None,
            limit: Some(10),
        };

        let response = service.recall(recall_params).await.unwrap();
        assert_eq!(response.total_count, 1);
        assert_eq!(response.memories[0].topic, "Tech Topic");
    }
}
