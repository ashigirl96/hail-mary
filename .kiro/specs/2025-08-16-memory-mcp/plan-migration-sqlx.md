# SQLx移行計画書

## 概要

Hail-Maryプロジェクトを現在の`rusqlite`から`SQLx`に移行するための包括的な計画書です。

## 移行の背景と理由

### 現在の問題点
1. **多数のコンパイルエラー**: `topic` → `title`フィールド名変更、`source`フィールド削除による不整合
2. **マイグレーション管理の複雑さ**: `rusqlite_migration`の機能が限定的
3. **テスト困難性**: データベーステストの設定が複雑
4. **非同期サポートなし**: rusqliteは同期的なAPIのみ

### SQLx移行のメリット
1. **強力なCLIツール**: `sqlx migrate add/run/revert/info`
2. **コンパイル時SQL検証**: `query!`マクロによる型安全性
3. **非同期サポート**: tokioベースの非同期処理
4. **自動テスト統合**: `#[sqlx::test]`で自動マイグレーション
5. **可逆マイグレーション**: up/downスクリプトの自動生成

## 移行スコープ

### フェーズ1: 基盤整備（1週間）
- SQLx依存関係の追加
- 基本的なデータベース接続の確立
- マイグレーションファイルの移行

### フェーズ2: コア機能移行（2週間）
- Repository層の書き換え
- MCPサーバーの非同期化
- 基本的なCRUD操作の移行

### フェーズ3: 高度な機能（1週間）
- FTS5全文検索の統合
- 埋め込みベクトル機能
- バッチ処理の最適化

### フェーズ4: テストと最適化（1週間）
- 包括的なテストスイート
- パフォーマンス最適化
- ドキュメント更新

## 技術的詳細

### 1. 依存関係の更新

#### Cargo.toml
```toml
[dependencies]
# Remove
# rusqlite = { version = "0.31", features = ["bundled", "vtab"] }
# rusqlite_migration = "1.0"

# Add
sqlx = { version = "0.8", features = [
    "runtime-tokio",
    "tls-native-tls", 
    "sqlite",
    "migrate",
    "macros",
    "chrono",
    "uuid"
]}
tokio = { version = "1", features = ["full"] }
```

### 2. データベース接続層

#### 現在のコード（rusqlite）
```rust
use rusqlite::{Connection, params};

pub struct MemoryRepository {
    db_path: PathBuf,
}

impl MemoryRepository {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let mut conn = Connection::open(&db_path)?;
        initialize_database(&mut conn)?;
        Ok(Self { db_path })
    }
}
```

#### 移行後のコード（SQLx）
```rust
use sqlx::{SqlitePool, SqlitePoolOptions};

pub struct MemoryRepository {
    pool: SqlitePool,
}

impl MemoryRepository {
    pub async fn new(db_path: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&format!("sqlite:{}", db_path))
            .await?;
        
        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await?;
        
        Ok(Self { pool })
    }
}
```

### 3. マイグレーション管理

#### ディレクトリ構造
```
migrations/
├── 001_initial_schema.up.sql
├── 001_initial_schema.down.sql
├── 002_add_fts5.up.sql
├── 002_add_fts5.down.sql
├── 003_add_embeddings.up.sql
└── 003_add_embeddings.down.sql
```

#### CLIコマンド
```bash
# 新しいマイグレーション作成
sqlx migrate add -r initial_schema

# マイグレーション実行
sqlx migrate run

# マイグレーション情報確認
sqlx migrate info

# マイグレーション巻き戻し
sqlx migrate revert
```

### 4. FTS5サポート

#### 現在のFTS5実装（rusqlite）
```rust
conn.execute(
    "CREATE VIRTUAL TABLE memories_fts USING fts5(
        memory_id UNINDEXED,
        title,
        tags,
        content,
        tokenize = 'porter unicode61'
    )",
    [],
)?;
```

#### SQLxでのFTS5実装
```rust
// マイグレーションファイルで定義
-- migrations/002_add_fts5.up.sql
CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
    memory_id UNINDEXED,
    title,
    tags,
    content,
    tokenize = 'porter unicode61'
);

// Rustコードでのクエリ
pub async fn search_memories(
    pool: &SqlitePool,
    query: &str,
    limit: i64
) -> Result<Vec<Memory>> {
    let results = sqlx::query_as!(
        Memory,
        r#"
        SELECT m.id, m.type as "memory_type: MemoryType",
               m.title, m.tags, m.content, m.examples,
               m.reference_count, m.confidence,
               m.created_at, m.last_accessed, m.deleted
        FROM memories m
        JOIN memories_fts fts ON m.id = fts.memory_id
        WHERE memories_fts MATCH ?1 AND m.deleted = 0
        ORDER BY bm25(memories_fts)
        LIMIT ?2
        "#,
        format!("{}*", query),
        limit
    )
    .fetch_all(pool)
    .await?;
    
    Ok(results)
}
```

