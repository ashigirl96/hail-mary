use crate::application::errors::ApplicationError;
use crate::application::repositories::MemoryRepository;
use crate::application::use_cases::remember_memory::RememberRequest;
use crate::application::use_cases::{recall_memory, remember_memory};
use crate::domain::entities::project::ProjectConfig;
use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    schemars::{self, JsonSchema},
    tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

// MCP request/response types
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MemoryInput {
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

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RememberParams {
    #[schemars(description = "Array of memories to store")]
    pub memories: Vec<MemoryInput>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RecallParams {
    #[schemars(description = "Search query for memories")]
    pub query: String,

    #[schemars(description = "Maximum number of results")]
    pub limit: Option<usize>,

    #[schemars(description = "Filter by memory type")]
    pub r#type: Option<String>,

    #[schemars(description = "Filter by tags")]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RememberResponse {
    pub memory_ids: Vec<String>,
    pub created_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecallResponse {
    pub content: String,
    pub total_count: usize,
}

// Service wrapper for dependency injection
pub struct MemoryService {
    pub repository: Box<dyn MemoryRepository>,
    pub config: ProjectConfig,
}

impl MemoryService {
    pub fn new(repository: Box<dyn MemoryRepository>, config: ProjectConfig) -> Self {
        Self { repository, config }
    }

    pub fn remember_batch(
        &mut self,
        memories: Vec<MemoryInput>,
    ) -> Result<Vec<crate::domain::entities::memory::Memory>, ApplicationError> {
        let requests: Vec<RememberRequest> = memories
            .into_iter()
            .map(|m| RememberRequest {
                memory_type: m.r#type,
                title: m.title,
                content: m.content,
                tags: m.tags,
                confidence: m.confidence,
            })
            .collect();

        remember_memory(&mut *self.repository, &self.config, requests)
    }

    pub fn recall(
        &mut self,
        query: &str,
        limit: usize,
        type_filter: Option<String>,
        tag_filter: Vec<String>,
    ) -> Result<String, ApplicationError> {
        recall_memory(&mut *self.repository, query, limit, type_filter, tag_filter)
    }
}

// MCP Server implementation
#[derive(Clone)]
pub struct MemoryMcpServer {
    service: Arc<Mutex<MemoryService>>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl MemoryMcpServer {
    pub fn new(service: MemoryService) -> Self {
        Self {
            service: Arc::new(Mutex::new(service)),
            tool_router: Self::tool_router(),
        }
    }

    #[tool(name = "remember", description = "Store memories for future recall")]
    pub async fn remember(
        &self,
        params: Parameters<RememberParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut service = self.service.lock().unwrap();
        let memories = service.remember_batch(params.0.memories).map_err(|e| {
            McpError::internal_error(format!("Failed to store memories: {}", e), None)
        })?;

        let response = RememberResponse {
            memory_ids: memories.iter().map(|m| m.id.to_string()).collect(),
            created_count: memories.len(),
        };

        let content = serde_json::to_string(&response).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize response: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(content)]))
    }

    #[tool(name = "recall", description = "Search and retrieve stored memories")]
    pub async fn recall(
        &self,
        params: Parameters<RecallParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut service = self.service.lock().unwrap();
        let markdown = service
            .recall(
                &params.0.query,
                params.0.limit.unwrap_or(10),
                params.0.r#type,
                params.0.tags.unwrap_or_default(),
            )
            .map_err(|e| {
                McpError::internal_error(format!("Failed to recall memories: {}", e), None)
            })?;

        Ok(CallToolResult::success(vec![Content::text(markdown)]))
    }
}

#[tool_handler]
impl ServerHandler for MemoryMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("Memory MCP Server v3\n\nAvailable memory types:\n- tech: General technical knowledge (languages, frameworks, algorithms)\n- project-tech: This project's specific technical implementation\n- domain: Business domain knowledge and requirements\n- workflow: Development workflows and processes\n- decision: Architecture decisions and their rationale".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_helpers::MockMemoryRepository;
    use crate::domain::entities::memory::Memory;
    use crate::domain::value_objects::confidence::Confidence;

    fn create_test_config() -> ProjectConfig {
        ProjectConfig::default_for_new_project()
    }

    fn create_test_service() -> MemoryService {
        let repo = Box::new(MockMemoryRepository::new());
        let config = create_test_config();
        MemoryService::new(repo, config)
    }

    fn create_test_service_with_memories(memories: Vec<Memory>) -> MemoryService {
        let repo = Box::new(MockMemoryRepository::new().with_memories(memories));
        let config = create_test_config();
        MemoryService::new(repo, config)
    }

    fn create_test_memory_input() -> MemoryInput {
        MemoryInput {
            r#type: "tech".to_string(),
            title: "Test Memory".to_string(),
            content: "Test content".to_string(),
            tags: vec!["test".to_string()],
            confidence: Some(0.8),
        }
    }

    #[test]
    fn test_memory_service_new() {
        // Red: テスト先行 - MemoryService初期化
        let repo = Box::new(MockMemoryRepository::new());
        let config = create_test_config();
        let mut service = MemoryService::new(repo, config);

        // サービスが正常に初期化されることを確認
        assert!(service.repository.find_all().is_ok());
    }

    #[test]
    fn test_memory_service_remember_batch_single() {
        // Red: テスト先行 - 単一メモリの保存
        let mut service = create_test_service();
        let memory_input = create_test_memory_input();

        let result = service.remember_batch(vec![memory_input]);
        assert!(result.is_ok());

        let memories = result.unwrap();
        assert_eq!(memories.len(), 1);
        assert_eq!(memories[0].memory_type, "tech");
        assert_eq!(memories[0].title, "Test Memory");
    }

    #[test]
    fn test_memory_service_remember_batch_multiple() {
        // Red: テスト先行 - 複数メモリのバッチ保存
        let mut service = create_test_service();
        let memories_input = vec![
            MemoryInput {
                r#type: "tech".to_string(),
                title: "Memory 1".to_string(),
                content: "Content 1".to_string(),
                tags: vec!["tag1".to_string()],
                confidence: Some(0.9),
            },
            MemoryInput {
                r#type: "domain".to_string(),
                title: "Memory 2".to_string(),
                content: "Content 2".to_string(),
                tags: vec!["tag2".to_string()],
                confidence: Some(0.7),
            },
        ];

        let result = service.remember_batch(memories_input);
        assert!(result.is_ok());

        let memories = result.unwrap();
        assert_eq!(memories.len(), 2);
    }

    #[test]
    fn test_memory_service_remember_batch_invalid_type() {
        // Red: テスト先行 - 無効なメモリタイプの検証
        let mut service = create_test_service();
        let memory_input = MemoryInput {
            r#type: "invalid-type".to_string(),
            title: "Test Memory".to_string(),
            content: "Test content".to_string(),
            tags: vec![],
            confidence: None,
        };

        let result = service.remember_batch(vec![memory_input]);
        assert!(result.is_err());
        match result.unwrap_err() {
            ApplicationError::InvalidMemoryType(memory_type) => {
                assert_eq!(memory_type, "invalid-type");
            }
            _ => panic!("Expected InvalidMemoryType error"),
        }
    }

    #[test]
    fn test_memory_service_recall_basic() {
        // Red: テスト先行 - 基本的な検索機能
        let test_memory = Memory::new(
            "tech".to_string(),
            "Rust Memory".to_string(),
            "Safe systems programming".to_string(),
        )
        .with_tags(vec!["rust".to_string()])
        .with_confidence(Confidence::new(0.9).unwrap());

        let repo = Box::new(MockMemoryRepository::new().with_memories(vec![test_memory]));
        let config = create_test_config();
        let mut service = MemoryService::new(repo, config);

        let result = service.recall("rust", 10, None, vec![]);
        assert!(result.is_ok());

        let markdown = result.unwrap();
        assert!(markdown.contains("# Search Results"));
        assert!(markdown.contains("## Rust Memory"));
    }

    #[test]
    fn test_memory_service_recall_with_filters() {
        // Red: テスト先行 - フィルターでの検索
        let memories = vec![
            Memory::new(
                "tech".to_string(),
                "Tech Memory".to_string(),
                "Content".to_string(),
            )
            .with_tags(vec!["important".to_string()])
            .with_confidence(Confidence::new(0.9).unwrap()),
            Memory::new(
                "domain".to_string(),
                "Domain Memory".to_string(),
                "Content".to_string(),
            )
            .with_tags(vec!["normal".to_string()])
            .with_confidence(Confidence::new(0.7).unwrap()),
        ];

        let repo = Box::new(MockMemoryRepository::new().with_memories(memories));
        let config = create_test_config();
        let mut service = MemoryService::new(repo, config);

        let result = service.recall(
            "memory",
            10,
            Some("tech".to_string()),
            vec!["important".to_string()],
        );
        assert!(result.is_ok());

        let markdown = result.unwrap();
        assert!(markdown.contains("Tech Memory"));
        assert!(!markdown.contains("Domain Memory"));
    }

    #[tokio::test]
    async fn test_mcp_server_new() {
        // Red: テスト先行 - MCPサーバー初期化
        let service = create_test_service();
        let server = MemoryMcpServer::new(service);

        // サーバーが正常に初期化されることを確認
        let info = server.get_info();
        assert!(info.instructions.is_some());
        assert!(info.capabilities.tools.is_some());
    }

    #[tokio::test]
    async fn test_mcp_server_remember_tool() {
        // Red: テスト先行 - rememberツールの実行
        let service = create_test_service();
        let server = MemoryMcpServer::new(service);

        let params = Parameters(RememberParams {
            memories: vec![create_test_memory_input()],
        });

        let result = server.remember(params).await;
        assert!(result.is_ok());

        let tool_result = result.unwrap();
        assert!(tool_result.is_error.is_none() || !tool_result.is_error.unwrap());
        assert!(tool_result.content.is_some());

        // レスポンスの内容を確認 - とりあえず構造が正しいことを確認
        let content = tool_result.content.unwrap();
        assert!(!content.is_empty());
    }

    #[tokio::test]
    async fn test_mcp_server_remember_tool_multiple() {
        // Red: テスト先行 - 複数メモリのrememberツール
        let service = create_test_service();
        let server = MemoryMcpServer::new(service);

        let params = Parameters(RememberParams {
            memories: vec![
                create_test_memory_input(),
                MemoryInput {
                    r#type: "domain".to_string(),
                    title: "Domain Memory".to_string(),
                    content: "Domain content".to_string(),
                    tags: vec!["domain".to_string()],
                    confidence: Some(0.7),
                },
            ],
        });

        let result = server.remember(params).await;
        assert!(result.is_ok());

        let tool_result = result.unwrap();
        assert!(tool_result.content.is_some());
        let content = tool_result.content.unwrap();
        assert!(!content.is_empty());
    }

    #[tokio::test]
    async fn test_mcp_server_remember_tool_invalid_type() {
        // Red: テスト先行 - 無効なタイプでのrememberツール
        let service = create_test_service();
        let server = MemoryMcpServer::new(service);

        let params = Parameters(RememberParams {
            memories: vec![MemoryInput {
                r#type: "invalid-type".to_string(),
                title: "Test Memory".to_string(),
                content: "Test content".to_string(),
                tags: vec![],
                confidence: None,
            }],
        });

        let result = server.remember(params).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        // エラーが返されることを確認（具体的なエラータイプは省略）
        assert!(format!("{:?}", error).contains("Failed to store memories"));
    }

    #[tokio::test]
    async fn test_mcp_server_recall_tool() {
        // Red: テスト先行 - recallツールの実行
        let test_memory = Memory::new(
            "tech".to_string(),
            "Test Memory".to_string(),
            "Test content".to_string(),
        );

        let repo = Box::new(MockMemoryRepository::new().with_memories(vec![test_memory]));
        let config = create_test_config();
        let service = MemoryService::new(repo, config);
        let server = MemoryMcpServer::new(service);

        let params = Parameters(RecallParams {
            query: "test".to_string(),
            limit: Some(10),
            r#type: None,
            tags: None,
        });

        let result = server.recall(params).await;
        assert!(result.is_ok());

        let tool_result = result.unwrap();
        assert!(tool_result.is_error.is_none() || !tool_result.is_error.unwrap());
        assert!(tool_result.content.is_some());

        // 基本的なレスポンス構造を確認
        let content = tool_result.content.unwrap();
        assert!(!content.is_empty());
    }

    #[tokio::test]
    async fn test_mcp_server_recall_tool_with_filters() {
        // Red: テスト先行 - フィルター付きrecallツール
        let memories = vec![
            Memory::new(
                "tech".to_string(),
                "Tech Memory".to_string(),
                "Content".to_string(),
            )
            .with_tags(vec!["important".to_string()]),
            Memory::new(
                "domain".to_string(),
                "Domain Memory".to_string(),
                "Content".to_string(),
            )
            .with_tags(vec!["normal".to_string()]),
        ];

        let repo = Box::new(MockMemoryRepository::new().with_memories(memories));
        let config = create_test_config();
        let service = MemoryService::new(repo, config);
        let server = MemoryMcpServer::new(service);

        let params = Parameters(RecallParams {
            query: "memory".to_string(),
            limit: Some(10),
            r#type: Some("tech".to_string()),
            tags: Some(vec!["important".to_string()]),
        });

        let result = server.recall(params).await;
        assert!(result.is_ok());

        let tool_result = result.unwrap();
        assert!(tool_result.content.is_some());
        let content = tool_result.content.unwrap();
        assert!(!content.is_empty());
    }

    #[tokio::test]
    async fn test_mcp_server_recall_tool_empty_results() {
        // Red: テスト先行 - 結果が空のrecallツール
        let service = create_test_service();
        let server = MemoryMcpServer::new(service);

        let params = Parameters(RecallParams {
            query: "nonexistent".to_string(),
            limit: Some(10),
            r#type: None,
            tags: None,
        });

        let result = server.recall(params).await;
        assert!(result.is_ok());

        let tool_result = result.unwrap();
        assert!(tool_result.content.is_some());
        let content = tool_result.content.unwrap();
        assert!(!content.is_empty());
    }

    #[test]
    fn test_server_info() {
        // Red: テスト先行 - ServerInfoの設定
        let service = create_test_service();
        let server = MemoryMcpServer::new(service);

        let info = server.get_info();
        assert_eq!(info.protocol_version, ProtocolVersion::V_2024_11_05);
        assert!(info.capabilities.tools.is_some());
        assert!(info.instructions.is_some());

        let instructions = info.instructions.unwrap();
        assert!(instructions.contains("Memory MCP Server v3"));
        assert!(instructions.contains("tech:"));
        assert!(instructions.contains("project-tech:"));
        assert!(instructions.contains("domain:"));
    }

    #[test]
    fn test_memory_input_to_remember_request_conversion() {
        // Red: テスト先行 - MemoryInputからRememberRequestへの変換
        let memory_input = MemoryInput {
            r#type: "tech".to_string(),
            title: "Test Title".to_string(),
            content: "Test Content".to_string(),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            confidence: Some(0.85),
        };

        let request = RememberRequest {
            memory_type: memory_input.r#type.clone(),
            title: memory_input.title.clone(),
            content: memory_input.content.clone(),
            tags: memory_input.tags.clone(),
            confidence: memory_input.confidence,
        };

        assert_eq!(request.memory_type, "tech");
        assert_eq!(request.title, "Test Title");
        assert_eq!(request.content, "Test Content");
        assert_eq!(request.tags, vec!["tag1", "tag2"]);
        assert_eq!(request.confidence, Some(0.85));
    }

    #[test]
    fn test_request_response_serialization() {
        // Red: テスト先行 - リクエスト/レスポンスのシリアライゼーション
        let remember_response = RememberResponse {
            memory_ids: vec!["id1".to_string(), "id2".to_string()],
            created_count: 2,
        };

        let json = serde_json::to_string(&remember_response).unwrap();
        assert!(json.contains("memory_ids"));
        assert!(json.contains("created_count"));
        assert!(json.contains("id1"));
        assert!(json.contains("id2"));

        let recall_response = RecallResponse {
            content: "# Results\n\nTest content".to_string(),
            total_count: 1,
        };

        let json = serde_json::to_string(&recall_response).unwrap();
        assert!(json.contains("content"));
        assert!(json.contains("total_count"));
    }
}
