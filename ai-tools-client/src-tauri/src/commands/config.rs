use crate::models::config::{ConfigHistory, WorkModeConfig, AppState as ConfigAppState, UpdateWorkModeRequest};
use crate::models::supplier::Supplier;
use crate::models::mcp_template::McpTemplate;
use crate::models::ApiResponse;
use tauri::State;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;

// 使用相同的应用状态
use crate::commands::supplier::AppState;

// 配置相关命令

#[tauri::command]
pub async fn backup_config(
    state: State<'_, AppState>,
    config_type: String,
    config_path: String,
    content: String,
    description: Option<String>,
) -> Result<ApiResponse<ConfigHistory>, String> {
    let pool = state.db_pool.lock().await;

    let backup = ConfigHistory::create(
        &pool,
        &config_type,
        &config_path,
        &content,
        "backup",
        description.as_deref(),
    )
    .await
    .map_err(|e| format!("创建配置备份失败: {}", e))?;

    Ok(ApiResponse::success(backup))
}

#[tauri::command]
pub async fn get_config_history(
    state: State<'_, AppState>,
    config_type: String,
    limit: Option<i64>,
) -> Result<ApiResponse<Vec<ConfigHistory>>, String> {
    let pool = state.db_pool.lock().await;

    let history = ConfigHistory::get_by_type(&pool, &config_type, limit)
        .await
        .map_err(|e| format!("获取配置历史失败: {}", e))?;

    Ok(ApiResponse::success(history))
}

#[tauri::command]
pub async fn get_latest_config_backup(
    state: State<'_, AppState>,
    config_type: String,
) -> Result<ApiResponse<Option<ConfigHistory>>, String> {
    let pool = state.db_pool.lock().await;

    let backup = ConfigHistory::get_latest(&pool, &config_type)
        .await
        .map_err(|e| format!("获取最新配置备份失败: {}", e))?;

    Ok(ApiResponse::success(backup))
}

#[tauri::command]
pub async fn restore_config_from_backup(
    state: State<'_, AppState>,
    backup_id: i64,
) -> Result<ApiResponse<ConfigHistory>, String> {
    let pool = state.db_pool.lock().await;

    // 获取备份记录
    let backup = ConfigHistory::get_by_id(&pool, backup_id)
        .await
        .map_err(|e| format!("获取备份记录失败: {}", e))?;

    if let Some(backup_record) = backup {
        // 创建恢复历史记录
        let restore_history = ConfigHistory::create(
            &pool,
            &backup_record.config_type,
            &backup_record.config_path,
            &backup_record.backup_content,
            "restore",
            Some(&format!("从备份ID {} 恢复", backup_id)),
        )
        .await
        .map_err(|e| format!("创建恢复历史记录失败: {}", e))?;

        // TODO: 实现实际的配置文件恢复逻辑
        // 这里需要根据配置类型和路径来恢复文件

        Ok(ApiResponse::success(restore_history))
    } else {
        Ok(ApiResponse::error("备份记录不存在".to_string()))
    }
}

#[tauri::command]
pub async fn cleanup_old_config_history(
    state: State<'_, AppState>,
    config_type: String,
    keep_count: i64,
) -> Result<ApiResponse<i64>, String> {
    let pool = state.db_pool.lock().await;

    let deleted_count = ConfigHistory::cleanup_old(&pool, &config_type, keep_count)
        .await
        .map_err(|e| format!("清理配置历史失败: {}", e))?;

    Ok(ApiResponse::success(deleted_count))
}

#[tauri::command]
pub async fn delete_config_history(
    state: State<'_, AppState>,
    backup_id: i64,
) -> Result<ApiResponse<bool>, String> {
    let pool = state.db_pool.lock().await;

    let deleted = ConfigHistory::delete(&pool, backup_id)
        .await
        .map_err(|e| format!("删除配置历史失败: {}", e))?;

    Ok(ApiResponse::success(deleted))
}

// 工作模式相关命令

#[tauri::command]
pub async fn get_work_mode_config(
    state: State<'_, AppState>,
    mode_name: String,
) -> Result<ApiResponse<Option<WorkModeConfig>>, String> {
    let pool = state.db_pool.lock().await;

    let config = WorkModeConfig::get_by_mode(&pool, &mode_name)
        .await
        .map_err(|e| format!("获取工作模式配置失败: {}", e))?;

    Ok(ApiResponse::success(config))
}

