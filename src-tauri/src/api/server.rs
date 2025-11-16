// API服务器配置和启动逻辑
//
// 提供HTTP服务器的配置、路由设置和启动功能
// 支持环境配置和优雅关闭

use crate::api::error::ApiError;
use crate::api::handlers::{agent_guide, claude, codex, common_config, mcp_server};
use crate::crypto::CryptoService;
use crate::database::DatabaseManager;
use axum::{http::StatusCode, response::IntoResponse, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;

/// 统一的API状态
#[derive(Clone)]
pub struct ApiState {
    pub db_manager: std::sync::Arc<DatabaseManager>,
    pub crypto_service: std::sync::Arc<CryptoService>,
    pub claude_service: crate::services::claude_service::ClaudeProviderService,
    pub codex_service: crate::services::codex_service::CodexProviderService,
}

/// API服务器配置
#[derive(Debug, Clone)]
pub struct ApiServerConfig {
    pub host: String,
    pub port: u16,
    pub enable_cors: bool,
    pub enable_tracing: bool,
}

impl Default for ApiServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            enable_cors: true,
            enable_tracing: true,
        }
    }
}

/// API服务器
pub struct ApiServer {
    config: ApiServerConfig,
    app: Router,
}

impl ApiServer {
    /// 创建新的API服务器实例
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Self::with_config(ApiServerConfig::default()).await
    }

    /// 使用自定义配置创建API服务器
    pub async fn with_config(config: ApiServerConfig) -> Result<Self, Box<dyn std::error::Error>> {
        // 初始化数据库和加密服务
        let db_config = crate::database::DatabaseConfig {
            url: "sqlite:data/ai_manager.db".to_string(),
            max_connections: 10,
            min_connections: 1,
            connect_timeout: std::time::Duration::from_secs(30),
            idle_timeout: std::time::Duration::from_secs(600),
            max_lifetime: std::time::Duration::from_secs(1800),
        };

        let db_manager = Arc::new(DatabaseManager::new(db_config).await?);
        let crypto_service = Arc::new(CryptoService::new(
            "T4jCbDRQ6Z10_dzcJlhvyn2EfK-tTS4-dbpf27Lc1k8=",
        )?);

        // 创建API状态
        let api_state = ApiState {
            db_manager: db_manager.clone(),
            crypto_service: crypto_service.clone(),
            claude_service: crate::services::claude_service::ClaudeProviderService::new(
                db_manager.clone(),
                crypto_service.clone(),
            ),
            codex_service: crate::services::codex_service::CodexProviderService::new(
                db_manager,
                crypto_service,
            ),
        };

        let app = Self::create_app(&config, api_state);
        Ok(Self { config, app })
    }

    /// 创建Axum应用
    fn create_app(config: &ApiServerConfig, api_state: ApiState) -> Router {
        let app = Router::new()
            // 健康检查端点
            .route("/health", axum::routing::get(health_check))
            // API版本信息
            .route("/api/v1/info", axum::routing::get(api_info))
            // Claude供应商管理路由
            .nest("/api/v1/claude-providers", claude::routes())
            // Codex供应商管理路由
            .nest("/api/v1/codex-providers", codex::routes())
            // Agent指导文件管理路由
            .nest("/api/v1/agent-guides", agent_guide::routes())
            // MCP服务器管理路由
            .nest("/api/v1/mcp-servers", mcp_server::routes())
            // 通用配置管理路由
            .nest("/api/v1/common-configs", common_config::routes())
            .with_state(api_state)
            // 404处理
            .fallback(handle_404);

        // 根据配置添加中间件
        if config.enable_cors || config.enable_tracing {
            Self::create_app_with_middleware(app, config)
        } else {
            app
        }
    }

    /// 创建带中间件的应用
    fn create_app_with_middleware(app: Router, config: &ApiServerConfig) -> Router {
        let mut app = app;

        if config.enable_cors && config.enable_tracing {
            let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);

            app = app.layer(cors).layer(TraceLayer::new_for_http());
        } else if config.enable_cors {
            let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);

            app = app.layer(cors);
        } else if config.enable_tracing {
            app = app.layer(TraceLayer::new_for_http());
        }

        app
    }

    /// 启动服务器
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", self.config.host, self.config.port).parse::<SocketAddr>()?;

        info!("🚀 启动AI Manager API服务器");
        info!("📍 监听地址: http://{}", addr);
        info!(
            "🔧 CORS支持: {}",
            if self.config.enable_cors {
                "启用"
            } else {
                "禁用"
            }
        );
        info!(
            "📊 追踪日志: {}",
            if self.config.enable_tracing {
                "启用"
            } else {
                "禁用"
            }
        );

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, self.app).await?;

        Ok(())
    }

    /// 获取服务器配置
    pub fn config(&self) -> &ApiServerConfig {
        &self.config
    }

    /// 获取应用路由（用于测试）
    pub fn app(&self) -> Router {
        self.app.clone()
    }
}

/// 健康检查处理器
async fn health_check() -> impl IntoResponse {
    tracing::debug!("健康检查请求");
    StatusCode::OK
}

/// API信息处理器
async fn api_info() -> impl IntoResponse {
    tracing::debug!("API信息请求");

    let info = serde_json::json!({
        "name": "AI Manager API",
        "version": "1.0.0",
        "description": "AI Manager 数据管理API服务",
        "status": "运行中",
        "timestamp": "2025-11-14T00:00:00Z"
    });

    (StatusCode::OK, axum::Json(info))
}

/// 404处理器
pub async fn handle_404() -> impl IntoResponse {
    ApiError::NotFound { resource: "请求的资源不存在".to_string() }
}

/// 500处理器
pub async fn handle_500(error: axum::BoxError) -> impl IntoResponse {
    tracing::error!("服务器内部错误: {}", error);

    ApiError::Internal { message: "服务器内部错误".to_string() }
}
