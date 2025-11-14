// 通用配置API处理器
//
// 提供通用配置的HTTP API接口实现

use axum::{
    extract::{Path, Query, State},
    response::Json,
    Router,
};
use serde::Deserialize;
use tracing::{info, warn, error};

use crate::api::error::ApiError;
use crate::api::responses::{ApiResponse, PagedResponse};
use crate::models::{CommonConfig, CreateCommonConfigRequest, UpdateCommonConfigRequest, PaginationParams};
use crate::repositories::{CommonConfigRepository, BaseRepository};
use crate::database::DatabaseManager;
use crate::crypto::CryptoService;

/// 重用API服务器的ApiState
pub use super::super::server::ApiState;

/// 查询参数
#[derive(Debug, Deserialize)]
pub struct CommonConfigQuery {
    pub search: Option<String>,
    pub category: Option<String>,
    pub active_only: Option<bool>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// 批量更新配置请求
#[derive(Debug, Deserialize)]
pub struct BatchUpdateRequest {
    pub configs: Vec<ConfigItem>,
}

/// 配置项
#[derive(Debug, Deserialize)]
pub struct ConfigItem {
    pub key: String,
    pub value: String,
}

/// 创建通用配置
pub async fn create_common_config(
    State(state): State<ApiState>,
    Json(request): Json<CreateCommonConfigRequest>,
) -> Result<Json<ApiResponse<CommonConfig>>, ApiError> {
    info!(
        key = %request.key,
        category = ?request.category,
        "创建通用配置请求"
    );

    // 创建Repository
    let repository = CommonConfigRepository::new(&state.db_manager, &state.crypto_service);

    // 验证请求
    if request.key.trim().is_empty() {
        return Err(ApiError::Validation("配置键不能为空".to_string()));
    }

    if request.value.trim().is_empty() {
        return Err(ApiError::Validation("配置值不能为空".to_string()));
    }

    // 检查key是否已存在
    if let Some(_) = repository.find_by_key(&request.key).await.map_err(|e| {
        error!(
            error = %e,
            key = %request.key,
            "检查配置键是否存在失败"
        );
        ApiError::Database(format!("检查配置键失败: {}", e))
    })? {
        warn!(
            key = %request.key,
            "配置键已存在"
        );
        return Err(ApiError::Validation("配置键已存在".to_string()));
    }

    // 创建记录
    let id = repository.create_common_config(&request).await.map_err(|e| {
        error!(
            error = %e,
            key = %request.key,
            "创建通用配置失败"
        );
        ApiError::Database(format!("创建通用配置失败: {}", e))
    })?;

    // 获取创建的记录
    if let Some(config) = repository.find_by_id_decrypted(id).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "获取新创建的通用配置失败"
        );
        ApiError::Database(format!("获取通用配置失败: {}", e))
    })? {
        info!(
            id = %id,
            key = %config.key,
            "通用配置创建成功"
        );

        Ok(Json(ApiResponse::success_with_message(
            config,
            "通用配置创建成功".to_string()
        )))
    } else {
        error!(
            id = %id,
            "创建通用配置后无法找到记录"
        );
        Err(ApiError::Internal("创建通用配置后无法找到记录".to_string()))
    }
}

