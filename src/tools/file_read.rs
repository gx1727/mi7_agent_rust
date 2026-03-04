//! 文件读取工具

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

use super::base::{Tool, ToolSchema};

pub struct FileReadTool {
    allowed_dirs: Vec<PathBuf>,
}

impl FileReadTool {
    pub fn new(allowed_dirs: Vec<String>) -> Self {
        Self {
            allowed_dirs: allowed_dirs.into_iter().map(PathBuf::from).collect(),
        }
    }
    
    fn is_allowed(&self, path: &PathBuf) -> bool {
        for dir in &self.allowed_dirs {
            if path.starts_with(dir) || path.starts_with("/root/work/") {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Params {
    path: String,
}

#[async_trait]
impl Tool for FileReadTool {
    fn name(&self) -> &str {
        "file_read"
    }

    fn description(&self) -> &str {
        "Read content from a file. Input: {\"path\": \"/path/to/file\"}"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to read"
                    }
                },
                "required": ["path"]
            }),
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, anyhow::Error> {
        let params: Params = serde_json::from_value(params)?;
        let path = PathBuf::from(&params.path);

        if !self.is_allowed(&path) {
            return Err(anyhow::anyhow!("Path not allowed: {}", params.path));
        }

        let content = tokio::fs::read_to_string(&path).await?;
        
        // 限制返回内容大小
        let max_size = 50 * 1024; // 50KB
        let content = if content.len() > max_size {
            format!("{}...[truncated {} bytes]", &content[..max_size], content.len() - max_size)
        } else {
            content
        };

        Ok(serde_json::json!({
            "success": true,
            "path": params.path,
            "content": content,
            "size": content.len()
        }))
    }
}
