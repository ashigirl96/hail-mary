-- V002__add_fts5_index.sql
-- FTS5 full-text search index for memories

-- FTS5 virtual table for full-text search with Japanese support
CREATE VIRTUAL TABLE memories_fts USING fts5(
    memory_id UNINDEXED,              -- ID of the memory (not searched)
    title,                            -- Searchable title
    tags,                             -- Searchable tags
    content,                          -- Searchable content
    tokenize = 'porter unicode61'     -- Tokenizer for Japanese support
);