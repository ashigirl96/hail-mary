use crate::memory::{
    models::{
        RmcpDeleteParams, RmcpDeleteResponse, RmcpRecallParams, RmcpRecallResponse,
        RmcpRememberParams, RmcpRememberResponse,
    },
    repository::SqliteMemoryRepository,
    service::MemoryService,
};
use anyhow::{Result, anyhow};
use rmcp::{
    ErrorData as McpError, Json,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    serve_server, tool, tool_handler, tool_router,
    transport::stdio,
};
use std::{path::Path, sync::Arc};
use tokio::sync::Mutex;
use tracing::{error, info};

/// Memory MCP サーバー (rmcp 0.5.0 based)
#[derive(Clone)]
pub struct MemoryMcpServer {
    service: Arc<Mutex<MemoryService<SqliteMemoryRepository>>>,
    tool_router: ToolRouter<Self>,
}

#[tool_handler(router = self.tool_router)]
impl rmcp::ServerHandler for MemoryMcpServer {}

#[tool_router(router = tool_router)]
impl MemoryMcpServer {
    /// 新しいサーバーを作成
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let repository = SqliteMemoryRepository::new(db_path)?;
        let service = MemoryService::new(repository);

        Ok(Self {
            service: Arc::new(Mutex::new(service)),
            tool_router: Self::tool_router(),
        })
    }

    /// サーバーを実行
    pub async fn run(self) -> Result<()> {
        info!("Starting Memory MCP server (rmcp 0.5.0)");
        serve_server(self, stdio()).await?;
        info!("Memory MCP server shutting down");
        Ok(())
    }

    /// Store a memory for future recall
    #[tool(name = "remember", description = "Store a memory for future recall")]
    pub async fn remember(
        &self,
        params: Parameters<RmcpRememberParams>,
    ) -> Result<Json<RmcpRememberResponse>, McpError> {
        let mut service = self.service.lock().await;
        let response = service.remember(params.0.into()).await.map_err(|e| {
            error!("Remember tool error: {}", e);
            McpError {
                code: rmcp::model::ErrorCode(-32603),
                message: format!("Error storing memory: {}", e).into(),
                data: None,
            }
        })?;
        Ok(Json(response.into()))
    }

    /// Search and retrieve stored memories
    #[tool(name = "recall", description = "Search and retrieve stored memories")]
    pub async fn recall(
        &self,
        params: Parameters<RmcpRecallParams>,
    ) -> Result<Json<RmcpRecallResponse>, McpError> {
        let mut service = self.service.lock().await;
        let response = service.recall(params.0.into()).await.map_err(|e| {
            error!("Recall tool error: {}", e);
            McpError {
                code: rmcp::model::ErrorCode(-32603),
                message: format!("Error recalling memory: {}", e).into(),
                data: None,
            }
        })?;
        Ok(Json(response.into()))
    }

    /// Delete a memory (soft delete)
    #[tool(name = "delete_memory", description = "Delete a stored memory")]
    pub async fn delete_memory(
        &self,
        params: Parameters<RmcpDeleteParams>,
    ) -> Result<Json<RmcpDeleteResponse>, McpError> {
        let mut service = self.service.lock().await;
        service
            .delete_memory(&params.0.memory_id)
            .await
            .map_err(|e| {
                error!("Delete memory tool error: {}", e);
                McpError {
                    code: rmcp::model::ErrorCode(-32603),
                    message: format!("Error deleting memory: {}", e).into(),
                    data: None,
                }
            })?;

        Ok(Json(RmcpDeleteResponse {
            deleted: true,
            memory_id: params.0.memory_id,
        }))
    }

    /// Get server information
    #[tool(name = "get_info", description = "Get server information")]
    pub async fn get_info(&self) -> String {
        format!(
            "Memory MCP Server v{} - Store and retrieve memories",
            env!("CARGO_PKG_VERSION")
        )
    }
}

/// データベースのデフォルトパスを取得
pub fn get_default_db_path() -> Result<std::path::PathBuf> {
    let data_dir =
        dirs::data_local_dir().ok_or_else(|| anyhow!("Failed to get local data directory"))?;

    let hail_mary_dir = data_dir.join("hail-mary");
    std::fs::create_dir_all(&hail_mary_dir)?;

    Ok(hail_mary_dir.join("memory.db"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_server_creation() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let server = MemoryMcpServer::new(&db_path);
        assert!(server.is_ok());
    }

    #[test]
    fn test_get_default_db_path() {
        let path = get_default_db_path().unwrap();
        assert!(path.to_str().unwrap().contains("hail-mary"));
        assert!(path.to_str().unwrap().contains("memory.db"));
    }

    #[tokio::test]
    async fn test_tool_router() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let server = MemoryMcpServer::new(&db_path).unwrap();

        // Test that tools are registered
        let tools = server.tool_router.list_all();
        assert_eq!(tools.len(), 4);

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
        assert!(tool_names.contains(&"remember"));
        assert!(tool_names.contains(&"recall"));
        assert!(tool_names.contains(&"delete_memory"));
        assert!(tool_names.contains(&"get_info"));
    }
}
