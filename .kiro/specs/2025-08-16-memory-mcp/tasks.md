# Memory MCP v3 - TDD実装タスクリスト

## 実装方針
- **TDD (Test-Driven Development)** による段階的実装
- **Red-Green-Refactor** サイクルの厳守
- 各サイクルごとにレビューを実施
- **Inside-Out** アプローチ（ドメイン層から外側へ）

---

## Phase 0: TDD環境準備 (1時間)

### 目的
テスト駆動開発の基盤を整備、プロジェクト構造の準備

### タスク
- [ ] プロジェクト構造の作成
  - [ ] `src/models/` ディレクトリ作成
  - [ ] `src/repositories/` ディレクトリ作成
  - [ ] `src/services/` ディレクトリ作成
  - [ ] `src/commands/memory/` ディレクトリ作成
  - [ ] `src/tests/common/` ディレクトリ作成
  - [ ] `migrations/` ディレクトリ作成
  - [ ] 各ディレクトリに `mod.rs` 作成

- [ ] Cargo.tomlに完全な依存関係追加
  ```toml
  # Core
  tokio = { version = "1", features = ["full", "test-util"] }
  anyhow = "1"
  thiserror = "1"
  
  # MCP
  rmcp = { version = "0.5.0", features = ["server", "macros", "transport-io"] }
  schemars = "1"
  serde = { version = "1", features = ["derive"] }
  serde_json = "1"
  
  # Database
  rusqlite = { version = "0.37", features = ["bundled", "json"] }
  refinery = { version = "0.8", features = ["rusqlite"] }
  
  # Utils
  uuid = { version = "1", features = ["v4"] }
  chrono = "0.4"
  toml = "0.8"
  tracing = "0.1"
  tracing-subscriber = "0.3"
  pulldown-cmark = "0.13"
  
  # Dev dependencies
  rstest = "0.23"
  pretty_assertions = "1"
  tempfile = "3"
  ```

- [ ] テストユーティリティモジュール作成 (`src/tests/common/mod.rs`)
  - [ ] テスト用のKiroConfig生成ヘルパー
  - [ ] 一時ディレクトリ管理ヘルパー
  - [ ] フィクスチャデータロード機能

---

## Phase 1: ドメインモデル (3-4時間)

### サイクル 1-1: Memoryモデル

#### テスト作成 (45分)
- [ ] `src/models/memory.rs` にテストモジュール追加
- [ ] `test_memory_new_creates_valid_instance()`
  - [ ] UUIDが生成されることを確認
  - [ ] デフォルト値の確認 (reference_count=0, confidence=1.0, deleted=false)
  - [ ] created_atが現在時刻であることを確認
  - [ ] last_accessedがNoneであることを確認
- [ ] `test_memory_with_tags_builder()`
  - [ ] タグが正しく設定されることを確認
- [ ] `test_memory_with_confidence_builder()`
  - [ ] 信頼度が正しく設定されることを確認（0.0-1.0の範囲）
- [ ] `test_memory_from_row()`
  - [ ] rusqlite::Rowからの変換テスト
  - [ ] タグのカンマ区切り文字列の分割
  - [ ] deleted=1がtrueに変換される

#### 実装 (45分)
- [ ] Memory構造体の定義（全フィールド含む）
  ```rust
  pub struct Memory {
      pub id: String,
      pub memory_type: MemoryType,
      pub title: String,
      pub tags: Vec<String>,
      pub content: String,
      pub reference_count: u32,
      pub confidence: f32,
      pub created_at: i64,
      pub last_accessed: Option<i64>,
      pub deleted: bool,
  }
  ```
- [ ] `Memory::new()` メソッド実装
- [ ] ビルダーメソッド実装 (`with_tags()`, `with_confidence()`)
- [ ] `Memory::from_row()` 実装（SQLite型変換）
- [ ] テストをグリーンにする

#### レビューポイント
- [ ] 命名規則の確認
- [ ] 不変性の確保
- [ ] SQLiteとの型変換の正確性
- [ ] エラーハンドリングの必要性

### サイクル 1-2: MemoryType

#### テスト作成 (20分)
- [ ] `test_memory_type_display()`
  - [ ] Tech → "tech"
  - [ ] ProjectTech → "project-tech"
  - [ ] Domain → "domain"
- [ ] `test_memory_type_from_str()`
  - [ ] "tech" → Tech
  - [ ] "project-tech" → ProjectTech
  - [ ] "invalid" → Error
- [ ] `test_memory_type_round_trip()`
  - [ ] Display → FromStr → Display の一貫性

