use anyhow::Result;
use clap::Parser;
use hail_mary::cli::args::{Cli, Commands, SteeringCommands};
use hail_mary::cli::commands::{CodeCommand, CompleteCommand, SteeringBackupCommand, completion};
use hail_mary::cli::formatters::format_error;
use std::process;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{}", format_error(&format!("{:#}", e)));
        process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Completion { shell } => {
            completion::handle_completion(&shell)?;
        }
        Commands::Complete => {
            let command = CompleteCommand::new();
            command.execute()?;
        }
        Commands::Code {
            no_danger,
            continue_conversation,
        } => {
            let command = CodeCommand::new(no_danger, continue_conversation);
            command.execute()?;
        }
        Commands::Steering { command } => match command {
            SteeringCommands::Backup => {
                let backup_command = SteeringBackupCommand::new();
                backup_command.execute()?;
            }
        },
    }

    Ok(())
}
