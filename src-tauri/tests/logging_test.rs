//! 日志系统测试
//!
//! 验证新的日志管理器功能是否正常工作

#[cfg(test)]
mod tests {
    use migration_ai_manager_lib::LoggingManager;
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
    fn test_development_logging() {
        init_logging();

        // 验证tracing宏是否可用
        tracing::info!("测试开发环境日志");
    }

    #[test]
    fn test_test_logging() {
        init_logging();

        tracing::warn!("测试警告日志");
        tracing::error!("测试错误日志");
    }

    #[test]
    fn test_log_config() {
        use migration_ai_manager_lib::LogConfig;

        let config = LogConfig::default();
        assert_eq!(config.level, tracing::Level::INFO);
        assert!(config.file_logging);
        assert!(config.console_logging);
        assert!(config.include_spans);
        assert!(config.include_target);
        assert!(!config.pretty_output);
    }

    #[test]
    fn test_log_cleanup() {
        let manager = LoggingManager::new("cleanup_test");

        // 这个测试主要验证方法不会崩溃
        // 实际的文件删除功能需要存在文件才能测试
        let result = manager.cleanup_old_logs(30);
        assert!(result.is_ok(), "日志清理操作应该成功");
    }
}