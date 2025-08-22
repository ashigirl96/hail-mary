This file is a merged representation of a subset of the codebase, containing specifically included files, combined into a single document by Repomix.
The content has been processed where comments have been removed, empty lines have been removed, security check has been disabled.

<file_summary>
This section contains a summary of this file.

<purpose>
This file contains a packed representation of a subset of the repository's contents that is considered the most important context.
It is designed to be easily consumable by AI systems for analysis, code review,
or other automated processes.
</purpose>

<file_format>
The content is organized as follows:
1. This summary section
2. Repository information
3. Directory structure
4. Repository files (if enabled)
5. Multiple file entries, each consisting of:
  - File path as an attribute
  - Full contents of the file
</file_format>

<usage_guidelines>
- This file should be treated as read-only. Any changes should be made to the
  original repository files, not this packed version.
- When processing this file, use the file path to distinguish
  between different files in the repository.
- Be aware that this file may contain sensitive information. Handle it with
  the same level of security as you would the original repository.
</usage_guidelines>

<notes>
- Some files may have been excluded based on .gitignore rules and Repomix's configuration
- Binary files are not included in this packed representation. Please refer to the Repository Structure section for a complete list of file paths, including binary files
- Only files matching these patterns are included: examples/**/*.rs
- Files matching patterns in .gitignore are excluded
- Files matching default ignore patterns are excluded
- Code comments have been removed from supported file types
- Empty lines have been removed from all files
- Security check has been disabled - content may contain sensitive information
- Files are sorted by Git change count (files with more changes are at the bottom)
</notes>

</file_summary>

<directory_structure>
examples/
  clients/
    src/
      auth/
        oauth_client.rs
      collection.rs
      everything_stdio.rs
      git_stdio.rs
      sampling_stdio.rs
      sse.rs
      streamable_http.rs
  rig-integration/
    src/
      config/
        mcp.rs
      chat.rs
      config.rs
      main.rs
      mcp_adaptor.rs
  servers/
    src/
      common/
        calculator.rs
        counter.rs
        generic_service.rs
        mod.rs
      complex_auth_sse.rs
      counter_hyper_streamable_http.rs
      counter_sse_directly.rs
      counter_sse.rs
      counter_stdio.rs
      counter_streamhttp.rs
      memory_stdio.rs
      sampling_stdio.rs
      simple_auth_sse.rs
      structured_output.rs
  simple-chat-client/
    src/
      bin/
        simple_chat.rs
      chat.rs
      client.rs
      config.rs
      error.rs
      lib.rs
      model.rs
      tool.rs
  transport/
    src/
      common/
        calculator.rs
        mod.rs
      http_upgrade.rs
      named-pipe.rs
      tcp.rs
      unix_socket.rs
      websocket.rs
  wasi/
    src/
      calculator.rs
      lib.rs
</directory_structure>

<files>
This section contains the contents of the repository's files.

<file path="examples/clients/src/auth/oauth_client.rs">
use std::{net::SocketAddr, sync::Arc};
use anyhow::{Context, Result};
use axum::{
    Router,
    extract::{Query, State},
    response::Html,
    routing::get,
};
use rmcp::{
    ServiceExt,
    model::ClientInfo,
    transport::{
        SseClientTransport,
        auth::{AuthClient, OAuthState},
        sse_client::SseClientConfig,
    },
};
use serde::Deserialize;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    sync::{Mutex, oneshot},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
const MCP_SERVER_URL: &str = "http://localhost:3000/mcp";
const MCP_REDIRECT_URI: &str = "http://localhost:8080/callback";
const MCP_SSE_URL: &str = "http://localhost:3000/mcp/sse";
const CALLBACK_PORT: u16 = 8080;
const CALLBACK_HTML: &str = include_str!("callback.html");
#[derive(Clone)]
struct AppState {
    code_receiver: Arc<Mutex<Option<oneshot::Sender<String>>>>,
}
#[derive(Debug, Deserialize)]
struct CallbackParams {
    code: String,
    #[allow(dead_code)]
    state: Option<String>,
}
async fn callback_handler(
    Query(params): Query<CallbackParams>,
    State(state): State<AppState>,
) -> Html<String> {
    tracing::info!("Received callback with code: {}", params.code);
    if let Some(sender) = state.code_receiver.lock().await.take() {
        let _ = sender.send(params.code);
    }
    Html(CALLBACK_HTML.to_string())
}
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let (code_sender, code_receiver) = oneshot::channel::<String>();
    let app_state = AppState {
        code_receiver: Arc::new(Mutex::new(Some(code_sender))),
    };
    let app = Router::new()
        .route("/callback", get(callback_handler))
        .with_state(app_state);
    let addr = SocketAddr::from(([127, 0, 0, 1], CALLBACK_PORT));
    tracing::info!("Starting callback server at: http://{}", addr);
    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        let result = axum::serve(listener, app).await;
        if let Err(e) = result {
            tracing::error!("Callback server error: {}", e);
        }
    });
    let server_url = MCP_SERVER_URL.to_string();
    tracing::info!("Using MCP server URL: {}", server_url);
    let mut oauth_state = OAuthState::new(&server_url, None)
        .await
        .context("Failed to initialize oauth state machine")?;
    oauth_state
        .start_authorization(&["mcp", "profile", "email"], MCP_REDIRECT_URI)
        .await
        .context("Failed to start authorization")?;
    let mut output = BufWriter::new(tokio::io::stdout());
    output.write_all(b"\n=== MCP OAuth Client ===\n\n").await?;
    output
        .write_all(b"Please open the following URL in your browser to authorize:\n\n")
        .await?;
    output
        .write_all(oauth_state.get_authorization_url().await?.as_bytes())
        .await?;
    output
        .write_all(b"\n\nWaiting for browser callback, please do not close this window...\n")
        .await?;
    output.flush().await?;
    tracing::info!("Waiting for authorization code...");
    let auth_code = code_receiver
        .await
        .context("Failed to get authorization code")?;
    tracing::info!("Received authorization code: {}", auth_code);
    tracing::info!("Exchanging authorization code for access token...");
    oauth_state
        .handle_callback(&auth_code)
        .await
        .context("Failed to handle callback")?;
    tracing::info!("Successfully obtained access token");
    output
        .write_all(b"\nAuthorization successful! Access token obtained.\n\n")
        .await?;
    output.flush().await?;
    tracing::info!("Establishing authorized connection to MCP server...");
    let am = oauth_state
        .into_authorization_manager()
        .ok_or_else(|| anyhow::anyhow!("Failed to get authorization manager"))?;
    let client = AuthClient::new(reqwest::Client::default(), am);
    let transport = SseClientTransport::start_with_client(
        client,
        SseClientConfig {
            sse_endpoint: MCP_SSE_URL.into(),
            ..Default::default()
        },
    )
    .await?;
    let client_service = ClientInfo::default();
    let client = client_service.serve(transport).await?;
    tracing::info!("Successfully connected to MCP server");
    output
        .write_all(b"Fetching available tools from server...\n")
        .await?;
    output.flush().await?;
    match client.peer().list_all_tools().await {
        Ok(tools) => {
            output
                .write_all(format!("Available tools: {}\n\n", tools.len()).as_bytes())
                .await?;
            for tool in tools {
                output
                    .write_all(
                        format!(
                            "- {} ({})\n",
                            tool.name,
                            tool.description.unwrap_or_default()
                        )
                        .as_bytes(),
                    )
                    .await?;
            }
        }
        Err(e) => {
            output
                .write_all(format!("Error fetching tools: {}\n", e).as_bytes())
                .await?;
        }
    }
    output
        .write_all(b"\nFetching available prompts from server...\n")
        .await?;
    output.flush().await?;
    match client.peer().list_all_prompts().await {
        Ok(prompts) => {
            output
                .write_all(format!("Available prompts: {}\n\n", prompts.len()).as_bytes())
                .await?;
            for prompt in prompts {
                output
                    .write_all(format!("- {}\n", prompt.name).as_bytes())
                    .await?;
            }
        }
        Err(e) => {
            output
                .write_all(format!("Error fetching prompts: {}\n", e).as_bytes())
                .await?;
        }
    }
    output
        .write_all(b"\nConnection established successfully. You are now authenticated with the MCP server.\n")
        .await?;
    output.flush().await?;
    output.write_all(b"\nPress Enter to exit...\n").await?;
    output.flush().await?;
    let mut input = String::new();
    let mut reader = BufReader::new(tokio::io::stdin());
    reader.read_line(&mut input).await?;
    Ok(())
}
</file>

<file path="examples/clients/src/collection.rs">
use std::collections::HashMap;
use anyhow::Result;
use rmcp::{
    model::CallToolRequestParam,
    service::ServiceExt,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use tokio::process::Command;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("info,{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let mut clients_map = HashMap::new();
    for idx in 0..10 {
        let client = ()
            .into_dyn()
            .serve(TokioChildProcess::new(Command::new("uvx").configure(
                |cmd| {
                    cmd.arg("mcp-client-git");
                },
            ))?)
            .await?;
        clients_map.insert(idx, client);
    }
    for (_, client) in clients_map.iter() {
        let _server_info = client.peer_info();
        let _tools = client.list_tools(Default::default()).await?;
        let _tool_result = client
            .call_tool(CallToolRequestParam {
                name: "git_status".into(),
                arguments: serde_json::json!({ "repo_path": "." }).as_object().cloned(),
            })
            .await?;
    }
    for (_, service) in clients_map {
        service.cancel().await?;
    }
    Ok(())
}
</file>

<file path="examples/clients/src/everything_stdio.rs">
use anyhow::Result;
use rmcp::{
    ServiceExt,
    model::{CallToolRequestParam, GetPromptRequestParam, ReadResourceRequestParam},
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use tokio::process::Command;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("info,{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let client = ()
        .serve(TokioChildProcess::new(Command::new("npx").configure(
            |cmd| {
                cmd.arg("-y").arg("@modelcontextprotocol/server-everything");
            },
        ))?)
        .await?;
    let server_info = client.peer_info();
    tracing::info!("Connected to server: {server_info:#?}");
    let tools = client.list_all_tools().await?;
    tracing::info!("Available tools: {tools:#?}");
    let tool_result = client
        .call_tool(CallToolRequestParam {
            name: "echo".into(),
            arguments: Some(object!({ "message": "hi from rmcp" })),
        })
        .await?;
    tracing::info!("Tool result for echo: {tool_result:#?}");
    let tool_result = client
        .call_tool(CallToolRequestParam {
            name: "longRunningOperation".into(),
            arguments: Some(object!({ "duration": 3, "steps": 1 })),
        })
        .await?;
    tracing::info!("Tool result for longRunningOperation: {tool_result:#?}");
    let resources = client.list_all_resources().await?;
    tracing::info!("Available resources: {resources:#?}");
    let resource = client
        .read_resource(ReadResourceRequestParam {
            uri: "test://static/resource/3".into(),
        })
        .await?;
    tracing::info!("Resource: {resource:#?}");
    let prompts = client.list_all_prompts().await?;
    tracing::info!("Available prompts: {prompts:#?}");
    let prompt = client
        .get_prompt(GetPromptRequestParam {
            name: "simple_prompt".into(),
            arguments: None,
        })
        .await?;
    tracing::info!("Prompt - simple: {prompt:#?}");
    let prompt = client
        .get_prompt(GetPromptRequestParam {
            name: "complex_prompt".into(),
            arguments: Some(object!({ "temperature": "0.5", "style": "formal" })),
        })
        .await?;
    tracing::info!("Prompt - complex: {prompt:#?}");
    let resource_templates = client.list_all_resource_templates().await?;
    tracing::info!("Available resource templates: {resource_templates:#?}");
    client.cancel().await?;
    Ok(())
}
</file>

<file path="examples/clients/src/git_stdio.rs">
use rmcp::{
    RmcpError,
    model::CallToolRequestParam,
    service::ServiceExt,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use tokio::process::Command;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
#[tokio::main]
async fn main() -> Result<(), RmcpError> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("info,{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let client = ()
        .serve(
            TokioChildProcess::new(Command::new("uvx").configure(|cmd| {
                cmd.arg("mcp-server-git");
            }))
            .map_err(RmcpError::transport_creation::<TokioChildProcess>)?,
        )
        .await?;
    let server_info = client.peer_info();
    tracing::info!("Connected to server: {server_info:#?}");
    let tools = client.list_tools(Default::default()).await?;
    tracing::info!("Available tools: {tools:#?}");
    let tool_result = client
        .call_tool(CallToolRequestParam {
            name: "git_status".into(),
            arguments: serde_json::json!({ "repo_path": "." }).as_object().cloned(),
        })
        .await?;
    tracing::info!("Tool result: {tool_result:#?}");
    client.cancel().await?;
    Ok(())
}
</file>

<file path="examples/clients/src/sampling_stdio.rs">
use anyhow::Result;
use rmcp::{
    ClientHandler, ServiceExt,
    model::*,
    object,
    service::{RequestContext, RoleClient},
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use tokio::process::Command;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
#[derive(Clone, Debug, Default)]
pub struct SamplingDemoClient;
impl SamplingDemoClient {
    fn mock_llm_response(
        &self,
        _messages: &[SamplingMessage],
        _system_prompt: Option<&str>,
    ) -> String {
        "It just a mock response".to_string()
    }
}
impl ClientHandler for SamplingDemoClient {
    async fn create_message(
        &self,
        params: CreateMessageRequestParam,
        _context: RequestContext<RoleClient>,
    ) -> Result<CreateMessageResult, ErrorData> {
        tracing::info!("Received sampling request with {:?}", params);
        let response_text =
            self.mock_llm_response(&params.messages, params.system_prompt.as_deref());
        Ok(CreateMessageResult {
            message: SamplingMessage {
                role: Role::Assistant,
                content: Content::text(response_text),
            },
            model: "mock_llm".to_string(),
            stop_reason: Some(CreateMessageResult::STOP_REASON_END_TURN.to_string()),
        })
    }
}
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("info,{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    tracing::info!("Starting Sampling Demo Client");
    let client = SamplingDemoClient;
    let servers_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("CARGO_MANIFEST_DIR is not set")
        .join("servers");
    let client = client
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run")
                    .arg("--example")
                    .arg("servers_sampling_stdio")
                    .current_dir(servers_dir);
            },
        ))?)
        .await
        .inspect_err(|e| {
            tracing::error!("client error: {:?}", e);
        })?;
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    let server_info = client.peer_info();
    tracing::info!("Connected to server: {server_info:#?}");
    match client.list_all_tools().await {
        Ok(tools) => {
            tracing::info!("Available tools: {tools:#?}");
            tracing::info!("Testing ask_llm tool...");
            match client
                .call_tool(CallToolRequestParam {
                    name: "ask_llm".into(),
                    arguments: Some(object!({
                        "question": "Hello world"
                    })),
                })
                .await
            {
                Ok(result) => tracing::info!("Ask LLM result: {result:#?}"),
                Err(e) => tracing::error!("Ask LLM error: {e}"),
            }
        }
        Err(e) => tracing::error!("Failed to list tools: {e}"),
    }
    tracing::info!("Sampling demo completed successfully!");
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    client.cancel().await?;
    Ok(())
}
</file>

