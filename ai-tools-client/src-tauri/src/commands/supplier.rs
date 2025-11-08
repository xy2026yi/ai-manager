use crate::models::supplier::{Supplier, CreateSupplierRequest, UpdateSupplierRequest, ConnectionTestResult, SupplierHealth, SupplierSwitchProgress, SupplierSwitchRequest, SupplierSwitchResult, FailoverConfig};
use crate::models::ApiResponse;
use tauri::State;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;
use chrono::Utc;
use std::time::{Duration, Instant};

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
        // 健康检查字段（初始值）
        is_healthy: None,
        last_check_time: None,
        response_time: None,
        consecutive_failures: None,
        uptime_percentage: None,
        total_requests: None,
        failed_requests: None,
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
        // 健康检查字段（初始值）
        is_healthy: None,
        last_check_time: None,
        response_time: None,
        consecutive_failures: None,
        uptime_percentage: None,
        total_requests: None,
        failed_requests: None,
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
            // 健康检查字段（初始值）
            is_healthy: None,
            last_check_time: None,
            response_time: None,
            consecutive_failures: None,
            uptime_percentage: None,
            total_requests: None,
            failed_requests: None,
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

// 健康检查相关命令

#[tauri::command]
pub async fn check_supplier_health(
    state: State<'_, AppState>,
    supplier_id: i64,
) -> Result<ApiResponse<SupplierHealth>, String> {
    let pool = state.db_pool.lock().await;

    let supplier = Supplier::get_by_id(&pool, supplier_id)
        .await
        .map_err(|e| format!("获取供应商失败: {}", e))?;

    if let Some(supplier) = supplier {
        // 执行健康检查
        let connection_result = supplier.test_connection().await;
        let now = Utc::now();

        // 更新供应商健康状态
        let is_healthy = connection_result.success;
        let response_time = connection_result.response_time.unwrap_or(0);

        // 计算健康状态
        let consecutive_failures = if is_healthy { 0 } else {
            // 这里应该从数据库读取当前连续失败次数，然后+1
            1 // 简化实现
        };

        let status = if is_healthy {
            crate::models::supplier::HealthStatus::Healthy
        } else if consecutive_failures < 3 {
            crate::models::supplier::HealthStatus::Degraded
        } else {
            crate::models::supplier::HealthStatus::Unhealthy
        };

        let health = SupplierHealth {
            supplier_id,
            is_healthy,
            last_check_time: now,
            response_time,
            consecutive_failures,
            uptime_percentage: if is_healthy { 100.0 } else { 0.0 }, // 简化计算
            total_requests: 1,
            failed_requests: if is_healthy { 0 } else { 1 },
            status,
            error_message: connection_result.error,
        };

        // TODO: 更新数据库中的健康状态
        // 这里可以添加更新数据库的逻辑

        Ok(ApiResponse::success(health))
    } else {
        Ok(ApiResponse::error("供应商不存在".to_string()))
    }
}

