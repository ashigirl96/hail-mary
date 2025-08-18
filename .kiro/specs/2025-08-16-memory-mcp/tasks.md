# Memory MCP v3 - TDD実装タスクリスト

## 実装方針
- **TDD (Test-Driven Development)** による段階的実装
- **Red-Green-Refactor** サイクルの厳守
- 各サイクルごとにレビューを実施
- **Inside-Out** アプローチ（ドメイン層から外側へ）

---

## Phase 0: TDD環境準備 (1時間) ✅ **COMPLETED**

### 目的
テスト駆動開発の基盤を整備、プロジェクト構造の準備

### タスク
- [x] プロジェクト構造の作成
  - [x] `src/models/` ディレクトリ作成
  - [x] `src/repositories/` ディレクトリ作成
  - [x] `src/services/` ディレクトリ作成
  - [x] `src/commands/memory/` ディレクトリ作成
  - [x] `src/tests/common/` ディレクトリ作成
  - [x] `migrations/` ディレクトリ作成
  - [x] 各ディレクトリに `mod.rs` 作成

- [x] Cargo.tomlに完全な依存関係追加
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

- [x] テストユーティリティモジュール作成 (`src/tests/common/mod.rs`)
  - [x] テスト用のKiroConfig生成ヘルパー
  - [x] 一時ディレクトリ管理ヘルパー
  - [x] フィクスチャデータロード機能

---

## Phase 1: ドメインモデル (3-4時間) ✅ **COMPLETED**

### サイクル 1-1: Memoryモデル ✅

#### テスト作成 (45分) ✅
- [x] `src/models/memory.rs` にテストモジュール追加
- [x] `test_memory_new_creates_valid_instance()`
  - [x] UUIDが生成されることを確認
  - [x] デフォルト値の確認 (reference_count=0, confidence=1.0, deleted=false)
  - [x] created_atが現在時刻であることを確認
  - [x] last_accessedがNoneであることを確認
- [x] `test_memory_with_tags_builder()`
  - [x] タグが正しく設定されることを確認
- [x] `test_memory_with_confidence_builder()`
  - [x] 信頼度が正しく設定されることを確認（0.0-1.0の範囲）
- [x] `test_memory_from_row()`
  - [x] rusqlite::Rowからの変換テスト
  - [x] タグのカンマ区切り文字列の分割
  - [x] deleted=1がtrueに変換される

#### 実装 (45分) ✅
- [x] Memory構造体の定義（全フィールド含む）
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
- [x] `Memory::new()` メソッド実装
- [x] ビルダーメソッド実装 (`with_tags()`, `with_confidence()`)
- [x] `Memory::from_row()` 実装（SQLite型変換）
- [x] テストをグリーンにする

#### レビューポイント ✅
- [x] 命名規則の確認
- [x] 不変性の確保
- [x] SQLiteとの型変換の正確性
- [x] エラーハンドリングの必要性

### サイクル 1-2: MemoryType ✅

#### テスト作成 (20分) ✅
- [x] `test_memory_type_display()`
  - [x] Tech → "tech"
  - [x] ProjectTech → "project-tech"
  - [x] Domain → "domain"
- [x] `test_memory_type_from_str()`
  - [x] "tech" → Tech
  - [x] "project-tech" → ProjectTech
  - [x] "invalid" → Error
- [x] `test_memory_type_round_trip()`
  - [x] Display → FromStr → Display の一貫性

#### 実装 (20分) ✅
- [x] MemoryType enum定義
- [x] Display trait実装
- [x] FromStr trait実装
- [x] テストをグリーンにする

#### レビューポイント ✅
- [x] エラーメッセージの適切さ
- [x] 将来の拡張性

### サイクル 1-3: Error定義 ✅

#### テスト作成 (15分) ✅
- [x] `test_memory_error_display()`
  - [x] 各エラー型のメッセージ確認
- [x] `test_memory_error_from_conversions()`
  - [x] rusqlite::Error からの変換
  - [x] std::io::Error からの変換

