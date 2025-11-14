// Claude供应商API处理器
//
// 提供Claude供应商的HTTP API接口实现

use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::{get, post, delete, put},
    Router,
};
use serde::Deserialize;
use tracing::{info, warn, error};

use crate::api::error::ApiError;
use crate::api::responses::{ApiResponse, PagedResponse};
use crate::models::{ClaudeProvider, CreateClaudeProviderRequest, UpdateClaudeProviderRequest, PaginationParams};
use crate::services::claude_service::{ClaudeProviderService, ClaudeServiceError};

// 使用服务器模块中的ApiState
use crate::api::server::ApiState;

/// 将Service错误转换为API错误
impl From<ClaudeServiceError> for ApiError {
    fn from(err: ClaudeServiceError) -> Self {
        match err {
            ClaudeServiceError::Validation(msg) => ApiError::Validation(msg),
            ClaudeServiceError::BusinessRule(msg) => ApiError::BusinessRule(msg),
            ClaudeServiceError::Repository(repo_err) => ApiError::Database(repo_err.to_string()),
            ClaudeServiceError::ProviderNotFound(id) => ApiError::NotFound(format!("供应商 {} 不存在", id)),
            ClaudeServiceError::NameAlreadyExists(name) => ApiError::Conflict(format!("供应商名称 '{}' 已存在", name)),
            ClaudeServiceError::NoActiveProvider => ApiError::NotFound("没有启用的供应商".to_string()),
        }
    }
}

/// 查询参数
#[derive(Debug, Deserialize)]
pub struct ClaudeProviderQuery {
    pub search: Option<String>,
    pub active_only: Option<bool>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// 创建Claude供应商
pub async fn create_claude_provider(
    State(state): State<ApiState>,
    Json(request): Json<CreateClaudeProviderRequest>,
) -> Result<Json<ApiResponse<ClaudeProvider>>, ApiError> {
    info!(
        name = %request.name,
        url = %request.url,
        "创建Claude供应商请求"
    );

    // 使用Service层创建供应商
    let id = state.claude_service.create_provider(request).await.map_err(|e| {
        error!(
            error = %e,
            "创建Claude供应商失败"
        );
        ApiError::from(e)
    })?;

    // 获取创建的记录
    if let Some(provider) = state.claude_service.get_provider(id).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "获取新创建的Claude供应商失败"
        );
        ApiError::from(e)
    })? {
        info!(
            id = %id,
            name = %provider.name,
            "Claude供应商创建成功"
        );

        Ok(Json(ApiResponse::success_with_message(
            provider,
            "Claude供应商创建成功".to_string()
        )))
    } else {
        error!(
            id = %id,
            "创建Claude供应商后无法找到记录"
        );
        Err(ApiError::Internal("创建Claude供应商后无法找到记录".to_string()))
    }
}

/// 获取Claude供应商详情
pub async fn get_claude_provider(
    State(state): State<ApiState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<ClaudeProvider>>, ApiError> {
    info!(
        id = %id,
        "获取Claude供应商详情请求"
    );

    match state.claude_service.get_provider(id).await {
        Ok(Some(provider)) => {
            info!(
                id = %id,
                name = %provider.name,
                "获取Claude供应商详情成功"
            );

            Ok(Json(ApiResponse::success_with_message(
                provider,
                "获取Claude供应商详情成功".to_string()
            )))
        }
        Ok(None) => {
            warn!(
                id = %id,
                "Claude供应商不存在"
            );
            Err(ApiError::NotFound("Claude供应商不存在".to_string()))
        }
        Err(e) => {
            error!(
                error = %e,
                id = %id,
                "获取Claude供应商详情失败"
            );
            Err(ApiError::from(e))
        }
    }
}

/// 更新Claude供应商
pub async fn update_claude_provider(
    State(state): State<ApiState>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateClaudeProviderRequest>,
) -> Result<Json<ApiResponse<ClaudeProvider>>, ApiError> {
    info!(
        id = %id,
        "更新Claude供应商请求"
    );

    // 执行更新
    let updated = state.claude_service.update_provider(id, request).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "更新Claude供应商失败"
        );
        ApiError::from(e)
    })?;

    if !updated {
        error!(
            id = %id,
            "更新Claude供应商未影响任何记录"
        );
        return Err(ApiError::Internal("更新Claude供应商失败".to_string()));
    }

    // 获取更新后的记录
    if let Some(provider) = state.claude_service.get_provider(id).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "获取更新后的Claude供应商失败"
        );
        ApiError::from(e)
    })? {
        info!(
            id = %id,
            name = %provider.name,
            "Claude供应商更新成功"
        );

        Ok(Json(ApiResponse::success_with_message(
            provider,
            "Claude供应商更新成功".to_string()
        )))
    } else {
        error!(
            id = %id,
            "更新Claude供应商后无法找到记录"
        );
        Err(ApiError::Internal("更新Claude供应商后无法找到记录".to_string()))
    }
}