### 5. 型安全なクエリ

#### 現在のクエリ（rusqlite）
```rust
let mut stmt = conn.prepare(
    "SELECT * FROM memories WHERE type = ? AND deleted = 0"
)?;
let memories = stmt.query_map(params![memory_type.to_string()], |row| {
    Memory::from_row(row)
})?;
```

#### SQLxの型安全クエリ
```rust
// コンパイル時にSQLが検証される
let memories = sqlx::query_as!(
    Memory,
    r#"
    SELECT id, type as "memory_type: MemoryType", title, 
           tags, content, examples, reference_count, 
           confidence, created_at, last_accessed, deleted
    FROM memories 
    WHERE type = ?1 AND deleted = 0
    "#,
    memory_type.to_string()
)
.fetch_all(&pool)
.await?;
```

### 6. テスト統合

#### SQLxテストマクロ
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;
    
    #[sqlx::test]
    async fn test_create_memory(pool: SqlitePool) -> sqlx::Result<()> {
        let repo = MemoryRepository::new_with_pool(pool).await?;
        
        let memory = Memory::new(
            MemoryType::Tech,
            "Test Title".to_string(),
            "Test Content".to_string()
        );
        
        let id = repo.create_memory(memory).await?;
        assert!(!id.is_empty());
        
        let retrieved = repo.get_memory(&id).await?;
        assert_eq!(retrieved.title, "Test Title");
        
        Ok(())
    }
}
```

## 移行手順

### ステップ1: 環境準備
```bash
# SQLx CLIインストール
cargo install sqlx-cli --no-default-features --features sqlite,native-tls

# データベースURL設定
export DATABASE_URL="sqlite:.kiro/memory/memory.db"
```

### ステップ2: 初期マイグレーション作成
```bash
# 既存のSQLファイルを可逆マイグレーションに変換
sqlx migrate add -r initial_schema
# 001_initial_schema.up.sql と 001_initial_schema.down.sql が作成される

# FTS5マイグレーション
sqlx migrate add -r add_fts5

# 埋め込みテーブルマイグレーション
sqlx migrate add -r add_embeddings
```

### ステップ3: コード移行

1. **Repository層の書き換え**
   - 同期メソッドを非同期メソッドに変換
   - `Connection`を`SqlitePool`に置換
   - `params!`を`query!`マクロに置換

2. **MCPサーバーの非同期化**
   - `tokio::spawn`で非同期タスク管理
   - `async`/`await`パターンの適用

3. **エラーハンドリング**
   - `rusqlite::Error`を`sqlx::Error`に置換
   - Result型の調整

### ステップ4: テスト移行

```rust
// テスト用の設定
#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    
    async fn setup_test_db() -> SqlitePool {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let pool = SqlitePool::connect(&format!("sqlite:{}", db_path.display()))
            .await
            .unwrap();
        
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .unwrap();
        
        pool
    }
}
```

## リスクと対策

### リスク1: FTS5バーチャルテーブルの互換性
- **問題**: SQLxでのバーチャルテーブル型情報の欠如
- **対策**: 生SQL実行とカスタム型マッピング

### リスク2: パフォーマンス劣化
- **問題**: 非同期オーバーヘッド
- **対策**: 接続プール最適化、バッチ処理

### リスク3: 既存データの移行
- **問題**: データ損失リスク
- **対策**: バックアップ機能、段階的移行

## タイムライン

### 週1: 基盤整備
- [ ] SQLx依存関係追加
- [ ] 基本的な接続確立
- [ ] マイグレーションファイル作成

### 週2-3: コア機能移行
- [ ] Repository層の書き換え
- [ ] CRUD操作の移行
- [ ] FTS5統合

### 週4: 高度な機能
- [ ] 埋め込み機能
- [ ] バッチ処理
- [ ] パフォーマンス最適化

### 週5: テストと文書化
- [ ] 包括的テスト
- [ ] ベンチマーク
- [ ] ドキュメント更新

## 成功指標

1. **機能的完全性**: 既存機能の100%移行
2. **パフォーマンス**: 現行比±10%以内
3. **テストカバレッジ**: 80%以上
4. **コンパイル時検証**: 全SQLクエリの型検証
5. **開発体験**: マイグレーション管理の簡素化

## 結論

SQLxへの移行により、以下の改善が期待できます：

1. **開発効率の向上**: 強力なCLIツールとコンパイル時検証
2. **保守性の改善**: 型安全性と自動テスト
3. **将来性**: 非同期処理とモダンなRustエコシステム
4. **信頼性**: 可逆マイグレーションとトランザクション管理

移行は段階的に実施し、各フェーズで動作確認を行いながら進めることで、リスクを最小化します。