<file path="examples/clients/src/sse.rs">
use anyhow::Result;
use rmcp::{
    ServiceExt,
    model::{CallToolRequestParam, ClientCapabilities, ClientInfo, Implementation},
    transport::SseClientTransport,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("info,{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let transport = SseClientTransport::start("http://localhost:8000/sse").await?;
    let client_info = ClientInfo {
        protocol_version: Default::default(),
        capabilities: ClientCapabilities::default(),
        client_info: Implementation {
            name: "test sse client".to_string(),
            version: "0.0.1".to_string(),
        },
    };
    let client = client_info.serve(transport).await.inspect_err(|e| {
        tracing::error!("client error: {:?}", e);
    })?;
    let server_info = client.peer_info();
    tracing::info!("Connected to server: {server_info:#?}");
    let tools = client.list_tools(Default::default()).await?;
    tracing::info!("Available tools: {tools:#?}");
    let tool_result = client
        .call_tool(CallToolRequestParam {
            name: "increment".into(),
            arguments: serde_json::json!({}).as_object().cloned(),
        })
        .await?;
    tracing::info!("Tool result: {tool_result:#?}");
    client.cancel().await?;
    Ok(())
}
</file>

<file path="examples/clients/src/streamable_http.rs">
use anyhow::Result;
use rmcp::{
    ServiceExt,
    model::{CallToolRequestParam, ClientCapabilities, ClientInfo, Implementation},
    transport::StreamableHttpClientTransport,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("info,{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let transport = StreamableHttpClientTransport::from_uri("http://localhost:8000/mcp");
    let client_info = ClientInfo {
        protocol_version: Default::default(),
        capabilities: ClientCapabilities::default(),
        client_info: Implementation {
            name: "test sse client".to_string(),
            version: "0.0.1".to_string(),
        },
    };
    let client = client_info.serve(transport).await.inspect_err(|e| {
        tracing::error!("client error: {:?}", e);
    })?;
    let server_info = client.peer_info();
    tracing::info!("Connected to server: {server_info:#?}");
    let tools = client.list_tools(Default::default()).await?;
    tracing::info!("Available tools: {tools:#?}");
    let tool_result = client
        .call_tool(CallToolRequestParam {
            name: "increment".into(),
            arguments: serde_json::json!({}).as_object().cloned(),
        })
        .await?;
    tracing::info!("Tool result: {tool_result:#?}");
    client.cancel().await?;
    Ok(())
}
</file>

<file path="examples/rig-integration/src/config/mcp.rs">
use std::{collections::HashMap, process::Stdio};
use rmcp::{RoleClient, ServiceExt, service::RunningService, transport::ConfigureCommandExt};
use serde::{Deserialize, Serialize};
use crate::mcp_adaptor::McpManager;
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct McpServerConfig {
    name: String,
    #[serde(flatten)]
    transport: McpServerTransportConfig,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "protocol", rename_all = "lowercase")]
pub enum McpServerTransportConfig {
    Streamable {
        url: String,
    },
    Sse {
        url: String,
    },
    Stdio {
        command: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        envs: HashMap<String, String>,
    },
}
#[derive(Debug, Serialize, Deserialize)]
pub struct McpConfig {
    server: Vec<McpServerConfig>,
}
impl McpConfig {
    pub async fn create_manager(&self) -> anyhow::Result<McpManager> {
        let mut clients = HashMap::new();
        let mut task_set = tokio::task::JoinSet::<anyhow::Result<_>>::new();
        for server in &self.server {
            let server = server.clone();
            task_set.spawn(async move {
                let client = server.transport.start().await?;
                anyhow::Result::Ok((server.name.clone(), client))
            });
        }
        let start_up_result = task_set.join_all().await;
        for result in start_up_result {
            match result {
                Ok((name, client)) => {
                    clients.insert(name, client);
                }
                Err(e) => {
                    eprintln!("Failed to start server: {:?}", e);
                }
            }
        }
        Ok(McpManager { clients })
    }
}
impl McpServerTransportConfig {
    pub async fn start(&self) -> anyhow::Result<RunningService<RoleClient, ()>> {
        let client = match self {
            McpServerTransportConfig::Streamable { url } => {
                let transport =
                    rmcp::transport::StreamableHttpClientTransport::from_uri(url.to_string());
                ().serve(transport).await?
            }
            McpServerTransportConfig::Sse { url } => {
                let transport = rmcp::transport::SseClientTransport::start(url.to_string()).await?;
                ().serve(transport).await?
            }
            McpServerTransportConfig::Stdio {
                command,
                args,
                envs,
            } => {
                let transport = rmcp::transport::TokioChildProcess::new(
                    tokio::process::Command::new(command).configure(|cmd| {
                        cmd.args(args).envs(envs).stderr(Stdio::null());
                    }),
                )?;
                ().serve(transport).await?
            }
        };
        Ok(client)
    }
}
</file>

<file path="examples/rig-integration/src/chat.rs">
use futures::StreamExt;
use rig::{
    agent::Agent,
    completion::{AssistantContent, CompletionModel},
    message::Message,
    streaming::StreamingChat,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
pub async fn cli_chatbot<M>(chatbot: Agent<M>) -> anyhow::Result<()>
where
    M: CompletionModel,
{
    let mut chat_log = vec![];
    let mut output = BufWriter::new(tokio::io::stdout());
    let mut input = BufReader::new(tokio::io::stdin());
    output.write_all(b"Enter :q to quit\n").await?;
    loop {
        output.write_all(b"\x1b[32muser>\x1b[0m ").await?;
        // Flush stdout to ensure the prompt appears before input
        output.flush().await?;
        let mut input_buf = String::new();
        input.read_line(&mut input_buf).await?;
        // Remove the newline character from the input
        let input = input_buf.trim();
        // Check for a command to exit
        if input == ":q" {
            break;
        }
        match chatbot.stream_chat(input, chat_log.clone()).await {
            Ok(mut response) => {
                tracing::info!(%input);
                chat_log.push(Message::user(input));
                stream_output_agent_start(&mut output).await?;
                let mut message_buf = String::new();
                while let Some(message) = response.next().await {
                    match message {
                        Ok(AssistantContent::Text(text)) => {
                            message_buf.push_str(&text.text);
                            output_agent(text.text, &mut output).await?;
                        }
                        Ok(AssistantContent::ToolCall(tool_call)) => {
                            let name = tool_call.function.name;
                            let arguments = tool_call.function.arguments;
                            chat_log.push(Message::assistant(format!(
                                "Calling tool: {name} with args: {arguments}"
                            )));
                            let result = chatbot.tools.call(&name, arguments.to_string()).await;
                            match result {
                                Ok(tool_call_result) => {
                                    stream_output_agent_finished(&mut output).await?;
                                    stream_output_toolcall(&tool_call_result, &mut output).await?;
                                    stream_output_agent_start(&mut output).await?;
                                    chat_log.push(Message::user(tool_call_result));
                                }
                                Err(e) => {
                                    output_error(e, &mut output).await?;
                                }
                            }
                        }
                        Err(error) => {
                            output_error(error, &mut output).await?;
                        }
                    }
                }
                chat_log.push(Message::assistant(message_buf));
                stream_output_agent_finished(&mut output).await?;
            }
            Err(error) => {
                output_error(error, &mut output).await?;
            }
        }
    }
    Ok(())
}
pub async fn output_error(
    e: impl std::fmt::Display,
    output: &mut BufWriter<tokio::io::Stdout>,
) -> std::io::Result<()> {
    output
        .write_all(b"\x1b[1;31m\xE2\x9D\x8C ERROR: \x1b[0m")
        .await?;
    output.write_all(e.to_string().as_bytes()).await?;
    output.write_all(b"\n").await?;
    output.flush().await?;
    Ok(())
}
pub async fn output_agent(
    content: impl std::fmt::Display,
    output: &mut BufWriter<tokio::io::Stdout>,
) -> std::io::Result<()> {
    output.write_all(content.to_string().as_bytes()).await?;
    output.flush().await?;
    Ok(())
}
pub async fn stream_output_toolcall(
    content: impl std::fmt::Display,
    output: &mut BufWriter<tokio::io::Stdout>,
) -> std::io::Result<()> {
    output
        .write_all(b"\x1b[1;33m\xF0\x9F\x9B\xA0 Tool Call: \x1b[0m")
        .await?;
    output.write_all(content.to_string().as_bytes()).await?;
    output.write_all(b"\n").await?;
    output.flush().await?;
    Ok(())
}
pub async fn stream_output_agent_start(
    output: &mut BufWriter<tokio::io::Stdout>,
) -> std::io::Result<()> {
    output
        .write_all(b"\x1b[1;34m\xF0\x9F\xA4\x96 Agent: \x1b[0m")
        .await?;
    output.flush().await?;
    Ok(())
}
pub async fn stream_output_agent_finished(
    output: &mut BufWriter<tokio::io::Stdout>,
) -> std::io::Result<()> {
    output.write_all(b"\n").await?;
    output.flush().await?;
    Ok(())
}
</file>

<file path="examples/rig-integration/src/config.rs">
use std::path::Path;
use serde::{Deserialize, Serialize};
pub mod mcp;
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub mcp: mcp::McpConfig,
    pub deepseek_key: Option<String>,
    pub cohere_key: Option<String>,
}
impl Config {
    pub async fn retrieve(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
}
</file>

<file path="examples/rig-integration/src/main.rs">
use rig::{
    client::{CompletionClient, ProviderClient},
    embeddings::EmbeddingsBuilder,
    providers::{cohere, deepseek},
    vector_store::in_memory_store::InMemoryVectorStore,
};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
pub mod chat;
pub mod config;
pub mod mcp_adaptor;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        "logs",
        format!("{}.log", env!("CARGO_CRATE_NAME")),
    );
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_writer(file_appender)
        .with_file(false)
        .with_ansi(false)
        .init();
    let config = config::Config::retrieve("config.toml").await?;
    let openai_client = {
        if let Some(key) = config.deepseek_key {
            deepseek::Client::new(&key)
        } else {
            deepseek::Client::from_env()
        }
    };
    let cohere_client = {
        if let Some(key) = config.cohere_key {
            cohere::Client::new(&key)
        } else {
            cohere::Client::from_env()
        }
    };
    let mcp_manager = config.mcp.create_manager().await?;
    tracing::info!(
        "MCP Manager created, {} servers started",
        mcp_manager.clients.len()
    );
    let tool_set = mcp_manager.get_tool_set().await?;
    let embedding_model =
        cohere_client.embedding_model(cohere::EMBED_MULTILINGUAL_V3, "search_document");
    let embeddings = EmbeddingsBuilder::new(embedding_model.clone())
        .documents(tool_set.schemas()?)?
        .build()
        .await?;
    let store = InMemoryVectorStore::from_documents_with_id_f(embeddings, |f| {
        tracing::info!("store tool {}", f.name);
        f.name.clone()
    });
    let index = store.index(embedding_model);
    let dpsk = openai_client
        .agent(deepseek::DEEPSEEK_CHAT)
        .dynamic_tools(4, index, tool_set)
        .build();
    chat::cli_chatbot(dpsk).await?;
    Ok(())
}
</file>

