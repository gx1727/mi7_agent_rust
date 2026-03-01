use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub llm_provider: String,
    pub llm_api_key: String,
    pub llm_model: String,
    pub llm_base_url: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Config {
            llm_provider: env::var("LLM_PROVIDER").unwrap_or_else(|_| "glm".to_string()),
            llm_api_key: env::var("LLM_API_KEY")?,
            llm_model: env::var("LLM_MODEL").unwrap_or_else(|_| "glm-4".to_string()),
            llm_base_url: env::var("LLM_BASE_URL").unwrap_or_else(|_| {
                "https://open.bigmodel.cn/api/paas/v4".to_string()
            }),
            max_tokens: env::var("LLM_MAX_TOKENS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()?,
            temperature: env::var("LLM_TEMPERATURE")
                .unwrap_or_else(|_| "0.7".to_string())
                .parse()?,
        })
    }
}
