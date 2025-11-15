//! 日志管理器
//!
//! 提供统一的日志配置和管理功能

use tracing::Level;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer, Registry,
};
use std::fs;
use std::path::PathBuf;

/// 日志管理器
pub struct LoggingManager {
    app_name: String,
    log_dir: PathBuf,
}

impl LoggingManager {
    /// 创建新的日志管理器
    pub fn new(app_name: impl Into<String>) -> Self {
        let app_name = app_name.into();
        let log_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("logs");

        Self {
            app_name,
            log_dir,
        }
    }

    /// 初始化生产环境日志
    pub fn init_production(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 确保日志目录存在
        fs::create_dir_all(&self.log_dir)?;

        let log_file = self.log_dir.join(format!("{}.log", self.app_name));
        let error_log_file = self.log_dir.join(format!("{}-error.log", self.app_name));

        // 创建文件输出
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)?;

        let error_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&error_log_file)?;

        // 创建环境过滤器
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info"));

        // 构建订阅者
        let subscriber = Registry::default()
            .with(env_filter)
            .with(
                fmt::layer()
                    .with_writer(file)
                    .with_span_events(FmtSpan::CLOSE)
                    .with_ansi(false)
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_thread_names(true)
                    .compact(),
            )
            .with(
                fmt::layer()
                    .with_writer(error_file)
                    .with_span_events(FmtSpan::CLOSE)
                    .with_ansi(false)
                    .with_target(true)
                    .with_filter(tracing_subscriber::filter::filter_fn(|metadata| {
                        metadata.level() >= &Level::WARN
                    })),
            );

        // 设置全局订阅者
        subscriber.init();

        tracing::info!("🚀 {} 生产环境日志系统初始化完成", self.app_name);
        tracing::info!("📁 普通日志: {}", log_file.display());
        tracing::info!("⚠️  错误日志: {}", error_log_file.display());

        Ok(())
    }

    /// 初始化开发环境日志
    pub fn init_development() -> Result<(), Box<dyn std::error::Error>> {
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| {
                EnvFilter::new("migration_ai_manager=debug")
                    .add_directive("sqlx=warn".parse().unwrap())
                    .add_directive("hyper=warn".parse().unwrap())
                    .add_directive("tokio=warn".parse().unwrap())
                    .add_directive("tower=warn".parse().unwrap())
            });

        let subscriber = tracing_subscriber::fmt()
            .with_max_level(Level::DEBUG)
            .with_target(true)
            .with_thread_ids(false)
            .with_thread_names(false)
            .with_file(true)
            .with_line_number(true)
            .with_span_events(FmtSpan::CLOSE)
            .pretty()
            .with_env_filter(env_filter);

        subscriber.init();

        tracing::info!("🔧 开发环境日志系统初始化完成");
        Ok(())
    }

    /// 初始化测试环境日志
    pub fn init_test() -> Result<(), Box<dyn std::error::Error>> {
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("warn"));

        let subscriber = tracing_subscriber::fmt()
            .with_max_level(Level::WARN)
            .with_target(false)
            .compact()
            .with_test_writer()
            .with_env_filter(env_filter);

        subscriber.init();

        Ok(())
    }

    /// 初始化基于环境的日志系统
    pub fn init_from_env() -> Result<(), Box<dyn std::error::Error>> {
        match std::env::var("RUST_LOG_ENV").unwrap_or_else(|_| "development".to_string()).as_str() {
            "production" => {
                let manager = Self::new("migration_ai_manager");
                manager.init_production()
            }
            "test" => Self::init_test(),
            _ => Self::init_development(),
        }
    }

    /// 创建性能日志记录器
    pub fn create_performance_logger() -> impl Layer<Registry> + Send + Sync + 'static {
        fmt::layer()
            .with_target(false)
            .with_ansi(false)
            .compact()
            .with_filter(tracing_subscriber::filter::filter_fn(|metadata| {
                metadata.target().starts_with("migration_ai_manager::performance")
                    || metadata.target().starts_with("migration_ai_manager::database")
            }))
    }

    /// 创建业务逻辑日志记录器
    pub fn create_business_logger() -> impl Layer<Registry> + Send + Sync + 'static {
        fmt::layer()
            .with_target(true)
            .with_file(true)
            .with_line_number(true)
            .compact()
            .with_filter(tracing_subscriber::filter::filter_fn(|metadata| {
                metadata.target().starts_with("migration_ai_manager::services")
                    || metadata.target().starts_with("migration_ai_manager::api")
            }))
    }

    /// 获取应用名称
    pub fn app_name(&self) -> &str {
        &self.app_name
    }

    /// 获取日志目录路径
    pub fn log_dir(&self) -> &PathBuf {
        &self.log_dir
    }

    /// 清理旧的日志文件
    pub fn cleanup_old_logs(&self, days: u64) -> Result<(), Box<dyn std::error::Error>> {
        if !self.log_dir.exists() {
            return Ok(());
        }

        let cutoff = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs()
            - days * 24 * 60 * 60;

        for entry in fs::read_dir(&self.log_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(modified_secs) = modified.duration_since(std::time::UNIX_EPOCH) {
                            if modified_secs.as_secs() < cutoff {
                                if let Err(e) = fs::remove_file(&path) {
                                    tracing::warn!("无法删除旧日志文件 {}: {}", path.display(), e);
                                } else {
                                    tracing::info!("已删除旧日志文件: {}", path.display());
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

/// 日志配置结构体
#[derive(Debug, Clone)]
pub struct LogConfig {
    pub level: Level,
    pub file_logging: bool,
    pub console_logging: bool,
    pub include_spans: bool,
    pub include_target: bool,
    pub pretty_output: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: Level::INFO,
            file_logging: true,
            console_logging: true,
            include_spans: true,
            include_target: true,
            pretty_output: false,
        }
    }
}

impl LoggingManager {
    /// 使用配置初始化日志
    pub fn init_with_config(config: LogConfig) -> Result<(), Box<dyn std::error::Error>> {
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(config.level.to_string()));

        let mut layers = Vec::new();

        // 控制台输出
        if config.console_logging {
            let console_layer = if config.pretty_output {
                fmt::layer()
                    .with_target(config.include_target)
                    .with_span_events(if config.include_spans {
                        FmtSpan::CLOSE
                    } else {
                        FmtSpan::NONE
                    })
                    .pretty()
                    .boxed()
            } else {
                fmt::layer()
                    .with_target(config.include_target)
                    .with_span_events(if config.include_spans {
                        FmtSpan::CLOSE
                    } else {
                        FmtSpan::NONE
                    })
                    .compact()
                    .boxed()
            };

            layers.push(console_layer);
        }

        // 文件输出
        if config.file_logging {
            let log_dir = std::env::current_dir()?.join("logs");
            fs::create_dir_all(&log_dir)?;

            let log_file = log_dir.join("app.log");
            let file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_file)?;

            let file_layer = fmt::layer()
                .with_writer(file)
                .with_target(config.include_target)
                .with_span_events(if config.include_spans {
                    FmtSpan::CLOSE
                } else {
                    FmtSpan::NONE
                })
                .with_ansi(false)
                .compact()
                .boxed();

            layers.push(file_layer);
        }

        // 组合所有层
        let subscriber = Registry::default()
            .with(env_filter)
            .with(layers);

        subscriber.init();

        tracing::info!("🚀 日志系统初始化完成 - 级别: {:?}", config.level);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    /// 初始化日志系统（只执行一次）
    fn init_logging() {
        INIT.call_once(|| {
            let _ = LoggingManager::init_test();
        });
    }

    #[test]
    fn test_logging_manager_creation() {
        let manager = LoggingManager::new("test_app");
        assert_eq!(manager.app_name(), "test_app");
        assert!(manager.log_dir().ends_with("logs"));
    }

    #[test]
    fn test_log_config_default() {
        let config = LogConfig::default();
        assert_eq!(config.level, Level::INFO);
        assert!(config.file_logging);
        assert!(config.console_logging);
        assert!(config.include_spans);
        assert!(config.include_target);
        assert!(!config.pretty_output);
    }

    #[test]
    fn test_development_init() {
        init_logging();
        // 验证日志系统已初始化且可用
        tracing::info!("测试开发环境日志初始化");
    }

    #[test]
    fn test_test_init() {
        init_logging();
        // 验证日志系统已初始化且可用
        tracing::warn!("测试测试环境日志初始化");
    }
}