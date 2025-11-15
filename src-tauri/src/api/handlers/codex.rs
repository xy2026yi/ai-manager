// Codex供应商API处理器
//
// 提供Codex供应商的HTTP API接口实现

use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde::Deserialize;
use tracing::{error, info, warn};

use crate::api::error::ApiError;
use crate::api::responses::{ApiResponse, PagedResponse};
use crate::models::{
    CodexProvider, CreateCodexProviderRequest, PaginationParams, UpdateCodexProviderRequest,
};

// 使用服务器模块中的ApiState
use crate::api::server::ApiState;

/// 查询参数
#[derive(Debug, Deserialize)]
pub struct CodexProviderQuery {
    pub search: Option<String>,
    pub active_only: Option<bool>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// 创建Codex供应商
pub async fn create_codex_provider(
    State(state): State<ApiState>,
    Json(request): Json<CreateCodexProviderRequest>,
) -> Result<Json<ApiResponse<CodexProvider>>, ApiError> {
    info!(
        name = %request.name,
        url = %request.url,
        "创建Codex供应商请求"
    );

    // 验证请求
    if request.name.trim().is_empty() {
        return Err(ApiError::validation("供应商名称不能为空".to_string()));
    }

    if request.token.trim().is_empty() {
        return Err(ApiError::validation("Token不能为空".to_string()));
    }

    // 创建记录
    let id = state.codex_service.create_provider(&request).await.map_err(|e| {
        error!(
            error = %e,
            name = %request.name,
            "创建Codex供应商失败"
        );
        match e {
            crate::services::codex_service::CodexServiceError::Validation(msg) => {
                ApiError::validation(msg)
            }
            crate::services::codex_service::CodexServiceError::BusinessRule(msg) => {
                ApiError::BusinessRule { message: msg }
            }
            crate::services::codex_service::CodexServiceError::Repository(repo_err) => {
                ApiError::Database { message: format!("数据库错误: {}", repo_err) }
            }
            _ => ApiError::Internal { message: format!("创建Codex供应商失败: {}", e) },
        }
    })?;

    // 获取创建的记录
    if let Some(provider) = state.codex_service.get_provider(id).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "获取新创建的Codex供应商失败"
        );
        ApiError::Database { message: format!("获取Codex供应商失败: {}", e) }
    })? {
        info!(
            id = %id,
            name = %provider.name,
            "Codex供应商创建成功"
        );

        Ok(Json(ApiResponse::success_with_message(
            provider,
            "Codex供应商创建成功".to_string(),
        )))
    } else {
        error!(
            id = %id,
            "创建Codex供应商后无法找到记录"
        );
        Err(ApiError::Internal {
            message: "创建Codex供应商后无法找到记录".to_string()
        })
    }
}

