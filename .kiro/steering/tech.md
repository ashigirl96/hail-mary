# Technology Stack

## Architecture
- **Language**: Rust (stable, latest version via rustup)
- **Architecture Pattern**: Clean layered architecture
  - CLI Layer: Command routing with clap
  - Application Layer: Use cases and business logic
  - Domain Layer: Entities and value objects (steering system)
  - Infrastructure Layer: Filesystem operations, process management
- **Concurrency**: Tokio async runtime for future extensibility
- **Error Handling**: `anyhow::Result` with `thiserror` for domain errors

## Core Systems
- **File Management**: TOML configuration and markdown file operations
- **Process Integration**: TTY-aware Claude Code launching
- **TUI Framework**: ratatui with crossterm for interactive interfaces
- **Serialization**: serde with JSON/TOML support for configuration
- **Template System**: Structured specification generation

## Development Environment
- **Build System**: Cargo workspace with multiple crates
- **Task Runner**: Just task runner for development commands
- **Testing**: rstest for parameterized tests, tempfile for isolation
- **Code Quality**: clippy, rustfmt, comprehensive test suite
- **Documentation**: Embedded documentation with cargo doc

## Common Commands

### Development Workflow
```bash
# Core development workflow
just build              # Standard build
just test               # Run all tests (unit + integration)
just fmt                # Format code
just lint               # Clippy with -D warnings
just ci                 # Full CI pipeline (format check + lint + test)

# Development utilities  
just run init           # Initialize project
just run memory serve  # Start MCP server
just dev                # Watch mode (check + test + run)
just test-watch         # Watch mode for tests only
```

### Application Commands
```bash
# Project initialization and management
hail-mary init                              # Initialize .kiro directory
hail-mary new <feature-name>                # Create feature specification
hail-mary complete                          # Interactive TUI for spec completion
hail-mary code [--no-danger]                # Launch Claude Code with context

# Shell completions
hail-mary shell-completions <shell>         # Generate completion scripts
```

### Testing Commands
```bash
# Development testing (Preferred)
just ci                                         # Full CI pipeline - USE THIS
just fix                                        # Format code before testing
just test                                       # Run all tests

# Direct cargo test (Avoid - use just commands instead)
cargo test                                      # All tests  
cargo test -- --nocapture                      # Test output visible
RUST_BACKTRACE=1 cargo test -- --nocapture    # With backtraces
```

## Testing Guidelines
**When**: After implementation completion
- Always use `just fix` and `just ci` instead of direct `cargo test`
- Ensures consistent formatting and comprehensive validation
- Maintains CI/CD compatibility

## Environment Variables
- `RUST_LOG`: Logging level (debug, info, warn, error)
- `RUST_BACKTRACE`: Error backtrace display (0, 1, full)
- `CARGO_MANIFEST_DIR`: Project root for integration tests

## Port Configuration
- **Development**: No network services or ports
- **Testing**: File-based operations with temporary directories

## Key Dependencies
```toml
[dependencies]
# CLI and async runtime
clap = { version = "4.5", features = ["derive"] }
clap_complete = "4.5"
tokio = { version = "1", features = ["full"] }

# Configuration and serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"

# TUI and terminal interaction
ratatui = "0.29"
crossterm = "0.28"

# Error handling and utilities
anyhow = "1"
thiserror = "1"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = "0.3"
pulldown-cmark = "0.11"
regex = "1"
```

---

# 🎯 今回のコーディングから得た学び

## 1. **統合テストでのバイナリ実行**
**問題**: 統合テストで実行ファイルが見つからない
```rust
// ❌ 失敗: 環境変数に依存
let cargo_bin = env::var("CARGO_BIN_EXE_hail-mary")
    .unwrap_or_else(|_| "target/debug/hail-mary".to_string());

// ✅ 解決: プロジェクトルートからの相対パス
let project_root = env::var("CARGO_MANIFEST_DIR")
    .map(PathBuf::from)
    .unwrap_or_else(|_| PathBuf::from("."));
let binary_path = project_root.join("target/debug/hail-mary");
```

