use anyhow::Result;
use std::process::Command;

pub struct ClaudeProcessLauncher;

impl ClaudeProcessLauncher {
    pub fn new() -> Self {
        Self
    }

    pub fn launch(&self, no_danger: bool, continue_conversation: bool) -> Result<()> {
        let claude_exists = Self::check_claude_availability()?;

        if !claude_exists {
            return Err(anyhow::anyhow!(
                "Claude Code CLI not found. Please install it first: https://claude.ai/code"
            ));
        }

        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;

            let mut cmd = Command::new("claude");

            if !no_danger {
                cmd.arg("--dangerously-skip-permissions");
            }

            if continue_conversation {
                cmd.arg("--continue");
            }

            let error = cmd.exec();
            Err(anyhow::anyhow!("Failed to exec Claude Code: {}", error))
        }

        #[cfg(not(unix))]
        {
            let mut cmd = Command::new("claude");

            if !no_danger {
                cmd.arg("--dangerously-skip-permissions");
            }

            if continue_conversation {
                cmd.arg("--continue");
            }

            cmd.spawn()
                .map_err(|e| anyhow::anyhow!("Failed to spawn Claude Code: {}", e))?;

            Ok(())
        }
    }

    fn check_claude_availability() -> Result<bool> {
        let command = if cfg!(target_os = "windows") {
            "where"
        } else {
            "which"
        };

        let output = Command::new(command).arg("claude").output()?;

        Ok(output.status.success())
    }
}

impl Default for ClaudeProcessLauncher {
    fn default() -> Self {
        Self::new()
    }
}
