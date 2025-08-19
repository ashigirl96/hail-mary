# Clean Architecture リファクタリング実装タスクプラン

## 概要

design-v3.mdに基づいて、hail-maryプロジェクトをClean Architectureに移行するための詳細な実装計画です。

### 実装方針
- **段階的移行**: 既存機能を維持しながら段階的に移行
- **並行作業**: 独立したタスクは並行実行で効率化
- **早期テスト**: 各層の実装直後にテスト作成
- **品質重視**: コードレビューとテストカバレッジ維持

### 全体見積もり
- **総工数**: 12-20時間（並行作業による短縮後）
- **期間**: 2-3日間
- **難易度**: 中程度

## タスク一覧

### Phase 1: Domain Layer 実装 [4-6時間]

#### Task 1.1: KiroConfigビジネスルール追加
- **優先度**: P0（ブロッカー）
- **見積もり**: S（1-2時間）
- **依存**: なし
- **担当者**: -
- **詳細**:
  ```rust
  // src/models/kiro_config.rs に追加
  - generate_feature_dir_name()
  - default_feature_files()
  - required_directories()
  - default_gitignore_entries()
  - config_template()
  ```
- **受け入れ基準**:
  - [x] 全メソッドが実装済み
  - [x] ユニットテスト作成
  - [x] ドキュメント追加

#### Task 1.2: KiroFeatureエンティティ作成
- **優先度**: P0（ブロッカー）
- **見積もり**: S（1-2時間）
- **依存**: なし
- **担当者**: -
- **詳細**:
  ```rust
  // src/models/kiro_feature.rs 新規作成
  - エンティティ定義
  - バリデーションロジック
  - ファクトリメソッド
  ```
- **受け入れ基準**:
  - [x] is_valid_name()実装
  - [x] can_create()実装
  - [x] serde対応

#### Task 1.3: Domain層ユニットテスト
- **優先度**: P1
- **見積もり**: S（1-2時間）
- **依存**: Task 1.1, 1.2
- **担当者**: -
- **詳細**:
  - KiroConfigのビジネスルールテスト
  - KiroFeatureのバリデーションテスト
  - エッジケーステスト
- **受け入れ基準**:
  - [x] カバレッジ100%
  - [x] 全テスト合格

### Phase 2: Infrastructure Layer 実装 [6-10時間]

#### Task 2.1: ProjectRepositoryトレイト定義
- **優先度**: P0（ブロッカー）
- **見積もり**: S（1-2時間）
- **依存**: Task 1.1, 1.2
- **担当者**: -
- **詳細**:
  ```rust
  // src/repositories/project.rs
  pub trait ProjectRepository {
      fn initialize_structure(&self, config: &KiroConfig) -> Result<()>;
      fn save_feature(&self, feature: &KiroFeature) -> Result<PathBuf>;
      fn find_feature_by_name(&self, name: &str) -> Result<Option<KiroFeature>>;
      fn list_all_features(&self) -> Result<Vec<KiroFeature>>;
      fn save_config(&self, config: &KiroConfig) -> Result<()>;
      fn load_config(&self) -> Result<KiroConfig>;
      fn find_kiro_root(&self) -> Result<PathBuf>;
      fn update_gitignore(&self, entries: &[String]) -> Result<()>;
  }
  ```
- **受け入れ基準**:
  - [x] トレイト定義完了
  - [x] ドキュメント追加

#### Task 2.2: FileProjectRepository実装
- **優先度**: P0
- **見積もり**: M（2-4時間）
- **依存**: Task 2.1
- **担当者**: -
- **詳細**:
  - std::fs直接使用（FileSystemトレイトなし）
  - .kiroディレクトリ構造の管理
  - spec.jsonのシリアライゼーション
- **受け入れ基準**:
  - [x] 全メソッド実装
  - [x] エラーハンドリング
  - [x] 統合テスト作成

#### Task 2.3: InMemoryProjectRepository実装
- **優先度**: P1
- **見積もり**: M（2-4時間）
- **依存**: Task 2.1
- **担当者**: -
- **詳細**:
  - テスト用インメモリ実装
  - HashMap/Arc/Mutexベース
  - ファイルシステム操作のシミュレーション
- **受け入れ基準**:
  - [x] 全メソッド実装
  - [x] テストヘルパー作成
  - [x] ドキュメント追加

#### Task 2.4: Repository層テスト
- **優先度**: P1
- **見積もり**: M（2-4時間）
- **依存**: Task 2.2, 2.3
- **担当者**: -
- **詳細**:
  - FileProjectRepositoryの統合テスト
  - InMemoryRepositoryのユニットテスト
  - 両実装の振る舞い一致確認
- **受け入れ基準**:
  - [x] カバレッジ80%以上
  - [x] tempfileでの実テスト

### Phase 3: Application Layer 実装 [4-6時間]

#### Task 3.1: ProjectService実装
- **優先度**: P0
- **見積もり**: M（2-4時間）
- **依存**: Task 2.1
- **担当者**: -
- **詳細**:
  ```rust
  // src/services/project.rs
  pub struct ProjectService<R: ProjectRepository> {
      repository: R,
      config: KiroConfig,
  }
  - initialize_project()
  - create_new_feature()
  - list_features()
  - find_feature()
  ```
- **受け入れ基準**:
  - [x] 全ユースケース実装
  - [x] 依存性注入対応
  - [x] エラーハンドリング

#### Task 3.2: ProjectManagerからロジック移行
- **優先度**: P1
- **見積もり**: S（1-2時間）
- **依存**: Task 3.1
- **担当者**: -
- **詳細**:
  - 既存ProjectManagerのロジック分析
  - ProjectServiceへの移行
  - 互換性確認