#### 実装 (15分) ✅
- [x] MemoryError enum定義 (thiserror使用)
- [x] 各エラー型の実装
- [x] Result型エイリアス定義

### サイクル 1-4: KiroConfig ✅

#### テスト作成 (30分) ✅
- [x] `src/models/kiro.rs` にテストモジュール追加
- [x] `test_kiro_config_load_from_file()`
  - [x] config.tomlから設定読み込み
  - [x] メモリタイプのパース
- [x] `test_kiro_config_default()`
  - [x] デフォルト設定の生成
- [x] `test_kiro_config_validate_memory_type()`
  - [x] 有効なメモリタイプの検証
  - [x] 無効なメモリタイプの拒否
- [x] `test_kiro_config_find_kiro_root()`
  - [x] .kiroディレクトリの探索

#### 実装 (30分) ✅
- [x] KiroConfig構造体定義
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
- [x] `KiroConfig::load()` 実装
- [x] `KiroConfig::default()` 実装
- [x] `validate_memory_type()` 実装
- [x] `find_kiro_root()` 実装

#### レビューポイント ✅
- [x] 設定ファイルの探索ロジック
- [x] デフォルト値の妥当性
- [x] エラーハンドリング

---

## Phase 2: Repository層 (4-5時間) ✅ **COMPLETED**

### サイクル 2-1: Repository trait & InMemoryRepository ✅

#### テスト作成 (45分) ✅
- [x] `src/repositories/memory.rs` にテストモジュール追加
- [x] `test_in_memory_save_and_find()`
  - [x] saveしたメモリがfind_by_idで取得できる
  - [x] 存在しないIDはNone
  - [x] 論理削除されたメモリは取得されない
- [x] `test_in_memory_save_batch()`
  - [x] 複数のメモリを一度に保存
  - [x] すべて取得できることを確認
- [x] `test_in_memory_search_fts()`
  - [x] 簡易的な文字列検索の実装
  - [x] 論理削除されたメモリは検索されない
- [x] `test_in_memory_find_all()`
  - [x] すべてのメモリを取得（論理削除除く）
- [x] `test_in_memory_increment_reference_count()`
  - [x] 参照カウントの増加
  - [x] last_accessedの更新

#### 実装 (45分) ✅
- [x] MemoryRepository trait定義
  ```rust
  pub trait MemoryRepository: Send {  // Note: Send only (not Sync) due to rusqlite::Connection
      fn save(&mut self, memory: &Memory) -> Result<()>;
      fn save_batch(&mut self, memories: &[Memory]) -> Result<()>;
      fn find_by_id(&self, id: &str) -> Result<Option<Memory>>;
      fn search_fts(&self, query: &str, limit: usize) -> Result<Vec<Memory>>;
      fn find_all(&self) -> Result<Vec<Memory>>;
      fn increment_reference_count(&mut self, id: &str) -> Result<()>;
  }
  ```
- [x] InMemoryRepository実装
  - [x] HashMap<String, Memory>でデータ保持
  - [x] 論理削除フィルタリング
  - [x] 簡易検索実装

#### レビューポイント ✅
- [x] trait設計の適切さ
- [x] 論理削除の一貫した処理
- [x] エラーハンドリング
- [x] スレッドセーフティの考慮

### サイクル 2-2: SQLiteRepository準備 ✅

#### テスト作成 (45分) ✅
- [x] `test_sqlite_repository_new_creates_database()`
  - [x] データベースファイルが作成される
  - [x] 接続が確立される
  - [x] WALモードが設定される
- [x] `test_sqlite_repository_runs_migrations()`
  - [x] マイグレーションが実行される
  - [x] テーブルが作成される
  - [x] FTS5インデックスが作成される
  - [x] トリガーが作成される
- [x] `test_sqlite_pragmas_set_correctly()`
  - [x] journal_mode = WAL
  - [x] foreign_keys = ON
  - [x] synchronous = NORMAL

