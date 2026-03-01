use async_trait::async_trait;
use anyhow::Result;

use super::base::Agent;
use crate::llm::{LLMClient, Message};

pub struct ResearcherAgent {
    name: String,
    role: String,
}

impl ResearcherAgent {
    pub fn new() -> Self {
        Self {
            name: "Researcher".to_string(),
            role: "Information retrieval and analysis".to_string(),
        }
    }
}

#[async_trait]
impl Agent for ResearcherAgent {
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
