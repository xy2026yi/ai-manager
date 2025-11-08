use crate::models::mcp_template::{McpTemplate, CreateMcpTemplateRequest, UpdateMcpTemplateRequest, McpTemplateValidationResult, McpTemplateCategory};
use crate::models::ApiResponse;
use tauri::State;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;

// 使用与供应商模块相同的应用状态
use crate::commands::supplier::AppState;

// MCP模板相关命令

#[tauri::command]
pub async fn list_mcp_templates(
    state: State<'_, AppState>,
    ai_type: Option<String>,
    platform_type: Option<String>,
) -> Result<ApiResponse<Vec<McpTemplate>>, String> {
    let pool = {
        let guard = state.db_pool.lock().await;
        guard.clone()
    };

    let templates = match (ai_type, platform_type) {
        (Some(ai_type), Some(platform_type)) => {
            McpTemplate::get_by_filters(&pool, &ai_type, &platform_type)
                .await
                .map_err(|e| format!("获取MCP模板失败: {}", e))?
        }
        (Some(ai_type), None) => {
            McpTemplate::get_by_ai_type(&pool, &ai_type)
                .await
                .map_err(|e| format!("获取MCP模板失败: {}", e))?
        }
        (None, Some(platform_type)) => {
            McpTemplate::get_by_platform_type(&pool, &platform_type)
                .await
                .map_err(|e| format!("获取MCP模板失败: {}", e))?
        }
        (None, None) => {
            McpTemplate::get_all(&pool)
                .await
                .map_err(|e| format!("获取MCP模板失败: {}", e))?
        }
    };

    Ok(ApiResponse::success(templates))
}

#[tauri::command]
pub async fn create_mcp_template(
    state: State<'_, AppState>,
    request: CreateMcpTemplateRequest,
) -> Result<ApiResponse<McpTemplate>, String> {
    let pool = {
        let guard = state.db_pool.lock().await;
        guard.clone()
    };

    // 验证请求
    let template = McpTemplate {
        id: None,
        name: request.name.clone(),
        version: request.version.clone().unwrap_or_else(|| "1.0.0".to_string()),
        ai_type: request.ai_type.clone(),
        platform_type: request.platform_type.clone(),
        config_content: request.config_content.clone(),
        description: request.description.clone(),
        is_builtin: Some(0),
        category: request.category.clone(),
        tags: request.tags.as_ref().map(|tags| serde_json::to_string(tags).unwrap_or_default()),
        usage_count: Some(0),
        created_at: None,
        updated_at: None,
    };

    let validation_result = template.validate_config();
    if !validation_result.valid {
        return Ok(ApiResponse::error(format!("模板验证失败: {}", validation_result.errors.join("; "))));
    }

    let created_template = McpTemplate::create(&pool, request)
        .await
        .map_err(|e| format!("创建MCP模板失败: {}", e))?;

    Ok(ApiResponse::success(created_template))
}

#[tauri::command]
pub async fn update_mcp_template(
    state: State<'_, AppState>,
    request: UpdateMcpTemplateRequest,
) -> Result<ApiResponse<Option<McpTemplate>>, String> {
    let pool = {
        let guard = state.db_pool.lock().await;
        guard.clone()
    };

    // 检查模板是否存在
    let existing_template = McpTemplate::get_by_id(&pool, request.id)
        .await
        .map_err(|e| format!("查询MCP模板失败: {}", e))?;

    let existing_template = match existing_template {
        Some(template) => template,
        None => return Ok(ApiResponse::error("MCP模板不存在".to_string())),
    };

    // 如果是内置模板，不允许修改
    if existing_template.is_builtin() {
        return Ok(ApiResponse::error("内置模板不允许修改".to_string()));
    }

    // 构建更新后的模板进行验证
    let updated_template_for_validation = McpTemplate {
        id: Some(request.id),
        name: request.name.clone().unwrap_or_else(|| existing_template.name.clone()),
        version: request.version.clone().unwrap_or_else(|| existing_template.version.clone()),
        ai_type: existing_template.ai_type.clone(),
        platform_type: existing_template.platform_type.clone(),
        config_content: request.config_content.clone().unwrap_or_else(|| existing_template.config_content.clone()),
        description: request.description.clone().or_else(|| existing_template.description.clone()),
        is_builtin: existing_template.is_builtin,
        category: request.category.clone().or_else(|| existing_template.category.clone()),
        tags: request.tags.as_ref().map(|tags| serde_json::to_string(tags).unwrap_or_default())
            .or_else(|| existing_template.tags.clone()),
        usage_count: existing_template.usage_count,
        created_at: existing_template.created_at,
        updated_at: existing_template.updated_at,
    };

    let validation_result = updated_template_for_validation.validate_config();
    if !validation_result.valid {
        return Ok(ApiResponse::error(format!("模板验证失败: {}", validation_result.errors.join("; "))));
    }

    let updated_template = McpTemplate::update(&pool, request)
        .await
        .map_err(|e| format!("更新MCP模板失败: {}", e))?;

    Ok(ApiResponse::success(updated_template))
}