/// 删除Claude供应商
pub async fn delete_claude_provider(
    State(state): State<ApiState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    info!(
        id = %id,
        "删除Claude供应商请求"
    );

    let deleted = state.claude_service.delete_provider(id).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "删除Claude供应商失败"
        );
        ApiError::from(e)
    })?;

    if !deleted {
        error!(
            id = %id,
            "删除Claude供应商未影响任何记录"
        );
        return Err(ApiError::Internal("删除Claude供应商失败".to_string()));
    }

    info!(
        id = %id,
        "Claude供应商删除成功"
    );

    Ok(Json(ApiResponse::success_with_message(
        (),
        "Claude供应商删除成功".to_string()
    )))
}

/// 获取Claude供应商列表
pub async fn list_claude_providers(
    State(state): State<ApiState>,
    Query(query): Query<ClaudeProviderQuery>,
) -> Result<Json<ApiResponse<PagedResponse<ClaudeProvider>>>, ApiError> {
    info!(
        search = ?query.search,
        active_only = ?query.active_only,
        page = ?query.page,
        limit = ?query.limit,
        "获取Claude供应商列表请求"
    );

    let result = if let Some(search_term) = query.search {
        // 搜索模式
        let limit = query.limit.or(Some(50));
        let providers = state.claude_service.search_providers(&search_term, limit).await.map_err(|e| {
            error!(
                error = %e,
                search_term = %search_term,
                "搜索Claude供应商失败"
            );
            ApiError::from(e)
        })?;

        // 转换为分页响应格式
        let total = providers.len() as i64;
        let paged_result = crate::models::PagedResult::new(providers, total, 1, total);

        info!(
            search_term = %search_term,
            count = %total,
            "Claude供应商搜索完成"
        );

        paged_result
    } else if query.active_only.unwrap_or(false) {
        // 仅获取活跃供应商
        let providers = state.claude_service.list_active_providers().await.map_err(|e| {
            error!(
                error = %e,
                "获取活跃Claude供应商列表失败"
            );
            ApiError::from(e)
        })?;

        // 转换为分页响应格式
        let total = providers.len() as i64;
        let paged_result = crate::models::PagedResult::new(providers, total, 1, total);

        info!(
            count = %total,
            "活跃Claude供应商列表获取完成"
        );

        paged_result
    } else {
        // 分页获取所有供应商
        let pagination_params = PaginationParams {
            page: query.page,
            limit: query.limit,
            offset: query.offset,
        };

        state.claude_service.list_providers(pagination_params).await.map_err(|e| {
            error!(
                error = %e,
                "分页获取Claude供应商列表失败"
            );
            ApiError::from(e)
        })?
    };

    let paged_response = crate::api::responses::PagedResponse::from_paged_result_with_message(
        result,
        "获取Claude供应商列表成功".to_string()
    );

    Ok(Json(ApiResponse::success(paged_response)))
}

