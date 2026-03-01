use anyhow::Result;
use futures::StreamExt;
use reqwest::Client;
use tracing::{debug, info};

use super::types::*;
use crate::config::Config;

pub struct LLMClient {
    client: Client,
    config: Config,
}

impl LLMClient {
    pub fn new(config: Config) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub async fn chat(&self, messages: Vec<Message>) -> Result<String> {
        let request = ChatCompletionRequest {
            model: self.config.llm_model.clone(),
            messages,
            max_tokens: Some(self.config.max_tokens),
            temperature: Some(self.config.temperature),
            stream: None,
        };

        let url = format!("{}/chat/completions", self.config.llm_base_url);
        
        debug!("Sending request to {}", url);
        
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.llm_api_key))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("LLM API error: {}", error_text);
        }

        let completion: ChatCompletionResponse = response.json().await?;
        
        info!("Chat completion received: {}", completion.id);

        Ok(completion.choices[0].message.content.clone())
    }

    pub async fn chat_stream(&self, messages: Vec<Message>) -> Result<()> {
        let request = ChatCompletionRequest {
            model: self.config.llm_model.clone(),
            messages,
            max_tokens: Some(self.config.max_tokens),
            temperature: Some(self.config.temperature),
            stream: Some(true),
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
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let text = String::from_utf8_lossy(&chunk);
            
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
