mod agents;
mod config;
mod llm;
mod memory;

use anyhow::Result;
use clap::Parser;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::agents::{Agent, ManagerAgent};
use crate::config::Config;
use crate::llm::{LLMClient, Message};
use crate::memory::ConversationHistory;

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
    
    // Create manager agent
    let manager = ManagerAgent::new();
    
    // Create conversation history
    let mut history = ConversationHistory::new(Args::parse().max_history);
    
    info!("Agent ready: {}", manager.name());
    
    // Parse CLI args
    let args = Args::parse();
    
    // Single prompt mode
    if let Some(prompt) = args.prompt {
        process_single_prompt(&llm_client, &manager, &mut history, prompt, args.stream).await?;
        return Ok(());
    }
    
    // Interactive mode
    run_interactive_mode(&llm_client, &manager, &mut history, args.stream).await?;
    
    Ok(())
}

async fn process_single_prompt(
    llm_client: &LLMClient,
    agent: &ManagerAgent,
    history: &mut ConversationHistory,
    prompt: String,
    stream: bool,
) -> Result<()> {
    // Add user message to history
    history.add_message("user".to_string(), prompt.clone());
    
    // Get all messages including history
    let messages = history.get_messages();
    
    // Process
    if stream {
        llm_client.chat_stream(messages).await?;
    } else {
        let response = agent.process(messages, llm_client).await?;
        println!("{}", response);
        
        // Add assistant response to history
        history.add_message("assistant".to_string(), response);
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
    stream: bool,
) -> Result<()> {
    use std::io::{self, Write};
    
    println!();
    println!("🤖 MI7 Agent Rust - Interactive Mode");
    println!("Type your message and press Enter. Type 'exit' or 'quit' to exit.");
    println!("Commands: clear (clear history), history (show history)");
    println!();
    
    loop {
        // Print prompt
        print!("You: ");
        io::stdout().flush()?;
        
        // Read input
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_string();
        
        // Check for exit commands
        if input == "exit" || input == "quit" {
            println!("Goodbye! 👋");
            break;
        }
        
        // Check for special commands
        if input == "clear" {
            history.clear();
            println!("✅ Conversation history cleared");
            continue;
        }
        
        if input == "history" {
            println!("📝 Conversation history ({} messages):", history.len());
            for (i, msg) in history.get_messages().iter().enumerate() {
                let content = truncate_string(&msg.content, 50);
                println!("  [{}] {}: {}", i + 1, msg.role, content);
            }
            continue;
        }
        
        // Skip empty input
        if input.is_empty() {
            continue;
        }
        
        // Add user message to history
        history.add_message("user".to_string(), input.clone());
        
        // Get all messages
        let messages = history.get_messages();
        
        // Print agent response
        print!("Agent: ");
        io::stdout().flush()?;
        
        // Process
        if stream {
            llm_client.chat_stream(messages).await?;
        } else {
            let response = agent.process(messages, llm_client).await?;
            println!("{}", response);
            
            // Add to history
            history.add_message("assistant".to_string(), response);
        }
        
        println!();
    }
    
    Ok(())
}
