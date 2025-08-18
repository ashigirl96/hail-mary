use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MemoryType {
    Tech,
    ProjectTech,
    Domain,
}

impl Display for MemoryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryType::Tech => write!(f, "tech"),
            MemoryType::ProjectTech => write!(f, "project-tech"),
            MemoryType::Domain => write!(f, "domain"),
        }
    }
}

impl FromStr for MemoryType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tech" => Ok(MemoryType::Tech),
            "project-tech" => Ok(MemoryType::ProjectTech),
            "domain" => Ok(MemoryType::Domain),
            _ => Err(format!("Invalid memory type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    pub memory_type: MemoryType,
    pub title: String,
    pub tags: Vec<String>,
    pub content: String,
    pub reference_count: u32,
    pub confidence: f32,
    pub created_at: i64,
    pub last_accessed: Option<i64>,
    pub deleted: bool,
}

impl Memory {
    /// Create a new Memory instance with default values
    pub fn new(memory_type: MemoryType, title: String, content: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            memory_type,
            title,
            tags: Vec::new(),
            content,
            reference_count: 0,
            confidence: 1.0,
            created_at: chrono::Utc::now().timestamp(),
            last_accessed: None,
            deleted: false,
        }
    }

    /// Builder method to set tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Builder method to set confidence level
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }

    /// Convert from SQLite row (placeholder for now)
    #[allow(dead_code)] // Used in SQLite repository implementations
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        let type_str: String = row.get("type")?;
        let tags_str: String = row.get("tags")?;
        let reference_count: i32 = row.get("reference_count")?;

        Ok(Self {
            id: row.get("id")?,
            memory_type: type_str.parse().unwrap(),
            title: row.get("title")?,
            tags: if tags_str.is_empty() {
                Vec::new()
            } else {
                tags_str.split(',').map(|s| s.to_string()).collect()
            },
            content: row.get("content")?,
            reference_count: reference_count as u32,
            confidence: row.get("confidence")?,
            created_at: row.get("created_at")?,
            last_accessed: row.get("last_accessed")?,
            deleted: row.get::<_, i32>("deleted")? != 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_memory_new_creates_valid_instance() {
        // Test that Memory::new creates a valid instance with correct defaults
        let memory = Memory::new(
            MemoryType::Tech,
            "Test Topic".to_string(),
            "Test Content".to_string(),
        );

        // UUID should be generated
        assert!(!memory.id.is_empty());
        assert!(Uuid::parse_str(&memory.id).is_ok());

        // Check core fields
        assert_eq!(memory.memory_type, MemoryType::Tech);
        assert_eq!(memory.title, "Test Topic");
        assert_eq!(memory.content, "Test Content");

        // Check default values
        assert_eq!(memory.reference_count, 0);
        assert_eq!(memory.confidence, 1.0);
        assert_eq!(memory.deleted, false);
        assert!(memory.tags.is_empty());

        // created_at should be set to current time (within reasonable range)
        let now = chrono::Utc::now().timestamp();
        assert!(memory.created_at <= now);
        assert!(memory.created_at >= now - 5); // Allow 5 seconds difference

        // last_accessed should be None initially
        assert_eq!(memory.last_accessed, None);
    }

    #[test]
    fn test_memory_with_tags_builder() {
        // Test the builder pattern for tags
        let tags = vec!["rust".to_string(), "async".to_string(), "tokio".to_string()];
        let memory = Memory::new(
            MemoryType::Tech,
            "Test Topic".to_string(),
            "Test Content".to_string(),
        )
        .with_tags(tags.clone());

        assert_eq!(memory.tags, tags);
    }

    #[test]
    fn test_memory_with_confidence_builder() {
        // Test the builder pattern for confidence
        let memory = Memory::new(
            MemoryType::Tech,
            "Test Topic".to_string(),
            "Test Content".to_string(),
        )
        .with_confidence(0.8);

        assert_eq!(memory.confidence, 0.8);
    }

    #[test]
    fn test_memory_with_confidence_range() {
        // Test confidence validation (0.0-1.0 range)
        let memory1 = Memory::new(MemoryType::Tech, "Test".to_string(), "Content".to_string())
            .with_confidence(0.0);
        assert_eq!(memory1.confidence, 0.0);

        let memory2 = Memory::new(MemoryType::Tech, "Test".to_string(), "Content".to_string())
            .with_confidence(1.0);
        assert_eq!(memory2.confidence, 1.0);
    }

    #[test]
    fn test_memory_from_row() {
        // This test will be implemented once we have rusqlite::Row available
        // For now, we'll create a placeholder test that should fail
        // This forces us to implement the from_row method

        // TODO: Implement this test when we have SQLite integration
        // We need to mock rusqlite::Row or create integration test
    }

    #[test]
    fn test_memory_builder_chain() {
        // Test chaining multiple builder methods
        let tags = vec!["test".to_string()];
        let memory = Memory::new(
            MemoryType::ProjectTech,
            "Chained Test".to_string(),
            "Chained Content".to_string(),
        )
        .with_tags(tags.clone())
        .with_confidence(0.9);

        assert_eq!(memory.tags, tags);
        assert_eq!(memory.confidence, 0.9);
        assert_eq!(memory.memory_type, MemoryType::ProjectTech);
    }
}

#[cfg(test)]
mod memory_type_tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::str::FromStr;

    #[test]
    fn test_memory_type_display() {
        // Test Display trait implementation
        assert_eq!(MemoryType::Tech.to_string(), "tech");
        assert_eq!(MemoryType::ProjectTech.to_string(), "project-tech");
        assert_eq!(MemoryType::Domain.to_string(), "domain");
    }

    #[test]
    fn test_memory_type_from_str() {
        // Test FromStr trait implementation
        assert_eq!(MemoryType::from_str("tech").unwrap(), MemoryType::Tech);
        assert_eq!(
            MemoryType::from_str("project-tech").unwrap(),
            MemoryType::ProjectTech
        );
        assert_eq!(MemoryType::from_str("domain").unwrap(), MemoryType::Domain);

        // Test invalid input
        assert!(MemoryType::from_str("invalid").is_err());
        assert!(MemoryType::from_str("").is_err());
        assert!(MemoryType::from_str("TECH").is_err()); // Case sensitive
    }

    #[test]
    fn test_memory_type_round_trip() {
        // Test Display → FromStr → Display consistency
        let types = vec![
            MemoryType::Tech,
            MemoryType::ProjectTech,
            MemoryType::Domain,
        ];

        for memory_type in types {
            let display_str = memory_type.to_string();
            let parsed = MemoryType::from_str(&display_str).unwrap();
            assert_eq!(parsed, memory_type);
            assert_eq!(parsed.to_string(), display_str);
        }
    }

    #[test]
    fn test_memory_type_error_messages() {
        // Test that error messages are informative
        let result = MemoryType::from_str("invalid");
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("invalid"));
        assert!(error_msg.contains("Invalid memory type"));
    }
}
