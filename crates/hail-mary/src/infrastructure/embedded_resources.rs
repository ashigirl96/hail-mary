//! Embedded resources for slash commands
//!
//! This module contains markdown files embedded at compile time for deployment
//! to projects during initialization.

/// Embedded slash command markdown files
pub struct EmbeddedSlashCommands;

impl EmbeddedSlashCommands {
    /// Steering remember command documentation
    const STEERING_REMEMBER: &'static str =
        include_str!("../../../../.claude/commands/hm/steering-remember.md");

    /// Steering command documentation
    const STEERING: &'static str = include_str!("../../../../.claude/commands/hm/steering.md");

    /// Steering merge command documentation
    const STEERING_MERGE: &'static str =
        include_str!("../../../../.claude/commands/hm/steering-merge.md");

    /// Returns all embedded slash command files as (filename, content) pairs
    pub fn get_all() -> Vec<(&'static str, &'static str)> {
        vec![
            ("steering-remember.md", Self::STEERING_REMEMBER),
            ("steering.md", Self::STEERING),
            ("steering-merge.md", Self::STEERING_MERGE),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_files_not_empty() {
        let files = EmbeddedSlashCommands::get_all();
        assert_eq!(files.len(), 3);

        for (name, content) in files {
            assert!(!name.is_empty(), "File name should not be empty");
            assert!(
                !content.is_empty(),
                "File content for {} should not be empty",
                name
            );
            assert!(
                name.ends_with(".md"),
                "File {} should be a markdown file",
                name
            );
        }
    }

    #[test]
    fn test_embedded_files_contain_expected_content() {
        let files = EmbeddedSlashCommands::get_all();

        // Check that steering-remember.md contains expected content
        let steering_remember = files
            .iter()
            .find(|(name, _)| *name == "steering-remember.md")
            .expect("steering-remember.md should exist");
        assert!(steering_remember.1.contains("Save new learnings"));

        // Check that steering.md contains expected content
        let steering = files
            .iter()
            .find(|(name, _)| *name == "steering.md")
            .expect("steering.md should exist");
        assert!(steering.1.contains("Kiro Steering Management"));

        // Check that steering-merge.md contains expected content
        let steering_merge = files
            .iter()
            .find(|(name, _)| *name == "steering-merge.md")
            .expect("steering-merge.md should exist");
        assert!(steering_merge.1.contains("Process and categorize"));
    }
}
