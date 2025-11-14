// Agent指导文件API处理器
//
// 提供Agent指导文件的HTTP API接口实现

use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use serde::Deserialize;
use tracing::{info, warn, error};

use crate::api::error::ApiError;
use crate::api::responses::{ApiResponse, PagedResponse};
use crate::models::{AgentGuide, CreateAgentGuideRequest, UpdateAgentGuideRequest, PaginationParams};
use crate::repositories::{AgentGuideRepository, BaseRepository};
use crate::database::DatabaseManager;
use crate::crypto::CryptoService;

/// 重用Claude供应商的ApiState
pub use super::claude::ApiState;

/// 查询参数
#[derive(Debug, Deserialize)]
pub struct AgentGuideQuery {
    pub search: Option<String>,
    pub guide_type: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// 创建Agent指导文件
pub async fn create_agent_guide(
    State(state): State<ApiState>,
    Json(request): Json<CreateAgentGuideRequest>,
) -> Result<Json<ApiResponse<AgentGuide>>, ApiError> {
    info!(
        name = %request.name,
        guide_type = %request.r#type,
        "创建Agent指导文件请求"
    );

    // 创建Repository
    let repository = AgentGuideRepository::new(&state.db_manager, &state.crypto_service);

    // 验证请求
    if request.name.trim().is_empty() {
        return Err(ApiError::Validation("指导文件名称不能为空".to_string()));
    }

    if request.text.trim().is_empty() {
        return Err(ApiError::Validation("指导文件内容不能为空".to_string()));
    }

    if !["only", "and"].contains(&request.r#type.as_str()) {
        return Err(ApiError::Validation("指导文件类型必须是 'only' 或 'and'".to_string()));
    }

    // 创建记录
    let id = repository.create_agent_guide(&request).await.map_err(|e| {
        error!(
            error = %e,
            name = %request.name,
            "创建Agent指导文件失败"
        );
        ApiError::Database(format!("创建Agent指导文件失败: {}", e))
    })?;

    // 获取创建的记录
    if let Some(guide) = repository.find_by_id_decrypted(id).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "获取新创建的Agent指导文件失败"
        );
        ApiError::Database(format!("获取Agent指导文件失败: {}", e))
    })? {
        info!(
            id = %id,
            name = %guide.name,
            "Agent指导文件创建成功"
        );

        Ok(Json(ApiResponse::success_with_message(
            guide,
            "Agent指导文件创建成功".to_string()
        )))
    } else {
        error!(
            id = %id,
            "创建Agent指导文件后无法找到记录"
        );
        Err(ApiError::Internal("创建Agent指导文件后无法找到记录".to_string()))
    }
}

