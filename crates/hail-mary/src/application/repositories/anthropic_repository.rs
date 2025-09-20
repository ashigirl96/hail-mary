use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;

use crate::domain::entities::steering_reminder::SteeringReminder;

/// Repository interface for Anthropic AI service interactions
#[async_trait]
pub trait AnthropicRepositoryInterface: Send + Sync {
    /// Analyze user input and identify relevant steering documents
    async fn analyze_steering_relevance(
        &self,
        user_input: &str,
        steering_contents: HashMap<String, String>,
    ) -> Result<Vec<SteeringReminder>>;
}
