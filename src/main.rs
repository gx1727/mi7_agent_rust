mod agents;
mod config;
mod error;
mod llm;
mod memory;
mod tools;

use std::sync::Arc;
use anyhow::Result;
use clap::Parser;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::agents::{Agent, ManagerAgent};
use crate::config::Config;
use crate::llm::{LLMClient, Message};
use crate::memory::{ConversationHistory, MemoryStore};
use crate::tools::{ToolRegistry, FileReadTool, FileWriteTool, ExecuteCommandTool, HttpRequestTool};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input prompt (optional, will enter interactive mode if not provided)
    #[arg(short, long)]
    prompt: Option<String>,
    
    /// Enable streaming output
    #[arg(short, long)]
    stream: bool,
    
    /// Maximum conversation history
    #[arg(long, default_value = "10")]
    max_history: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Load config
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;
    
    info!("MI7 Agent Rust v0.1.0 starting...");
    info!("LLM Provider: {}", config.llm_provider);
    
    // Create LLM client
    let llm_client = LLMClient::new(config);
    
    // Initialize tools
    let registry = init_tools();
    info!("Tools registered: {:?}", registry.list_tools());
    
    // Create manager agent with tools
    let manager = ManagerAgent::new().with_tools(Arc::new(registry));
    
    // Create conversation history
    let mut history = ConversationHistory::new(Args::parse().max_history);
    
    // Initialize memory store (SQLite)
    let memory_store = MemoryStore::new(None)?;
    let session_id = "default";
    memory_store.create_session(session_id, "default")?;
    info!("Memory store initialized");
    
    info!("Agent ready: {}", manager.name());
    
    // Parse CLI args
    let args = Args::parse();
    
    // Single prompt mode
    if let Some(prompt) = args.prompt {
        process_single_prompt(&llm_client, &manager, &mut history, &memory_store, session_id, prompt, args.stream).await?;
        return Ok(());
    }
    
    // Interactive mode
    run_interactive_mode(&llm_client, &manager, &mut history, &memory_store, session_id, args.stream).await?;
    
    Ok(())
}

/// Initialize tool registry
fn init_tools() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    
    // File tools (allowed /root/work)
    registry.register(Arc::new(FileReadTool::new(vec![
        "/root/work".to_string(),
    ])));
    registry.register(Arc::new(FileWriteTool::new(vec![
        "/root/work".to_string(),
    ])));
    
    // Command tool (restricted)
    registry.register(Arc::new(ExecuteCommandTool::new(
        vec!["ls".to_string(), "cat".to_string(), "grep".to_string(), "find".to_string()],
        30, // 30s timeout
    )));
    
    // HTTP Request tool (需要手动配置 token)
    // 从环境变量读取 token
    let token = std::env::var("HTTP_TOOL_TOKEN").ok();
    if let Some(t) = token {
        registry.register(Arc::new(HttpRequestTool::with_token(t)));
    }
    
    registry
}

async fn process_single_prompt(
    llm_client: &LLMClient,
    agent: &ManagerAgent,
    history: &mut ConversationHistory,
    memory_store: &MemoryStore,
    session_id: &str,
    prompt: String,
    stream: bool,
) -> Result<()> {
    // Add user message to history
    history.add_message("user".to_string(), prompt.clone());
    
    // Save to SQLite
    memory_store.save_message(session_id, "user", &prompt)?;
    
    // Get all messages including history
    let messages = history.get_messages();
    
    // Process
    if stream {
        llm_client.chat_stream(messages).await?;
    } else {
        let response = agent.process(messages, llm_client).await?;
        println!("{}", response);
        
        // Add assistant response to history
        history.add_message("assistant".to_string(), response.clone());
        
        // Save to SQLite
        memory_store.save_message(session_id, "assistant", &response)?;
    }
    
    Ok(())
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        // 找到字符边界
        let mut end = max_len;
        while !s.is_char_boundary(end) && end > 0 {
            end -= 1;
        }
        format!("{}...", &s[..end])
    }
}

async fn run_interactive_mode(
    llm_client: &LLMClient,
    agent: &ManagerAgent,
    history: &mut ConversationHistory,
    memory_store: &MemoryStore,
    session_id: &str,
    stream: bool,
) -> Result<()> {
    use std::io::{self, Write};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::signal::ctrl_c;
    
    // 捕获 Ctrl+C 信号
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    
    tokio::spawn(async move {
        ctrl_c().await.ok();
        println!("\n⚠️ 收到 Ctrl+C，继续运行，输入 'quit' 退出");
        r.store(false, Ordering::SeqCst);
    });
    
    println!();
    println!("🤖 MI7 Agent Rust - 交互模式");
    println!("输入内容后按回车发送。输入 'exit' 或 'quit' 退出。");
    println!("命令: clear(清除历史), history(查看历史), memory(查看存储)");
    println!();
    
    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();
    
    while running.load(Ordering::SeqCst) {
        // Print prompt
        print!("你: ");
        io::stdout().flush()?;
        
        // Read input (async)
        let input = match lines.next_line().await {
            Ok(Some(line)) => line,
            Ok(None) => {
                println!("\n输入已关闭，继续运行，输入 'quit' 退出");
                continue;
            }
            Err(_) => {
                println!("\n读取错误，继续运行，输入 'quit' 退出");
                continue;
            }
        };
        let input = input.trim().to_string();
        
        // Check for exit commands
        if input == "exit" || input == "quit" {
            println!("再见! 👋");
            break;
        }
        
        // Check for special commands
        if input == "clear" {
            history.clear();
            println!("✅ 已清除会话历史");
            continue;
        }
        
        if input == "history" {
            println!("📝 会话历史 ({} 条消息):", history.len());
            for (i, msg) in history.get_messages().iter().enumerate() {
                let content = truncate_string(&msg.content, 50);
                println!("  [{}] {}: {}", i + 1, msg.role, content);
            }
            continue;
        }
        
        if input == "memory" {
            println!("💾 SQLite 存储 ({} 条消息):", history.len());
            if let Ok(messages) = memory_store.get_messages(session_id, 10) {
                for (i, msg) in messages.iter().enumerate() {
                    let content = truncate_string(&msg.content, 50);
                    println!("  [{}] {}: {}", i + 1, msg.role, content);
                }
            }
            continue;
        }
        
        // Skip empty input
        if input.is_empty() {
            continue;
        }
        
        // Add user message to history
        history.add_message("user".to_string(), input.clone());
        
        // Save to SQLite
        let _ = memory_store.save_message(session_id, "user", &input);
        
        // Get all messages
        let messages = history.get_messages();
        
        // Print agent response
        print!("助手: ");
        io::stdout().flush()?;
        
        // Process
        if stream {
            llm_client.chat_stream(messages).await?;
        } else {
            let response = agent.process(messages, llm_client).await?;
            println!("{}", response);
            
            // Add to history
            history.add_message("assistant".to_string(), response.clone());
            
            // Save to SQLite
            let _ = memory_store.save_message(session_id, "assistant", &response);
        }
        
        println!();
    }
    
    Ok(())
}
