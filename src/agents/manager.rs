use std::sync::Arc;
use std::collections::HashMap;
use async_trait::async_trait;
use anyhow::Result;
use serde_json::Value;
use tracing::{info, debug, warn};

use super::base::{Agent, AgentMessage, TaskType};
use super::{ResearcherAgent, EngineerAgent, PlannerAgent};
use crate::llm::{LLMClient, Message, Function, FunctionCall, ToolCall};
use crate::tools::ToolRegistry;
use crate::error::AgentError;

pub struct ManagerAgent {
    name: String,
    role: String,
    sub_agents: HashMap<String, Box<dyn Agent>>,
    tools: Option<Arc<ToolRegistry>>,
}

impl ManagerAgent {
    pub fn new() -> Self {
        let mut agents = HashMap::new();
        agents.insert("researcher".to_string(), Box::new(ResearcherAgent::new()) as Box<dyn Agent>);
        agents.insert("engineer".to_string(), Box::new(EngineerAgent::new()) as Box<dyn Agent>);
        agents.insert("planner".to_string(), Box::new(PlannerAgent::new()) as Box<dyn Agent>);
        
        Self {
            name: "Manager".to_string(),
            role: "Coordinator and router".to_string(),
            sub_agents: agents,
            tools: None,
        }
    }
    
    pub fn with_tools(mut self, tools: Arc<ToolRegistry>) -> Self {
        self.tools = Some(tools);
        self
    }

    fn route_task(&self, task_content: &str) -> TaskType {
        let task_lower = task_content.to_lowercase();
        
        if task_lower.contains("research") || task_lower.contains("find") || 
           task_lower.contains("search") || task_lower.contains("investigate") ||
           task_lower.contains("analyze") || task_lower.contains("information") {
            TaskType::Research
        } else if task_lower.contains("code") || task_lower.contains("implement") ||
                  task_lower.contains("build") || task_lower.contains("create") ||
                  task_lower.contains("fix") || task_lower.contains("debug") {
            TaskType::Engineering
        } else if task_lower.contains("plan") || task_lower.contains("schedule") ||
                  task_lower.contains("organize") || task_lower.contains("coordinate") {
            TaskType::Planning
        } else {
            TaskType::Unknown
        }
    }

    fn get_agent_for_task(&self, task_type: TaskType) -> Option<&Box<dyn Agent>> {
        match task_type {
            TaskType::Research => self.sub_agents.get("researcher"),
            TaskType::Engineering => self.sub_agents.get("engineer"),
            TaskType::Planning => self.sub_agents.get("planner"),
            TaskType::Coordination | TaskType::Unknown => None,
        }
    }

    async fn execute_tool_calls(&self, tool_calls: Vec<ToolCall>) -> Result<Vec<(String, String)>> {
        let mut results = Vec::new();
        
        for tool_call in tool_calls {
            let function: FunctionCall = tool_call.function;
            let tool_name = function.name;
            let args = function.arguments;
            
            debug!("Executing tool: {}", tool_name);
            
            if let Some(tool) = self.tools.as_ref().and_then(|t: &Arc<ToolRegistry>| t.get(&tool_name)) {
                let params: Value = serde_json::from_str(&args).unwrap_or(Value::Null);
                match tool.execute(params).await {
                    Ok(result) => {
                        results.push((tool_call.id, result.to_string()));
                    }
                    Err(e) => {
                        warn!("Tool execution failed: {}", e);
                        results.push((tool_call.id, format!("Error: {}", e)));
                    }
                }
            } else {
                results.push((tool_call.id, format!("Tool not found: {}", tool_name)));
            }
        }
        
        Ok(results)
    }

    pub async fn process_with_routing(&self, messages: Vec<Message>, llm: &LLMClient) -> Result<String> {
        let tools = self.tools.as_ref().map(|t: &Arc<ToolRegistry>| {
            t.get_all_schemas().into_iter().map(|s| {
                Function {
                    type_field: "function".to_string(),
                    name: s.name,
                    description: s.description,
                    parameters: s.parameters,
                }
            }).collect()
        });

        let response = llm.chat_with_tools(messages.clone(), tools).await?;
        
        if let Ok(tool_calls) = serde_json::from_str::<Vec<ToolCall>>(&response) {
            if !tool_calls.is_empty() {
                info!("Executing {} tool calls", tool_calls.len());
                
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

    pub async fn coordinate_sub_agents(&self, task: &str, llm: &LLMClient) -> Result<String> {
        let task_type = self.route_task(task);
        
        if let Some(agent) = self.get_agent_for_task(task_type) {
            info!("Routing task to {:?} agent", task_type);
            
            let messages = vec![
                Message::system(format!("You are a {}. {}", agent.role(), agent.name())),
                Message::user(task.to_string()),
            ];
            
            return agent.process(messages, llm).await;
        }
        
        info!("No specific agent found, processing task directly");
        self.process_with_routing(
            vec![Message::user(task.to_string())], 
            llm
        ).await
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
        self.process_with_routing(messages, llm).await
    }

    async fn process_with_tools(
        &self, 
        messages: Vec<Message>, 
        llm: &LLMClient, 
        tools: Option<Arc<ToolRegistry>>
    ) -> Result<String> {
        self.process_with_routing(messages, llm).await
    }
}
