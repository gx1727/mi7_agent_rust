use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;

use super::base::Agent;
use crate::llm::{LLMClient, Message, Function, ToolCall};
use crate::tools::ToolRegistry;

pub struct ResearcherAgent {
    name: String,
    role: String,
    tools: Option<Arc<ToolRegistry>>,
}

impl ResearcherAgent {
    pub fn new() -> Self {
        Self {
            name: "Researcher".to_string(),
            role: "Information retrieval and analysis".to_string(),
            tools: None,
        }
    }
    
    pub fn with_tools(mut self, tools: Arc<ToolRegistry>) -> Self {
        self.tools = Some(tools);
        self
    }

    async fn execute_tool_calls(&self, tool_calls: Vec<ToolCall>) -> Result<Vec<(String, String)>> {
        let mut results = Vec::new();
        
        for tool_call in tool_calls {
            let function = tool_call.function;
            let tool_name = function.name;
            let args = function.arguments;
            
            if let Some(tool) = self.tools.as_ref().and_then(|t: &Arc<ToolRegistry>| t.get(&tool_name)) {
                let params: serde_json::Value = serde_json::from_str(&args).unwrap_or(serde_json::Value::Null);
                match tool.execute(params).await {
                    Ok(result) => {
                        results.push((tool_call.id, result.to_string()));
                    }
                    Err(e) => {
                        results.push((tool_call.id, format!("Error: {}", e)));
                    }
                }
            } else {
                results.push((tool_call.id, format!("Tool not found: {}", tool_name)));
            }
        }
        
        Ok(results)
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
        let tools = self.tools.as_ref().map(|t: &Arc<ToolRegistry>| {
            t.get_all_schemas().into_iter().map(|s| {
                Function {
                    name: s.name,
                    description: s.description,
                    parameters: s.parameters,
                }
            }).collect()
        });
        
        let response = llm.chat_with_tools(messages.clone(), tools).await?;
        
        if let Ok(tool_calls) = serde_json::from_str::<Vec<ToolCall>>(&response) {
            if !tool_calls.is_empty() {
                let tool_results = self.execute_tool_calls(tool_calls).await?;
                
                let mut final_messages = messages.clone();
                final_messages.push(Message::assistant(response));
                
                for (call_id, result) in tool_results {
                    final_messages.push(Message::tool(result, call_id));
                }
                
                return llm.chat_with_tools(final_messages, None).await;
            }
        }
        
        Ok(response)
    }
}
