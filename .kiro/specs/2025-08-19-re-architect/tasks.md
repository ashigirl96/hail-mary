# Implementation Tasks - Hail-Mary Memory MCP v3

## Overview

Design-v3.mdのMigration Planに従い、Clean Architecture + TDDで全39ファイルを段階的に実装します。
各ファイル内で`#[cfg(test)]`モジュールを使用したテスト先行開発を行います。

## TDDサイクル
各ファイルで以下のサイクルを実行:
1. **Red**: テストを書く（失敗する）
2. **Green**: 最小限の実装でテストを通す
3. **Refactor**: コードを改善する

---

## Phase 1: Domain Layer (7ファイル)

### 1.1 Domain Errors
- [x] `src/domain/errors.rs` - テスト先行
  - `#[cfg(test)] mod tests`でエラー型テスト作成
- [x] `src/domain/errors.rs` - 実装追加
  - DomainError enumの実装

### 1.2 Value Objects
- [x] `src/domain/value_objects/confidence.rs` - テスト先行
  - バリデーションテスト（0.0-1.0範囲チェック）
- [x] `src/domain/value_objects/confidence.rs` - 実装追加
  - Confidence構造体とバリデーション実装
- [x] `src/domain/value_objects/mod.rs`

### 1.3 Entities
- [x] `src/domain/entities/memory.rs` - テスト先行
  - Memory作成・操作テスト
- [x] `src/domain/entities/memory.rs` - 実装追加
  - Memory構造体とメソッド実装
- [x] `src/domain/entities/project.rs` - テスト先行
  - ProjectConfig設定検証テスト
- [x] `src/domain/entities/project.rs` - 実装追加
  - ProjectConfig構造体実装
- [x] `src/domain/entities/mod.rs`

---

## Phase 2: Application Layer (11ファイル)

### 2.1 Application Errors
- [x] `src/application/errors.rs` - テスト先行
  - ApplicationErrorのテスト作成
- [x] `src/application/errors.rs` - 実装追加
  - ApplicationError enum実装

### 2.2 Repository Traits (先にtrait定義)
- [x] `src/application/repositories/memory_repository.rs` - テスト先行
  - Mockを使用したtrait動作テスト
- [x] `src/application/repositories/memory_repository.rs` - 実装追加
  - MemoryRepository trait定義
- [x] `src/application/repositories/project_repository.rs` - テスト先行
  - Mockを使用したtrait動作テスト
- [x] `src/application/repositories/project_repository.rs` - 実装追加
  - ProjectRepository trait定義
- [x] `src/application/repositories/mod.rs`

### 2.3 Use Cases (trait使用)
- [x] `src/application/use_cases/initialize_project.rs` - テスト先行
  - プロジェクト初期化ロジックテスト
- [x] `src/application/use_cases/initialize_project.rs` - 実装追加
  - initialize_project関数実装
- [x] `src/application/use_cases/create_feature.rs` - テスト先行
  - 機能作成ロジックテスト
- [x] `src/application/use_cases/create_feature.rs` - 実装追加
  - create_feature関数実装
- [x] `src/application/use_cases/remember_memory.rs` - テスト先行
  - メモリ保存ロジックテスト
- [x] `src/application/use_cases/remember_memory.rs` - 実装追加
  - remember_memory関数実装
- [x] `src/application/use_cases/recall_memory.rs` - テスト先行
  - メモリ検索ロジックテスト
- [x] `src/application/use_cases/recall_memory.rs` - 実装追加
  - recall_memory関数実装
- [x] `src/application/use_cases/generate_document.rs` - テスト先行
  - ドキュメント生成テスト
- [x] `src/application/use_cases/generate_document.rs` - 実装追加
  - generate_document関数実装
- [x] `src/application/use_cases/reindex_memories.rs` - テスト先行
  - データベース最適化テスト
- [x] `src/application/use_cases/reindex_memories.rs` - 実装追加
  - reindex_memories関数実装
- [x] `src/application/use_cases/mod.rs`

---

## Phase 3: Infrastructure Layer (9ファイル)

### 3.1 Filesystem
- [x] `src/infrastructure/filesystem/path_manager.rs` - テスト先行
  - パス解決テスト（tempdir使用）
- [x] `src/infrastructure/filesystem/path_manager.rs` - 実装追加
  - PathManager構造体実装
- [x] `src/infrastructure/filesystem/mod.rs`

### 3.2 Database Migrations
- [x] `src/infrastructure/migrations/embedded.rs` - テスト先行
  - マイグレーション実行テスト
- [x] `src/infrastructure/migrations/embedded.rs` - 実装追加
  - Refineryマイグレーション実装