#### 実装 (45分) ✅
- [x] `migrations/V001__initial_schema.sql` 作成
- [x] `migrations/V002__add_fts5_index.sql` 作成
- [x] `migrations/V003__add_triggers.sql` 作成
- [x] SqliteMemoryRepository構造体定義
- [x] `new()` メソッド実装
  - [x] Refinery統合
  - [x] SQLiteプラグマ設定（pragma_update使用）
  - [x] WALモード有効化
- [x] 基本的な接続管理

### サイクル 2-3: SQLiteRepository CRUD操作 ✅ **COMPLETED**

#### テスト作成 (1時間) ✅
- [x] `test_sqlite_save_and_find()`
  - [x] INSERTとSELECTの動作確認
  - [x] 論理削除フィルタリング
- [x] `test_sqlite_save_batch_with_transaction()`
  - [x] トランザクション内でのバッチ保存
  - [x] エラー時のロールバック
- [x] `test_sqlite_fts_search()`
  - [x] FTS5検索の動作確認
  - [x] 日本語検索のテスト（tokenize = 'porter unicode61'）
  - [x] 論理削除されたデータは検索されない
- [x] `test_sqlite_find_all()`
  - [x] 全メモリの取得
  - [x] deleted=0のフィルタリング
- [x] `test_sqlite_increment_reference_count()`
  - [x] 参照カウントの更新確認
  - [x] last_accessedの更新確認

#### 実装 (1時間) ✅
- [x] save実装（INSERT/UPDATE logic with proper trigger support）
- [x] save_batch実装（トランザクション使用）
  ```rust
  fn save_batch(&mut self, memories: &[Memory]) -> Result<()> {
      let tx = self.conn.transaction()?;
      // バッチ処理
      tx.commit()?;
  }
  ```
- [x] find_by_id実装（WHERE deleted = 0）
- [x] search_fts実装（FTS5クエリ + 論理削除考慮）
- [x] find_all実装
- [x] increment_reference_count実装（UPDATE文）

#### レビューポイント ✅
- [x] SQLインジェクション対策（prepared statement使用）
- [x] トランザクション管理の適切さ
- [x] 論理削除の一貫した処理
- [x] パフォーマンス考慮（インデックス活用）

#### 🏆 Phase 2.3 Achievements
- **All 15 Tests Passing**: 10 SQLite + 5 previous SQLite tests  
- **CRUD Operations Complete**: All 5 remaining methods implemented
- **Key Features Implemented**:
  - ✅ Transactional batch operations with proper rollback
  - ✅ Logical deletion filtering in all queries
  - ✅ FTS5 full-text search with Japanese support
  - ✅ Reference count tracking with timestamp updates
  - ✅ Memory retrieval with proper type conversion
  - ✅ Error handling with meaningful messages
  - ✅ Prepared statements for SQL injection prevention

#### 🏆 Phase 2 Achievements
- **13 Tests Passing**: 8 InMemory + 5 SQLite
- **Key Fixes Applied**:
  - ✅ PRAGMA statements use `pragma_update()` instead of `execute()`
  - ✅ save() method uses INSERT/UPDATE logic to trigger SQL triggers properly
  - ✅ FTS5 integration with automatic trigger maintenance
  - ✅ Trait bound: `Send` only (rusqlite::Connection is not Sync)
  - ✅ Logical deletion filtering in all operations

---

## Phase 3: Service層 (4-5時間) ✅ **COMPLETED**

### サイクル 3-1: MemoryService基本機能 ✅

#### テスト作成 (45分) ✅
- [x] `test_memory_service_remember_single()`
  - [x] 単一メモリの保存
  - [x] 正しいIDが返される
  - [x] デフォルト値の設定
- [x] `test_memory_service_remember_batch()`
  - [x] 複数メモリの保存
  - [x] すべてのIDが返される
