use anyhow::Result;
use clap::Parser;
use hail_mary::cli::args::{Cli, Commands};
use hail_mary::cli::commands::{CodeCommand, CompleteCommand, completion};
use hail_mary::cli::formatters::format_error;
use std::process;

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", format_error(&format!("{:#}", e)));
        process::exit(1);
    }
}

fn run() -> Result<()> {
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
    }

    Ok(())
}
