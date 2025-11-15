//! API错误处理模块
//!
//! 这个模块定义了应用程序中所有API相关的错误类型，
//! 提供统一的错误响应格式和用户友好的中文错误信息。
//!
//! # 主要功能
//!
//! - **错误类型定义**: 涵盖所有常见的API错误场景
//! - **HTTP状态码映射**: 自动将错误映射到正确的HTTP状态码
//! - **错误响应格式**: 统一的JSON错误响应格式
//! - **类型转换**: 从各种错误类型自动转换为ApiError
//!
//! # 使用示例
//!
//! ```rust
//! use crate::api::error::{ApiError, ApiResult};
//!
//! fn get_user(id: i64) -> ApiResult<User> {
//!     if id <= 0 {
//!         return Err(ApiError::BadRequest {
//!             message: "用户ID必须大于0".to_string()
//!         });
//!     }
//!     // ... 获取用户逻辑
//! }
//! ```

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use tracing::error;

/// API错误类型
///
/// 统一的应用程序错误处理，提供用户友好的中文错误信息
#[derive(Error, Debug)]
pub enum ApiError {
    /// 请求参数错误 (400)
    #[error("请求参数无效: {message}")]
    BadRequest { message: String },

    /// 输入验证失败 (400)
    #[error("输入验证失败: {message}")]
    ValidationError { message: String, field: Option<String> },

    /// 业务规则冲突 (409)
    #[error("业务规则冲突: {message}")]
    BusinessRule { message: String },

    /// 未授权访问 (401)
    #[error("未授权访问: {message}")]
    Unauthorized { message: String },

    /// 权限不足 (403)
    #[error("权限不足: {message}")]
    Forbidden { message: String },

    /// 资源不存在 (404)
    #[error("资源不存在: {resource}")]
    NotFound { resource: String },

    /// 资源冲突 (409)
    #[error("资源冲突: {message}")]
    Conflict { message: String },

    /// 请求过于频繁 (429)
    #[error("请求过于频繁，请稍后重试")]
    TooManyRequests,

    /// 数据库操作错误 (500)
    #[error("数据库操作失败: {message}")]
    Database { message: String },

    /// 加密处理错误 (500)
    #[error("数据处理失败: {message}")]
    Crypto { message: String },

    /// 服务器内部错误 (500)
    #[error("服务器内部错误: {message}")]
    Internal { message: String },

    /// 服务暂时不可用 (503)
    #[error("服务暂时不可用，请稍后重试")]
    ServiceUnavailable,

    /// 配置错误 (500)
    #[error("配置错误: {message}")]
    Configuration { message: String },
}

impl ApiError {
    /// 创建验证错误（简化版本）
    pub fn Validation(message: impl Into<String>) -> Self {
        Self::ValidationError {
            message: message.into(),
            field: None,
        }
    }

    /// 获取HTTP状态码
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::BadRequest { .. } => StatusCode::BAD_REQUEST,
            ApiError::ValidationError { .. } => StatusCode::BAD_REQUEST,
            ApiError::BusinessRule { .. } => StatusCode::CONFLICT,
            ApiError::Unauthorized { .. } => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden { .. } => StatusCode::FORBIDDEN,
            ApiError::NotFound { .. } => StatusCode::NOT_FOUND,
            ApiError::Conflict { .. } => StatusCode::CONFLICT,
            ApiError::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            ApiError::Database { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Crypto { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            ApiError::Configuration { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// 获取错误代码
    pub fn error_code(&self) -> &'static str {
        match self {
            ApiError::BadRequest { .. } => "BAD_REQUEST",
            ApiError::ValidationError { .. } => "VALIDATION_ERROR",
            ApiError::BusinessRule { .. } => "BUSINESS_RULE_VIOLATION",
            ApiError::Unauthorized { .. } => "UNAUTHORIZED",
            ApiError::Forbidden { .. } => "FORBIDDEN",
            ApiError::NotFound { .. } => "NOT_FOUND",
            ApiError::Conflict { .. } => "CONFLICT",
            ApiError::TooManyRequests => "TOO_MANY_REQUESTS",
            ApiError::Database { .. } => "DATABASE_ERROR",
            ApiError::Crypto { .. } => "CRYPTO_ERROR",
            ApiError::Internal { .. } => "INTERNAL_ERROR",
            ApiError::ServiceUnavailable => "SERVICE_UNAVAILABLE",
            ApiError::Configuration { .. } => "CONFIGURATION_ERROR",
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status_code = self.status_code();
        let error_code = self.error_code();
        let message = self.to_string();

        error!(
            error_code = %error_code,
            status_code = %status_code,
            message = %message,
            "API错误响应"
        );

        let error_response = json!({
            "success": false,
            "error": {
                "code": error_code,
                "message": message,
                "details": self.get_details()
            },
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        (status_code, Json(error_response)).into_response()
    }
}

impl ApiError {
    /// 获取错误详情
    fn get_details(&self) -> Option<serde_json::Value> {
        match self {
            ApiError::ValidationError { field, .. } => field.as_ref().map(|f| {
                json!({
                    "field": f
                })
            }),
            ApiError::Database { .. } => Some(json!({
                "type": "database_operation"
            })),
            ApiError::Crypto { .. } => Some(json!({
                "type": "encryption_operation"
            })),
            _ => None,
        }
    }
}

/// API结果类型
pub type ApiResult<T> = Result<T, ApiError>;

/// 从数据库错误转换
impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        error!("数据库错误: {}", err);
        match err {
            sqlx::Error::RowNotFound => {
                ApiError::NotFound { resource: "记录不存在".to_string() }
            }
            sqlx::Error::Database(db_err) => {
                if db_err.is_unique_violation() {
                    ApiError::Conflict {
                        message: "数据已存在，违反唯一性约束".to_string()
                    }
                } else {
                    ApiError::Database {
                        message: format!("数据库操作失败: {}", db_err.message())
                    }
                }
            }
            _ => ApiError::Database { message: format!("数据库连接或查询错误: {}", err) },
        }
    }
}

/// 从加密错误转换
impl From<crate::crypto::CryptoError> for ApiError {
    fn from(err: crate::crypto::CryptoError) -> Self {
        error!("加密错误: {}", err);
        ApiError::Crypto { message: format!("数据处理失败: {}", err) }
    }
}

/// 从验证错误转换
impl From<validator::ValidationErrors> for ApiError {
    fn from(err: validator::ValidationErrors) -> Self {
        error!("验证错误: {:?}", err);
        ApiError::ValidationError {
            message: "输入数据验证失败".to_string(),
            field: Some(format!("{:?}", err)),
        }
    }
}

/// 从IO错误转换
impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        error!("IO错误: {}", err);
        ApiError::Internal { message: format!("文件操作失败: {}", err) }
    }
}

/// 从序列化错误转换
impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        error!("JSON序列化错误: {}", err);
        ApiError::BadRequest { message: format!("数据格式错误: {}", err) }
    }
}