<file path="examples/rig-integration/src/mcp_adaptor.rs">
use std::collections::HashMap;
use rig::tool::{ToolDyn as RigTool, ToolEmbeddingDyn, ToolSet};
use rmcp::{
    RoleClient,
    model::{CallToolRequestParam, CallToolResult, Tool as McpTool},
    service::{RunningService, ServerSink},
};
pub struct McpToolAdaptor {
    tool: McpTool,
    server: ServerSink,
}
impl RigTool for McpToolAdaptor {
    fn name(&self) -> String {
        self.tool.name.to_string()
    }
    fn definition(
        &self,
        _prompt: String,
    ) -> std::pin::Pin<Box<dyn Future<Output = rig::completion::ToolDefinition> + Send + Sync + '_>>
    {
        Box::pin(std::future::ready(rig::completion::ToolDefinition {
            name: self.name(),
            description: self
                .tool
                .description
                .as_deref()
                .unwrap_or_default()
                .to_string(),
            parameters: self.tool.schema_as_json_value(),
        }))
    }
    fn call(
        &self,
        args: String,
    ) -> std::pin::Pin<
        Box<dyn Future<Output = Result<String, rig::tool::ToolError>> + Send + Sync + '_>,
    > {
        let server = self.server.clone();
        Box::pin(async move {
            let call_mcp_tool_result = server
                .call_tool(CallToolRequestParam {
                    name: self.tool.name.clone(),
                    arguments: serde_json::from_str(&args)
                        .map_err(rig::tool::ToolError::JsonError)?,
                })
                .await
                .inspect(|result| tracing::info!(?result))
                .inspect_err(|error| tracing::error!(%error))
                .map_err(|e| rig::tool::ToolError::ToolCallError(Box::new(e)))?;
            Ok(convert_mcp_call_tool_result_to_string(call_mcp_tool_result))
        })
    }
}
impl ToolEmbeddingDyn for McpToolAdaptor {
    fn context(&self) -> serde_json::Result<serde_json::Value> {
        serde_json::to_value(self.tool.clone())
    }
    fn embedding_docs(&self) -> Vec<String> {
        vec![
            self.tool
                .description
                .as_deref()
                .unwrap_or_default()
                .to_string(),
        ]
    }
}
pub struct McpManager {
    pub clients: HashMap<String, RunningService<RoleClient, ()>>,
}
impl McpManager {
    pub async fn get_tool_set(&self) -> anyhow::Result<ToolSet> {
        let mut tool_set = ToolSet::default();
        let mut task = tokio::task::JoinSet::<anyhow::Result<_>>::new();
        for client in self.clients.values() {
            let server = client.peer().clone();
            task.spawn(get_tool_set(server));
        }
        let results = task.join_all().await;
        for result in results {
            match result {
                Err(e) => {
                    tracing::error!(error = %e, "Failed to get tool set");
                }
                Ok(tools) => {
                    tool_set.add_tools(tools);
                }
            }
        }
        Ok(tool_set)
    }
}
pub fn convert_mcp_call_tool_result_to_string(result: CallToolResult) -> String {
    serde_json::to_string(&result).unwrap()
}
pub async fn get_tool_set(server: ServerSink) -> anyhow::Result<ToolSet> {
    let tools = server.list_all_tools().await?;
    let mut tool_builder = ToolSet::builder();
    for tool in tools {
        tracing::info!("get tool: {}", tool.name);
        let adaptor = McpToolAdaptor {
            tool: tool.clone(),
            server: server.clone(),
        };
        tool_builder = tool_builder.dynamic_tool(adaptor);
    }
    let tool_set = tool_builder.build();
    Ok(tool_set)
}
</file>

<file path="examples/servers/src/common/calculator.rs">
#![allow(dead_code)]
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::Parameters, wrapper::Json},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
};
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SumRequest {
    #[schemars(description = "the left hand side number")]
    pub a: i32,
    pub b: i32,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SubRequest {
    #[schemars(description = "the left hand side number")]
    pub a: i32,
    #[schemars(description = "the right hand side number")]
    pub b: i32,
}
#[derive(Debug, Clone)]
pub struct Calculator {
    tool_router: ToolRouter<Self>,
}
#[tool_router]
impl Calculator {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
    #[tool(description = "Calculate the sum of two numbers")]
    fn sum(&self, Parameters(SumRequest { a, b }): Parameters<SumRequest>) -> String {
        (a + b).to_string()
    }
    #[tool(description = "Calculate the difference of two numbers")]
    fn sub(&self, Parameters(SubRequest { a, b }): Parameters<SubRequest>) -> Json<i32> {
        Json(a - b)
    }
}
#[tool_handler]
impl ServerHandler for Calculator {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("A simple calculator".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
</file>

<file path="examples/servers/src/common/counter.rs">
#![allow(dead_code)]
use std::sync::Arc;
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    schemars,
    service::RequestContext,
    tool, tool_handler, tool_router,
};
use serde_json::json;
use tokio::sync::Mutex;
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StructRequest {
    pub a: i32,
    pub b: i32,
}
#[derive(Clone)]
pub struct Counter {
    counter: Arc<Mutex<i32>>,
    tool_router: ToolRouter<Counter>,
}
#[tool_router]
impl Counter {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            counter: Arc::new(Mutex::new(0)),
            tool_router: Self::tool_router(),
        }
    }
    fn _create_resource_text(&self, uri: &str, name: &str) -> Resource {
        RawResource::new(uri, name.to_string()).no_annotation()
    }
    #[tool(description = "Increment the counter by 1")]
    async fn increment(&self) -> Result<CallToolResult, McpError> {
        let mut counter = self.counter.lock().await;
        *counter += 1;
        Ok(CallToolResult::success(vec![Content::text(
            counter.to_string(),
        )]))
    }
    #[tool(description = "Decrement the counter by 1")]
    async fn decrement(&self) -> Result<CallToolResult, McpError> {
        let mut counter = self.counter.lock().await;
        *counter -= 1;
        Ok(CallToolResult::success(vec![Content::text(
            counter.to_string(),
        )]))
    }
    #[tool(description = "Get the current counter value")]
    async fn get_value(&self) -> Result<CallToolResult, McpError> {
        let counter = self.counter.lock().await;
        Ok(CallToolResult::success(vec![Content::text(
            counter.to_string(),
        )]))
    }
    #[tool(description = "Say hello to the client")]
    fn say_hello(&self) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text("hello")]))
    }
    #[tool(description = "Repeat what you say")]
    fn echo(&self, Parameters(object): Parameters<JsonObject>) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::Value::Object(object).to_string(),
        )]))
    }
    #[tool(description = "Calculate the sum of two numbers")]
    fn sum(
        &self,
        Parameters(StructRequest { a, b }): Parameters<StructRequest>,
    ) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text(
            (a + b).to_string(),
        )]))
    }
}
#[tool_handler]
impl ServerHandler for Counter {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_prompts()
                .enable_resources()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("This server provides a counter tool that can increment and decrement values. The counter starts at 0 and can be modified using the 'increment' and 'decrement' tools. Use 'get_value' to check the current count.".to_string()),
        }
    }
    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult {
            resources: vec![
                self._create_resource_text("str:////Users/to/some/path/", "cwd"),
                self._create_resource_text("memo://insights", "memo-name"),
            ],
            next_cursor: None,
        })
    }
    async fn read_resource(
        &self,
        ReadResourceRequestParam { uri }: ReadResourceRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        match uri.as_str() {
            "str:////Users/to/some/path/" => {
                let cwd = "/Users/to/some/path/";
                Ok(ReadResourceResult {
                    contents: vec![ResourceContents::text(cwd, uri)],
                })
            }
            "memo://insights" => {
                let memo = "Business Intelligence Memo\n\nAnalysis has revealed 5 key insights ...";
                Ok(ReadResourceResult {
                    contents: vec![ResourceContents::text(memo, uri)],
                })
            }
            _ => Err(McpError::resource_not_found(
                "resource_not_found",
                Some(json!({
                    "uri": uri
                })),
            )),
        }
    }
    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        Ok(ListPromptsResult {
            next_cursor: None,
            prompts: vec![Prompt::new(
                "example_prompt",
                Some("This is an example prompt that takes one required argument, message"),
                Some(vec![PromptArgument {
                    name: "message".to_string(),
                    description: Some("A message to put in the prompt".to_string()),
                    required: Some(true),
                }]),
            )],
        })
    }
    async fn get_prompt(
        &self,
        GetPromptRequestParam { name, arguments }: GetPromptRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        match name.as_str() {
            "example_prompt" => {
                let message = arguments
                    .and_then(|json| json.get("message")?.as_str().map(|s| s.to_string()))
                    .ok_or_else(|| {
                        McpError::invalid_params("No message provided to example_prompt", None)
                    })?;
                let prompt =
                    format!("This is an example prompt with your message here: '{message}'");
                Ok(GetPromptResult {
                    description: None,
                    messages: vec![PromptMessage {
                        role: PromptMessageRole::User,
                        content: PromptMessageContent::text(prompt),
                    }],
                })
            }
            _ => Err(McpError::invalid_params("prompt not found", None)),
        }
    }
    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        Ok(ListResourceTemplatesResult {
            next_cursor: None,
            resource_templates: Vec::new(),
        })
    }
    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        if let Some(http_request_part) = context.extensions.get::<axum::http::request::Parts>() {
            let initialize_headers = &http_request_part.headers;
            let initialize_uri = &http_request_part.uri;
            tracing::info!(?initialize_headers, %initialize_uri, "initialize from http server");
        }
        Ok(self.get_info())
    }
}
</file>

