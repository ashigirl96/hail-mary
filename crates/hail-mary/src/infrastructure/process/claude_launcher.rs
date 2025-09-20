use anyhow::Result;
use std::process::Command;

pub struct ClaudeProcessLauncher;

impl ClaudeProcessLauncher {
    pub fn new() -> Self {
        Self
    }

    pub fn launch(&self, system_prompt: &str, no_danger: bool) -> Result<()> {
        // Check if claude command exists
        let claude_exists = Self::check_claude_availability()?;

        if !claude_exists {
            return Err(anyhow::anyhow!(
                "Claude Code CLI not found. Please install it first: https://claude.ai/code"
            ));
        }

        // Create inline settings JSON with UserPromptSubmit hook
        let settings_json = r#"{
  "hooks": {
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "jq -r '.prompt' | hail-mary steering remind --hook"
          }
        ]
      }
    ]
  }
}"#;

        // Use exec to replace current process with Claude Code
        // This preserves TTY access while allowing backgrounding via shell job control

        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;

            // On Unix systems, use exec to replace the current process
            let mut cmd = Command::new("claude");

            // Set environment variables
            cmd.env("DISABLE_INTERLEAVED_THINKING", "1")
                .env("DISABLE_MICROCOMPACT", "1")
                .env("FORCE_AUTO_BACKGROUND_TASKS", "1")
                .env("ENABLE_BACKGROUND_TASKS", "1")
                .env("CLAUDE_BASH_MAINTAIN_PROJECT_WORKING_DIR", "1");

            // Add arguments
            cmd.arg("--append-system-prompt")
                .arg(system_prompt)
                .arg("--model")
                .arg("opus")
                .arg("--permission-mode")
                .arg("plan")
                .arg("--settings")
                .arg(settings_json);

            // Conditionally add --dangerously-skip-permissions (add it unless --no-danger is specified)
            if !no_danger {
                cmd.arg("--dangerously-skip-permissions");
            }

            let error = cmd.exec(); // This never returns if successful

            // If we reach here, exec failed
            Err(anyhow::anyhow!("Failed to exec Claude Code: {}", error))
        }

        #[cfg(not(unix))]
        {
            // Fallback for non-Unix systems
            let mut cmd = Command::new("claude");

            // Set environment variables
            cmd.env("DISABLE_INTERLEAVED_THINKING", "1")
                .env("DISABLE_MICROCOMPACT", "1")
                .env("FORCE_AUTO_BACKGROUND_TASKS", "1")
                .env("ENABLE_BACKGROUND_TASKS", "1")
                .env("CLAUDE_BASH_MAINTAIN_PROJECT_WORKING_DIR", "1");

            // Add arguments
            cmd.arg("--append-system-prompt")
                .arg(system_prompt)
                .arg("--model")
                .arg("opus")
                .arg("--permission-mode")
                .arg("plan")
                .arg("--settings")
                .arg(settings_json);

            // Conditionally add --dangerously-skip-permissions (add it unless --no-danger is specified)
            if !no_danger {
                cmd.arg("--dangerously-skip-permissions");
            }

            cmd.spawn()
                .map_err(|e| anyhow::anyhow!("Failed to spawn Claude Code: {}", e))?;

            Ok(())
        }
    }

    fn check_claude_availability() -> Result<bool> {
        // Use 'which' on Unix-like systems, 'where' on Windows
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_launcher_new() {
        let launcher = ClaudeProcessLauncher::new();
        // Just ensure it can be created without panicking
        assert!(std::mem::size_of_val(&launcher) == 0);
    }

    #[test]
    fn test_claude_launcher_default() {
        let launcher = ClaudeProcessLauncher::new();
        // Just ensure default works
        assert!(std::mem::size_of_val(&launcher) == 0);
    }

    #[test]
    fn test_check_claude_availability() {
        // This test will depend on whether claude is actually installed
        // We just test that the function doesn't panic
        let result = ClaudeProcessLauncher::check_claude_availability();
        assert!(result.is_ok());
    }

    // Note: We don't test launch() method in unit tests as it would actually
    // try to launch Claude. This should be tested in integration tests with mocks.
}
