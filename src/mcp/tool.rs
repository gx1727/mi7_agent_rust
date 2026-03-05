use std::process::Stdio;
use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::RwLock;

use crate::mcp::protocol::{JsonRpcRequest, JsonRpcResponse, METHOD_TOOLS_CALL};

use crate::tools::{Tool, ToolSchema};

#[derive(Clone)]
pub struct McpTool {
    name: String,
    description: String,
    input_schema: Value,
    command: String,
    args: Vec<String>,
    initialized: Arc<RwLock<bool>>,
}

impl McpTool {
    pub fn new(name: String, description: String, input_schema: Value, command: String, args: Vec<String>) -> Self {
        Self {
            name,
            description,
            input_schema,
            command,
            args,
            initialized: Arc::new(RwLock::new(false)),
        }
    }

    pub fn new_http(name: String, description: String, input_schema: Value) -> Self {
        Self {
            name,
            description,
            input_schema,
            command: String::new(),
            args: Vec::new(),
            initialized: Arc::new(RwLock::new(true)),
        }
    }

    pub fn is_http(&self) -> bool {
        self.command.is_empty()
    }

    async fn ensure_initialized(&self) -> Result<()> {
        let mut initialized = self.initialized.write().await;
        if *initialized {
            return Ok(());
        }

        let mut cmd = Command::new(&self.command);
        cmd.args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().context("Failed to spawn MCP server process")?;

        let stdin = child.stdin.take().context("Failed to get stdin")?;
        let stdout = child.stdout.take().context("Failed to get stdout")?;

        let mut writer = tokio::io::BufWriter::new(stdin);
        let mut reader = BufReader::new(stdout).lines();

        let init_request = JsonRpcRequest::new(
            "initialize",
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

        if let Some(line) = reader.next_line().await? {
            let _response: JsonRpcResponse = serde_json::from_str(&line)
                .context("Failed to parse initialize response")?;
        }

        *initialized = true;
        Ok(())
    }

    pub async fn execute_remote(&self, params: Value) -> Result<Value> {
        self.ensure_initialized().await?;

        let mut cmd = Command::new(&self.command);
        cmd.args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().context("Failed to spawn MCP server process")?;

        let stdin = child.stdin.take().context("Failed to get stdin")?;
        let stdout = child.stdout.take().context("Failed to get stdout")?;

        let mut writer = tokio::io::BufWriter::new(stdin);
        let mut reader = BufReader::new(stdout).lines();

        let init_request = JsonRpcRequest::new(
            "initialize",
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

        if let Some(line) = reader.next_line().await? {
            let _response: JsonRpcResponse = serde_json::from_str(&line)
                .context("Failed to parse initialize response")?;
        }

        let tool_request = JsonRpcRequest::new(
            METHOD_TOOLS_CALL,
            Some(serde_json::json!({
                "name": self.name,
                "arguments": params
            })),
        );

        let request_json = serde_json::to_string(&tool_request)?;
        writer.write_all(request_json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;

        if let Some(line) = reader.next_line().await? {
            let response: JsonRpcResponse = serde_json::from_str(&line)
                .context("Failed to parse tool call response")?;

            if let Some(error) = response.error {
                return Err(anyhow::anyhow!("Tool call error: {}", error.message));
            }

            return Ok(response.result.unwrap_or(Value::Null));
        }

        Err(anyhow::anyhow!("No response from tool call"))
    }
}

#[async_trait]
impl Tool for McpTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name.clone(),
            description: self.description.clone(),
            parameters: self.input_schema.clone(),
        }
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        self.execute_remote(params).await
    }
}
