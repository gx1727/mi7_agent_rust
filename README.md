# MI7 Agent Rust

<div align="center">

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![DeepSeek](https://img.shields.io/badge/DeepSeek-4285F4?style=for-the-badge&logo=google&logoColor=white)
![Tokio](https://img.shields.io/badge/Tokio-FF5733?style=for-the-badge&logo=rust&logoColor=white)

[![GitHub stars](https://img.shields.io/github/stars/gx1727/mi7_agent_rust?style=social)](https://github.com/gx1727/mi7_agent_rust/stargazers)
[![GitHub forks](https://img.shields.io/github/forks/gx1727/mi7_agent_rust?style=social)](https://github.com/gx1727/mi7_agent_rust/network/members)
[![GitHub issues](https://img.shields.io/github/issues/gx1727/mi7_agent_rust)](https://github.com/gx1727/mi7_agent_rust/issues)
[![GitHub license](https://img.shields.io/github/license/gx1727/mi7_agent_rust)](https://github.com/gx1727/mi7_agent_rust/blob/master/LICENSE)

**🦀 Rust 实现的高性能多 Agent AI 系统**

*High-performance multi-agent AI system built with Rust*

[English](#english) | [中文](#中文)

</div>

---

## 中文

### ✨ 特性

- 🦀 **高性能** - Rust 原生性能，启动时间 <100ms
- 🤖 **多 Agent 系统** - Manager, Researcher, Engineer, Planner 四种专家
- 🌊 **流式输出** - 实时响应流，无延迟体验
- 🔧 **易扩展** - 清晰的 Agent trait，易于添加新功能
- 📝 **类型安全** - Rust 编译时类型检查
- 🚀 **单二进制** - 5.1MB 可执行文件，零依赖部署

### 📊 性能对比

| 指标 | Python mi7_agent | Rust mi7_agent_rust | 提升 |
|------|------------------|---------------------|------|
| 启动时间 | ~1s | <100ms | **10x** ⚡ |
| 内存占用 | ~100MB | ~20MB | **5x** 💾 |
| 二进制大小 | N/A | 5.1MB | ✅ |
| 部署复杂度 | 高 | 低（单二进制） | ✅ |

### 🏗️ 架构

```
mi7_agent_rust/
├── src/
│   ├── agents/          # Agent 实现
│   │   ├── base.rs      # Agent trait
│   │   ├── manager.rs   # 协调者
│   │   ├── researcher.rs # 研究员
│   │   ├── engineer.rs  # 工程师
│   │   └── planner.rs   # 规划师
│   ├── llm/             # LLM 客户端
│   │   ├── client.rs    # HTTP 客户端
│   │   └── types.rs     # 类型定义
│   ├── config/          # 配置管理
│   ├── tools/           # 工具系统（待实现）
│   ├── memory/          # 记忆系统（待实现）
│   └── main.rs          # 主入口
└── Cargo.toml
```

### 🚀 快速开始

#### 1. 克隆仓库

```bash
git clone https://github.com/gx1727/mi7_agent_rust.git
cd mi7_agent_rust
```

#### 2. 配置 API Key

```bash
# 复制配置模板
cp .env.example .env

# 编辑 .env 文件，填入你的 DeepSeek API Key
nano .env
```

**DeepSeek 配置（推荐，国内稳定）：**
```env
LLM_PROVIDER=deepseek
LLM_API_KEY=your_deepseek_api_key_here
LLM_MODEL=deepseek-chat
LLM_BASE_URL=https://api.deepseek.com
LLM_MAX_TOKENS=1000
LLM_TEMPERATURE=0.7
```

**获取 DeepSeek API Key：**
- 访问：https://platform.deepseek.com/
- 注册账号并获取 API Key

#### 3. 构建

```bash
cargo build --release
```

#### 4. 运行

```bash
# 基本用法
./target/release/mi7-agent --prompt "你好，请介绍一下自己"

# 流式输出
./target/release/mi7-agent --prompt "请用三句话介绍 Rust" --stream

# 查看帮助
./target/release/mi7-agent --help
```

### 📦 依赖

**核心依赖：**
- **tokio** - 异步运行时
- **reqwest** - HTTP 客户端（支持流式）
- **serde** - 序列化框架
- **clap** - CLI 参数解析
- **tracing** - 日志系统
- **anyhow** - 错误处理
- **async-trait** - 异步 trait

### 🎯 开发状态

- ✅ **已完成**
  - 基础 LLM 客户端
  - DeepSeek API 集成
  - 流式输出支持
  - Agent trait 系统
  - CLI 参数解析
  - 配置管理

- 🔧 **进行中**
  - 工具调用（Function Calling）
  - 对话历史管理
  - 多轮对话

- 📋 **计划中**
  - 向量存储（Qdrant/Chroma）
  - Agent 路由逻辑
  - 技能系统
  - Web UI

### 📝 配置说明

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `LLM_PROVIDER` | LLM 提供商（deepseek/glm） | `deepseek` |
| `LLM_API_KEY` | API 密钥 | **必填** |
| `LLM_MODEL` | 模型名称 | `deepseek-chat` |
| `LLM_BASE_URL` | API 基础 URL | `https://api.deepseek.com` |
| `LLM_MAX_TOKENS` | 最大 Token 数 | `1000` |
| `LLM_TEMPERATURE` | 温度参数 | `0.7` |

### 🧪 测试

查看详细测试报告：[TEST_REPORT.md](./TEST_REPORT.md)

**测试评分：** ⭐⭐⭐⭐☆ (4.5/5)

### 🤝 贡献

欢迎贡献！请查看 [Issues](https://github.com/gx1727/mi7_agent_rust/issues) 页面。

### 📄 许可证

MIT License

### 👤 作者

**gx1727**
- GitHub: [@gx1727](https://github.com/gx1727)

---

## English

### ✨ Features

- 🦀 **High Performance** - Rust native performance, <100ms startup
- 🤖 **Multi-Agent System** - Manager, Researcher, Engineer, Planner agents
- 🌊 **Streaming Support** - Real-time response streaming
- 🔧 **Extensible** - Clear Agent trait, easy to extend
- 📝 **Type Safe** - Rust compile-time type checking
- 🚀 **Single Binary** - 5.1MB executable, zero dependencies

### 🚀 Quick Start

```bash
# Clone
git clone https://github.com/gx1727/mi7_agent_rust.git
cd mi7_agent_rust

# Configure
cp .env.example .env
# Edit .env with your DeepSeek API key

# Build
cargo build --release

# Run
./target/release/mi7-agent --prompt "Hello, introduce yourself"
```

### 📊 Performance

- **Startup:** <100ms
- **Memory:** ~20MB
- **Binary Size:** 5.1MB
- **Response Time:** ~2s

### 🎯 Roadmap

- ✅ LLM Client (DeepSeek)
- ✅ Streaming Output
- 🔧 Function Calling
- 📋 Vector Memory
- 📋 Agent Routing

### 📄 License

MIT

---

<div align="center">

**⭐ 如果这个项目对你有帮助，请给一个 Star！**

Made with ❤️ by gx1727

</div>
