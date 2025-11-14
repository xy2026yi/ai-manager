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

/// API状态
#[derive(Clone)]
pub struct ApiState {
    pub claude_service: ClaudeProviderService,
}

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

    let repository = ClaudeProviderRepository::new(&state.db_manager, &state.crypto_service);

    if id <= 0 {
        return Err(ApiError::Validation("无效的ID".to_string()));
    }

    // 检查记录是否存在
    let existing = repository.find_by_id_decrypted(id).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "检查Claude供应商是否存在失败"
        );
        ApiError::Database(format!("检查Claude供应商失败: {}", e))
    })?;

    if existing.is_none() {
        warn!(
            id = %id,
            "尝试更新不存在的Claude供应商"
        );
        return Err(ApiError::NotFound("Claude供应商不存在".to_string()));
    }

    // 验证更新数据
    if let Some(ref name) = request.name {
        if name.trim().is_empty() {
            return Err(ApiError::Validation("供应商名称不能为空".to_string()));
        }
    }

    if let Some(ref token) = request.token {
        if token.trim().is_empty() {
            return Err(ApiError::Validation("Token不能为空".to_string()));
        }
    }

    // 更新记录
    let updated = repository.update_claude_provider(id, &request).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "更新Claude供应商失败"
        );
        ApiError::Database(format!("更新Claude供应商失败: {}", e))
    })?;

    if !updated {
        error!(
            id = %id,
            "更新Claude供应商未影响任何记录"
        );
        return Err(ApiError::Internal("更新Claude供应商失败".to_string()));
    }

    // 获取更新后的记录
    if let Some(provider) = repository.find_by_id_decrypted(id).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "获取更新后的Claude供应商失败"
        );
        ApiError::Database(format!("获取Claude供应商失败: {}", e))
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

    let repository = ClaudeProviderRepository::new(&state.db_manager, &state.crypto_service);

    if id <= 0 {
        return Err(ApiError::Validation("无效的ID".to_string()));
    }

    // 检查记录是否存在
    let existing = repository.find_by_id::<ClaudeProvider>(id).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "检查Claude供应商是否存在失败"
        );
        ApiError::Database(format!("检查Claude供应商失败: {}", e))
    })?;

    if existing.is_none() {
        warn!(
            id = %id,
            "尝试删除不存在的Claude供应商"
        );
        return Err(ApiError::NotFound("Claude供应商不存在".to_string()));
    }

    // 删除记录
    let deleted = repository.delete(id).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "删除Claude供应商失败"
        );
        ApiError::Database(format!("删除Claude供应商失败: {}", e))
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

    let repository = ClaudeProviderRepository::new(&state.db_manager, &state.crypto_service);

    let result = if let Some(search_term) = query.search {
        // 搜索模式
        let limit = query.limit.or(Some(50));
        let providers = repository.search_claude_providers(&search_term, limit).await.map_err(|e| {
            error!(
                error = %e,
                search_term = %search_term,
                "搜索Claude供应商失败"
            );
            ApiError::Database(format!("搜索Claude供应商失败: {}", e))
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
        let providers = repository.list_active_providers().await.map_err(|e| {
            error!(
                error = %e,
                "获取活跃Claude供应商列表失败"
            );
            ApiError::Database(format!("获取Claude供应商列表失败: {}", e))
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

        repository.paginate::<ClaudeProvider>(&pagination_params).await.map_err(|e| {
            error!(
                error = %e,
                "分页获取Claude供应商列表失败"
            );
            ApiError::Database(format!("获取Claude供应商列表失败: {}", e))
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

    let repository = ClaudeProviderRepository::new(&state.db_manager, &state.crypto_service);

    if id <= 0 {
        return Err(ApiError::Validation("无效的ID".to_string()));
    }

    match repository.test_connection(id).await {
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
            Err(ApiError::Database(format!("连接测试失败: {}", e)))
        }
    }
}

/// 获取Claude供应商统计信息
pub async fn get_claude_provider_stats(
    State(state): State<ApiState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    info!("获取Claude供应商统计信息请求");

    let repository = ClaudeProviderRepository::new(&state.db_manager, &state.crypto_service);

    // 获取总数
    let total = repository.count().await.map_err(|e| {
        error!(
            error = %e,
            "获取Claude供应商总数失败"
        );
        ApiError::Database(format!("获取统计信息失败: {}", e))
    })?;

    // 获取活跃数量
    let active = repository.count_by_status(true).await.map_err(|e| {
        error!(
            error = %e,
            "获取活跃Claude供应商数量失败"
        );
        ApiError::Database(format!("获取统计信息失败: {}", e))
    })?;

    // 获取非活跃数量
    let inactive = repository.count_by_status(false).await.map_err(|e| {
        error!(
            error = %e,
            "获取非活跃Claude供应商数量失败"
        );
        ApiError::Database(format!("获取统计信息失败: {}", e))
    })?;

    let stats = serde_json::json!({
        "total": total,
        "active": active,
        "inactive": inactive,
        "active_rate": if total > 0 { (active as f64 / total as f64 * 100.0).round() } else { 0.0 }
    });

    info!(
        total = %total,
        active = %active,
        inactive = %inactive,
        "Claude供应商统计信息获取完成"
    );

    Ok(Json(ApiResponse::success_with_message(
        stats,
        "获取Claude供应商统计信息成功".to_string()
    )))
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
        // 获取单个Claude供应商
        .route("/:id", get(get_claude_provider))
        // 更新Claude供应商
        .route("/:id", put(update_claude_provider))
        // 删除Claude供应商
        .route("/:id", delete(delete_claude_provider))
        // 测试Claude供应商连接
        .route("/:id/test", get(test_claude_provider_connection))
}