use crate::application::use_cases::reindex_memories::ReindexStats;
use crate::domain::entities::memory::Memory;
use chrono::{DateTime, Utc};
use serde_json;
use std::path::Path;

/// Format a Memory entity as human-readable text
pub fn format_memory_as_text(memory: &Memory) -> String {
    let tags_str = if memory.tags.is_empty() {
        "None".to_string()
    } else {
        memory.tags.join(", ")
    };

    let last_accessed_str = memory
        .last_accessed
        .map(|dt| format_datetime(&dt))
        .unwrap_or_else(|| "Never".to_string());

    format!(
        "Memory ID: {}
Type: {}
Title: {}
Tags: {}
Confidence: {:.2}
References: {}
Created: {}
Last Accessed: {}
Content:
{}",
        memory.id,
        memory.memory_type,
        memory.title,
        tags_str,
        memory.confidence.value(),
        memory.reference_count,
        format_datetime(&memory.created_at),
        last_accessed_str,
        memory.content
    )
}

/// Format a Memory entity as JSON
pub fn format_memory_as_json(memory: &Memory) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&serde_json::json!({
        "id": memory.id.to_string(),
        "type": memory.memory_type,
        "title": memory.title,
        "content": memory.content,
        "tags": memory.tags,
        "confidence": memory.confidence.value(),
        "reference_count": memory.reference_count,
        "created_at": memory.created_at.to_rfc3339(),
        "last_accessed": memory.last_accessed.map(|dt| dt.to_rfc3339()),
        "deleted": memory.deleted
    }))
}

/// Format a Memory entity as Markdown
pub fn format_memory_as_markdown(memory: &Memory) -> String {
    let tags_str = if memory.tags.is_empty() {
        "_None_"
    } else {
        &memory
            .tags
            .iter()
            .map(|t| format!("`{}`", t))
            .collect::<Vec<_>>()
            .join(", ")
    };

    let last_accessed_str = memory
        .last_accessed
        .map(|dt| format_datetime(&dt))
        .unwrap_or_else(|| "_Never_".to_string());

    format!(
        "## {}

**ID**: `{}`  
**Type**: `{}`  
**Tags**: {}  
**Confidence**: {:.2}  
**References**: {}  
**Created**: {}  
**Last Accessed**: {}

### Content

{}

---",
        memory.title,
        memory.id,
        memory.memory_type,
        tags_str,
        memory.confidence.value(),
        memory.reference_count,
        format_datetime(&memory.created_at),
        last_accessed_str,
        memory.content
    )
}

/// Format multiple memories as Markdown document
pub fn format_memories_as_markdown(memories: &[Memory], memory_type: &str) -> String {
    if memories.is_empty() {
        return format!("# {} Memories\n\nNo memories found.", memory_type);
    }

    let mut content = format!(
        "# {} Memories\n\nTotal: {} memories\n\n",
        memory_type,
        memories.len()
    );

    for memory in memories {
        content.push_str(&format_memory_as_markdown(memory));
        content.push('\n');
    }

    content
}

/// Format ReindexStats as text
pub fn format_reindex_stats_as_text(stats: &ReindexStats) -> String {
    let mut lines = Vec::new();

    lines.push("Database Reindex Complete".to_string());
    lines.push(format!(
        "  Deleted entries removed: {}",
        stats.deleted_entries
    ));

    if stats.index_rebuilt {
        lines.push("  FTS5 index: Rebuilt ✓".to_string());
    } else {
        lines.push("  FTS5 index: Not rebuilt".to_string());
    }

    if stats.database_optimized {
        lines.push("  Database: Optimized ✓".to_string());
    } else {
        lines.push("  Database: Not optimized".to_string());
    }

    lines.join("\n")
}

/// Format ReindexStats as JSON
pub fn format_reindex_stats_as_json(stats: &ReindexStats) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&serde_json::json!({
        "deleted_entries": stats.deleted_entries,
        "index_rebuilt": stats.index_rebuilt,
        "database_optimized": stats.database_optimized
    }))
}

/// Format a success message
pub fn format_success(message: &str) -> String {
    format!("✅ {}", message)
}

/// Format an error message
pub fn format_error(message: &str) -> String {
    format!("❌ Error: {}", message)
}

/// Format a warning message
pub fn format_warning(message: &str) -> String {
    format!("⚠️  Warning: {}", message)
}

/// Format an info message
pub fn format_info(message: &str) -> String {
    format!("ℹ️  {}", message)
}

/// Format a path for display
pub fn format_path(path: &Path) -> String {
    path.display().to_string()
}

