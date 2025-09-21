use std::collections::HashMap;

const STEERING_ANALYSIS_PROMPT_TEMPLATE: &str = include_str!("prompt_template.md");

/// Value object for constructing steering analysis prompts
pub struct SteeringAnalysisPrompt {
    steering_contents: HashMap<String, String>,
}

impl SteeringAnalysisPrompt {
    /// Create a new SteeringAnalysisPrompt with steering contents
    pub fn new(steering_contents: HashMap<String, String>) -> Self {
        Self { steering_contents }
    }

    /// Build the system prompt for steering analysis
    pub fn build_system_prompt(&self) -> String {
        let steering_section = self
            .steering_contents
            .iter()
            .map(|(name, content)| format!("=== STEERING: {} ===\n{}\n", name, content))
            .collect::<Vec<_>>()
            .join("\n");

        STEERING_ANALYSIS_PROMPT_TEMPLATE.replace("{}", &steering_section)
    }

    /// Build a user message for analysis
    pub fn build_user_message(user_input: &str) -> String {
        format!(
            "Analyze this user input and identify relevant steering documents:\n\n{}",
            user_input
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_system_prompt() {
        let mut contents = HashMap::new();
        contents.insert(
            "tech".to_string(),
            "# Technology\nDevelopment commands...".to_string(),
        );
        contents.insert(
            "documentation".to_string(),
            "# Documentation\nMarkdown formatting...".to_string(),
        );

        let prompt = SteeringAnalysisPrompt::new(contents);
        let system_prompt = prompt.build_system_prompt();

        assert!(system_prompt.contains("=== STEERING: tech ==="));
        assert!(system_prompt.contains("=== STEERING: documentation ==="));
        assert!(system_prompt.contains("Remember: <steering-NAME>"));
        assert!(system_prompt.contains("confidence > 0.7"));
    }

    #[test]
    fn test_build_user_message() {
        let message = SteeringAnalysisPrompt::build_user_message("How do I run tests?");
        assert!(message.contains("How do I run tests?"));
        assert!(message.contains("Analyze this user input"));
    }
}
