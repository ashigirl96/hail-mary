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
    Init {
        /// Force initialization even if project already exists
        #[arg(short, long)]
        force: bool,
    },

    /// Create a new feature specification
    New {
        /// Feature name in kebab-case
        name: String,
    },

    /// Memory operations
    Memory {
        #[command(subcommand)]
        command: MemoryCommands,
    },

    /// Generate shell completion scripts
    Completion {
        /// Shell type to generate completions for
        shell: Shell,
    },
}

#[derive(Subcommand, Debug)]
pub enum MemoryCommands {
    /// Start MCP server
    Serve,

    /// Generate documentation
    Document {
        /// Generate documentation for specific memory type
        #[arg(short = 't', long = "type")]
        memory_type: Option<String>,
    },

    /// Reindex database
    Reindex {
        /// Run in dry-run mode (no changes)
        #[arg(long)]
        dry_run: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
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
        matches!(self, Commands::Init { .. })
    }

    pub fn is_new(&self) -> bool {
        matches!(self, Commands::New { .. })
    }

    pub fn is_memory(&self) -> bool {
        matches!(self, Commands::Memory { .. })
    }

    pub fn is_completion(&self) -> bool {
        matches!(self, Commands::Completion { .. })
    }
}

impl Commands {
    pub fn get_init_force(&self) -> Option<bool> {
        match self {
            Commands::Init { force } => Some(*force),
            _ => None,
        }
    }

    pub fn get_new_name(&self) -> Option<&str> {
        match self {
            Commands::New { name } => Some(name.as_str()),
            _ => None,
        }
    }

    pub fn get_memory_command(&self) -> Option<&MemoryCommands> {
        match self {
            Commands::Memory { command } => Some(command),
            _ => None,
        }
    }

    pub fn get_completion_shell(&self) -> Option<&Shell> {
        match self {
            Commands::Completion { shell } => Some(shell),
            _ => None,
        }
    }
}

impl MemoryCommands {
    pub fn is_serve(&self) -> bool {
        matches!(self, MemoryCommands::Serve)
    }

    pub fn is_document(&self) -> bool {
        matches!(self, MemoryCommands::Document { .. })
    }

    pub fn is_reindex(&self) -> bool {
        matches!(self, MemoryCommands::Reindex { .. })
    }

    pub fn get_document_type(&self) -> Option<&str> {
        match self {
            MemoryCommands::Document { memory_type } => memory_type.as_deref(),
            _ => None,
        }
    }

    pub fn get_reindex_options(&self) -> Option<(bool, bool)> {
        match self {
            MemoryCommands::Reindex { dry_run, verbose } => Some((*dry_run, *verbose)),
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
        assert_eq!(cli.command.get_init_force(), Some(false));
    }

    #[test]
    fn test_cli_parse_init_with_force() {
        let cli = Cli::parse_from(["hail-mary", "init", "--force"]);
        assert!(cli.command.is_init());
        assert_eq!(cli.command.get_init_force(), Some(true));
    }

    #[test]
    fn test_cli_parse_new_command() {
        let cli = Cli::parse_from(["hail-mary", "new", "my-feature"]);
        assert!(cli.command.is_new());
        assert_eq!(cli.command.get_new_name(), Some("my-feature"));
    }

    #[test]
    fn test_cli_parse_memory_serve() {
        let cli = Cli::parse_from(["hail-mary", "memory", "serve"]);
        assert!(cli.command.is_memory());

        let memory_cmd = cli.command.get_memory_command().unwrap();
        assert!(memory_cmd.is_serve());
    }

    #[test]
    fn test_cli_parse_memory_document() {
        let cli = Cli::parse_from(["hail-mary", "memory", "document"]);
        assert!(cli.command.is_memory());

        let memory_cmd = cli.command.get_memory_command().unwrap();
        assert!(memory_cmd.is_document());
        assert_eq!(memory_cmd.get_document_type(), None);
    }

    #[test]
    fn test_cli_parse_memory_document_with_type() {
        let cli = Cli::parse_from(["hail-mary", "memory", "document", "--type", "tech"]);
        assert!(cli.command.is_memory());

        let memory_cmd = cli.command.get_memory_command().unwrap();
        assert!(memory_cmd.is_document());
        assert_eq!(memory_cmd.get_document_type(), Some("tech"));
    }

    #[test]
    fn test_cli_parse_memory_reindex() {
        let cli = Cli::parse_from(["hail-mary", "memory", "reindex"]);
        assert!(cli.command.is_memory());

        let memory_cmd = cli.command.get_memory_command().unwrap();
        assert!(memory_cmd.is_reindex());
        assert_eq!(memory_cmd.get_reindex_options(), Some((false, false)));
    }

    #[test]
    fn test_cli_parse_memory_reindex_with_options() {
        let cli = Cli::parse_from(["hail-mary", "memory", "reindex", "--dry-run", "--verbose"]);
        assert!(cli.command.is_memory());

        let memory_cmd = cli.command.get_memory_command().unwrap();
        assert!(memory_cmd.is_reindex());
        assert_eq!(memory_cmd.get_reindex_options(), Some((true, true)));
    }

    #[test]
    fn test_commands_is_methods() {
        let init_cmd = Commands::Init { force: false };
        assert!(init_cmd.is_init());
        assert!(!init_cmd.is_new());
        assert!(!init_cmd.is_memory());

        let new_cmd = Commands::New {
            name: "test".to_string(),
        };
        assert!(!new_cmd.is_init());
        assert!(new_cmd.is_new());
        assert!(!new_cmd.is_memory());

        let memory_cmd = Commands::Memory {
            command: MemoryCommands::Serve,
        };
        assert!(!memory_cmd.is_init());
        assert!(!memory_cmd.is_new());
        assert!(memory_cmd.is_memory());
    }

    #[test]
    fn test_memory_commands_is_methods() {
        let serve_cmd = MemoryCommands::Serve;
        assert!(serve_cmd.is_serve());
        assert!(!serve_cmd.is_document());
        assert!(!serve_cmd.is_reindex());

        let doc_cmd = MemoryCommands::Document { memory_type: None };
        assert!(!doc_cmd.is_serve());
        assert!(doc_cmd.is_document());
        assert!(!doc_cmd.is_reindex());

        let reindex_cmd = MemoryCommands::Reindex {
            dry_run: false,
            verbose: false,
        };
        assert!(!reindex_cmd.is_serve());
        assert!(!reindex_cmd.is_document());
        assert!(reindex_cmd.is_reindex());
    }
}
