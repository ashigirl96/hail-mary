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
    /// Initialize a new project
    Init,

    /// Create a new feature specification
    New {
        /// Feature name in kebab-case
        name: String,
    },

    /// Generate shell completion scripts
    #[command(name = "shell-completions")]
    Completion {
        /// Shell type to generate completions for
        shell: Shell,
    },

    /// Mark feature specifications as complete
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
        /// Input text to analyze
        #[arg(value_name = "INPUT", conflicts_with = "hook")]
        input: Option<String>,

        /// Hook mode: read from stdin and passthrough
        #[arg(long)]
        hook: bool,

        /// Enable AI-powered relevance analysis using Claude Haiku
        #[arg(long, alias = "haiku")]
        analyze: bool,
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
    pub fn is_init(&self) -> bool {
        matches!(self, Commands::Init)
    }

    pub fn is_new(&self) -> bool {
        matches!(self, Commands::New { .. })
    }

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
    pub fn get_new_name(&self) -> Option<&str> {
        match self {
            Commands::New { name } => Some(name.as_str()),
            _ => None,
        }
    }

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
    fn test_cli_parse_init_command() {
        let cli = Cli::parse_from(["hail-mary", "init"]);
        assert!(cli.command.is_init());
    }

    #[test]
    fn test_cli_parse_new_command() {
        let cli = Cli::parse_from(["hail-mary", "new", "my-feature"]);
        assert!(cli.command.is_new());
        assert_eq!(cli.command.get_new_name(), Some("my-feature"));
    }

    #[test]
    fn test_commands_is_methods() {
        let init_cmd = Commands::Init;
        assert!(init_cmd.is_init());
        assert!(!init_cmd.is_new());

        let new_cmd = Commands::New {
            name: "test".to_string(),
        };
        assert!(!new_cmd.is_init());
        assert!(new_cmd.is_new());

        let code_cmd = Commands::Code {
            no_danger: false,
            continue_conversation: false,
        };
        assert!(!code_cmd.is_init());
        assert!(!code_cmd.is_new());
        assert!(code_cmd.is_code());
    }
}
