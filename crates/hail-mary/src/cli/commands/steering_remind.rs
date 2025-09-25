use std::io::{self, Read};

use anyhow::Result;
use serde_json::json;

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
            match io::stdin().read_to_string(&mut buffer) {
                Ok(_) => buffer,
                Err(e) => {
                    // In hook mode, handle errors gracefully
                    if self.post_tool_use {
                        // PostToolUse: output valid empty JSON
                        let empty_json = json!({
                            "hookSpecificOutput": {
                                "hookEventName": "PostToolUse",
                                "additionalContext": "No custom steering patterns configured."
                            }
                        });
                        println!(
                            "{}",
                            serde_json::to_string(&empty_json).unwrap_or_else(|_| "{}".to_string())
                        );
                    }
                    // Log error to stderr for debugging
                    eprintln!("Failed to read stdin: {}", e);
                    return Ok(()); // Exit with code 0
                }
            }
        } else if let Some(ref input) = self.input {
            input.clone()
        } else {
            return Err(anyhow::anyhow!(
                "Please provide input either as argument or via --user-prompt-submit/--post-tool-use"
            ));
        };

        // PostToolUse mode: validate JSON structure
        if self.post_tool_use {
            // Try to parse as JSON and validate structure
            match serde_json::from_str::<serde_json::Value>(&user_input) {
                Ok(json_value) => {
                    // Check if tool_name field exists
                    if json_value.get("tool_name").is_none() {
                        // Invalid format: output valid empty JSON
                        let empty_json = json!({
                            "hookSpecificOutput": {
                                "hookEventName": "PostToolUse",
                                "additionalContext": "No custom steering patterns configured."
                            }
                        });
                        println!(
                            "{}",
                            serde_json::to_string(&empty_json).unwrap_or_else(|_| "{}".to_string())
                        );
                        return Ok(());
                    }
                }
                Err(_) => {
                    // JSON parse error: output valid empty JSON
                    let empty_json = json!({
                        "hookSpecificOutput": {
                            "hookEventName": "PostToolUse",
                            "additionalContext": "No custom steering patterns configured."
                        }
                    });
                    println!(
                        "{}",
                        serde_json::to_string(&empty_json).unwrap_or_else(|_| "{}".to_string())
                    );
                    return Ok(());
                }
            }
        }

        // Hook mode: check if .kiro directory exists
        if is_hook {
            let current_dir = std::env::current_dir()?;
            let kiro_dir = current_dir.join(".kiro");

            if !kiro_dir.exists() {
                // No .kiro directory, just passthrough
                if self.post_tool_use {
                    // PostToolUse: output valid empty JSON
                    let empty_json = json!({
                        "hookSpecificOutput": {
                            "hookEventName": "PostToolUse",
                            "additionalContext": "No custom steering patterns configured."
                        }
                    });
                    println!(
                        "{}",
                        serde_json::to_string(&empty_json).unwrap_or_else(|_| "{}".to_string())
                    );
                } else {
                    // UserPromptSubmit: passthrough original input
                    print!("{}", user_input);
                }
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
                    if self.post_tool_use {
                        // PostToolUse: return valid empty JSON
                        let empty_json = json!({
                            "hookSpecificOutput": {
                                "hookEventName": "PostToolUse",
                                "additionalContext": "No custom steering patterns configured."
                            }
                        });
                        println!(
                            "{}",
                            serde_json::to_string(&empty_json).unwrap_or_else(|_| "{}".to_string())
                        );
                    } else {
                        // UserPromptSubmit: passthrough original input
                        print!("{}", user_input);
                    }
                    // Log error for debugging
                    eprintln!("Steering remind error: {}", e);
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

    #[test]
    fn test_steering_remind_command_post_tool_use_mode() {
        let cmd = SteeringRemindCommand::new(None, false, true);
        assert!(cmd.post_tool_use);
        assert!(!cmd.user_prompt_submit);
    }

    #[test]
    fn test_steering_remind_command_user_prompt_submit_mode() {
        let cmd = SteeringRemindCommand::new(None, true, false);
        assert!(cmd.user_prompt_submit);
        assert!(!cmd.post_tool_use);
    }
}