<file path="examples/servers/src/common/generic_service.rs">
use std::sync::Arc;
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
};
#[allow(dead_code)]
pub trait DataService: Send + Sync + 'static {
    fn get_data(&self) -> String;
    fn set_data(&mut self, data: String);
}
#[derive(Debug, Clone)]
pub struct MemoryDataService {
    data: String,
}
impl MemoryDataService {
    #[allow(dead_code)]
    pub fn new(initial_data: impl Into<String>) -> Self {
        Self {
            data: initial_data.into(),
        }
    }
}
impl DataService for MemoryDataService {
    fn get_data(&self) -> String {
        self.data.clone()
    }
    fn set_data(&mut self, data: String) {
        self.data = data;
    }
}
#[derive(Debug, Clone)]
pub struct GenericService<DS: DataService> {
    #[allow(dead_code)]
    data_service: Arc<DS>,
    tool_router: ToolRouter<Self>,
}
#[derive(Debug, schemars::JsonSchema, serde::Deserialize, serde::Serialize)]
pub struct SetDataRequest {
    pub data: String,
}
#[tool_router]
impl<DS: DataService> GenericService<DS> {
    #[allow(dead_code)]
    pub fn new(data_service: DS) -> Self {
        Self {
            data_service: Arc::new(data_service),
            tool_router: Self::tool_router(),
        }
    }
    #[tool(description = "get memory from service")]
    pub async fn get_data(&self) -> String {
        self.data_service.get_data()
    }
    #[tool(description = "set memory to service")]
    pub async fn set_data(
        &self,
        Parameters(SetDataRequest { data }): Parameters<SetDataRequest>,
    ) -> String {
        let new_data = data.clone();
        format!("Current memory: {}", new_data)
    }
}
#[tool_handler]
impl<DS: DataService> ServerHandler for GenericService<DS> {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("generic data service".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
</file>

<file path="examples/servers/src/common/mod.rs">
pub mod calculator;
pub mod counter;
pub mod generic_service;
</file>

<file path="examples/servers/src/complex_auth_sse.rs">
use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};
use anyhow::Result;
use askama::Template;
use axum::{
    Json, Router,
    body::Body,
    extract::{Form, Query, State},
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use rand::{Rng, distr::Alphanumeric};
use rmcp::transport::{
    SseServer,
    auth::{
        AuthorizationMetadata, ClientRegistrationRequest, ClientRegistrationResponse,
        OAuthClientConfig,
    },
    sse_server::SseServerConfig,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tower_http::cors::{Any, CorsLayer};
use tracing::{debug, error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;
mod common;
use common::counter::Counter;
const BIND_ADDRESS: &str = "127.0.0.1:3000";
const INDEX_HTML: &str = include_str!("html/mcp_oauth_index.html");
#[derive(Clone, Debug)]
struct McpOAuthStore {
    clients: Arc<RwLock<HashMap<String, OAuthClientConfig>>>,
    auth_sessions: Arc<RwLock<HashMap<String, AuthSession>>>,
    access_tokens: Arc<RwLock<HashMap<String, McpAccessToken>>>,
}
impl McpOAuthStore {
    fn new() -> Self {
        let mut clients = HashMap::new();
        clients.insert(
            "mcp-client".to_string(),
            OAuthClientConfig {
                client_id: "mcp-client".to_string(),
                client_secret: Some("mcp-client-secret".to_string()),
                scopes: vec!["profile".to_string(), "email".to_string()],
                redirect_uri: "http://localhost:8080/callback".to_string(),
            },
        );
        Self {
            clients: Arc::new(RwLock::new(clients)),
            auth_sessions: Arc::new(RwLock::new(HashMap::new())),
            access_tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    async fn validate_client(
        &self,
        client_id: &str,
        redirect_uri: &str,
    ) -> Option<OAuthClientConfig> {
        let clients = self.clients.read().await;
        if let Some(client) = clients.get(client_id) {
            if client.redirect_uri.contains(&redirect_uri.to_string()) {
                return Some(client.clone());
            }
        }
        None
    }
    async fn create_auth_session(
        &self,
        client_id: String,
        scope: Option<String>,
        state: Option<String>,
        session_id: String,
    ) -> String {
        let session = AuthSession {
            client_id,
            scope,
            _state: state,
            _created_at: chrono::Utc::now(),
            auth_token: None,
        };
        self.auth_sessions
            .write()
            .await
            .insert(session_id.clone(), session);
        session_id
    }
    async fn update_auth_session_token(
        &self,
        session_id: &str,
        token: AuthToken,
    ) -> Result<(), String> {
        let mut sessions = self.auth_sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.auth_token = Some(token);
            Ok(())
        } else {
            Err("Session not found".to_string())
        }
    }
    async fn create_mcp_token(&self, session_id: &str) -> Result<McpAccessToken, String> {
        let sessions = self.auth_sessions.read().await;
        if let Some(session) = sessions.get(session_id) {
            if let Some(auth_token) = &session.auth_token {
                let access_token = format!("mcp-token-{}", Uuid::new_v4());
                let token = McpAccessToken {
                    access_token: access_token.clone(),
                    token_type: "Bearer".to_string(),
                    expires_in: 3600,
                    refresh_token: format!("mcp-refresh-{}", Uuid::new_v4()),
                    scope: session.scope.clone(),
                    auth_token: auth_token.clone(),
                    client_id: session.client_id.clone(),
                };
                self.access_tokens
                    .write()
                    .await
                    .insert(access_token.clone(), token.clone());
                Ok(token)
            } else {
                Err("No third-party token available for session".to_string())
            }
        } else {
            Err("Session not found".to_string())
        }
    }
    async fn validate_token(&self, token: &str) -> Option<McpAccessToken> {
        self.access_tokens.read().await.get(token).cloned()
    }
}
#[derive(Clone, Debug)]
struct AuthSession {
    client_id: String,
    scope: Option<String>,
    _state: Option<String>,
    _created_at: chrono::DateTime<chrono::Utc>,
    auth_token: Option<AuthToken>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct AuthToken {
    access_token: String,
    token_type: String,
    expires_in: u64,
    refresh_token: String,
    scope: Option<String>,
}
#[derive(Clone, Debug, Serialize)]
struct McpAccessToken {
    access_token: String,
    token_type: String,
    expires_in: u64,
    refresh_token: String,
    scope: Option<String>,
    auth_token: AuthToken,
    client_id: String,
}
#[derive(Debug, Deserialize)]
struct AuthorizeQuery {
    #[allow(dead_code)]
    response_type: String,
    client_id: String,
    redirect_uri: String,
    scope: Option<String>,
    state: Option<String>,
}
#[derive(Debug, Deserialize, Serialize)]
struct TokenRequest {
    grant_type: String,
    #[serde(default)]
    code: String,
    #[serde(default)]
    client_id: String,
    #[serde(default)]
    client_secret: String,
    #[serde(default)]
    redirect_uri: String,
    #[serde(default)]
    code_verifier: Option<String>,
    #[serde(default)]
    refresh_token: String,
}
#[derive(Debug, Deserialize, Serialize)]
struct UserInfo {
    sub: String,
    name: String,
    email: String,
    username: String,
}
fn generate_random_string(length: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}
async fn index() -> Html<&'static str> {
    Html(INDEX_HTML)
}
#[derive(Template)]
#[template(path = "mcp_oauth_authorize.html")]
struct OAuthAuthorizeTemplate {
    client_id: String,
    redirect_uri: String,
    scope: String,
    state: String,
    scopes: String,
}
// Initial OAuth authorize endpoint
async fn oauth_authorize(
    Query(params): Query<AuthorizeQuery>,
    State(state): State<Arc<McpOAuthStore>>,
) -> impl IntoResponse {
    debug!("doing oauth_authorize");
    if let Some(_client) = state
        .validate_client(&params.client_id, &params.redirect_uri)
        .await
    {
        let template = OAuthAuthorizeTemplate {
            client_id: params.client_id,
            redirect_uri: params.redirect_uri,
            scope: params.scope.clone().unwrap_or_default(),
            state: params.state.clone().unwrap_or_default(),
            scopes: params
                .scope
                .clone()
                .unwrap_or_else(|| "Basic scope".to_string()),
        };
        Html(template.render().unwrap()).into_response()
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "invalid_request",
                "error_description": "invalid client id or redirect uri"
            })),
        )
            .into_response()
    }
}
// handle approval of authorization
#[derive(Debug, Deserialize)]
struct ApprovalForm {
    client_id: String,
    redirect_uri: String,
    scope: String,
    state: String,
    approved: String,
}
async fn oauth_approve(
    State(state): State<Arc<McpOAuthStore>>,
    Form(form): Form<ApprovalForm>,
) -> impl IntoResponse {
    if form.approved != "true" {
        // user rejected the authorization request
        let redirect_url = format!(
            "{}?error=access_denied&error_description={}{}",
            form.redirect_uri,
            "user rejected the authorization request",
            if form.state.is_empty() {
                "".to_string()
            } else {
                format!("&state={}", form.state)
            }
        );
        return Redirect::to(&redirect_url).into_response();
    }
    // user approved the authorization request, generate authorization code
    let session_id = Uuid::new_v4().to_string();
    let auth_code = format!("mcp-code-{}", session_id);
    // create new session record authorization information
    let session_id = state
        .create_auth_session(
            form.client_id.clone(),
            Some(form.scope.clone()),
            Some(form.state.clone()),
            session_id.clone(),
        )
        .await;
    // create token
    let created_token = AuthToken {
        access_token: format!("tp-token-{}", Uuid::new_v4()),
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        refresh_token: format!("tp-refresh-{}", Uuid::new_v4()),
        scope: Some(form.scope),
    };
    // update session token
    if let Err(e) = state
        .update_auth_session_token(&session_id, created_token)
        .await
    {
        error!("Failed to update session token: {}", e);
    }
    // redirect back to client, with authorization code
    let redirect_url = format!(
        "{}?code={}{}",
        form.redirect_uri,
        auth_code,
        if form.state.is_empty() {
            "".to_string()
        } else {
            format!("&state={}", form.state)
        }
    );
    info!("authorization approved, redirecting to: {}", redirect_url);
    Redirect::to(&redirect_url).into_response()
}
// Handle token request from the MCP client
async fn oauth_token(
    State(state): State<Arc<McpOAuthStore>>,
    request: axum::http::Request<Body>,
) -> impl IntoResponse {
    info!("Received token request");
    let bytes = match axum::body::to_bytes(request.into_body(), usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("can't read request body: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "invalid_request",
                    "error_description": "can't read request body"
                })),
            )
                .into_response();
        }
    };
    let body_str = String::from_utf8_lossy(&bytes);
    info!("request body: {}", body_str);
    let token_req = match serde_urlencoded::from_bytes::<TokenRequest>(&bytes) {
        Ok(form) => {
            info!("successfully parsed form data: {:?}", form);
            form
        }
        Err(e) => {
            error!("can't parse form data: {}", e);
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({
                    "error": "invalid_request",
                    "error_description": format!("can't parse form data: {}", e)
                })),
            )
                .into_response();
        }
    };
    if token_req.grant_type == "refresh_token" {
        warn!("this easy server only support authorization_code now");
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "unsupported_grant_type",
                "error_description": "only authorization_code is supported"
            })),
        )
            .into_response();
    }
    if token_req.grant_type != "authorization_code" {
        info!("unsupported grant type: {}", token_req.grant_type);
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "unsupported_grant_type",
                "error_description": "only authorization_code is supported"
            })),
        )
            .into_response();
    }
    if !token_req.code.starts_with("mcp-code-") {
        info!("invalid authorization code: {}", token_req.code);
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "invalid_grant",
                "error_description": "invalid authorization code"
            })),
        )
            .into_response();
    }
    let client_id = if token_req.client_id.is_empty() {
        "mcp-client".to_string()
    } else {
        token_req.client_id.clone()
    };
    match state
        .validate_client(&client_id, &token_req.redirect_uri)
        .await
    {
        Some(_) => {
            let session_id = token_req.code.replace("mcp-code-", "");
            info!("got session id: {}", session_id);
            // create mcp access token
            match state.create_mcp_token(&session_id).await {
                Ok(token) => {
                    info!("successfully created access token");
                    (
                        StatusCode::OK,
                        Json(serde_json::json!({
                            "access_token": token.access_token,
                            "token_type": token.token_type,
                            "expires_in": token.expires_in,
                            "refresh_token": token.refresh_token,
                            "scope": token.scope,
                        })),
                    )
                        .into_response()
                }
                Err(e) => {
                    error!("failed to create access token: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "error": "server_error",
                            "error_description": format!("failed to create access token: {}", e)
                        })),
                    )
                        .into_response()
                }
            }
        }
        None => {
            info!(
                "invalid client id or redirect uri: {} / {}",
                client_id, token_req.redirect_uri
            );
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "invalid_client",
                    "error_description": "invalid client id or redirect uri"
                })),
            )
                .into_response()
        }
    }
}
async fn validate_token_middleware(
    State(token_store): State<Arc<McpOAuthStore>>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    debug!("validate_token_middleware");
    let auth_header = request.headers().get("Authorization");
    let token = match auth_header {
        Some(header) => {
            let header_str = header.to_str().unwrap_or("");
            if let Some(stripped) = header_str.strip_prefix("Bearer ") {
                stripped.to_string()
            } else {
                return StatusCode::UNAUTHORIZED.into_response();
            }
        }
        None => {
            return StatusCode::UNAUTHORIZED.into_response();
        }
    };
    // Validate the token
    match token_store.validate_token(&token).await {
        Some(_) => next.run(request).await,
        None => StatusCode::UNAUTHORIZED.into_response(),
    }
}
// handle oauth server metadata request
async fn oauth_authorization_server() -> impl IntoResponse {
    let mut additional_fields = HashMap::new();
    additional_fields.insert(
        "response_types_supported".into(),
        Value::Array(vec![Value::String("code".into())]),
    );
    additional_fields.insert(
        "code_challenge_methods_supported".into(),
        Value::Array(vec![Value::String("S256".into())]),
    );
    let metadata = AuthorizationMetadata {
        authorization_endpoint: format!("http://{}/oauth/authorize", BIND_ADDRESS),
        token_endpoint: format!("http://{}/oauth/token", BIND_ADDRESS),
        scopes_supported: Some(vec!["profile".to_string(), "email".to_string()]),
        registration_endpoint: format!("http://{}/oauth/register", BIND_ADDRESS),
        issuer: Some(BIND_ADDRESS.to_string()),
        jwks_uri: Some(format!("http://{}/oauth/jwks", BIND_ADDRESS)),
        additional_fields,
    };
    debug!("metadata: {:?}", metadata);
    (StatusCode::OK, Json(metadata))
}
async fn oauth_register(
    State(state): State<Arc<McpOAuthStore>>,
    Json(req): Json<ClientRegistrationRequest>,
) -> impl IntoResponse {
    debug!("register request: {:?}", req);
    if req.redirect_uris.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "invalid_request",
                "error_description": "at least one redirect uri is required"
            })),
        )
            .into_response();
    }
    let client_id = format!("client-{}", Uuid::new_v4());
    let client_secret = generate_random_string(32);
    let client = OAuthClientConfig {
        client_id: client_id.clone(),
        client_secret: Some(client_secret.clone()),
        redirect_uri: req.redirect_uris[0].clone(),
        scopes: vec![],
    };
    state
        .clients
        .write()
        .await
        .insert(client_id.clone(), client);
    let response = ClientRegistrationResponse {
        client_id,
        client_secret: Some(client_secret),
        client_name: req.client_name,
        redirect_uris: req.redirect_uris,
        additional_fields: HashMap::new(),
    };
    (StatusCode::CREATED, Json(response)).into_response()
}
async fn log_request(request: Request<Body>, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let version = request.version();
    let headers = request.headers().clone();
    let mut header_log = String::new();
    for (key, value) in headers.iter() {
        let value_str = value.to_str().unwrap_or("<binary>");
        header_log.push_str(&format!("\n  {}: {}", key, value_str));
    }
    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let request_info = if content_type.contains("application/x-www-form-urlencoded")
        || content_type.contains("application/json")
    {
        format!(
            "{} {} {:?}{}\nContent-Type: {}",
            method, uri, version, header_log, content_type
        )
    } else {
        format!("{} {} {:?}{}", method, uri, version, header_log)
    };
    info!("REQUEST: {}", request_info);
    let response = next.run(request).await;
    let status = response.status();
    info!("RESPONSE: {} for {} {}", status, method, uri);
    response
}
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let oauth_store = Arc::new(McpOAuthStore::new());
    let addr = BIND_ADDRESS.parse::<SocketAddr>()?;
    let sse_config = SseServerConfig {
        bind: addr,
        sse_path: "/mcp/sse".to_string(),
        post_path: "/mcp/message".to_string(),
        ct: CancellationToken::new(),
        sse_keep_alive: Some(Duration::from_secs(15)),
    };
    let (sse_server, sse_router) = SseServer::new(sse_config);
    let protected_sse_router = sse_router.layer(middleware::from_fn_with_state(
        oauth_store.clone(),
        validate_token_middleware,
    ));
    let cors_layer = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    let oauth_server_router = Router::new()
        .route(
            "/.well-known/oauth-authorization-server",
            get(oauth_authorization_server).options(oauth_authorization_server),
        )
        .route("/oauth/token", post(oauth_token).options(oauth_token))
        .route(
            "/oauth/register",
            post(oauth_register).options(oauth_register),
        )
        .layer(cors_layer)
        .with_state(oauth_store.clone());
    let app = Router::new()
        .route("/", get(index))
        .route("/mcp", get(index))
        .route("/oauth/authorize", get(oauth_authorize))
        .route("/oauth/approve", post(oauth_approve))
        .merge(oauth_server_router)
        .with_state(oauth_store.clone())
        .layer(middleware::from_fn(log_request));
    let app = app.merge(protected_sse_router);
    let cancel_token = sse_server.config.ct.clone();
    let cancel_token2 = sse_server.config.ct.clone();
    sse_server.with_service(Counter::new);
    info!("MCP OAuth Server started on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let server = axum::serve(listener, app).with_graceful_shutdown(async move {
        cancel_token.cancelled().await;
        info!("Server is shutting down");
    });
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                info!("Received Ctrl+C, shutting down");
                cancel_token2.cancel();
            }
            Err(e) => error!("Failed to listen for Ctrl+C: {}", e),
        }
    });
    if let Err(e) = server.await {
        error!("Server error: {}", e);
    }
    Ok(())
}
</file>

