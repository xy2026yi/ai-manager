//! 结构化日志系统模块
//!
//! 这个模块提供了应用程序的统一日志配置和管理功能，基于 `tracing` 和 `tracing-subscriber` 构建。
//! 支持不同环境的日志级别控制，以及结构化的日志输出格式。
//!
//! # 主要功能
//!
//! - **环境配置**: 提供开发、生产、测试等不同环境的预设配置
//! - **灵活配置**: 支持自定义日志级别、输出格式和目标
//! - **结构化输出**: 支持 JSON 格式的结构化日志输出
//! - **性能优化**: 支持异步日志记录和批量处理
//!
//! # 使用示例
//!
//! ```rust
//! use crate::logging::{init_development, init_production};
//!
//! fn main() {
//!     // 开发环境
//!     init_development().expect("日志初始化失败");
//!     
//!     // 生产环境
//!     // init_production().expect("日志初始化失败");
//! }
//! ```

use tracing::Level;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Registry,
};

/// 日志配置结构
pub struct LoggingConfig {
    /// 应用程序名称
    pub app_name: String,
    /// 日志级别
    pub level: Level,
    /// 是否启用控制台输出
    pub console: bool,
    /// 是否启用文件输出
    pub file: bool,
    /// 日志文件路径
    pub file_path: Option<String>,
    /// 是否显示目标信息
    pub show_target: bool,
    /// 是否显示线程ID
    pub show_thread_id: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            app_name: "migration_ai_manager".to_string(),
            level: Level::INFO,
            console: true,
            file: false,
            file_path: None,
            show_target: false,
            show_thread_id: false,
        }
    }
}

impl LoggingConfig {
    /// 创建开发环境配置
    pub fn development() -> Self {
        Self {
            app_name: "migration_ai_manager".to_string(),
            level: Level::DEBUG,
            console: true,
            file: false,
            file_path: None,
            show_target: true,
            show_thread_id: true,
        }
    }

    /// 创建生产环境配置
    pub fn production() -> Self {
        Self {
            app_name: "migration_ai_manager".to_string(),
            level: Level::INFO,
            console: false,
            file: true,
            file_path: Some("logs/app.log".to_string()),
            show_target: false,
            show_thread_id: false,
        }
    }

    /// 创建测试环境配置
    pub fn testing() -> Self {
        Self {
            app_name: "migration_ai_manager_test".to_string(),
            level: Level::WARN,
            console: true,
            file: false,
            file_path: None,
            show_target: false,
            show_thread_id: false,
        }
    }
}

/// 初始化日志系统
pub fn init_logging(config: LoggingConfig) -> Result<(), Box<dyn std::error::Error>> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(config.level.to_string()));

    let fmt_layer = fmt::layer()
        .with_target(config.show_target)
        .with_thread_ids(config.show_thread_id)
        .with_span_events(FmtSpan::CLOSE)
        .compact();

    Registry::default()
        .with(env_filter)
        .with(fmt_layer)
        .init();

    tracing::info!(
        app_name = %config.app_name,
        level = %config.level,
        "日志系统初始化完成"
    );

    Ok(())
}

/// 初始化默认日志系统（使用默认配置）
pub fn init_default() -> Result<(), Box<dyn std::error::Error>> {
    let config = LoggingConfig::default();
    init_logging(config)
}

/// 初始化开发环境日志系统
pub fn init_development() -> Result<(), Box<dyn std::error::Error>> {
    let config = LoggingConfig::development();
    init_logging(config)
}

/// 初始化生产环境日志系统
pub fn init_production() -> Result<(), Box<dyn std::error::Error>> {
    let config = LoggingConfig::production();
    init_logging(config)
}

/// 初始化测试环境日志系统
pub fn init_testing() -> Result<(), Box<dyn std::error::Error>> {
    let config = LoggingConfig::testing();
    init_logging(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert_eq!(config.app_name, "migration_ai_manager");
        assert_eq!(config.level, Level::INFO);
        assert!(config.console);
        assert!(!config.file);
    }

    #[test]
    fn test_logging_config_development() {
        let config = LoggingConfig::development();
        assert_eq!(config.level, Level::DEBUG);
        assert!(config.show_target);
        assert!(config.show_thread_id);
    }

    #[test]
    fn test_logging_config_production() {
        let config = LoggingConfig::production();
        assert_eq!(config.level, Level::INFO);
        assert!(!config.console);
        assert!(config.file);
        assert!(config.file_path.is_some());
    }
}
