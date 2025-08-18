mod commands;
mod core;
mod models;
mod repositories;
mod services;
mod utils;

#[cfg(test)]
pub mod tests;

use clap::{Parser, Subcommand};
use commands::init::InitCommand;
use commands::new::NewCommand;
use utils::error::Result;

#[derive(Parser)]
#[command(name = "hail-mary")]
#[command(about = "A CLI tool for Memory MCP and Rust project specification management")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize .kiro directory and configuration
    Init(InitCommand),

    /// Create a new feature specification directory and files
    New(NewCommand),

    /// Memory MCP commands
    Memory {
        #[command(subcommand)]
        memory_command: MemoryCommands,
    },
}

#[derive(Subcommand)]
enum MemoryCommands {
    /// Start the Memory MCP server
    Serve,

    /// Generate memory documentation
    Document {
        /// Filter by memory type
        #[arg(long)]
        r#type: Option<String>,
    },

    /// Reindex and optimize the memory database
    Reindex {
        /// Show what would be done without making changes
        #[arg(long)]
        dry_run: bool,

        /// Enable verbose logging
        #[arg(long)]
        verbose: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init(init_command) => {
            init_command.execute()?;
        }
        Commands::New(new_command) => {
            new_command.execute()?;
        }
        Commands::Memory { memory_command } => match memory_command {
            MemoryCommands::Serve => {
                commands::memory::serve().await?;
            }
            MemoryCommands::Document { r#type } => {
                commands::memory::document(r#type).await?;
            }
            MemoryCommands::Reindex { dry_run, verbose } => {
                commands::memory::reindex(dry_run, verbose).await?;
            }
        },
    }

    Ok(())
}
