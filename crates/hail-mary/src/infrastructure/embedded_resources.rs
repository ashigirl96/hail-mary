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

    /// Returns all embedded slash command files as (filename, content) pairs
    pub fn get_all() -> Vec<(&'static str, &'static str)> {
        vec![
            ("steering-remember.md", Self::STEERING_REMEMBER),
            ("steering.md", Self::STEERING),
        ]
    }
}

/// Embedded spec command markdown files
pub struct EmbeddedSpecCommands;

impl EmbeddedSpecCommands {
    /// Requirements command documentation
    const REQUIREMENTS: &'static str =
        include_str!("../../../../.claude/commands/spec/requirements.md");

    /// Investigate command documentation
    const INVESTIGATE: &'static str =
        include_str!("../../../../.claude/commands/spec/investigate.md");

    /// Design command documentation
    const DESIGN: &'static str = include_str!("../../../../.claude/commands/spec/design.md");

    /// Timeline command documentation
    const TIMELINE: &'static str = include_str!("../../../../.claude/commands/spec/timeline.md");

    /// Status command documentation
    const STATUS: &'static str = include_str!("../../../../.claude/commands/spec/status.md");

    /// Returns all embedded spec command files as (filename, content) pairs
    pub fn get_all() -> Vec<(&'static str, &'static str)> {
        vec![
            ("requirements.md", Self::REQUIREMENTS),
            ("investigate.md", Self::INVESTIGATE),
            ("design.md", Self::DESIGN),
            ("timeline.md", Self::TIMELINE),
            ("status.md", Self::STATUS),
        ]
    }
}

/// Embedded PBI command markdown files
pub struct EmbeddedPbiCommands;

impl EmbeddedPbiCommands {
    /// Decompose PBI command documentation
    const DECOMPOSE: &'static str = include_str!("../../../../.claude/commands/pbi/decompose.md");

    /// Add SBI command documentation
    const ADD_SBI: &'static str = include_str!("../../../../.claude/commands/pbi/add-sbi.md");

    /// Returns all embedded PBI command files as (filename, content) pairs
    pub fn get_all() -> Vec<(&'static str, &'static str)> {
        vec![
            ("decompose.md", Self::DECOMPOSE),
            ("add-sbi.md", Self::ADD_SBI),
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

    /// Backend architect agent
    const BACKEND_ARCHITECT: &'static str =
        include_str!("../../../../.claude/agents/backend-architect.md");

    /// Frontend architect agent
    const FRONTEND_ARCHITECT: &'static str =
        include_str!("../../../../.claude/agents/frontend-architect.md");

    /// System architect agent
    const SYSTEM_ARCHITECT: &'static str =
        include_str!("../../../../.claude/agents/system-architect.md");

    /// Evidence collector agent
    const EVIDENCE_COLLECTOR: &'static str =
        include_str!("../../../../.claude/agents/evidence-collector.md");

    /// Returns all embedded agent files as (filename, content) pairs
    pub fn get_all() -> Vec<(&'static str, &'static str)> {
        vec![
            ("steering-investigator.md", Self::STEERING_INVESTIGATOR),
            ("root-cause-investigator.md", Self::ROOT_CAUSE_INVESTIGATOR),
            ("backend-architect.md", Self::BACKEND_ARCHITECT),
            ("frontend-architect.md", Self::FRONTEND_ARCHITECT),
            ("system-architect.md", Self::SYSTEM_ARCHITECT),
            ("evidence-collector.md", Self::EVIDENCE_COLLECTOR),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_commands_not_empty() {
        let files = EmbeddedSlashCommands::get_all();
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

    #[test]
    fn test_embedded_spec_commands_not_empty() {
        let files = EmbeddedSpecCommands::get_all();
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
    fn test_embedded_pbi_commands_not_empty() {
        let files = EmbeddedPbiCommands::get_all();
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

    #[test]
    fn test_embedded_agents_not_empty() {
        let files = EmbeddedAgents::get_all();
        assert_eq!(files.len(), 6);

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
