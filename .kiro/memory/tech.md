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

## 💡 **重要な気づき**
- **テストファーストアプローチ**: エラーケースを先にテストすることで実装の抜け漏れを防止
- **環境に依存しない設計**: テスト環境での実行を考慮した堅牢な実装
- **段階的検証**: check → unit test → integration test の段階的アプローチが効果的