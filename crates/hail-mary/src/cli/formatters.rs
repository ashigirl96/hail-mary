use chrono::{DateTime, Utc};
use std::path::Path;

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
    use std::path::PathBuf;

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
}