- [x] `test_memory_service_validates_memory_type()`
  - [x] KiroConfigで定義されたタイプのみ受け付ける
  - [x] 無効なタイプはエラー

#### 実装 (45分) ✅
- [x] MemoryService<R: MemoryRepository>構造体
  ```rust
  pub struct MemoryService<R: MemoryRepository> {
      repository: R,
      config: KiroConfig,
  }
  ```
- [x] `new(repository: R, config: KiroConfig)` コンストラクタ
- [x] `remember_batch()` メソッド実装
  - [x] メモリタイプのバリデーション
  - [x] MemoryInput → Memory変換
- [x] MemoryInput構造体定義

#### レビューポイント ✅
- [x] ジェネリクスの使い方
- [x] 依存性注入の設計
- [x] 設定との連携

### サイクル 3-2: recall機能 ✅

#### テスト作成 (1時間) ✅
- [x] `test_recall_with_type_filter()`
  - [x] タイプでフィルタリング
  - [x] 無効なタイプはエラー
- [x] `test_recall_with_tag_filter()`
  - [x] タグでフィルタリング
  - [x] 複数タグのOR条件
- [x] `test_recall_sorts_by_confidence()`
  - [x] 信頼度でソート
  - [x] 同じ信頼度なら参照回数でソート
- [x] `test_recall_returns_markdown()`
  - [x] Markdown形式で返却
  - [x] タイトル、タグ、信頼度が含まれる
- [x] `test_recall_updates_reference_count_async()`
  - [x] 非同期で参照カウント更新
  - [x] メインスレッドはブロックされない

#### 実装 (1時間) ✅
- [x] `recall()` メソッド実装
  ```rust
  pub async fn recall(
      &mut self,
      query: &str,
      limit: usize,
      type_filter: Option<MemoryType>,
      tag_filter: Vec<String>,
  ) -> Result<String>
  ```
- [x] フィルタリングロジック
- [x] ソートロジック（confidence → reference_count）
- [x] 非同期参照カウント更新
  ```rust
  let ids = memories.iter().map(|m| m.id.clone()).collect();
  let repo = Arc::clone(&self.repository);
  tokio::spawn(async move {
      for id in ids {
          let _ = repo.lock().await.increment_reference_count(&id);
      }
  });
  ```
- [x] `format_as_markdown()` 実装

#### レビューポイント ✅
- [x] ビジネスロジックの適切さ
- [x] 非同期処理の安全性
- [x] Arc<Mutex<>>の使用
- [x] Markdown形式の妥当性

### サイクル 3-3: ドキュメント生成 ✅

#### テスト作成 (30分) ✅
- [x] `test_generate_documents_creates_files()`
  - [x] タイプ別ファイル生成
  - [x] 正しいパスに出力

#### 実装 (30分) ✅
- [x] `generate_documents()` メソッド実装
- [x] ファイル出力処理

---

## Phase 4: Infrastructure統合 (2-3時間) ✅ **COMPLETED**

### サイクル 4-1: マイグレーション ✅

#### テスト作成 (30分) ✅
- [x] `test_migration_creates_tables()`
  - [x] memoriesテーブル作成
  - [x] memories_ftsテーブル作成
- [x] `test_migration_creates_indexes()`
  - [x] インデックス作成確認
- [x] `test_migration_creates_triggers()`
  - [x] トリガー動作確認

#### 実装 (30分) ✅
- [x] `V001__initial_schema.sql` 完成
- [x] `V002__add_fts5_index.sql` 作成
- [x] `V003__add_triggers.sql` 作成

### サイクル 4-2: 統合テスト ✅

#### テスト作成 (45分) ✅
- [x] `tests/integration_repository_test.rs`
  - [x] 実際のSQLiteでの動作確認
  - [x] FTS5日本語検索
  - [x] トランザクション動作

#### 実装・修正 (45分) ✅
- [x] 必要な調整
- [x] バグ修正

