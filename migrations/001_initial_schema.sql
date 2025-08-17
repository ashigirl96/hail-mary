-- 統合されたメモリ管理データベーススキーマ
-- このマイグレーションには全ての必要なテーブルとインデックスが含まれています

-- =============================================================================
-- メインテーブル
-- =============================================================================

CREATE TABLE memories (
    id TEXT PRIMARY KEY,              -- UUID v4
    type TEXT NOT NULL CHECK(         -- 記憶のカテゴリ
        type IN ('tech', 'project-tech', 'domain')
    ),
    title TEXT NOT NULL,              -- タイトル/要約（人間が読みやすい）
    tags TEXT,                        -- JSON配列でタグを保存
    content TEXT NOT NULL,            -- 本文
    examples TEXT,                    -- JSON配列でコード例などを保存
    reference_count INTEGER DEFAULT 0, -- 参照された回数
    confidence REAL DEFAULT 1.0       -- 信頼度スコア (0.0-1.0)
        CHECK(confidence >= 0 AND confidence <= 1),
    created_at INTEGER DEFAULT (unixepoch()), -- 作成日時
    last_accessed INTEGER,            -- 最終アクセス日時
    deleted INTEGER DEFAULT 0         -- 論理削除フラグ
);

-- =============================================================================
-- 全文検索インデックス
-- =============================================================================

-- FTS5全文検索インデックス
CREATE VIRTUAL TABLE memories_fts USING fts5(
    memory_id UNINDEXED,              -- 検索対象外
    title,                            -- 検索対象
    tags,                             -- 検索対象
    content,                          -- 検索対象
    tokenize = 'porter unicode61'     -- 日本語対応トークナイザー
);

-- =============================================================================
-- ベクトル埋め込みテーブル
-- =============================================================================

-- ベクトル埋め込み保存テーブル
CREATE TABLE memory_embeddings (
    memory_id TEXT PRIMARY KEY,
    embedding BLOB NOT NULL,
    embedding_model TEXT NOT NULL DEFAULT 'BAAI/bge-small-en-v1.5',
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    FOREIGN KEY (memory_id) REFERENCES memories(id) ON DELETE CASCADE
);

-- 埋め込みインデックステーブル（高速類似検索用）
CREATE TABLE embedding_index (
    memory_id TEXT PRIMARY KEY,
    embedding_hash TEXT NOT NULL,  -- 埋め込みのハッシュ（キャッシュ検証用）
    magnitude REAL NOT NULL,        -- 事前計算された大きさ（高速類似度計算用）
    cluster_id INTEGER,             -- オプションのクラスタ割り当て
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (memory_id) REFERENCES memories(id) ON DELETE CASCADE
);

-- 量子化埋め込みテーブル（メモリ使用量削減用）
CREATE TABLE embeddings_quantized (
    memory_id TEXT PRIMARY KEY,
    embedding_q8 BLOB NOT NULL,      -- 8ビット量子化埋め込み
    scale REAL NOT NULL,             -- 量子化スケールファクタ
    offset REAL NOT NULL,            -- 量子化オフセット
    original_norm REAL NOT NULL,     -- 再構築用の元のベクトルノルム
    created_at INTEGER NOT NULL,
    FOREIGN KEY (memory_id) REFERENCES memories(id) ON DELETE CASCADE
);

-- 埋め込みキャッシュテーブル
CREATE TABLE embedding_cache (
    cache_key TEXT PRIMARY KEY,     -- 入力テキストのハッシュ
    embedding BLOB NOT NULL,         -- キャッシュされた埋め込みベクトル
    model_name TEXT NOT NULL,        -- 生成に使用されたモデル
    dimension INTEGER NOT NULL,      -- 埋め込み次元
    created_at INTEGER NOT NULL,
    accessed_at INTEGER NOT NULL,
    access_count INTEGER DEFAULT 1
);

-- =============================================================================
-- 重複管理テーブル
-- =============================================================================

-- 重複メモリ追跡テーブル
CREATE TABLE duplicate_memories (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    original_id TEXT NOT NULL,
    duplicate_id TEXT NOT NULL,
    similarity_score REAL NOT NULL,
    merged_into TEXT,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    FOREIGN KEY (original_id) REFERENCES memories(id),
    FOREIGN KEY (duplicate_id) REFERENCES memories(id),
    FOREIGN KEY (merged_into) REFERENCES memories(id)
);