<file path="examples/servers/src/counter_hyper_streamable_http.rs">
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
</file>

<file path="examples/servers/src/counter_sse_directly.rs">
use rmcp::transport::sse_server::SseServer;
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    {self},
};
mod common;
use common::counter::Counter;
const BIND_ADDRESS: &str = "127.0.0.1:8000";
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let ct = SseServer::serve(BIND_ADDRESS.parse()?)
        .await?
        .with_service_directly(Counter::new);
    tokio::signal::ctrl_c().await?;
    ct.cancel();
    Ok(())
}
</file>

<file path="examples/servers/src/counter_sse.rs">
use rmcp::transport::sse_server::{SseServer, SseServerConfig};
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    {self},
};
mod common;
use common::counter::Counter;
const BIND_ADDRESS: &str = "127.0.0.1:8000";
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let config = SseServerConfig {
        bind: BIND_ADDRESS.parse()?,
        sse_path: "/sse".to_string(),
        post_path: "/message".to_string(),
        ct: tokio_util::sync::CancellationToken::new(),
        sse_keep_alive: None,
    };
    let (sse_server, router) = SseServer::new(config);
    let listener = tokio::net::TcpListener::bind(sse_server.config.bind).await?;
    let ct = sse_server.config.ct.child_token();
    let server = axum::serve(listener, router).with_graceful_shutdown(async move {
        ct.cancelled().await;
        tracing::info!("sse server cancelled");
    });
    tokio::spawn(async move {
        if let Err(e) = server.await {
            tracing::error!(error = %e, "sse server shutdown with error");
        }
    });
    let ct = sse_server.with_service(Counter::new);
    tokio::signal::ctrl_c().await?;
    ct.cancel();
    Ok(())
}
</file>

<file path="examples/servers/src/counter_stdio.rs">
use anyhow::Result;
use common::counter::Counter;
use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::{self, EnvFilter};
mod common;
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();
    tracing::info!("Starting MCP server");
    let service = Counter::new().serve(stdio()).await.inspect_err(|e| {
        tracing::error!("serving error: {:?}", e);
    })?;
    service.waiting().await?;
    Ok(())
}
</file>

<file path="examples/servers/src/counter_streamhttp.rs">
use rmcp::transport::streamable_http_server::{
    StreamableHttpService, session::local::LocalSessionManager,
};
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    {self},
};
mod common;
use common::counter::Counter;
const BIND_ADDRESS: &str = "127.0.0.1:8000";
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let service = StreamableHttpService::new(
        || Ok(Counter::new()),
        LocalSessionManager::default().into(),
        Default::default(),
    );
    let router = axum::Router::new().nest_service("/mcp", service);
    let tcp_listener = tokio::net::TcpListener::bind(BIND_ADDRESS).await?;
    let _ = axum::serve(tcp_listener, router)
        .with_graceful_shutdown(async { tokio::signal::ctrl_c().await.unwrap() })
        .await;
    Ok(())
}
</file>

<file path="examples/servers/src/memory_stdio.rs">
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
</file>

<file path="examples/servers/src/sampling_stdio.rs">
use std::sync::Arc;
use anyhow::Result;
use rmcp::{
    ServerHandler, ServiceExt,
    model::*,
    service::{RequestContext, RoleServer},
    transport::stdio,
};
use tracing_subscriber::{self, EnvFilter};
#[derive(Clone, Debug, Default)]
pub struct SamplingDemoServer;
impl ServerHandler for SamplingDemoServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(concat!(
                "This is a demo server that requests sampling from clients. It provides tools that use LLM capabilities.\n\n",
                "IMPORTANT: This server requires a client that supports the 'sampling/createMessage' method. ",
                "Without sampling support, the tools will return errors."
            ).into()),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            ..Default::default()
        }
    }
    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        match request.name.as_ref() {
            "ask_llm" => {
                let question = request
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("question"))
                    .and_then(|q| q.as_str())
                    .unwrap_or("Hello LLM");
                let response = context
                    .peer
                    .create_message(CreateMessageRequestParam {
                        messages: vec![SamplingMessage {
                            role: Role::User,
                            content: Content::text(question),
                        }],
                        model_preferences: Some(ModelPreferences {
                            hints: Some(vec![ModelHint {
                                name: Some("claude".to_string()),
                            }]),
                            cost_priority: Some(0.3),
                            speed_priority: Some(0.8),
                            intelligence_priority: Some(0.7),
                        }),
                        system_prompt: Some("You are a helpful assistant.".to_string()),
                        include_context: Some(ContextInclusion::None),
                        temperature: Some(0.7),
                        max_tokens: 150,
                        stop_sequences: None,
                        metadata: None,
                    })
                    .await
                    .map_err(|e| {
                        ErrorData::new(
                            ErrorCode::INTERNAL_ERROR,
                            format!("Sampling request failed: {}", e),
                            None,
                        )
                    })?;
                tracing::debug!("Response: {:?}", response);
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Question: {}\nAnswer: {}",
                    question,
                    response
                        .message
                        .content
                        .as_text()
                        .map(|t| &t.text)
                        .unwrap_or(&"No text response".to_string())
                ))]))
            }
            _ => Err(ErrorData::new(
                ErrorCode::INTERNAL_ERROR,
                format!("Unknown tool: {}", request.name),
                None,
            )),
        }
    }
    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult {
            tools: vec![Tool {
                name: "ask_llm".into(),
                description: Some("Ask a question to the LLM through sampling".into()),
                input_schema: Arc::new(
                    serde_json::from_value(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "question": {
                                "type": "string",
                                "description": "The question to ask the LLM"
                            }
                        },
                        "required": ["question"]
                    }))
                    .unwrap(),
                ),
                output_schema: None,
                annotations: None,
            }],
            next_cursor: None,
        })
    }
}
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();
    tracing::info!("Starting Sampling Demo Server");
    let service = SamplingDemoServer.serve(stdio()).await.inspect_err(|e| {
        tracing::error!("Serving error: {:?}", e);
    })?;
    service.waiting().await?;
    Ok(())
}
</file>

<file path="examples/servers/src/simple_auth_sse.rs">
use std::{net::SocketAddr, sync::Arc, time::Duration};
use anyhow::Result;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, Request, StatusCode},
    middleware::{self, Next},
    response::{Html, Response},
    routing::get,
};
use rmcp::transport::{SseServer, sse_server::SseServerConfig};
use tokio_util::sync::CancellationToken;
mod common;
use common::counter::Counter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
const BIND_ADDRESS: &str = "127.0.0.1:8000";
const INDEX_HTML: &str = include_str!("html/sse_auth_index.html");
struct TokenStore {
    valid_tokens: Vec<String>,
}
impl TokenStore {
    fn new() -> Self {
        Self {
            valid_tokens: vec!["demo-token".to_string(), "test-token".to_string()],
        }
    }
    fn is_valid(&self, token: &str) -> bool {
        self.valid_tokens.contains(&token.to_string())
    }
}
fn extract_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|auth_header| {
            auth_header
                .strip_prefix("Bearer ")
                .map(|stripped| stripped.to_string())
        })
}
async fn auth_middleware(
    State(token_store): State<Arc<TokenStore>>,
    headers: HeaderMap,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    match extract_token(&headers) {
        Some(token) if token_store.is_valid(&token) => {
            Ok(next.run(request).await)
        }
        _ => {
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}
async fn index() -> Html<&'static str> {
    Html(INDEX_HTML)
}
// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}
async fn get_token(Path(token_id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    if token_id == "demo" || token_id == "test" {
        let token = format!("{}-token", token_id);
        Ok(Json(serde_json::json!({
            "access_token": token,
            "token_type": "Bearer",
            "expires_in": 3600
        })))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let token_store = Arc::new(TokenStore::new());
    let addr = BIND_ADDRESS.parse::<SocketAddr>()?;
    let sse_config = SseServerConfig {
        bind: addr,
        sse_path: "/sse".to_string(),
        post_path: "/message".to_string(),
        ct: CancellationToken::new(),
        sse_keep_alive: Some(Duration::from_secs(15)),
    };
    let (sse_server, sse_router) = SseServer::new(sse_config);
    let api_routes = Router::new()
        .route("/health", get(health_check))
        .route("/token/{token_id}", get(get_token));
    let protected_sse_router = sse_router.layer(middleware::from_fn_with_state(
        token_store.clone(),
        auth_middleware,
    ));
    let app = Router::new()
        .route("/", get(index))
        .nest("/api", api_routes)
        .merge(protected_sse_router)
        .with_state(());
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let ct = sse_server.config.ct.clone();
    sse_server.with_service(Counter::new);
    let cancel_token = ct.clone();
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                println!("Received Ctrl+C, shutting down server...");
                cancel_token.cancel();
            }
            Err(err) => {
                eprintln!("Unable to listen for Ctrl+C signal: {}", err);
            }
        }
    });
    tracing::info!("Server started on {}", addr);
    let server = axum::serve(listener, app).with_graceful_shutdown(async move {
        ct.cancelled().await;
        println!("Server is shutting down...");
    });
    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
    println!("Server has been shut down");
    Ok(())
}
</file>

