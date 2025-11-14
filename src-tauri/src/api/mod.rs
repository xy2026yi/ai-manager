// API模块 - RESTful API服务
//
// 此模块提供完整的RESTful API服务，支持所有数据实体的CRUD操作
// 包括Claude供应商、Codex供应商、Agent指导文件、MCP服务器和通用配置

pub mod server;
pub mod middleware;
pub mod error;
pub mod responses;
pub mod handlers;

// 重新导出主要组件
pub use server::ApiServer;
pub use middleware::{RequestContext, request_tracking_middleware, add_request_id_header, auth_middleware, api_key_middleware, global_error_handler};
pub use error::{ApiError, ApiResult};
pub use responses::{ApiResponse, PagedResponse};