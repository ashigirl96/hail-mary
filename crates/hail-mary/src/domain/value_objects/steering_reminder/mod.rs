use std::fmt::Write;

use serde::{Deserialize, Serialize};

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

    /// Format reminders as text output
    pub fn format_text(&self, analyze: bool, user_input: &str) -> String {
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

        // Build the steering list
        let mut steering_list = String::new();
        if analyze {
            // AI Analysis mode - use detailed format_reminder output
            for reminder in &self.reminders {
                writeln!(&mut steering_list, "{}", reminder.format_reminder()).unwrap();
            }
        } else {
            // Light mode - use simple bullet list
            for reminder in &self.reminders {
                writeln!(
                    &mut steering_list,
                    "- <steering-{}> - {}",
                    reminder.steering_name, reminder.reasoning
                )
                .unwrap();
            }
        }

        // Replace placeholder in template
        let template = SteeringReminderOutput::template();
        let formatted = template.replace("{steering_list}", &steering_list);

        write!(&mut output, "{}\nuser input: {}", formatted, user_input).unwrap();

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

        // Test normal mode
        let text = steering_reminders.format_text(false, "test input");
        assert!(text.contains("📌 STEERING REMINDER"));
        assert!(text.contains("⚠️ DIRECTIVE"));
        assert!(text.contains("<steering-tech>"));
        assert!(text.contains("<steering-documentation>"));
        assert!(text.contains("user input: test input"));

        // Test analyze mode - same template but different content in the list
        let text_analyze = steering_reminders.format_text(true, "test input");
        assert!(text_analyze.contains("📌 STEERING REMINDER"));
        assert!(text_analyze.contains("⚠️ DIRECTIVE"));
        assert!(text_analyze.contains("Remember: <steering-tech>")); // format_reminder output

        // Test empty reminders
        let empty_reminders = SteeringReminders::new(vec![]);
        let empty_text = empty_reminders.format_text(false, "");
        assert!(empty_text.contains("No relevant steering sections found"));
    }
}
