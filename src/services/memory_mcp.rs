use crate::models::error::MemoryError;
use crate::models::kiro::KiroConfig;
use crate::models::memory::MemoryType;
use crate::repositories::memory::MemoryRepository;
use crate::services::memory::{MemoryInput, MemoryService};
#[cfg(test)]
use rmcp::ServerHandler;
use rmcp::{
    ErrorData as McpError, Json,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::{ErrorCode, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Input structure for MCP remember tool
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct MemoryInputMcp {
    #[schemars(description = "Type of memory (tech, project-tech, domain, etc.)")]
    pub r#type: String,
    #[schemars(description = "Title or summary of the memory")]
    pub title: String,
    #[schemars(description = "Detailed content of the memory")]
    pub content: String,
    #[schemars(description = "Tags for categorization and search")]
    pub tags: Vec<String>,
    #[schemars(description = "Confidence level between 0.0 and 1.0")]
    pub confidence: Option<f32>,
}

/// Parameters for the remember tool
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RememberParams {
    #[schemars(description = "Array of memories to store")]
    pub memories: Vec<MemoryInputMcp>,
}

/// Response from the remember tool
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RememberResponse {
    #[schemars(description = "Array of IDs for the created memories")]
    pub memory_ids: Vec<String>,
    #[schemars(description = "Number of memories successfully created")]
    pub created_count: usize,
}

/// Parameters for the recall tool
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RecallParams {
    #[schemars(description = "Search query for memories")]
    pub query: String,
    #[schemars(description = "Filter by memory type (optional)")]
    pub r#type: Option<String>,
    #[schemars(description = "Filter by tags (optional)")]
    pub tags: Option<Vec<String>>,
    #[schemars(description = "Maximum number of results (default: 10)")]
    pub limit: Option<u32>,
}

/// Response from the recall tool
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RecallResponse {
    #[schemars(description = "Memories formatted as Markdown")]
    pub content: String,
    #[schemars(description = "Total number of memories found")]
    pub total_count: usize,
}

/// Memory MCP Server implementation
#[derive(Clone)]
pub struct MemoryMcpServer<R: MemoryRepository> {
    service: Arc<Mutex<MemoryService<R>>>,
    config: KiroConfig,
    tool_router: ToolRouter<Self>,
}

#[tool_handler(router = self.tool_router)]
impl<R: MemoryRepository + 'static> rmcp::ServerHandler for MemoryMcpServer<R> {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(format!(
                "Memory MCP Server v{}\n\n{}",
                env!("CARGO_PKG_VERSION"),
                self.config.memory.instructions
            )),
        }
    }
}

