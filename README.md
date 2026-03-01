# MI7 Agent Rust

A high-performance multi-agent AI system built with Rust.

## Features

- 🦀 **High Performance** - Rust native performance
- 🤖 **Multi-Agent System** - Manager, Researcher, Engineer, Planner agents
- 🌊 **Streaming Support** - Real-time response streaming
- 🔧 **Extensible** - Easy to add new agents and tools
- 📝 **Type Safe** - Full type safety with Rust's type system

## Architecture

```
mi7_agent_rust/
├── src/
│   ├── agents/          # Agent implementations
│   │   ├── base.rs      # Agent trait
│   │   ├── manager.rs   # Manager agent
│   │   ├── researcher.rs
│   │   ├── engineer.rs
│   │   └── planner.rs
│   ├── llm/             # LLM client
│   │   ├── client.rs
│   │   └── types.rs
│   ├── tools/           # Tool system
│   ├── memory/          # Memory management
│   ├── config/          # Configuration
│   └── main.rs
└── Cargo.toml
```

## Quick Start

### 1. Setup

```bash
cd /root/work/mi7_agent_rust

# Create .env file
cat > .env << 'ENVEOF'
LLM_PROVIDER=glm
LLM_API_KEY=your_api_key_here
LLM_MODEL=glm-4
LLM_MAX_TOKENS=1000
LLM_TEMPERATURE=0.7
ENVEOF
```

### 2. Build

```bash
cargo build --release
```

### 3. Run

```bash
# Basic usage
./target/release/mi7-agent --prompt "What is Rust?"

# With streaming
./target/release/mi7-agent --prompt "Explain async/await" --stream
```

## Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `LLM_PROVIDER` | LLM provider (glm/minimax) | `glm` |
| `LLM_API_KEY` | API key | Required |
| `LLM_MODEL` | Model name | `glm-4` |
| `LLM_MAX_TOKENS` | Max tokens | `1000` |
| `LLM_TEMPERATURE` | Temperature | `0.7` |

## Dependencies

- **tokio** - Async runtime
- **reqwest** - HTTP client
- **serde** - Serialization
- **clap** - CLI parser
- **tracing** - Logging

## Development Status

- ✅ Basic LLM client
- ✅ Streaming support
- ✅ Agent trait system
- 🔧 Tool calling (WIP)
- 🔧 Vector memory (WIP)

## License

MIT

## Author

gx1727

---

**Note:** This is a Rust implementation of the Python mi7_agent project.
