use std::path::Path;

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

        let content = format!(
            r#"**Current**: {} (`{}`)

These files track the current feature's lifecycle:
- <tasks-file>{}</tasks-file> - Task tracking and timeline
- <brainstorming-file>{}</brainstorming-file> - Exploratory dialogue report
- <memo-file>{}</memo-file> - Internal notes (**DO NOT ACCESS**)"#,
            spec_name, path_str, tasks_path, brainstorming_path, memo_path
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
    fn test_system_prompt_content() {
        let spec_name = "test-feature";
        let spec_path = PathBuf::from(".kiro/specs/2025-01-01-test-feature");

        let prompt = SystemPrompt::new(spec_name, &spec_path);
        let content = prompt.as_str();

        assert!(content.contains("**Current**: test-feature"));
        assert!(content.contains("<tasks-file>"));
        assert!(content.contains("tasks.md</tasks-file>"));
        assert!(content.contains("<brainstorming-file>"));
        assert!(content.contains("<memo-file>"));
        assert!(content.contains("DO NOT ACCESS"));
    }
}