#[tauri::command]
pub async fn get_all_work_mode_configs(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<WorkModeConfig>>, String> {
    let pool = state.db_pool.lock().await;

    let configs = WorkModeConfig::get_all(&pool)
        .await
        .map_err(|e| format!("获取所有工作模式配置失败: {}", e))?;

    Ok(ApiResponse::success(configs))
}

#[tauri::command]
pub async fn update_work_mode_config(
    state: State<'_, AppState>,
    request: UpdateWorkModeRequest,
) -> Result<ApiResponse<WorkModeConfig>, String> {
    let pool = {
        let guard = state.db_pool.lock().await;
        guard.clone()
    };

    // 验证工作模式名称
    let temp_config = WorkModeConfig {
        id: None,
        mode_name: request.mode_name.clone(),
        active_claude_supplier_id: request.active_claude_supplier_id,
        active_codex_supplier_id: request.active_codex_supplier_id,
        mcp_template_ids: request.mcp_template_ids.as_ref().map(|ids| serde_json::to_string(ids).unwrap_or_default()),
        created_at: None,
        updated_at: None,
    };

    if let Err(e) = temp_config.validate() {
        return Ok(ApiResponse::error(e));
    }

    let updated_config = WorkModeConfig::update(&pool, request.clone())
        .await
        .map_err(|e| format!("更新工作模式配置失败: {}", e))?;

    // 同时更新应用状态中的当前模式
    ConfigAppState::set_current_mode(&pool, &request.mode_name)
        .await
        .map_err(|e| format!("更新当前模式失败: {}", e))?;

    Ok(ApiResponse::success(updated_config))
}

// 应用状态相关命令

#[tauri::command]
pub async fn get_app_state(
    state: State<'_, AppState>,
    key: String,
) -> Result<ApiResponse<Option<ConfigAppState>>, String> {
    let pool = state.db_pool.lock().await;

    let state = ConfigAppState::get(&pool, &key)
        .await
        .map_err(|e| format!("获取应用状态失败: {}", e))?;

    Ok(ApiResponse::success(state))
}

#[tauri::command]
pub async fn set_app_state(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<ApiResponse<ConfigAppState>, String> {
    let pool = state.db_pool.lock().await;

    let state = ConfigAppState::set(&pool, &key, &value)
        .await
        .map_err(|e| format!("设置应用状态失败: {}", e))?;

    Ok(ApiResponse::success(state))
}

#[tauri::command]
pub async fn get_current_mode(
    state: State<'_, AppState>,
) -> Result<ApiResponse<String>, String> {
    let pool = state.db_pool.lock().await;

    let current_mode = ConfigAppState::get_current_mode(&pool)
        .await
        .map_err(|e| format!("获取当前模式失败: {}", e))?;

    Ok(ApiResponse::success(current_mode))
}

#[tauri::command]
pub async fn get_all_app_states(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<ConfigAppState>>, String> {
    let pool = state.db_pool.lock().await;

    let states = ConfigAppState::get_all(&pool)
        .await
        .map_err(|e| format!("获取所有应用状态失败: {}", e))?;

    Ok(ApiResponse::success(states))
}

#[tauri::command]
pub async fn get_database_stats(
    state: State<'_, AppState>,
) -> Result<ApiResponse<serde_json::Value>, String> {
    let pool = {
        let guard = state.db_pool.lock().await;
        guard.clone()
    };

    // 获取数据库统计信息
    let stats = crate::services::database::Database::get_db_stats(&pool)
        .await
        .map_err(|e| format!("获取数据库统计失败: {}", e))?;

    Ok(ApiResponse::success(serde_json::to_value(&stats).unwrap_or_default()))
}

#[tauri::command]
pub async fn test_database_connection(
    state: State<'_, AppState>,
) -> Result<ApiResponse<bool>, String> {
    let pool = {
        let guard = state.db_pool.lock().await;
        guard.clone()
    };

    let result = crate::services::database::Database::test_db_connection(&pool)
        .await
        .map_err(|e| format!("数据库连接测试失败: {}", e))?;

    Ok(ApiResponse::success(result))
}

#[tauri::command]
pub async fn export_all_data(
    state: State<'_, AppState>,
) -> Result<ApiResponse<serde_json::Value>, String> {
    let pool = {
        let guard = state.db_pool.lock().await;
        guard.clone()
    };

    // 获取所有数据
    let suppliers = Supplier::get_all(&pool)
        .await
        .map_err(|e| format!("导出供应商数据失败: {}", e))?;

    let templates = McpTemplate::get_all(&pool)
        .await
        .map_err(|e| format!("导出MCP模板数据失败: {}", e))?;

    let work_modes = WorkModeConfig::get_all(&pool)
        .await
        .map_err(|e| format!("导出工作模式数据失败: {}", e))?;

    let app_states = ConfigAppState::get_all(&pool)
        .await
        .map_err(|e| format!("导出应用状态数据失败: {}", e))?;

    let export_data = serde_json::json!({
        "suppliers": suppliers,
        "mcp_templates": templates,
        "work_modes": work_modes,
        "app_states": app_states,
        "export_time": chrono::Utc::now().to_rfc3339()
    });

    Ok(ApiResponse::success(export_data))
}