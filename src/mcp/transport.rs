use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::mcp::protocol::{
    CallToolParams, CallToolResult, InitializeResult, JsonRpcRequest, JsonRpcResponse,
    ListToolsResult, METHOD_INITIALIZE, METHOD_TOOLS_CALL, METHOD_TOOLS_LIST,
    ServerCapabilities, ServerInfo,
};

const DEFAULT_HTTP_TIMEOUT: u64 = 30;

#[derive(Debug, Clone)]
pub struct HttpTransportConfig {
    pub server_url: String,
    pub timeout: Duration,
    pub headers: std::collections::HashMap<String, String>,
}

impl HttpTransportConfig {
    pub fn from_env() -> Result<Self> {
        let server_url = std::env::var("MCP_SERVER_URL")
            .context("MCP_SERVER_URL environment variable not set")?;

        let timeout = std::env::var("MCP_TIMEOUT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_HTTP_TIMEOUT);

        let mut headers = std::collections::HashMap::new();
        if let Ok(auth) = std::env::var("MCP_AUTH") {
            headers.insert("Authorization".to_string(), auth);
        }

        Ok(Self {
            server_url,
            timeout: Duration::from_secs(timeout),
            headers,
        })
    }
}

pub struct HttpMcpTransport {
    config: HttpTransportConfig,
    client: Client,
    initialized: Arc<RwLock<bool>>,
    server_capabilities: Arc<RwLock<Option<ServerCapabilities>>>,
    server_info: Arc<RwLock<Option<ServerInfo>>>,
    session_id: Arc<RwLock<Option<String>>>,
}

impl HttpMcpTransport {
    pub fn new(config: HttpTransportConfig) -> Self {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            config,
            client,
            initialized: Arc::new(RwLock::new(false)),
            server_capabilities: Arc::new(RwLock::new(None)),
            server_info: Arc::new(RwLock::new(None)),
            session_id: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn connect(&self) -> Result<()> {
        info!("Connecting to MCP server via HTTP: {}", self.config.server_url);
        
        // Simple connection test - try to reach the server
        let url = format!("{}/", self.config.server_url.trim_end_matches('/'));
        let response = self.client.get(&url).send().await;
        
        match response {
            Ok(resp) => {
                if resp.status().is_success() || resp.status().as_u16() == 404 {
                    // 404 is OK - it means server is running
                    info!("MCP server connection verified");
                    return Ok(());
                }
                Err(anyhow::anyhow!("Server returned: {}", resp.status()))
            }
            Err(e) => Err(anyhow::anyhow!("Failed to connect: {}", e))
        }
    }

    pub async fn initialize(&self) -> Result<InitializeResult> {
        if *self.initialized.read().await {
            return Err(anyhow::anyhow!("Already initialized"));
        }

        info!("Initializing MCP HTTP transport");

        let init_result = self
            .send_request(METHOD_INITIALIZE, Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "mi7-agent-rust",
                    "version": "0.1.0"
                }
            })))
            .await?;

        let result: InitializeResult = serde_json::from_value(init_result)
            .context("Failed to parse initialize result")?;

        *self.server_capabilities.write().await = Some(result.capabilities.clone());
        *self.server_info.write().await = Some(result.server_info.clone());
        *self.initialized.write().await = true;

        info!("MCP HTTP transport initialized: {:?}", result.server_info);

        Ok(result)
    }

    pub async fn list_tools(&self) -> Result<Vec<crate::mcp::protocol::Tool>> {
        if !*self.initialized.read().await {
            return Err(anyhow::anyhow!("Not initialized"));
        }

        let result = self.send_request(METHOD_TOOLS_LIST, None).await?;
        let list_result: ListToolsResult = serde_json::from_value(result)
            .context("Failed to parse tools list result")?;

        Ok(list_result.tools)
    }

    pub async fn call_tool(&self, name: &str, arguments: Option<Value>) -> Result<CallToolResult> {
        if !*self.initialized.read().await {
            return Err(anyhow::anyhow!("Not initialized"));
        }

        let params = CallToolParams {
            name: name.to_string(),
            arguments,
        };

        let result = self
            .send_request(METHOD_TOOLS_CALL, Some(serde_json::to_value(params)?))
            .await?;

        let call_result: CallToolResult = serde_json::from_value(result)
            .context("Failed to parse tool call result")?;

        Ok(call_result)
    }

    pub async fn send_request(&self, method: &str, params: Option<Value>) -> Result<Value> {
        let request = JsonRpcRequest::new(method, params);
        let request_json = serde_json::to_string(&request)
            .context("Failed to serialize JSON-RPC request")?;

        debug!("Sending HTTP request: {}", request_json);

        let url = format!("{}/rpc", self.config.server_url.trim_end_matches('/'));
        let mut req_builder = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");

        for (key, value) in &self.config.headers {
            req_builder = req_builder.header(key.as_str(), value.as_str());
        }

        let response = req_builder
            .body(request_json)
            .send()
            .await
            .context("Failed to send HTTP request")?;

        let status = response.status();
        if !status.is_success() {
            return Err(anyhow::anyhow!("HTTP error: {} - {}", status, response.text().await.unwrap_or_default()));
        }

        let response_text = response.text().await
            .context("Failed to read response body")?;

        debug!("Received HTTP response: {}", response_text);

        let rpc_response: JsonRpcResponse = serde_json::from_str(&response_text)
            .context("Failed to parse JSON-RPC response")?;

        if let Some(error) = rpc_response.error {
            return Err(anyhow::anyhow!("JSON-RPC error: {} - {}", error.code, error.message));
        }

        rpc_response.result
            .ok_or_else(|| anyhow::anyhow!("No result in response"))
    }

    pub async fn get_server_info(&self) -> Option<ServerInfo> {
        self.server_info.read().await.clone()
    }

    pub async fn get_capabilities(&self) -> Option<ServerCapabilities> {
        self.server_capabilities.read().await.clone()
    }

    pub async fn is_initialized(&self) -> bool {
        *self.initialized.read().await
    }

    pub fn server_url(&self) -> &str {
        &self.config.server_url
    }
}

pub async fn create_http_client_from_env() -> Result<HttpMcpTransport> {
    let config = HttpTransportConfig::from_env()?;
    let transport = HttpMcpTransport::new(config);
    transport.connect().await?;
    transport.initialize().await?;
    Ok(transport)
}
