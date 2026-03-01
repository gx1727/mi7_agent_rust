use async_trait::async_trait;
use anyhow::Result;

use super::base::Agent;
use crate::llm::{LLMClient, Message};

pub struct PlannerAgent {
    name: String,
    role: String,
}

impl PlannerAgent {
    pub fn new() -> Self {
        Self {
            name: "Planner".to_string(),
            role: "Task decomposition and planning".to_string(),
        }
    }
}

#[async_trait]
impl Agent for PlannerAgent {
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