/// 测试Claude供应商连接
pub async fn test_claude_provider_connection(
    State(state): State<ApiState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<bool>>, ApiError> {
    info!(
        id = %id,
        "测试Claude供应商连接请求"
    );

    match state.claude_service.test_provider_connection(id).await {
        Ok(success) => {
            info!(
                id = %id,
                success = %success,
                "Claude供应商连接测试完成"
            );

            let message = if success { "连接测试成功" } else { "连接测试失败" };
            Ok(Json(ApiResponse::success_with_message(
                success,
                message.to_string()
            )))
        }
        Err(e) => {
            error!(
                error = %e,
                id = %id,
                "Claude供应商连接测试失败"
            );
            Err(ApiError::from(e))
        }
    }
}

/// 获取Claude供应商统计信息
pub async fn get_claude_provider_stats(
    State(state): State<ApiState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    info!("获取Claude供应商统计信息请求");

    let stats = state.claude_service.get_provider_stats().await.map_err(|e| {
        error!(
            error = %e,
            "获取Claude供应商统计信息失败"
        );
        ApiError::from(e)
    })?;

    info!(
        "Claude供应商统计信息获取完成"
    );

    Ok(Json(ApiResponse::success_with_message(
        stats,
        "获取Claude供应商统计信息成功".to_string()
    )))
}

/// 启用Claude供应商
pub async fn enable_claude_provider(
    State(state): State<ApiState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    info!(
        id = %id,
        "启用Claude供应商请求"
    );

    let enabled = state.claude_service.enable_provider(id).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "启用Claude供应商失败"
        );
        ApiError::from(e)
    })?;

    if !enabled {
        error!(
            id = %id,
            "启用Claude供应商未影响任何记录"
        );
        return Err(ApiError::Internal("启用Claude供应商失败".to_string()));
    }

    info!(
        id = %id,
        "Claude供应商启用成功"
    );

    Ok(Json(ApiResponse::success_with_message(
        (),
        "Claude供应商启用成功".to_string()
    )))
}

/// 禁用Claude供应商
pub async fn disable_claude_provider(
    State(state): State<ApiState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    info!(
        id = %id,
        "禁用Claude供应商请求"
    );

    let disabled = state.claude_service.disable_provider(id).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "禁用Claude供应商失败"
        );
        ApiError::from(e)
    })?;

    if !disabled {
        error!(
            id = %id,
            "禁用Claude供应商未影响任何记录"
        );
        return Err(ApiError::Internal("禁用Claude供应商失败".to_string()));
    }

    info!(
        id = %id,
        "Claude供应商禁用成功"
    );

    Ok(Json(ApiResponse::success_with_message(
        (),
        "Claude供应商禁用成功".to_string()
    )))
}

/// 获取当前启用的Claude供应商
pub async fn get_current_claude_provider(
    State(state): State<ApiState>,
) -> Result<Json<ApiResponse<ClaudeProvider>>, ApiError> {
    info!("获取当前启用的Claude供应商请求");

    match state.claude_service.get_current_provider().await {
        Ok(Some(provider)) => {
            info!(
                id = %provider.id,
                name = %provider.name,
                "获取当前启用的Claude供应商成功"
            );

            Ok(Json(ApiResponse::success_with_message(
                provider,
                "获取当前启用的Claude供应商成功".to_string()
            )))
        }
        Ok(None) => {
            warn!("没有找到启用的Claude供应商");
            Err(ApiError::NotFound("没有启用的Claude供应商".to_string()))
        }
        Err(e) => {
            error!(
                error = %e,
                "获取当前启用的Claude供应商失败"
            );
            Err(ApiError::from(e))
        }
    }
}

/// Claude供应商管理路由
pub fn routes() -> Router<ApiState> {
    Router::new()
        // 创建Claude供应商
        .route("/", post(create_claude_provider))
        // 获取Claude供应商列表
        .route("/", get(list_claude_providers))
        // 获取Claude供应商统计信息
        .route("/stats", get(get_claude_provider_stats))
        // 获取当前启用的供应商
        .route("/current", get(get_current_claude_provider))
        // 获取单个Claude供应商
        .route("/:id", get(get_claude_provider))
        // 更新Claude供应商
        .route("/:id", put(update_claude_provider))
        // 删除Claude供应商
        .route("/:id", delete(delete_claude_provider))
        // 启用Claude供应商
        .route("/:id/enable", post(enable_claude_provider))
        // 禁用Claude供应商
        .route("/:id/disable", post(disable_claude_provider))
        // 测试Claude供应商连接
        .route("/:id/test", get(test_claude_provider_connection))
}