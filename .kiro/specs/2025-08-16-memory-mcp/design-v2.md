# Memory MCP v2 設計仕様書

## 1. 概要

### 1.1 コンセプト
Memory MCP v2は、**シンプルさと実用性**を最優先した永続的メモリシステムです。過度な複雑性を避け、段階的に価値を提供できる設計を採用しています。

### 1.2 設計原則
- **KISS (Keep It Simple, Stupid)**: 最小限の機能から始める
- **YAGNI (You Aren't Gonna Need It)**: 今必要ないものは作らない
- **疎結合**: 各機能が独立して動作可能
- **進化的アーキテクチャ**: 実際の使用に基づいて成長

### 1.3 主要な特徴
- 📝 **シンプルなデータモデル**: 単一テーブル + FTS5
- 🔍 **高速な全文検索**: SQLite FTS5による日本語対応検索
- 📚 **ドキュメント生成**: Markdown形式で記憶を整理
- 🔄 **定期的な最適化**: reindex機能による重複排除と再構築
- 🏷️ **記憶の分類**: tech / project-tech / domain の3カテゴリ
- 🚀 **段階的実装**: 各フェーズで動く価値を提供

## 2. アーキテクチャ

### 2.0 技術選定の根拠

#### SQLite + rusqlite の選択理由

**なぜDieselではなくrusqliteなのか？**

1. **FTS5との完全な統合**
   - Memory MCPの中核機能である全文検索にFTS5が必須
   - DieselはFTS5を直接サポートしていない（`sql_query`での回避策が必要）
   - rusqliteはFTS5とシームレスに統合

2. **シンプルな構造に適合**
   - テーブルが1つだけの単純な構造
   - 複雑なリレーションがない
   - ORMのオーバーヘッドが不要

3. **開発速度**
   - Phase 1（2-3日）での迅速な実装が可能
   - 学習曲線が緩やか
   - FTS5統合に追加作業が不要

4. **マイグレーション管理**
   - `rusqlite_migration`クレートで十分な管理が可能
   - 将来Dieselへの移行も可能な設計

**型安全性の補完策**:
```rust
// SQLクエリを定数化して管理
const INSERT_MEMORY: &str = "INSERT INTO memories ...";
const SEARCH_FTS: &str = "SELECT * FROM memories_fts ...";

// Repository層で型安全なインターフェースを提供
trait MemoryRepository {
    fn save(&mut self, memory: &Memory) -> Result<()>;
    fn search(&self, query: &str) -> Result<Vec<Memory>>;
}
```

### 2.1 全体構成

```mermaid
%%{init: {
  'theme': 'dark',
  'themeCSS': '
    .nodeLabel { color: #FD971F !important; }
    .edgeLabel { color: #A6E22E !important; background-color: transparent !important; }
    .cluster rect { fill: #272822 !important; stroke: #F92672 !important; stroke-width: 2px !important; rx: 5px !important; ry: 5px !important; }
    .node rect, .node circle, .node ellipse, .node polygon, .node path { fill: #272822 !important; stroke: #A6E22E !important; stroke-width: 2px !important; }
    .flowchart-link { stroke: #66D9EF !important; stroke-width: 2px !important; }
  '
}}%%
graph TB
    subgraph "User Interface"
        A[Claude Code<br/>MCP Client]
        B[CLI<br/>hail-mary]
        C[Generated Docs<br/>*.md files]
    end
    
    subgraph "Application Layer"
        D[MCP Server<br/>remember/recall]
        E[Document Generator<br/>markdown export]
        F[Reindexer<br/>optimization]
    end
    
    subgraph "Data Layer"
        G[(SQLite DB<br/>memory.db)]
        H[FTS5 Index]
        I[Archive<br/>old DBs]
    end
    
    A -.->|stdio| D
    B --> D
    B --> E
    B --> F
    
    D --> G
    E --> G
    F --> G
    
    G --> H
    F --> I
    E --> C
    
    A -.->|@tech.md| C
    
    classDef type1 fill:#272822,stroke:#A6E22E,stroke-width:2px;
    classDef type2 fill:#272822,stroke:#66D9EF,stroke-width:2px;
    classDef type3 fill:#272822,stroke:#F92672,stroke-width:2px;
    classDef highlighted fill:#AE81FF,stroke:#66D9EF,stroke-width:3px,color:#FFF;
    
    class A,B,C type1;
    class D,E,F type2;
    class G,H,I highlighted;
```

### 2.2 コンポーネント説明

#### Application Layer
- **MCP Server**: remember/recallツールを提供するMCPサーバー
- **Document Generator**: 記憶をMarkdown形式でエクスポート
- **Reindexer**: 定期的な最適化と重複排除（Phase 3で実装）

#### Data Layer
- **SQLite DB**: すべての記憶を保存する単一データベース
- **FTS5 Index**: 高速全文検索インデックス
- **Archive**: reindex時の旧データベース保存

## 3. データベース設計

### 3.1 最小限のスキーマ

```sql
-- メインテーブル（これだけ！）
CREATE TABLE memories (
    id TEXT PRIMARY KEY,              -- UUID v4
    type TEXT NOT NULL CHECK(         -- 記憶のカテゴリ
        type IN ('tech', 'project-tech', 'domain')
    ),
    topic TEXT NOT NULL,              -- タイトル/要約（人間が読みやすい）
    tags TEXT,                        -- カンマ区切りのタグ（例: "rust,async,tokio"）
    content TEXT NOT NULL,            -- 本文
    examples TEXT,                    -- JSON配列でコード例などを保存
    reference_count INTEGER DEFAULT 0, -- 参照された回数
    confidence REAL DEFAULT 1.0       -- 信頼度スコア (0.0-1.0)
        CHECK(confidence >= 0 AND confidence <= 1),
    created_at INTEGER DEFAULT (unixepoch()), -- 作成日時
    last_accessed INTEGER,            -- 最終アクセス日時
    source TEXT,                      -- 情報源（オプション）
    deleted INTEGER DEFAULT 0         -- 論理削除フラグ
);

-- FTS5全文検索インデックス
CREATE VIRTUAL TABLE memories_fts USING fts5(
    memory_id UNINDEXED,              -- 検索対象外
    topic,                            -- 検索対象
    tags,                             -- 検索対象
    content,                          -- 検索対象
    tokenize = 'porter unicode61'     -- 日本語対応トークナイザー
);

-- 必要最小限のインデックス
CREATE INDEX idx_memories_type 
    ON memories(type) 
    WHERE deleted = 0;

CREATE INDEX idx_memories_ref_count 
    ON memories(reference_count DESC) 
    WHERE deleted = 0;

CREATE INDEX idx_memories_created 
    ON memories(created_at DESC) 
    WHERE deleted = 0;

-- トリガー: FTS5インデックスの自動更新
CREATE TRIGGER memories_ai AFTER INSERT ON memories
WHEN NEW.deleted = 0
BEGIN
    INSERT INTO memories_fts(memory_id, topic, tags, content)
    VALUES (NEW.id, NEW.topic, NEW.tags, NEW.content);
END;

CREATE TRIGGER memories_au AFTER UPDATE ON memories
WHEN NEW.deleted = 0 AND OLD.deleted = 0
BEGIN
    UPDATE memories_fts 
    SET topic = NEW.topic, tags = NEW.tags, content = NEW.content
    WHERE memory_id = NEW.id;
END;

CREATE TRIGGER memories_ad AFTER DELETE ON memories
BEGIN
    DELETE FROM memories_fts WHERE memory_id = OLD.id;
END;

-- 論理削除時のFTS削除
CREATE TRIGGER memories_soft_delete AFTER UPDATE ON memories
WHEN NEW.deleted = 1 AND OLD.deleted = 0
BEGIN
    DELETE FROM memories_fts WHERE memory_id = NEW.id;
END;
```

### 3.2 データ型の説明

| フィールド | 型 | 説明 | 例 |
|-----------|-----|------|-----|
| type | TEXT | 記憶の分類 | 'tech', 'project-tech', 'domain' |
| topic | TEXT | 人間が読みやすいタイトル | "Rustの非同期プログラミング" |
| tags | TEXT | 検索用キーワード | "rust,async,tokio,futures" |
| content | TEXT | 詳細な内容 | "Rustでは async/await を使って..." |
| examples | TEXT | JSON配列のコード例 | '["async fn main() {}", "tokio::spawn"]' |

## 4. 機能仕様

### 4.1 MCP Tools

#### 4.1.1 remember
```typescript
interface RememberParams {
  type: 'tech' | 'project-tech' | 'domain';
  topic: string;        // タイトル（必須）
  content: string;      // 本文（必須）
  tags?: string[];      // タグリスト
  examples?: string[];  // コード例など
  source?: string;      // 情報源
}

interface RememberResponse {
  memory_id: string;
  action: 'created' | 'updated';
  similar_count?: number;  // 類似記憶の数（Phase 3で追加）
}
```

#### 4.1.2 recall
```typescript
interface RecallParams {
  query: string;        // 検索クエリ
  type?: 'tech' | 'project-tech' | 'domain';  // フィルタ
  tags?: string[];      // タグフィルタ
  limit?: number;       // 結果数上限（デフォルト: 10）
}

interface RecallResponse {
  memories: Memory[];
  total_count: number;
}

interface Memory {
  id: string;
  type: string;
  topic: string;
  tags: string[];
  content: string;
  examples?: string[];
  reference_count: number;
  confidence: number;
  created_at: number;
}
```

### 4.2 CLIコマンド

#### 4.2.1 MCPサーバー起動
```bash
# Memory MCPサーバーを起動
$ hail-mary memory serve

# バックグラウンドで起動
$ hail-mary memory serve --daemon
```

#### 4.2.2 ドキュメント生成
```bash
# 記憶をMarkdownファイルにエクスポート
$ hail-mary memory document

# 出力:
# - ./memory-docs/tech.md
# - ./memory-docs/project-tech.md
# - ./memory-docs/domain.md

# 特定のタイプのみ
$ hail-mary memory document --type tech

# 出力先を指定
$ hail-mary memory document --output ./docs/
```

#### 4.2.3 Reindex（Phase 3）
```bash
# データベースを最適化・再構築
$ hail-mary memory reindex

# ドライラン（変更内容を確認）
$ hail-mary memory reindex --dry-run

# 詳細ログ付き
$ hail-mary memory reindex --verbose
```

## 5. データフロー

### 5.1 Remember（記憶）フロー

```mermaid
%%{init: {
  'theme': 'dark',
  'themeCSS': '
    .nodeLabel { color: #FD971F !important; }
    .edgeLabel { color: #A6E22E !important; background-color: transparent !important; }
    .cluster rect { fill: #272822 !important; stroke: #F92672 !important; stroke-width: 2px !important; rx: 5px !important; ry: 5px !important; }
    .node rect, .node circle, .node ellipse, .node polygon, .node path { fill: #272822 !important; stroke: #A6E22E !important; stroke-width: 2px !important; }
    .flowchart-link { stroke: #66D9EF !important; stroke-width: 2px !important; }
  '
}}%%
flowchart TD
    A[Claude: remember request] --> B{Input Validation}
    B -->|Valid| C[Generate UUID]
    B -->|Invalid| Z[Error Response]
    
    C --> D[Prepare Tags]
    D --> E[Format Examples as JSON]
    
    E --> F[Check Duplicates<br/>by topic + type]
    F -->|Exists| G[Update Existing]
    F -->|New| H[Insert New Memory]
    
    G --> I[Increment reference_count]
    I --> J[Update FTS Index]
    
    H --> K[Insert into memories]
    K --> L[Insert into FTS]
    
    J --> M[Success Response]
    L --> M
    
    style A fill:#F92672
    style M fill:#A6E22E
    style Z fill:#FF6188
```

### 5.2 Recall（検索）フロー

```mermaid
%%{init: {
  'theme': 'dark',
  'themeCSS': '
    .nodeLabel { color: #FD971F !important; }
    .edgeLabel { color: #A6E22E !important; background-color: transparent !important; }
    .cluster rect { fill: #272822 !important; stroke: #F92672 !important; stroke-width: 2px !important; rx: 5px !important; ry: 5px !important; }
    .node rect, .node circle, .node ellipse, .node polygon, .node path { fill: #272822 !important; stroke: #A6E22E !important; stroke-width: 2px !important; }
    .flowchart-link { stroke: #66D9EF !important; stroke-width: 2px !important; }
  '
}}%%
flowchart TD
    A[Claude: recall request] --> B[Parse Query]
    B --> C{Search Strategy}
    
    C -->|With query| D[FTS5 Search]
    C -->|No query| E[Browse by Type/Tags]
    
    D --> F[Apply Filters<br/>type, tags]
    E --> F
    
    F --> G[Order by Score<br/>+ reference_count]
    G --> H[Apply Limit]
    H --> I[Load Full Records]
    I --> J[Update last_accessed]
    J --> K[Format Response]
    K --> L[Return Results]
    
    style A fill:#F92672
    style L fill:#A6E22E
```

### 5.3 Document Generation フロー

```mermaid
%%{init: {
  'theme': 'dark',
  'themeCSS': '
    .nodeLabel { color: #FD971F !important; }
    .edgeLabel { color: #A6E22E !important; background-color: transparent !important; }
    .cluster rect { fill: #272822 !important; stroke: #F92672 !important; stroke-width: 2px !important; rx: 5px !important; ry: 5px !important; }
    .node rect, .node circle, .node ellipse, .node polygon, .node path { fill: #272822 !important; stroke: #A6E22E !important; stroke-width: 2px !important; }
    .flowchart-link { stroke: #66D9EF !important; stroke-width: 2px !important; }
  '
}}%%
flowchart TD
    A[hail-mary memory document] --> B[Query memories<br/>WHERE deleted = 0]
    
    B --> C[Group by Type]
    C --> D[tech memories]
    C --> E[project-tech memories]
    C --> F[domain memories]
    
    D --> G[Sort by confidence<br/>+ reference_count]
    E --> H[Sort by confidence<br/>+ reference_count]
    F --> I[Sort by confidence<br/>+ reference_count]
    
    G --> J[Generate tech.md]
    H --> K[Generate project-tech.md]
    I --> L[Generate domain.md]
    
    J --> M[Write to ./memory-docs/]
    K --> M
    L --> M
    
    M --> N[Success: 3 files generated]
    
    style A fill:#F92672
    style N fill:#A6E22E
```

### 5.4 Reindex フロー（Phase 3）

```mermaid
%%{init: {
  'theme': 'dark',
  'themeCSS': '
    .nodeLabel { color: #FD971F !important; }
    .edgeLabel { color: #A6E22E !important; background-color: transparent !important; }
    .cluster rect { fill: #272822 !important; stroke: #F92672 !important; stroke-width: 2px !important; rx: 5px !important; ry: 5px !important; }
    .node rect, .node circle, .node ellipse, .node polygon, .node path { fill: #272822 !important; stroke: #A6E22E !important; stroke-width: 2px !important; }
    .flowchart-link { stroke: #66D9EF !important; stroke-width: 2px !important; }
  '
}}%%
flowchart TD
    A[hail-mary memory reindex] --> B[Backup current DB]
    B --> C[Load all memories]
    
    C --> D[Generate embeddings<br/>fastembed]
    D --> E[Calculate similarities]
    
    E --> F{For each pair}
    F -->|Similarity > 0.85| G[Merge Memories]
    F -->|Similarity < 0.85| H[Keep Separate]
    
    G --> I[Combine content<br/>Sum reference_count<br/>Average confidence]
    H --> J[Keep as is]
    
    I --> K[Create new DB]
    J --> K
    
    K --> L[Remove deleted = 1]
    L --> M[Rebuild FTS index]
    M --> N[Replace old DB]
    N --> O[Archive old DB]
    
    style A fill:#F92672
    style O fill:#A6E22E
```

## 6. 実装詳細

### 6.1 プロジェクト構造（3層アーキテクチャ）

```
hail-mary/
├── src/
│   ├── commands/
│   │   └── memory/
│   │       ├── mod.rs       # サブコマンドエントリ
│   │       ├── serve.rs     # MCPサーバー起動
│   │       ├── document.rs  # ドキュメント生成
│   │       └── reindex.rs   # 再構築処理
│   ├── memory/              # ドメインロジック層
│   │   ├── mod.rs
│   │   ├── models.rs        # データモデル
│   │   ├── repository.rs    # Repository層（データアクセス）
│   │   ├── service.rs       # Service層（ビジネスロジック）
│   │   └── migration.rs     # マイグレーション管理
│   └── mcp/                 # インフラ層
│       ├── mod.rs
│       ├── server.rs        # MCPプロトコル実装
│       └── handlers.rs      # MCPツールハンドラー
├── data/
│   ├── memory.db            # 現在のデータベース
│   └── archive/             # 旧DBのアーカイブ
│       └── memory_20250116.db
└── memory-docs/             # 生成されたドキュメント
    ├── tech.md
    ├── project-tech.md
    └── domain.md
```

### 6.2 依存関係（Cargo.toml）

```toml
[dependencies]
# Phase 1: 基本機能 - Updated to rmcp 0.5.0
rmcp = { version = "0.5.0", features = ["server", "macros", "transport-io"] }
rusqlite = { version = "0.31", features = ["bundled", "json"] }
rusqlite_migration = "1.0"  # マイグレーション管理
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
schemars = "1"  # For structured output schemas
uuid = { version = "1", features = ["v4"] }
anyhow = "1"
thiserror = "1"  # エラー定義
tracing = "0.1"  # ロギング
tracing-subscriber = "0.3"

# Phase 2: ドキュメント生成
pulldown-cmark = "0.9"  # Markdown処理

# Phase 3: Reindex機能（後で追加）
# fastembed = "3"
# sqlite-vec = "0.1"
```

### 6.3 アーキテクチャ実装

#### 6.3.1 Repository層

```rust
use rusqlite::{Connection, Result, params};
use crate::memory::models::Memory;

// SQLクエリを定数化（型安全性の補完）
const INSERT_MEMORY: &str = r#"
    INSERT INTO memories (id, type, topic, tags, content, examples, 
                         reference_count, confidence, created_at, 
                         source, deleted)
    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
"#;

const SEARCH_MEMORIES_FTS: &str = r#"
    SELECT m.* FROM memories m
    JOIN memories_fts f ON m.id = f.memory_id
    WHERE f.memories_fts MATCH ?
    AND m.deleted = 0
    ORDER BY rank
    LIMIT ?
"#;

pub trait MemoryRepository {
    fn save(&mut self, memory: &Memory) -> Result<()>;
    fn find_by_id(&self, id: &str) -> Result<Option<Memory>>;
    fn search(&self, query: &str, limit: usize) -> Result<Vec<Memory>>;
    fn update_reference_count(&mut self, id: &str) -> Result<()>;
}

pub struct SqliteMemoryRepository {
    conn: Connection,
}

impl MemoryRepository for SqliteMemoryRepository {
    fn save(&mut self, memory: &Memory) -> Result<()> {
        self.conn.execute(
            INSERT_MEMORY,
            params![
                &memory.id,
                &memory.memory_type.to_string(),
                &memory.topic,
                &memory.tags.join(","),
                &memory.content,
                serde_json::to_string(&memory.examples).unwrap(),
                memory.reference_count,
                memory.confidence,
                memory.created_at,
                &memory.source,
                memory.deleted as i32,
            ],
        )?;
        Ok(())
    }
    
    fn search(&self, query: &str, limit: usize) -> Result<Vec<Memory>> {
        let mut stmt = self.conn.prepare(SEARCH_MEMORIES_FTS)?;
        let memory_iter = stmt.query_map(params![query, limit], |row| {
            Memory::from_row(row)
        })?;
        
        let mut memories = Vec::new();
        for memory in memory_iter {
            memories.push(memory?);
        }
        Ok(memories)
    }
    
    // 他のメソッド実装...
}
```

#### 6.3.2 Service層

```rust
use anyhow::Result;
use crate::memory::{
    models::{Memory, MemoryType},
    repository::MemoryRepository,
};

pub struct MemoryService<R: MemoryRepository> {
    repository: R,
}

impl<R: MemoryRepository> MemoryService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
    
    pub async fn remember(
        &mut self,
        memory_type: MemoryType,
        topic: String,
        content: String,
        tags: Vec<String>,
    ) -> Result<Memory> {
        // ビジネスロジック: 重複チェック
        if let Some(existing) = self.find_by_topic(&topic).await? {
            // 既存の記憶を更新
            self.repository.update_reference_count(&existing.id)?;
            return Ok(existing);
        }
        
        // 新規作成
        let memory = Memory::new(memory_type, topic, content);
        self.repository.save(&memory)?;
        Ok(memory)
    }
    
    pub async fn recall(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<Memory>> {
        // 検索実行
        let mut memories = self.repository.search(query, limit)?;
        
        // ビジネスロジック: 信頼度でソート
        memories.sort_by(|a, b| {
            b.confidence.partial_cmp(&a.confidence).unwrap()
        });
        
        Ok(memories)
    }
    
    async fn find_by_topic(&self, topic: &str) -> Result<Option<Memory>> {
        // トピックでの重複チェック実装
        Ok(None) // 簡略化
    }
}
```

#### 6.3.3 Handler層（MCP統合）

```rust
use rmcp::{
    ErrorData as McpError, Json, ServiceExt,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    tool, tool_handler, tool_router,
    transport::stdio,
    serve_server,
};
use schemars::JsonSchema;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::memory::service::MemoryService;
use crate::memory::repository::SqliteMemoryRepository;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RememberParams {
    pub r#type: String,
    pub topic: String,
    pub content: String,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RememberResponse {
    pub memory_id: String,
    pub action: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RecallParams {
    pub query: String,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RecallResponse {
    pub memories: Vec<Memory>,
    pub total_count: usize,
}

#[derive(Clone)]
pub struct MemoryMcpServer {
    service: Arc<Mutex<MemoryService<SqliteMemoryRepository>>>,
    tool_router: ToolRouter<Self>,
}

#[tool_handler(router = self.tool_router)]
impl rmcp::ServerHandler for MemoryMcpServer {}

#[tool_router(router = tool_router)]
impl MemoryMcpServer {
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let repository = SqliteMemoryRepository::new(db_path)?;
        let service = MemoryService::new(repository);
        
        Ok(Self {
            service: Arc::new(Mutex::new(service)),
            tool_router: Self::tool_router(),
        })
    }
    
    #[tool(name = "remember", description = "Store a memory for future recall")]
    pub async fn remember(
        &self,
        params: Parameters<RememberParams>,
    ) -> Result<Json<RememberResponse>, McpError> {
        let mut service = self.service.lock().await;
        let response = service.remember(params.0.into()).await
            .map_err(|e| McpError {
                code: -32603,
                message: e.to_string(),
                data: None,
            })?;
        Ok(Json(response.into()))
    }
    
    #[tool(name = "recall", description = "Search and retrieve stored memories")]
    pub async fn recall(
        &self,
        params: Parameters<RecallParams>,
    ) -> Result<Json<RecallResponse>, McpError> {
        let service = self.service.lock().await;
        let response = service.recall(params.0.into()).await
            .map_err(|e| McpError {
                code: -32603,
                message: e.to_string(),
                data: None,
            })?;
        Ok(Json(response.into()))
    }
}

// Server startup in main function
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = MemoryMcpServer::new("memory.db")?;
    serve_server(server, stdio()).await?;
    Ok()
}
```

### 6.4 データモデル

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryType {
    Tech,         // プロジェクトに依存しない技術
    ProjectTech,  // プロジェクト固有の技術
    Domain,       // ドメイン知識
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    pub memory_type: MemoryType,
    pub topic: String,
    pub tags: Vec<String>,
    pub content: String,
    pub examples: Vec<String>,
    pub reference_count: u32,
    pub confidence: f32,
    pub created_at: i64,
    pub last_accessed: Option<i64>,
    pub source: Option<String>,
    pub deleted: bool,
}

impl Memory {
    pub fn new(
        memory_type: MemoryType,
        topic: String,
        content: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            memory_type,
            topic,
            tags: Vec::new(),
            content,
            examples: Vec::new(),
            reference_count: 0,
            confidence: 1.0,
            created_at: chrono::Utc::now().timestamp(),
            last_accessed: None,
            source: None,
            deleted: false,
        }
    }
}
```

## 7. 生成されるドキュメントフォーマット

### 7.1 tech.md の例

```markdown
# Technical Knowledge

