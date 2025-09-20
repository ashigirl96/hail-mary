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
}
