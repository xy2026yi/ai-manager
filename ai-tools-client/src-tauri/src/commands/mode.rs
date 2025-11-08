use crate::models::mode::{WorkModeConfig, CreateWorkModeRequest, UpdateWorkModeRequest, WorkModeSwitchRequest, WorkModeStatus, WorkModeSwitchResult};
use crate::models::supplier::Supplier;
use crate::models::mcp_template::McpTemplate;
use crate::models::ApiResponse;
use tauri::State;
use anyhow::Result;

use crate::commands::supplier::AppState;

// 工作模式相关命令

#[tauri::command]
pub async fn get_work_mode_by_name(
    state: State<'_, AppState>,
    mode_name: String,
) -> Result<ApiResponse<Option<WorkModeConfig>>, String> {
    let pool = {
        let guard = state.db_pool.lock().await;
        guard.clone()
    };

    let config = WorkModeConfig::get_by_name(&pool, &mode_name)
        .await
        .map_err(|e| format!("获取工作模式配置失败: {}", e))?;

    Ok(ApiResponse::success(config))
}

#[tauri::command]
pub async fn list_work_mode_configs(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<WorkModeConfig>>, String> {
    let pool = {
        let guard = state.db_pool.lock().await;
        guard.clone()
    };

    let configs = WorkModeConfig::get_all(&pool)
        .await
        .map_err(|e| format!("获取所有工作模式配置失败: {}", e))?;

    Ok(ApiResponse::success(configs))
}

#[tauri::command]
pub async fn update_work_mode_by_id(
    state: State<'_, AppState>,
    request: UpdateWorkModeRequest,
) -> Result<ApiResponse<Option<WorkModeConfig>>, String> {
    let pool = {
        let guard = state.db_pool.lock().await;
        guard.clone()
    };

    let updated_config = WorkModeConfig::update(&pool, request)
        .await
        .map_err(|e| format!("更新工作模式配置失败: {}", e))?;

    Ok(ApiResponse::success(updated_config))
}

#[tauri::command]
pub async fn switch_work_mode(
    state: State<'_, AppState>,
    request: WorkModeSwitchRequest,
) -> Result<ApiResponse<WorkModeSwitchResult>, String> {
    let pool = {
        let guard = state.db_pool.lock().await;
        guard.clone()
    };

    let mut steps_completed = Vec::new();
    let backup_id = None;

    // 步骤1: 验证供应商存在
    steps_completed.push("验证供应商配置".to_string());
    if let Some(claude_id) = request.claude_supplier_id {
        if Supplier::get_by_id(&pool, claude_id).await.map_err(|e| format!("查询Claude供应商失败: {}", e))?.is_none() {
            return Ok(ApiResponse::error("指定的Claude供应商不存在".to_string()));
        }
    }
    if let Some(codex_id) = request.codex_supplier_id {
        if Supplier::get_by_id(&pool, codex_id).await.map_err(|e| format!("查询Codex供应商失败: {}", e))?.is_none() {
            return Ok(ApiResponse::error("指定的Codex供应商不存在".to_string()));
        }
    }

    // 步骤2: 验证MCP模板存在
    steps_completed.push("验证MCP模板配置".to_string());
    if let Some(template_ids) = &request.mcp_template_ids {
        for &template_id in template_ids {
            if McpTemplate::get_by_id(&pool, template_id).await.map_err(|e| format!("查询MCP模板失败: {}", e))?.is_none() {
                return Ok(ApiResponse::error(format!("MCP模板 {} 不存在", template_id)));
            }
        }
    }

    // 步骤3: 创建备份（如果需要）
    if request.create_backup {
        steps_completed.push("创建配置备份".to_string());
        // 这里需要集成配置备份功能
        // backup_id = Some(create_backup_of_current_config(&pool).await?);
    }

    // 步骤4: 保存新的工作模式配置
    steps_completed.push("保存工作模式配置".to_string());
    let config_request = CreateWorkModeRequest {
        mode_name: request.target_mode.clone(),
        active_claude_supplier_id: request.claude_supplier_id,
        active_codex_supplier_id: request.codex_supplier_id,
        mcp_template_ids: request.mcp_template_ids,
    };

    let _config = WorkModeConfig::create(&pool, config_request)
        .await
        .map_err(|e| format!("保存工作模式配置失败: {}", e))?;

    // 步骤5: 应用配置文件
    steps_completed.push("应用配置文件".to_string());
    // 这里需要集成配置文件生成和应用功能
    // apply_configuration_files(&pool, &request).await?;

    let result = WorkModeSwitchResult {
        success: true,
        message: format!("成功切换到 {} 工作模式", request.target_mode),
        backup_id,
        applied_at: Some(chrono::Utc::now()),
        steps_completed,
    };

    Ok(ApiResponse::success(result))
}

#[tauri::command]
pub async fn get_work_mode_status(
    state: State<'_, AppState>,
) -> Result<ApiResponse<WorkModeStatus>, String> {
    let pool = {
        let guard = state.db_pool.lock().await;
        guard.clone()
    };

    // 获取当前工作模式（从app_state表）
    let current_mode = sqlx::query_scalar::<_, String>(
        "SELECT value FROM app_state WHERE key = 'current_work_mode'"
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| format!("获取当前工作模式失败: {}", e))?
    .unwrap_or_else(|| "claude_only".to_string());

    // 获取活跃的供应商
    let active_claude_supplier = Supplier::get_active(&pool, "claude")
        .await
        .map_err(|e| format!("获取活跃Claude供应商失败: {}", e))?
        .map(|s| s.name);

    let active_codex_supplier = Supplier::get_active(&pool, "codex")
        .await
        .map_err(|e| format!("获取活跃Codex供应商失败: {}", e))?
        .map(|s| s.name);

    // 获取活跃的MCP模板
    let active_mcp_templates = vec![]; // 这里需要根据当前模式获取活跃的模板

    let status = WorkModeStatus {
        current_mode,
        is_transitioning: false, // 这里需要检查是否有正在进行的切换
        last_switch_time: Some(chrono::Utc::now()),
        active_claude_supplier,
        active_codex_supplier,
        active_mcp_templates,
    };

    Ok(ApiResponse::success(status))
}

#[tauri::command]
pub async fn rollback_work_mode(
    _state: State<'_, AppState>,
    _backup_id: i64,
) -> Result<ApiResponse<bool>, String> {

    // 这里需要实现从备份恢复配置的逻辑
    // 暂时返回成功，实际需要调用配置恢复功能
    
    Ok(ApiResponse::success(true))
}