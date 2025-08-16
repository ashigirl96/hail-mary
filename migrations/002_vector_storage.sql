-- Migration 002: Add vector storage for embeddings
-- This migration adds support for vector similarity search using sqlite-vec

-- Create table for storing embeddings
CREATE TABLE IF NOT EXISTS memory_embeddings (
    memory_id TEXT PRIMARY KEY,
    embedding BLOB NOT NULL,
    embedding_model TEXT NOT NULL DEFAULT 'BAAI/bge-small-en-v1.5',
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    FOREIGN KEY (memory_id) REFERENCES memories(id) ON DELETE CASCADE
);

-- Create index for faster lookups
CREATE INDEX IF NOT EXISTS idx_memory_embeddings_memory_id ON memory_embeddings(memory_id);

-- Create table for tracking duplicates found during reindex
CREATE TABLE IF NOT EXISTS duplicate_memories (
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

-- Create index for duplicate tracking
CREATE INDEX IF NOT EXISTS idx_duplicate_memories_original ON duplicate_memories(original_id);
CREATE INDEX IF NOT EXISTS idx_duplicate_memories_duplicate ON duplicate_memories(duplicate_id);

-- Create table for reindex history
CREATE TABLE IF NOT EXISTS reindex_history (
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