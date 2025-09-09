use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum DocumentFormat {
    #[default]
    Markdown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectConfig {
    #[serde(default = "default_instructions")]
    pub instructions: String,
    #[serde(default)]
    pub document_format: DocumentFormat,
    #[serde(
        default = "crate::domain::entities::steering::SteeringConfig::default_for_new_project"
    )]
    pub steering: crate::domain::entities::steering::SteeringConfig,
}

impl ProjectConfig {
    pub fn default_for_new_project() -> Self {
        Self {
            instructions: DEFAULT_INSTRUCTIONS.to_string(),
            document_format: DocumentFormat::Markdown,
            steering: crate::domain::entities::steering::SteeringConfig::default_for_new_project(),
        }
    }

    pub fn validate_spec_name(name: &str) -> Result<(), crate::domain::errors::DomainError> {
        if name.is_empty() {
            return Err(crate::domain::errors::DomainError::InvalidFeatureName(
                "Name cannot be empty".to_string(),
            ));
        }

        // Validate kebab-case format
        let regex = regex::Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$").unwrap();
        if !regex.is_match(name) {
            return Err(crate::domain::errors::DomainError::InvalidFeatureName(
                name.to_string(),
            ));
        }

        Ok(())
    }
}

// TODO: 別ファイルに切り出す
const DEFAULT_INSTRUCTIONS: &str = r#"Hail-Mary Project Management System

File-based steering system for context management:
- product.md: Product overview and value proposition
- tech.md: Technical stack and development environment
- structure.md: Code organization patterns and conventions"#;

fn default_instructions() -> String {
    DEFAULT_INSTRUCTIONS.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_config_default() {
        let config = ProjectConfig::default_for_new_project();

        assert!(!config.steering.types.is_empty());

        assert!(!config.instructions.is_empty());
        assert!(
            config
                .instructions
                .contains("Hail-Mary Project Management System")
        );

        assert_eq!(config.document_format, DocumentFormat::Markdown);
    }

    // Memory type validation tests removed - using steering system

    // Memory type validation tests removed - using steering system

    #[test]
    fn test_project_config_clone() {
        let config1 = ProjectConfig::default_for_new_project();
        let config2 = config1.clone();

        assert_eq!(config1, config2);
        assert_eq!(config1.steering, config2.steering);
        assert_eq!(config1.instructions, config2.instructions);
        assert_eq!(config1.document_format, config2.document_format);
    }

    #[test]
    fn test_project_config_debug() {
        let config = ProjectConfig::default_for_new_project();
        let debug_str = format!("{:?}", config);

        assert!(debug_str.contains("ProjectConfig"));
        assert!(debug_str.contains("steering"));
        assert!(debug_str.contains("Markdown"));
    }

    #[test]
    fn test_document_format_enum() {
        let format = DocumentFormat::Markdown;

        assert_eq!(format, DocumentFormat::Markdown);
        assert_eq!(format.clone(), DocumentFormat::Markdown);

        let debug_str = format!("{:?}", format);
        assert_eq!(debug_str, "Markdown");
    }

    // Memory types ordering tests removed - using steering system

    #[test]
    fn test_default_instructions_content() {
        let config = ProjectConfig::default_for_new_project();

        assert!(config.instructions.contains("product.md:"));
        assert!(config.instructions.contains("tech.md:"));
        assert!(config.instructions.contains("structure.md:"));
    }

    // Custom memory types test removed - using steering system

    #[test]
    fn test_validate_spec_name_valid() {
        assert!(ProjectConfig::validate_spec_name("valid-name").is_ok());
        assert!(ProjectConfig::validate_spec_name("feature-123").is_ok());
        assert!(ProjectConfig::validate_spec_name("simple").is_ok());
        assert!(ProjectConfig::validate_spec_name("2025-09-05-feature").is_ok());
    }

    #[test]
    fn test_validate_spec_name_invalid() {
        use crate::domain::errors::DomainError;

        // Empty name
        assert!(matches!(
            ProjectConfig::validate_spec_name(""),
            Err(DomainError::InvalidFeatureName(_))
        ));

        // Uppercase letters
        assert!(matches!(
            ProjectConfig::validate_spec_name("Invalid-Name"),
            Err(DomainError::InvalidFeatureName(_))
        ));

        // Underscore
        assert!(matches!(
            ProjectConfig::validate_spec_name("invalid_name"),
            Err(DomainError::InvalidFeatureName(_))
        ));

        // Starts with dash
        assert!(matches!(
            ProjectConfig::validate_spec_name("-invalid"),
            Err(DomainError::InvalidFeatureName(_))
        ));

        // Ends with dash
        assert!(matches!(
            ProjectConfig::validate_spec_name("invalid-"),
            Err(DomainError::InvalidFeatureName(_))
        ));

        // Double dash
        assert!(matches!(
            ProjectConfig::validate_spec_name("invalid--name"),
            Err(DomainError::InvalidFeatureName(_))
        ));
    }
}