#### 実装 (20分)
- [ ] MemoryType enum定義
- [ ] Display trait実装
- [ ] FromStr trait実装
- [ ] テストをグリーンにする

#### レビューポイント
- [ ] エラーメッセージの適切さ
- [ ] 将来の拡張性

### サイクル 1-3: Error定義

#### テスト作成 (15分)
- [ ] `test_memory_error_display()`
  - [ ] 各エラー型のメッセージ確認
- [ ] `test_memory_error_from_conversions()`
  - [ ] rusqlite::Error からの変換
  - [ ] std::io::Error からの変換

#### 実装 (15分)
- [ ] MemoryError enum定義 (thiserror使用)
- [ ] 各エラー型の実装
- [ ] Result型エイリアス定義

### サイクル 1-4: KiroConfig

#### テスト作成 (30分)
- [ ] `src/models/kiro.rs` にテストモジュール追加
- [ ] `test_kiro_config_load_from_file()`
  - [ ] config.tomlから設定読み込み
  - [ ] メモリタイプのパース
- [ ] `test_kiro_config_default()`
  - [ ] デフォルト設定の生成
- [ ] `test_kiro_config_validate_memory_type()`
  - [ ] 有効なメモリタイプの検証
  - [ ] 無効なメモリタイプの拒否
- [ ] `test_kiro_config_find_kiro_root()`
  - [ ] .kiroディレクトリの探索

#### 実装 (30分)
- [ ] KiroConfig構造体定義
  ```rust
  #[derive(Debug, Clone, Deserialize)]
  pub struct KiroConfig {
      pub memory: MemoryConfig,
  }
  
  #[derive(Debug, Clone, Deserialize)]
  pub struct MemoryConfig {
      pub types: Vec<String>,
      pub instructions: String,
      pub document: DocumentConfig,
      pub database: DatabaseConfig,
  }
  ```
- [ ] `KiroConfig::load()` 実装
- [ ] `KiroConfig::default()` 実装
- [ ] `validate_memory_type()` 実装
- [ ] `find_kiro_root()` 実装

#### レビューポイント
- [ ] 設定ファイルの探索ロジック
- [ ] デフォルト値の妥当性
- [ ] エラーハンドリング

---

## Phase 2: Repository層 (4-5時間)

### サイクル 2-1: Repository trait & InMemoryRepository

#### テスト作成 (45分)
- [ ] `src/repositories/memory.rs` にテストモジュール追加
- [ ] `test_in_memory_save_and_find()`
  - [ ] saveしたメモリがfind_by_idで取得できる
  - [ ] 存在しないIDはNone
  - [ ] 論理削除されたメモリは取得されない
- [ ] `test_in_memory_save_batch()`
  - [ ] 複数のメモリを一度に保存
  - [ ] すべて取得できることを確認
- [ ] `test_in_memory_search_fts()`
  - [ ] 簡易的な文字列検索の実装
  - [ ] 論理削除されたメモリは検索されない
- [ ] `test_in_memory_find_all()`
  - [ ] すべてのメモリを取得（論理削除除く）
- [ ] `test_in_memory_increment_reference_count()`
  - [ ] 参照カウントの増加
  - [ ] last_accessedの更新

#### 実装 (45分)
- [ ] MemoryRepository trait定義
  ```rust
  pub trait MemoryRepository: Send + Sync {
      fn save(&mut self, memory: &Memory) -> Result<()>;
      fn save_batch(&mut self, memories: &[Memory]) -> Result<()>;
      fn find_by_id(&self, id: &str) -> Result<Option<Memory>>;
      fn search_fts(&self, query: &str, limit: usize) -> Result<Vec<Memory>>;
      fn find_all(&self) -> Result<Vec<Memory>>;
      fn increment_reference_count(&mut self, id: &str) -> Result<()>;
  }
  ```
- [ ] InMemoryRepository実装
  - [ ] HashMap<String, Memory>でデータ保持
  - [ ] 論理削除フィルタリング
  - [ ] 簡易検索実装

#### レビューポイント
- [ ] trait設計の適切さ
- [ ] 論理削除の一貫した処理
- [ ] エラーハンドリング
- [ ] スレッドセーフティの考慮

### サイクル 2-2: SQLiteRepository準備

#### テスト作成 (45分)
- [ ] `test_sqlite_repository_new_creates_database()`
  - [ ] データベースファイルが作成される
  - [ ] 接続が確立される
  - [ ] WALモードが設定される
- [ ] `test_sqlite_repository_runs_migrations()`
  - [ ] マイグレーションが実行される
  - [ ] テーブルが作成される
  - [ ] FTS5インデックスが作成される
  - [ ] トリガーが作成される
