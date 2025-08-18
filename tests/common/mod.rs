use std::path::PathBuf;
use tempfile::TempDir;

/// Test utilities for Memory MCP v3 tests

/// Setup temporary directory for tests
pub fn setup_test_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

/// Initialize test logging
pub fn init_test_logging() {
    use tracing_subscriber;
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .try_init();
}

/// Create test TOML content for KiroConfig testing
pub fn create_test_config_toml() -> String {
    r#"
[memory]
types = ["tech", "project-tech", "domain"]
instructions = "Test memory types for unit testing"

[memory.document]
output_dir = ".kiro/memory"
format = "markdown"

[memory.database]
path = ".kiro/memory/db.sqlite3"
"#.to_string()
}

/// Test data fixtures
pub mod fixtures {
    pub fn test_memory_title() -> String {
        "Test Memory Title".to_string()
    }
    
    pub fn test_memory_content() -> String {
        "This is test memory content for testing purposes.".to_string()
    }
    
    pub fn test_tags() -> Vec<String> {
        vec!["test".to_string(), "unit".to_string(), "rust".to_string()]
    }
}