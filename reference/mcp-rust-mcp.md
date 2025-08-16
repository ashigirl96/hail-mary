# MCP Rust SDK Reference

An official Rust Model Context Protocol SDK implementation with tokio async runtime.

## Crates

- **rmcp**: The core crate providing the RMCP protocol implementation
- **rmcp-macros**: A procedural macro crate for generating RMCP tool implementations

## Installation

Add to your `Cargo.toml`:

```toml
rmcp = { version = "0.2.0", features = ["server"] }
## or development channel
rmcp = { git = "https://github.com/modelcontextprotocol/rust-sdk", branch = "main" }
```

## Dependencies

Basic dependencies:
- [tokio](https://github.com/tokio-rs/tokio) (required)
- [serde](https://github.com/serde-rs/serde) (required)

## Building a Client

```rust
use rmcp::{ServiceExt, transport::{TokioChildProcess, ConfigureCommandExt}};
use tokio::process::Command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ().serve(TokioChildProcess::new(Command::new("npx").configure(|cmd| {
        cmd.arg("-y").arg("@modelcontextprotocol/server-everything");
    }))?).await?;
    Ok(())
}
```

## Building a Server

### Build a transport
```rust
use tokio::io::{stdin, stdout};
let transport = (stdin(), stdout());
```

### Build a service
You can easily build a service by using [`ServerHandler`] or [`ClientHandler`].

```rust
let service = common::counter::Counter::new();
```

### Start the server
```rust
// this call will finish the initialization process
let server = service.serve(transport).await?;
```

### Interact with the server
Once the server is initialized, you can send requests or notifications:

```rust
// request
let roots = server.list_roots().await?;

// or send notification
server.notify_cancelled(...).await?;
```

### Waiting for service shutdown
```rust
let quit_reason = server.waiting().await?;
// or cancel it
let quit_reason = server.cancel().await?;
```

## Examples

### Example 1: Simple Server with stdin/stdout

```rust
use std::error::Error;
mod common;
use common::generic_service::{GenericService, MemoryDataService};
use rmcp::serve_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let memory_service = MemoryDataService::new("initial data");

    let generic_service = GenericService::new(memory_service);

    println!("start server, connect to standard input/output");

    let io = (tokio::io::stdin(), tokio::io::stdout());

    serve_server(generic_service, io).await?;
    Ok(())
}
```

### Example 2: HTTP Server with Local Session Management

```rust
mod common;
use common::counter::Counter;
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto::Builder,
    service::TowerToHyperService,
};
use rmcp::transport::streamable_http_server::{
    StreamableHttpService, session::local::LocalSessionManager,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let service = TowerToHyperService::new(StreamableHttpService::new(
        || Ok(Counter::new()),
        LocalSessionManager::default().into(),
        Default::default(),
    ));
    let listener = tokio::net::TcpListener::bind("[::1]:8080").await?;
    loop {
        let io = tokio::select! {
            _ = tokio::signal::ctrl_c() => break,
            accept = listener.accept() => {
                TokioIo::new(accept?.0)
            }
        };
        let service = service.clone();
        tokio::spawn(async move {
            let _result = Builder::new(TokioExecutor::default())
                .serve_connection(io, service)
                .await;
        });
    }
    Ok(())
}
```

### Example 3: Structured Output Server with Tool Router

```rust
use rmcp::{
    Json, ServiceExt,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    tool, tool_handler, tool_router,
    transport::stdio,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WeatherRequest {
    pub city: String,
    pub units: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WeatherResponse {
    pub temperature: f64,
    pub description: String,
    pub humidity: u8,
    pub wind_speed: f64,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CalculationRequest {
    pub numbers: Vec<i32>,
    pub operation: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CalculationResult {
    pub result: f64,
    pub operation: String,
    pub input_count: usize,
}

#[derive(Clone)]
pub struct StructuredOutputServer {
    tool_router: ToolRouter<Self>,
}

#[tool_handler(router = self.tool_router)]
impl rmcp::ServerHandler for StructuredOutputServer {}

impl Default for StructuredOutputServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_router(router = tool_router)]
impl StructuredOutputServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    /// Get weather information for a city (returns structured data)
    #[tool(name = "get_weather", description = "Get current weather for a city")]
    pub async fn get_weather(
        &self,
        params: Parameters<WeatherRequest>,
    ) -> Result<Json<WeatherResponse>, String> {
        // Simulate weather API call
        let weather = WeatherResponse {
            temperature: match params.0.units.as_deref() {
                Some("fahrenheit") => 72.5,
                _ => 22.5, // celsius by default
            },
            description: "Partly cloudy".to_string(),
            humidity: 65,
            wind_speed: 12.5,
        };

        Ok(Json(weather))
    }

    /// Perform calculations on a list of numbers (returns structured data)
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

    /// Get server info (returns unstructured text)
    #[tool(name = "get_info", description = "Get server information")]
    pub async fn get_info(&self) -> String {
        "Structured Output Example Server v1.0".to_string()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    eprintln!("Starting structured output example server...");
    eprintln!();
    eprintln!("This server demonstrates:");
    eprintln!("- Tools that return structured JSON data");
    eprintln!("- Automatic output schema generation");
    eprintln!("- Mixed structured and unstructured outputs");
    eprintln!();
    eprintln!("Tools available:");
    eprintln!("- get_weather: Returns structured weather data");
    eprintln!("- calculate: Returns structured calculation results");
    eprintln!("- get_info: Returns plain text");
    eprintln!();

    let server = StructuredOutputServer::new();

    // Print the tools with their schemas for demonstration
    eprintln!("Tool schemas:");
    for tool in server.tool_router.list_all() {
        eprintln!("\n{}: {}", tool.name, tool.description.unwrap_or_default());
        if let Some(output_schema) = &tool.output_schema {
            eprintln!(
                "  Output schema: {}",
                serde_json::to_string_pretty(output_schema).unwrap()
            );
        } else {
            eprintln!("  Output: Unstructured text");
        }
    }
    eprintln!();

    // Start the server
    eprintln!("Starting server. Connect with an MCP client to test the tools.");
    eprintln!("Press Ctrl+C to stop.");

    let service = server.serve(stdio()).await?;
    service.waiting().await?;

    Ok(())
}
```

## Related Resources

- [MCP Specification](https://spec.modelcontextprotocol.io/specification/2024-11-05/)
- [Schema](https://github.com/modelcontextprotocol/specification/blob/main/schema/2024-11-05/schema.ts)