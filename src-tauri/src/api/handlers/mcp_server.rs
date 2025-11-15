// MCP服务器API处理器
//
// 提供MCP服务器的HTTP API接口实现

use axum::{
    extract::{Path, Query, State},
    response::Json,
    Router,
};
use serde::Deserialize;
use tracing::{error, info, warn};

use crate::api::error::ApiError;
use crate::api::responses::{ApiResponse, PagedResponse};
use crate::models::{CreateMcpServerRequest, McpServer, PaginationParams, UpdateMcpServerRequest};
use crate::repositories::{BaseRepository, McpServerRepository};

/// 重用API服务器的ApiState
pub use super::super::server::ApiState;

/// 查询参数
#[derive(Debug, Deserialize)]
pub struct McpServerQuery {
    pub search: Option<String>,
    pub server_type: Option<String>,
    pub active_only: Option<bool>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// 创建MCP服务器
pub async fn create_mcp_server(
    State(state): State<ApiState>,
    Json(request): Json<CreateMcpServerRequest>,
) -> Result<Json<ApiResponse<McpServer>>, ApiError> {
    info!(
        name = %request.name,
        server_type = ?request.r#type,
        command = %request.command,
        "创建MCP服务器请求"
    );

    // 创建Repository
    let repository = McpServerRepository::new(&state.db_manager, &state.crypto_service);

    // 验证请求
    if request.name.trim().is_empty() {
        return Err(ApiError::Validation("服务器名称不能为空".to_string()));
    }

    if request.command.trim().is_empty() {
        return Err(ApiError::Validation("启动命令不能为空".to_string()));
    }

    if let Some(timeout) = request.timeout {
        if timeout <= 0 {
            return Err(ApiError::Validation("超时时间必须大于0".to_string()));
        }
    }

    // 验证服务器类型（如果提供）
    if let Some(ref server_type) = request.r#type {
        if !["stdio", "sse", "websocket"].contains(&server_type.as_str()) {
            return Err(ApiError::Validation(
                "服务器类型必须是 'stdio'、'sse' 或 'websocket'".to_string(),
            ));
        }
    }

    // 创建记录
    let id = repository.create_mcp_server(&request).await.map_err(|e| {
        error!(
            error = %e,
            name = %request.name,
            "创建MCP服务器失败"
        );
        ApiError::Database { message: format!("创建MCP服务器失败: {}", e) }
    })?;

    // 获取创建的记录
    if let Some(server) = repository.find_by_id_parsed(id).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "获取新创建的MCP服务器失败"
        );
        ApiError::Database { message: format!("获取MCP服务器失败: {}", e) }
    })? {
        info!(
            id = %id,
            name = %server.name,
            "MCP服务器创建成功"
        );

        Ok(Json(ApiResponse::success_with_message(
            server,
            "MCP服务器创建成功".to_string(),
        )))
    } else {
        error!(
            id = %id,
            "创建MCP服务器后无法找到记录"
        );
        Err(ApiError::Internal {
            message: "创建MCP服务器后无法找到记录".to_string(),
        })
    }
}

