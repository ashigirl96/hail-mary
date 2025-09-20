use std::io::{self, Read};

use anyhow::Result;
use serde_json;

use crate::application::use_cases::remind_steering::{SteeringRemindOptions, remind_steering};
use crate::cli::args::OutputFormat;
use crate::domain::entities::steering_reminder::SteeringReminder;
use crate::infrastructure::filesystem::path_manager::PathManager;
use crate::infrastructure::repositories::{
    AnthropicRepository, ConfigRepository, SteeringRepository,
};

pub struct SteeringRemindCommand {
    input: Option<String>,
    hook: bool,
    analyze: bool,
    format: OutputFormat,
}

impl SteeringRemindCommand {
    pub fn new(input: Option<String>, hook: bool, analyze: bool, format: OutputFormat) -> Self {
        Self {
            input,
            hook,
            analyze,
            format,
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
                if self.hook {
                    self.output_hook_reminders(reminders, &user_input)?;
                } else {
                    self.output_reminders(reminders)?;
                }
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

    fn output_reminders(&self, reminders: Vec<SteeringReminder>) -> Result<()> {
        match self.format {
            OutputFormat::Text => {
                if reminders.is_empty() {
                    println!("No relevant steering sections found.");
                } else if self.analyze {
                    // AI Analysis mode output
                    println!("📌 STEERING REMINDER (AI Analysis)\n");
                    for reminder in reminders {
                        println!("{}\n", reminder.format_reminder());
                    }
                } else {
                    // Light mode output
                    println!("📌 STEERING REMINDER\n");
                    println!("Available steering types for this project:");
                    for reminder in reminders {
                        println!(
                            "• <steering-{}> - {}",
                            reminder.steering_name, reminder.reasoning
                        );
                    }
                    println!("\n💡 Use these tags to reference project context in Claude Code.");
                }
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&reminders)?);
            }
        }
        Ok(())
    }

    fn output_hook_reminders(
        &self,
        reminders: Vec<SteeringReminder>,
        user_input: &str,
    ) -> Result<()> {
        // Output reminders in light mode format for hooks
        if !reminders.is_empty() {
            println!("📌 STEERING REMINDER\n");
            println!("Available steering types for this project:");
            for reminder in reminders {
                println!(
                    "• <steering-{}> - {}",
                    reminder.steering_name, reminder.reasoning
                );
            }
            println!("\n💡 Use these tags to reference project context in Claude Code.");
            println!();
        }

        // Always passthrough the original input
        print!("{}", user_input);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_steering_remind_command_new() {
        let cmd = SteeringRemindCommand::new(
            Some("test input".to_string()),
            false,
            false,
            OutputFormat::Text,
        );
        assert_eq!(cmd.input, Some("test input".to_string()));
        assert!(!cmd.hook);
        assert!(!cmd.analyze);
    }

    #[test]
    fn test_output_reminders_text() {
        let cmd = SteeringRemindCommand::new(None, false, false, OutputFormat::Text);
        let reminders = vec![SteeringReminder::new(
            "tech".to_string(),
            vec!["Development Commands".to_string()],
            "User wants to run tests".to_string(),
            0.85,
        )];

        let result = cmd.output_reminders(reminders);
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_reminders_json() {
        let cmd = SteeringRemindCommand::new(None, false, false, OutputFormat::Json);
        let reminders = vec![SteeringReminder::new(
            "documentation".to_string(),
            vec!["Code Block Formatting".to_string()],
            "Markdown output".to_string(),
            0.75,
        )];

        let result = cmd.output_reminders(reminders);
        assert!(result.is_ok());
    }
}
