use crate::domain::value_objects::steering::Steerings;
use std::path::Path;

const SYSTEM_PROMPT_TEMPLATE: &str = include_str!("template.md");
const SPECIFICATION_SECTION_TEMPLATE: &str = include_str!("specification_section_template.md");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemPrompt {
    content: String,
}

impl SystemPrompt {
    pub fn new(spec_name: Option<&str>, spec_path: Option<&Path>, steerings: &Steerings) -> Self {
        // Format steering content using Display trait
        let steering_content = steerings.to_string();

        // Generate specification section if spec is provided
        let specification_section = if let (Some(name), Some(path)) = (spec_name, spec_path) {
            let path_str = path.display().to_string();
            SPECIFICATION_SECTION_TEMPLATE
                .replace("{spec_name}", name)
                .replace("{spec_path}", &path_str)
        } else {
            // No specification section when running without spec
            String::new()
        };

        let content = SYSTEM_PROMPT_TEMPLATE
            .replace("{specification_section}", &specification_section)
            .replace("{steering_content}", &steering_content);

        Self { content }
    }

    pub fn as_str(&self) -> &str {
        &self.content
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::steering::{Criterion, Steering, SteeringType};
    use std::path::PathBuf;

    #[test]
    fn test_clone_and_equality() {
        let spec_name = "test-spec";
        let spec_path = PathBuf::from(".kiro/specs/test-spec");
        let steerings = Steerings(vec![]);

        let prompt1 = SystemPrompt::new(Some(spec_name), Some(&spec_path), &steerings);
        let prompt2 = prompt1.clone();
        let prompt3 = SystemPrompt::new(Some(spec_name), Some(&spec_path), &steerings);

        assert_eq!(prompt1, prompt2);
        assert_eq!(prompt1, prompt3);
    }

    #[test]
    fn test_system_prompt_with_steerings() {
        let spec_name = "test-spec";
        let spec_path = PathBuf::from(".kiro/specs/test-spec");

        let product_steering = Steering {
            steering_type: SteeringType {
                name: "product".to_string(),
                purpose: "Product overview".to_string(),
                criteria: vec![Criterion {
                    name: "Overview".to_string(),
                    description: "Brief description".to_string(),
                }],
                allowed_operations: vec![],
            },
            content: "Product content here".to_string(),
        };

        let tech_steering = Steering {
            steering_type: SteeringType {
                name: "tech".to_string(),
                purpose: "Technical stack".to_string(),
                criteria: vec![Criterion {
                    name: "Stack".to_string(),
                    description: "Technology choices".to_string(),
                }],
                allowed_operations: vec![],
            },
            content: "Tech content here".to_string(),
        };

        let steerings = Steerings(vec![product_steering, tech_steering]);
        let prompt = SystemPrompt::new(Some(spec_name), Some(&spec_path), &steerings);
        let content = prompt.as_str();

        // Check that steering content is included with individual tags
        assert!(content.contains("<steering-product>"));
        assert!(content.contains("Product content here"));
        assert!(content.contains("</steering-product>"));

        assert!(content.contains("<steering-tech>"));
        assert!(content.contains("Tech content here"));
        assert!(content.contains("</steering-tech>"));

        // Check that the old format is NOT present
        assert!(!content.contains("<steering>\n"));
    }

    #[test]
    fn test_system_prompt_without_spec() {
        let product_steering = Steering {
            steering_type: SteeringType {
                name: "product".to_string(),
                purpose: "Product overview".to_string(),
                criteria: vec![],
                allowed_operations: vec![],
            },
            content: "Product content".to_string(),
        };

        let steerings = Steerings(vec![product_steering]);
        let prompt = SystemPrompt::new(None, None, &steerings);
        let content = prompt.as_str();

        // Should not contain spec-related content
        assert!(!content.contains(".kiro/specs/"));
        assert!(!content.contains("requirements.md"));
        assert!(!content.contains("design.md"));
        assert!(!content.contains("## Specification"));

        // Should still contain steering content
        assert!(content.contains("<steering-product>"));
        assert!(content.contains("Product content"));
        assert!(content.contains("Kiro: Specification-Driven Development Context"));
    }

    #[test]
    fn test_system_prompt_with_spec() {
        let spec_name = "test-feature";
        let spec_path = PathBuf::from(".kiro/specs/2025-09-09-test-feature");
        let steerings = Steerings(vec![]);

        let prompt = SystemPrompt::new(Some(spec_name), Some(&spec_path), &steerings);
        let content = prompt.as_str();

        // Should contain spec section
        assert!(content.contains("## Specification"));
        assert!(content.contains(&format!("**Current**: {} (", spec_name)));
        assert!(content.contains(".kiro/specs/2025-09-09-test-feature/requirements.md"));
        assert!(content.contains(".kiro/specs/2025-09-09-test-feature/design.md"));
        assert!(content.contains(".kiro/specs/2025-09-09-test-feature/tasks.md"));
        assert!(content.contains(".kiro/specs/2025-09-09-test-feature/investigation.md"));
        assert!(content.contains(".kiro/specs/2025-09-09-test-feature/memo.md"));
    }
}