/// 获取MCP服务器详情
pub async fn get_mcp_server(
    State(state): State<ApiState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<McpServer>>, ApiError> {
    info!(
        id = %id,
        "获取MCP服务器详情请求"
    );

    let repository = McpServerRepository::new(&state.db_manager, &state.crypto_service);

    if id <= 0 {
        return Err(ApiError::Validation("无效的ID".to_string()));
    }

    match repository.find_by_id_parsed(id).await {
        Ok(Some(server)) => {
            info!(
                id = %id,
                name = %server.name,
                "获取MCP服务器详情成功"
            );

            Ok(Json(ApiResponse::success_with_message(
                server,
                "获取MCP服务器详情成功".to_string(),
            )))
        }
        Ok(None) => {
            warn!(
                id = %id,
                "MCP服务器不存在"
            );
            Err(ApiError::NotFound { resource: "MCP服务器不存在".to_string() })
        }
        Err(e) => {
            error!(
                error = %e,
                id = %id,
                "获取MCP服务器详情失败"
            );
            Err(ApiError::Database { message: format!("获取MCP服务器失败: {}", e) })
        }
    }
}

/// 更新MCP服务器
pub async fn update_mcp_server(
    State(state): State<ApiState>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateMcpServerRequest>,
) -> Result<Json<ApiResponse<McpServer>>, ApiError> {
    info!(
        id = %id,
        "更新MCP服务器请求"
    );

    let repository = McpServerRepository::new(&state.db_manager, &state.crypto_service);

    if id <= 0 {
        return Err(ApiError::Validation("无效的ID".to_string()));
    }

    // 检查记录是否存在
    let existing = repository.find_by_id_parsed(id).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "检查MCP服务器是否存在失败"
        );
        ApiError::Database { message: format!("检查MCP服务器失败: {}", e) }
    })?;

    if existing.is_none() {
        warn!(
            id = %id,
            "尝试更新不存在的MCP服务器"
        );
        return Err(ApiError::NotFound { resource: "MCP服务器不存在".to_string() });
    }

    // 验证更新数据
    if let Some(ref name) = request.name {
        if name.trim().is_empty() {
            return Err(ApiError::Validation("服务器名称不能为空".to_string()));
        }
    }

    if let Some(ref command) = request.command {
        if command.trim().is_empty() {
            return Err(ApiError::Validation("启动命令不能为空".to_string()));
        }
    }

    if let Some(timeout) = request.timeout {
        if timeout <= 0 {
            return Err(ApiError::Validation("超时时间必须大于0".to_string()));
        }
    }

    if let Some(ref server_type) = request.r#type {
        if !["stdio", "sse", "websocket"].contains(&server_type.as_str()) {
            return Err(ApiError::Validation(
                "服务器类型必须是 'stdio'、'sse' 或 'websocket'".to_string(),
            ));
        }
    }

    // 更新记录
    let updated = repository.update_mcp_server(id, &request).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "更新MCP服务器失败"
        );
        ApiError::Database { message: format!("更新MCP服务器失败: {}", e) }
    })?;

    if !updated {
        error!(
            id = %id,
            "更新MCP服务器未影响任何记录"
        );
        return Err(ApiError::Internal { message: "更新MCP服务器失败".to_string() });
    }

    // 获取更新后的记录
    if let Some(server) = repository.find_by_id_parsed(id).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "获取更新后的MCP服务器失败"
        );
        ApiError::Database { message: format!("获取MCP服务器失败: {}", e) }
    })? {
        info!(
            id = %id,
            name = %server.name,
            "MCP服务器更新成功"
        );

        Ok(Json(ApiResponse::success_with_message(
            server,
            "MCP服务器更新成功".to_string(),
        )))
    } else {
        error!(
            id = %id,
            "更新MCP服务器后无法找到记录"
        );
        Err(ApiError::Internal {
            message: "更新MCP服务器后无法找到记录".to_string(),
        })
    }
}

/// 删除MCP服务器
pub async fn delete_mcp_server(
    State(state): State<ApiState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    info!(
        id = %id,
        "删除MCP服务器请求"
    );

    let repository = McpServerRepository::new(&state.db_manager, &state.crypto_service);

    if id <= 0 {
        return Err(ApiError::Validation("无效的ID".to_string()));
    }

    // 检查记录是否存在
    let existing = repository.find_by_id::<McpServer>(id).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "检查MCP服务器是否存在失败"
        );
        ApiError::Database { message: format!("检查MCP服务器失败: {}", e) }
    })?;

    if existing.is_none() {
        warn!(
            id = %id,
            "尝试删除不存在的MCP服务器"
        );
        return Err(ApiError::NotFound { resource: "MCP服务器不存在".to_string() });
    }

    // 删除记录
    let deleted = repository.delete(id).await.map_err(|e| {
        error!(
            error = %e,
            id = %id,
            "删除MCP服务器失败"
        );
        ApiError::Database { message: format!("删除MCP服务器失败: {}", e) }
    })?;

    if !deleted {
        error!(
            id = %id,
            "删除MCP服务器未影响任何记录"
        );
        return Err(ApiError::Internal { message: "删除MCP服务器失败".to_string() });
    }

    info!(
        id = %id,
        "MCP服务器删除成功"
    );

    Ok(Json(ApiResponse::success_with_message(
        (),
        "MCP服务器删除成功".to_string(),
    )))
}