/// 获取Agent指导文件详情
pub async fn get_agent_guide(
    State(state): State<ApiState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<AgentGuide>>, ApiError> {
    info!(
        id = %id,
        "获取Agent指导文件详情请求"
    );

    let repository = AgentGuideRepository::new(&state.db_manager, &state.crypto_service);

    if id <= 0 {
        return Err(ApiError::Validation("无效的ID".to_string()));
    }

    match repository.find_by_id_decrypted(id).await {
        Ok(Some(guide)) => {
            info!(
                id = %id,
                name = %guide.name,
                "获取Agent指导文件详情成功"
            );

            Ok(Json(ApiResponse::success_with_message(
                guide,
                "获取Agent指导文件详情成功".to_string()
            )))
        }
        Ok(None) => {
            warn!(
                id = %id,
                "Agent指导文件不存在"
            );
            Err(ApiError::NotFound("Agent指导文件不存在".to_string()))
        }
        Err(e) => {
            error!(
                error = %e,
                id = %id,
                "获取Agent指导文件详情失败"
            );
            Err(ApiError::Database(format!("获取Agent指导文件失败: {}", e)))
        }
    }
}

/// 更新Agent指导文件
pub async fn update_agent_guide(
    State(state): State<ApiState>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateAgentGuideRequest>,
) -> Result<Json<ApiResponse<AgentGuide>>, ApiError> {
    info!(
        id = %id,
        "更新Agent指导文件请求"
    );

    let repository = AgentGuideRepository::new(&state.db_manager, &state.crypto_service);

    if id <= 0 {
        return Err(ApiError::Validation("无效的ID".to_string()));
    }

    // 检查记录是否存在
    let existing = repository.find_by_id_decrypted(id).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "检查Agent指导文件是否存在失败"
        );
        ApiError::Database(format!("检查Agent指导文件失败: {}", e))
    })?;

    if existing.is_none() {
        warn!(
            id = %id,
            "尝试更新不存在的Agent指导文件"
        );
        return Err(ApiError::NotFound("Agent指导文件不存在".to_string()));
    }

    // 验证更新数据
    if let Some(ref name) = request.name {
        if name.trim().is_empty() {
            return Err(ApiError::Validation("指导文件名称不能为空".to_string()));
        }
    }

    if let Some(ref text) = request.text {
        if text.trim().is_empty() {
            return Err(ApiError::Validation("指导文件内容不能为空".to_string()));
        }
    }

    if let Some(ref guide_type) = request.r#type {
        if !["only", "and"].contains(&guide_type.as_str()) {
            return Err(ApiError::Validation("指导文件类型必须是 'only' 或 'and'".to_string()));
        }
    }

    // 更新记录
    let updated = repository.update_agent_guide(id, &request).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "更新Agent指导文件失败"
        );
        ApiError::Database(format!("更新Agent指导文件失败: {}", e))
    })?;

    if !updated {
        error!(
            id = %id,
            "更新Agent指导文件未影响任何记录"
        );
        return Err(ApiError::Internal("更新Agent指导文件失败".to_string()));
    }

    // 获取更新后的记录
    if let Some(guide) = repository.find_by_id_decrypted(id).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "获取更新后的Agent指导文件失败"
        );
        ApiError::Database(format!("获取Agent指导文件失败: {}", e))
    })? {
        info!(
            id = %id,
            name = %guide.name,
            "Agent指导文件更新成功"
        );

        Ok(Json(ApiResponse::success_with_message(
            guide,
            "Agent指导文件更新成功".to_string()
        )))
    } else {
        error!(
            id = %id,
            "更新Agent指导文件后无法找到记录"
        );
        Err(ApiError::Internal("更新Agent指导文件后无法找到记录".to_string()))
    }
}

/// 删除Agent指导文件
pub async fn delete_agent_guide(
    State(state): State<ApiState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    info!(
        id = %id,
        "删除Agent指导文件请求"
    );

    let repository = AgentGuideRepository::new(&state.db_manager, &state.crypto_service);

    if id <= 0 {
        return Err(ApiError::Validation("无效的ID".to_string()));
    }

    // 检查记录是否存在
    let existing = repository.find_by_id::<AgentGuide>(id).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "检查Agent指导文件是否存在失败"
        );
        ApiError::Database(format!("检查Agent指导文件失败: {}", e))
    })?;

    if existing.is_none() {
        warn!(
            id = %id,
            "尝试删除不存在的Agent指导文件"
        );
        return Err(ApiError::NotFound("Agent指导文件不存在".to_string()));
    }

    // 删除记录
    let deleted = repository.delete(id).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "删除Agent指导文件失败"
        );
        ApiError::Database(format!("删除Agent指导文件失败: {}", e))
    })?;

    if !deleted {
        error!(
            id = %id,
            "删除Agent指导文件未影响任何记录"
        );
        return Err(ApiError::Internal("删除Agent指导文件失败".to_string()));
    }

    info!(
        id = %id,
        "Agent指导文件删除成功"
    );

    Ok(Json(ApiResponse::success_with_message(
        (),
        "Agent指导文件删除成功".to_string()
    )))
}

