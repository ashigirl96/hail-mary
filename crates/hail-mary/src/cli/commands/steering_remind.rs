use std::io::{self, Read};

use anyhow::Result;

use crate::application::use_cases::remind_steering::{SteeringRemindOptions, remind_steering};
use crate::domain::value_objects::steering_reminder::SteeringReminders;
use crate::infrastructure::filesystem::path_manager::PathManager;
use crate::infrastructure::repositories::{
    AnthropicRepository, ConfigRepository, SteeringRepository,
};

pub struct SteeringRemindCommand {
    input: Option<String>,
    hook: bool,
    analyze: bool,
}

impl SteeringRemindCommand {
    pub fn new(input: Option<String>, hook: bool, analyze: bool) -> Self {
        Self {
            input,
            hook,
            analyze,
        }
    }

    pub async fn execute(&self) -> Result<()> {
        // Get input text
        let user_input = if self.hook {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            buffer
        } else if let Some(ref input) = self.input {
            input.clone()
        } else {
            return Err(anyhow::anyhow!(
                "Please provide input either as argument or via --hook"
            ));
        };

        // Hook mode: check if .kiro directory exists
        if self.hook {
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
        let anthropic_repo = AnthropicRepository::new();

        // Create options
        let options = SteeringRemindOptions {
            analyze_mode: self.analyze,
        };

        // Execute remind use case
        match remind_steering(
            &user_input,
            &steering_repo,
            &anthropic_repo,
            &config_repo,
            options,
        )
        .await
        {
            Ok(reminders) => {
                let steering_reminders = SteeringReminders::new(reminders);
                let output = steering_reminders.format_text(self.analyze, &user_input);
                print!("{}", output);

                Ok(())
            }
            Err(e) => {
                if self.hook {
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
        assert!(!cmd.hook);
        assert!(!cmd.analyze);
    }
}