/// 获取MCP服务器列表
pub async fn list_mcp_servers(
    State(state): State<ApiState>,
    Query(query): Query<McpServerQuery>,
) -> Result<Json<ApiResponse<PagedResponse<McpServer>>>, ApiError> {
    info!(
        search = ?query.search,
        server_type = ?query.server_type,
        active_only = ?query.active_only,
        page = ?query.page,
        limit = ?query.limit,
        "获取MCP服务器列表请求"
    );

    let repository = McpServerRepository::new(&state.db_manager, &state.crypto_service);

    let result = if let Some(search_term) = query.search {
        // 搜索模式
        let limit = query.limit.or(Some(50));
        let servers = repository.search_mcp_servers(&search_term, limit).await.map_err(|e| {
            error!(
                error = %e,
                search_term = %search_term,
                "搜索MCP服务器失败"
            );
            ApiError::Database { message: format!("搜索MCP服务器失败: {}", e) }
        })?;

        // 转换为分页响应格式
        let total = servers.len() as i64;
        let paged_result = crate::models::PagedResult::new(servers, total, 1, total);

        info!(
            search_term = %search_term,
            count = %total,
            "MCP服务器搜索完成"
        );

        paged_result
    } else if let Some(server_type) = query.server_type {
        // 按类型筛选
        let servers = repository.find_by_type(&server_type).await.map_err(|e| {
            error!(
                error = %e,
                server_type = %server_type,
                "根据类型获取MCP服务器列表失败"
            );
            ApiError::Database { message: format!("获取MCP服务器列表失败: {}", e) }
        })?;

        // 转换为分页响应格式
        let total = servers.len() as i64;
        let paged_result = crate::models::PagedResult::new(servers, total, 1, total);

        info!(
            server_type = %server_type,
            count = %total,
            "按类型获取MCP服务器列表完成"
        );

        paged_result
    } else if query.active_only.unwrap_or(false) {
        // 仅获取活跃服务器
        let servers = repository.list_active_servers().await.map_err(|e| {
            error!(
                error = %e,
                "获取活跃MCP服务器列表失败"
            );
            ApiError::Database { message: format!("获取MCP服务器列表失败: {}", e) }
        })?;

        // 转换为分页响应格式
        let total = servers.len() as i64;
        let paged_result = crate::models::PagedResult::new(servers, total, 1, total);

        info!(
            count = %total,
            "活跃MCP服务器列表获取完成"
        );

        paged_result
    } else {
        // 分页获取所有服务器
        let pagination_params =
            PaginationParams { page: query.page, limit: query.limit, offset: query.offset };

        repository.paginate::<McpServer>(&pagination_params).await.map_err(|e| {
            error!(
                error = %e,
                "分页获取MCP服务器列表失败"
            );
            ApiError::Database { message: format!("获取MCP服务器列表失败: {}", e) }
        })?
    };

    let paged_response = crate::api::responses::PagedResponse::from_paged_result_with_message(
        result,
        "获取MCP服务器列表成功".to_string(),
    );

    Ok(Json(ApiResponse::success(paged_response)))
}

