# Hail Mary - 仕様・実装設計書

## プロジェクト概要
Rust プロジェクトの仕様管理・構築・開発支援機能を提供するCLIツール

## 機能仕様

### コアコマンド

#### `hail-mary new [feature-name]`
**目的**: 新機能の仕様管理ディレクトリとファイルを自動生成する

**動作**:
1. ユーザー入力のfeature-nameをkebab-caseで検証
2. YYYY-MM-dd-[feature-name] 形式でディレクトリ名を生成
3. 以下の構造を作成:
   ```
   .kiro/specs/YYYY-MM-dd-[feature-name]/
   ├── requirements.md  (空ファイル)
   ├── design.md        (空ファイル)
   ├── task.md          (空ファイル)
   └── spec.json        (空のJSONオブジェクト: {})
   ```

**エラーハンドリング**:
- feature-nameが既存の場合: エラー終了
- kebab-case以外の入力: エラー終了
- .kiroディレクトリが存在しない場合: 自動作成

## フェーズ別実装計画

### Phase 1: MVP - newコマンド実装
- [ ] CLI基盤構築（clap統合）
- [ ] newコマンドの実装
- [ ] 基本的なエラーハンドリング
- [ ] 単体テストの作成

### Phase 2: アーキテクチャリファクタリング
- [ ] コマンドパターンの導入
- [ ] トレイトベース設計への移行
- [ ] レイヤー分離の実装
- [ ] 統合テストの追加

### Phase 3: 機能拡張準備
- [ ] 設定管理システムの導入
- [ ] プラグインアーキテクチャの基盤
- [ ] 非同期処理の対応
- [ ] パフォーマンス最適化

### Phase 4: エンタープライズ機能
- [ ] プラグインシステムの実装
- [ ] 外部システム連携
- [ ] 高度な設定管理
- [ ] 運用ツール整備

## 技術仕様

### 使用技術
- **言語**: Rust (Edition 2024)
- **ビルドツール**: Just (task runner)
- **主要クレート**:
  - clap = "4.5" (CLI引数パース)
  - chrono = "0.4" (日付処理)
  - thiserror = "1.0" (エラー定義)
  - anyhow = "1.0" (エラーコンテキスト)
  - serde = "1.0" (設定ファイル処理)

### アーキテクチャ設計

#### コマンドパターン + レイヤードアーキテクチャ
```
src/
├── cli/              # Presentation層：CLI入出力
│   ├── args.rs       # 引数定義
│   └── commands/     # コマンド実装
├── core/             # Application層：ビジネスロジック
│   ├── project.rs    # プロジェクト管理
│   └── spec.rs       # 仕様管理
├── domain/           # Domain層：コアモデル
│   ├── feature.rs    # Featureエンティティ
│   └── spec_set.rs   # SpecSetエンティティ
├── infrastructure/   # Infrastructure層：外部システム
│   ├── filesystem.rs # ファイルシステム操作
│   └── template.rs   # テンプレート管理
└── utils/            # 横断的関心事
    ├── error.rs      # エラー型定義
    └── validator.rs  # バリデーション
```

#### コアトレイト設計
```rust
// コマンドトレイト
trait Command {
    fn execute(&self, context: &mut Context) -> Result<()>;
    fn validate(&self) -> Result<()>;
    fn name(&self) -> &str;
}

// ファイルシステム抽象化（テスト容易性）
trait FileSystem {
    fn create_dir(&self, path: &Path) -> Result<()>;
    fn write_file(&self, path: &Path, content: &str) -> Result<()>;
    fn exists(&self, path: &Path) -> bool;
}

// 実行コンテキスト
struct Context {
    config: Config,
    filesystem: Box<dyn FileSystem>,
    logger: Box<dyn Logger>,
}
```

#### エラーハンドリング戦略
```rust
#[derive(Error, Debug)]
pub enum HailMaryError {
    #[error("Feature '{0}' already exists")]
    FeatureAlreadyExists(String),
    
    #[error("Invalid feature name: {0}. Must be kebab-case")]
    InvalidFeatureName(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, HailMaryError>;
```

## 実装詳細

### Phase 1: MVP実装

#### 1. プロジェクト構造
```
src/
├── main.rs           # エントリーポイント
├── commands/
│   ├── mod.rs
│   └── new.rs        # newコマンド実装
├── core/
│   ├── mod.rs
│   └── project.rs    # プロジェクト操作
└── utils/
    ├── mod.rs
    ├── error.rs      # エラー型
    └── validator.rs  # バリデーション
```

#### 2. 主要実装タスク
- [ ] CLI引数パースの実装
- [ ] kebab-case検証ロジック
- [ ] ディレクトリ/ファイル生成処理
- [ ] 日付フォーマット処理
- [ ] エラーハンドリング
- [ ] 単体テストの作成

#### 3. 検証項目
- [ ] 正常系：新規機能作成
- [ ] 異常系：重複機能名
- [ ] 異常系：不正な機能名
- [ ] エッジケース：権限不足等

### Phase 2以降の実装内容

#### Phase 2: アーキテクチャリファクタリング
- トレイトベース設計への移行
- 依存性注入の実装
- モックを使った単体テスト
- 統合テストの追加

#### Phase 3: 機能拡張準備
- 設定管理システム（TOML/YAML）
- テンプレートシステム
- フック機構の基盤
- 非同期処理の導入

#### Phase 4: エンタープライズ機能
- WebAssemblyプラグインシステム
- 外部ツール統合（Git、CI/CD）
- 高度な設定階層管理
- パフォーマンス監視・最適化

### テスト戦略

#### 単体テスト
- 各コマンドのロジックテスト
- バリデーション機能のテスト
- エラーケースの網羅的テスト
- FileSystemトレイトのモック使用

#### 統合テスト
- CLI引数から実行完了までのE2E
- 実際のファイルシステムでの動作確認
- tempfileクレートで一時ディレクトリ使用

#### プロパティベーステスト
- kebab-case検証のファジング
- ファイル名生成の境界値テスト
- proptestクレートの活用

## 将来的な拡張性

### 設定管理
- **階層的設定**（優先順位順）：
  1. コマンドライン引数
  2. 環境変数（HAIL_MARY_*）
  3. プロジェクトローカル設定（.kiro/config.toml）
  4. グローバル設定（~/.config/hail-mary/config.toml）
  5. デフォルト値

### プラグインシステム（Phase 4）
- WebAssembly（WASI）による安全なプラグイン実行
- プラグインAPI定義とバージョニング
- プラグインレジストリの構築

### パフォーマンス最適化
- 遅延読み込みによるコマンド実装
- tokioによる非同期処理
- rayonによる並列処理
- 設定ファイルのキャッシュ機構

## 注意事項
- **段階的実装**: 小さく始めて段階的に成長
- **後方互換性**: 既存機能の互換性維持
- **テスト駆動**: 各Phase でテスト充実化
- **ドキュメント**: 仕様変更時の文書更新
- **セキュリティ**: 入力検証とエラーハンドリングの徹底