- [ ] `test_sqlite_pragmas_set_correctly()`
  - [ ] journal_mode = WAL
  - [ ] foreign_keys = ON
  - [ ] synchronous = NORMAL

#### 実装 (45分)
- [ ] `migrations/V001__initial_schema.sql` 作成
- [ ] `migrations/V002__add_fts5_index.sql` 作成
- [ ] `migrations/V003__add_triggers.sql` 作成
- [ ] SqliteMemoryRepository構造体定義
- [ ] `new()` メソッド実装
  - [ ] Refinery統合
  - [ ] SQLiteプラグマ設定
  - [ ] WALモード有効化
- [ ] 基本的な接続管理

### サイクル 2-3: SQLiteRepository CRUD操作

#### テスト作成 (1時間)
- [ ] `test_sqlite_save_and_find()`
  - [ ] INSERTとSELECTの動作確認
  - [ ] 論理削除フィルタリング
- [ ] `test_sqlite_save_batch_with_transaction()`
  - [ ] トランザクション内でのバッチ保存
  - [ ] エラー時のロールバック
- [ ] `test_sqlite_fts_search()`
  - [ ] FTS5検索の動作確認
  - [ ] 日本語検索のテスト（tokenize = 'porter unicode61'）
  - [ ] 論理削除されたデータは検索されない
- [ ] `test_sqlite_find_all()`
  - [ ] 全メモリの取得
  - [ ] deleted=0のフィルタリング
- [ ] `test_sqlite_increment_reference_count()`
  - [ ] 参照カウントの更新確認
  - [ ] last_accessedの更新確認

#### 実装 (1時間)
- [ ] save実装（prepared statement使用）
- [ ] save_batch実装（トランザクション使用）
  ```rust
  fn save_batch(&mut self, memories: &[Memory]) -> Result<()> {
      let tx = self.conn.transaction()?;
      // バッチ処理
      tx.commit()?;
  }
  ```
- [ ] find_by_id実装（WHERE deleted = 0）
- [ ] search_fts実装（FTS5クエリ + 論理削除考慮）
- [ ] find_all実装
- [ ] increment_reference_count実装（UPDATE文）

#### レビューポイント
- [ ] SQLインジェクション対策（prepared statement使用）
- [ ] トランザクション管理の適切さ
- [ ] 論理削除の一貫した処理
- [ ] パフォーマンス考慮（インデックス活用）

---

## Phase 3: Service層 (4-5時間)

### サイクル 3-1: MemoryService基本機能

#### テスト作成 (45分)
- [ ] `test_memory_service_remember_single()`
  - [ ] 単一メモリの保存
  - [ ] 正しいIDが返される
  - [ ] デフォルト値の設定
- [ ] `test_memory_service_remember_batch()`
  - [ ] 複数メモリの保存
  - [ ] すべてのIDが返される
- [ ] `test_memory_service_validates_memory_type()`
  - [ ] KiroConfigで定義されたタイプのみ受け付ける
  - [ ] 無効なタイプはエラー

#### 実装 (45分)
- [ ] MemoryService<R: MemoryRepository>構造体
  ```rust
  pub struct MemoryService<R: MemoryRepository> {
      repository: R,
      config: KiroConfig,
  }
  ```
- [ ] `new(repository: R, config: KiroConfig)` コンストラクタ
- [ ] `remember_batch()` メソッド実装
  - [ ] メモリタイプのバリデーション
  - [ ] MemoryInput → Memory変換
- [ ] MemoryInput構造体定義

#### レビューポイント
- [ ] ジェネリクスの使い方
- [ ] 依存性注入の設計
- [ ] 設定との連携

### サイクル 3-2: recall機能

#### テスト作成 (1時間)
- [ ] `test_recall_with_type_filter()`
  - [ ] タイプでフィルタリング
  - [ ] 無効なタイプはエラー
- [ ] `test_recall_with_tag_filter()`
  - [ ] タグでフィルタリング
  - [ ] 複数タグのOR条件
- [ ] `test_recall_sorts_by_confidence()`
  - [ ] 信頼度でソート
  - [ ] 同じ信頼度なら参照回数でソート
- [ ] `test_recall_returns_markdown()`
  - [ ] Markdown形式で返却
  - [ ] タイトル、タグ、信頼度が含まれる
- [ ] `test_recall_updates_reference_count_async()`
  - [ ] 非同期で参照カウント更新
  - [ ] メインスレッドはブロックされない