/// 获取通用配置详情
pub async fn get_common_config(
    State(state): State<ApiState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<CommonConfig>>, ApiError> {
    info!(
        id = %id,
        "获取通用配置详情请求"
    );

    let repository = CommonConfigRepository::new(&state.db_manager, &state.crypto_service);

    if id <= 0 {
        return Err(ApiError::Validation("无效的ID".to_string()));
    }

    match repository.find_by_id_decrypted(id).await {
        Ok(Some(config)) => {
            info!(
                id = %id,
                key = %config.key,
                "获取通用配置详情成功"
            );

            Ok(Json(ApiResponse::success_with_message(
                config,
                "获取通用配置详情成功".to_string()
            )))
        }
        Ok(None) => {
            warn!(
                id = %id,
                "通用配置不存在"
            );
            Err(ApiError::NotFound("通用配置不存在".to_string()))
        }
        Err(e) => {
            error!(
                error = %e,
                id = %id,
                "获取通用配置详情失败"
            );
            Err(ApiError::Database(format!("获取通用配置失败: {}", e)))
        }
    }
}

/// 根据key获取配置
pub async fn get_common_config_by_key(
    State(state): State<ApiState>,
    Path(key): Path<String>,
) -> Result<Json<ApiResponse<CommonConfig>>, ApiError> {
    info!(
        key = %key,
        "根据key获取通用配置请求"
    );

    let repository = CommonConfigRepository::new(&state.db_manager, &state.crypto_service);

    if key.trim().is_empty() {
        return Err(ApiError::Validation("配置键不能为空".to_string()));
    }

    match repository.find_by_key(&key).await {
        Ok(Some(config)) => {
            info!(
                key = %key,
                "根据key获取通用配置成功"
            );

            Ok(Json(ApiResponse::success_with_message(
                config,
                "根据key获取通用配置成功".to_string()
            )))
        }
        Ok(None) => {
            warn!(
                key = %key,
                "通用配置不存在"
            );
            Err(ApiError::NotFound("通用配置不存在".to_string()))
        }
        Err(e) => {
            error!(
                error = %e,
                key = %key,
                "根据key获取通用配置失败"
            );
            Err(ApiError::Database(format!("获取通用配置失败: {}", e)))
        }
    }
}

/// 更新通用配置
pub async fn update_common_config(
    State(state): State<ApiState>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateCommonConfigRequest>,
) -> Result<Json<ApiResponse<CommonConfig>>, ApiError> {
    info!(
        id = %id,
        "更新通用配置请求"
    );

    let repository = CommonConfigRepository::new(&state.db_manager, &state.crypto_service);

    if id <= 0 {
        return Err(ApiError::Validation("无效的ID".to_string()));
    }

    // 检查记录是否存在
    let existing = repository.find_by_id_decrypted(id).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "检查通用配置是否存在失败"
        );
        ApiError::Database(format!("检查通用配置失败: {}", e))
    })?;

    if existing.is_none() {
        warn!(
            id = %id,
            "尝试更新不存在的通用配置"
        );
        return Err(ApiError::NotFound("通用配置不存在".to_string()));
    }

    // 验证更新数据
    if let Some(ref key) = request.key {
        if key.trim().is_empty() {
            return Err(ApiError::Validation("配置键不能为空".to_string()));
        }

        // 如果更新key，检查新key是否已存在
        if let Some(existing_config) = existing.as_ref() {
            if key != &existing_config.key {
                if let Some(_) = repository.find_by_key(key).await.map_err(|e| {
                    error!(
                        error = %e,
                        key = %key,
                        "检查新配置键是否存在失败"
                    );
                    ApiError::Database(format!("检查配置键失败: {}", e))
                })? {
                    warn!(
                        key = %key,
                        "新配置键已存在"
                    );
                    return Err(ApiError::Validation("新配置键已存在".to_string()));
                }
            }
        }
    }

    if let Some(ref value) = request.value {
        if value.trim().is_empty() {
            return Err(ApiError::Validation("配置值不能为空".to_string()));
        }
    }

    // 更新记录
    let updated = repository.update_common_config(id, &request).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "更新通用配置失败"
        );
        ApiError::Database(format!("更新通用配置失败: {}", e))
    })?;

    if !updated {
        error!(
            id = %id,
            "更新通用配置未影响任何记录"
        );
        return Err(ApiError::Internal("更新通用配置失败".to_string()));
    }

    // 获取更新后的记录
    if let Some(config) = repository.find_by_id_decrypted(id).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "获取更新后的通用配置失败"
        );
        ApiError::Database(format!("获取通用配置失败: {}", e))
    })? {
        info!(
            id = %id,
            key = %config.key,
            "通用配置更新成功"
        );

        Ok(Json(ApiResponse::success_with_message(
            config,
            "通用配置更新成功".to_string()
        )))
    } else {
        error!(
            id = %id,
            "更新通用配置后无法找到记录"
        );
        Err(ApiError::Internal("更新通用配置后无法找到记录".to_string()))
    }
}

