-- Create embedding index table for fast similarity search
CREATE TABLE IF NOT EXISTS embedding_index (
    memory_id TEXT PRIMARY KEY,
    embedding_hash TEXT NOT NULL,  -- Hash of embedding for cache validation
    magnitude REAL NOT NULL,        -- Pre-computed magnitude for fast similarity
    cluster_id INTEGER,             -- Optional cluster assignment
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (memory_id) REFERENCES memories(id) ON DELETE CASCADE
);

-- Create indices for fast lookups
CREATE INDEX IF NOT EXISTS idx_embedding_index_cluster ON embedding_index(cluster_id);
CREATE INDEX IF NOT EXISTS idx_embedding_index_magnitude ON embedding_index(magnitude);
CREATE INDEX IF NOT EXISTS idx_embedding_index_updated ON embedding_index(updated_at);

-- Create embedding cache table
CREATE TABLE IF NOT EXISTS embedding_cache (
    cache_key TEXT PRIMARY KEY,     -- Hash of input text
    embedding BLOB NOT NULL,         -- Cached embedding vector
    model_name TEXT NOT NULL,        -- Model used for generation
    dimension INTEGER NOT NULL,      -- Embedding dimension
    created_at INTEGER NOT NULL,
    accessed_at INTEGER NOT NULL,
    access_count INTEGER DEFAULT 1
);

-- Create index for cache cleanup
CREATE INDEX IF NOT EXISTS idx_embedding_cache_accessed ON embedding_cache(accessed_at);
CREATE INDEX IF NOT EXISTS idx_embedding_cache_count ON embedding_cache(access_count);

-- Create quantized embeddings table for reduced memory usage
CREATE TABLE IF NOT EXISTS embeddings_quantized (
    memory_id TEXT PRIMARY KEY,
    embedding_q8 BLOB NOT NULL,      -- 8-bit quantized embedding
    scale REAL NOT NULL,             -- Quantization scale factor
    offset REAL NOT NULL,            -- Quantization offset
    original_norm REAL NOT NULL,     -- Original vector norm for reconstruction
    created_at INTEGER NOT NULL,
    FOREIGN KEY (memory_id) REFERENCES memories(id) ON DELETE CASCADE
);

-- Create vocabulary table for TF-IDF optimization
CREATE TABLE IF NOT EXISTS vocabulary (
    word TEXT PRIMARY KEY,
    idf_score REAL NOT NULL,         -- Inverse document frequency
    document_count INTEGER NOT NULL, -- Number of documents containing the word
    total_count INTEGER NOT NULL,    -- Total occurrences across all documents
    updated_at INTEGER NOT NULL
);

-- Create index for vocabulary lookups
CREATE INDEX IF NOT EXISTS idx_vocabulary_idf ON vocabulary(idf_score);

-- Create batch processing queue table
CREATE TABLE IF NOT EXISTS embedding_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    memory_id TEXT NOT NULL,
    priority INTEGER DEFAULT 0,      -- Higher priority processed first
    status TEXT DEFAULT 'pending',   -- pending, processing, completed, failed
    retry_count INTEGER DEFAULT 0,
    error_message TEXT,
    created_at INTEGER NOT NULL,
    processed_at INTEGER,
    FOREIGN KEY (memory_id) REFERENCES memories(id) ON DELETE CASCADE
);

-- Create indices for queue processing
CREATE INDEX IF NOT EXISTS idx_embedding_queue_status ON embedding_queue(status, priority DESC);
CREATE INDEX IF NOT EXISTS idx_embedding_queue_memory ON embedding_queue(memory_id);