/// 获取Codex供应商详情
pub async fn get_codex_provider(
    State(state): State<ApiState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<CodexProvider>>, ApiError> {
    info!(
        id = %id,
        "获取Codex供应商详情请求"
    );

    if id <= 0 {
        return Err(ApiError::validation("无效的ID".to_string()));
    }

    match state.codex_service.get_provider(id).await {
        Ok(Some(provider)) => {
            info!(
                id = %id,
                name = %provider.name,
                "获取Codex供应商详情成功"
            );

            Ok(Json(ApiResponse::success_with_message(
                provider,
                "获取Codex供应商详情成功".to_string(),
            )))
        }
        Ok(None) => {
            warn!(
                id = %id,
                "Codex供应商不存在"
            );
            Err(ApiError::NotFound { resource: "Codex供应商不存在".to_string() })
        }
        Err(e) => {
            error!(
                error = %e,
                id = %id,
                "获取Codex供应商详情失败"
            );
            Err(match e {
                crate::services::codex_service::CodexServiceError::Validation(msg) => {
                    ApiError::validation(msg)
                }
                crate::services::codex_service::CodexServiceError::ProviderNotFound(_) => {
                    ApiError::NotFound { resource: "Codex供应商不存在".to_string() }
                }
                _ => ApiError::Database { message: format!("获取Codex供应商失败: {}", e) },
            })
        }
    }
}

/// 更新Codex供应商
pub async fn update_codex_provider(
    State(state): State<ApiState>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateCodexProviderRequest>,
) -> Result<Json<ApiResponse<CodexProvider>>, ApiError> {
    info!(
        id = %id,
        "更新Codex供应商请求"
    );

    if id <= 0 {
        return Err(ApiError::validation("无效的ID".to_string()));
    }

    // 更新记录
    let updated = state.codex_service.update_provider(id, request).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "更新Codex供应商失败"
        );
        match e {
            crate::services::codex_service::CodexServiceError::Validation(msg) => {
                ApiError::validation(msg)
            }
            crate::services::codex_service::CodexServiceError::BusinessRule(msg) => {
                ApiError::BusinessRule { message: msg }
            }
            crate::services::codex_service::CodexServiceError::ProviderNotFound(_) => {
                ApiError::NotFound { resource: "Codex供应商不存在".to_string() }
            }
            crate::services::codex_service::CodexServiceError::Repository(repo_err) => {
                ApiError::Database { message: format!("数据库错误: {}", repo_err) }
            }
            _ => ApiError::Internal { message: format!("更新Codex供应商失败: {}", e) },
        }
    })?;

    if !updated {
        error!(
            id = %id,
            "更新Codex供应商未影响任何记录"
        );
        return Err(ApiError::Internal { message: "更新Codex供应商失败".to_string() });
    }

    // 获取更新后的记录
    if let Some(provider) = state.codex_service.get_provider(id).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "获取更新后的Codex供应商失败"
        );
        ApiError::Database { message: format!("获取Codex供应商失败: {}", e) }
    })? {
        info!(
            id = %id,
            name = %provider.name,
            "Codex供应商更新成功"
        );

        Ok(Json(ApiResponse::success_with_message(
            provider,
            "Codex供应商更新成功".to_string(),
        )))
    } else {
        error!(
            id = %id,
            "更新Codex供应商后无法找到记录"
        );
        Err(ApiError::Internal {
            message: "更新Codex供应商后无法找到记录".to_string()
        })
    }
}

/// 删除Codex供应商
pub async fn delete_codex_provider(
    State(state): State<ApiState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    info!(
        id = %id,
        "删除Codex供应商请求"
    );

    if id <= 0 {
        return Err(ApiError::validation("无效的ID".to_string()));
    }

    // 删除记录
    let deleted = state.codex_service.delete_provider(id).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "删除Codex供应商失败"
        );
        match e {
            crate::services::codex_service::CodexServiceError::Validation(msg) => {
                ApiError::validation(msg)
            }
            crate::services::codex_service::CodexServiceError::ProviderNotFound(_) => {
                ApiError::NotFound { resource: "Codex供应商不存在".to_string() }
            }
            crate::services::codex_service::CodexServiceError::Repository(repo_err) => {
                ApiError::Database { message: format!("数据库错误: {}", repo_err) }
            }
            _ => ApiError::Internal { message: format!("删除Codex供应商失败: {}", e) },
        }
    })?;

    if !deleted {
        error!(
            id = %id,
            "删除Codex供应商未影响任何记录"
        );
        return Err(ApiError::Internal { message: "删除Codex供应商失败".to_string() });
    }

    info!(
        id = %id,
        "Codex供应商删除成功"
    );

    Ok(Json(ApiResponse::success_with_message(
        (),
        "Codex供应商删除成功".to_string(),
    )))
}