/// 删除通用配置
pub async fn delete_common_config(
    State(state): State<ApiState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    info!(
        id = %id,
        "删除通用配置请求"
    );

    let repository = CommonConfigRepository::new(&state.db_manager, &state.crypto_service);

    if id <= 0 {
        return Err(ApiError::Validation("无效的ID".to_string()));
    }

    // 检查记录是否存在
    let existing = repository.find_by_id::<CommonConfig>(id).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "检查通用配置是否存在失败"
        );
        ApiError::Database(format!("检查通用配置失败: {}", e))
    })?;

    if existing.is_none() {
        warn!(
            id = %id,
            "尝试删除不存在的通用配置"
        );
        return Err(ApiError::NotFound("通用配置不存在".to_string()));
    }

    // 删除记录
    let deleted = repository.delete(id).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "删除通用配置失败"
        );
        ApiError::Database(format!("删除通用配置失败: {}", e))
    })?;

    if !deleted {
        error!(
            id = %id,
            "删除通用配置未影响任何记录"
        );
        return Err(ApiError::Internal("删除通用配置失败".to_string()));
    }

    info!(
        id = %id,
        "通用配置删除成功"
    );

    Ok(Json(ApiResponse::success_with_message(
        (),
        "通用配置删除成功".to_string()
    )))
}

/// 获取通用配置列表
pub async fn list_common_configs(
    State(state): State<ApiState>,
    Query(query): Query<CommonConfigQuery>,
) -> Result<Json<ApiResponse<PagedResponse<CommonConfig>>>, ApiError> {
    info!(
        search = ?query.search,
        category = ?query.category,
        active_only = ?query.active_only,
        page = ?query.page,
        limit = ?query.limit,
        "获取通用配置列表请求"
    );

    let repository = CommonConfigRepository::new(&state.db_manager, &state.crypto_service);

    let result = if let Some(search_term) = query.search {
        // 搜索模式
        let limit = query.limit.or(Some(50));
        let configs = repository.search_common_configs(&search_term, limit).await.map_err(|e| {
            error!(
                error = %e,
                search_term = %search_term,
                "搜索通用配置失败"
            );
            ApiError::Database(format!("搜索通用配置失败: {}", e))
        })?;

        // 转换为分页响应格式
        let total = configs.len() as i64;
        let paged_result = crate::models::PagedResult::new(configs, total, 1, total);

        info!(
            search_term = %search_term,
            count = %total,
            "通用配置搜索完成"
        );

        paged_result
    } else if let Some(category) = query.category {
        // 按类别筛选
        let configs = repository.find_by_category(&category).await.map_err(|e| {
            error!(
                error = %e,
                category = %category,
                "根据类别获取通用配置列表失败"
            );
            ApiError::Database(format!("获取通用配置列表失败: {}", e))
        })?;

        // 转换为分页响应格式
        let total = configs.len() as i64;
        let paged_result = crate::models::PagedResult::new(configs, total, 1, total);

        info!(
            category = %category,
            count = %total,
            "按类别获取通用配置列表完成"
        );

        paged_result
    } else if query.active_only.unwrap_or(false) {
        // 仅获取活跃配置
        let configs = repository.list_active_configs().await.map_err(|e| {
            error!(
                error = %e,
                "获取活跃通用配置列表失败"
            );
            ApiError::Database(format!("获取通用配置列表失败: {}", e))
        })?;

        // 转换为分页响应格式
        let total = configs.len() as i64;
        let paged_result = crate::models::PagedResult::new(configs, total, 1, total);

        info!(
            count = %total,
            "活跃通用配置列表获取完成"
        );

        paged_result
    } else {
        // 分页获取所有配置
        let pagination_params = PaginationParams {
            page: query.page,
            limit: query.limit,
            offset: query.offset,
        };

        repository.paginate::<CommonConfig>(&pagination_params).await.map_err(|e| {
            error!(
                error = %e,
                "分页获取通用配置列表失败"
            );
            ApiError::Database(format!("获取通用配置列表失败: {}", e))
        })?
    };

    let paged_response = crate::api::responses::PagedResponse::from_paged_result_with_message(
        result,
        "获取通用配置列表成功".to_string()
    );

    Ok(Json(ApiResponse::success(paged_response)))
}