<file path="examples/servers/src/structured_output.rs">
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
    #[tool(name = "get_weather", description = "Get current weather for a city")]
    pub async fn get_weather(
        &self,
        params: Parameters<WeatherRequest>,
    ) -> Result<Json<WeatherResponse>, String> {
        let weather = WeatherResponse {
            temperature: match params.0.units.as_deref() {
                Some("fahrenheit") => 72.5,
                _ => 22.5,
            },
            description: "Partly cloudy".to_string(),
            humidity: 65,
            wind_speed: 12.5,
        };
        Ok(Json(weather))
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
    eprintln!("Starting server. Connect with an MCP client to test the tools.");
    eprintln!("Press Ctrl+C to stop.");
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
</file>

<file path="examples/simple-chat-client/src/bin/simple_chat.rs">
use std::{process::exit, sync::Arc};
use anyhow::Result;
use clap::{Parser, Subcommand};
use simple_chat_client::{
    chat::ChatSession,
    client::OpenAIClient,
    config::Config,
    tool::{Tool, ToolSet, get_mcp_tools},
};
#[derive(Parser)]
#[command(author, version, about = "Simple Chat Client")]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    Config,
    Chat {
        #[arg(short, long)]
        model: Option<String>,
    },
}
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Config => {
            println!("{}", include_str!("../config.toml"));
            return Ok(());
        }
        Commands::Chat { model } => {
            let config_path = cli.config;
            let mut config = match config_path {
                Some(path) => Config::load(&path).await?,
                None => {
                    println!("No config file provided, using default config");
                    exit(-1);
                }
            };
            if let Some(model_name) = model {
                config.model_name = Some(model_name);
            }
            let api_key = config
                .openai_key
                .clone()
                .unwrap_or_else(|| std::env::var("OPENAI_API_KEY").expect("need set api key"));
            let url = config.chat_url.clone();
            println!("use api address: {:?}", url);
            let openai_client = Arc::new(OpenAIClient::new(api_key, url, config.proxy));
            let mut tool_set = ToolSet::default();
            if config.mcp.is_some() {
                let mcp_clients = config.create_mcp_clients().await?;
                for (name, client) in mcp_clients.iter() {
                    println!("load MCP tool: {}", name);
                    let server = client.peer().clone();
                    let tools = get_mcp_tools(server).await?;
                    for tool in tools {
                        println!("add tool: {}", tool.name());
                        tool_set.add_tool(tool);
                    }
                }
                tool_set.set_clients(mcp_clients);
            }
            let mut session = ChatSession::new(
                openai_client,
                tool_set,
                config
                    .model_name
                    .unwrap_or_else(|| "gpt-4o-mini".to_string()),
            );
            let support_tool = config.support_tool.unwrap_or(true);
            let mut system_prompt;
            if !support_tool {
                system_prompt =
            "you are a assistant, you can help user to complete various tasks. you have the following tools to use:\n".to_string();
                for tool in session.get_tools() {
                    system_prompt.push_str(&format!(
                        "\ntool name: {}\ndescription: {}\nparameters: {}\n",
                        tool.name(),
                        tool.description(),
                        serde_json::to_string_pretty(&tool.parameters())
                            .expect("failed to serialize tool parameters")
                    ));
                }
                system_prompt.push_str(
                    "\nif you need to call tool, please use the following format:\n\
            Tool: <tool name>\n\
            Inputs: <inputs>\n",
                );
                println!("system prompt: {}", system_prompt);
            } else {
                system_prompt =
                    "you are a assistant, you can help user to complete various tasks.".to_string();
            }
            session.add_system_prompt(system_prompt);
            session.chat(support_tool).await?;
        }
    }
    Ok(())
}
</file>

<file path="examples/simple-chat-client/src/chat.rs">
use std::{
    io::{self, Write},
    sync::Arc,
};
use anyhow::Result;
use serde_json;
use crate::{
    client::ChatClient,
    model::{CompletionRequest, Message, ToolFunction},
    tool::{Tool as ToolTrait, ToolSet},
};
pub struct ChatSession {
    client: Arc<dyn ChatClient>,
    tool_set: ToolSet,
    model: String,
    messages: Vec<Message>,
}
impl ChatSession {
    pub fn new(client: Arc<dyn ChatClient>, tool_set: ToolSet, model: String) -> Self {
        Self {
            client,
            tool_set,
            model,
            messages: Vec::new(),
        }
    }
    pub fn add_system_prompt(&mut self, prompt: impl ToString) {
        self.messages.push(Message::system(prompt));
    }
    pub fn get_tools(&self) -> Vec<Arc<dyn ToolTrait>> {
        self.tool_set.tools()
    }
    pub async fn analyze_tool_call(&mut self, response: &Message) {
        let mut tool_calls_func = Vec::new();
        if let Some(tool_calls) = response.tool_calls.as_ref() {
            for tool_call in tool_calls {
                if tool_call._type == "function" {
                    tool_calls_func.push(tool_call.function.clone());
                }
            }
        } else {
            if response.content.contains("Tool:") {
                let lines: Vec<&str> = response.content.split('\n').collect();
                let mut tool_name = None;
                let mut args_text = Vec::new();
                let mut parsing_args = false;
                for line in lines {
                    if line.starts_with("Tool:") {
                        tool_name = line.strip_prefix("Tool:").map(|s| s.trim().to_string());
                        parsing_args = false;
                    } else if line.starts_with("Inputs:") {
                        parsing_args = true;
                    } else if parsing_args {
                        args_text.push(line.trim());
                    }
                }
                if let Some(name) = tool_name {
                    tool_calls_func.push(ToolFunction {
                        name,
                        arguments: args_text.join("\n"),
                    });
                }
            }
        }
        for tool_call in tool_calls_func {
            println!("tool call: {:?}", tool_call);
            let tool = self.tool_set.get_tool(&tool_call.name);
            if let Some(tool) = tool {
                let args = serde_json::from_str::<serde_json::Value>(&tool_call.arguments)
                    .unwrap_or_default();
                match tool.call(args).await {
                    Ok(result) => {
                        if result.is_error.is_some_and(|b| b) {
                            self.messages
                                .push(Message::user("tool call failed, mcp call error"));
                        } else if let Some(contents) = &result.content {
                            contents.iter().for_each(|content| {
                                if let Some(content_text) = content.as_text() {
                                    let json_result = serde_json::from_str::<serde_json::Value>(
                                        &content_text.text,
                                    )
                                    .unwrap_or_default();
                                    let pretty_result =
                                        serde_json::to_string_pretty(&json_result).unwrap();
                                    println!("call tool result: {}", pretty_result);
                                    self.messages.push(Message::user(format!(
                                        "call tool result: {}",
                                        pretty_result
                                    )));
                                }
                            });
                        }
                    }
                    Err(e) => {
                        println!("tool call failed: {}", e);
                        self.messages
                            .push(Message::user(format!("tool call failed: {}", e)));
                    }
                }
            } else {
                println!("tool not found: {}", tool_call.name);
            }
        }
    }
    pub async fn chat(&mut self, support_tool: bool) -> Result<()> {
        println!("welcome to use simple chat client, use 'exit' to quit");
        loop {
            print!("> ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            input = input.trim().to_string();
            if input.is_empty() {
                continue;
            }
            if input == "exit" {
                break;
            }
            self.messages.push(Message::user(&input));
            let tool_definitions = if support_tool {
                let tools = self.tool_set.tools();
                if !tools.is_empty() {
                    Some(
                        tools
                            .iter()
                            .map(|tool| crate::model::Tool {
                                name: tool.name(),
                                description: tool.description(),
                                parameters: tool.parameters(),
                            })
                            .collect(),
                    )
                } else {
                    None
                }
            } else {
                None
            };
            let request = CompletionRequest {
                model: self.model.clone(),
                messages: self.messages.clone(),
                temperature: Some(0.7),
                tools: tool_definitions,
            };
            let response = self.client.complete(request).await?;
            let choice = response.choices.first().unwrap();
            println!("AI > {}", choice.message.content);
            self.analyze_tool_call(&choice.message).await;
        }
        Ok(())
    }
}
</file>

<file path="examples/simple-chat-client/src/client.rs">
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client as HttpClient;
use crate::model::{CompletionRequest, CompletionResponse};
#[async_trait]
pub trait ChatClient: Send + Sync {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;
}
pub struct OpenAIClient {
    api_key: String,
    client: HttpClient,
    base_url: String,
}
impl OpenAIClient {
    pub fn new(api_key: String, url: Option<String>, proxy: Option<bool>) -> Self {
        let base_url = url.unwrap_or("https://api.openai.com/v1/chat/completions".to_string());
        let proxy = proxy.unwrap_or(false);
        let client = if proxy {
            HttpClient::new()
        } else {
            HttpClient::builder()
                .no_proxy()
                .build()
                .unwrap_or_else(|_| HttpClient::new())
        };
        Self {
            api_key,
            client,
            base_url,
        }
    }
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }
}
#[async_trait]
impl ChatClient for OpenAIClient {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let response = self
            .client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;
        if !response.status().is_success() {
            let error_text = response.text().await?;
            println!("API error: {}", error_text);
            return Err(anyhow::anyhow!("API Error: {}", error_text));
        }
        let text_data = response.text().await?;
        println!("Received response: {}", text_data);
        let completion: CompletionResponse = serde_json::from_str(&text_data)
            .map_err(anyhow::Error::from)
            .unwrap();
        Ok(completion)
    }
}
</file>

<file path="examples/simple-chat-client/src/config.rs">
use std::{collections::HashMap, path::Path, process::Stdio};
use anyhow::Result;
use rmcp::{RoleClient, ServiceExt, service::RunningService, transport::ConfigureCommandExt};
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub openai_key: Option<String>,
    pub chat_url: Option<String>,
    pub mcp: Option<McpConfig>,
    pub model_name: Option<String>,
    pub proxy: Option<bool>,
    pub support_tool: Option<bool>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct McpConfig {
    pub server: Vec<McpServerConfig>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct McpServerConfig {
    pub name: String,
    #[serde(flatten)]
    pub transport: McpServerTransportConfig,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "protocol", rename_all = "lowercase")]
pub enum McpServerTransportConfig {
    Streamable {
        url: String,
    },
    Sse {
        url: String,
    },
    Stdio {
        command: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        envs: HashMap<String, String>,
    },
}
impl McpServerTransportConfig {
    pub async fn start(&self) -> Result<RunningService<RoleClient, ()>> {
        let client = match self {
            McpServerTransportConfig::Streamable { url } => {
                let transport =
                    rmcp::transport::StreamableHttpClientTransport::from_uri(url.to_string());
                ().serve(transport).await?
            }
            McpServerTransportConfig::Sse { url } => {
                let transport =
                    rmcp::transport::sse_client::SseClientTransport::start(url.to_owned()).await?;
                ().serve(transport).await?
            }
            McpServerTransportConfig::Stdio {
                command,
                args,
                envs,
            } => {
                let transport = rmcp::transport::child_process::TokioChildProcess::new(
                    tokio::process::Command::new(command).configure(|cmd| {
                        cmd.args(args)
                            .envs(envs)
                            .stderr(Stdio::inherit())
                            .stdout(Stdio::inherit());
                    }),
                )?;
                ().serve(transport).await?
            }
        };
        Ok(client)
    }
}
impl Config {
    pub async fn load(path: impl AsRef<Path>) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
    pub async fn create_mcp_clients(
        &self,
    ) -> Result<HashMap<String, RunningService<RoleClient, ()>>> {
        let mut clients = HashMap::new();
        if let Some(mcp_config) = &self.mcp {
            for server in &mcp_config.server {
                let client = server.transport.start().await?;
                clients.insert(server.name.clone(), client);
            }
        }
        Ok(clients)
    }
}
</file>

