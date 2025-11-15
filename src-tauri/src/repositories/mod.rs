// Repository数据访问层模块
//
// 提供统一的数据库访问抽象层，支持CRUD操作、分页、搜索等功能
// 透明处理加密数据的解密/加密操作

pub mod agent_guide_repository;
pub mod base_repository;
pub mod claude_provider_repository;
pub mod codex_provider_repository;
pub mod common_config_repository;
pub mod mcp_server_repository;

// 重新导出主要组件
pub use agent_guide_repository::AgentGuideRepository;
pub use base_repository::{BaseRepository, RepositoryResult};
pub use claude_provider_repository::ClaudeProviderRepository;
pub use codex_provider_repository::CodexProviderRepository;
pub use common_config_repository::CommonConfigRepository;
pub use mcp_server_repository::McpServerRepository;