/// Format a DateTime as human-readable string
pub fn format_datetime(dt: &DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// Format a list of items with bullet points
pub fn format_list(items: &[String]) -> String {
    items
        .iter()
        .map(|item| format!("  • {}", item))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Format a section header
pub fn format_header(text: &str) -> String {
    let line = "─".repeat(text.len() + 4);
    format!("┌{}┐\n│ {} │\n└{}┘", line, text, line)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::confidence::Confidence;
    use std::path::PathBuf;

    fn create_test_memory() -> Memory {
        let mut memory = Memory::new(
            "tech".to_string(),
            "Test Memory".to_string(),
            "This is test content".to_string(),
        );
        memory.tags = vec!["rust".to_string(), "testing".to_string()];
        memory.confidence = Confidence::new(0.85).unwrap();
        memory.reference_count = 5;
        memory
    }

    #[test]
    fn test_format_memory_as_text() {
        let memory = create_test_memory();
        let text = format_memory_as_text(&memory);

        assert!(text.contains("Test Memory"));
        assert!(text.contains("tech"));
        assert!(text.contains("rust, testing"));
        assert!(text.contains("0.85"));
        assert!(text.contains("5")); // reference count
        assert!(text.contains("This is test content"));
        assert!(text.contains("Never")); // last accessed
    }

    #[test]
    fn test_format_memory_as_json() {
        let memory = create_test_memory();
        let json = format_memory_as_json(&memory).unwrap();

        assert!(json.contains("\"type\": \"tech\""));
        assert!(json.contains("\"title\": \"Test Memory\""));
        assert!(json.contains("\"confidence\": 0.85"));
        assert!(json.contains("\"reference_count\": 5"));
        assert!(json.contains("\"tags\": ["));
        assert!(json.contains("\"rust\""));
        assert!(json.contains("\"testing\""));
    }

    #[test]
    fn test_format_memory_as_markdown() {
        let memory = create_test_memory();
        let markdown = format_memory_as_markdown(&memory);

        assert!(markdown.contains("## Test Memory"));
        assert!(markdown.contains("**Type**: `tech`"));
        assert!(markdown.contains("`rust`"));
        assert!(markdown.contains("`testing`"));
        assert!(markdown.contains("**Confidence**: 0.85"));
        assert!(markdown.contains("**References**: 5"));
        assert!(markdown.contains("This is test content"));
        assert!(markdown.contains("---"));
    }

    #[test]
    fn test_format_memories_as_markdown_empty() {
        let memories = vec![];
        let markdown = format_memories_as_markdown(&memories, "tech");

        assert!(markdown.contains("# tech Memories"));
        assert!(markdown.contains("No memories found"));
    }

    #[test]
    fn test_format_memories_as_markdown_multiple() {
        let memories = vec![create_test_memory(), create_test_memory()];
        let markdown = format_memories_as_markdown(&memories, "tech");

        assert!(markdown.contains("# tech Memories"));
        assert!(markdown.contains("Total: 2 memories"));
        assert_eq!(markdown.matches("## Test Memory").count(), 2);
    }

    #[test]
    fn test_format_reindex_stats_as_text() {
        let stats = ReindexStats {
            deleted_entries: 10,
            index_rebuilt: true,
            database_optimized: true,
        };
        let text = format_reindex_stats_as_text(&stats);

        assert!(text.contains("Database Reindex Complete"));
        assert!(text.contains("Deleted entries removed: 10"));
        assert!(text.contains("FTS5 index: Rebuilt ✓"));
        assert!(text.contains("Database: Optimized ✓"));
    }

    #[test]
    fn test_format_reindex_stats_as_text_partial() {
        let stats = ReindexStats {
            deleted_entries: 0,
            index_rebuilt: false,
            database_optimized: true,
        };
        let text = format_reindex_stats_as_text(&stats);

        assert!(text.contains("Deleted entries removed: 0"));
        assert!(text.contains("FTS5 index: Not rebuilt"));
        assert!(text.contains("Database: Optimized ✓"));
    }

    #[test]
    fn test_format_reindex_stats_as_json() {
        let stats = ReindexStats {
            deleted_entries: 5,
            index_rebuilt: true,
            database_optimized: false,
        };
        let json = format_reindex_stats_as_json(&stats).unwrap();

        assert!(json.contains("\"deleted_entries\": 5"));
        assert!(json.contains("\"index_rebuilt\": true"));
        assert!(json.contains("\"database_optimized\": false"));
    }

    #[test]
    fn test_format_success() {
        let msg = format_success("Operation completed");
        assert_eq!(msg, "✅ Operation completed");
    }

    #[test]
    fn test_format_error() {
        let msg = format_error("Something went wrong");
        assert_eq!(msg, "❌ Error: Something went wrong");
    }

    #[test]
    fn test_format_warning() {
        let msg = format_warning("This might be a problem");
        assert_eq!(msg, "⚠️  Warning: This might be a problem");
    }

    #[test]
    fn test_format_info() {
        let msg = format_info("Just so you know");
        assert_eq!(msg, "ℹ️  Just so you know");
    }

    #[test]
    fn test_format_path() {
        let path = PathBuf::from("/home/user/project");
        let formatted = format_path(&path);
        assert_eq!(formatted, "/home/user/project");
    }

    #[test]
    fn test_format_datetime() {
        let dt = DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let formatted = format_datetime(&dt);
        assert_eq!(formatted, "2024-01-15 10:30:00 UTC");
    }

    #[test]
    fn test_format_list() {
        let items = vec![
            "First item".to_string(),
            "Second item".to_string(),
            "Third item".to_string(),
        ];
        let formatted = format_list(&items);

        assert!(formatted.contains("  • First item"));
        assert!(formatted.contains("  • Second item"));
        assert!(formatted.contains("  • Third item"));
    }

    #[test]
    fn test_format_header() {
        let header = format_header("Test Header");

        assert!(header.contains("Test Header"));
        assert!(header.contains("┌"));
        assert!(header.contains("┐"));
        assert!(header.contains("└"));
        assert!(header.contains("┘"));
        assert!(header.contains("─"));
    }

    #[test]
    fn test_memory_with_last_accessed() {
        let mut memory = create_test_memory();
        let accessed_time = DateTime::parse_from_rfc3339("2024-01-20T15:45:00Z")
            .unwrap()
            .with_timezone(&Utc);
        memory.last_accessed = Some(accessed_time);

        let text = format_memory_as_text(&memory);
        assert!(text.contains("2024-01-20 15:45:00 UTC"));
        assert!(!text.contains("Never"));
    }

    #[test]
    fn test_memory_without_tags() {
        let mut memory = create_test_memory();
        memory.tags = vec![];

        let text = format_memory_as_text(&memory);
        assert!(text.contains("Tags: None"));

        let markdown = format_memory_as_markdown(&memory);
        assert!(markdown.contains("**Tags**: _None_"));
    }
}