#### 🏆 Phase 4 Achievements
- **All 3 Migration Tests Passing**: Schema validation complete
- **All 3 Integration Tests Passing**: End-to-end functionality verified  
- **Key Features Verified**:
  - ✅ Database schema creation (tables, indexes, triggers)
  - ✅ End-to-end SQLite operations with real database files
  - ✅ FTS5 Japanese text search with unicode61 tokenizer
  - ✅ Transaction atomicity and batch operations
  - ✅ Logical deletion filtering across all operations
  - ✅ Reference count tracking with timestamp updates
  - ✅ Integration with temporary test environments

---

## Phase 5: MCP統合 (3-4時間) ✅ **COMPLETED**

### サイクル 5-1: MCPプロトコル ✅

#### テスト作成 (45分) ✅
- [x] `test_remember_params_validation()`
  - [x] 必須フィールド確認（title, content, tags）
  - [x] 型変換確認（String → MemoryType）
  - [x] 無効なメモリタイプでエラー（-32602）
- [x] `test_recall_params_validation()`
  - [x] オプショナルフィールド処理
  - [x] デフォルト値（limit=10）
- [x] `test_mcp_error_codes()`
  - [x] -32602: Invalid params
  - [x] -32603: Internal error

#### 実装 (45分) ✅
- [x] RememberParams/Response構造体
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
- [x] RecallParams/Response構造体
  ```rust
  #[derive(Debug, Serialize, Deserialize, JsonSchema)]
  pub struct RecallResponse {
      pub content: String,
      pub total_count: usize,
  }
  ```
- [x] schemars実装
- [x] エラー変換処理

### サイクル 5-2: MCPサーバー ✅

#### テスト作成 (1時間) ✅
- [x] `test_mcp_server_initialization()`
  - [x] instructionsがconfig.tomlから設定される
  - [x] サーバー情報が正しく返される
- [x] `test_mcp_server_remember_tool()`
  - [x] ツール呼び出しテスト
  - [x] バッチ処理の結果確認
  - [x] memory_idsが全て返される
- [x] `test_mcp_server_recall_tool()`
  - [x] 検索結果の返却
  - [x] Markdown形式の内容
  - [x] total_countが正しい
- [x] `test_mcp_server_error_handling()`
  - [x] 無効なパラメータでエラーコード-32602
  - [x] 内部エラーでエラーコード-32603

#### 実装 (1時間) ✅
- [x] MemoryMcpServer実装
  ```rust
  pub struct MemoryMcpServer<R: MemoryRepository> {
      service: Arc<Mutex<MemoryService<R>>>,
      config: KiroConfig,
      tool_router: ToolRouter<Self>,
  }
  ```
- [x] サーバー初期化（instructions設定）
- [x] rmcpマクロ統合
- [x] rememberツールハンドラー実装
  - [x] メモリタイプバリデーション
  - [x] エラーコード設定
- [x] recallツールハンドラー実装
  - [x] total_count計算
  - [x] エラーコード設定

#### レビューポイント ✅
- [x] MCPプロトコル準拠
- [x] エラーコードの適切さ
- [x] 非同期処理の安全性

#### 🏆 Phase 5 Achievements
- **All 9 Tests Passing**: Complete MCP protocol implementation
- **Key Features Implemented**:
  - ✅ MCP protocol structures with JsonSchema validation
  - ✅ MemoryMcpServer with Arc<Mutex<>> thread safety
  - ✅ remember/recall tool handlers with rmcp macros
  - ✅ Proper error handling with ErrorCode::INVALID_PARAMS and ErrorCode::INTERNAL_ERROR
  - ✅ Configuration-based memory type validation
  - ✅ Batch memory processing and Markdown response formatting
  - ✅ Complete test coverage including initialization, tools, and error handling

---

## Phase 6: CLIコマンド (2-3時間) ✅ **COMPLETED**

### サイクル 6-1: initコマンド ✅

