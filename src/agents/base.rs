use async_trait::async_trait;
use anyhow::Result;
use std::sync::Arc;

use crate::llm::{LLMClient, Message};
use crate::tools::ToolRegistry;

#[derive(Debug, Clone)]
pub enum AgentMessage {
    Task { task_id: String, content: String, task_type: TaskType },
    Result { task_id: String, content: String, from_agent: String },
    Error { task_id: String, error: String },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskType {
    Research,
    Engineering,
    Planning,
    Coordination,
    Unknown,
}

impl TaskType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "research" | "researcher" | "investigation" => TaskType::Research,
            "engineering" | "engineer" | "code" | "implementation" => TaskType::Engineering,
            "planning" | "planner" | "plan" => TaskType::Planning,
            "coordination" | "coordination" | "coordinate" => TaskType::Coordination,
            _ => TaskType::Unknown,
        }
    }
}

#[async_trait]
pub trait Agent: Send + Sync {
    fn name(&self) -> &str;
    fn role(&self) -> &str;
    
    async fn process(&self, messages: Vec<Message>, llm: &LLMClient) -> Result<String>;
    
    async fn process_with_tools(
        &self, 
        messages: Vec<Message>, 
        llm: &LLMClient, 
        tools: Option<Arc<ToolRegistry>>
    ) -> Result<String> {
        self.process(messages, llm).await
    }
}

pub struct BaseAgent {
    pub name: String,
    pub role: String,
    pub tools: Option<Arc<ToolRegistry>>,
}

impl BaseAgent {
    pub fn new(name: String, role: String) -> Self {
        Self { name, role, tools: None }
    }
    
    pub fn with_tools(mut self, tools: Arc<ToolRegistry>) -> Self {
        self.tools = Some(tools);
        self
    }
}