- **受け入れ基準**:
  - [x] 全機能移行完了
  - [x] リグレッションなし

#### Task 3.3: Service層テスト
- **優先度**: P1
- **見積もり**: S（1-2時間）
- **依存**: Task 3.1, 3.2
- **担当者**: -
- **詳細**:
  - MockRepositoryでのユニットテスト
  - ビジネスロジックのテスト
  - エラーケーステスト
- **受け入れ基準**:
  - [x] カバレッジ90%以上
  - [x] 全シナリオテスト

### Phase 4: 移行と統合 [3-5時間]

#### Task 4.1: Commands層更新
- **優先度**: P0
- **見積もり**: S（1-2時間）
- **依存**: Task 3.1
- **担当者**: -
- **詳細**:
  ```rust
  // src/commands/init.rs
  // src/commands/new.rs
  - ProjectManagerからProjectServiceへ切り替え
  - 依存性注入の実装
  ```
- **受け入れ基準**:
  - [x] CLIコマンド動作確認
  - [x] エラーメッセージ確認

#### Task 4.2: E2E統合テスト
- **優先度**: P1
- **見積もり**: M（2-3時間）
- **依存**: Task 4.1
- **担当者**: -
- **詳細**:
  - 全ワークフローのテスト
  - 実ファイルシステムでのテスト
  - CLIコマンドのテスト
- **受け入れ基準**:
  - [x] 全機能動作確認
  - [x] パフォーマンステスト合格

#### Task 4.3: ProjectManager削除とクリーンアップ
- **優先度**: P2
- **見積もり**: S（1時間）
- **依存**: Task 4.2
- **担当者**: -
- **詳細**:
  - src/core/project.rs削除
  - 不要なインポート削除
  - ドキュメント更新
- **受け入れ基準**:
  - [x] ビルド成功
  - [x] 全テスト合格
  - [x] clippy警告なし

## 依存関係図

```
Task 1.1 ─┐
          ├─→ Task 1.3
Task 1.2 ─┘    │
               ↓
Task 2.1 ─────────→ Task 2.2 ─┐
     │                         ├─→ Task 2.4
     └──────────→ Task 2.3 ───┘
     │
     ↓
Task 3.1 ─→ Task 3.2 ─→ Task 3.3
     │
     ↓
Task 4.1 ─→ Task 4.2 ─→ Task 4.3
```

## 並行実行計画

### Day 1
- **Morning**: 
  - Task 1.1, 1.2（並行）
  - Task 1.3
- **Afternoon**:
  - Task 2.1
  - Task 2.2, 2.3（並行）

### Day 2
- **Morning**:
  - Task 2.4
  - Task 3.1
- **Afternoon**:
  - Task 3.2, 3.3
  - Task 4.1

### Day 3
- **Morning**:
  - Task 4.2
  - Task 4.3
- **Afternoon**:
  - 最終確認とドキュメント

## リスク管理

### リスク1: 既存機能の破損
- **影響度**: 高
- **発生確率**: 中
- **軽減策**:
  - Feature flagで新旧切り替え
  - 既存テストの充実
  - 段階的移行

### リスク2: データ互換性
- **影響度**: 中
- **発生確率**: 低
- **軽減策**:
  - spec.jsonフォーマット維持
  - マイグレーションスクリプト

### リスク3: パフォーマンス劣化
- **影響度**: 低
- **発生確率**: 低
- **軽減策**:
  - ベンチマークテスト
  - プロファイリング

## 成功基準

### 機能要件
- [ ] 全CLIコマンドが動作
- [ ] 既存機能の維持
- [ ] エラーハンドリング統一

### 非機能要件
- [ ] テストカバレッジ80%以上
- [ ] パフォーマンス劣化なし
- [ ] コードレビュー完了

### 品質基準
- [ ] cargo fmt合格
- [ ] cargo clippy警告なし
- [ ] ドキュメント更新完了

## チェックポイント

### Milestone 1: Domain Layer完成
- **期日**: Day 1 AM
- **成果物**: KiroConfig, KiroFeature
- **レビュー**: ビジネスルール確認

### Milestone 2: Infrastructure Layer完成
- **期日**: Day 1 PM
- **成果物**: Repository実装
- **レビュー**: 永続化戦略確認

### Milestone 3: Application Layer完成
- **期日**: Day 2 PM
- **成果物**: ProjectService
- **レビュー**: ユースケース確認

### Milestone 4: 移行完了
- **期日**: Day 3 AM
- **成果物**: 完全動作するシステム
- **レビュー**: 最終確認

## 技術スタック

### 必須クレート
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
thiserror = "1.0"
anyhow = "1.0"

[dev-dependencies]
tempfile = "3.0"
pretty_assertions = "1.0"
```

### コーディング規約
- Rust 2021 Edition
- cargo fmt必須
- cargo clippy -D warnings
- ドキュメントコメント必須

## ドキュメント更新

### 更新対象
1. CLAUDE.md - 新アーキテクチャの説明
2. README.md - 使用方法（変更なし）
3. API仕様書 - Repository/Service層

### 作成対象
1. アーキテクチャ図
2. シーケンス図
3. 移行ガイド

## 完了定義

タスクが「完了」と見なされる条件：

1. **コード完了**
   - 実装完了
   - テスト作成
   - ドキュメント追加

2. **品質確認**
   - cargo fmt実行
   - cargo clippy合格
   - テスト全合格

3. **レビュー完了**
   - セルフレビュー
   - コードレビュー
   - 動作確認

## 次のステップ

1. このタスクプランのレビューと承認
2. Task 1.1, 1.2から実装開始
3. 日次進捗確認
4. 完了後の振り返り

---

*このドキュメントは実装の進捗に応じて更新されます。*