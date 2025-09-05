use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemPrompt {
    content: String,
}

impl SystemPrompt {
    pub fn new(spec_name: &str, spec_path: &Path) -> Self {
        let content = format!(
            r#"# Kiro Specification Context

You are working on a Kiro project specification. Your task is to implement the requirements defined in the specification files below.

## Current Specification

Name: {}
Path: {}

## Specification Files

<kiro_spec_name>{}</kiro_spec_name>
<kiro_spec_path>{}</kiro_spec_path>
<kiro_requirements_path>{}/requirements.md</kiro_requirements_path>
<kiro_design_path>{}/design.md</kiro_design_path>
<kiro_tasks_path>{}/tasks.md</kiro_tasks_path>
<kiro_memo_path>{}/memo.md</kiro_memo_path>

## File Descriptions

- **requirements.md**: Comprehensive requirements including user stories, acceptance criteria, and functional requirements
- **design.md**: Technical design with architecture decisions and implementation approach
- **tasks.md**: Implementation tasks with priorities and dependencies
- **memo.md**: Additional notes and context from the user

## Instructions

1. Read the requirements in <kiro_requirements_path/> to understand what needs to be built
2. Follow the technical approach in <kiro_design_path/>
3. Track your progress against tasks in <kiro_tasks_path/>
4. Consider any additional context in <kiro_memo_path/>

When you need to reference these files, use the XML tag paths provided above."#,
            spec_name,
            spec_path.display(),
            spec_name,
            spec_path.display(),
            spec_path.display(),
            spec_path.display(),
            spec_path.display(),
            spec_path.display()
        );

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
