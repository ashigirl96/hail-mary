use std::path::Path;

const SYSTEM_PROMPT_TEMPLATE: &str = include_str!("system_prompt_template.md");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemPrompt {
    content: String,
}

impl SystemPrompt {
    pub fn new(spec_name: &str, spec_path: &Path) -> Self {
        let path_str = spec_path.display().to_string();

        let content = SYSTEM_PROMPT_TEMPLATE
            .replace("{spec_name}", spec_name)
            .replace("{path_str}", &path_str);

        Self { content }
    }

    pub fn as_str(&self) -> &str {
        &self.content
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_new_system_prompt() {
        let spec_name = "2025-09-05-test-feature";
        let spec_path = PathBuf::from(".kiro/specs/2025-09-05-test-feature");

        let prompt = SystemPrompt::new(spec_name, &spec_path);
        let content = prompt.as_str();

        assert!(content.contains("# Kiro Specification Context"));
        assert!(content.contains("2025-09-05-test-feature"));
        assert!(content.contains("<kiro_spec_name>2025-09-05-test-feature</kiro_spec_name>"));
        assert!(content.contains("<kiro_requirements_path>.kiro/specs/2025-09-05-test-feature/requirements.md</kiro_requirements_path>"));
        assert!(content.contains(
            "When you need to reference these files, use the XML tag paths provided above."
        ));
    }

    #[test]
    fn test_xml_tags_are_properly_formatted() {
        let spec_name = "my-feature";
        let spec_path = PathBuf::from(".kiro/specs/my-feature");

        let prompt = SystemPrompt::new(spec_name, &spec_path);
        let content = prompt.as_str();

        // Check all XML tags are present
        assert!(content.contains("<kiro_spec_name>my-feature</kiro_spec_name>"));
        assert!(content.contains("<kiro_spec_path>.kiro/specs/my-feature</kiro_spec_path>"));
        assert!(content.contains("<kiro_requirements_path>.kiro/specs/my-feature/requirements.md</kiro_requirements_path>"));
        assert!(
            content
                .contains("<kiro_design_path>.kiro/specs/my-feature/design.md</kiro_design_path>")
        );
        assert!(
            content.contains("<kiro_tasks_path>.kiro/specs/my-feature/tasks.md</kiro_tasks_path>")
        );
        assert!(
            content.contains("<kiro_memo_path>.kiro/specs/my-feature/memo.md</kiro_memo_path>")
        );
        assert!(
            content.contains("<kiro_investigation_path>.kiro/specs/my-feature/investigation.md</kiro_investigation_path>")
        );
    }

    #[test]
    fn test_clone_and_equality() {
        let spec_name = "test-spec";
        let spec_path = PathBuf::from(".kiro/specs/test-spec");

        let prompt1 = SystemPrompt::new(spec_name, &spec_path);
        let prompt2 = prompt1.clone();
        let prompt3 = SystemPrompt::new(spec_name, &spec_path);

        assert_eq!(prompt1, prompt2);
        assert_eq!(prompt1, prompt3);
    }
}
