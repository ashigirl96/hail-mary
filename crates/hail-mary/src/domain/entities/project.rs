#[derive(Debug, Clone, PartialEq)]
pub enum DocumentFormat {
    Markdown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectConfig {
    pub memory_types: Vec<String>,
    pub instructions: String,
    pub document_format: DocumentFormat,
}

impl ProjectConfig {
    pub fn default_for_new_project() -> Self {
        Self {
            memory_types: vec![
                "tech".to_string(),
                "project-tech".to_string(),
                "domain".to_string(),
                "workflow".to_string(),
                "decision".to_string(),
            ],
            instructions: DEFAULT_INSTRUCTIONS.to_string(),
            document_format: DocumentFormat::Markdown,
        }
    }

    pub fn validate_memory_type(&self, memory_type: &str) -> bool {
        self.memory_types.contains(&memory_type.to_string())
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
const DEFAULT_INSTRUCTIONS: &str = r#"Memory MCP Server v3

Available memory types:
- tech: General technical knowledge (languages, frameworks, algorithms)
- project-tech: This project's specific technical implementation
- domain: Business domain knowledge and requirements
- workflow: Development workflows and processes
- decision: Architecture decisions and their rationale"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_config_default() {
        let config = ProjectConfig::default_for_new_project();

        assert_eq!(config.memory_types.len(), 5);
        assert!(config.memory_types.contains(&"tech".to_string()));
        assert!(config.memory_types.contains(&"project-tech".to_string()));
        assert!(config.memory_types.contains(&"domain".to_string()));
        assert!(config.memory_types.contains(&"workflow".to_string()));
        assert!(config.memory_types.contains(&"decision".to_string()));

        assert!(!config.instructions.is_empty());
        assert!(config.instructions.contains("Memory MCP Server v3"));

        assert_eq!(config.document_format, DocumentFormat::Markdown);
    }

    #[test]
    fn test_validate_memory_type_valid() {
        let config = ProjectConfig::default_for_new_project();

        assert!(config.validate_memory_type("tech"));
        assert!(config.validate_memory_type("project-tech"));
        assert!(config.validate_memory_type("domain"));
        assert!(config.validate_memory_type("workflow"));
        assert!(config.validate_memory_type("decision"));
    }

    #[test]
    fn test_validate_memory_type_invalid() {
        let config = ProjectConfig::default_for_new_project();

        assert!(!config.validate_memory_type("invalid"));
        assert!(!config.validate_memory_type("unknown"));
        assert!(!config.validate_memory_type(""));
        assert!(!config.validate_memory_type("TECH")); // case sensitive
    }

    #[test]
    fn test_project_config_clone() {
        let config1 = ProjectConfig::default_for_new_project();
        let config2 = config1.clone();

        assert_eq!(config1, config2);
        assert_eq!(config1.memory_types, config2.memory_types);
        assert_eq!(config1.instructions, config2.instructions);
        assert_eq!(config1.document_format, config2.document_format);
    }

    #[test]
    fn test_project_config_debug() {
        let config = ProjectConfig::default_for_new_project();
        let debug_str = format!("{:?}", config);

        assert!(debug_str.contains("ProjectConfig"));
        assert!(debug_str.contains("tech"));
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

    #[test]
    fn test_memory_types_ordering() {
        let config = ProjectConfig::default_for_new_project();

        assert_eq!(config.memory_types[0], "tech");
        assert_eq!(config.memory_types[1], "project-tech");
        assert_eq!(config.memory_types[2], "domain");
        assert_eq!(config.memory_types[3], "workflow");
        assert_eq!(config.memory_types[4], "decision");
    }

    #[test]
    fn test_default_instructions_content() {
        let config = ProjectConfig::default_for_new_project();

        assert!(config.instructions.contains("tech:"));
        assert!(config.instructions.contains("project-tech:"));
        assert!(config.instructions.contains("domain:"));
        assert!(config.instructions.contains("workflow:"));
        assert!(config.instructions.contains("decision:"));
    }

    #[test]
    fn test_custom_memory_types() {
        let mut config = ProjectConfig::default_for_new_project();
        config.memory_types.push("custom".to_string());

        assert!(config.validate_memory_type("custom"));
        assert_eq!(config.memory_types.len(), 6);
    }

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
