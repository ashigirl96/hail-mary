use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a feature specification in the Kiro project
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KiroFeature {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub directory_name: String,
    pub path: Option<PathBuf>,
}

impl KiroFeature {
    /// Create a new KiroFeature
    pub fn new(name: String, directory_name: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            created_at: Utc::now(),
            directory_name,
            path: None,
        }
    }

    /// Restore an existing KiroFeature from persistence
    #[allow(dead_code)] // Used in persistence layer and future features
    pub fn restore(
        id: String,
        name: String,
        created_at: DateTime<Utc>,
        directory_name: String,
        path: Option<PathBuf>,
    ) -> Self {
        Self {
            id,
            name,
            created_at,
            directory_name,
            path,
        }
    }

    /// Set the path for this feature
    #[allow(dead_code)] // Used in persistence layer and future features
    pub fn with_path(mut self, path: PathBuf) -> Self {
        self.path = Some(path);
        self
    }

    // ===== Business Rules: Validation =====

    /// Validate if a feature name is valid (kebab-case)
    ///
    /// Rules:
    /// - Must not be empty
    /// - Must be <= 50 characters
    /// - Only lowercase letters, numbers, and hyphens allowed
    /// - Cannot start or end with a hyphen
    /// - Cannot contain consecutive hyphens
    pub fn is_valid_name(name: &str) -> bool {
        !name.is_empty()
            && name.len() <= 50
            && name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c == '-' || c.is_ascii_digit())
            && !name.starts_with('-')
            && !name.ends_with('-')
            && !name.contains("--")
    }

    /// Check if a feature can be created (no duplicates)
    pub fn can_create(name: &str, existing_features: &[KiroFeature]) -> bool {
        !existing_features.iter().any(|f| f.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kiro_feature_new() {
        let feature = KiroFeature::new(
            "test-feature".to_string(),
            "2024-01-01-test-feature".to_string(),
        );

        assert!(!feature.id.is_empty());
        assert_eq!(feature.name, "test-feature");
        assert_eq!(feature.directory_name, "2024-01-01-test-feature");
        assert!(feature.path.is_none());
    }

    #[test]
    fn test_kiro_feature_restore() {
        let created_at = Utc::now();
        let feature = KiroFeature::restore(
            "test-id".to_string(),
            "test-feature".to_string(),
            created_at,
            "2024-01-01-test-feature".to_string(),
            Some(PathBuf::from("/path/to/feature")),
        );

        assert_eq!(feature.id, "test-id");
        assert_eq!(feature.name, "test-feature");
        assert_eq!(feature.created_at, created_at);
        assert_eq!(feature.directory_name, "2024-01-01-test-feature");
        assert_eq!(feature.path, Some(PathBuf::from("/path/to/feature")));
    }

    #[test]
    fn test_kiro_feature_with_path() {
        let feature = KiroFeature::new(
            "test-feature".to_string(),
            "2024-01-01-test-feature".to_string(),
        )
        .with_path(PathBuf::from("/test/path"));

        assert_eq!(feature.path, Some(PathBuf::from("/test/path")));
    }

    #[test]
    fn test_is_valid_name() {
        // Valid names
        assert!(KiroFeature::is_valid_name("feature"));
        assert!(KiroFeature::is_valid_name("my-feature"));
        assert!(KiroFeature::is_valid_name("feature-123"));
        assert!(KiroFeature::is_valid_name("complex-feature-name"));
        assert!(KiroFeature::is_valid_name("a"));
        assert!(KiroFeature::is_valid_name("123"));

        // Invalid names
        assert!(!KiroFeature::is_valid_name(""));
        assert!(!KiroFeature::is_valid_name("-feature"));
        assert!(!KiroFeature::is_valid_name("feature-"));
        assert!(!KiroFeature::is_valid_name("feature--name"));
        assert!(!KiroFeature::is_valid_name("Feature"));
        assert!(!KiroFeature::is_valid_name("FEATURE"));
        assert!(!KiroFeature::is_valid_name("feature_name"));
        assert!(!KiroFeature::is_valid_name("feature.name"));
        assert!(!KiroFeature::is_valid_name("feature name"));
        assert!(!KiroFeature::is_valid_name("feature@name"));

        // Too long (>50 chars)
        let long_name = "a".repeat(51);
        assert!(!KiroFeature::is_valid_name(&long_name));
    }

    #[test]
    fn test_can_create() {
        let existing = vec![
            KiroFeature::new(
                "feature-one".to_string(),
                "2024-01-01-feature-one".to_string(),
            ),
            KiroFeature::new(
                "feature-two".to_string(),
                "2024-01-02-feature-two".to_string(),
            ),
        ];

        // Can create new feature
        assert!(KiroFeature::can_create("feature-three", &existing));

        // Cannot create duplicate
        assert!(!KiroFeature::can_create("feature-one", &existing));
        assert!(!KiroFeature::can_create("feature-two", &existing));

        // Can create when no existing features
        assert!(KiroFeature::can_create("any-feature", &[]));
    }
}
