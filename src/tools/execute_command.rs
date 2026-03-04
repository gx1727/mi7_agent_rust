//! 命令执行工具

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::Stdio;
use tokio::process::Command;

use super::base::{Tool, ToolSchema};

pub struct ExecuteCommandTool {
    allowed_commands: Vec<String>,
    timeout_secs: u64,
}

impl ExecuteCommandTool {
    pub fn new(allowed_commands: Vec<String>, timeout_secs: u64) -> Self {
        Self {
            allowed_commands,
            timeout_secs,
        }
    }
    
    fn is_allowed(&self, cmd: &str) -> bool {
        // 检查是否在允许列表中
        for allowed in &self.allowed_commands {
            if cmd.starts_with(allowed) || allowed == "*" {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Params {
    command: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    cwd: Option<String>,
}

#[async_trait]
impl Tool for ExecuteCommandTool {
    fn name(&self) -> &str {
        "execute_command"
    }

    fn description(&self) -> &str {
        "Execute a shell command. Input: {\"command\": \"ls\", \"args\": [\"-la\"], \"cwd\": \"/path\"}"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "Command to execute"
                    },
                    "args": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Command arguments"
                    },
                    "cwd": {
                        "type": "string",
                        "description": "Working directory"
                    }
                },
                "required": ["command"]
            }),
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, anyhow::Error> {
        let params: Params = serde_json::from_value(params)?;

        if !self.is_allowed(&params.command) {
            return Err(anyhow::anyhow!(
                "Command not allowed: {}. Allowed: {:?}",
                params.command,
                self.allowed_commands
            ));
        }

        let mut cmd = Command::new(&params.command);
        cmd.args(&params.args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(cwd) = &params.cwd {
            cmd.current_dir(cwd);
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to execute: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // 限制输出大小
        let max_size = 20 * 1024; // 20KB
        let stdout = if stdout.len() > max_size {
            format!("{}...[truncated]", &stdout[..max_size])
        } else {
            stdout.to_string()
        };

        Ok(serde_json::json!({
            "success": output.status.success(),
            "exit_code": output.status.code(),
            "stdout": stdout,
            "stderr": stderr.to_string(),
        }))
    }
}