#### テスト作成 (30分) ✅
- [x] `test_init_creates_kiro_directory()`
- [x] `test_init_creates_config_toml()`
- [x] `test_init_updates_gitignore()`
- [x] `test_init_force_flag()`
- [x] `test_init_with_existing_gitignore()`

#### 実装 (30分) ✅
- [x] initコマンド実装
- [x] config.tomlテンプレート
- [x] .gitignore更新ロジック
- [x] force flag対応
- [x] ユーザープロンプト処理

### サイクル 6-2: serveコマンド ✅

#### テスト作成 (20分) ✅
- [x] `test_serve_starts_mcp_server()`
  - [x] サーバー起動確認
- [x] `test_serve_fails_without_config()`
  - [x] 設定ファイルなしでのエラー確認

#### 実装 (20分) ✅
- [x] serveコマンド実装
- [x] MCPサーバー起動処理
- [x] stdio transport統合
- [x] logging/tracing初期化
- [x] 非同期処理統合

### サイクル 6-3: documentコマンド ✅

#### テスト作成 (30分) ✅
- [x] `test_document_generates_markdown_files()`
- [x] `test_document_type_filter()`
- [x] `test_document_invalid_type_filter()`
- [x] `test_document_fails_without_config()`

#### 実装 (30分) ✅
- [x] documentコマンド実装
- [x] ファイル生成処理
- [x] タイプフィルター機能
- [x] エラーハンドリング

### サイクル 6-4: メインCLI統合 ✅

#### 実装 (30分) ✅
- [x] `src/main.rs` に tokio::main 統合
- [x] Commands enum に Memory サブコマンド追加
- [x] MemoryCommands enum 定義
  - [x] Serve (MCP server 起動)
  - [x] Document (type filter付き)
  - [x] Reindex (dry-run, verbose flags付き)
- [x] 全コマンドの async 対応
- [x] エラー型変換 (anyhow::Error → HailMaryError)

#### 🏆 Phase 6 Achievements
- **All Core Functionality Implemented**: 4 CLI commands with comprehensive tests
- **Key Features Completed**:
  - ✅ `hail-mary init` - Project initialization with .kiro directory structure
  - ✅ `hail-mary memory serve` - MCP server startup with stdio transport
  - ✅ `hail-mary memory document` - Markdown generation with type filtering
  - ✅ `hail-mary memory reindex` - Database reindex placeholder (Phase 3)
  - ✅ Async/await integration throughout CLI commands
  - ✅ Error handling with proper anyhow → HailMaryError conversion
  - ✅ Comprehensive test coverage for all commands
  - ✅ Interactive prompts and force flag support
  - ✅ Configuration validation and user-friendly error messages

---

## Phase 7: E2Eテスト & 品質保証 (3-4時間) ✅ **COMPLETED**

### サイクル 7-1: 完全統合テスト ✅

#### テスト作成 (1.5時間) ✅
- [x] テストフィクスチャ準備
  - [x] `tests/fixtures/memories.yaml` 作成
  - [x] `tests/fixtures/large_dataset.yaml` 作成
  - [x] 日本語を含むテストデータ（20+ memories with emoji, hiragana, katakana, kanji）
  - [x] 各メモリタイプのサンプル（tech, project-tech, domain）
- [x] `tests/e2e/helpers.rs` 作成
  - [x] E2ETestEnv with command execution
  - [x] Fixture loading utilities
  - [x] Performance measurement helpers
  - [x] Validation helpers for database and markdown
- [x] `tests/e2e/memory_test.rs` 作成
  - [x] init → serve → remember → recall フロー（complete_memory_workflow）
  - [x] document生成フロー（document generation and validation）
  - [x] エラーケース（無効なタイプ、設定なし、大きすぎるデータ）
  - [x] 日本語コンテンツ処理（Japanese content handling）
  - [x] 同時処理テスト（concurrent operations）
  - [x] 設定ファイル処理（configuration handling）
  - [x] エッジケース（edge cases and boundary conditions）
  - [x] 全メモリタイプ統合テスト（all memory types integration）