#### 実装 (1時間)
- [ ] `recall()` メソッド実装
  ```rust
  pub async fn recall(
      &mut self,
      query: &str,
      limit: usize,
      type_filter: Option<MemoryType>,
      tag_filter: Vec<String>,
  ) -> Result<String>
  ```
- [ ] フィルタリングロジック
- [ ] ソートロジック（confidence → reference_count）
- [ ] 非同期参照カウント更新
  ```rust
  let ids = memories.iter().map(|m| m.id.clone()).collect();
  let repo = Arc::clone(&self.repository);
  tokio::spawn(async move {
      for id in ids {
          let _ = repo.lock().await.increment_reference_count(&id);
      }
  });
  ```
- [ ] `format_as_markdown()` 実装

#### レビューポイント
- [ ] ビジネスロジックの適切さ
- [ ] 非同期処理の安全性
- [ ] Arc<Mutex<>>の使用
- [ ] Markdown形式の妥当性

### サイクル 3-3: ドキュメント生成

#### テスト作成 (30分)
- [ ] `test_generate_documents_creates_files()`
  - [ ] タイプ別ファイル生成
  - [ ] 正しいパスに出力

#### 実装 (30分)
- [ ] `generate_documents()` メソッド実装
- [ ] ファイル出力処理

---

## Phase 4: Infrastructure統合 (2-3時間)

### サイクル 4-1: マイグレーション

#### テスト作成 (30分)
- [ ] `test_migration_creates_tables()`
  - [ ] memoriesテーブル作成
  - [ ] memories_ftsテーブル作成
- [ ] `test_migration_creates_indexes()`
  - [ ] インデックス作成確認
- [ ] `test_migration_creates_triggers()`
  - [ ] トリガー動作確認

#### 実装 (30分)
- [ ] `V001__initial_schema.sql` 完成
- [ ] `V002__add_fts5_index.sql` 作成
- [ ] `V003__add_triggers.sql` 作成

### サイクル 4-2: 統合テスト

#### テスト作成 (45分)
- [ ] `tests/integration/repository_test.rs`
  - [ ] 実際のSQLiteでの動作確認
  - [ ] FTS5日本語検索
  - [ ] トランザクション動作

#### 実装・修正 (45分)
- [ ] 必要な調整
- [ ] バグ修正

---

## Phase 5: MCP統合 (3-4時間)

### サイクル 5-1: MCPプロトコル

#### テスト作成 (45分)
- [ ] `test_remember_params_validation()`
  - [ ] 必須フィールド確認（title, content, tags）
  - [ ] 型変換確認（String → MemoryType）
  - [ ] 無効なメモリタイプでエラー（-32602）
- [ ] `test_recall_params_validation()`
  - [ ] オプショナルフィールド処理
  - [ ] デフォルト値（limit=10）
- [ ] `test_mcp_error_codes()`
  - [ ] -32602: Invalid params
  - [ ] -32603: Internal error

#### 実装 (45分)
- [ ] RememberParams/Response構造体
  ```rust
  #[derive(Debug, Serialize, Deserialize, JsonSchema)]
  pub struct RememberParams {
      pub memories: Vec<MemoryInput>,
  }
  
  #[derive(Debug, Serialize, Deserialize, JsonSchema)]
  pub struct RememberResponse {
      pub memory_ids: Vec<String>,
      pub created_count: usize,
  }
  ```
- [ ] RecallParams/Response構造体
  ```rust
  #[derive(Debug, Serialize, Deserialize, JsonSchema)]
  pub struct RecallResponse {
      pub content: String,
      pub total_count: usize,
  }
  ```
- [ ] schemars実装
- [ ] エラー変換処理

### サイクル 5-2: MCPサーバー

#### テスト作成 (1時間)
- [ ] `test_mcp_server_initialization()`
  - [ ] instructionsがconfig.tomlから設定される
  - [ ] サーバー情報が正しく返される
- [ ] `test_mcp_server_remember_tool()`
  - [ ] ツール呼び出しテスト
  - [ ] バッチ処理の結果確認
  - [ ] memory_idsが全て返される
- [ ] `test_mcp_server_recall_tool()`
  - [ ] 検索結果の返却
  - [ ] Markdown形式の内容
  - [ ] total_countが正しい
- [ ] `test_mcp_server_error_handling()`
  - [ ] 無効なパラメータでエラーコード-32602
  - [ ] 内部エラーでエラーコード-32603

#### 実装 (1時間)
- [ ] MemoryMcpServer実装
  ```rust
  pub struct MemoryMcpServer<R: MemoryRepository> {
      service: Arc<Mutex<MemoryService<R>>>,
      config: KiroConfig,
      tool_router: ToolRouter<Self>,
  }
  ```
