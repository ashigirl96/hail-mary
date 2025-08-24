# MCP Rust SDK Reference

An official Rust Model Context Protocol SDK implementation with tokio async runtime.

## Crates

- **rmcp**: The core crate providing the RMCP protocol implementation
- **rmcp-macros**: A procedural macro crate for generating RMCP tool implementations

## Installation

Add to your `Cargo.toml`:

```toml
# Basic server setup
rmcp = { version = "0.5.0", features = ["server", "macros"] }

# With stdio transport support
rmcp = { version = "0.5.0", features = ["server", "macros", "transport-io"] }

# Development channel
rmcp = { git = "https://github.com/modelcontextprotocol/rust-sdk", branch = "main", features = ["server", "macros", "transport-io"] }
```

## Dependencies

Basic dependencies:
- [tokio](https://github.com/tokio-rs/tokio) (required)
- [serde](https://github.com/serde-rs/serde) (required)
- [schemars](https://github.com/GREsau/schemars) (for structured output schemas)

## Building a Server

### Complete Server Implementation

Here's a complete implementation of a Memory MCP server using the latest patterns:

```rust
use rmcp::{
    ErrorData as McpError, Json, ServiceExt,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    tool, tool_handler, tool_router,
    transport::stdio,
    serve_server,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{future::Future, sync::Arc};
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RememberParams {
    pub r#type: String,
    pub topic: String,
    pub content: String,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RememberResponse {
    pub memory_id: String,
    pub action: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RecallParams {
    pub query: String,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RecallResponse {
    pub memories: Vec<Memory>,
    pub total_count: usize,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Memory {
    pub id: String,
    pub topic: String,
    pub content: String,
    pub tags: Vec<String>,
}

#[derive(Clone)]
pub struct MemoryServer {
    memories: Arc<Mutex<Vec<Memory>>>,
    tool_router: ToolRouter<Self>,
}

#[tool_handler(router = self.tool_router)]
impl rmcp::ServerHandler for MemoryServer {}

#[tool_router(router = tool_router)]
impl MemoryServer {
    pub fn new() -> Self {
        Self {
            memories: Arc::new(Mutex::new(Vec::new())),
            tool_router: Self::tool_router(),
        }
    }

    /// Store a memory for future recall
    #[tool(name = "remember", description = "Store a memory for future recall")]
    pub async fn remember(
        &self,
        params: Parameters<RememberParams>,
    ) -> Result<Json<RememberResponse>, McpError> {
        let mut memories = self.memories.lock().await;
        let memory_id = format!("mem_{}", memories.len() + 1);
        
        let memory = Memory {
            id: memory_id.clone(),
            topic: params.0.topic,
            content: params.0.content,
            tags: params.0.tags.unwrap_or_default(),
        };
        
        memories.push(memory);
        
        Ok(Json(RememberResponse {
            memory_id,
            action: "created".to_string(),
        }))
    }

    /// Search and retrieve stored memories
    #[tool(name = "recall", description = "Search and retrieve stored memories")]
    pub async fn recall(
        &self,
        params: Parameters<RecallParams>,
    ) -> Result<Json<RecallResponse>, McpError> {
        let memories = self.memories.lock().await;
        let limit = params.0.limit.unwrap_or(10) as usize;
        
        let filtered: Vec<Memory> = memories
            .iter()
            .filter(|m| {
                m.content.contains(&params.0.query) || 
                m.topic.contains(&params.0.query) ||
                m.tags.iter().any(|tag| tag.contains(&params.0.query))
            })
            .take(limit)
            .cloned()
            .collect();
        
        Ok(Json(RecallResponse {
            total_count: filtered.len(),
            memories: filtered,
        }))
    }

    /// Get server information
    #[tool(name = "get_info", description = "Get server information")]
    pub async fn get_info(&self) -> String {
        "Memory MCP Server v1.0 - Store and retrieve memories".to_string()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = MemoryServer::new();
    
    // Method 1: Using serve_server function
    serve_server(server, stdio()).await?;
    
    // Method 2: Using ServiceExt trait
    // let service = server.serve(stdio()).await?;
    // service.waiting().await?;
    
    Ok(())
}
```

### Key Components Explained

#### 1. Tool Router Pattern
```rust
#[derive(Clone)]
pub struct MyServer {
    tool_router: ToolRouter<Self>,  // Required for tool routing
}

#[tool_router(router = tool_router)]  // Generates the tool_router() method
impl MyServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),  // Auto-generated method
        }
    }
}
```

#### 2. ServerHandler Implementation
```rust
#[tool_handler(router = self.tool_router)]
impl rmcp::ServerHandler for MyServer {}
```

#### 3. Tool Definition
```rust
#[tool(name = "tool_name", description = "Tool description")]
pub async fn my_tool(
    &self,
    params: Parameters<MyParams>,
) -> Result<Json<MyResponse>, McpError> {
    // Tool implementation
    Ok(Json(MyResponse { /* ... */ }))
}
```

## Structured Output

### JSON Schema Generation

The SDK automatically generates JSON schemas for structured output:

```rust
use rmcp::{Json, tool, handler::server::tool::Parameters};
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, JsonSchema)]
struct CalculationRequest {
    pub numbers: Vec<i32>,
    pub operation: String,  // "sum", "average", "product"
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct CalculationResult {
    pub result: f64,
    pub operation: String,
    pub input_count: usize,
}

#[tool(name = "calculate", description = "Perform calculations on numbers")]
pub async fn calculate(
    &self,
    params: Parameters<CalculationRequest>,
) -> Result<Json<CalculationResult>, String> {
    let numbers = &params.0.numbers;
    if numbers.is_empty() {
        return Err("No numbers provided".to_string());
    }

    let result = match params.0.operation.as_str() {
        "sum" => numbers.iter().sum::<i32>() as f64,
        "average" => numbers.iter().sum::<i32>() as f64 / numbers.len() as f64,
        "product" => numbers.iter().product::<i32>() as f64,
        _ => return Err(format!("Unknown operation: {}", params.0.operation)),
    };

    Ok(Json(CalculationResult {
        result,
        operation: params.0.operation,
        input_count: numbers.len(),
    }))
}
```

### Mixed Output Types

Tools can return either structured JSON or plain text:

```rust
// Structured output
#[tool(name = "get_weather")]
pub async fn get_weather(&self, params: Parameters<WeatherRequest>) 
    -> Result<Json<WeatherResponse>, String> {
    // Returns structured JSON with schema
}

// Plain text output  
#[tool(name = "get_info")]
pub async fn get_info(&self) -> String {
    "Server information as plain text".to_string()
}
```

## Transport Options

### Standard I/O (Most Common)
```rust
use rmcp::transport::stdio;

let service = server.serve(stdio()).await?;
```

### Manual Transport Setup
```rust
use tokio::io::{stdin, stdout};

let transport = (stdin(), stdout());
let service = server.serve(transport).await?;
```

## Building a Client

```rust
use rmcp::{
    model::CallToolRequestParam,
    service::ServiceExt,
    transport::{TokioChildProcess, ConfigureCommandExt}
};
use tokio::process::Command;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Connect to a server via child process
    let service = ().serve(TokioChildProcess::new(Command::new("uvx").configure(|cmd| {
        cmd.arg("mcp-server-git");
    }))?).await?;

    // Initialize
    let server_info = service.peer_info();
    println!("Connected to server: {server_info:#?}");

    // List available tools
    let tools = service.list_tools(Default::default()).await?;
    println!("Available tools: {tools:#?}");

    // Call a tool
    let tool_result = service
        .call_tool(CallToolRequestParam {
            name: "git_status".into(),
            arguments: serde_json::json!({ "repo_path": "." }).as_object().cloned(),
        })
        .await?;
    println!("Tool result: {tool_result:#?}");

    service.cancel().await?;
    Ok(())
}
```

## Server Lifecycle

### Starting the Server
```rust
// Option 1: serve_server function (recommended)
serve_server(server, stdio()).await?;

// Option 2: ServiceExt trait
let service = server.serve(stdio()).await?;
service.waiting().await?;
```

### Graceful Shutdown
```rust
let service = server.serve(stdio()).await?;

// Wait for shutdown signal
tokio::select! {
    _ = tokio::signal::ctrl_c() => {
        println!("Received shutdown signal");
    }
    result = service.waiting() => {
        println!("Service ended: {:?}", result);
    }
}
```

## Error Handling

### Tool Errors
```rust
use rmcp::ErrorData as McpError;

#[tool(name = "my_tool")]
pub async fn my_tool(&self, params: Parameters<MyParams>) 
    -> Result<Json<MyResponse>, McpError> {
    
    if params.0.value < 0 {
        return Err(McpError {
            code: -32602,  // Invalid params
            message: "Value must be non-negative".to_string(),
            data: None,
        });
    }
    
    Ok(Json(MyResponse { /* ... */ }))
}
```

### String Errors (Simpler)
```rust
#[tool(name = "simple_tool")]
pub async fn simple_tool(&self, params: Parameters<MyParams>) 
    -> Result<String, String> {
    
    if params.0.value < 0 {
        return Err("Value must be non-negative".to_string());
    }
    
    Ok("Success".to_string())
}
```

## Advanced Features

### Tool Schema Introspection
```rust
// List all tools with their schemas
for tool in server.tool_router.list_all() {
    println!("Tool: {}", tool.name);
    println!("Description: {}", tool.description.unwrap_or_default());
    
    if let Some(input_schema) = &tool.input_schema {
        println!("Input schema: {}", serde_json::to_string_pretty(input_schema)?);
    }
    
    if let Some(output_schema) = &tool.output_schema {
        println!("Output schema: {}", serde_json::to_string_pretty(output_schema)?);
    }
}
```

### Custom Error Types
```rust
use rmcp::ErrorData as McpError;
use thiserror::Error;

#[derive(Error, Debug)]
enum MyError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Database error: {0}")]
    Database(String),
}

impl From<MyError> for McpError {
    fn from(err: MyError) -> Self {
        McpError {
            code: -32603,  // Internal error
            message: err.to_string(),
            data: None,
        }
    }
}
```

## Best Practices

### 1. Structure Organization
```rust
// Separate request/response types
mod types {
    use serde::{Deserialize, Serialize};
    use schemars::JsonSchema;
    
    #[derive(Debug, Serialize, Deserialize, JsonSchema)]
    pub struct RememberParams { /* ... */ }
    
    #[derive(Debug, Serialize, Deserialize, JsonSchema)]
    pub struct RememberResponse { /* ... */ }
}

// Main server implementation
use types::*;
```

### 2. Error Handling
```rust
// Use specific error types for better debugging
#[tool(name = "my_tool")]
pub async fn my_tool(&self, params: Parameters<MyParams>) 
    -> Result<Json<MyResponse>, McpError> {
    
    let value = params.0.value
        .ok_or_else(|| McpError {
            code: -32602,
            message: "Missing required field 'value'".to_string(),
            data: None,
        })?;
    
    // ... tool logic
}
```

### 3. Async State Management
```rust
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct MyServer {
    state: Arc<Mutex<MyState>>,
    tool_router: ToolRouter<Self>,
}

// Always use Arc<Mutex<T>> for shared state
```

## Integration with Existing Memory MCP

For integrating with the existing hail-mary Memory MCP implementation:

```rust
// Replace the custom JSON-RPC implementation with rmcp
use rmcp::{serve_server, transport::stdio};

// Use existing MemoryService but wrap in rmcp server
#[derive(Clone)]
pub struct MemoryMcpServer {
    service: Arc<Mutex<MemoryService<SqliteMemoryRepository>>>,
    tool_router: ToolRouter<Self>,
}

#[tool_handler(router = self.tool_router)]
impl rmcp::ServerHandler for MemoryMcpServer {}

#[tool_router(router = tool_router)]
impl MemoryMcpServer {
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let repository = SqliteMemoryRepository::new(db_path)?;
        let service = MemoryService::new(repository);
        
        Ok(Self {
            service: Arc::new(Mutex::new(service)),
            tool_router: Self::tool_router(),
        })
    }
    
    #[tool(name = "remember", description = "Store a memory")]
    pub async fn remember(&self, params: Parameters<RememberParams>) 
        -> Result<Json<RememberResponse>, McpError> {
        let mut service = self.service.lock().await;
        let response = service.remember(params.0.into()).await
            .map_err(|e| McpError {
                code: -32603,
                message: e.to_string(),
                data: None,
            })?;
        Ok(Json(response.into()))
    }
    
    #[tool(name = "recall", description = "Search memories")]
    pub async fn recall(&self, params: Parameters<RecallParams>) 
        -> Result<Json<RecallResponse>, McpError> {
        let service = self.service.lock().await;
        let response = service.recall(params.0.into()).await
            .map_err(|e| McpError {
                code: -32603,
                message: e.to_string(),
                data: None,
            })?;
        Ok(Json(response.into()))
    }
}

// In main function
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = MemoryMcpServer::new("memory.db")?;
    serve_server(server, stdio()).await?;
    Ok(())
}
```

## Related Resources

- [MCP Specification](https://spec.modelcontextprotocol.io/specification/2024-11-05/)
- [Schema](https://github.com/modelcontextprotocol/specification/blob/main/schema/2024-11-05/schema.ts)
- [Official Rust SDK Repository](https://github.com/modelcontextprotocol/rust-sdk)
- [Examples Directory](https://github.com/modelcontextprotocol/rust-sdk/tree/main/examples)