<file path="examples/simple-chat-client/src/error.rs">
use std::fmt;
use serde::Serialize;
#[derive(Debug, Serialize)]
pub struct McpError {
    pub message: String,
}
impl fmt::Display for McpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl std::error::Error for McpError {}
impl McpError {
    pub fn new(message: impl ToString) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}
</file>

<file path="examples/simple-chat-client/src/lib.rs">
pub mod chat;
pub mod client;
pub mod config;
pub mod error;
pub mod model;
pub mod tool;
</file>

<file path="examples/simple-chat-client/src/model.rs">
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}
impl Message {
    pub fn system(content: impl ToString) -> Self {
        Self {
            role: "system".to_string(),
            content: content.to_string(),
            tool_calls: None,
        }
    }
    pub fn user(content: impl ToString) -> Self {
        Self {
            role: "user".to_string(),
            content: content.to_string(),
            tool_calls: None,
        }
    }
    pub fn assistant(content: impl ToString) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.to_string(),
            tool_calls: None,
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: Message,
    pub finish_reason: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub function: ToolFunction,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolFunction {
    pub name: String,
    pub arguments: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub contents: Vec<Content>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    pub content_type: String,
    pub body: String,
}
impl Content {
    pub fn text(content: impl ToString) -> Self {
        Self {
            content_type: "text/plain".to_string(),
            body: content.to_string(),
        }
    }
}
</file>

<file path="examples/simple-chat-client/src/tool.rs">
use std::{collections::HashMap, sync::Arc};
use anyhow::Result;
use async_trait::async_trait;
use rmcp::{
    RoleClient,
    model::{CallToolRequestParam, CallToolResult, Tool as McpTool},
    service::{RunningService, ServerSink},
};
use serde_json::Value;
use crate::{
    error::McpError,
    model::{Content, ToolResult},
};
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> String;
    fn description(&self) -> String;
    fn parameters(&self) -> Value;
    async fn call(&self, args: Value) -> Result<CallToolResult>;
}
pub struct McpToolAdapter {
    tool: McpTool,
    server: ServerSink,
}
impl McpToolAdapter {
    pub fn new(tool: McpTool, server: ServerSink) -> Self {
        Self { tool, server }
    }
}
#[async_trait]
impl Tool for McpToolAdapter {
    fn name(&self) -> String {
        self.tool.name.clone().to_string()
    }
    fn description(&self) -> String {
        self.tool
            .description
            .clone()
            .unwrap_or_default()
            .to_string()
    }
    fn parameters(&self) -> Value {
        serde_json::to_value(&self.tool.input_schema).unwrap_or(serde_json::json!({}))
    }
    async fn call(&self, args: Value) -> Result<CallToolResult> {
        let arguments = match args {
            Value::Object(map) => Some(map),
            _ => None,
        };
        println!("arguments: {:?}", arguments);
        let call_result = self
            .server
            .call_tool(CallToolRequestParam {
                name: self.tool.name.clone(),
                arguments,
            })
            .await?;
        Ok(call_result)
    }
}
#[derive(Default)]
pub struct ToolSet {
    tools: HashMap<String, Arc<dyn Tool>>,
    clients: HashMap<String, RunningService<RoleClient, ()>>,
}
impl ToolSet {
    pub fn set_clients(&mut self, clients: HashMap<String, RunningService<RoleClient, ()>>) {
        self.clients = clients;
    }
    pub fn add_tool<T: Tool + 'static>(&mut self, tool: T) {
        self.tools.insert(tool.name(), Arc::new(tool));
    }
    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }
    pub fn tools(&self) -> Vec<Arc<dyn Tool>> {
        self.tools.values().cloned().collect()
    }
}
pub async fn get_mcp_tools(server: ServerSink) -> Result<Vec<McpToolAdapter>> {
    let tools = server.list_all_tools().await?;
    Ok(tools
        .into_iter()
        .map(|tool| McpToolAdapter::new(tool, server.clone()))
        .collect())
}
pub trait IntoCallToolResult {
    fn into_call_tool_result(self) -> Result<ToolResult, McpError>;
}
impl<T> IntoCallToolResult for Result<T, McpError>
where
    T: serde::Serialize,
{
    fn into_call_tool_result(self) -> Result<ToolResult, McpError> {
        match self {
            Ok(response) => {
                let content = Content {
                    content_type: "application/json".to_string(),
                    body: serde_json::to_string(&response).unwrap_or_default(),
                };
                Ok(ToolResult {
                    success: true,
                    contents: vec![content],
                })
            }
            Err(error) => {
                let content = Content {
                    content_type: "application/json".to_string(),
                    body: serde_json::to_string(&error).unwrap_or_default(),
                };
                Ok(ToolResult {
                    success: false,
                    contents: vec![content],
                })
            }
        }
    }
}
</file>

<file path="examples/transport/src/common/calculator.rs">
#![allow(dead_code)]
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::Parameters, wrapper::Json},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
};
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SumRequest {
    #[schemars(description = "the left hand side number")]
    pub a: i32,
    pub b: i32,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SubRequest {
    #[schemars(description = "the left hand side number")]
    pub a: i32,
    #[schemars(description = "the right hand side number")]
    pub b: i32,
}
#[derive(Debug, Clone)]
pub struct Calculator {
    tool_router: ToolRouter<Self>,
}
#[tool_router]
impl Calculator {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
    #[tool(description = "Calculate the sum of two numbers")]
    fn sum(&self, Parameters(SumRequest { a, b }): Parameters<SumRequest>) -> String {
        (a + b).to_string()
    }
    #[tool(description = "Calculate the difference of two numbers")]
    fn sub(&self, Parameters(SubRequest { a, b }): Parameters<SubRequest>) -> Json<i32> {
        Json(a - b)
    }
}
#[tool_handler]
impl ServerHandler for Calculator {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("A simple calculator".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
</file>

<file path="examples/transport/src/common/mod.rs">
pub mod calculator;
</file>

<file path="examples/transport/src/http_upgrade.rs">
use common::calculator::Calculator;
use hyper::{
    Request, StatusCode,
    body::Incoming,
    header::{HeaderValue, UPGRADE},
};
use hyper_util::rt::TokioIo;
use rmcp::{RoleClient, ServiceExt, service::RunningService};
use tracing_subscriber::EnvFilter;
mod common;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();
    start_server().await?;
    let client = http_client("127.0.0.1:8001").await?;
    let tools = client.list_all_tools().await?;
    client.cancel().await?;
    tracing::info!("{:#?}", tools);
    Ok(())
}
async fn http_server(req: Request<Incoming>) -> Result<hyper::Response<String>, hyper::Error> {
    tokio::spawn(async move {
        let upgraded = hyper::upgrade::on(req).await?;
        let service = Calculator::new().serve(TokioIo::new(upgraded)).await?;
        service.waiting().await?;
        anyhow::Result::<()>::Ok(())
    });
    let mut response = hyper::Response::new(String::new());
    *response.status_mut() = StatusCode::SWITCHING_PROTOCOLS;
    response
        .headers_mut()
        .insert(UPGRADE, HeaderValue::from_static("mcp"));
    Ok(response)
}
async fn http_client(uri: &str) -> anyhow::Result<RunningService<RoleClient, ()>> {
    let tcp_stream = tokio::net::TcpStream::connect(uri).await?;
    let (mut s, c) =
        hyper::client::conn::http1::handshake::<_, String>(TokioIo::new(tcp_stream)).await?;
    tokio::spawn(c.with_upgrades());
    let mut req = Request::new(String::new());
    req.headers_mut()
        .insert(UPGRADE, HeaderValue::from_static("mcp"));
    let response = s.send_request(req).await?;
    let upgraded = hyper::upgrade::on(response).await?;
    let client = ().serve(TokioIo::new(upgraded)).await?;
    Ok(client)
}
async fn start_server() -> anyhow::Result<()> {
    let tcp_listener = tokio::net::TcpListener::bind("127.0.0.1:8001").await?;
    let service = hyper::service::service_fn(http_server);
    tokio::spawn(async move {
        while let Ok((stream, addr)) = tcp_listener.accept().await {
            tracing::info!("accepted connection from: {}", addr);
            let conn = hyper::server::conn::http1::Builder::new()
                .serve_connection(TokioIo::new(stream), service)
                .with_upgrades();
            tokio::spawn(conn);
        }
    });
    Ok(())
}
</file>

<file path="examples/transport/src/named-pipe.rs">
mod common;
#[cfg(target_family = "windows")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use common::calculator::Calculator;
    use rmcp::{serve_client, serve_server};
    use tokio::net::windows::named_pipe::{ClientOptions, ServerOptions};
    const PIPE_NAME: &str = r"\\.\pipe\rmcp_example";
    async fn server(name: &str) -> anyhow::Result<()> {
        let mut server = ServerOptions::new()
            .first_pipe_instance(true)
            .create(name)?;
        while let Ok(_) = server.connect().await {
            let stream = server;
            server = ServerOptions::new().create(name)?;
            tokio::spawn(async move {
                match serve_server(Calculator, stream).await {
                    Ok(server) => {
                        println!("Server initialized successfully");
                        if let Err(e) = server.waiting().await {
                            println!("Error while server waiting: {}", e);
                        }
                    }
                    Err(e) => println!("Server initialization failed: {}", e),
                }
                anyhow::Ok(())
            });
        }
        Ok(())
    }
    async fn client() -> anyhow::Result<()> {
        println!("Client connecting to {}", PIPE_NAME);
        let stream = ClientOptions::new().open(PIPE_NAME)?;
        let client = serve_client((), stream).await?;
        println!("Client connected and initialized successfully");
        let tools = client.peer().list_tools(Default::default()).await?;
        println!("Available tools: {:?}", tools);
        if let Some(sum_tool) = tools.tools.iter().find(|t| t.name.contains("sum")) {
            println!("Calling sum tool: {}", sum_tool.name);
            let result = client
                .peer()
                .call_tool(rmcp::model::CallToolRequestParam {
                    name: sum_tool.name.clone(),
                    arguments: Some(rmcp::object!({
                        "a": 10,
                        "b": 20
                    })),
                })
                .await?;
            println!("Result: {:?}", result);
        }
        Ok(())
    }
    tokio::spawn(server(PIPE_NAME));
    let mut clients = vec![];
    for _ in 0..100 {
        clients.push(client());
    }
    for client in clients {
        client.await?;
    }
    Ok(())
}
#[cfg(not(target_family = "windows"))]
fn main() {
    println!("Unix socket example is not supported on this platform.");
}
</file>

<file path="examples/transport/src/tcp.rs">
use common::calculator::Calculator;
use rmcp::{serve_client, serve_server};
mod common;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tokio::spawn(server());
    client().await?;
    Ok(())
}
async fn server() -> anyhow::Result<()> {
    let tcp_listener = tokio::net::TcpListener::bind("127.0.0.1:8001").await?;
    while let Ok((stream, _)) = tcp_listener.accept().await {
        tokio::spawn(async move {
            let server = serve_server(Calculator::new(), stream).await?;
            server.waiting().await?;
            anyhow::Ok(())
        });
    }
    Ok(())
}
async fn client() -> anyhow::Result<()> {
    let stream = tokio::net::TcpSocket::new_v4()?
        .connect("127.0.0.1:8001".parse()?)
        .await?;
    let client = serve_client((), stream).await?;
    let tools = client.peer().list_tools(Default::default()).await?;
    println!("{:?}", tools);
    Ok(())
}
</file>

