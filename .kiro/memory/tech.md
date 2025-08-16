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