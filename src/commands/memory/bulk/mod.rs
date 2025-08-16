// Bulk operations module - placeholder for now
// Will be populated with actual bulk commands in subsequent phases

pub mod delete;
pub mod tag;
pub mod update;

use crate::utils::error::Result;
use clap::Subcommand;

/// Bulk operations commands for managing multiple memories
#[derive(clap::Args)]
pub struct BulkCommand {
    #[command(subcommand)]
    pub command: BulkSubcommands,
}

#[derive(Subcommand)]
pub enum BulkSubcommands {
    /// Bulk delete memories with filtering
    Delete(delete::BulkDeleteCommand),
    /// Bulk update memory properties
    Update(update::BulkUpdateCommand),
    /// Bulk tag operations
    Tag(tag::BulkTagCommand),
}

impl BulkCommand {
    /// Execute the bulk command
    pub fn execute(self) -> Result<()> {
        match self.command {
            BulkSubcommands::Delete(cmd) => cmd.execute(),
            BulkSubcommands::Update(cmd) => cmd.execute(),
            BulkSubcommands::Tag(cmd) => cmd.execute(),
        }
    }
}
