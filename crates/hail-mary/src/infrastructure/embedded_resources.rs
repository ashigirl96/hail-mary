//! Embedded resources for slash commands and agents
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

    /// Requirements command documentation
    const REQUIREMENTS: &'static str =
        include_str!("../../../../.claude/commands/hm/requirements.md");

    /// Investigate command documentation
    const INVESTIGATE: &'static str =
        include_str!("../../../../.claude/commands/hm/investigate.md");

    /// Interactive investigate command documentation
    const INTERACTIVE_INVESTIGATE: &'static str =
        include_str!("../../../../.claude/commands/hm/interactive-investigate.md");

    /// Returns all embedded slash command files as (filename, content) pairs
    pub fn get_all() -> Vec<(&'static str, &'static str)> {
        vec![
            ("steering-remember.md", Self::STEERING_REMEMBER),
            ("steering.md", Self::STEERING),
            ("requirements.md", Self::REQUIREMENTS),
            ("investigate.md", Self::INVESTIGATE),
            ("interactive-investigate.md", Self::INTERACTIVE_INVESTIGATE),
        ]
    }
}

/// Embedded agent markdown files
pub struct EmbeddedAgents;

impl EmbeddedAgents {
    /// Steering investigator agent
    const STEERING_INVESTIGATOR: &'static str =
        include_str!("../../../../.claude/agents/steering-investigator.md");

    /// Root cause investigator agent
    const ROOT_CAUSE_INVESTIGATOR: &'static str =
        include_str!("../../../../.claude/agents/root-cause-investigator.md");

    /// Returns all embedded agent files as (filename, content) pairs
    pub fn get_all() -> Vec<(&'static str, &'static str)> {
        vec![
            ("steering-investigator.md", Self::STEERING_INVESTIGATOR),
            ("root-cause-investigator.md", Self::ROOT_CAUSE_INVESTIGATOR),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_commands_not_empty() {
        let files = EmbeddedSlashCommands::get_all();
        assert_eq!(files.len(), 5);

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
    fn test_embedded_agents_not_empty() {
        let files = EmbeddedAgents::get_all();
        assert_eq!(files.len(), 2);

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
}