## 2. **テストでの作業ディレクトリ管理**
**学び**: tempfileクレートと適切なディレクトリ切り替え
```rust
// ✅ 一時ディレクトリでのE2Eテスト
let temp_dir = TempDir::new().unwrap();
let temp_path = temp_dir.path().to_str().unwrap();

Command::new(binary_path)
    .args(args)
    .current_dir(working_dir)  // 重要：作業ディレクトリを指定
    .output()
```

## 3. **構造化エラーハンドリング**
**学び**: thiserrorで読みやすいエラーメッセージ
```rust
#[derive(Error, Debug)]
pub enum HailMaryError {
    #[error("Feature '{0}' already exists")]
    FeatureAlreadyExists(String),
    
    #[error("Invalid feature name: {0}. Must be kebab-case")]
    InvalidFeatureName(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

## 4. **CLIテストでの出力検証**
**学び**: エラーメッセージは実際の型名で検証
```rust
// ❌ 人間向けメッセージで検証
assert!(stderr.contains("Invalid feature name"));

// ✅ 実際のエラー型名で検証
assert!(stderr.contains("InvalidFeatureName"));
assert!(stderr.contains("FeatureAlreadyExists"));
```

## 5. **段階的実装とテスト駆動開発**
**学び**: 小さな単位での実装→テスト→統合のサイクル
```
1. 基本構造 → cargo check
2. 単体テスト → cargo test (unit)
3. 統合テスト → cargo test --test integration
4. 全体検証 → cargo test
```

## 6. **rmcp 0.5.0 マイグレーション**
**学び**: 公式SDKへの移行による大幅なコード削減と保守性向上
```rust
// ❌ カスタムJSON-RPC実装 (270行)
pub struct CustomMcpServer {
    // 手動JSON-RPCハンドリング
}

// ✅ rmcp Tool Router パターン (143行)
#[derive(Clone)]
pub struct MemoryMcpServer {
    tool_router: ToolRouter<Self>,
}

#[tool_router(router = tool_router)]
impl MemoryMcpServer {
    #[tool(name = "remember")]
    pub async fn remember(&self, params: Parameters<RmcpRememberParams>) 
        -> Result<Json<RmcpRememberResponse>, McpError> {
        // 自動的なJSON-RPCハンドリング
    }
}
```

## 7. **未使用コード分析の重要性**
**学び**: 設計仕様との照合による適切な判断
```rust
// ❌ 単純削除 - 設計仕様を無視
warning: function `embed_text` is never used

// ✅ 設計仕様確認 - Phase 3で必要と判明
// デザイン仕様書 Section 5.4: "Generate embeddings fastembed"
// → reindex機能で必要なため保持

// ✅ 真に不要な機能のみ削除
pub fn with_examples() -> Self { } // テストでも未使用
```

## 8. **条件付きコンパイルでのテスト専用メソッド**
**学び**: `#[cfg(test)]`による適切な範囲制限
```rust
// ✅ テスト専用メソッドの正しい定義
#[cfg(test)]
pub fn with_tags(
    memory_type: MemoryType,
    topic: String,
    content: String,
    tags: Vec<String>,
) -> Self {
    // テスト時のみコンパイル
    // 本番ビルドでは警告も出ない
}
```

## 9. **データベースマイグレーション管理**
**学び**: 段階的スキーマ拡張の重要性
```rust
// ✅ 適切なマイグレーション順序
pub fn migrations() -> Migrations<'static> {
    Migrations::new(vec![
        M::up(include_str!("../../migrations/001_initial_schema.sql")),
        M::up(include_str!("../../migrations/002_vector_storage.sql")), // 追加
    ])
}
```

## 10. **JSON Schemaとrmcp統合**
**学び**: 構造化出力のための型安全性
```rust
// ✅ rmcp互換の型定義
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RmcpRememberParams {
    pub r#type: String,           // Rust予約語のエスケープ
    pub topic: String,
    pub content: String,
    pub tags: Option<Vec<String>>,
}

// ✅ 既存型との変換実装
impl From<RmcpRememberParams> for RememberParams {
    fn from(params: RmcpRememberParams) -> Self {
        // 型変換ロジック
    }
}
```

