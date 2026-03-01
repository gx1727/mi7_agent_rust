# Changelog

## [0.2.0] - 2026-03-01

### Added

#### 记忆系统（短期记忆）
- ✅ 对话历史管理 (`ConversationHistory`)
- ✅ 自动保存最近 N 条消息（默认 10 条）
- ✅ 支持清除历史 (`clear` 命令)
- ✅ 支持查看历史 (`history` 命令)

#### 交互式对话模式
- ✅ 交互式 CLI 界面
- ✅ 多轮对话支持
- ✅ UTF-8 字符安全截断
- ✅ 友好的退出提示

#### 工具系统（基础架构）
- ✅ Tool trait 定义
- ✅ ToolRegistry 工具注册中心
- ✅ WeatherTool 示例工具
- ✅ 工具 Schema 定义

### Changed
- 改进 CLI 参数解析（prompt 改为可选）
- 添加 `--max-history` 参数
- 优化错误处理

### Fixed
- 修复 UTF-8 字符截断 bug

---

## [0.1.0] - 2026-03-01

### Added
- 基础项目结构
- LLM 客户端（DeepSeek API）
- 流式输出支持
- Agent trait 系统
- 4 个基础 Agent（Manager, Researcher, Engineer, Planner）
- 配置管理
- GitHub 仓库

---

## Roadmap

### 短期（1-2周）
- [ ] 工具调用集成（Function Calling）
- [ ] Agent 协作路由
- [ ] 更丰富的 CLI（颜色、表格等）
- [ ] 任务规划器
- [ ] 技能系统扩展

### 中期（1-2月）
- [ ] 向量数据库集成（Qdrant/Chroma）
- [ ] 长期记忆系统
- [ ] RAG 检索增强
- [ ] 多 LLM Provider 支持
- [ ] 配置文件持久化

### 长期（3-6月）
- [ ] Web UI
- [ ] gRPC API
- [ ] 多用户支持
- [ ] 插件系统
- [ ] 部署工具

---

*最后更新：2026-03-01*
