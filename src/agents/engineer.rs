use async_trait::async_trait;
use anyhow::Result;

use super::base::Agent;
use crate::llm::{LLMClient, Message};

pub struct EngineerAgent {
    name: String,
    role: String,
}

impl EngineerAgent {
    pub fn new() -> Self {
        Self {
            name: "Engineer".to_string(),
            role: "Code implementation and technical solutions".to_string(),
        }
    }
}

#[async_trait]
impl Agent for EngineerAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn role(&self) -> &str {
        &self.role
    }

    async fn process(&self, messages: Vec<Message>, llm: &LLMClient) -> Result<String> {
        llm.chat(messages).await
    }
}
