use serde::{Deserialize, Serialize};
use sqlx::FromRow;
// chrono 在将来的时间处理功能中会用到

// Claude供应商数据模型
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ClaudeProvider {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub token: String, // 加密存储
    pub timeout: Option<i64>,
    pub auto_update: Option<i64>, // 1-禁用遥测，0-启用遥测
    pub r#type: String, // paid 或 public_welfare
    pub enabled: i64, // 0-未启用，1-启用
    pub opus_model: Option<String>,
    pub sonnet_model: Option<String>,
    pub haiku_model: Option<String>,
    pub created_at: Option<String>, // ISO 8601 字符串
    pub updated_at: Option<String>, // ISO 8601 字符串
}

// 创建Claude供应商的请求结构
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateClaudeProviderRequest {
    pub name: String,
    pub url: String,
    pub token: String,
    pub timeout: Option<i64>,
    pub auto_update: Option<i64>,
    pub r#type: Option<String>,
    pub opus_model: Option<String>,
    pub sonnet_model: Option<String>,
    pub haiku_model: Option<String>,
}

// 更新Claude供应商的请求结构
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateClaudeProviderRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub token: Option<String>,
    pub timeout: Option<i64>,
    pub auto_update: Option<i64>,
    pub r#type: Option<String>,
    pub enabled: Option<i64>,
    pub opus_model: Option<String>,
    pub sonnet_model: Option<String>,
    pub haiku_model: Option<String>,
}

// Codex供应商数据模型
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CodexProvider {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub token: String, // 加密存储
    pub r#type: String, // paid 或 public_welfare
    pub enabled: i64, // 0-未启用，1-启用
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

// 创建Codex供应商的请求结构
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCodexProviderRequest {
    pub name: String,
    pub url: String,
    pub token: String,
    pub r#type: Option<String>,
}

// 更新Codex供应商的请求结构
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCodexProviderRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub token: Option<String>,
    pub r#type: Option<String>,
    pub enabled: Option<i64>,
}

// Agent指导文件数据模型
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AgentGuide {
    pub id: i64,
    pub name: String,
    pub r#type: String, // 'only' 或 'and'
    pub text: String, // 文件完整内容
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

// 创建Agent指导文件的请求结构
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAgentGuideRequest {
    pub name: String,
    pub r#type: String,
    pub text: String,
}

// 更新Agent指导文件的请求结构
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAgentGuideRequest {
    pub name: Option<String>,
    pub r#type: Option<String>,
    pub text: Option<String>,
}

// MCP服务器数据模型
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct McpServer {
    pub id: i64,
    pub name: String,
    pub r#type: Option<String>, // stdio, sse等
    pub timeout: Option<i64>, // 默认30000ms
    pub command: String, // 命令，如npx, uvx, python等
    pub args: String, // 命令参数，存储为JSON字符串
    pub env: Option<String>, // 环境变量，存储为JSON字符串
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

// 创建MCP服务器的请求结构
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMcpServerRequest {
    pub name: String,
    pub r#type: Option<String>,
    pub timeout: Option<i64>,
    pub command: String,
    pub args: Vec<String>,
    pub env: Option<std::collections::HashMap<String, String>>,
}

// 更新MCP服务器的请求结构
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateMcpServerRequest {
    pub name: Option<String>,
    pub r#type: Option<String>,
    pub timeout: Option<i64>,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<Option<std::collections::HashMap<String, String>>>,
}

// 通用配置数据模型
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CommonConfig {
    pub id: i64,
    pub key: String,
    pub value: String, // 支持环境变量替换，如 ${HOME}
    pub description: Option<String>,
    pub category: String, // 配置分类
    pub is_active: i64, // 是否启用：1-启用，0-禁用
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

// 创建通用配置的请求结构
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCommonConfigRequest {
    pub key: String,
    pub value: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub is_active: Option<i64>,
}

// 更新通用配置的请求结构
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCommonConfigRequest {
    pub key: Option<String>,
    pub value: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub is_active: Option<i64>,
}

// 数据库记录的公共trait
pub trait DbRecord {
    fn table_name() -> &'static str;
    fn id(&self) -> i64;
    fn created_at(&self) -> Option<&str>;
    fn updated_at(&self) -> Option<&str>;
}

// 为每个模型实现DbRecord trait
impl DbRecord for ClaudeProvider {
    fn table_name() -> &'static str { "claude_providers" }
    fn id(&self) -> i64 { self.id }
    fn created_at(&self) -> Option<&str> { self.created_at.as_deref() }
    fn updated_at(&self) -> Option<&str> { self.updated_at.as_deref() }
}

impl DbRecord for CodexProvider {
    fn table_name() -> &'static str { "codex_providers" }
    fn id(&self) -> i64 { self.id }
    fn created_at(&self) -> Option<&str> { self.created_at.as_deref() }
    fn updated_at(&self) -> Option<&str> { self.updated_at.as_deref() }
}

impl DbRecord for AgentGuide {
    fn table_name() -> &'static str { "agent_guides" }
    fn id(&self) -> i64 { self.id }
    fn created_at(&self) -> Option<&str> { self.created_at.as_deref() }
    fn updated_at(&self) -> Option<&str> { self.updated_at.as_deref() }
}

impl DbRecord for McpServer {
    fn table_name() -> &'static str { "mcp_servers" }
    fn id(&self) -> i64 { self.id }
    fn created_at(&self) -> Option<&str> { self.created_at.as_deref() }
    fn updated_at(&self) -> Option<&str> { self.updated_at.as_deref() }
}

impl DbRecord for CommonConfig {
    fn table_name() -> &'static str { "common_configs" }
    fn id(&self) -> i64 { self.id }
    fn created_at(&self) -> Option<&str> { self.created_at.as_deref() }
    fn updated_at(&self) -> Option<&str> { self.updated_at.as_deref() }
}

// 分页查询参数
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(20),
            offset: Some(0),
        }
    }
}

// 分页结果
#[derive(Debug, Serialize, Deserialize)]
pub struct PagedResult<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
    pub total_pages: i64,
}

impl<T> PagedResult<T> {
    pub fn new(data: Vec<T>, total: i64, page: i64, limit: i64) -> Self {
        let total_pages = (total + limit - 1) / limit;
        Self {
            data,
            total,
            page,
            limit,
            total_pages,
        }
    }
}