use thiserror::Error;

#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] refinery::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("MCP protocol error: {0}")]
    #[allow(dead_code)] // Used in MCP integration tests and future features
    Mcp(String),

    // Domain errors
    #[error("Memory not found: {0}")]
    NotFound(String),

    #[error("Duplicate memory: {0}")]
    #[allow(dead_code)] // Used in tests and future duplicate detection
    Duplicate(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Invalid memory type: {0}")]
    #[allow(dead_code)] // Used in tests and future type validation
    InvalidType(String),
}

// Result type alias
pub type Result<T> = std::result::Result<T, MemoryError>;

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::error::Error;
    use std::io;

    #[test]
    fn test_memory_error_display() {
        // Test error message formatting
        let error = MemoryError::NotFound("test-id".to_string());
        assert_eq!(error.to_string(), "Memory not found: test-id");

        let error = MemoryError::InvalidInput("invalid data".to_string());
        assert_eq!(error.to_string(), "Invalid input: invalid data");

        let error = MemoryError::InvalidType("unknown".to_string());
        assert_eq!(error.to_string(), "Invalid memory type: unknown");

        let error = MemoryError::Duplicate("duplicate-id".to_string());
        assert_eq!(error.to_string(), "Duplicate memory: duplicate-id");

        let error = MemoryError::Mcp("protocol error".to_string());
        assert_eq!(error.to_string(), "MCP protocol error: protocol error");
    }

    #[test]
    fn test_memory_error_from_io_error() {
        // Test From trait conversion from io::Error
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let memory_error: MemoryError = io_error.into();

        match memory_error {
            MemoryError::Io(_) => {
                assert!(memory_error.to_string().contains("IO error"));
                assert!(memory_error.to_string().contains("file not found"));
            }
            _ => panic!("Expected Io error variant"),
        }
    }

    #[test]
    fn test_memory_error_from_serde_json_error() {
        // Test From trait conversion from serde_json::Error
        let json_str = "{ invalid json }";
        let serde_error = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
        let memory_error: MemoryError = serde_error.into();

        match memory_error {
            MemoryError::Serialization(_) => {
                assert!(memory_error.to_string().contains("Serialization error"));
            }
            _ => panic!("Expected Serialization error variant"),
        }
    }

    #[test]
    fn test_memory_error_chain() {
        // Test that error source chain is preserved
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let memory_error: MemoryError = io_error.into();

        // Check that the source error is preserved
        assert!(memory_error.source().is_some());
    }

    #[test]
    fn test_result_type_alias() {
        // Test that Result<T> alias works correctly
        fn test_function() -> Result<String> {
            Ok("success".to_string())
        }

        fn test_error_function() -> Result<String> {
            Err(MemoryError::NotFound("test".to_string()))
        }

        assert!(test_function().is_ok());
        assert!(test_error_function().is_err());

        let result = test_error_function();
        match result {
            Err(MemoryError::NotFound(id)) => assert_eq!(id, "test"),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_memory_error_debug() {
        // Test that Debug trait works properly
        let error = MemoryError::InvalidInput("test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("InvalidInput"));
        assert!(debug_str.contains("test"));
    }
}
