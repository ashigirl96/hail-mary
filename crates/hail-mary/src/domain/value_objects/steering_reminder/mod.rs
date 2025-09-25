use std::fmt::Write;

use serde::{Deserialize, Serialize};
use serde_json::json;

/// Core domain entity representing a steering reminder
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SteeringReminder {
    /// Name of the steering document (e.g., "tech", "documentation")
    pub steering_name: String,

    /// Relevant sections from the steering document
    pub relevant_sections: Vec<String>,

    /// Reasoning for why this steering is relevant
    pub reasoning: String,

    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
}

impl SteeringReminder {
    /// Create a new SteeringReminder
    pub fn new(
        steering_name: String,
        relevant_sections: Vec<String>,
        reasoning: String,
        confidence: f64,
    ) -> Self {
        Self {
            steering_name,
            relevant_sections,
            reasoning,
            confidence,
        }
    }

    /// Check if the reminder meets the confidence threshold
    pub fn meets_threshold(&self, threshold: f64) -> bool {
        self.confidence >= threshold
    }

    /// Format as reminder message for output
    pub fn format_reminder(&self) -> String {
        format!(
            "Remember: <steering-{}>\nsections: {}\nreasoning: {}",
            self.steering_name,
            self.relevant_sections.join(", "),
            self.reasoning
        )
    }
}

/// Template for steering reminder output
pub struct SteeringReminderOutput;

impl SteeringReminderOutput {
    /// Get the template for steering reminder output
    pub fn template() -> &'static str {
        include_str!("template.md")
    }
}

/// Value object for formatting a collection of steering reminders
#[derive(Debug)]
pub struct SteeringReminders {
    reminders: Vec<SteeringReminder>,
}

impl SteeringReminders {
    /// Create a new collection of steering reminders
    pub fn new(reminders: Vec<SteeringReminder>) -> Self {
        Self { reminders }
    }

    /// Format reminders for UserPromptSubmit hook (current template format)
    pub fn format_user_prompt_submit(&self, user_input: &str) -> String {
        self.format_text(user_input)
    }

    /// Format reminders for PostToolUse hook (JSON format)
    pub fn format_post_tool_use(&self) -> String {
        // Core steering types to exclude from PostToolUse reminders
        const CORE_TYPES: &[&str] = &["product", "tech", "structure"];

        // Filter out core types, keep only user-defined custom steering
        let custom_steering_tags: Vec<String> = self
            .reminders
            .iter()
            .filter(|r| !CORE_TYPES.contains(&r.steering_name.as_str()))
            .map(|r| format!("<steering-{}>", r.steering_name))
            .collect();

        let context = if custom_steering_tags.is_empty() {
            // No custom steering configured, no reminder needed
            "No custom steering patterns configured.".to_string()
        } else {
            format!(
                "Reminder: Check adherence to steering patterns - {}",
                custom_steering_tags.join(", ")
            )
        };

        // Create JSON output for PostToolUse
        let output = json!({
            "hookSpecificOutput": {
                "hookEventName": "PostToolUse",
                "additionalContext": context
            }
        });

        serde_json::to_string(&output).unwrap_or_else(|_| "{}".to_string())
    }

