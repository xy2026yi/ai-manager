// API中间件
//
// 提供CORS、日志记录、认证等中间件功能

use axum::{
    extract::Request,
    http::{StatusCode, header},
    middleware::Next,
    response::{Response, IntoResponse},
};
use std::time::Instant;
use tracing::{info, warn, error};
use uuid::Uuid;
use crate::api::error::ApiError;

/// 请求上下文信息
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub request_id: String,
    pub start_time: Instant,
    pub method: String,
    pub uri: String,
    pub user_agent: String,
    pub client_ip: Option<String>,
}

/// 请求追踪中间件
pub async fn request_tracking_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 生成请求ID
    let request_id = Uuid::new_v4().to_string();
    let start_time = Instant::now();

    // 提取请求信息
    let method = request.method().clone().to_string();
    let uri = request.uri().clone().to_string();
    let user_agent = request
        .headers()
        .get(header::USER_AGENT)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("Unknown")
        .to_string();

    // 提取客户端IP
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .or_else(|| request.headers().get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    // 创建请求上下文
    let context = RequestContext {
        request_id: request_id.clone(),
        start_time,
        method: method.clone(),
        uri: uri.clone(),
        user_agent: user_agent.clone(),
        client_ip,
    };

    // 记录请求开始
    info!(
        request_id = %request_id,
        method = %method,
        uri = %uri,
        user_agent = %user_agent,
        client_ip = ?context.client_ip,
        "开始处理API请求"
    );

    // 将上下文添加到请求扩展中
    let mut request = request;
    request.extensions_mut().insert(context);

    // 继续处理请求
    let response = next.run(request).await;

    // 计算处理时间
    let duration = start_time.elapsed();
    let status = response.status();

    // 记录响应信息
    if status.is_success() {
        info!(
            request_id = %request_id,
            status = %status,
            duration_ms = %duration.as_millis(),
            "API请求处理成功"
        );
    } else {
        warn!(
            request_id = %request_id,
            status = %status,
            duration_ms = %duration.as_millis(),
            "API请求处理失败"
        );
    }

    Ok(response)
}

/// 添加请求ID到响应头的中间件
pub async fn add_request_id_header(
    request: Request,
    next: Next,
) -> Response {
    let request_id = request.extensions()
        .get::<RequestContext>()
        .map(|ctx| ctx.request_id.clone())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let mut response = next.run(request).await;

    // 添加请求ID到响应头
    let headers = response.headers_mut();
    headers.insert("x-request-id", request_id.parse().unwrap());

    response
}

/// 认证中间件
pub async fn auth_middleware(
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // 获取认证头
    let auth_header = request.headers().get(header::AUTHORIZATION);

    // 获取请求上下文
    let request_context = request.extensions().get::<RequestContext>().cloned();

    let request_id = request_context
        .as_ref()
        .map(|ctx| ctx.request_id.clone())
        .unwrap_or_else(|| "unknown".to_string());

    // 目前暂时跳过认证，直接处理请求
    // 在生产环境中，这里应该验证JWT token或API Key

    if let Some(auth_header) = auth_header {
        let auth_str = auth_header.to_str().unwrap_or("invalid");
        info!(
            request_id = %request_id,
            auth_header = ?auth_str,
            "收到认证头，暂时跳过认证验证"
        );

        // TODO: 实现具体的认证逻辑
        // 1. 验证Bearer token格式
        // 2. 解析JWT token
        // 3. 验证token有效性
        // 4. 提取用户信息
    } else {
        info!(
            request_id = %request_id,
            "未提供认证头，允许匿名访问"
        );
    }

    // 继续处理请求
    Ok(next.run(request).await)
}

/// 简单的API Key认证中间件（可选实现）
pub async fn api_key_middleware(
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    const API_KEY: &str = "ai-manager-api-key-2024";

    let api_key_header = request.headers().get("x-api-key");
    let request_context = request.extensions().get::<RequestContext>().cloned();
    let request_id = request_context
        .as_ref()
        .map(|ctx| ctx.request_id.clone())
        .unwrap_or_else(|| "unknown".to_string());

    match api_key_header {
        Some(key) if key.to_str().unwrap_or("") == API_KEY => {
            info!(request_id = %request_id, "API Key验证成功");
            Ok(next.run(request).await)
        }
        Some(key) => {
            warn!(
                request_id = %request_id,
                api_key = ?key,
                "API Key验证失败"
            );
            Err(ApiError::Unauthorized("API Key无效".to_string()))
        }
        None => {
            warn!(request_id = %request_id, "缺少API Key");
            Err(ApiError::Unauthorized("缺少API Key".to_string()))
        }
    }
}

/// 全局错误处理中间件
pub async fn global_error_handler(
    request: Request,
    next: Next,
) -> Response {
    let request_context = request.extensions().get::<RequestContext>().cloned();
    let request_id = request_context
        .as_ref()
        .map(|ctx| ctx.request_id.clone())
        .unwrap_or_else(|| "unknown".to_string());

    // 直接处理请求，错误会在Axum的框架层面被处理
    // 这个中间件主要用于日志记录
    let response = next.run(request).await;

    // 记录响应状态
    let status = response.status();
    if !status.is_success() {
        warn!(
            request_id = %request_id,
            status = %status,
            "请求返回错误状态码"
        );
    }

    response
}