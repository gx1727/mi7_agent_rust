//! 长期记忆存储 - SQLite 实现
//! 
//! 使用 SQLite 作为持久化存储，支持多会话对话历史

use rusqlite::{Connection, params, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use anyhow::Result;

/// 对话消息（持久化版本）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMessage {
    pub id: i64,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub timestamp: i64,
}

/// 会话信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub message_count: i64,
}

/// SQLite 记忆存储
pub struct MemoryStore {
    conn: Mutex<Connection>,
}

impl MemoryStore {
    /// 创建新的记忆存储
    pub fn new(db_path: Option<PathBuf>) -> Result<Self> {
        let path = db_path.unwrap_or_else(|| {
            dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("mi7_agent_rust")
                .join("memory.db")
        });
        
        // 确保目录存在
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let conn = Connection::open(&path)?;
        
        // 初始化数据库表
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                message_count INTEGER DEFAULT 0
            );
            
            CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                FOREIGN KEY (session_id) REFERENCES sessions(id)
            );
            
            CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id);
            CREATE INDEX IF NOT EXISTS idx_messages_timestamp ON messages(timestamp);
            "
        )?;
        
        tracing::info!(path = ?path, "Memory store initialized");
        
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
    
    /// 创建新会话
    pub fn create_session(&self, session_id: &str, name: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();
        
        conn.execute(
            "INSERT INTO sessions (id, name, created_at, updated_at, message_count) 
             VALUES (?1, ?2, ?3, ?4, 0)",
            params![session_id, name, now, now],
        )?;
        
        tracing::info!(session_id = session_id, name = name, "Session created");
        Ok(())
    }
    
    /// 保存消息
    pub fn save_message(&self, session_id: &str, role: &str, content: &str) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();
        
        conn.execute(
            "INSERT INTO messages (session_id, role, content, timestamp) 
             VALUES (?1, ?2, ?3, ?4)",
            params![session_id, role, content, now],
        )?;
        
        let message_id = conn.last_insert_rowid();
        
        // 更新会话消息计数
        conn.execute(
            "UPDATE sessions SET updated_at = ?1, message_count = message_count + 1 
             WHERE id = ?2",
            params![now, session_id],
        )?;
        
        Ok(message_id)
    }
    
    /// 获取会话消息
    pub fn get_messages(&self, session_id: &str, limit: usize) -> Result<Vec<StoredMessage>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, session_id, role, content, timestamp 
             FROM messages 
             WHERE session_id = ?1 
             ORDER BY timestamp ASC 
             LIMIT ?2"
        )?;
        
        let messages = stmt.query_map(params![session_id, limit], |row| {
            Ok(StoredMessage {
                id: row.get(0)?,
                session_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                timestamp: row.get(4)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
        
        Ok(messages)
    }
    
    /// 获取所有会话
    pub fn get_sessions(&self) -> Result<Vec<Session>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, name, created_at, updated_at, message_count 
             FROM sessions 
             ORDER BY updated_at DESC"
        )?;
        
        let sessions = stmt.query_map([], |row| {
            Ok(Session {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
                message_count: row.get(4)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
        
        Ok(sessions)
    }
    
    /// 删除会话
    pub fn delete_session(&self, session_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute("DELETE FROM messages WHERE session_id = ?1", params![session_id])?;
        conn.execute("DELETE FROM sessions WHERE id = ?1", params![session_id])?;
        
        tracing::info!(session_id = session_id, "Session deleted");
        Ok(())
    }
    
    /// 清理旧消息（保留最近 N 条）
    pub fn cleanup_old_messages(&self, session_id: &str, keep_count: usize) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        
        // 获取要保留的消息 ID 范围
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM messages WHERE session_id = ?1",
            params![session_id],
            |row| row.get(0),
        )?;
        
        if count as usize <= keep_count {
            return Ok(0);
        }
        
        let delete_count = count as usize - keep_count;
        
        // 删除最旧的消息
        conn.execute(
            "DELETE FROM messages WHERE id IN (
                SELECT id FROM messages 
                WHERE session_id = ?1 
                ORDER BY timestamp ASC 
                LIMIT ?2
            )",
            params![session_id, delete_count],
        )?;
        
        tracing::info!(session_id = session_id, deleted = delete_count, "Old messages cleaned up");
        Ok(delete_count)
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new(None).expect("Failed to create memory store")
    }
}