## Rustの非同期プログラミング
*Tags: rust, async, tokio*
*References: 15, Confidence: 0.95*

Rustでは `async`/`await` 構文を使用して非同期プログラミングを行います。
`tokio` ランタイムが最も一般的に使用されています。

### Examples:
```rust
#[tokio::main]
async fn main() {
    let result = fetch_data().await;
}

---

## React Hooksのベストプラクティス
*Tags: react, hooks, frontend*
*References: 8, Confidence: 0.88*

（以下続く）
```

### 7.2 project-tech.md の例

```markdown
# Project Technical Standards

## エラーハンドリング規約
*Tags: error-handling, rust, project-standard*
*References: 12, Confidence: 0.92*

このプロジェクトでは、すべてのエラーは `anyhow::Result` を使用して処理します。
カスタムエラー型は `thiserror` を使用して定義します。

### Examples:
```rust
use anyhow::Result;

pub fn process_data() -> Result<()> {
    // 実装
}

---

（以下続く）
```

## 8. 実装計画

### 8.1 Phase 1: 基本機能（2-3日）

**目標**: 最小限のMCPサーバーを動かす

- [ ] SQLiteデータベースの初期化
- [ ] memoriesテーブルとFTS5インデックスの作成
- [ ] rusqlite_migrationでマイグレーション管理
- [ ] Repository/Service/Handler の3層実装
- [ ] 基本的なMCPサーバー実装（JSON-RPC over stdio）
- [ ] rememberツールの実装
- [ ] recallツールの実装（FTS5検索）
- [ ] 基本的なテスト

**成果物**: `hail-mary memory serve` で起動し、Claudeから記憶の保存と検索が可能

### 8.2 Phase 2: ドキュメント生成（1-2日）

**目標**: 記憶をMarkdownで参照可能にする

- [ ] `hail-mary memory document` コマンドの実装
- [ ] Markdown生成ロジック
- [ ] タイプ別のファイル分割
- [ ] フォーマッティングとソート
- [ ] Claude Codeから `@tech.md` で参照可能に

**成果物**: 生成されたMarkdownファイルを直接参照可能

### 8.3 Phase 3: Reindex機能（2-3日）

**目標**: 定期的な最適化と重複排除

- [ ] `hail-mary memory reindex` コマンドの実装
- [ ] fastembed統合（この時点で追加）
- [ ] sqlite-vec統合（この時点で追加）
- [ ] 類似度計算とマージロジック
- [ ] データベースのバックアップとアーカイブ
- [ ] 論理削除の物理削除

**成果物**: データベースの自動最適化機能

## 9. パフォーマンス目標

### 9.1 レスポンスタイム

| 操作 | 目標時間 | 備考 |
|------|---------|------|
| remember | < 50ms | 単純なINSERT/UPDATE |
| recall (FTS) | < 100ms | 1000件での検索 |
| document生成 | < 1s | 1000件での生成 |
| reindex | < 30s | 1000件での再構築 |

### 9.2 スケーラビリティ

- 10,000件の記憶まで問題なく動作
- データベースサイズ: < 100MB（10,000件時）
- メモリ使用量: < 50MB（通常運用時）

## 10. セキュリティとプライバシー

### 10.1 基本方針

- **完全ローカル処理**: 外部APIを一切使用しない
- **データ保護**: SQLiteファイルへの適切なアクセス権限
- **センシティブ情報**: 自動検出と警告（Phase 4で検討）

### 10.2 データ管理

- データベースファイルは `~/.local/share/hail-mary/` に保存
- アーカイブは自動的に圧縮（Phase 4で検討）
- エクスポート時のフィルタリング機能

## 11. エラーハンドリング

### 11.1 エラー分類（拡充版）

```rust
#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("MCP connection error: {0}")]
    Connection(String),
    
    #[error("Database migration error: {0}")]
    Migration(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    // ドメインエラー
    #[error("Memory not found: {0}")]
    NotFound(String),
    
    #[error("Duplicate memory: {0}")]
    Duplicate(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}
```

### 11.2 エラー処理方針

- データベースエラー: リトライまたは graceful degradation
- 入力エラー: 明確なエラーメッセージで即座に返却
- 重複エラー: 既存の記憶を更新

## 12. テスト戦略

### 12.1 単体テスト

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_creation() {
        let memory = Memory::new(
            MemoryType::Tech,
            "Test Topic".to_string(),
            "Test Content".to_string(),
        );
        assert!(!memory.id.is_empty());
    }
    
    #[tokio::test]
    async fn test_remember_recall() {
        // FTS5検索のテスト
    }
}
```

### 12.2 統合テスト

- MCPプロトコル準拠テスト
- エンドツーエンドシナリオ
- ドキュメント生成の確認

## 13. 将来の拡張可能性

### 13.1 Phase 4以降の機能候補

- **関係性グラフ**: memories間の関連を管理
- **自動タグ生成**: contentから自動的にタグを抽出
- **インポート/エクスポート**: JSON/CSV形式での入出力
- **Web UI**: ブラウザから記憶を管理
- **同期機能**: 複数デバイス間での同期（暗号化付き）

### 13.2 拡張ポイント

- MemoryTypeの追加（例: personal, team）
- 検索アルゴリズムの改善
- より高度な重複検出
- マルチユーザー対応

## 14. まとめ

Memory MCP v2は、**シンプルさと実用性**を重視した設計により、1週間以内に実用的なメモリシステムを構築できます。

### 主な利点

1. **即座に価値を提供**: Phase 1だけでも実用的
2. **理解しやすい**: 単一テーブル + FTS5のシンプル構成
3. **拡張可能**: 将来の機能追加が容易
4. **疎結合**: 各機能が独立して動作
5. **実用的**: ドキュメント生成で直接参照可能

### 成功の鍵

- **段階的実装**: 各フェーズで動くものを提供
- **フィードバック重視**: 実際の使用に基づいて改善
- **シンプルさの維持**: 複雑さを避け、必要な時に追加

この設計により、過度な複雑性を避けながら、実用的で拡張可能なメモリシステムを実現します。