<file path="examples/transport/src/unix_socket.rs">
mod common;
#[cfg(target_family = "unix")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use std::fs;
    use common::calculator::Calculator;
    use rmcp::{serve_client, serve_server};
    use tokio::net::{UnixListener, UnixStream};
    const SOCKET_PATH: &str = "/tmp/rmcp_example.sock";
    async fn server(unix_listener: UnixListener) -> anyhow::Result<()> {
        while let Ok((stream, addr)) = unix_listener.accept().await {
            println!("Client connected: {:?}", addr);
            tokio::spawn(async move {
                match serve_server(Calculator::new(), stream).await {
                    Ok(server) => {
                        println!("Server initialized successfully");
                        if let Err(e) = server.waiting().await {
                            println!("Error while server waiting: {}", e);
                        }
                    }
                    Err(e) => println!("Server initialization failed: {}", e),
                }
                anyhow::Ok(())
            });
        }
        Ok(())
    }
    async fn client() -> anyhow::Result<()> {
        println!("Client connecting to {}", SOCKET_PATH);
        let stream = UnixStream::connect(SOCKET_PATH).await?;
        let client = serve_client((), stream).await?;
        println!("Client connected and initialized successfully");
        let tools = client.peer().list_tools(Default::default()).await?;
        println!("Available tools: {:?}", tools);
        if let Some(sum_tool) = tools.tools.iter().find(|t| t.name.contains("sum")) {
            println!("Calling sum tool: {}", sum_tool.name);
            let result = client
                .peer()
                .call_tool(rmcp::model::CallToolRequestParam {
                    name: sum_tool.name.clone(),
                    arguments: Some(rmcp::object!({
                        "a": 10,
                        "b": 20
                    })),
                })
                .await?;
            println!("Result: {:?}", result);
        }
        Ok(())
    }
    let _ = fs::remove_file(SOCKET_PATH);
    match UnixListener::bind(SOCKET_PATH) {
        Ok(unix_listener) => {
            println!("Server successfully listening on {}", SOCKET_PATH);
            tokio::spawn(server(unix_listener));
        }
        Err(e) => {
            println!("Unable to bind to {}: {}", SOCKET_PATH, e);
        }
    }
    client().await?;
    let _ = fs::remove_file(SOCKET_PATH);
    Ok(())
}
#[cfg(not(target_family = "unix"))]
fn main() {
    println!("Unix socket example is not supported on this platform.");
}
</file>

<file path="examples/transport/src/websocket.rs">
use std::marker::PhantomData;
use common::calculator::Calculator;
use futures::{Sink, Stream};
use rmcp::{
    RoleClient, RoleServer, ServiceExt,
    service::{RunningService, RxJsonRpcMessage, ServiceRole, TxJsonRpcMessage},
};
use tokio_tungstenite::tungstenite;
use tracing_subscriber::EnvFilter;
mod common;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();
    start_server().await?;
    let client = http_client("ws://127.0.0.1:8001").await?;
    let tools = client.list_all_tools().await?;
    client.cancel().await?;
    tracing::info!("{:#?}", tools);
    Ok(())
}
async fn http_client(uri: &str) -> anyhow::Result<RunningService<RoleClient, ()>> {
    let (stream, response) = tokio_tungstenite::connect_async(uri).await?;
    if response.status() != tungstenite::http::StatusCode::SWITCHING_PROTOCOLS {
        return Err(anyhow::anyhow!("failed to upgrade connection"));
    }
    let transport = WebsocketTransport::new_client(stream);
    let client = ().serve(transport).await?;
    Ok(client)
}
async fn start_server() -> anyhow::Result<()> {
    let tcp_listener = tokio::net::TcpListener::bind("127.0.0.1:8001").await?;
    tokio::spawn(async move {
        while let Ok((stream, addr)) = tcp_listener.accept().await {
            tracing::info!("accepted connection from: {}", addr);
            tokio::spawn(async move {
                let ws_stream = tokio_tungstenite::accept_async(stream).await?;
                let transport = WebsocketTransport::new_server(ws_stream);
                let server = Calculator::new().serve(transport).await?;
                server.waiting().await?;
                Ok::<(), anyhow::Error>(())
            });
        }
    });
    Ok(())
}
pin_project_lite::pin_project! {
    pub struct WebsocketTransport<R, S, E> {
        #[pin]
        stream: S,
        marker: PhantomData<(fn() -> E, fn() -> R)>
    }
}
impl<R, S, E> WebsocketTransport<R, S, E> {
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            marker: PhantomData,
        }
    }
}
impl<S, E> WebsocketTransport<RoleClient, S, E> {
    pub fn new_client(stream: S) -> Self {
        Self {
            stream,
            marker: PhantomData,
        }
    }
}
impl<S, E> WebsocketTransport<RoleServer, S, E> {
    pub fn new_server(stream: S) -> Self {
        Self {
            stream,
            marker: PhantomData,
        }
    }
}
impl<R, S, E> Stream for WebsocketTransport<R, S, E>
where
    S: Stream<Item = Result<tungstenite::Message, E>>,
    R: ServiceRole,
    E: std::error::Error,
{
    type Item = RxJsonRpcMessage<R>;
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.as_mut().project();
        match this.stream.poll_next(cx) {
            std::task::Poll::Ready(Some(Ok(message))) => {
                let message = match message {
                    tungstenite::Message::Text(json) => json,
                    _ => return self.poll_next(cx),
                };
                let message = match serde_json::from_str::<RxJsonRpcMessage<R>>(&message) {
                    Ok(message) => message,
                    Err(e) => {
                        tracing::warn!(error = %e, "serde_json parse error");
                        return self.poll_next(cx);
                    }
                };
                std::task::Poll::Ready(Some(message))
            }
            std::task::Poll::Ready(Some(Err(e))) => {
                tracing::warn!(error = %e, "websocket error");
                self.poll_next(cx)
            }
            std::task::Poll::Ready(None) => std::task::Poll::Ready(None),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}
impl<R, S, E> Sink<TxJsonRpcMessage<R>> for WebsocketTransport<R, S, E>
where
    S: Sink<tungstenite::Message, Error = E>,
    R: ServiceRole,
{
    type Error = E;
    fn poll_ready(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        let this = self.project();
        this.stream.poll_ready(cx)
    }
    fn start_send(
        self: std::pin::Pin<&mut Self>,
        item: TxJsonRpcMessage<R>,
    ) -> Result<(), Self::Error> {
        let this = self.project();
        let message = tungstenite::Message::Text(
            serde_json::to_string(&item)
                .expect("jsonrpc should be valid json")
                .into(),
        );
        this.stream.start_send(message)
    }
    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        let this = self.project();
        this.stream.poll_flush(cx)
    }
    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        let this = self.project();
        this.stream.poll_close(cx)
    }
}
</file>

<file path="examples/wasi/src/calculator.rs">
#![allow(dead_code)]
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::Parameters, wrapper::Json},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
};
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SumRequest {
    #[schemars(description = "the left hand side number")]
    pub a: i32,
    pub b: i32,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SubRequest {
    #[schemars(description = "the left hand side number")]
    pub a: i32,
    #[schemars(description = "the right hand side number")]
    pub b: i32,
}
#[derive(Debug, Clone)]
pub struct Calculator {
    tool_router: ToolRouter<Self>,
}
impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}
#[tool_router]
impl Calculator {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
    #[tool(description = "Calculate the sum of two numbers")]
    fn sum(&self, Parameters(SumRequest { a, b }): Parameters<SumRequest>) -> String {
        (a + b).to_string()
    }
    #[tool(description = "Calculate the difference of two numbers")]
    fn sub(&self, Parameters(SubRequest { a, b }): Parameters<SubRequest>) -> Json<i32> {
        Json(a - b)
    }
}
#[tool_handler]
impl ServerHandler for Calculator {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("A simple calculator".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
</file>

<file path="examples/wasi/src/lib.rs">
pub mod calculator;
use std::task::{Poll, Waker};
use rmcp::ServiceExt;
use tokio::io::{AsyncRead, AsyncWrite};
use tracing_subscriber::EnvFilter;
use wasi::{
    cli::{
        stdin::{InputStream, get_stdin},
        stdout::{OutputStream, get_stdout},
    },
    io::streams::Pollable,
};
pub fn wasi_io() -> (AsyncInputStream, AsyncOutputStream) {
    let input = AsyncInputStream { inner: get_stdin() };
    let output = AsyncOutputStream {
        inner: get_stdout(),
    };
    (input, output)
}
pub struct AsyncInputStream {
    inner: InputStream,
}
impl AsyncRead for AsyncInputStream {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let bytes = self
            .inner
            .read(buf.remaining() as u64)
            .map_err(std::io::Error::other)?;
        if bytes.is_empty() {
            let pollable = self.inner.subscribe();
            let waker = cx.waker().clone();
            runtime_poll(waker, pollable);
            return Poll::Pending;
        }
        buf.put_slice(&bytes);
        std::task::Poll::Ready(Ok(()))
    }
}
pub struct AsyncOutputStream {
    inner: OutputStream,
}
fn runtime_poll(waker: Waker, pollable: Pollable) {
    tokio::task::spawn(async move {
        loop {
            if pollable.ready() {
                waker.wake();
                break;
            } else {
                tokio::task::yield_now().await;
            }
        }
    });
}
impl AsyncWrite for AsyncOutputStream {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        let writable_len = self.inner.check_write().map_err(std::io::Error::other)?;
        if writable_len == 0 {
            let pollable = self.inner.subscribe();
            let waker = cx.waker().clone();
            runtime_poll(waker, pollable);
            return Poll::Pending;
        }
        let bytes_to_write = buf.len().min(writable_len as usize);
        self.inner
            .write(&buf[0..bytes_to_write])
            .map_err(std::io::Error::other)?;
        Poll::Ready(Ok(bytes_to_write))
    }
    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        self.inner.flush().map_err(std::io::Error::other)?;
        Poll::Ready(Ok(()))
    }
    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        self.poll_flush(cx)
    }
}
struct TokioCliRunner;
impl wasi::exports::cli::run::Guest for TokioCliRunner {
    fn run() -> Result<(), ()> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
            tracing_subscriber::fmt()
                .with_env_filter(
                    EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()),
                )
                .with_writer(std::io::stderr)
                .with_ansi(false)
                .init();
            let server = calculator::Calculator::new()
                .serve(wasi_io())
                .await
                .unwrap();
            server.waiting().await.unwrap();
        });
        Ok(())
    }
}
wasi::cli::command::export!(TokioCliRunner);
</file>

</files>


<macro-usage>
# rust-mcp-macros.

A procedural macro, part of the [rust-mcp-sdk](https://github.com/rust-mcp-stack/rust-mcp-sdk) ecosystem, to generate `rust_mcp_schema::Tool` instance from a struct.

The `mcp_tool` macro generates an implementation for the annotated struct that includes:

- A `tool_name()` method returning the tool's name as a string.
- A `tool()` method returning a `rust_mcp_schema::Tool` instance with the tool's name,
  description, and input schema derived from the struct's fields.

## Attributes

- `name` - The name of the tool (required, non-empty string).
- `description` - A description of the tool (required, non-empty string).
- `title` - An optional human-readable and easily understood title.
- `meta` - An optional JSON string that provides additional metadata for the tool.
- `destructive_hint` – Optional boolean, indicates whether the tool may make destructive changes to its environment.
- `idempotent_hint` – Optional boolean, indicates whether repeated calls with the same input have the same effect.
- `open_world_hint` – Optional boolean, indicates whether the tool can interact with external or unknown entities.
- `read_only_hint` – Optional boolean, indicates whether the tool makes no modifications to its environment.



## Usage Example

```rust
#[mcp_tool(
   name = "write_file",
   title = "Write File Tool"
   description = "Create a new file or completely overwrite an existing file with new content."
   destructive_hint = false
   idempotent_hint = false
   open_world_hint = false
   read_only_hint = false
   meta = r#"{
       "key" : "value",
       "string_meta" : "meta value",
       "numeric_meta" : 15
   }"#
)]
#[derive(rust_mcp_macros::JsonSchema)]
pub struct WriteFileTool {
    /// The target file's path for writing content.
    pub path: String,
    /// The string content to be written to the file
    pub content: String,
}

fn main() {

    assert_eq!(WriteFileTool::tool_name(), "write_file");

    let tool: rust_mcp_schema::Tool = WriteFileTool::tool();
    assert_eq!(tool.name, "write_file");
    assert_eq!(tool.title.as_ref().unwrap(), "Write File Tool");
    assert_eq!( tool.description.unwrap(),"Create a new file or completely overwrite an existing file with new content.");

    let meta: &Map<String, Value> = tool.meta.as_ref().unwrap();
    assert_eq!(
        meta.get("key").unwrap(),
        &Value::String("value".to_string())
    );

    let schema_properties = tool.input_schema.properties.unwrap();
    assert_eq!(schema_properties.len(), 2);
    assert!(schema_properties.contains_key("path"));
    assert!(schema_properties.contains_key("content"));

    // get the `content` prop from schema
    let content_prop = schema_properties.get("content").unwrap();

    // assert the type
    assert_eq!(content_prop.get("type").unwrap(), "string");
    // assert the description
    assert_eq!(
        content_prop.get("description").unwrap(),
        "The string content to be written to the file"
    );
}

```
</macro-usage>