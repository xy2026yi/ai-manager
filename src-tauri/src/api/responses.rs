// API响应格式
//
// 定义统一的API响应格式和分页响应

use crate::models::PagedResult;
use serde::{Deserialize, Serialize};

/// 统一API响应格式
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub timestamp: String,
}

impl<T> ApiResponse<T> {
    /// 创建成功响应
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// 创建成功响应（带消息）
    pub fn success_with_message(data: T, message: String) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: Some(message),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// 创建空成功响应
    pub fn success_empty() -> Self {
        Self {
            success: true,
            data: None,
            message: Some("操作成功".to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// 创建空成功响应（带消息）
    pub fn success_empty_with_message(message: String) -> Self {
        Self {
            success: true,
            data: None,
            message: Some(message),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// 分页响应格式
#[derive(Debug, Serialize, Deserialize)]
pub struct PagedResponse<T> {
    pub success: bool,
    pub data: Vec<T>,
    pub pagination: PaginationInfo,
    pub message: Option<String>,
    pub timestamp: String,
}

impl<T> PagedResponse<T> {
    /// 从PagedResult创建分页响应
    pub fn from_paged_result(paged_result: PagedResult<T>) -> Self {
        Self {
            success: true,
            data: paged_result.data,
            pagination: PaginationInfo {
                page: paged_result.page,
                limit: paged_result.limit,
                total: paged_result.total,
                total_pages: paged_result.total_pages,
            },
            message: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// 创建分页响应（带消息）
    pub fn from_paged_result_with_message(paged_result: PagedResult<T>, message: String) -> Self {
        Self {
            success: true,
            data: paged_result.data,
            pagination: PaginationInfo {
                page: paged_result.page,
                limit: paged_result.limit,
                total: paged_result.total,
                total_pages: paged_result.total_pages,
            },
            message: Some(message),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// 分页信息
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub page: i64,
    pub limit: i64,
    pub total: i64,
    pub total_pages: i64,
}

/// API错误响应格式
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: ErrorInfo,
    pub timestamp: String,
}

impl ErrorResponse {
    /// 创建错误响应
    pub fn new(code: String, message: String, details: Option<serde_json::Value>) -> Self {
        Self {
            success: false,
            error: ErrorInfo { code, message, details },
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// 错误信息
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}
