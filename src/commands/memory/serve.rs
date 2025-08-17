use crate::mcp::server::{MemoryMcpServer, get_default_db_path};
use crate::utils::error::Result;
use clap::Args;
use std::path::PathBuf;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Start the Memory MCP server
#[derive(Args)]
pub struct ServeCommand {
    /// Path to the database file (defaults to .kiro/memory/memory.db)
    #[arg(long, value_name = "PATH")]
    pub db_path: Option<PathBuf>,

    /// Run the server in daemon mode (background)
    #[arg(long)]
    pub daemon: bool,

    /// Enable verbose logging
    #[arg(long, short)]
    pub verbose: bool,
}

impl ServeCommand {
    /// Execute the serve command
    pub fn execute(self) -> Result<()> {
        // Initialize tracing based on verbosity
        // IMPORTANT: MCP uses stdio, so all logs must go to stderr to avoid JSON parsing errors
        if self.verbose {
            tracing_subscriber::registry()
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_writer(std::io::stderr)
                        .with_target(true),
                )
                .init();
        } else {
            tracing_subscriber::registry()
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_writer(std::io::stderr)
                        .with_target(false)
                        .with_level(false),
                )
                .init();
        }

        // Determine database path
        let db_path = self
            .db_path
            .unwrap_or_else(|| get_default_db_path().expect("Failed to get default database path"));

        info!("Using database at: {:?}", db_path);

        // Create runtime
        let runtime = tokio::runtime::Runtime::new()?;

        if self.daemon {
            // TODO: Implement proper daemon mode
            // For now, just run in foreground
            // Use tracing instead of eprintln to avoid interfering with MCP stdio
            tracing::warn!("--daemon mode is not yet implemented, running in foreground");
        }

        // Run the server
        let result = runtime.block_on(async {
            let server = MemoryMcpServer::new(&db_path)?;

            info!("Starting Memory MCP server");
            info!("Database path: {:?}", db_path);
            info!("Press Ctrl+C to stop the server");

            // Set up signal handler for graceful shutdown
            let shutdown = async {
                tokio::signal::ctrl_c().await.ok();
                info!("Received shutdown signal");
            };

            // Run server with shutdown handler
            tokio::select! {
                result = server.run() => {
                    match result {
                        Ok(()) => {
                            info!("Server completed normally");
                        }
                        Err(e) => {
                            error!("Server error: {}", e);
                            return Err(e);
                        }
                    }
                }
                _ = shutdown => {
                    info!("Shutting down gracefully");
                }
            }

            Ok(())
        });

        // Convert anyhow::Result to our Result
        result.map_err(|e| e.into())
    }
}
