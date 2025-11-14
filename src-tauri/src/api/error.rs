// API错误处理
//
// 定义API专用的错误类型和HTTP状态码映射

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use tracing::error;

/// API错误类型
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("请求参数无效: {0}")]
    BadRequest(String),

    #[error("验证失败: {0}")]
    Validation(String),

    #[error("业务规则冲突: {0}")]
    BusinessRule(String),

    #[error("未授权访问: {0}")]
    Unauthorized(String),

    #[error("权限不足: {0}")]
    Forbidden(String),

    #[error("资源不存在: {0}")]
    NotFound(String),

    #[error("资源冲突: {0}")]
    Conflict(String),

    #[error("请求过于频繁")]
    TooManyRequests,

    #[error("数据库错误: {0}")]
    Database(String),

    #[error("服务器内部错误: {0}")]
    Internal(String),

    #[error("服务器内部错误: {0}")]
    InternalServerError(String),

    #[error("服务暂时不可用")]
    ServiceUnavailable,

    #[error("数据库错误: {0}")]
    DatabaseError(String),

    #[error("加密处理错误: {0}")]
    CryptoError(String),

    #[error("数据验证失败: {0}")]
    ValidationError(String),
}

impl ApiError {
    /// 获取HTTP状态码
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Validation(_) => StatusCode::BAD_REQUEST,
            ApiError::BusinessRule(_) => StatusCode::CONFLICT,
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden(_) => StatusCode::FORBIDDEN,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::Conflict(_) => StatusCode::CONFLICT,
            ApiError::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            ApiError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            ApiError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::CryptoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ValidationError(_) => StatusCode::BAD_REQUEST,
        }
    }

    /// 获取错误代码
    pub fn error_code(&self) -> &'static str {
        match self {
            ApiError::BadRequest(_) => "BAD_REQUEST",
            ApiError::Validation(_) => "VALIDATION_ERROR",
            ApiError::BusinessRule(_) => "BUSINESS_RULE_VIOLATION",
            ApiError::Unauthorized(_) => "UNAUTHORIZED",
            ApiError::Forbidden(_) => "FORBIDDEN",
            ApiError::NotFound(_) => "NOT_FOUND",
            ApiError::Conflict(_) => "CONFLICT",
            ApiError::TooManyRequests => "TOO_MANY_REQUESTS",
            ApiError::Database(_) => "DATABASE_ERROR",
            ApiError::Internal(_) => "INTERNAL_ERROR",
            ApiError::InternalServerError(_) => "INTERNAL_SERVER_ERROR",
            ApiError::ServiceUnavailable => "SERVICE_UNAVAILABLE",
            ApiError::DatabaseError(_) => "DATABASE_ERROR",
            ApiError::CryptoError(_) => "CRYPTO_ERROR",
            ApiError::ValidationError(_) => "VALIDATION_ERROR",
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
            ApiError::ValidationError(details) => Some(json!({
                "validation_errors": details
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
            sqlx::Error::RowNotFound => ApiError::NotFound("记录不存在".to_string()),
            sqlx::Error::Database(db_err) => {
                if db_err.is_unique_violation() {
                    ApiError::Conflict("数据已存在".to_string())
                } else {
                    ApiError::DatabaseError(db_err.message().to_string())
                }
            }
            _ => ApiError::DatabaseError(err.to_string()),
        }
    }
}

/// 从加密错误转换
impl From<crate::crypto::CryptoError> for ApiError {
    fn from(err: crate::crypto::CryptoError) -> Self {
        error!("加密错误: {}", err);
        ApiError::CryptoError(err.to_string())
    }
}

/// 从验证错误转换
impl From<validator::ValidationErrors> for ApiError {
    fn from(err: validator::ValidationErrors) -> Self {
        error!("验证错误: {:?}", err);
        ApiError::ValidationError(format!("输入验证失败: {:?}", err))
    }
}