/// 获取Agent指导文件列表
pub async fn list_agent_guides(
    State(state): State<ApiState>,
    Query(query): Query<AgentGuideQuery>,
) -> Result<Json<ApiResponse<PagedResponse<AgentGuide>>>, ApiError> {
    info!(
        search = ?query.search,
        guide_type = ?query.guide_type,
        page = ?query.page,
        limit = ?query.limit,
        "获取Agent指导文件列表请求"
    );

    let repository = AgentGuideRepository::new(&state.db_manager, &state.crypto_service);

    let result = if let Some(search_term) = query.search {
        // 搜索模式
        let limit = query.limit.or(Some(50));
        let guides = repository.search_agent_guides(&search_term, limit).await.map_err(|e| {
            error!(
                error = %e,
                search_term = %search_term,
                "搜索Agent指导文件失败"
            );
            ApiError::Database(format!("搜索Agent指导文件失败: {}", e))
        })?;

        // 转换为分页响应格式
        let total = guides.len() as i64;
        let paged_result = crate::models::PagedResult::new(guides, total, 1, total);

        info!(
            search_term = %search_term,
            count = %total,
            "Agent指导文件搜索完成"
        );

        paged_result
    } else if let Some(guide_type) = query.guide_type {
        // 按类型筛选
        let guides = repository.find_by_type(&guide_type).await.map_err(|e| {
            error!(
                error = %e,
                guide_type = %guide_type,
                "根据类型获取Agent指导文件列表失败"
            );
            ApiError::Database(format!("获取Agent指导文件列表失败: {}", e))
        })?;

        // 转换为分页响应格式
        let total = guides.len() as i64;
        let paged_result = crate::models::PagedResult::new(guides, total, 1, total);

        info!(
            guide_type = %guide_type,
            count = %total,
            "按类型获取Agent指导文件列表完成"
        );

        paged_result
    } else {
        // 分页获取所有指导文件
        let pagination_params = PaginationParams {
            page: query.page,
            limit: query.limit,
            offset: query.offset,
        };

        repository.paginate::<AgentGuide>(&pagination_params).await.map_err(|e| {
            error!(
                error = %e,
                "分页获取Agent指导文件列表失败"
            );
            ApiError::Database(format!("获取Agent指导文件列表失败: {}", e))
        })?
    };

    let paged_response = crate::api::responses::PagedResponse::from_paged_result_with_message(
        result,
        "获取Agent指导文件列表成功".to_string()
    );

    Ok(Json(ApiResponse::success(paged_response)))
}

/// 验证Agent指导文件内容
pub async fn validate_agent_guide(
    State(state): State<ApiState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<bool>>, ApiError> {
    info!(
        id = %id,
        "验证Agent指导文件内容请求"
    );

    let repository = AgentGuideRepository::new(&state.db_manager, &state.crypto_service);

    if id <= 0 {
        return Err(ApiError::Validation("无效的ID".to_string()));
    }

    match repository.validate_guide_content(id).await {
        Ok(is_valid) => {
            info!(
                id = %id,
                is_valid = %is_valid,
                "Agent指导文件内容验证完成"
            );

            let message = if is_valid { "内容验证通过" } else { "内容验证失败" };
            Ok(Json(ApiResponse::success_with_message(
                is_valid,
                message.to_string()
            )))
        }
        Err(e) => {
            error!(
                error = %e,
                id = %id,
                "Agent指导文件内容验证失败"
            );
            Err(ApiError::Database(format!("内容验证失败: {}", e)))
        }
    }
}

/// 获取Agent指导文件统计信息
pub async fn get_agent_guide_stats(
    State(state): State<ApiState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    info!("获取Agent指导文件统计信息请求");

    let repository = AgentGuideRepository::new(&state.db_manager, &state.crypto_service);

    // 获取总数
    let total = repository.count().await.map_err(|e| {
        error!(
            error = %e,
            "获取Agent指导文件总数失败"
        );
        ApiError::Database(format!("获取统计信息失败: {}", e))
    })?;

    // 获取only类型数量
    let only_count = repository.count_by_type("only").await.map_err(|e| {
        error!(
            error = %e,
            "获取only类型Agent指导文件数量失败"
        );
        ApiError::Database(format!("获取统计信息失败: {}", e))
    })?;

    // 获取and类型数量
    let and_count = repository.count_by_type("and").await.map_err(|e| {
        error!(
            error = %e,
            "获取and类型Agent指导文件数量失败"
        );
        ApiError::Database(format!("获取统计信息失败: {}", e))
    })?;

    let stats = serde_json::json!({
        "total": total,
        "only_type": only_count,
        "and_type": and_count,
        "only_type_rate": if total > 0 { (only_count as f64 / total as f64 * 100.0).round() } else { 0.0 },
        "and_type_rate": if total > 0 { (and_count as f64 / total as f64 * 100.0).round() } else { 0.0 }
    });

    info!(
        total = %total,
        only_type = %only_count,
        and_type = %and_count,
        "Agent指导文件统计信息获取完成"
    );

    Ok(Json(ApiResponse::success_with_message(
        stats,
        "获取Agent指导文件统计信息成功".to_string()
    )))
}