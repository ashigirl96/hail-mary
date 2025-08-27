use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

// OAuth authentication structure matching TypeScript
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OAuthAuth {
    #[serde(rename = "type")]
    auth_type: String,
    pub access: Option<String>,
    pub refresh: Option<String>,
    pub expires: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthFile {
    anthropic: OAuthAuth,
}

// Load OAuth authentication from auth.json file
pub async fn load_auth() -> Result<OAuthAuth> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    let auth_path = home.join(".local/share/opencode/auth.json");

    let auth_content = tokio::fs::read_to_string(&auth_path).await?;
    let auth_file: AuthFile = serde_json::from_str(&auth_content)?;

    Ok(auth_file.anthropic)
}

// Refresh OAuth token if expired
pub async fn refresh_token(auth: &mut OAuthAuth) -> Result<()> {
    if auth.refresh.is_none() {
        return Err(anyhow::anyhow!("No refresh token available"));
    }

    let client = reqwest::Client::new();
    let refresh_body = json!({
        "grant_type": "refresh_token",
        "refresh_token": auth.refresh.as_ref().unwrap(),
        "client_id": "9d1c250a-e61b-44d9-88ed-5944d1962f5e"
    });

    let response = client
        .post("https://console.anthropic.com/v1/oauth/token")
        .json(&refresh_body)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow::anyhow!("Token refresh failed: {}", error_text));
    }

    #[derive(Deserialize)]
    struct TokenResponse {
        access_token: String,
        #[allow(dead_code)]
        refresh_token: Option<String>,
        expires_in: u64,
    }

    let token_response: TokenResponse = response.json().await?;

    auth.access = Some(token_response.access_token);
    auth.expires =
        Some(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() + token_response.expires_in);

    // Save updated auth
    save_auth(auth).await?;

    Ok(())
}

// Save updated auth back to file
async fn save_auth(auth: &OAuthAuth) -> Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    let auth_path = home.join(".local/share/opencode/auth.json");

    let auth_file = AuthFile {
        anthropic: auth.clone(),
    };

    let auth_content = serde_json::to_string_pretty(&auth_file)?;
    tokio::fs::write(&auth_path, auth_content).await?;

    Ok(())
}

// Check if token is expired
fn is_token_expired(auth: &OAuthAuth) -> bool {
    match auth.expires {
        Some(expires) => {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            now >= expires
        }
        None => true,
    }
}

// Non-streaming complete request
pub async fn complete(model: &str, message: &str, auth: &mut OAuthAuth) -> Result<String> {
    // Refresh token if expired
    if is_token_expired(auth) {
        refresh_token(auth).await?;
    }

    let access_token = auth
        .access
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No access token available"))?;

    // Prepare request headers (matching TypeScript order and content exactly)
    let mut headers = HeaderMap::new();
    headers.insert("content-type", HeaderValue::from_static("application/json"));
    headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
    headers.insert(
        "authorization",
        HeaderValue::from_str(&format!("Bearer {}", access_token))?,
    );
    headers.insert("anthropic-beta", HeaderValue::from_static("oauth-2025-04-20,claude-code-20250219,interleaved-thinking-2025-05-14,fine-grained-tool-streaming-2025-05-14"));
    // Set x-api-key to empty string for OAuth (matching TypeScript behavior)
    headers.insert("x-api-key", HeaderValue::from_static(""));
    // Add User-Agent to appear more like a browser request
    headers.insert("user-agent", HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"));

    // Prepare system prompts (similar to TypeScript version)
    let system_prompts = vec![
        json!({"type": "text", "text": "You are Claude Code, Anthropic's official CLI for Claude."}),
        json!({"type": "text", "text": "You are an interactive CLI tool that helps users with software engineering tasks."}),
        json!({"type": "text", "text": "IMPORTANT: Assist with defensive security tasks only."}),
        json!({"type": "text", "text": "Answer concisely with fewer than 4 lines unless user asks for detail."}),
        json!({"type": "text", "text": "Use Github-flavored markdown for formatting."}),
    ];

    // Prepare request payload (matching TypeScript field order exactly)
    // Note: Non-streaming doesn't include "stream" field (defaults to false)
    let payload = json!({
        "model": model,
        "max_tokens": 8192,
        "temperature": 0.7,
        "system": system_prompts,
        "messages": [{
            "role": "user",
            "content": [{
                "type": "text",
                "text": message
            }]
        }]
    });

    let client = reqwest::Client::new();
    let response = client
        .post("https://console.anthropic.com/v1/messages")
        .headers(headers)
        .json(&payload)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await?;
        return Err(anyhow::anyhow!(
            "Request failed ({}): {}",
            status,
            error_text
        ));
    }

    let response_json: Value = response.json().await?;

    // Extract text from response
    let mut result = String::new();
    if let Some(content) = response_json["content"].as_array() {
        for block in content {
            if block["type"] == "text" {
                if let Some(text) = block["text"].as_str() {
                    result.push_str(text);
                }
            }
        }
    }

    Ok(result)
}
