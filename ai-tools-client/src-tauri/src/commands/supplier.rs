use crate::models::supplier::{Supplier, CreateSupplierRequest, UpdateSupplierRequest, ConnectionTestResult};
use crate::models::ApiResponse;
use tauri::State;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;

// 应用状态
pub struct AppState {
    pub db_pool: Arc<Mutex<sqlx::SqlitePool>>,
}

// 供应商相关命令

#[tauri::command]
pub async fn list_suppliers(
    state: State<'_, AppState>,
    supplier_type: Option<String>,
) -> Result<ApiResponse<Vec<Supplier>>, String> {
    let pool = {
        let guard = state.db_pool.lock().await;
        guard.clone()
    };

    let suppliers = if let Some(supplier_type) = supplier_type {
        Supplier::get_by_type(&pool, &supplier_type)
            .await
            .map_err(|e| format!("获取供应商列表失败: {}", e))?
    } else {
        Supplier::get_all(&pool)
            .await
            .map_err(|e| format!("获取供应商列表失败: {}", e))?
    };

    Ok(ApiResponse::success(suppliers))
}

#[tauri::command]
pub async fn create_supplier(
    state: State<'_, AppState>,
    request: CreateSupplierRequest,
) -> Result<ApiResponse<Supplier>, String> {
    let pool = state.db_pool.lock().await;

    // 验证请求
    let supplier = Supplier {
        id: None,
        r#type: request.r#type.clone(),
        name: request.name.clone(),
        base_url: request.base_url.clone(),
        auth_token: request.auth_token.clone(),
        timeout_ms: request.timeout_ms,
        auto_update: request.auto_update.map(|b| if b { 1 } else { 0 }),
        opus_model: request.opus_model.clone(),
        sonnet_model: request.sonnet_model.clone(),
        haiku_model: request.haiku_model.clone(),
        is_active: Some(0),
        sort_order: Some(0),
        created_at: None,
        updated_at: None,
    };

    if let Err(e) = supplier.validate() {
        return Ok(ApiResponse::error(e));
    }

    let created_supplier = Supplier::create(&pool, request)
        .await
        .map_err(|e| format!("创建供应商失败: {}", e))?;

    Ok(ApiResponse::success(created_supplier))
}

#[tauri::command]
pub async fn update_supplier(
    state: State<'_, AppState>,
    request: UpdateSupplierRequest,
) -> Result<ApiResponse<Option<Supplier>>, String> {
    let pool = state.db_pool.lock().await;

    // 检查供应商是否存在
    if Supplier::get_by_id(&pool, request.id).await.map_err(|e| format!("查询供应商失败: {}", e))?.is_none() {
        return Ok(ApiResponse::error("供应商不存在".to_string()));
    }

    let updated_supplier = Supplier::update(&pool, request)
        .await
        .map_err(|e| format!("更新供应商失败: {}", e))?;

    Ok(ApiResponse::success(updated_supplier))
}

#[tauri::command]
pub async fn delete_supplier(
    state: State<'_, AppState>,
    id: i64,
) -> Result<ApiResponse<bool>, String> {
    let pool = state.db_pool.lock().await;

    let deleted = Supplier::delete(&pool, id)
        .await
        .map_err(|e| format!("删除供应商失败: {}", e))?;

    if deleted {
        Ok(ApiResponse::success(true))
    } else {
        Ok(ApiResponse::error("供应商不存在或删除失败".to_string()))
    }
}

#[tauri::command]
pub async fn get_supplier_by_id(
    state: State<'_, AppState>,
    id: i64,
) -> Result<ApiResponse<Option<Supplier>>, String> {
    let pool = state.db_pool.lock().await;

    let supplier = Supplier::get_by_id(&pool, id)
        .await
        .map_err(|e| format!("获取供应商失败: {}", e))?;

    Ok(ApiResponse::success(supplier))
}

#[tauri::command]
pub async fn set_active_supplier(
    state: State<'_, AppState>,
    id: i64,
    is_active: bool,
) -> Result<ApiResponse<bool>, String> {
    let pool = state.db_pool.lock().await;

    // 检查供应商是否存在
    if Supplier::get_by_id(&pool, id).await.map_err(|e| format!("查询供应商失败: {}", e))?.is_none() {
        return Ok(ApiResponse::error("供应商不存在".to_string()));
    }

    let success = Supplier::set_active(&pool, id, is_active)
        .await
        .map_err(|e| format!("设置激活状态失败: {}", e))?;

    Ok(ApiResponse::success(success))
}

