mod commands;
mod core;
mod utils;

use clap::{Parser, Subcommand};
use commands::new::NewCommand;
use utils::error::Result;

#[derive(Parser)]
#[command(name = "hail-mary")]
#[command(about = "A CLI tool for Rust project specification management and development support")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new feature specification directory and files
    New(NewCommand),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New(new_command) => {
            new_command.execute()?;
        }
    }

    Ok(())
}
