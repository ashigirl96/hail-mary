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
const PATTERN_ROUTER_BRAINSTORMING: &str = include_str!("pattern_router/10_brainstorming.md");
const PATTERN_ROUTER_SPEC_FILES: &str = include_str!("pattern_router/11_spec_files.md");
const PATTERN_ROUTER_SPEC_FILES_SBI: &str = include_str!("pattern_router/11_spec_files_sbi.md");

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
            // Check if this is an SBI context
            let is_sbi = is_sbi_context(path);

            // Build spec_files section based on context
            let spec_files_section = if is_sbi {
                build_sbi_spec_files(name, path)
            } else {
                build_pbi_spec_files(name, path)
            };

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
                .replace("{brainstorming}", PATTERN_ROUTER_BRAINSTORMING)
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

/// Check if the given path is an SBI context
fn is_sbi_context(spec_path: &Path) -> bool {
    // SBI path pattern: .kiro/specs/[pbi-name]/sbi-X-[title]
    if let Some(dir_name) = spec_path.file_name()
        && let Some(name_str) = dir_name.to_str()
    {
        return name_str.starts_with("sbi-");
    }
    false
}

/// Build spec_files section for PBI or single spec (non-SBI)
fn build_pbi_spec_files(spec_name: &str, spec_path: &Path) -> String {
    let path_str = spec_path.display().to_string();

    let requirements_path = format!("{}/requirements.md", path_str);
    let tasks_path = format!("{}/tasks.md", path_str);
    let brainstorming_path = format!("{}/brainstorming.md", path_str);
    let memo_path = format!("{}/memo.md", path_str);

    PATTERN_ROUTER_SPEC_FILES
        .replace("{spec_name}", spec_name)
        .replace("{spec_path}", &path_str)
        .replace("{requirements_path}", &requirements_path)
        .replace("{tasks_path}", &tasks_path)
        .replace("{brainstorming_path}", &brainstorming_path)
        .replace("{memo_path}", &memo_path)
}

/// Build spec_files section for SBI context
fn build_sbi_spec_files(sbi_name: &str, sbi_path: &Path) -> String {
    // Extract PBI path
    let pbi_path = sbi_path.parent().expect("SBI must have parent PBI");

    let pbi_path_str = pbi_path.display().to_string();
    let sbi_path_str = sbi_path.display().to_string();

    // SBI paths
    let sbi_requirements_path = format!("{}/requirements.md", sbi_path_str);
    let sbi_tasks_path = format!("{}/tasks.md", sbi_path_str);
    let sbi_brainstorming_path = format!("{}/brainstorming.md", sbi_path_str);
    let sbi_memo_path = format!("{}/memo.md", sbi_path_str);

    // PBI paths
    let pbi_requirements_path = format!("{}/requirements.md", pbi_path_str);

    PATTERN_ROUTER_SPEC_FILES_SBI
        .replace("{sbi_name}", sbi_name)
        .replace("{sbi_path}", &sbi_path_str)
        .replace("{sbi_requirements_path}", &sbi_requirements_path)
        .replace("{sbi_tasks_path}", &sbi_tasks_path)
        .replace("{sbi_brainstorming_path}", &sbi_brainstorming_path)
        .replace("{sbi_memo_path}", &sbi_memo_path)
        .replace("{pbi_requirements_path}", &pbi_requirements_path)
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
        assert!(content.contains("<tasks-file>"));

        // Should NOT contain removed file references
        assert!(!content.contains("<investigation-file>"));
        assert!(!content.contains("<design-file>"));

        // Should contain steering section
        assert!(content.contains("## About Steering"));

        // Should NOT contain PBI references (single spec)
        assert!(!content.contains("<pbi-file>"));
    }

    #[test]
    fn test_system_prompt_with_sbi() {
        let sbi_name = "sbi-1-backend-api";
        let sbi_path = PathBuf::from(".kiro/specs/payment-system/sbi-1-backend-api");
        let steerings = Steerings(vec![]);

        let prompt = SystemPrompt::new(Some(sbi_name), Some(&sbi_path), &steerings);
        let content = prompt.as_str();

        // Should contain SBI context markers
        assert!(content.contains("**Current Context**: SBI (Sprint Backlog Item)"));
        assert!(content.contains("**Current SBI**: sbi-1-backend-api"));

        // Should contain both SBI and PBI file references
        assert!(content.contains("<requirements-file>"));
        assert!(content.contains("<pbi-file>"));

        // Should contain PBI requirements path
        assert!(content.contains("payment-system/requirements.md"));

        // Should contain SBI paths
        assert!(content.contains("sbi-1-backend-api/requirements.md"));

        // Should NOT contain PBI investigation/design (removed from template)
        assert!(!content.contains("<pbi-investigation-file>"));
        assert!(!content.contains("<pbi-design-file>"));
    }

    #[test]
    fn test_system_prompt_with_brainstorming() {
        let spec_name = "test-brainstorm";
        let spec_path = PathBuf::from(".kiro/specs/test-brainstorm");
        let steerings = Steerings(vec![]);

        let prompt = SystemPrompt::new(Some(spec_name), Some(&spec_path), &steerings);
        let content = prompt.as_str();

        // Should contain brainstorming-file tag
        assert!(content.contains("<brainstorming-file>"));
        assert!(content.contains("test-brainstorm/brainstorming.md"));

        // Should contain BRAINSTORM pattern definition
        assert!(content.contains("<kiro-patterns>"));
        assert!(content.contains("BRAINSTORM Patterns"));

        // Should contain Brainstorm Pipeline definition
        assert!(content.contains("<kiro-workflows>"));
        assert!(content.contains("Brainstorm Pipeline"));

        // Should contain brainstorming document structure
        assert!(content.contains("<kiro-brainstorming>"));
        assert!(content.contains("Brainstorming Document Structure"));
    }

    #[test]
    fn test_brainstorm_pipeline_characteristics() {
        let spec_name = "test-brainstorm";
        let spec_path = PathBuf::from(".kiro/specs/test-brainstorm");
        let steerings = Steerings(vec![]);

        let prompt = SystemPrompt::new(Some(spec_name), Some(&spec_path), &steerings);
        let content = prompt.as_str();

        // Verify Brainstorm Pipeline has no Hub/Gates access
        assert!(content.contains("Hub/Gates access: None"));

        // Verify two nudge events
        assert!(content.contains("brainstorm:nudge-save"));
        assert!(content.contains("brainstorm:nudge-next"));
    }

    #[test]
    fn test_is_sbi_context() {
        let sbi_path = PathBuf::from(".kiro/specs/payment/sbi-1-backend");
        assert!(is_sbi_context(&sbi_path));

        let single_spec_path = PathBuf::from(".kiro/specs/payment");
        assert!(!is_sbi_context(&single_spec_path));

        let another_single_spec = PathBuf::from(".kiro/specs/2025-10-07-project");
        assert!(!is_sbi_context(&another_single_spec));
    }
}
