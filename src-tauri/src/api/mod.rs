// API模块 - RESTful API服务
//
// 此模块提供完整的RESTful API服务，支持所有数据实体的CRUD操作
// 包括Claude供应商、Codex供应商、Agent指导文件、MCP服务器和通用配置

pub mod error;
pub mod handlers;
pub mod middleware;
pub mod responses;
pub mod server;

// 重新导出主要组件
pub use error::{ApiError, ApiResult};
pub use middleware::RequestContext;
pub use responses::{ApiResponse, PagedResponse};
pub use server::ApiServer;
