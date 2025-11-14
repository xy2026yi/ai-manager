# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

这是一个从 Python/FastAPI 到 Rust/Tauri 的 AI Manager 迁移项目。项目旨在将现有的 Python AI 管理工具迁移为轻量级、高性能的桌面应用，保持所有功能完整性和数据兼容性。

**技术栈**:
- 后端: Rust + Tauri 1.5 + sqlx + fernet
- 前端: React 18 + TypeScript + Vite + Tailwind CSS + Jotai
- 数据库: SQLite（与原项目保持一致的 schema）

## 常用开发命令

### 项目初始化（项目开始时执行）
```bash
# 创建 Tauri 项目（尚未执行）
npm create tauri-app@latest

# 安装前端依赖（计划中）
npm install jotai react-router-dom react-hook-form @headlessui/react @heroicons/react @tauri-apps/api

# 配置 Tailwind CSS（计划中）
npm install -D tailwindcss postcss autoprefixer
npx tailwindcss init -p
```

### 开发命令
```bash
# 启动开发服务器（目标命令）
npm run tauri dev

# Rust 编译检查
cargo check

# Rust 构建
cargo build

# Rust 发布构建
cargo build --release
```

### 测试命令
```bash
# 运行 Rust 测试
cargo test

# 运行特定模块测试
cargo test crypto
cargo test providers

# 测试覆盖率（需要安装 tarpaulin）
cargo install cargo-tarpaulin
cargo tarpaulin --out html
```

### 数据库操作
```bash
# 安装 sqlx-cli（一次性）
cargo install sqlx-cli --no-default-features --features sqlite

# 创建数据库迁移
sqlx migrate add <migration_name>

# 运行迁移（从 src-tauri/ 目录）
sqlx migrate run --database-url sqlite:ai_manager.db

# 准备离线模式（避免在线检查）
export SQLX_OFFLINE=true
```

### 构建和打包
```bash
# 构建生产版本
npm run tauri build

# 检查打包结果
ls -la src-tauri/target/release/bundle/
```

## 项目架构

### 整体架构
```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   React 前端    │ ←→  │   Tauri Bridge  │ ←→  │   Rust 后端     │
│                 │     │                 │     │                 │
│ • UI 组件       │     │ • Commands      │     │ • Services      │
│ • 状态管理      │     │ • IPC 通信       │     │ • 数据模型      │
│ • 路由          │     │                 │     │ • 加密服务      │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                                                       ↓
                                              ┌─────────────────┐
                                              │  SQLite 数据库   │
                                              │                 │
                                              │ • 5个核心表      │
                                              │ • 加密数据       │
                                              │ • 索引优化      │
                                              └─────────────────┘
```

### 核心数据模型
项目使用 5 个核心数据表，与原 Python 项目保持一致：

1. **claude_providers**: Claude 供应商配置（11个字段）
2. **codex_providers**: Codex 供应商配置（5个字段）
3. **agent_guides**: Agent 指导文件（4个字段）
4. **mcp_servers**: MCP 服务器配置（7个字段）
5. **common_configs**: 通用配置（6个字段）

### 关键业务逻辑

#### 供应商唯一性原则
- 任何时刻只能有一个启用的供应商
- 启用新供应商时自动禁用所有其他供应商
- 确保配置文件的一致性

#### 配置文件自动生成
- **Claude**: `~/.claude/settings.json`
- **Codex**: `~/.codex/auth.json` 和 `~/.codex/config.toml`
- 使用 Handlebars 模板引擎渲染配置

#### 加密兼容性
- 使用 fernet 对称加密，与原 Python 项目完全兼容
- 密钥格式：Base64 编码的 32 字节
- 支持从原数据库无缝迁移加密数据

## 开发工作流

### 1. 任务管理
项目使用 shrimp-task-manager 进行任务管理：

```bash
# 查看所有任务
<mcp_commands>查看任务列表</mcp_commands>

# 执行特定任务（需要 task_id）
<mcp_commands>执行任务</mcp_commands>

# 任务规划
<mcp_commands>规划任务</mcp_commands>
```

### 2. 开发阶段
项目分为 6 个主要阶段：
1. **阶段1**: 项目初始化（3个任务）
2. **阶段2**: 数据层迁移（5个任务）
3. **阶段3**: 业务逻辑层（7个任务）
4. **阶段4**: 前端开发（8个任务）
5. **阶段5**: 集成测试（2个任务）
6. **阶段6**: 优化和部署（5个任务）

### 3. 代码规范
- **强制使用中文注释**: 所有代码注释、文档、提交信息必须使用简体中文
- **类型安全**: Rust 使用编译时检查，TypeScript 提供前端类型安全
- **测试驱动**: 每个模块都需要对应的单元测试
- **性能优先**: 目标启动时间 < 2s，内存占用 < 100MB

## 关键技术点

### 1. 数据库迁移
使用 sqlx 进行类型安全的数据库操作：
- 编译时 SQL 检查
- 自动迁移管理
- 连接池优化

### 2. 加密服务
实现 CryptoManager 确保与 Python 的兼容性：
```rust
// 核心加密接口
impl CryptoManager {
    pub fn new(key: &str) -> Result<Self, String>
    pub fn encrypt(&self, plaintext: &str) -> String
    pub fn decrypt(&self, ciphertext: &str) -> Result<String, String>
}
```

### 3. Tauri Commands
前后端通过 Tauri Commands 通信：
```rust
#[tauri::command]
async fn get_claude_providers(
    service: State<'_, ProviderService>
) -> Result<Vec<ClaudeProvider>, AppError>
```

### 4. 前端状态管理
使用 Jotai 进行原子化状态管理：
- providersAtom: 供应商列表状态
- selectedProviderAtom: 选中供应商状态
- loadingAtom: 加载状态
- errorAtom: 错误状态

## 性能目标

- **应用体积**: < 15MB（相比 Python/Electron 方案减小 90%+）
- **启动时间**: < 2s
- **内存占用**: < 100MB
- **操作响应**: < 500ms

## 数据兼容性

项目必须保持与原 Python 项目的完全兼容：
- 数据库 schema 完全一致
- 加密数据可以无缝迁移
- 配置文件格式保持一致
- 支持一键数据迁移工具

## 测试策略

- **单元测试**: 覆盖率 > 80%
- **集成测试**: 完整用户流程验证
- **兼容性测试**: 与原项目的数据交换测试
- **性能测试**: 启动时间、内存占用、响应时间

## 文档要求

所有文档和注释必须使用简体中文：
- 代码注释：描述意图和约束
- API 文档：完整的接口说明
- 用户文档：安装、使用、迁移指南
- 技术文档：架构设计、开发指南

## 开发注意事项

1. **安全优先**: 加密密钥从环境变量读取，不在代码中硬编码
2. **错误处理**: 提供用户友好的中文错误信息
3. **并发安全**: Rust 的所有权系统确保线程安全
4. **资源管理**: 使用 RAII 模式，避免资源泄漏
5. **跨平台**: 确保在 Windows/macOS/Linux 上都能正常运行

## 相关资源

- **原项目**: `/Git/project/ai-manager`
- **Tauri 文档**: https://tauri.app/v1/guides/
- **sqlx 文档**: https://github.com/launchbadge/sqlx
- **详细迁移计划**: `docs/migration-plan.md`
- **任务列表**: 通过 shrimp-task-manager 查看