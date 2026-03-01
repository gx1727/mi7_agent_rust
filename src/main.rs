mod agents;
mod config;
mod llm;

use anyhow::Result;
use clap::Parser;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::agents::{Agent, ManagerAgent};
use crate::config::Config;
use crate::llm::{LLMClient, Message};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input prompt
    #[arg(short, long)]
    prompt: String,
    
    /// Enable streaming output
    #[arg(short, long)]
    stream: bool,
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
    
    // Parse CLI args
    let args = Args::parse();
    
    // Create manager agent
    let manager = ManagerAgent::new();
    
    info!("Processing with agent: {}", manager.name());
    
    // Create message
    let messages = vec![Message {
        role: "user".to_string(),
        content: args.prompt,
    }];
    
    // Process
    if args.stream {
        llm_client.chat_stream(messages).await?;
    } else {
        let response = manager.process(messages, &llm_client).await?;
        println!("{}", response);
    }
    
    Ok(())
}