-- =============================================================================
-- 分析・最適化テーブル
-- =============================================================================

-- 語彙テーブル（TF-IDF最適化用）
CREATE TABLE vocabulary (
    word TEXT PRIMARY KEY,
    idf_score REAL NOT NULL,         -- 逆文書頻度
    document_count INTEGER NOT NULL, -- その単語を含む文書数
    total_count INTEGER NOT NULL,    -- 全文書での総出現回数
    updated_at INTEGER NOT NULL
);

-- 再インデックス履歴テーブル
CREATE TABLE reindex_history (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    started_at INTEGER NOT NULL,
    completed_at INTEGER,
    total_memories INTEGER NOT NULL DEFAULT 0,
    duplicates_found INTEGER NOT NULL DEFAULT 0,
    duplicates_merged INTEGER NOT NULL DEFAULT 0,
    backup_path TEXT,
    status TEXT NOT NULL DEFAULT 'running',
    error_message TEXT
);

-- 埋め込み処理キューテーブル
CREATE TABLE embedding_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    memory_id TEXT NOT NULL,
    priority INTEGER DEFAULT 0,      -- 高い優先度から処理
    status TEXT DEFAULT 'pending',   -- pending, processing, completed, failed
    retry_count INTEGER DEFAULT 0,
    error_message TEXT,
    created_at INTEGER NOT NULL,
    processed_at INTEGER,
    FOREIGN KEY (memory_id) REFERENCES memories(id) ON DELETE CASCADE
);

-- =============================================================================
-- インデックス
-- =============================================================================

-- メインテーブルのインデックス
CREATE INDEX idx_memories_type 
    ON memories(type) 
    WHERE deleted = 0;

CREATE INDEX idx_memories_ref_count 
    ON memories(reference_count DESC) 
    WHERE deleted = 0;

CREATE INDEX idx_memories_created 
    ON memories(created_at DESC) 
    WHERE deleted = 0;

-- 埋め込み関連のインデックス
CREATE INDEX idx_memory_embeddings_memory_id ON memory_embeddings(memory_id);
CREATE INDEX idx_embedding_index_cluster ON embedding_index(cluster_id);
CREATE INDEX idx_embedding_index_magnitude ON embedding_index(magnitude);
CREATE INDEX idx_embedding_index_updated ON embedding_index(updated_at);

-- キャッシュ関連のインデックス
CREATE INDEX idx_embedding_cache_accessed ON embedding_cache(accessed_at);
CREATE INDEX idx_embedding_cache_count ON embedding_cache(access_count);

-- 重複管理のインデックス
CREATE INDEX idx_duplicate_memories_original ON duplicate_memories(original_id);
CREATE INDEX idx_duplicate_memories_duplicate ON duplicate_memories(duplicate_id);

-- 分析用のインデックス
CREATE INDEX idx_vocabulary_idf ON vocabulary(idf_score);
CREATE INDEX idx_embedding_queue_status ON embedding_queue(status, priority DESC);
CREATE INDEX idx_embedding_queue_memory ON embedding_queue(memory_id);

-- =============================================================================
-- トリガー（FTS5インデックスの自動更新）
-- =============================================================================

-- 新規挿入時のFTSインデックス更新
CREATE TRIGGER memories_ai AFTER INSERT ON memories
WHEN NEW.deleted = 0
BEGIN
    INSERT INTO memories_fts(memory_id, title, tags, content)
    VALUES (NEW.id, NEW.title, NEW.tags, NEW.content);
END;

-- 更新時のFTSインデックス更新
CREATE TRIGGER memories_au AFTER UPDATE ON memories
WHEN NEW.deleted = 0 AND OLD.deleted = 0
BEGIN
    UPDATE memories_fts 
    SET title = NEW.title, tags = NEW.tags, content = NEW.content
    WHERE memory_id = NEW.id;
END;

-- 削除時のFTSインデックス削除
CREATE TRIGGER memories_ad AFTER DELETE ON memories
BEGIN
    DELETE FROM memories_fts WHERE memory_id = OLD.id;
END;

-- 論理削除時のFTSインデックス削除
CREATE TRIGGER memories_soft_delete AFTER UPDATE ON memories
WHEN NEW.deleted = 1 AND OLD.deleted = 0
BEGIN
    DELETE FROM memories_fts WHERE memory_id = NEW.id;
END;