#[tauri::command]
pub async fn delete_mcp_template(
    state: State<'_, AppState>,
    id: i64,
) -> Result<ApiResponse<bool>, String> {
    let pool = {
        let guard = state.db_pool.lock().await;
        guard.clone()
    };

    // 检查模板是否存在
    let existing_template = McpTemplate::get_by_id(&pool, id)
        .await
        .map_err(|e| format!("查询MCP模板失败: {}", e))?;

    if let Some(template) = existing_template {
        // 如果是内置模板，不允许删除
        if template.is_builtin() {
            return Ok(ApiResponse::error("内置模板不允许删除".to_string()));
        }

        let deleted = McpTemplate::delete(&pool, id)
            .await
            .map_err(|e| format!("删除MCP模板失败: {}", e))?;

        if deleted {
            Ok(ApiResponse::success(true))
        } else {
            Ok(ApiResponse::error("删除失败".to_string()))
        }
    } else {
        Ok(ApiResponse::error("MCP模板不存在".to_string()))
    }
}

#[tauri::command]
pub async fn get_mcp_template_by_id(
    state: State<'_, AppState>,
    id: i64,
) -> Result<ApiResponse<Option<McpTemplate>>, String> {
    let pool = {
        let guard = state.db_pool.lock().await;
        guard.clone()
    };

    let template = McpTemplate::get_by_id(&pool, id)
        .await
        .map_err(|e| format!("获取MCP模板失败: {}", e))?;

    Ok(ApiResponse::success(template))
}

#[tauri::command]
pub async fn validate_mcp_template(
    state: State<'_, AppState>,
    request: CreateMcpTemplateRequest,
) -> Result<ApiResponse<McpTemplateValidationResult>, String> {
    let template = McpTemplate {
        id: None,
        name: request.name.clone(),
        version: request.version.clone().unwrap_or_else(|| "1.0.0".to_string()),
        ai_type: request.ai_type.clone(),
        platform_type: request.platform_type.clone(),
        config_content: request.config_content.clone(),
        description: request.description.clone(),
        is_builtin: Some(0),
        category: request.category.clone(),
        tags: request.tags.as_ref().map(|tags| serde_json::to_string(tags).unwrap_or_default()),
        usage_count: Some(0),
        created_at: None,
        updated_at: None,
    };

    let validation_result = template.validate_config();
    Ok(ApiResponse::success(validation_result))
}