/// 获取Codex供应商列表
pub async fn list_codex_providers(
    State(state): State<ApiState>,
    Query(query): Query<CodexProviderQuery>,
) -> Result<Json<ApiResponse<PagedResponse<CodexProvider>>>, ApiError> {
    info!(
        search = ?query.search,
        active_only = ?query.active_only,
        page = ?query.page,
        limit = ?query.limit,
        "获取Codex供应商列表请求"
    );

    let result = if let Some(search_term) = query.search {
        // 搜索模式
        let limit = query.limit.or(Some(50));
        let providers =
            state.codex_service.search_providers(&search_term, limit).await.map_err(|e| {
                error!(
                    error = %e,
                    search_term = %search_term,
                    "搜索Codex供应商失败"
                );
                ApiError::Database { message: format!("搜索Codex供应商失败: {}", e) }
            })?;

        // 转换为分页响应格式
        let total = providers.len() as i64;
        let paged_result = crate::models::PagedResult::new(providers, total, 1, total);

        info!(
            search_term = %search_term,
            count = %total,
            "Codex供应商搜索完成"
        );

        paged_result
    } else if query.active_only.unwrap_or(false) {
        // 仅获取活跃供应商
        let providers = state.codex_service.list_active_providers().await.map_err(|e| {
            error!(
                error = %e,
                "获取活跃Codex供应商列表失败"
            );
            ApiError::Database { message: format!("获取Codex供应商列表失败: {}", e) }
        })?;

        // 转换为分页响应格式
        let total = providers.len() as i64;
        let paged_result = crate::models::PagedResult::new(providers, total, 1, total);

        info!(
            count = %total,
            "活跃Codex供应商列表获取完成"
        );

        paged_result
    } else {
        // 分页获取所有供应商
        let pagination_params =
            PaginationParams { page: query.page, limit: query.limit, offset: query.offset };

        state.codex_service.list_providers(pagination_params).await.map_err(|e| {
            error!(
                error = %e,
                "分页获取Codex供应商列表失败"
            );
            ApiError::Database { message: format!("获取Codex供应商列表失败: {}", e) }
        })?
    };

    let paged_response = crate::api::responses::PagedResponse::from_paged_result_with_message(
        result,
        "获取Codex供应商列表成功".to_string(),
    );

    Ok(Json(ApiResponse::success(paged_response)))
}

/// 测试Codex供应商连接
pub async fn test_codex_provider_connection(
    State(state): State<ApiState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<bool>>, ApiError> {
    info!(
        id = %id,
        "测试Codex供应商连接请求"
    );

    if id <= 0 {
        return Err(ApiError::validation("无效的ID".to_string()));
    }

    match state.codex_service.test_provider_connection(id).await {
        Ok(success) => {
            info!(
                id = %id,
                success = %success,
                "Codex供应商连接测试完成"
            );

            let message = if success {
                "连接测试成功"
            } else {
                "连接测试失败"
            };
            Ok(Json(ApiResponse::success_with_message(
                success,
                message.to_string(),
            )))
        }
        Err(e) => {
            error!(
                error = %e,
                id = %id,
                "Codex供应商连接测试失败"
            );
            Err(match e {
                crate::services::codex_service::CodexServiceError::Validation(msg) => {
                    ApiError::validation(msg)
                }
                crate::services::codex_service::CodexServiceError::ProviderNotFound(_) => {
                    ApiError::NotFound { resource: "Codex供应商不存在".to_string() }
                }
                _ => ApiError::Database { message: format!("连接测试失败: {}", e) },
            })
        }
    }
}

/// 获取Codex供应商统计信息
pub async fn get_codex_provider_stats(
    State(state): State<ApiState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    info!("获取Codex供应商统计信息请求");

    let stats = state.codex_service.get_provider_stats().await.map_err(|e| {
        error!(
            error = %e,
            "获取Codex供应商统计信息失败"
        );
        ApiError::Database { message: format!("获取统计信息失败: {}", e) }
    })?;

    info!("Codex供应商统计信息获取完成");

    Ok(Json(ApiResponse::success_with_message(
        stats,
        "获取Codex供应商统计信息成功".to_string(),
    )))
}

/// Codex供应商API路由
pub fn routes() -> Router<ApiState> {
    Router::new()
        // 创建Codex供应商
        .route("/", post(create_codex_provider))
        // 获取Codex供应商列表
        .route("/", get(list_codex_providers))
        // 获取Codex供应商统计信息
        .route("/stats", get(get_codex_provider_stats))
        // 获取单个Codex供应商
        .route("/:id", get(get_codex_provider))
        // 更新Codex供应商
        .route("/:id", put(update_codex_provider))
        // 删除Codex供应商
        .route("/:id", delete(delete_codex_provider))
        // 测试Codex供应商连接
        .route("/:id/test", get(test_codex_provider_connection))
}
