use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "hail-mary")]
#[command(about = "Memory MCP and project management system")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate shell completion scripts
    #[command(name = "shell-completions")]
    Completion {
        /// Shell type to generate completions for
        shell: Shell,
    },

    /// Mark specifications as complete
    Complete,

    /// Launch Claude Code with Kiro specification context
    Code {
        /// Skip the dangerous permissions flag (--dangerously-skip-permissions)
        #[arg(long)]
        no_danger: bool,
        /// Continue previous Claude conversation (passes --continue flag)
        #[arg(short = 'c', long = "continue")]
        continue_conversation: bool,
    },

    /// Steering system management
    Steering {
        #[command(subcommand)]
        command: SteeringCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum SteeringCommands {
    /// Create a backup of steering files
    Backup,

    /// Remind relevant steering sections based on input
    Remind {
        /// Input text (for hook mode, reads from stdin)
        #[arg(value_name = "INPUT", conflicts_with_all = ["user_prompt_submit", "post_tool_use"])]
        input: Option<String>,

        /// UserPromptSubmit hook mode: read from stdin and format for template
        #[arg(long, conflicts_with = "post_tool_use")]
        user_prompt_submit: bool,

        /// PostToolUse hook mode: read from stdin and format as JSON
        #[arg(long, conflicts_with = "user_prompt_submit")]
        post_tool_use: bool,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Shell {
    /// Bash shell completions
    Bash,
    /// Zsh shell completions
    Zsh,
    /// Fish shell completions
    Fish,
    /// PowerShell completions
    PowerShell,
    /// Elvish shell completions
    Elvish,
}

impl Commands {
    pub fn is_completion(&self) -> bool {
        matches!(self, Commands::Completion { .. })
    }

    pub fn is_complete(&self) -> bool {
        matches!(self, Commands::Complete)
    }

    pub fn is_code(&self) -> bool {
        matches!(self, Commands::Code { .. })
    }
}

impl Commands {
    pub fn get_completion_shell(&self) -> Option<&Shell> {
        match self {
            Commands::Completion { shell } => Some(shell),
            _ => None,
        }
    }

    pub fn get_code_no_danger(&self) -> Option<bool> {
        match self {
            Commands::Code { no_danger, .. } => Some(*no_danger),
            _ => None,
        }
    }

    pub fn get_code_continue(&self) -> Option<bool> {
        match self {
            Commands::Code {
                continue_conversation,
                ..
            } => Some(*continue_conversation),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commands_is_methods() {
        let code_cmd = Commands::Code {
            no_danger: false,
            continue_conversation: false,
        };
        assert!(code_cmd.is_code());
    }
}