- [ ] サーバー初期化（instructions設定）
- [ ] rmcpマクロ統合
- [ ] rememberツールハンドラー実装
  - [ ] メモリタイプバリデーション
  - [ ] エラーコード設定
- [ ] recallツールハンドラー実装
  - [ ] total_count計算
  - [ ] エラーコード設定

#### レビューポイント
- [ ] MCPプロトコル準拠
- [ ] エラーコードの適切さ
- [ ] 非同期処理の安全性

---

## Phase 6: CLIコマンド (2-3時間)

### サイクル 6-1: initコマンド

#### テスト作成 (30分)
- [ ] `test_init_creates_kiro_directory()`
- [ ] `test_init_creates_config_toml()`
- [ ] `test_init_updates_gitignore()`
- [ ] `test_init_force_flag()`

#### 実装 (30分)
- [ ] initコマンド実装
- [ ] config.tomlテンプレート
- [ ] .gitignore更新ロジック

### サイクル 6-2: serveコマンド

#### テスト作成 (20分)
- [ ] `test_serve_starts_mcp_server()`
  - [ ] サーバー起動確認

#### 実装 (20分)
- [ ] serveコマンド実装
- [ ] MCPサーバー起動処理

### サイクル 6-3: documentコマンド

#### テスト作成 (30分)
- [ ] `test_document_generates_markdown_files()`
- [ ] `test_document_type_filter()`

#### 実装 (30分)
- [ ] documentコマンド実装
- [ ] ファイル生成処理

---

## Phase 7: E2Eテスト & 品質保証 (3-4時間)

### サイクル 7-1: 完全統合テスト

#### テスト作成 (1.5時間)
- [ ] テストフィクスチャ準備
  - [ ] `tests/fixtures/memories.yaml` 作成
  - [ ] 日本語を含むテストデータ
  - [ ] 各メモリタイプのサンプル
- [ ] `tests/e2e/memory_test.rs`
  - [ ] init → serve → remember → recall フロー
  - [ ] document生成フロー
  - [ ] エラーケース（無効なタイプ、大きすぎるデータ）
- [ ] ロギング/トレーシング初期化
  - [ ] `tracing_subscriber::fmt::init()` in tests
  - [ ] 詳細なデバッグログ

#### 実装・修正 (1時間)
- [ ] バグ修正
- [ ] エラーメッセージの改善
- [ ] ログ出力の追加

### サイクル 7-2: パフォーマンステスト

#### テスト作成 (1時間)
- [ ] `test_performance_remember_under_50ms()`
  - [ ] 単一メモリ保存が50ms以内
- [ ] `test_performance_recall_under_100ms()`
  - [ ] 1000件での検索が100ms以内
- [ ] `test_performance_document_under_1s()`
  - [ ] 1000件でのドキュメント生成が1秒以内
- [ ] `test_fts5_japanese_search()`
  - [ ] 日本語検索の精度確認
  - [ ] tokenize = 'porter unicode61'の動作確認
- [ ] `test_database_size_limits()`
  - [ ] 10,000件で100MB以内

#### 実装・最適化 (30分)
- [ ] インデックス最適化
- [ ] クエリチューニング
- [ ] バッチ処理の最適化

---

## 完了条件

### 必須
- [ ] すべてのテストがグリーン
- [ ] カバレッジ80%以上
- [ ] `cargo clippy` 警告なし
- [ ] `cargo fmt` 実行済み

### ドキュメント
- [ ] README.md更新
- [ ] 使用例の追加
- [ ] MCPクライアント設定例

### 最終確認
- [ ] Claude Codeから実際に使用可能
- [ ] パフォーマンス目標達成
  - [ ] remember < 50ms
  - [ ] recall < 100ms
  - [ ] document生成 < 1s

---

## 見積もり時間

- **Phase 0**: 1時間（環境準備 + プロジェクト構造）
- **Phase 1**: 3-4時間（ドメインモデル + KiroConfig）
- **Phase 2**: 4-5時間（Repository層 + SQLite統合）
- **Phase 3**: 4-5時間（Service層 + 非同期処理）
- **Phase 4**: 2-3時間（Infrastructure統合）
- **Phase 5**: 3-4時間（MCP統合 + エラーハンドリング）
- **Phase 6**: 2-3時間（CLIコマンド）
- **Phase 7**: 3-4時間（E2Eテスト + パフォーマンス）

**合計**: 22-29時間 (3-4日間)

各サイクルでレビューを挟むことで、高品質なコードを維持しながら着実に実装を進められます。