/// 批量更新配置
pub async fn batch_update_common_configs(
    State(state): State<ApiState>,
    Json(request): Json<BatchUpdateRequest>,
) -> Result<Json<ApiResponse<usize>>, ApiError> {
    info!(
        config_count = %request.configs.len(),
        "批量更新通用配置请求"
    );

    let repository = CommonConfigRepository::new(&state.db_manager, &state.crypto_service);

    if request.configs.is_empty() {
        return Err(ApiError::Validation("配置列表不能为空".to_string()));
    }

    if request.configs.len() > 100 {
        return Err(ApiError::Validation("单次批量更新配置数量不能超过100个".to_string()));
    }

    // 验证配置项
    for config in &request.configs {
        if config.key.trim().is_empty() {
            return Err(ApiError::Validation("配置键不能为空".to_string()));
        }
        if config.value.trim().is_empty() {
            return Err(ApiError::Validation("配置值不能为空".to_string()));
        }
    }

    // 转换为元组格式
    let configs: Vec<(String, String)> = request.configs
        .into_iter()
        .map(|item| (item.key, item.value))
        .collect();

    // 批量更新
    let updated_count = repository.batch_update_configs(&configs).await.map_err(|e| {
        error!(
            error = %e,
            "批量更新通用配置失败"
        );
        ApiError::Database(format!("批量更新通用配置失败: {}", e))
    })?;

    info!(
        updated_count = %updated_count,
        total_configs = %configs.len(),
        "批量更新通用配置完成"
    );

    Ok(Json(ApiResponse::success_with_message(
        updated_count,
        format!("成功更新{}个配置", updated_count)
    )))
}

/// 验证通用配置值
pub async fn validate_common_config(
    State(state): State<ApiState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<bool>>, ApiError> {
    info!(
        id = %id,
        "验证通用配置值请求"
    );

    let repository = CommonConfigRepository::new(&state.db_manager, &state.crypto_service);

    if id <= 0 {
        return Err(ApiError::Validation("无效的ID".to_string()));
    }

    match repository.validate_config_value(id).await {
        Ok(is_valid) => {
            info!(
                id = %id,
                is_valid = %is_valid,
                "通用配置值验证完成"
            );

            let message = if is_valid { "配置值验证通过" } else { "配置值验证失败" };
            Ok(Json(ApiResponse::success_with_message(
                is_valid,
                message.to_string()
            )))
        }
        Err(e) => {
            error!(
                error = %e,
                id = %id,
                "通用配置值验证失败"
            );
            Err(ApiError::Database(format!("配置值验证失败: {}", e)))
        }
    }
}

/// 获取通用配置统计信息
pub async fn get_common_config_stats(
    State(state): State<ApiState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    info!("获取通用配置统计信息请求");

    let repository = CommonConfigRepository::new(&state.db_manager, &state.crypto_service);

    // 获取总数
    let total = repository.count().await.map_err(|e| {
        error!(
            error = %e,
            "获取通用配置总数失败"
        );
        ApiError::Database(format!("获取统计信息失败: {}", e))
    })?;

    // 获取活跃数量
    let active = repository.count_active().await.map_err(|e| {
        error!(
            error = %e,
            "获取活跃通用配置数量失败"
        );
        ApiError::Database(format!("获取统计信息失败: {}", e))
    })?;

    // 获取非活跃数量
    let inactive = total - active;

    // 获取所有类别
    let categories = repository.get_all_categories().await.map_err(|e| {
        error!(
            error = %e,
            "获取配置类别失败"
        );
        ApiError::Database(format!("获取统计信息失败: {}", e))
    })?;

    let stats = serde_json::json!({
        "total": total,
        "active": active,
        "inactive": inactive,
        "active_rate": if total > 0 { (active as f64 / total as f64 * 100.0).round() } else { 0.0 },
        "categories": categories,
        "category_count": categories.len()
    });

    info!(
        total = %total,
        active = %active,
        inactive = %inactive,
        category_count = %categories.len(),
        "通用配置统计信息获取完成"
    );

    Ok(Json(ApiResponse::success_with_message(
        stats,
        "获取通用配置统计信息成功".to_string()
    )))
}

/// 通用配置API路由
pub fn routes() -> Router<ApiState> {
    use axum::routing::{get, post, delete, put};

    Router::new()
        // 创建通用配置
        .route("/", post(create_common_config))
        // 获取通用配置列表
        .route("/", get(list_common_configs))
        // 批量更新通用配置
        .route("/batch", post(batch_update_common_configs))
        // 获取通用配置统计信息
        .route("/stats", get(get_common_config_stats))
        // 获取单个通用配置
        .route("/:id", get(get_common_config))
        // 更新通用配置
        .route("/:id", put(update_common_config))
        // 删除通用配置
        .route("/:id", delete(delete_common_config))
        // 验证通用配置值
        .route("/:id/validate", get(validate_common_config))
        // 根据key获取配置
        .route("/key/:key", get(get_common_config_by_key))
}