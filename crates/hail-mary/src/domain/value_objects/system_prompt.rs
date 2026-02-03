use std::path::Path;

const BASE_TEMPLATE: &str = include_str!("system_prompt_template.md");

const SPEC_FILES_TEMPLATE: &str = r#"# Spec Files

**Current**: {spec_name} (`{spec_path}`)

These files track the current feature's lifecycle:
- <tasks-file>{tasks_path}</tasks-file> - Task tracking and timeline
- <brainstorming-file>{brainstorming_path}</brainstorming-file> - Exploratory dialogue report
- <memo-file>{memo_path}</memo-file> - Internal notes (**DO NOT ACCESS**)

"#;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemPrompt {
    content: String,
}

impl SystemPrompt {
    /// Create a new SystemPrompt with spec context
    pub fn new(spec_name: &str, spec_path: &Path) -> Self {
        let path_str = spec_path.display().to_string();

        let tasks_path = format!("{}/tasks.md", path_str);
        let brainstorming_path = format!("{}/brainstorming.md", path_str);
        let memo_path = format!("{}/memo.md", path_str);

        let spec_section = SPEC_FILES_TEMPLATE
            .replace("{spec_name}", spec_name)
            .replace("{spec_path}", &path_str)
            .replace("{tasks_path}", &tasks_path)
            .replace("{brainstorming_path}", &brainstorming_path)
            .replace("{memo_path}", &memo_path);

        let content = format!("{}{}", spec_section, BASE_TEMPLATE);

        Self { content }
    }

    /// Create a new SystemPrompt without spec context (base template only)
    pub fn base() -> Self {
        Self {
            content: BASE_TEMPLATE.to_string(),
        }
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
    fn test_system_prompt_with_spec() {
        let spec_name = "test-feature";
        let spec_path = PathBuf::from(".kiro/specs/2025-01-01-test-feature");

        let prompt = SystemPrompt::new(spec_name, &spec_path);
        let content = prompt.as_str();

        assert!(content.contains("# Spec Files"));
        assert!(content.contains("**Current**: test-feature"));
        assert!(content.contains("<tasks-file>"));
        assert!(content.contains("tasks.md</tasks-file>"));
        assert!(content.contains("<brainstorming-file>"));
        assert!(content.contains("<memo-file>"));
        assert!(content.contains("DO NOT ACCESS"));
        assert!(content.contains("# Role"));
    }

    #[test]
    fn test_system_prompt_base_only() {
        let prompt = SystemPrompt::base();
        let content = prompt.as_str();

        assert!(!content.contains("# Spec Files"));
        assert!(!content.contains("<tasks-file>"));
        assert!(content.contains("# Role"));
        assert!(content.contains("# Core Behaviors"));
    }
}
