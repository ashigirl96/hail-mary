use crate::models::kiro::KiroConfig;
use crate::repositories::memory::SqliteMemoryRepository;
use crate::services::memory::MemoryService;
use crate::services::memory_mcp::MemoryMcpServer;
use anyhow::{Context, Result};
use rmcp::{ServiceExt, transport::stdio};

pub async fn execute() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".to_string().into()),
        )
        .init();

    tracing::info!("Starting Memory MCP server...");

    // Load configuration
    let config = KiroConfig::load()
        .context("Failed to load configuration. Did you run 'hail-mary init'?")?;

    tracing::info!("Using database: {}", config.memory.database.path.display());

    // Create repository and service
    let repository =
        SqliteMemoryRepository::new(&config).context("Failed to initialize database")?;

    let service = MemoryService::new(repository, config.clone());
    let mcp_server = MemoryMcpServer::new(service, config);

    tracing::info!("Memory MCP server ready. Connect with MCP client via stdio.");

    // Start MCP server with stdio transport
    let service = mcp_server
        .serve(stdio())
        .await
        .context("Failed to start MCP server")?;

    // Wait for the service to complete
    service.waiting().await.context("MCP server error")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::common::TestDirectory;
    use std::fs;

    #[tokio::test]
    async fn test_serve_starts_mcp_server() {
        // This test verifies that the serve command can initialize properly
        // We can't easily test the actual server startup without mocking,
        // but we can test the initialization path

        let test_dir = TestDirectory::new();

        // Create .kiro directory structure
        let kiro_dir = test_dir.path().join(".kiro");
        let memory_dir = kiro_dir.join("memory");
        fs::create_dir_all(&memory_dir).unwrap();

        // Create config.toml
        let config_content = r#"
[memory]
types = ["tech", "project-tech", "domain"]
instructions = "Test instructions"

[memory.document]
output_dir = ".kiro/memory"
format = "markdown"

[memory.database]
path = ".kiro/memory/db.sqlite3"
"#;
        fs::write(kiro_dir.join("config.toml"), config_content).unwrap();

        // Test that we can load config and create services
        // (actual server startup would block, so we just test initialization)
        let config = KiroConfig::load();
        assert!(config.is_ok(), "Should be able to load config");

        let config = config.unwrap();
        let repository_result = SqliteMemoryRepository::new(&config);
        assert!(
            repository_result.is_ok(),
            "Should be able to create repository"
        );
    }

    #[tokio::test]
    async fn test_serve_fails_without_config() {
        let _test_dir = TestDirectory::new();
        // No .kiro directory created

        // Test that config loading fails appropriately
        let config = KiroConfig::load();
        assert!(config.is_err(), "Should fail without config");
    }
}