/// 测试MCP服务器配置
pub async fn test_mcp_server(
    State(state): State<ApiState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<bool>>, ApiError> {
    info!(
        id = %id,
        "测试MCP服务器配置请求"
    );

    let repository = McpServerRepository::new(&state.db_manager, &state.crypto_service);

    if id <= 0 {
        return Err(ApiError::Validation("无效的ID".to_string()));
    }

    match repository.test_server_config(id).await {
        Ok(is_valid) => {
            info!(
                id = %id,
                is_valid = %is_valid,
                "MCP服务器配置测试完成"
            );

            let message = if is_valid {
                "配置测试通过"
            } else {
                "配置测试失败"
            };
            Ok(Json(ApiResponse::success_with_message(
                is_valid,
                message.to_string(),
            )))
        }
        Err(e) => {
            error!(
                error = %e,
                id = %id,
                "MCP服务器配置测试失败"
            );
            Err(ApiError::Database { message: format!("配置测试失败: {}", e) })
        }
    }
}

/// 获取MCP服务器统计信息
pub async fn get_mcp_server_stats(
    State(state): State<ApiState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    info!("获取MCP服务器统计信息请求");

    let repository = McpServerRepository::new(&state.db_manager, &state.crypto_service);

    // 获取总数
    let total = repository.count().await.map_err(|e| {
        error!(
            error = %e,
            "获取MCP服务器总数失败"
        );
        ApiError::Database { message: format!("获取统计信息失败: {}", e) }
    })?;

    // 获取stdio类型数量
    let stdio_count = repository.count_by_type("stdio").await.map_err(|e| {
        error!(
            error = %e,
            "获取stdio类型MCP服务器数量失败"
        );
        ApiError::Database { message: format!("获取统计信息失败: {}", e) }
    })?;

    // 获取sse类型数量
    let sse_count = repository.count_by_type("sse").await.map_err(|e| {
        error!(
            error = %e,
            "获取sse类型MCP服务器数量失败"
        );
        ApiError::Database { message: format!("获取统计信息失败: {}", e) }
    })?;

    // 获取活跃服务器数量
    let active_servers = repository.list_active_servers().await.map_err(|e| {
        error!(
            error = %e,
            "获取活跃MCP服务器数量失败"
        );
        ApiError::Database { message: format!("获取统计信息失败: {}", e) }
    })?;
    let active_count = active_servers.len() as i64;

    let stats = serde_json::json!({
        "total": total,
        "stdio_type": stdio_count,
        "sse_type": sse_count,
        "active_count": active_count,
        "inactive_count": total - active_count,
        "stdio_type_rate": if total > 0 { (stdio_count as f64 / total as f64 * 100.0).round() } else { 0.0 },
        "sse_type_rate": if total > 0 { (sse_count as f64 / total as f64 * 100.0).round() } else { 0.0 },
        "active_rate": if total > 0 { (active_count as f64 / total as f64 * 100.0).round() } else { 0.0 }
    });

    info!(
        total = %total,
        stdio_type = %stdio_count,
        sse_type = %sse_count,
        active_count = %active_count,
        "MCP服务器统计信息获取完成"
    );

    Ok(Json(ApiResponse::success_with_message(
        stats,
        "获取MCP服务器统计信息成功".to_string(),
    )))
}

/// MCP服务器API路由
pub fn routes() -> Router<ApiState> {
    use axum::routing::{delete, get, post, put};

    Router::new()
        // 创建MCP服务器
        .route("/", post(create_mcp_server))
        // 获取MCP服务器列表
        .route("/", get(list_mcp_servers))
        // 获取MCP服务器统计信息
        .route("/stats", get(get_mcp_server_stats))
        // 获取单个MCP服务器
        .route("/:id", get(get_mcp_server))
        // 更新MCP服务器
        .route("/:id", put(update_mcp_server))
        // 删除MCP服务器
        .route("/:id", delete(delete_mcp_server))
        // 测试MCP服务器配置
        .route("/:id/test", get(test_mcp_server))
}