#[tool_router(router = tool_router)]
impl<R: MemoryRepository + 'static> MemoryMcpServer<R> {
    /// Create a new MCP server with the given service and configuration
    pub fn new(service: MemoryService<R>, config: KiroConfig) -> Self {
        Self {
            service: Arc::new(Mutex::new(service)),
            config,
            tool_router: Self::tool_router(),
        }
    }

    /// Store memories for future recall
    #[tool(name = "remember", description = "Store memories for future recall")]
    pub async fn remember(
        &self,
        params: Parameters<RememberParams>,
    ) -> std::result::Result<Json<RememberResponse>, McpError> {
        let mut service = self.service.lock().await;

        // Convert MCP inputs to service layer inputs with validation
        let memory_inputs: Vec<MemoryInput> = params
            .0
            .memories
            .into_iter()
            .map(|input| self.convert_mcp_input_to_service(input))
            .collect::<Result<Vec<_>, MemoryError>>()
            .map_err(|e| {
                McpError::new(
                    ErrorCode::INVALID_PARAMS,
                    format!("Invalid parameters: {}", e),
                    None,
                )
            })?;

        // Store memories using the service layer
        let created_memories = service.remember_batch(memory_inputs).await.map_err(|e| {
            McpError::new(
                ErrorCode::INTERNAL_ERROR,
                format!("Internal error: {}", e),
                None,
            )
        })?;

        let memory_ids: Vec<String> = created_memories.iter().map(|m| m.id.clone()).collect();

        Ok(Json(RememberResponse {
            memory_ids: memory_ids.clone(),
            created_count: memory_ids.len(),
        }))
    }

    /// Search and retrieve stored memories
    #[tool(name = "recall", description = "Search and retrieve stored memories")]
    pub async fn recall(
        &self,
        params: Parameters<RecallParams>,
    ) -> std::result::Result<Json<RecallResponse>, McpError> {
        let mut service = self.service.lock().await;

        // Parse type filter with validation
        let type_filter = if let Some(type_str) = params.0.r#type {
            Some(type_str.parse::<MemoryType>().map_err(|e| {
                McpError::new(
                    ErrorCode::INVALID_PARAMS,
                    format!("Invalid memory type: {}", e),
                    None,
                )
            })?)
        } else {
            None
        };

        // Validate type against config if provided
        if let Some(ref memory_type) = type_filter {
            let type_str = memory_type.to_string();
            if !self.config.validate_memory_type(&type_str) {
                return Err(McpError::new(
                    ErrorCode::INVALID_PARAMS,
                    format!(
                        "Invalid memory type '{}'. Allowed types: {:?}",
                        type_str, self.config.memory.types
                    ),
                    None,
                ));
            }
        }

        let tag_filter = params.0.tags.unwrap_or_default();
        let limit = params.0.limit.unwrap_or(10) as usize;

        // Recall memories using the service layer
        let content = service
            .recall(&params.0.query, limit, type_filter, tag_filter)
            .await
            .map_err(|e| {
                McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    format!("Internal error: {}", e),
                    None,
                )
            })?;

        // Count total memories in the markdown content
        let total_count = content.matches("## ").count();

        Ok(Json(RecallResponse {
            content,
            total_count,
        }))
    }

    /// Convert MCP input to service layer input with validation
    fn convert_mcp_input_to_service(
        &self,
        input: MemoryInputMcp,
    ) -> std::result::Result<MemoryInput, MemoryError> {
        // Parse and validate memory type
        let memory_type = input.r#type.parse::<MemoryType>().map_err(|e| {
            MemoryError::InvalidInput(format!("Invalid memory type '{}': {}", input.r#type, e))
        })?;

        // Validate against KiroConfig
        if !self.config.validate_memory_type(&input.r#type) {
            return Err(MemoryError::InvalidInput(format!(
                "Invalid memory type '{}'. Allowed types: {:?}",
                input.r#type, self.config.memory.types
            )));
        }

        Ok(MemoryInput {
            memory_type,
            title: input.title,
            content: input.content,
            tags: input.tags,
            confidence: input.confidence,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::memory::MemoryType;
    use crate::repositories::memory::InMemoryRepository;
    use pretty_assertions::assert_eq;

    fn setup_test_server() -> MemoryMcpServer<InMemoryRepository> {
        let repository = InMemoryRepository::new();
        let config = KiroConfig::default();
        let service = MemoryService::new(repository, config.clone());
        MemoryMcpServer::new(service, config)
    }

    fn create_test_memory_input_mcp() -> MemoryInputMcp {
        MemoryInputMcp {
            r#type: "tech".to_string(),
            title: "Test Memory".to_string(),
            content: "Test content for memory".to_string(),
            tags: vec!["test".to_string(), "rust".to_string()],
            confidence: Some(0.9),
        }
    }

    #[tokio::test]
    async fn test_remember_params_validation() {
        let server = setup_test_server();

        // Test valid parameters
        let valid_input = create_test_memory_input_mcp();
        let valid_params = RememberParams {
            memories: vec![valid_input],
        };

        let result = server.remember(Parameters(valid_params)).await;

        assert!(result.is_ok(), "Valid parameters should succeed");

        let response = result.unwrap().0;
        assert_eq!(response.memory_ids.len(), 1);
        assert_eq!(response.created_count, 1);
        assert!(!response.memory_ids[0].is_empty());
    }

    #[tokio::test]
    async fn test_remember_params_validation_invalid_type() {
        let server = setup_test_server();

        // Test invalid memory type
        let invalid_input = MemoryInputMcp {
            r#type: "invalid-type".to_string(),
            title: "Test".to_string(),
            content: "Content".to_string(),
            tags: vec![],
            confidence: None,
        };

        let invalid_params = RememberParams {
            memories: vec![invalid_input],
        };

        let result = server.remember(Parameters(invalid_params)).await;

        assert!(result.is_err(), "Invalid memory type should fail");

        if let Err(error) = result {
            assert_eq!(
                error.code,
                ErrorCode::INVALID_PARAMS,
                "Should return invalid params error code"
            );
            assert!(
                error.message.contains("Invalid parameters"),
                "Error message should mention invalid parameters: {}",
                error.message
            );
        }
    }

    #[tokio::test]
    async fn test_remember_params_validation_required_fields() {
        let server = setup_test_server();

        // Test empty title
        let empty_title = MemoryInputMcp {
            r#type: "tech".to_string(),
            title: "".to_string(),
            content: "Valid content".to_string(),
            tags: vec![],
            confidence: None,
        };

        let params = RememberParams {
            memories: vec![empty_title],
        };

        let result = server.remember(Parameters(params)).await;
        assert!(result.is_err(), "Empty title should fail");

        if let Err(error) = result {
            assert_eq!(
                error.code,
                ErrorCode::INTERNAL_ERROR,
                "Should return internal error code"
            );
        }
    }

    #[tokio::test]
    async fn test_recall_params_validation() {
        let server = setup_test_server();

        // First, add some test data
        let memory_input = create_test_memory_input_mcp();
        let remember_params = RememberParams {
            memories: vec![memory_input],
        };
        server.remember(Parameters(remember_params)).await.unwrap();

        // Test valid recall with optional fields
        let valid_params = RecallParams {
            query: "test".to_string(),
            r#type: Some("tech".to_string()),
            tags: Some(vec!["rust".to_string()]),
            limit: Some(5),
        };

        let result = server.recall(Parameters(valid_params)).await;

        assert!(result.is_ok(), "Valid recall params should succeed");

        let response = result.unwrap().0;
        assert!(!response.content.is_empty());
        assert!(response.total_count > 0);
    }

    #[tokio::test]
    async fn test_recall_params_validation_invalid_type() {
        let server = setup_test_server();

        // Test invalid memory type in recall
        let invalid_params = RecallParams {
            query: "test".to_string(),
            r#type: Some("invalid-type".to_string()),
            tags: None,
            limit: None,
        };

        let result = server.recall(Parameters(invalid_params)).await;

        assert!(result.is_err(), "Invalid memory type should fail");

        if let Err(error) = result {
            assert_eq!(
                error.code,
                ErrorCode::INVALID_PARAMS,
                "Should return invalid params error code"
            );
            assert!(
                error.message.contains("Invalid memory type"),
                "Error message should mention invalid type: {}",
                error.message
            );
        }
    }

    #[tokio::test]
    async fn test_recall_params_validation_defaults() {
        let server = setup_test_server();

        // Test with minimal parameters (defaults should be applied)
        let minimal_params = RecallParams {
            query: "test".to_string(),
            r#type: None,
            tags: None,
            limit: None,
        };

        let result = server.recall(Parameters(minimal_params)).await;

        assert!(
            result.is_ok(),
            "Minimal params should succeed with defaults"
        );

        let response = result.unwrap().0;
        // Should use default limit of 10
        assert!(response.total_count <= 10);
    }

    #[tokio::test]
    async fn test_mcp_error_codes() {
        let server = setup_test_server();

        // Test -32602: Invalid params (invalid memory type)
        let invalid_params = RememberParams {
            memories: vec![MemoryInputMcp {
                r#type: "nonexistent".to_string(),
                title: "Test".to_string(),
                content: "Content".to_string(),
                tags: vec![],
                confidence: None,
            }],
        };

        let result = server.remember(Parameters(invalid_params)).await;

        assert!(result.is_err());
        if let Err(error) = result {
            assert_eq!(
                error.code,
                ErrorCode::INVALID_PARAMS,
                "Should return INVALID_PARAMS for invalid params"
            );
        }

        // Test -32603: Internal error (empty title triggers service validation)
        let service_error_params = RememberParams {
            memories: vec![MemoryInputMcp {
                r#type: "tech".to_string(),
                title: "".to_string(), // This will cause service layer validation error
                content: "Content".to_string(),
                tags: vec![],
                confidence: None,
            }],
        };

        let result = server.remember(Parameters(service_error_params)).await;

        assert!(result.is_err());
        if let Err(error) = result {
            assert_eq!(
                error.code,
                ErrorCode::INTERNAL_ERROR,
                "Should return INTERNAL_ERROR for internal error"
            );
        }
    }

    #[tokio::test]
    async fn test_mcp_server_initialization() {
        let server = setup_test_server();

        // Test that server info includes instructions from config
        let info = server.get_info();

        assert!(
            info.instructions.is_some(),
            "Server should have instructions"
        );

        let instructions = info.instructions.unwrap();
        assert!(
            instructions.contains("Memory MCP Server"),
            "Instructions should contain Memory MCP Server"
        );
        assert!(
            instructions.contains(&server.config.memory.instructions),
            "Instructions should contain config instructions: {}",
            instructions
        );

        // Test that capabilities include tools
        assert!(
            info.capabilities.tools.is_some(),
            "Server should support tools"
        );
    }

    #[tokio::test]
    async fn test_convert_mcp_input_to_service() {
        let server = setup_test_server();

        // Test valid conversion
        let mcp_input = create_test_memory_input_mcp();
        let service_input = server
            .convert_mcp_input_to_service(mcp_input.clone())
            .unwrap();

        assert_eq!(service_input.memory_type, MemoryType::Tech);
        assert_eq!(service_input.title, mcp_input.title);
        assert_eq!(service_input.content, mcp_input.content);
        assert_eq!(service_input.tags, mcp_input.tags);
        assert_eq!(service_input.confidence, mcp_input.confidence);

        // Test invalid type conversion
        let invalid_input = MemoryInputMcp {
            r#type: "invalid".to_string(),
            title: "Test".to_string(),
            content: "Content".to_string(),
            tags: vec![],
            confidence: None,
        };

        let result = server.convert_mcp_input_to_service(invalid_input);
        assert!(result.is_err(), "Invalid type should fail conversion");
    }
}