#[tauri::command]
pub async fn get_mcp_template_categories(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<McpTemplateCategory>>, String> {
    let pool = {
        let guard = state.db_pool.lock().await;
        guard.clone()
    };

    let categories = McpTemplate::get_categories(&pool)
        .await
        .map_err(|e| format!("获取MCP模板分类失败: {}", e))?;

    Ok(ApiResponse::success(categories))
}

#[tauri::command]
pub async fn increment_template_usage(
    state: State<'_, AppState>,
    id: i64,
) -> Result<ApiResponse<bool>, String> {
    let pool = {
        let guard = state.db_pool.lock().await;
        guard.clone()
    };

    let incremented = McpTemplate::increment_usage_count(&pool, id)
        .await
        .map_err(|e| format!("增加使用计数失败: {}", e))?;

    Ok(ApiResponse::success(incremented))
}

#[tauri::command]
pub async fn clone_mcp_template(
    state: State<'_, AppState>,
    id: i64,
    new_name: String,
) -> Result<ApiResponse<Option<McpTemplate>>, String> {
    let pool = {
        let guard = state.db_pool.lock().await;
        guard.clone()
    };

    // 检查原模板是否存在
    let existing_template = McpTemplate::get_by_id(&pool, id)
        .await
        .map_err(|e| format!("查询MCP模板失败: {}", e))?;

    if existing_template.is_none() {
        return Ok(ApiResponse::error("原模板不存在".to_string()));
    }

    // 检查新名称是否已存在
    let existing_with_new_name = sqlx::query("SELECT id FROM mcp_templates WHERE name = ?")
        .bind(&new_name)
        .fetch_optional(&pool)
        .await
        .map_err(|e| format!("检查模板名称失败: {}", e))?;

    if existing_with_new_name.is_some() {
        return Ok(ApiResponse::error("模板名称已存在".to_string()));
    }

    let cloned_template = McpTemplate::clone_template(&pool, id, &new_name)
        .await
        .map_err(|e| format!("克隆模板失败: {}", e))?;

    Ok(ApiResponse::success(cloned_template))
}

#[tauri::command]
pub async fn get_mcp_template_stats(
    state: State<'_, AppState>,
) -> Result<ApiResponse<serde_json::Value>, String> {
    let pool = {
        let guard = state.db_pool.lock().await;
        guard.clone()
    };

    // 获取总模板数
    let total_count = sqlx::query_scalar::<_, Option<i64>>("SELECT COUNT(*) FROM mcp_templates")
        .fetch_one(&pool)
        .await
        .map_err(|e| format!("获取模板总数失败: {}", e))?
        .unwrap_or(0);

    // 获取内置模板数
    let builtin_count = sqlx::query_scalar::<_, Option<i64>>("SELECT COUNT(*) FROM mcp_templates WHERE is_builtin = 1")
        .fetch_one(&pool)
        .await
        .map_err(|e| format!("获取内置模板数失败: {}", e))?
        .unwrap_or(0);

    // 获取自定义模板数
    let custom_count = sqlx::query_scalar::<_, Option<i64>>("SELECT COUNT(*) FROM mcp_templates WHERE is_builtin = 0")
        .fetch_one(&pool)
        .await
        .map_err(|e| format!("获取自定义模板数失败: {}", e))?
        .unwrap_or(0);

    // 获取Claude模板数
    let claude_count = sqlx::query_scalar::<_, Option<i64>>("SELECT COUNT(*) FROM mcp_templates WHERE ai_type = 'claude'")
        .fetch_one(&pool)
        .await
        .map_err(|e| format!("获取Claude模板数失败: {}", e))?
        .unwrap_or(0);

    // 获取Codex模板数
    let codex_count = sqlx::query_scalar::<_, Option<i64>>("SELECT COUNT(*) FROM mcp_templates WHERE ai_type = 'codex'")
        .fetch_one(&pool)
        .await
        .map_err(|e| format!("获取Codex模板数失败: {}", e))?
        .unwrap_or(0);

    // 获取Unix平台模板数
    let unix_count = sqlx::query_scalar::<_, Option<i64>>("SELECT COUNT(*) FROM mcp_templates WHERE platform_type = 'unix'")
        .fetch_one(&pool)
        .await
        .map_err(|e| format!("获取Unix模板数失败: {}", e))?
        .unwrap_or(0);

    // 获取Windows平台模板数
    let windows_count = sqlx::query_scalar::<_, Option<i64>>("SELECT COUNT(*) FROM mcp_templates WHERE platform_type = 'windows'")
        .fetch_one(&pool)
        .await
        .map_err(|e| format!("获取Windows模板数失败: {}", e))?
        .unwrap_or(0);

    let stats = serde_json::json!({
        "total": total_count,
        "builtin": builtin_count,
        "custom": custom_count,
        "claude": claude_count,
        "codex": codex_count,
        "unix": unix_count,
        "windows": windows_count
    });

    Ok(ApiResponse::success(stats))
}

#[tauri::command]
pub async fn import_mcp_templates(
    state: State<'_, AppState>,
    templates: Vec<CreateMcpTemplateRequest>,
) -> Result<ApiResponse<Vec<McpTemplate>>, String> {
    let pool = {
        let guard = state.db_pool.lock().await;
        guard.clone()
    };
    let mut created_templates = Vec::new();
    let mut errors = Vec::new();

    for request in templates.into_iter() {
        // 验证模板
        let template = McpTemplate {
            id: None,
            name: request.name.clone(),
            version: request.version.clone().unwrap_or_else(|| "1.0.0".to_string()),
            ai_type: request.ai_type.clone(),
            platform_type: request.platform_type.clone(),
            config_content: request.config_content.clone(),
            description: request.description.clone(),
            is_builtin: Some(0),
            category: request.category.clone(),
            tags: request.tags.as_ref().map(|tags| serde_json::to_string(tags).unwrap_or_default()),
            usage_count: Some(0),
            created_at: None,
            updated_at: None,
        };

        let validation_result = template.validate_config();
        if !validation_result.valid {
            errors.push(format!("模板 '{}' 验证失败: {}", template.name, validation_result.errors.join("; ")));
            continue;
        }

        match McpTemplate::create(&pool, request).await {
            Ok(created) => created_templates.push(created),
            Err(e) => errors.push(format!("导入模板 '{}' 失败: {}", template.name, e)),
        }
    }

    if !errors.is_empty() {
        return Ok(ApiResponse::error(format!("导入过程中发生错误: {}", errors.join("; "))));
    }

    Ok(ApiResponse::success(created_templates))
}

#[tauri::command]
pub async fn export_mcp_templates(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<McpTemplate>>, String> {
    let pool = {
        let guard = state.db_pool.lock().await;
        guard.clone()
    };

    let templates = McpTemplate::get_all(&pool)
        .await
        .map_err(|e| format!("导出MCP模板失败: {}", e))?;

    Ok(ApiResponse::success(templates))
}