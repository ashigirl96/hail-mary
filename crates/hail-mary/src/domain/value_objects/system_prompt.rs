use crate::domain::entities::steering::Steerings;
use std::path::Path;

const SYSTEM_PROMPT_TEMPLATE: &str = include_str!("system_prompt_template.md");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemPrompt {
    content: String,
}

impl SystemPrompt {
    pub fn new(spec_name: &str, spec_path: &Path, steerings: &Steerings) -> Self {
        let path_str = spec_path.display().to_string();

        // Format steering content using Display trait
        let steering_content = steerings.to_string();

        let content = SYSTEM_PROMPT_TEMPLATE
            .replace("{spec_name}", spec_name)
            .replace("{path_str}", &path_str)
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
    use crate::domain::entities::steering::{Criterion, Steering, SteeringType};
    use std::path::PathBuf;

    #[test]
    fn test_clone_and_equality() {
        let spec_name = "test-spec";
        let spec_path = PathBuf::from(".kiro/specs/test-spec");
        let steerings = Steerings(vec![]);

        let prompt1 = SystemPrompt::new(spec_name, &spec_path, &steerings);
        let prompt2 = prompt1.clone();
        let prompt3 = SystemPrompt::new(spec_name, &spec_path, &steerings);

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
        let prompt = SystemPrompt::new(spec_name, &spec_path, &steerings);
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
}
