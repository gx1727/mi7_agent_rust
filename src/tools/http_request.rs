//! HTTP 请求工具 - 支持自定义 Headers 和 Body

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use super::base::{Tool, ToolSchema};

pub struct HttpRequestTool {
    default_headers: HashMap<String, String>,
}

impl HttpRequestTool {
    pub fn new(default_headers: HashMap<String, String>) -> Self {
        Self { default_headers }
    }
    
    pub fn with_token(token: String) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        Self::new(headers)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Params {
    url: String,
    #[serde(default = "default_method")]
    method: String,
    #[serde(default)]
    headers: HashMap<String, String>,
    #[serde(default)]
    body: Option<Value>,
}

fn default_method() -> String {
    "GET".to_string()
}

#[async_trait]
impl Tool for HttpRequestTool {
    fn name(&self) -> &str {
        "http_request"
    }

    fn description(&self) -> &str {
        "发送 HTTP 请求。用于调用外部 API"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "请求 URL"
                    },
                    "method": {
                        "type": "string",
                        "description": "请求方法：GET, POST, PUT, DELETE",
                        "default": "GET"
                    },
                    "headers": {
                        "type": "object",
                        "description": "自定义请求头（会合并默认 headers）"
                    },
                    "body": {
                        "type": "object",
                        "description": "请求体（JSON）"
                    }
                },
                "required": ["url"]
            }),
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, anyhow::Error> {
        let params: Params = serde_json::from_value(params)?;
        
        let client = reqwest::Client::new();
        
        // 构建请求
        let mut request = match params.method.to_uppercase().as_str() {
            "GET" => client.get(&params.url),
            "POST" => client.post(&params.url),
            "PUT" => client.put(&params.url),
            "DELETE" => client.delete(&params.url),
            _ => return Err(anyhow::anyhow!("不支持的方法: {}", params.method)),
        };
        
        // 添加默认 headers
        for (key, value) in &self.default_headers {
            request = request.header(key, value);
        }
        
        // 添加自定义 headers
        for (key, value) in &params.headers {
            request = request.header(key, value);
        }
        
        // 添加 body
        if let Some(body) = params.body {
            request = request.json(&body);
        }
        
        // 发送请求
        let mut response = request.send().await?;
        
        let status = response.status().as_u16();
        let success = response.status().is_success();
        
        let headers: HashMap<String, String> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        
        // 尝试解析 JSON 响应
        let body_text = response.text().await?;
        let body = match serde_json::from_str::<Value>(&body_text) {
            Ok(json) => json,
            Err(_) => Value::String(body_text),
        };

        Ok(serde_json::json!({
            "success": success,
            "status": status,
            "headers": headers,
            "body": body
        }))
    }
}
