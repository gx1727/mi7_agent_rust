use async_trait::async_trait;
use anyhow::Result;

use super::base::Agent;
use crate::llm::{LLMClient, Message};

pub struct ManagerAgent {
    name: String,
    role: String,
}

impl ManagerAgent {
    pub fn new() -> Self {
        Self {
            name: "Manager".to_string(),
            role: "Coordinator and router".to_string(),
        }
    }
}

#[async_trait]
impl Agent for ManagerAgent {
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
