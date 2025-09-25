use std::io::{self, Read};

use anyhow::Result;

use crate::application::use_cases::remind_steering::remind_steering;
use crate::domain::value_objects::steering_reminder::SteeringReminders;
use crate::infrastructure::filesystem::path_manager::PathManager;
use crate::infrastructure::repositories::{ConfigRepository, SteeringRepository};

pub struct SteeringRemindCommand {
    input: Option<String>,
    user_prompt_submit: bool,
    post_tool_use: bool,
}

impl SteeringRemindCommand {
    pub fn new(input: Option<String>, user_prompt_submit: bool, post_tool_use: bool) -> Self {
        Self {
            input,
            user_prompt_submit,
            post_tool_use,
        }
    }

    pub async fn execute(&self) -> Result<()> {
        // Determine if we're in hook mode
        let is_hook = self.user_prompt_submit || self.post_tool_use;

        // Get input text
        let user_input = if is_hook {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            buffer
        } else if let Some(ref input) = self.input {
            input.clone()
        } else {
            return Err(anyhow::anyhow!(
                "Please provide input either as argument or via --user-prompt-submit/--post-tool-use"
            ));
        };

        // Hook mode: check if .kiro directory exists
        if is_hook {
            let current_dir = std::env::current_dir()?;
            let kiro_dir = current_dir.join(".kiro");

            if !kiro_dir.exists() {
                // No .kiro directory, just passthrough
                print!("{}", user_input);
                return Ok(());
            }
        }

        // Use current directory as project root
        let current_dir = std::env::current_dir()?;
        let path_manager = PathManager::new(current_dir);

        // Create repositories
        let steering_repo = SteeringRepository::new(path_manager.clone());
        let config_repo = ConfigRepository::new(path_manager);

        // Execute remind use case
        match remind_steering(&steering_repo, &config_repo).await {
            Ok(reminders) => {
                let steering_reminders = SteeringReminders::new(reminders);

                // Choose output format based on flags
                let output = if self.post_tool_use {
                    steering_reminders.format_post_tool_use()
                } else {
                    // Default to user_prompt_submit format (current format)
                    steering_reminders.format_user_prompt_submit(&user_input)
                };

                print!("{}", output);

                Ok(())
            }
            Err(e) => {
                if is_hook {
                    // In hook mode, silently fail and passthrough
                    print!("{}", user_input);
                    Ok(())
                } else {
                    Err(anyhow::anyhow!(e))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_steering_remind_command_new() {
        let cmd = SteeringRemindCommand::new(Some("test input".to_string()), false, false);
        assert_eq!(cmd.input, Some("test input".to_string()));
        assert!(!cmd.user_prompt_submit);
        assert!(!cmd.post_tool_use);
    }
}
