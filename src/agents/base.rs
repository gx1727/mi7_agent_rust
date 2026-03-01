use async_trait::async_trait;
use anyhow::Result;

use crate::llm::{LLMClient, Message};

#[async_trait]
pub trait Agent: Send + Sync {
    fn name(&self) -> &str;
    fn role(&self) -> &str;
    
    async fn process(&self, messages: Vec<Message>, llm: &LLMClient) -> Result<String>;
}

pub struct BaseAgent {
    pub name: String,
    pub role: String,
}

impl BaseAgent {
    pub fn new(name: String, role: String) -> Self {
        Self { name, role }
    }
}
