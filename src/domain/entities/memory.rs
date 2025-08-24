use crate::domain::value_objects::confidence::Confidence;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Memory {
    pub id: Uuid,
    pub memory_type: String, // Simple string for flexibility
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub confidence: Confidence,
    pub reference_count: u32,
    pub created_at: DateTime<Utc>,
    pub last_accessed: Option<DateTime<Utc>>,
    pub deleted: bool,
}

impl Memory {
    pub fn new(memory_type: String, title: String, content: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            memory_type,
            title,
            content,
            tags: Vec::new(),
            confidence: Confidence::default(),
            reference_count: 0,
            created_at: Utc::now(),
            last_accessed: None,
            deleted: false,
        }
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_confidence(mut self, confidence: Confidence) -> Self {
        self.confidence = confidence;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_creation() {
        let memory = Memory::new(
            "tech".to_string(),
            "Test Memory".to_string(),
            "Test content".to_string(),
        );

        assert!(!memory.id.is_nil());
        assert_eq!(memory.memory_type, "tech");
        assert_eq!(memory.title, "Test Memory");
        assert_eq!(memory.content, "Test content");
        assert!(memory.tags.is_empty());
        assert_eq!(memory.confidence.value(), 1.0);
        assert_eq!(memory.reference_count, 0);
        assert!(memory.last_accessed.is_none());
        assert!(!memory.deleted);
    }

    #[test]
    fn test_memory_with_tags() {
        let tags = vec!["rust".to_string(), "programming".to_string()];
        let memory = Memory::new(
            "tech".to_string(),
            "Test".to_string(),
            "Content".to_string(),
        )
        .with_tags(tags.clone());

        assert_eq!(memory.tags, tags);
    }

    #[test]
    fn test_memory_with_confidence() {
        let confidence = Confidence::new(0.8).unwrap();
        let memory = Memory::new(
            "tech".to_string(),
            "Test".to_string(),
            "Content".to_string(),
        )
        .with_confidence(confidence.clone());

        assert_eq!(memory.confidence.value(), 0.8);
    }

    #[test]
    fn test_memory_builder_chain() {
        let tags = vec!["tag1".to_string(), "tag2".to_string()];
        let confidence = Confidence::new(0.9).unwrap();

        let memory = Memory::new(
            "project-tech".to_string(),
            "Builder Test".to_string(),
            "Builder pattern test".to_string(),
        )
        .with_tags(tags.clone())
        .with_confidence(confidence);

        assert_eq!(memory.memory_type, "project-tech");
        assert_eq!(memory.title, "Builder Test");
        assert_eq!(memory.content, "Builder pattern test");
        assert_eq!(memory.tags, tags);
        assert_eq!(memory.confidence.value(), 0.9);
    }

    #[test]
    fn test_memory_clone() {
        let memory1 = Memory::new(
            "domain".to_string(),
            "Clone Test".to_string(),
            "Clone test content".to_string(),
        );

        let memory2 = memory1.clone();

        assert_eq!(memory1.id, memory2.id);
        assert_eq!(memory1.memory_type, memory2.memory_type);
        assert_eq!(memory1.title, memory2.title);
        assert_eq!(memory1.content, memory2.content);
    }

    #[test]
    fn test_memory_default_values() {
        let memory = Memory::new(
            "test".to_string(),
            "Default Test".to_string(),
            "Default values test".to_string(),
        );

        assert_eq!(memory.reference_count, 0);
        assert_eq!(memory.confidence.value(), 1.0);
        assert!(memory.tags.is_empty());
        assert!(!memory.deleted);
        assert!(memory.last_accessed.is_none());
        assert!(memory.created_at <= Utc::now());
    }

    #[test]
    fn test_memory_uuid_uniqueness() {
        let memory1 = Memory::new(
            "test".to_string(),
            "Test1".to_string(),
            "Content1".to_string(),
        );
        let memory2 = Memory::new(
            "test".to_string(),
            "Test2".to_string(),
            "Content2".to_string(),
        );

        assert_ne!(memory1.id, memory2.id);
    }

    #[test]
    fn test_memory_debug() {
        let memory = Memory::new(
            "debug".to_string(),
            "Debug Test".to_string(),
            "Debug test content".to_string(),
        );

        let debug_str = format!("{:?}", memory);
        assert!(debug_str.contains("Memory"));
        assert!(debug_str.contains("debug"));
        assert!(debug_str.contains("Debug Test"));
    }
}
