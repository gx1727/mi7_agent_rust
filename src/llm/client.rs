use std::sync::Arc;

use anyhow::Result;
use futures::StreamExt;
use reqwest::Client;
use tracing::{debug, info, warn};

use super::types::*;
use crate::config::Config;
use crate::error::{AgentError, RetryConfig, RetryState, classify_error};

pub struct LLMClient {
    client: Client,
    config: Config,
    retry_config: RetryConfig,
}

impl LLMClient {
    pub fn new(config: Config) -> Self {
        Self {
            client: Client::new(),
            config,
            retry_config: RetryConfig::default(),
        }
    }

    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    pub async fn chat(&self, messages: Vec<Message>) -> Result<String> {
        self.chat_with_tools(messages, None).await
    }

    pub async fn chat_with_tools(&self, messages: Vec<Message>, tools: Option<Vec<Function>>) -> Result<String> {
        let mut retry_state = RetryState::new();
        
        loop {
            match self.do_chat(messages.clone(), tools.clone()).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    let error = if e.downcast_ref::<reqwest::Error>().is_some() {
                        AgentError::Network(e.to_string())
                    } else {
                        AgentError::Api(e.to_string())
                    };
                    
                    warn!("LLM request failed: {}", error);
                    
                    if !error.is_retryable() || !retry_state.should_retry(&self.retry_config) {
                        return Err(e);
                    }
                    
                    let delay = retry_state.next_delay(&self.retry_config);
                    retry_state.record_attempt(Arc::new(error));
                    
                    info!("Retrying in {}ms (attempt {}/{})", 
                        delay, retry_state.attempts, self.retry_config.max_retries);
                    
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                }
            }
        }
    }

    async fn do_chat(&self, messages: Vec<Message>, tools: Option<Vec<Function>>) -> Result<String> {
        let mut request = ChatCompletionRequest {
            model: self.config.llm_model.clone(),
            messages,
            max_tokens: Some(self.config.max_tokens),
            temperature: Some(self.config.temperature),
            stream: None,
            tools,
        };

        let url = format!("{}/chat/completions", self.config.llm_base_url);
        
        debug!("Sending request to {}", url);
        
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.llm_api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    anyhow::Error::from(AgentError::Timeout(e.to_string()))
                } else if e.is_connect() {
                    anyhow::Error::from(AgentError::Network(e.to_string()))
                } else {
                    anyhow::Error::from(AgentError::Network(e.to_string()))
                }
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(classify_error(status, &error_text).into());
        }

        let completion: ChatCompletionResponse = response.json().await?;
        
        info!("Chat completion received: {}", completion.id);

        let message = &completion.choices[0].message;
        
        if let Some(tool_calls) = &message.tool_calls {
            if !tool_calls.is_empty() {
                return Ok(serde_json::to_string(&message.tool_calls)?);
            }
        }
        
        Ok(message.content.clone())
    }

    pub async fn chat_stream(&self, messages: Vec<Message>) -> Result<()> {
        let request = ChatCompletionRequest {
            model: self.config.llm_model.clone(),
            messages,
            max_tokens: Some(self.config.max_tokens),
            temperature: Some(self.config.temperature),
            stream: Some(true),
            tools: None,
        };

        let url = format!("{}/chat/completions", self.config.llm_base_url);
        
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.llm_api_key))
            .json(&request)
            .send()
            .await?;

        let mut stream = response.bytes_stream();
        let mut buffer = Vec::new();
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            buffer.extend_from_slice(&chunk);
            
            // 尝试将缓冲区转换为字符串，处理可能的 UTF-8 截断
            let text = match std::str::from_utf8(&buffer) {
                Ok(s) => {
                    // 成功转换，清空缓冲区
                    let s = s.to_string();
                    buffer.clear();
                    s
                }
                Err(e) => {
                    // UTF-8 截断，保留有效部分
                    let valid_len = e.valid_up_to();
                    if valid_len == 0 {
                        // 没有有效数据，继续等待更多数据
                        continue;
                    }
                    let s = std::str::from_utf8(&buffer[..valid_len]).unwrap().to_string();
                    buffer = buffer[valid_len..].to_vec();
                    s
                }
            };
            
            // Parse SSE data
            for line in text.lines() {
                if line.starts_with("data: ") {
                    let data = &line[6..];
                    if data == "[DONE]" {
                        println!();
                        return Ok(());
                    }
                    
                    if let Ok(stream_response) = serde_json::from_str::<StreamResponse>(data) {
                        if let Some(choice) = stream_response.choices.first() {
                            if let Some(content) = &choice.delta.content {
                                print!("{}", content);
                                use std::io::Write;
                                std::io::stdout().flush().ok();
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
