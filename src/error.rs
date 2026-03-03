use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock;
use reqwest::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("API error: {0}")]
    Api(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("Tool execution error: {0}")]
    ToolExecution(String),
    
    #[error("Agent routing error: {0}")]
    Routing(String),
    
    #[error("Task timeout: {0}")]
    Timeout(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl AgentError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            AgentError::Network(_) | AgentError::RateLimit(_) | AgentError::Timeout(_)
        )
    }
    
    pub fn is_api_error(&self) -> bool {
        matches!(self, AgentError::Api(_) | AgentError::RateLimit(_))
    }
    
    pub fn is_network_error(&self) -> bool {
        matches!(self, AgentError::Network(_) | AgentError::Timeout(_))
    }
}

pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
        }
    }
}

pub struct RetryState {
    pub attempts: u32,
    pub last_error: Option<Arc<AgentError>>,
}

impl RetryState {
    pub fn new() -> Self {
        Self { attempts: 0, last_error: None }
    }
    
    pub fn should_retry(&self, config: &RetryConfig) -> bool {
        self.attempts < config.max_retries
    }
    
    pub fn next_delay(&self, config: &RetryConfig) -> u64 {
        let delay = (config.initial_delay_ms as f64 * config.backoff_multiplier.powi(self.attempts as i32)) as u64;
        delay.min(config.max_delay_ms)
    }
    
    pub fn record_attempt(&mut self, error: Arc<AgentError>) {
        self.attempts += 1;
        self.last_error = Some(error);
    }
}

impl Default for RetryState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn classify_error(status: StatusCode, message: &str) -> AgentError {
    if status == StatusCode::TOO_MANY_REQUESTS {
        AgentError::RateLimit(message.to_string())
    } else if status.is_server_error() {
        AgentError::Api(format!("Server error {}: {}", status, message))
    } else if status.is_client_error() {
        AgentError::InvalidRequest(format!("Client error {}: {}", status, message))
    } else {
        AgentError::Api(message.to_string())
    }
}
