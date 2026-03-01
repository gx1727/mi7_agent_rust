use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub role: String,
    pub content: String,
    pub timestamp: i64,
}

pub struct ConversationHistory {
    messages: VecDeque<ConversationMessage>,
    max_messages: usize,
}

impl ConversationHistory {
    pub fn new(max_messages: usize) -> Self {
        Self {
            messages: VecDeque::with_capacity(max_messages),
            max_messages,
        }
    }

    pub fn add_message(&mut self, role: String, content: String) {
        let timestamp = chrono::Utc::now().timestamp();
        
        self.messages.push_back(ConversationMessage {
            role,
            content,
            timestamp,
        });

        // 保持最大消息数限制
        while self.messages.len() > self.max_messages {
            self.messages.pop_front();
        }
    }

    pub fn get_messages(&self) -> Vec<crate::llm::Message> {
        self.messages
            .iter()
            .map(|msg| crate::llm::Message {
                role: msg.role.clone(),
                content: msg.content.clone(),
            })
            .collect()
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

impl Default for ConversationHistory {
    fn default() -> Self {
        Self::new(10) // 默认保存最近 10 条消息
    }
}
