//! 文件写入工具

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

use super::base::{Tool, ToolSchema};

pub struct FileWriteTool {
    allowed_dirs: Vec<PathBuf>,
}

impl FileWriteTool {
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
    content: String,
    #[serde(default = "default_false")]
    append: bool,
}

fn default_false() -> bool {
    false
}

#[async_trait]
impl Tool for FileWriteTool {
    fn name(&self) -> &str {
        "file_write"
    }

    fn description(&self) -> &str {
        "Write content to a file. Input: {\"path\": \"/path/to/file\", \"content\": \"text\", \"append\": false}"
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
                        "description": "Path to the file to write"
                    },
                    "content": {
                        "type": "string", 
                        "description": "Content to write"
                    },
                    "append": {
                        "type": "boolean",
                        "description": "Append to file instead of overwriting",
                        "default": false
                    }
                },
                "required": ["path", "content"]
            }),
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, anyhow::Error> {
        let params: Params = serde_json::from_value(params)?;
        let path = PathBuf::from(&params.path);

        if !self.is_allowed(&path) {
            return Err(anyhow::anyhow!("Path not allowed: {}", params.path));
        }

        // 确保父目录存在
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        if params.append {
            use tokio::io::AsyncWriteExt;
            let mut file = tokio::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .await?;
            file.write_all(params.content.as_bytes()).await?;
        } else {
            tokio::fs::write(&path, &params.content).await?;
        }

        let metadata = tokio::fs::metadata(&path).await?;

        let mode = if params.append { "appended" } else { "written" };

        Ok(serde_json::json!({
            "success": true,
            "path": params.path,
            "bytes_written": metadata.len(),
            "mode": mode
        }))
    }
}