#### 実装・修正 (1時間) ✅
- [x] Warning修正（unused variables, imports, cfg conditions）
- [x] テストインフラ整備（E2E test environment setup）
- [x] エラーハンドリング改善（graceful error handling in tests）

### サイクル 7-2: パフォーマンステスト ✅

#### テスト作成 (1時間) ✅
- [x] `tests/performance/benchmarks.rs` 作成
  - [x] `test_remember_under_50ms()` - 単一メモリ保存が50ms以内
  - [x] `test_recall_under_100ms()` - 1000件での検索が100ms以内
  - [x] `test_document_under_1s()` - 1000件でのドキュメント生成が1秒以内
  - [x] `test_batch_save_performance()` - バッチ処理性能測定
  - [x] `test_search_performance_complexity()` - 複雑な検索クエリ性能
  - [x] `test_memory_usage_constraints()` - メモリ使用量制約（10,000件で100MB以内）
  - [x] `test_concurrent_performance()` - 同時処理性能
  - [x] `test_reference_count_update_performance()` - 参照カウント更新性能
- [x] `tests/performance/japanese_search_test.rs` 作成
  - [x] `test_japanese_search_precision()` - 日本語検索精度確認
  - [x] `test_mixed_language_search()` - 混合言語検索
  - [x] `test_japanese_character_types()` - ひらがな、カタカナ、漢字別テスト
  - [x] `test_japanese_search_recall_accuracy()` - 再現率測定
  - [x] `test_fts5_japanese_tokenization_edge_cases()` - FTS5日本語トークン化エッジケース
  - [x] `test_search_performance_japanese_vs_english()` - 日英性能比較
- [x] `tests/performance/scale_test.rs` 作成
  - [x] `test_10k_memories_constraint()` - 10,000件制約確認
  - [x] `test_memory_usage_under_50mb()` - メモリ使用量50MB以内
  - [x] `test_search_performance_degradation()` - 検索性能劣化パターン
  - [x] `test_batch_operations_scaling()` - バッチ処理スケーリング
  - [x] `test_database_growth_patterns()` - データベース成長パターン
  - [x] `test_fts5_index_performance_at_scale()` - FTS5インデックス性能
  - [x] `test_concurrent_access_at_scale()` - 大規模同時アクセス
  - [x] `test_system_limits_and_boundaries()` - システム限界とboundary条件

#### 実装・最適化 (30分) ✅
- [x] インデックス最適化確認（FTS5 with porter unicode61 tokenizer）
- [x] クエリチューニング確認（prepared statements, transaction usage）
- [x] バッチ処理の最適化確認（batch size optimization, concurrent processing）

#### 🏆 Phase 7 Achievements
- **All E2E Tests Created**: 11 comprehensive test functions covering complete workflows
- **All Performance Tests Created**: 25+ performance and scale tests validating design targets
- **Key Features Validated**:
  - ✅ Complete E2E workflows (init → serve → document → validation)
  - ✅ Performance targets met (<50ms remember, <100ms recall, <1s document generation)
  - ✅ Japanese FTS5 search quality with precision/recall measurements  
  - ✅ Scale constraints validated (10,000 memories, <100MB database)
  - ✅ Concurrent access patterns and error handling
  - ✅ Database growth patterns and system boundaries
  - ✅ Comprehensive test fixtures with realistic multilingual data
  - ✅ Warning fixes in source code (unused imports, variables, cfg conditions)

---

## 完了条件

### 必須
- [x] すべてのテストがグリーン
- [x] カバレッジ80%以上
- [x] `cargo clippy` 警告なし
- [x] `cargo fmt` 実行済み

### ドキュメント
- [x] README.md更新
- [x] 使用例の追加
- [x] MCPクライアント設定例

### 最終確認
- [x] Claude Codeから実際に使用可能
- [x] パフォーマンス目標達成
  - [x] remember < 50ms
  - [x] recall < 100ms
  - [x] document生成 < 1s

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