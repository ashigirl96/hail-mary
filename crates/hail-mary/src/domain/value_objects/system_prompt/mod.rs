use crate::domain::value_objects::steering::Steerings;
use std::path::Path;

const BASE_TEMPLATE: &str = include_str!("base_template.md");
const STEERING_TEMPLATE: &str = include_str!("steering_template.md");

// Pattern Router templates
const PATTERN_ROUTER_INDEX: &str = include_str!("pattern_router/index.md");
const PATTERN_ROUTER_PHILOSOPHY: &str = include_str!("pattern_router/00_philosophy.md");
const PATTERN_ROUTER_PRINCIPLES: &str = include_str!("pattern_router/01_principles.md");
const PATTERN_ROUTER_HUB: &str = include_str!("pattern_router/02_hub.md");
const PATTERN_ROUTER_PATTERNS: &str = include_str!("pattern_router/03_patterns.md");
const PATTERN_ROUTER_WORKFLOWS: &str = include_str!("pattern_router/04_workflows.md");
const PATTERN_ROUTER_GATES: &str = include_str!("pattern_router/05_gates.md");
const PATTERN_ROUTER_NUDGES: &str = include_str!("pattern_router/06_nudges.md");
const PATTERN_ROUTER_REQUIREMENTS: &str = include_str!("pattern_router/07_requirements.md");
const PATTERN_ROUTER_INVESTIGATION: &str = include_str!("pattern_router/08_investigation.md");
const PATTERN_ROUTER_DESIGN: &str = include_str!("pattern_router/09_design.md");
const PATTERN_ROUTER_SPEC_FILES: &str = include_str!("pattern_router/10_spec_files.md");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemPrompt {
    content: String,
}

impl SystemPrompt {
    pub fn new(spec_name: Option<&str>, spec_path: Option<&Path>, steerings: &Steerings) -> Self {
        // Format steering content using Display trait
        let steering_content = steerings.to_string();

        // Build the system prompt by composing templates
        let mut content = String::new();

        // 1. Always start with base Kiro introduction
        content.push_str(BASE_TEMPLATE);
        content.push('\n');

        // 2. Add specification section if spec is provided
        if let (Some(name), Some(path)) = (spec_name, spec_path) {
            let path_str = path.display().to_string();

            // Build spec file paths
            let requirements_path = format!("{}/requirements.md", path_str);
            let design_path = format!("{}/design.md", path_str);
            let tasks_path = format!("{}/tasks.md", path_str);
            let investigation_path = format!("{}/investigation.md", path_str);
            let memo_path = format!("{}/memo.md", path_str);

            // Build spec_files section
            let spec_files_section = PATTERN_ROUTER_SPEC_FILES
                .replace("{spec_name}", name)
                .replace("{spec_path}", &path_str)
                .replace("{requirements_path}", &requirements_path)
                .replace("{design_path}", &design_path)
                .replace("{tasks_path}", &tasks_path)
                .replace("{investigation_path}", &investigation_path)
                .replace("{memo_path}", &memo_path);

            // Build the pattern router template by replacing all placeholders
            let specification_section = PATTERN_ROUTER_INDEX
                .replace("{philosophy}", PATTERN_ROUTER_PHILOSOPHY)
                .replace("{principles}", PATTERN_ROUTER_PRINCIPLES)
                .replace("{hub}", PATTERN_ROUTER_HUB)
                .replace("{patterns}", PATTERN_ROUTER_PATTERNS)
                .replace("{workflows}", PATTERN_ROUTER_WORKFLOWS)
                .replace("{gates}", PATTERN_ROUTER_GATES)
                .replace("{nudges}", PATTERN_ROUTER_NUDGES)
                .replace("{requirements}", PATTERN_ROUTER_REQUIREMENTS)
                .replace("{investigation}", PATTERN_ROUTER_INVESTIGATION)
                .replace("{design}", PATTERN_ROUTER_DESIGN)
                .replace("{spec_files}", &spec_files_section);

            content.push_str(&specification_section);
            content.push('\n');
        }

        // 3. Always add steering section
        let steering_section = STEERING_TEMPLATE.replace("{steering_content}", &steering_content);
        content.push_str(&steering_section);

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
        assert!(!content.contains("<requirements-file>"));
        assert!(!content.contains("<design-file>"));
        assert!(!content.contains("<kiro-spec-files>"));

        // Should still contain base and steering content
        assert!(content.contains("Kiro: Specification-Driven Development Context"));
        assert!(content.contains("## About Steering"));
        assert!(content.contains("<steering-product>"));
        assert!(content.contains("Product content"));
    }

    #[test]
    fn test_system_prompt_with_spec() {
        let spec_name = "test-feature";
        let spec_path = PathBuf::from(".kiro/specs/2025-09-09-test-feature");
        let steerings = Steerings(vec![]);

        let prompt = SystemPrompt::new(Some(spec_name), Some(&spec_path), &steerings);
        let content = prompt.as_str();

        // Should contain base template
        assert!(content.contains("Kiro: Specification-Driven Development Context"));

        // Should contain pattern router sections
        assert!(content.contains("<kiro-spec-driven>"));
        assert!(content.contains("<kiro-philosophy>"));
        assert!(content.contains("## Kiro Philosophy"));
        assert!(content.contains("<kiro-principles>"));
        assert!(content.contains("## Universal Principles"));
        assert!(content.contains("<kiro-hub>"));
        assert!(content.contains("## Tasks.md Central Hub"));

        // Should contain spec file references
        assert!(content.contains("<kiro-spec-files>"));
        assert!(content.contains("## Specification Files"));
        assert!(content.contains("<requirements-file>"));
        assert!(content.contains("<investigation-file>"));
        assert!(content.contains("<design-file>"));
        assert!(content.contains("<tasks-file>"));

        // Should contain steering section
        assert!(content.contains("## About Steering"));
    }
}
