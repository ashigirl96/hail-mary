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
    #[allow(dead_code)] // Reserved for future automatic embedding configuration
    pub fn set_auto_embeddings(&mut self, enabled: bool) {
        self.auto_generate_embeddings = enabled;
    }

    /// 日本語と英語の境界にスペースを挿入する
    /// FTS5が正しくトークン化できるようにするため
    fn normalize_content_for_fts(content: &str) -> String {
        let mut result = String::new();
        let mut prev_is_ascii = false;
        let mut prev_char: Option<char> = None;

        for ch in content.chars() {
            let curr_is_ascii = ch.is_ascii() && !ch.is_ascii_whitespace();
            let curr_is_japanese = matches!(ch, '\u{3040}'..='\u{309F}' | '\u{30A0}'..='\u{30FF}' | '\u{4E00}'..='\u{9FAF}');

            // 境界検出: ASCII→日本語 または 日本語→ASCII
            if let Some(prev) = prev_char {
                let prev_is_japanese = matches!(prev, '\u{3040}'..='\u{309F}' | '\u{30A0}'..='\u{30FF}' | '\u{4E00}'..='\u{9FAF}');

                // ASCII（ハイフン含む）と日本語の境界
                if (prev_is_ascii && curr_is_japanese) || (prev_is_japanese && curr_is_ascii) {
                    // スペースがまだない場合のみ挿入
                    if !prev.is_ascii_whitespace() && !ch.is_ascii_whitespace() {
                        result.push(' ');
                    }
                }
            }

            result.push(ch);
            prev_is_ascii = curr_is_ascii;
            prev_char = Some(ch);
        }

        result
    }

    /// FTS5クエリを部分マッチ対応に強化する
    fn enhance_query_for_partial_match(query: &str) -> String {
        // 既にワイルドカードがある場合はそのまま返す
        if query.contains('*') {
            return query.to_string();
        }

        // FTS5のブーリアン演算子をチェック（AND, OR, NOT）
        // これらが含まれる場合は複雑なクエリなのでそのまま返す
        if query.contains(" AND ") || query.contains(" OR ") || query.contains(" NOT ") {
            return query.to_string();
        }

        // 特殊文字が多数含まれる場合（@#$%など）は、クエリ全体をダブルクォートで囲む
        // FTS5で問題を起こす特殊文字を確認
        let problematic_chars = ['@', '#', '$', '%', '&', '^', '~', '`', '|', '\\'];
        if query.chars().any(|c| problematic_chars.contains(&c)) {
            // これらの文字が含まれる場合は、リテラル検索として扱う
            let escaped = query.replace('"', "\"\"");
            return format!("\"{}\"", escaped);
        }

        // ハイフンを含む場合は特別に扱う（FTS5はハイフンを単語区切りとして扱うため）
        // 例: "hail-mary", "semi-colon", など
        let has_hyphen = query.contains('-');
        let has_non_ascii = !query.is_ascii();

        if has_hyphen {
            // ハイフンを含むクエリは全体をダブルクォートで囲む
            let escaped = query.replace('"', "\"\"");
            return format!("\"{}\"", escaped);
        }

        // 日本語（またはその他の非ASCII文字）とハイフンが混在する場合も同様
        if has_non_ascii && has_hyphen {
            // 全体をダブルクォートで囲んでリテラル検索として扱う
            let escaped = query.replace('"', "\"\"");
            return format!("\"{}\"", escaped);
        }

        // :: を含む場合（名前空間やモジュール参照）は特別に扱う
        if query.contains("::") {
            // :: を一時的に置換して処理
            let processed = query.replace("::", "_COLON_COLON_");
            let words: Vec<String> = processed
                .split_whitespace()
                .map(|word| {
                    let restored = word.replace("_COLON_COLON_", "::");
                    // ダブルクォートで囲んで特殊文字をエスケープ
                    format!("\"{}\"", restored)
                })
                .collect();
            return words.join(" ");
        }

        // FTS5用の特殊文字エスケープ
        // FTS5では特殊文字をダブルクォートで囲む必要がある
        let needs_escaping = query.contains('\'') || query.contains('"') || query.contains(';');

        if needs_escaping {
            // 特殊文字を含む場合は、ダブルクォートで囲んでエスケープ
            let escaped = query.replace('"', "\"\"");
            return format!("\"{}\"", escaped);
        }

        // 日本語（またはその他の非ASCII文字）が含まれる場合
        if has_non_ascii {
            // 日本語の場合は各単語をダブルクォートで囲む
            // スペースで分割して各部分を処理
            return query
                .split_whitespace()
                .map(|word| format!("\"{}\"", word))
                .collect::<Vec<_>>()
                .join(" ");
        }

        // 各単語にワイルドカードを追加（プレフィックス検索）
        query
            .split_whitespace()
            .map(|word| {
                // 特殊文字をさらにエスケープ
                let safe_word = word
                    .replace('(', "\\(")
                    .replace(')', "\\)")
                    .replace('[', "\\[")
                    .replace(']', "\\]");
                format!("{}*", safe_word)
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// メモリの埋め込みを生成・保存
    async fn generate_and_store_embedding(&mut self, memory: &Memory) -> Result<()> {
        if let Some(service) = &self.embedding_service {
            let text = format!("{} {}", memory.title, memory.content);
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
        // コンテンツとタイトルを正規化（日本語と英語の境界にスペースを挿入）
        let normalized_content = Self::normalize_content_for_fts(&params.content);
        let normalized_title = Self::normalize_content_for_fts(&params.title);

        // ビジネスロジック: 重複チェック
        if let Some(mut existing) = self
            .repository
            .find_by_title(&normalized_title, &params.memory_type)?
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
            // source field removed

            // 内容も更新（正規化済みのコンテンツを使用）
            existing.content = normalized_content;
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

        // 新規作成（正規化済みのタイトルとコンテンツを使用）
        let mut memory = Memory::new(params.memory_type, normalized_title, normalized_content);

        if let Some(tags) = params.tags {
            memory.tags = tags;
        }
        if let Some(examples) = params.examples {
            memory.examples = examples;
        }
        // source field removed

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
        use tracing::info;

        let limit = params.limit.unwrap_or(10);
        info!(
            "Service recall: query='{}', memory_type={:?}, tags={:?}, limit={}, invalid_type={}",
            params.query, params.memory_type, params.tags, limit, params.invalid_type
        );

        // If an invalid type was provided, return empty results immediately
        if params.invalid_type {
            info!("Invalid type provided - returning empty results");
            return Ok(RecallResponse {
                memories: Vec::new(),
                total_count: 0,
            });
        }

        // 検索戦略の選択
        let mut memories = if params.query.is_empty() {
            info!("Empty query - browsing by type");
            // クエリがない場合はタイプでブラウズ
            if let Some(memory_type) = params.memory_type {
                let results = self.repository.browse_by_type(&memory_type, limit)?;
                info!(
                    "Browse by type {} returned {} memories",
                    memory_type,
                    results.len()
                );
                results
            } else {
                // 全タイプをブラウズ
                let results = self.repository.browse_all(limit)?;
                info!("Browse all types returned {} memories", results.len());
                results
            }
        } else {
            info!("Non-empty query - using FTS search");
            // FTS5検索のクエリ強化を実行
            let enhanced_query = Self::enhance_query_for_partial_match(&params.query);
            info!("Enhanced query: '{}' -> '{}'", params.query, enhanced_query);

            // Debug: log if the query contains non-ASCII characters
            if !params.query.is_ascii() {
                info!("Query contains non-ASCII characters (likely Japanese/Unicode)");
            }

            // FTS5検索を実行
            if let Some(memory_type) = params.memory_type {
                info!(
                    "Calling search_with_type with query='{}', type={}, limit={}",
                    enhanced_query, memory_type, limit
                );
                // タイプ指定がある場合はSQL側でフィルタリング
                let search_results =
                    self.repository
                        .search_with_type(&enhanced_query, &memory_type, limit)?;
                info!(
                    "search_with_type returned {} memories",
                    search_results.len()
                );
                search_results
            } else {
                info!(
                    "Calling search with query='{}', limit={}",
                    enhanced_query, limit
                );
                // タイプ指定がない場合は通常の検索
                let search_results = self.repository.search(&enhanced_query, limit)?;
                info!("search returned {} memories", search_results.len());
                search_results
            }
        };

        // タグフィルタを適用（空の配列の場合は適用しない）
        // 空クエリでもFTS検索でも同じように適用
        if let Some(tags) = params.tags.clone() {
            if !tags.is_empty() {
                info!("Applying tag filter: {:?}", tags);
                let before_count = memories.len();
                memories.retain(|m| tags.iter().all(|tag| m.tags.contains(tag)));
                info!(
                    "Tag filter reduced {} -> {} memories",
                    before_count,
                    memories.len()
                );
            } else {
                info!("Empty tag array provided - skipping tag filter");
            }
        }

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
        info!("Final recall result: {} memories", total_count);

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
    pub async fn delete_memory(&mut self, id: &str) -> Result<bool> {
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
            title: "Test Topic".to_string(),
            content: "Test Content".to_string(),
            tags: Some(vec!["test".to_string()]),
            examples: None,
            // source field removed
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
            title: "Test Topic".to_string(),
            content: "Test Content 1".to_string(),
            tags: None,
            examples: None,
            // source field removed
        };

        let response1 = service.remember(params1).await.unwrap();
        assert_eq!(response1.action, "created");

        let params2 = RememberParams {
            memory_type: MemoryType::Tech,
            title: "Test Topic".to_string(),
            content: "Test Content 2".to_string(),
            tags: None,
            examples: None,
            // source field removed
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
            title: "Rust async".to_string(),
            content: "Rust async/await programming".to_string(),
            tags: Some(vec!["rust".to_string(), "async".to_string()]),
            examples: None,
            // source field removed
        };
        service.remember(params1).await.unwrap();

        let params2 = RememberParams {
            memory_type: MemoryType::Tech,
            title: "Python decorators".to_string(),
            content: "Python decorator patterns".to_string(),
            tags: Some(vec!["python".to_string()]),
            examples: None,
            // source field removed
        };
        service.remember(params2).await.unwrap();

        // 検索
        let recall_params = RecallParams {
            query: "rust".to_string(),
            memory_type: None,
            tags: None,
            limit: Some(10),
            invalid_type: false,
        };

        let response = service.recall(recall_params).await.unwrap();
        assert_eq!(response.total_count, 1);
        assert_eq!(response.memories[0].title, "Rust async");
    }

    #[tokio::test]
    async fn test_recall_by_type() {
        let repo = SqliteMemoryRepository::new_in_memory().unwrap();
        let mut service = MemoryService::new(repo);

        // Tech タイプを追加
        let params1 = RememberParams {
            memory_type: MemoryType::Tech,
            title: "Tech Topic".to_string(),
            content: "Tech Content".to_string(),
            tags: None,
            examples: None,
            // source field removed
        };
        service.remember(params1).await.unwrap();

        // Domain タイプを追加
        let params2 = RememberParams {
            memory_type: MemoryType::Domain,
            title: "Domain Topic".to_string(),
            content: "Domain Content".to_string(),
            tags: None,
            examples: None,
            // source field removed
        };
        service.remember(params2).await.unwrap();

        // Tech タイプで検索
        let recall_params = RecallParams {
            query: "".to_string(), // 空のクエリでブラウズ
            memory_type: Some(MemoryType::Tech),
            tags: None,
            limit: Some(10),
            invalid_type: false,
        };

        let response = service.recall(recall_params).await.unwrap();
        assert_eq!(response.total_count, 1);
        assert_eq!(response.memories[0].title, "Tech Topic");
    }
}
