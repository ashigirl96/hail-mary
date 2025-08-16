pub mod analytics;
pub mod bulk;
pub mod cluster;
pub mod common;
pub mod dedup;
pub mod delete;
pub mod document;
pub mod embed_analytics;
pub mod export;
pub mod import;
pub mod index;
pub mod list;
pub mod reindex;
pub mod related;
pub mod search;
pub mod serve;

use crate::utils::error::Result;
use clap::Subcommand;

/// Memory management commands
#[derive(clap::Args)]
pub struct MemoryCommand {
    #[command(subcommand)]
    pub command: MemorySubcommands,
}

#[derive(Subcommand)]
pub enum MemorySubcommands {
    /// Start the Memory MCP server
    Serve(serve::ServeCommand),
    /// Generate documentation from stored memories
    Document(document::DocumentCommand),
    /// Reindex and optimize the memory database
    Reindex(reindex::ReindexCommand),
    /// Delete a memory from the database
    Delete(delete::DeleteCommand),
    /// List memories from the database
    List(list::ListCommand),
    /// Advanced search for memories with filtering and sorting
    Search(search::SearchCommand),
    /// Find memories related to a specific memory using semantic similarity
    Related(related::RelatedCommand),
    /// Find and merge duplicate memories using semantic similarity
    Dedup(dedup::DedupCommand),
    /// Cluster memories into groups based on semantic similarity
    Cluster(cluster::ClusterCommand),
    /// Analyze embeddings and vector space characteristics
    EmbedAnalytics(embed_analytics::EmbedAnalyticsCommand),
    /// Manage embedding indices for performance optimization
    Index(index::IndexCommand),
    /// Export memories to JSON or CSV format
    Export(export::ExportCommand),
    /// Import memories from JSON or CSV format
    Import(import::ImportCommand),
    /// Bulk operations for managing multiple memories
    Bulk(bulk::BulkCommand),
    /// Analytics and statistics for memory database insights
    Analytics(analytics::AnalyticsCommand),
}

impl MemoryCommand {
    /// Execute the memory command
    pub fn execute(self) -> Result<()> {
        match self.command {
            MemorySubcommands::Serve(cmd) => cmd.execute(),
            MemorySubcommands::Document(cmd) => cmd.execute(),
            MemorySubcommands::Reindex(cmd) => cmd.execute(),
            MemorySubcommands::Delete(cmd) => cmd.execute(),
            MemorySubcommands::List(cmd) => cmd.execute(),
            MemorySubcommands::Search(cmd) => cmd.execute(),
            MemorySubcommands::Related(cmd) => cmd.execute(),
            MemorySubcommands::Dedup(cmd) => cmd.execute(),
            MemorySubcommands::Cluster(cmd) => cmd.execute(),
            MemorySubcommands::EmbedAnalytics(cmd) => cmd.execute(),
            MemorySubcommands::Index(cmd) => cmd.execute(),
            MemorySubcommands::Export(cmd) => cmd.execute(),
            MemorySubcommands::Import(cmd) => cmd.execute(),
            MemorySubcommands::Bulk(cmd) => cmd.execute(),
            MemorySubcommands::Analytics(cmd) => cmd.execute(),
        }
    }
}
