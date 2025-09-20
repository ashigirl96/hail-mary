use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::env;

use anthropic_client::{Message, OAuthAuth, complete_with_system, load_auth};

use crate::application::repositories::AnthropicRepositoryInterface;
use crate::domain::entities::steering_reminder::SteeringReminder;
use crate::domain::value_objects::steering_analysis_prompt::SteeringAnalysisPrompt;

/// Concrete implementation of Anthropic repository
pub struct AnthropicRepository {
    model: String,
    auth: Option<OAuthAuth>,
}

impl AnthropicRepository {
    /// Create a new AnthropicRepository
    pub fn new() -> Self {
        let model =
            env::var("STEERING_MODEL").unwrap_or_else(|_| "claude-3-5-haiku-20241022".to_string());

        Self { model, auth: None }
    }

    /// Initialize authentication if not already done
    #[allow(dead_code)]
    async fn ensure_auth(&mut self) -> Result<()> {
        if self.auth.is_none() {
            self.auth = Some(load_auth().await?);
        }
        Ok(())
    }

    /// Parse the response from Anthropic into SteeringReminder entities
    fn parse_response(&self, response: &str) -> Result<Vec<SteeringReminder>> {
        let mut reminders = Vec::new();
        let lines: Vec<&str> = response.lines().collect();

        let mut i = 0;
        while i < lines.len() {
            if lines[i].starts_with("Remember: ") {
                // Extract steering name - handle both formats: "Remember: tech" and "Remember: <steering-tech>"
                let name = if lines[i].contains("<steering-") {
                    lines[i]
                        .trim_start_matches("Remember: <steering-")
                        .trim_end_matches('>')
                        .trim()
                } else {
                    lines[i].trim_start_matches("Remember: ").trim()
                };

                let mut sections = Vec::new();
                let mut reasoning = String::new();
                let mut confidence = 0.8; // Default confidence

                // Parse sections
                if i + 1 < lines.len() && lines[i + 1].starts_with("sections:") {
                    sections = lines[i + 1]
                        .trim_start_matches("sections:")
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    i += 1;
                }

                // Parse reasoning
                if i + 1 < lines.len() && lines[i + 1].starts_with("reasoning:") {
                    reasoning = lines[i + 1]
                        .trim_start_matches("reasoning:")
                        .trim()
                        .to_string();
                    i += 1;
                }

                // Optional: parse confidence if present
                if i + 1 < lines.len() && lines[i + 1].starts_with("confidence:") {
                    if let Ok(conf) = lines[i + 1]
                        .trim_start_matches("confidence:")
                        .trim()
                        .parse::<f64>()
                    {
                        confidence = conf;
                    }
                    i += 1;
                }

                reminders.push(SteeringReminder::new(
                    name.to_string(),
                    sections,
                    reasoning,
                    confidence,
                ));
            }
            i += 1;
        }

        Ok(reminders)
    }
}

#[async_trait]
impl AnthropicRepositoryInterface for AnthropicRepository {
    async fn analyze_steering_relevance(
        &self,
        user_input: &str,
        steering_contents: HashMap<String, String>,
    ) -> Result<Vec<SteeringReminder>> {
        let start_total = std::time::Instant::now();

        // Ensure we have authentication
        let start_auth = std::time::Instant::now();
        let mut auth = if let Some(ref auth) = self.auth {
            auth.clone()
        } else {
            eprintln!("🔑 Loading authentication...");
            load_auth().await?
        };
        eprintln!("⏱️ Authentication took: {:?}", start_auth.elapsed());

        // Build the system prompt using the value object
        let start_prompt = std::time::Instant::now();
        let prompt_builder = SteeringAnalysisPrompt::new(steering_contents);
        let system_prompt = prompt_builder.build_system_prompt();
        eprintln!("📝 System prompt size: {} chars", system_prompt.len());
        eprintln!("⏱️ Prompt building took: {:?}", start_prompt.elapsed());

        // Create the user message
        let user_message = SteeringAnalysisPrompt::build_user_message(user_input);
        eprintln!("💬 User message: {}", user_input);

        // Call Anthropic API
        let start_api = std::time::Instant::now();
        eprintln!("📡 Calling Anthropic API with model: {}", self.model);
        let response = complete_with_system(
            &self.model,
            vec![system_prompt],
            vec![Message {
                role: "user".to_string(),
                content: user_message,
            }],
            &mut auth,
        )
        .await?;
        eprintln!("✅ API response received ({} chars)", response.len());
        eprintln!("⏱️ API call took: {:?}", start_api.elapsed());

        // Parse response into SteeringReminder entities
        let start_parse = std::time::Instant::now();
        let result = self.parse_response(&response)?;
        eprintln!("🎨 Parsed {} reminders", result.len());
        eprintln!("⏱️ Response parsing took: {:?}", start_parse.elapsed());

        eprintln!(
            "⏱️ Total analyze_steering_relevance took: {:?}",
            start_total.elapsed()
        );
        Ok(result)
    }
}

impl Default for AnthropicRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_response() {
        let repo = AnthropicRepository::new();
        let response = r#"Remember: <steering-tech>
sections: Development Commands, Common Commands
reasoning: User wants to run tests which requires 'just test' command

Remember: <steering-documentation>
sections: Code Block Formatting
reasoning: Results will be formatted in markdown requiring 4 backticks"#;

        let reminders = repo.parse_response(response).unwrap();
        assert_eq!(reminders.len(), 2);

        assert_eq!(reminders[0].steering_name, "tech");
        assert_eq!(
            reminders[0].relevant_sections,
            vec!["Development Commands", "Common Commands"]
        );
        assert!(reminders[0].reasoning.contains("just test"));

        assert_eq!(reminders[1].steering_name, "documentation");
        assert_eq!(
            reminders[1].relevant_sections,
            vec!["Code Block Formatting"]
        );
        assert!(reminders[1].reasoning.contains("4 backticks"));
    }

    #[test]
    fn test_parse_response_with_confidence() {
        let repo = AnthropicRepository::new();
        let response = r#"Remember: <steering-structure>
sections: Directory Organization
reasoning: Creating new files
confidence: 0.95"#;

        let reminders = repo.parse_response(response).unwrap();
        assert_eq!(reminders.len(), 1);
        assert_eq!(reminders[0].confidence, 0.95);
    }
}