#[tauri::command]
pub async fn test_supplier_connection(
    state: State<'_, AppState>,
    id: i64,
) -> Result<ApiResponse<ConnectionTestResult>, String> {
    let pool = state.db_pool.lock().await;

    let supplier = Supplier::get_by_id(&pool, id)
        .await
        .map_err(|e| format!("获取供应商失败: {}", e))?;

    if let Some(supplier) = supplier {
        let result = supplier.test_connection().await;
        Ok(ApiResponse::success(result))
    } else {
        Ok(ApiResponse::error("供应商不存在".to_string()))
    }
}

#[tauri::command]
pub async fn validate_supplier_config(
    state: State<'_, AppState>,
    request: CreateSupplierRequest,
) -> Result<ApiResponse<bool>, String> {
    let supplier = Supplier {
        id: None,
        r#type: request.r#type.clone(),
        name: request.name.clone(),
        base_url: request.base_url.clone(),
        auth_token: request.auth_token.clone(),
        timeout_ms: request.timeout_ms,
        auto_update: request.auto_update.map(|b| if b { 1 } else { 0 }),
        opus_model: request.opus_model.clone(),
        sonnet_model: request.sonnet_model.clone(),
        haiku_model: request.haiku_model.clone(),
        is_active: Some(0),
        sort_order: Some(0),
        created_at: None,
        updated_at: None,
    };

    match supplier.validate() {
        Ok(()) => Ok(ApiResponse::success(true)),
        Err(e) => Ok(ApiResponse::error(e)),
    }
}

#[tauri::command]
pub async fn get_supplier_stats(
    state: State<'_, AppState>,
) -> Result<ApiResponse<serde_json::Value>, String> {
    let pool = state.db_pool.lock().await;

    // 获取Claude供应商数量
    let claude_count = Supplier::get_by_type(&pool, "claude")
        .await
        .map_err(|e| format!("获取Claude供应商失败: {}", e))?
        .len() as i64;

    // 获取Codex供应商数量
    let codex_count = Supplier::get_by_type(&pool, "codex")
        .await
        .map_err(|e| format!("获取Codex供应商失败: {}", e))?
        .len() as i64;

    // 获取激活的供应商
    let active_claude = Supplier::get_active(&pool, "claude")
        .await
        .map_err(|e| format!("获取激活的Claude供应商失败: {}", e))?;

    let active_codex = Supplier::get_active(&pool, "codex")
        .await
        .map_err(|e| format!("获取激活的Codex供应商失败: {}", e))?;

    let stats = serde_json::json!({
        "claude": claude_count,
        "codex": codex_count,
        "total": claude_count + codex_count,
        "active_claude": active_claude.map(|s| s.name),
        "active_codex": active_codex.map(|s| s.name)
    });

    Ok(ApiResponse::success(stats))
}

#[tauri::command]
pub async fn import_suppliers(
    state: State<'_, AppState>,
    suppliers: Vec<CreateSupplierRequest>,
) -> Result<ApiResponse<Vec<Supplier>>, String> {
    let pool = state.db_pool.lock().await;
    let mut created_suppliers = Vec::new();
    let mut errors = Vec::new();

    for (index, request) in suppliers.into_iter().enumerate() {
        // 验证每个供应商
        let supplier = Supplier {
            id: None,
            r#type: request.r#type.clone(),
            name: request.name.clone(),
            base_url: request.base_url.clone(),
            auth_token: request.auth_token.clone(),
            timeout_ms: request.timeout_ms,
            auto_update: request.auto_update.map(|b| if b { 1 } else { 0 }),
            opus_model: request.opus_model.clone(),
            sonnet_model: request.sonnet_model.clone(),
            haiku_model: request.haiku_model.clone(),
            is_active: Some(0),
            sort_order: Some(index as i64),
            created_at: None,
            updated_at: None,
        };

        match supplier.validate() {
            Ok(()) => {
                match Supplier::create(&pool, request).await {
                    Ok(created) => created_suppliers.push(created),
                    Err(e) => errors.push(format!("导入供应商 '{}' 失败: {}", supplier.name, e)),
                }
            }
            Err(e) => errors.push(format!("供应商 '{}' 验证失败: {}", supplier.name, e)),
        }
    }

    if !errors.is_empty() {
        return Ok(ApiResponse::error(format!("导入过程中发生错误: {}", errors.join("; "))));
    }

    Ok(ApiResponse::success(created_suppliers))
}

#[tauri::command]
pub async fn export_suppliers(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<Supplier>>, String> {
    let pool = state.db_pool.lock().await;

    let suppliers = Supplier::get_all(&pool)
        .await
        .map_err(|e| format!("导出供应商失败: {}", e))?;

    Ok(ApiResponse::success(suppliers))
}