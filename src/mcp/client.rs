use std::process::Stdio;
use std::sync::Arc;

use anyhow::{Context, Result};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Command, Child};
use tokio::sync::RwLock;

use crate::mcp::protocol::*;
use crate::mcp::tool::McpTool;
use crate::mcp::transport::{create_http_client_from_env, HttpMcpTransport};
use crate::tools::Tool;

pub struct McpClient {
    inner: Arc<RwLock<Option<McpConnection>>>,
    server_info: Arc<RwLock<Option<ServerInfo>>>,
    tools: Arc<RwLock<Vec<McpTool>>>,
    http_transport: Arc<RwLock<Option<HttpMcpTransport>>>,
}

enum McpConnection {
    Stdio(Child),
}

impl McpClient {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            inner: Arc::new(RwLock::new(None)),
            server_info: Arc::new(RwLock::new(None)),
            tools: Arc::new(RwLock::new(Vec::new())),
            http_transport: Arc::new(RwLock::new(None)),
        })
    }

    pub async fn connect_http(&self) -> Result<()> {
        let transport = create_http_client_from_env().await?;
        let server_info = transport.get_server_info().await
            .ok_or_else(|| anyhow::anyhow!("Failed to get server info"))?;
        let tools_list = transport.list_tools().await?;

        *self.server_info.write().await = Some(server_info);

        let mut tools = Vec::new();
        for tool in tools_list {
            tools.push(McpTool::new_http(
                tool.name,
                tool.description,
                tool.input_schema,
            ));
        }

        *self.tools.write().await = tools;
        *self.http_transport.write().await = Some(transport);

        Ok(())
    }

    pub async fn connect_http_with_url(&self, url: &str) -> Result<()> {
        use crate::mcp::transport::{HttpMcpTransport, HttpTransportConfig};

        let config = HttpTransportConfig {
            server_url: url.to_string(),
            timeout: std::time::Duration::from_secs(30),
            headers: std::collections::HashMap::new(),
        };

        let transport = HttpMcpTransport::new(config);
        transport.connect().await?;
        let init_result = transport.initialize().await?;

        *self.server_info.write().await = Some(init_result.server_info);

        let tools_list = transport.list_tools().await?;

        let mut tools = Vec::new();
        for tool in tools_list {
            tools.push(McpTool::new_http(
                tool.name,
                tool.description,
                tool.input_schema,
            ));
        }

        *self.tools.write().await = tools;
        *self.http_transport.write().await = Some(transport);

        Ok(())
    }

    pub async fn connect_stdio(&self, command: &str, args: Vec<String>) -> Result<()> {
        let args_for_cmd = args.clone();
        let args_for_tools = args.clone();
        
        let mut cmd = Command::new(command);
        cmd.args(args_for_cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().context("Failed to spawn MCP server process")?;

        let stdin = child.stdin.take().context("Failed to get stdin")?;
        let stdout = child.stdout.take().context("Failed to get stdout")?;

        let mut writer = tokio::io::BufWriter::new(stdin);
        let mut reader = BufReader::new(stdout);

        // Initialize request
        let init_request = JsonRpcRequest::new(
            METHOD_INITIALIZE,
            Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "mi7-agent-rust",
                    "version": "0.1.0"
                }
            })),
        );

        let request_json = serde_json::to_string(&init_request)?;
        writer.write_all(request_json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;

        // Read response line by line
        let mut response_line = String::new();
        reader.read_line(&mut response_line).await?;
        
        if !response_line.is_empty() {
            let response: JsonRpcResponse = serde_json::from_str(&response_line)
                .context("Failed to parse initialize response")?;

            if let Some(result) = response.result {
                let init_result: InitializeResult = serde_json::from_value(result)
                    .context("Failed to parse initialize result")?;

                *self.server_info.write().await = Some(init_result.server_info);
            }
        }

        // List tools request
        let tools_list_request = JsonRpcRequest::new(METHOD_TOOLS_LIST, None);
        let request_json = serde_json::to_string(&tools_list_request)?;
        writer.write_all(request_json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;

        // Read tools response
        let mut tools_line = String::new();
        reader.read_line(&mut tools_line).await?;
        
        if !tools_line.is_empty() {
            let response: JsonRpcResponse = serde_json::from_str(&tools_line)
                .context("Failed to parse tools list response")?;

            if let Some(result) = response.result {
                let list_result: ListToolsResult = serde_json::from_value(result)
                    .context("Failed to parse tools list result")?;

                let mut tools = Vec::new();
                for tool in list_result.tools {
                    tools.push(McpTool::new(
                        tool.name,
                        tool.description,
                        tool.input_schema,
                        command.to_string(),
                        args_for_tools.clone(),
                    ));
                }

                *self.tools.write().await = tools;
            }
        }

        *self.inner.write().await = Some(McpConnection::Stdio(child));

        Ok(())
    }

    pub async fn list_tools(&self) -> Vec<McpTool> {
        self.tools.read().await.clone()
    }

    pub async fn get_server_info(&self) -> Option<ServerInfo> {
        self.server_info.read().await.clone()
    }

    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<Value> {
        if let Some(transport) = self.http_transport.read().await.as_ref() {
            let result = transport.call_tool(name, Some(arguments)).await?;
            return Ok(serde_json::to_value(result)?);
        }

        let tools = self.tools.read().await;
        let tool = tools.iter()
            .find(|t| t.name() == name)
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", name))?;

        tool.execute(arguments).await
    }
}
