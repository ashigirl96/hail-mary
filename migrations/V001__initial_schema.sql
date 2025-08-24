-- V001__initial_schema.sql
-- Initial schema for Memory MCP v3

-- Main memories table
CREATE TABLE memories (
    id TEXT PRIMARY KEY,              -- UUID v4
    type TEXT NOT NULL,               -- Memory category (defined in config.toml)
    title TEXT NOT NULL,              -- Title/summary (human readable)
    tags TEXT,                        -- Comma-separated tags (e.g. "rust,async,tokio")
    content TEXT NOT NULL,            -- Main content
    reference_count INTEGER DEFAULT 0, -- Number of times referenced
    confidence REAL DEFAULT 1.0       -- Confidence score (0.0-1.0)
        CHECK(confidence >= 0 AND confidence <= 1),
    created_at INTEGER DEFAULT (unixepoch()), -- Creation timestamp
    last_accessed INTEGER,            -- Last access timestamp
    deleted INTEGER DEFAULT 0         -- Logical deletion flag
);

-- Essential indexes for performance
CREATE INDEX idx_memories_type 
    ON memories(type) 
    WHERE deleted = 0;

CREATE INDEX idx_memories_ref_count 
    ON memories(reference_count DESC) 
    WHERE deleted = 0;

CREATE INDEX idx_memories_created 
    ON memories(created_at DESC) 
    WHERE deleted = 0;