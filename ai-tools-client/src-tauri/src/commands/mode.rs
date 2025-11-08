use crate::models::mode::{WorkMode, CreateWorkModeRequest, UpdateWorkModeRequest};
use crate::models::ApiResponse;
use tauri::State;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;

use crate::commands::supplier::AppState;

// 工作模式相关命令

#[tauri::command]
pub async fn list_work_modes(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<WorkMode>>, String> {
    let pool = state.db_pool.lock().await;

    // 暂时返回空列表，实际实现需要查询数据库
    Ok(ApiResponse::success(vec![]))
}

#[tauri::command]
pub async fn create_work_mode(
    state: State<'_, AppState>,
    request: CreateWorkModeRequest,
) -> Result<ApiResponse<WorkMode>, String> {
    let pool = state.db_pool.lock().await;

    // 暂时返回默认值，实际实现需要创建数据库记录
    Ok(ApiResponse::success(WorkMode {
        id: Some(1),
        name: request.name,
        description: request.description,
        supplier_config: request.supplier_config,
        created_at: Some(chrono::Utc::now()),
        updated_at: Some(chrono::Utc::now()),
    }))
}

#[tauri::command]
pub async fn update_work_mode(
    state: State<'_, AppState>,
    request: UpdateWorkModeRequest,
) -> Result<ApiResponse<Option<WorkMode>>, String> {
    let pool = state.db_pool.lock().await;

    // 暂时返回None，实际实现需要更新数据库记录
    Ok(ApiResponse::success(None))
}

#[tauri::command]
pub async fn delete_work_mode(
    state: State<'_, AppState>,
    id: i64,
) -> Result<ApiResponse<bool>, String> {
    let pool = state.db_pool.lock().await;

    // 暂时返回false，实际实现需要删除数据库记录
    Ok(ApiResponse::success(false))
}