    /// Format reminders as text output
    pub fn format_text(&self, user_input: &str) -> String {
        let mut output = String::new();

        if self.reminders.is_empty() {
            writeln!(
                &mut output,
                "No relevant steering sections found.\nuser input: {}",
                user_input
            )
            .unwrap();
            return output;
        }

        // Build the steering list - use simple bullet list
        let mut steering_list = String::new();
        for reminder in &self.reminders {
            writeln!(
                &mut steering_list,
                "- <steering-{}> - {}",
                reminder.steering_name, reminder.reasoning
            )
            .unwrap();
        }

        // Replace placeholder in template
        let template = SteeringReminderOutput::template();
        let formatted = template.replace("{steering_list}", &steering_list);

        write!(
            &mut output,
            "{}\n<user-input>{}</user-input>",
            formatted, user_input
        )
        .unwrap();

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_steering_reminder_creation() {
        let reminder = SteeringReminder::new(
            "tech".to_string(),
            vec![
                "Development Commands".to_string(),
                "Common Commands".to_string(),
            ],
            "User wants to run tests".to_string(),
            0.85,
        );

        assert_eq!(reminder.steering_name, "tech");
        assert_eq!(reminder.relevant_sections.len(), 2);
        assert_eq!(reminder.confidence, 0.85);
    }

    #[test]
    fn test_confidence_threshold() {
        let reminder = SteeringReminder::new(
            "documentation".to_string(),
            vec!["Code Block Formatting".to_string()],
            "Markdown output required".to_string(),
            0.75,
        );

        assert!(reminder.meets_threshold(0.7));
        assert!(reminder.meets_threshold(0.75));
        assert!(!reminder.meets_threshold(0.8));
    }

    #[test]
    fn test_format_reminder() {
        let reminder = SteeringReminder::new(
            "structure".to_string(),
            vec![
                "Directory Organization".to_string(),
                "File Naming".to_string(),
            ],
            "Creating new files".to_string(),
            0.9,
        );

        let formatted = reminder.format_reminder();
        assert!(formatted.contains("Remember: <steering-structure>"));
        assert!(formatted.contains("sections: Directory Organization, File Naming"));
        assert!(formatted.contains("reasoning: Creating new files"));
    }

    #[test]
    fn test_steering_reminders_format_text() {
        let reminders = vec![
            SteeringReminder::new(
                "tech".to_string(),
                vec!["Development Commands".to_string()],
                "User wants to run tests".to_string(),
                0.85,
            ),
            SteeringReminder::new(
                "documentation".to_string(),
                vec!["Markdown Standards".to_string()],
                "Writing docs".to_string(),
                0.75,
            ),
        ];

        let steering_reminders = SteeringReminders::new(reminders);

        // Test text formatting
        let text = steering_reminders.format_text("test input");
        assert!(text.contains("**Checked**"));
        assert!(text.contains("REQUIRED FIRST"));
        assert!(text.contains("<steering-tech>"));
        assert!(text.contains("<steering-documentation>"));
        assert!(text.contains("<user-input>test input</user-input>"));

        // Test empty reminders
        let empty_reminders = SteeringReminders::new(vec![]);
        let empty_text = empty_reminders.format_text("");
        assert!(empty_text.contains("No relevant steering sections found"));
    }

    #[test]
    fn test_format_user_prompt_submit() {
        let reminders = vec![SteeringReminder::new(
            "tech".to_string(),
            vec![],
            "Technology stack".to_string(),
            1.0,
        )];

        let steering_reminders = SteeringReminders::new(reminders);
        let output = steering_reminders.format_user_prompt_submit("test");

        // Should use the template format
        assert!(output.contains("**Checked**"));
        assert!(output.contains("<steering-tech>"));
        assert!(output.contains("<user-input>test</user-input>"));
    }

    #[test]
    fn test_format_post_tool_use() {
        let reminders = vec![
            SteeringReminder::new(
                "tech".to_string(),
                vec![],
                "Technology stack".to_string(),
                1.0,
            ),
            SteeringReminder::new(
                "rust-dev".to_string(),
                vec![],
                "Rust development tools".to_string(),
                1.0,
            ),
        ];

        let steering_reminders = SteeringReminders::new(reminders);
        let json_output = steering_reminders.format_post_tool_use();

        // Parse as JSON to verify structure
        let parsed: serde_json::Value = serde_json::from_str(&json_output).unwrap();

        assert!(parsed["hookSpecificOutput"]["hookEventName"] == "PostToolUse");
        let context = parsed["hookSpecificOutput"]["additionalContext"]
            .as_str()
            .unwrap();
        assert!(context.contains("Reminder: Check adherence to steering patterns"));
        assert!(!context.contains("<steering-tech>")); // Core type should be filtered out
        assert!(context.contains("<steering-rust-dev>")); // Custom type should be included
        assert!(!context.contains("Technology stack")); // Should not include reasoning
        assert!(!context.contains("Rust development tools")); // Should not include reasoning
    }

    #[test]
    fn test_format_post_tool_use_filters_core_types() {
        let reminders = vec![
            SteeringReminder::new(
                "product".to_string(),
                vec![],
                "Product overview".to_string(),
                1.0,
            ),
            SteeringReminder::new(
                "tech".to_string(),
                vec![],
                "Technology stack".to_string(),
                1.0,
            ),
            SteeringReminder::new(
                "structure".to_string(),
                vec![],
                "Code organization".to_string(),
                1.0,
            ),
            SteeringReminder::new(
                "prompt-engineering".to_string(),
                vec![],
                "Prompt patterns".to_string(),
                1.0,
            ),
        ];

        let steering_reminders = SteeringReminders::new(reminders);
        let json_output = steering_reminders.format_post_tool_use();

        let parsed: serde_json::Value = serde_json::from_str(&json_output).unwrap();
        let context = parsed["hookSpecificOutput"]["additionalContext"]
            .as_str()
            .unwrap();

        // Core types should be filtered out
        assert!(!context.contains("<steering-product>"));
        assert!(!context.contains("<steering-tech>"));
        assert!(!context.contains("<steering-structure>"));

        // Custom type should be included
        assert!(context.contains("<steering-prompt-engineering>"));
    }
}
