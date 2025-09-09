use crate::cli::args::{Cli, Shell};
use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{
    generate,
    shells::{Bash, Elvish, Fish, PowerShell, Zsh},
};
use std::io;

pub fn handle_completion(shell: &Shell) -> Result<()> {
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();

    match shell {
        Shell::Bash => generate(Bash, &mut cmd, &bin_name, &mut io::stdout()),
        Shell::Zsh => generate(Zsh, &mut cmd, &bin_name, &mut io::stdout()),
        Shell::Fish => generate(Fish, &mut cmd, &bin_name, &mut io::stdout()),
        Shell::PowerShell => generate(PowerShell, &mut cmd, &bin_name, &mut io::stdout()),
        Shell::Elvish => generate(Elvish, &mut cmd, &bin_name, &mut io::stdout()),
    }

    Ok(())
}