## 11. **Borrowing CheckerとMutable Reference**
**学び**: データを変更するメソッドには`&mut`が必須
```rust
// ❌ コンパイルエラー: cannot borrow as mutable
fn import_memories(&self, repository: &SqliteMemoryRepository, memories: Vec<Memory>)

// ✅ 解決: mutable referenceが必要
fn import_memories(&self, repository: &mut SqliteMemoryRepository, memories: Vec<Memory>)
let mut repository = SqliteMemoryRepository::new(&db_path)?;
```

## 12. **Error Trait Chain with thiserror**
**学び**: `#[from]`で他のエラー型から自動変換、エラーチェーンで具体的な原因を保持
```rust
#[derive(Error, Debug)]
pub enum HailMaryError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),  // 新しく追加
}
```

## 13. **clap ValueEnum for CLI Integration**
**学び**: `clap::ValueEnum`でenumを直接CLI引数として使用可能
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, JsonSchema, clap::ValueEnum)]
pub enum MemoryType {
    Tech, ProjectTech, Domain,
}

// CLI引数として自動的に使用可能
#[arg(long, value_enum)]
pub r#type: Option<MemoryType>,
```

## 14. **条件付きSerialization**
**学び**: `skip_serializing_if`でJSONを簡潔に、API設計で重要
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportMemory {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
    #[serde(rename = "type")]
    pub memory_type: String,
}
```

## 15. **CSV Parsing with Quote Handling**
**学び**: CSVのエスケープ処理、ダブルクォートは`""`で表現
```rust
// CSVフィールドの適切なエスケープ処理
let value = if value.starts_with('"') && value.ends_with('"') && value.len() > 1 {
    &value[1..value.len()-1].replace("\"\"", "\"")
} else {
    value
};

fn escape_csv_field(field: &str, delimiter: &str) -> String {
    if field.contains(delimiter) || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}
```

## 16. **Pattern Matching with Validation**
**学び**: Resultのmatch文でエラーハンドリング、早期リターンパターン
```rust
let regex = if self.case_sensitive {
    Regex::new(&self.query)
} else {
    Regex::new(&format!("(?i){}", self.query))
};

let regex = match regex {
    Ok(r) => r,
    Err(e) => {
        eprintln!("Error: Invalid regex pattern: {}", e);
        return Ok(Vec::new());  // 早期リターン
    }
};
```

## 17. **Iterator Chains with Complex Filtering**
**学び**: `all()`と`any()`の組み合わせで複雑な条件フィルタリング
```rust
// 全てのタグが部分一致することを確認
memories.retain(|m| {
    filter_tags.iter().all(|tag| 
        m.tags.iter().any(|mem_tag| 
            mem_tag.to_lowercase().contains(&tag.to_lowercase())
        )
    )
});

// 年齢フィルタ
if let Some(max_age) = self.max_age_days {
    let cutoff_time = chrono::Utc::now().timestamp() - (max_age * 24 * 60 * 60);
    memories.retain(|m| m.created_at >= cutoff_time);
}
```

## 18. **String Formatting with Context**
**学び**: 文字列の切り取りと条件付きフォーマット、ユーザビリティ向上
```rust
// プレビュー表示のための文字列切り取り
let content = if memory.content.len() > 200 {
    format!("{}...", &memory.content[..200])
} else {
    memory.content.clone()
};

// 複数パターンでの表示制御
let snippet = if self.verbose {
    &memory.content
} else if self.snippets {
    &preview_content
} else {
    &short_content
};
```

## 19. **並列テスト実行における`current_dir`競合問題**
**学び**: プロセス全体のグローバル状態とスレッド間競合の理解

### 問題の発見
```rust
// ❌ 並列テスト実行で競合が発生
// 原因: env::set_current_dir() はプロセス全体のグローバル状態を変更
Thread A: set_current_dir("/tmp/uuid-A") 
Thread B: set_current_dir("/tmp/uuid-B")  // Thread Aを上書き！
Thread A: Path::new(".kiro") -> 実際は /tmp/uuid-B/.kiro を参照
```