- [x] `src/infrastructure/migrations/mod.rs`

### 3.3 Repository Implementations (trait実装)
- [ ] `src/infrastructure/repositories/memory.rs` - テスト先行
  - SQLite統合テスト（tempdb使用）
- [ ] `src/infrastructure/repositories/memory.rs` - 実装追加
  - SqliteMemoryRepository構造体実装
- [ ] `src/infrastructure/repositories/project.rs` - テスト先行
  - ファイルシステム統合テスト（tempdir使用）
- [ ] `src/infrastructure/repositories/project.rs` - 実装追加
  - ProjectRepository具体実装
- [ ] `src/infrastructure/repositories/mod.rs`

### 3.4 MCP Server
- [ ] `src/infrastructure/mcp/server.rs` - テスト先行
  - MCPプロトコル通信テスト
- [ ] `src/infrastructure/mcp/server.rs` - 実装追加
  - MemoryMcpServer構造体実装
- [ ] `src/infrastructure/mcp/mod.rs`

---

## Phase 4: CLI Layer (7ファイル)

### 4.1 基盤コンポーネント
- [ ] `src/cli/args.rs` - テスト先行
  - 引数解析テスト
- [ ] `src/cli/args.rs` - 実装追加
  - Clap構造体定義
- [ ] `src/cli/formatters.rs` - テスト先行
  - 出力フォーマットテスト
- [ ] `src/cli/formatters.rs` - 実装追加
  - フォーマッター関数実装

### 4.2 Command Implementations
- [ ] `src/cli/commands/init.rs` - テスト先行
  - initコマンド統合テスト
- [ ] `src/cli/commands/init.rs` - 実装追加
  - InitCommand構造体実装
- [ ] `src/cli/commands/new.rs` - テスト先行
  - newコマンド統合テスト
- [ ] `src/cli/commands/new.rs` - 実装追加
  - NewCommand構造体実装
- [ ] `src/cli/commands/memory.rs` - テスト先行
  - memoryサブコマンド統合テスト
- [ ] `src/cli/commands/memory.rs` - 実装追加
  - MemoryCommand構造体実装
- [ ] `src/cli/commands/mod.rs`
- [ ] `src/cli/mod.rs`

---

## Phase 5: Integration (5ファイル)

### 5.1 Database Schema
- [ ] `migrations/V001__initial_schema.sql`
  - 基本テーブル構造
- [ ] `migrations/V002__add_fts5_index.sql`
  - FTS5仮想テーブル
- [ ] `migrations/V003__add_triggers.sql`
  - 自動同期トリガー

### 5.2 Final Integration
- [ ] `src/lib.rs` - 全モジュールエクスポート
- [ ] `src/main.rs` - DI設定 + E2Eテスト追加
  - `#[cfg(test)] mod integration_tests`でE2Eテスト

---

## 進捗管理

**合計**: 39ファイル（78タスク: テスト先行 + 実装）

### Phase別進捗
- [ ] Phase 1: Domain Layer (7ファイル・14タスク)
- [ ] Phase 2: Application Layer (11ファイル・22タスク)
- [ ] Phase 3: Infrastructure Layer (9ファイル・18タスク)
- [ ] Phase 4: CLI Layer (7ファイル・14タスク)
- [ ] Phase 5: Integration (5ファイル・10タスク)

### TDD品質チェック
各Phase完了時:
- [ ] すべてのテストがパスしている
- [ ] テストカバレッジ >= 80%
- [ ] コードレビュー完了
- [ ] リファクタリング実施済み

---

## 実装時の注意点

### 依存関係順序
1. **Domain**: errors → value_objects → entities
2. **Application**: errors + repository traits → use cases
3. **Infrastructure**: filesystem → migrations → repositories → mcp
4. **CLI**: args/formatters → commands
5. **Integration**: migrations → lib.rs → main.rs

### テスト戦略
- **Unit Tests**: Domain/Application layer
- **Integration Tests**: Infrastructure layer（実DB/ファイル使用）
- **E2E Tests**: CLIコマンドのフルワークフロー

### TDD実装例
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_confidence_valid_range() {
        // Red: 失敗するテストを書く
        assert!(Confidence::new(0.5).is_ok());
        assert!(Confidence::new(1.5).is_err());
    }
}

// Green: テストを通す最小実装
pub struct Confidence(f32);
impl Confidence {
    pub fn new(value: f32) -> Result<Self, DomainError> {
        if value < 0.0 || value > 1.0 {
            return Err(DomainError::InvalidConfidence(value));
        }
        Ok(Self(value))
    }
}

// Refactor: 必要に応じてコード改善
```