#[tauri::command]
pub async fn check_all_suppliers_health(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<SupplierHealth>>, String> {
    let pool = state.db_pool.lock().await;

    let suppliers = Supplier::get_all(&pool)
        .await
        .map_err(|e| format!("获取供应商列表失败: {}", e))?;

    let mut health_results = Vec::new();

    for supplier in suppliers {
        if let Some(id) = supplier.id {
            let health = check_supplier_health(state.clone(), id).await?;
            if let Some(health_data) = health.data {
                health_results.push(health_data);
            }
        }
    }

    Ok(ApiResponse::success(health_results))
}

#[tauri::command]
pub async fn switch_supplier(
    state: State<'_, AppState>,
    request: SupplierSwitchRequest,
) -> Result<ApiResponse<SupplierSwitchResult>, String> {
    let pool = state.db_pool.lock().await;

    // 验证供应商存在
    let from_supplier = Supplier::get_by_id(&pool, request.from_supplier_id)
        .await
        .map_err(|e| format!("查询源供应商失败: {}", e))?;

    let to_supplier = Supplier::get_by_id(&pool, request.to_supplier_id)
        .await
        .map_err(|e| format!("查询目标供应商失败: {}", e))?;

    if from_supplier.is_none() || to_supplier.is_none() {
        return Ok(ApiResponse::error("供应商不存在".to_string()));
    }

    // 执行切换
    let start_time = Instant::now();
    let switch_time = Utc::now();

    // 设置目标供应商为激活状态
    let success = Supplier::set_active(&pool, request.to_supplier_id, true)
        .await
        .map_err(|e| format!("设置目标供应商失败: {}", e))?;

    if success {
        let duration = start_time.elapsed().as_millis() as u32;

        let result = SupplierSwitchResult {
            success: true,
            message: format!("成功从供应商 {} 切换到供应商 {}", request.from_supplier_id, request.to_supplier_id),
            from_supplier_id: request.from_supplier_id,
            to_supplier_id: request.to_supplier_id,
            switch_time,
            rollback_available: true,
            backup_id: None,
            error: None,
        };

        Ok(ApiResponse::success(result))
    } else {
        let result = SupplierSwitchResult {
            success: false,
            message: "供应商切换失败".to_string(),
            from_supplier_id: request.from_supplier_id,
            to_supplier_id: request.to_supplier_id,
            switch_time,
            rollback_available: false,
            backup_id: None,
            error: Some("设置激活状态失败".to_string()),
        };

        Ok(ApiResponse::error("供应商切换失败".to_string()))
    }
}

#[tauri::command]
pub async fn auto_failover(
    state: State<'_, AppState>,
    supplier_type: String,
) -> Result<ApiResponse<SupplierSwitchResult>, String> {
    let pool = state.db_pool.lock().await;

    // 获取故障转移配置
    let config_result = get_failover_config(state.clone(), supplier_type.clone()).await?;
    let config = match config_result.data {
        Some(c) => c,
        None => return Ok(ApiResponse::error("无法获取故障转移配置".to_string())),
    };

    // 如果故障转移未启用，直接返回
    if !config.enabled {
        return Ok(ApiResponse::error("自动故障转移已禁用".to_string()));
    }

    // 获取当前激活的供应商
    let current_active = Supplier::get_active(&pool, &supplier_type)
        .await
        .map_err(|e| format!("获取当前激活供应商失败: {}", e))?;

    if let Some(current_supplier) = current_active {
        // 检查当前供应商健康状态
        let health_result = check_supplier_health(state.clone(), current_supplier.id.unwrap())
            .await?;

        if let Some(health) = health_result.data {
            // 智能故障转移决策
            let should_failover = evaluate_failover_conditions(&health, &config);

            if should_failover {
                // 获取所有备用供应商并评估其健康状况
                let backup_suppliers = Supplier::get_by_type(&pool, &supplier_type)
                    .await
                    .map_err(|e| format!("获取备用供应商失败: {}", e))?;

                // 评估备用供应商并找到最佳候选
                let mut best_candidate: Option<(Supplier, SupplierHealth, f64)> = None;
                let mut candidates = Vec::new();

                for supplier in backup_suppliers {
                    if let Some(id) = supplier.id {
                        if id != current_supplier.id.unwrap() {
                            if let Ok(backup_health_result) = check_supplier_health(state.clone(), id).await {
                                if let Some(h) = backup_health_result.data {
                                    if h.is_healthy {
                                        // 计算备用供应商的综合评分
                                        let score = calculate_supplier_score(&h, &config);
                                        candidates.push((supplier.clone(), h.clone(), score));

                                        // 更新最佳候选
                                        if best_candidate.is_none() || score > best_candidate.as_ref().unwrap().2 {
                                            best_candidate = Some((supplier, h, score));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // 如果找到最佳候选，执行切换
                if let Some((best_supplier, best_health, score)) = best_candidate {
                    println!("选择备用供应商: {} (评分: {:.2})", best_supplier.name, score);

                    let switch_request = SupplierSwitchRequest {
                        from_supplier_id: current_supplier.id.unwrap(),
                        to_supplier_id: best_supplier.id.unwrap(),
                        switch_reason: crate::models::supplier::SwitchReason::AutoFailover,
                        create_backup: true,
                        rollback_on_failure: config.auto_rollback,
                    };

                    return switch_supplier(state.clone(), switch_request).await;
                } else {
                    return Ok(ApiResponse::error("没有健康的备用供应商可用".to_string()));
                }
            } else {
                return Ok(ApiResponse::error("当前供应商健康，无需故障转移".to_string()));
            }
        } else {
            return Ok(ApiResponse::error("无法获取供应商健康状态".to_string()));
        }
    } else {
        return Ok(ApiResponse::error("没有激活的供应商".to_string()));
    }
}

// 评估故障转移条件
fn evaluate_failover_conditions(health: &SupplierHealth, config: &FailoverConfig) -> bool {
    // 条件1: 连续失败次数超过阈值
    if health.consecutive_failures >= config.max_consecutive_failures as i64 {
        return true;
    }

    // 条件2: 响应时间超过阈值且成功率低
    if health.response_time > config.max_response_time_ms as i64 && health.uptime_percentage < config.min_success_rate {
        return true;
    }

    // 条件3: 成功率过低
    if health.uptime_percentage < config.min_success_rate - 10.0 {
        return true;
    }

    // 条件4: 完全不健康
    if !health.is_healthy {
        return true;
    }

    false
}

// 计算供应商综合评分
fn calculate_supplier_score(health: &SupplierHealth, config: &FailoverConfig) -> f64 {
    let mut score = 100.0;

    // 成功率权重 (40%)
    let success_rate_score = (health.uptime_percentage / 100.0) * 40.0;

    // 响应时间权重 (30%) - 响应时间越短得分越高
    let response_time_score = if health.response_time > 0 {
        let optimal_time = config.max_response_time_ms as f64 * 0.5;
        let actual_score = if health.response_time as f64 <= optimal_time {
            30.0
        } else {
            let penalty = ((health.response_time as f64 - optimal_time) / optimal_time).min(2.0) * 15.0;
            (30.0 - penalty).max(0.0)
        };
        actual_score
    } else {
        15.0 // 默认中等分数
    };

    // 连续失败惩罚 (20%) - 失败次数越少得分越高
    let failure_penalty = (health.consecutive_failures as f64 / config.max_consecutive_failures as f64) * 20.0;
    let failure_score = 20.0 - failure_penalty.min(20.0);

    // 稳定性权重 (10%) - 基于总请求数和失败数的比例
    let stability_score = if health.total_requests > 0 {
        let failure_ratio = health.failed_requests as f64 / health.total_requests as f64;
        (1.0 - failure_ratio) * 10.0
    } else {
        5.0 // 默认中等分数
    };

    score = success_rate_score + response_time_score + failure_score + stability_score;
    score.round()
}

#[tauri::command]
pub async fn get_failover_config(
    state: State<'_, AppState>,
    supplier_type: String,
) -> Result<ApiResponse<FailoverConfig>, String> {
    // TODO: 从数据库或配置文件中读取故障转移配置
    // 这里返回默认配置
    let config = FailoverConfig {
        enabled: true,
        trigger_conditions: vec![],
        auto_rollback: true,
        rollback_delay_seconds: 300,
        max_consecutive_failures: 3,
        max_response_time_ms: 5000,
        min_success_rate: 95.0,
    };

    Ok(ApiResponse::success(config))
}

#[tauri::command]
pub async fn update_failover_config(
    state: State<'_, AppState>,
    supplier_type: String,
    config: FailoverConfig,
) -> Result<ApiResponse<bool>, String> {
    // TODO: 保存故障转移配置到数据库或配置文件
    // 这里暂时返回成功
    Ok(ApiResponse::success(true))
}

#[tauri::command]
pub async fn get_supplier_switch_progress(
    state: State<'_, AppState>,
    switch_id: String,
) -> Result<ApiResponse<Option<SupplierSwitchProgress>>, String> {
    // TODO: 从状态缓存中获取切换进度
    // 这里暂时返回None
    Ok(ApiResponse::success(None))
}