### 解決策: Mutexによる同期化
```rust
// ✅ グローバルMutexでcurrent_dir操作を同期化
static TEST_DIR_MUTEX: Mutex<()> = Mutex::new(());

pub struct TestDirectory {
    _temp_dir: TempDir,           // 独立したUUIDディレクトリ
    original_dir: PathBuf,
    _guard: MutexGuard<'static, ()>,  // グローバルロック
}

impl TestDirectory {
    pub fn new() -> Self {
        let guard = TEST_DIR_MUTEX.lock().expect("Failed to acquire test directory mutex");
        let original_dir = env::current_dir().expect("Failed to get current directory");
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
        
        env::set_current_dir(temp_dir.path())
            .expect("Failed to change to temp directory");
            
        Self { _temp_dir: temp_dir, original_dir, _guard: guard }
    }
}
```

### 学んだポイント
- **UUIDディレクトリは別々**: 各テストが独立した一時ディレクトリを作成 ✅
- **current_dirはグローバル**: `env::set_current_dir()`はプロセス単位でグローバル ❌
- **並列実行での競合**: 複数スレッドが同じプロセスの`current_dir`を競合
- **最小限の同期化**: テスト自体は並列実行、`current_dir`操作のみ同期化
- **RAIIパターン**: MutexGuardの自動解放で確実なクリーンアップ

## 20. **TestDirectoryのRAIIパターン設計**
**学び**: リソース管理の自動化と例外安全性

### RAII (Resource Acquisition Is Initialization) の実装
```rust
// ✅ 完全自動化されたテスト環境管理
let _test_dir = TestDirectory::new();
// テスト処理
// Drop時に自動復元（パニック時も確実に実行）

// Before: 手動管理（脆弱）
let temp_dir = setup_test_dir();
let original_dir = env::current_dir().unwrap();
env::set_current_dir(temp_dir.path()).unwrap();
// テスト処理
env::set_current_dir(original_dir).unwrap(); // 手動復元（パニック時に失敗）
```

### 設計の利点
- **例外安全性**: パニック時も確実にリソース解放
- **コードの簡潔性**: 手動復元コードが不要
- **テスト間分離**: 各テストが完全に独立した環境で実行
- **開発者体験**: 忘れがちなクリーンアップを自動化

## 💡 **重要な気づき**
- **テストファーストアプローチ**: エラーケースを先にテストすることで実装の抜け漏れを防止
- **環境に依存しない設計**: テスト環境での実行を考慮した堅牢な実装
- **段階的検証**: check → unit test → integration test の段階的アプローチが効果的
- **設計仕様との整合性**: 未使用コード警告は設計文書と照合して判断する
- **公式SDK活用**: カスタム実装より公式ライブラリを優先し、保守コストを削減
- **条件付きコンパイル**: テスト専用機能は`#[cfg(test)]`で適切に分離
- **Borrowing Checker理解**: 実行時エラーを防ぐコンパイル時チェックの重要性
- **エラーチェーン設計**: 具体的なエラー原因を保持する階層的エラーハンドリング
- **CLI設計**: enumとclapの統合でタイプセーフなコマンドライン引数
- **データ変換設計**: JSONとCSVの相互変換における適切なエスケープ処理
- **並列テスト設計**: グローバル状態とスレッド間競合を理解した適切な同期化
- **RAIIパターン**: リソース管理の自動化による例外安全性とコードの簡潔性

## 21. **TOML構造的パース vs 文字列検索**
**学び**: `toml` crateによる型安全なTOML操作
```rust
// ❌ 脆弱な文字列検索
if content.contains("[steering.backup]") {
    // コメント内の文字列でも反応してしまう
}

// ✅ 構造的パース
let parsed: toml::Value = toml::from_str(&content)?;
if let Some(steering) = parsed.get("steering")
    && let Some(_backup) = steering.get("backup")
{
    // TOMLの実際の構造を検証
    // 型安全でパース失敗も適切にハンドリング
}
```

**利点**:
- パース失敗時の適切なエラーハンドリング
- コメントや文字列リテラル内の誤検知を回避
- 将来的なTOML構造変更に対応可能
- コードの意図が明確
