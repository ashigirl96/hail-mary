-- V003__add_triggers.sql
-- Triggers to automatically maintain FTS5 index

-- Trigger: INSERT - Add to FTS5 when new memory is inserted
CREATE TRIGGER memories_ai AFTER INSERT ON memories
WHEN NEW.deleted = 0
BEGIN
    INSERT INTO memories_fts(memory_id, title, tags, content)
    VALUES (NEW.id, NEW.title, NEW.tags, NEW.content);
END;

-- Trigger: UPDATE - Update FTS5 when memory is updated (and not deleted)
CREATE TRIGGER memories_au AFTER UPDATE ON memories
WHEN NEW.deleted = 0 AND OLD.deleted = 0
BEGIN
    UPDATE memories_fts 
    SET title = NEW.title, tags = NEW.tags, content = NEW.content
    WHERE memory_id = NEW.id;
END;

-- Trigger: DELETE - Remove from FTS5 when memory is physically deleted
CREATE TRIGGER memories_ad AFTER DELETE ON memories
BEGIN
    DELETE FROM memories_fts WHERE memory_id = OLD.id;
END;

-- Trigger: SOFT DELETE - Remove from FTS5 when memory is logically deleted
CREATE TRIGGER memories_soft_delete AFTER UPDATE ON memories
WHEN NEW.deleted = 1 AND OLD.deleted = 0
BEGIN
    DELETE FROM memories_fts WHERE memory_id = NEW.id;
END;