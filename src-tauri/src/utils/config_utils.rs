//! 配置文件工具
//!
//! 提供配置文件的读取、写入和验证功能

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// 应用程序配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 数据库配置
    pub database: DatabaseConfig,
    /// 服务器配置
    pub server: ServerConfig,
    /// 日志配置
    pub logging: LoggingConfig,
    /// 安全配置
    pub security: SecurityConfig,
    /// 应用程序设置
    pub app: AppSettings,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig::default(),
            server: ServerConfig::default(),
            logging: LoggingConfig::default(),
            security: SecurityConfig::default(),
            app: AppSettings::default(),
        }
    }
}

/// 数据库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// 数据库URL
    pub url: String,
    /// 最大连接数
    pub max_connections: u32,
    /// 最小连接数
    pub min_connections: u32,
    /// 连接超时（秒）
    pub connect_timeout: u64,
    /// 空闲超时（秒）
    pub idle_timeout: u64,
    /// 连接最大生存时间（秒）
    pub max_lifetime: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite:ai_manager.db".to_string(),
            max_connections: 10,
            min_connections: 1,
            connect_timeout: 30,
            idle_timeout: 600,
            max_lifetime: 1800,
        }
    }
}

/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// 服务器地址
    pub host: String,
    /// 服务器端口
    pub port: u16,
    /// 启用CORS
    pub enable_cors: bool,
    /// 请求超时（毫秒）
    pub request_timeout: u64,
    /// 启用请求日志
    pub enable_request_logging: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            enable_cors: true,
            request_timeout: 30000,
            enable_request_logging: true,
        }
    }
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// 日志级别
    pub level: String,
    /// 日志格式
    pub format: String,
    /// 输出到文件
    pub file_output: bool,
    /// 日志文件路径
    pub file_path: Option<String>,
    /// 日志文件最大大小（MB）
    pub max_file_size: u64,
    /// 保留的日志文件数量
    pub max_files: u32,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "json".to_string(),
            file_output: false,
            file_path: None,
            max_file_size: 10,
            max_files: 5,
        }
    }
}

/// 安全配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// 加密密钥
    pub encryption_key: String,
    /// JWT密钥
    pub jwt_secret: String,
    /// 会话超时时间（分钟）
    pub session_timeout: u64,
    /// 最大登录尝试次数
    pub max_login_attempts: u32,
    /// 账户锁定时间（分钟）
    pub account_lockout_duration: u64,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            encryption_key: "".to_string(),
            jwt_secret: "".to_string(),
            session_timeout: 30,
            max_login_attempts: 5,
            account_lockout_duration: 15,
        }
    }
}

/// 应用程序设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// 应用程序名称
    pub name: String,
    /// 版本号
    pub version: String,
    /// 环境模式
    pub environment: String,
    /// 启用调试模式
    pub debug_mode: bool,
    /// 默认语言
    pub default_language: String,
    /// 时区
    pub timezone: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            name: "AI Manager".to_string(),
            version: "0.1.0".to_string(),
            environment: "development".to_string(),
            debug_mode: true,
            default_language: "zh-CN".to_string(),
            timezone: "Asia/Shanghai".to_string(),
        }
    }
}

/// 配置文件错误类型
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("文件不存在: {0}")]
    FileNotFound(String),
    #[error("读取文件失败: {0}")]
    ReadError(String),
    #[error("写入文件失败: {0}")]
    WriteError(String),
    #[error("解析配置文件失败: {0}")]
    ParseError(String),
    #[error("验证配置失败: {0}")]
    ValidationError(String),
    #[error("环境变量错误: {0}")]
    EnvVarError(String),
}

/// 配置管理器
pub struct ConfigManager {
    config: AppConfig,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new() -> Self {
        Self { config: AppConfig::default() }
    }

    /// 从文件加载配置
    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), ConfigError> {
        let content =
            fs::read_to_string(path).map_err(|e| ConfigError::ReadError(e.to_string()))?;

        self.config =
            serde_json::from_str(&content).map_err(|e| ConfigError::ParseError(e.to_string()))?;

        Ok(())
    }

    /// 从环境变量加载配置
    pub fn load_from_env(&mut self) -> Result<(), ConfigError> {
        // 数据库配置
        if let Ok(url) = std::env::var("DATABASE_URL") {
            self.config.database.url = url;
        }

        if let Ok(max_conn) = std::env::var("MAX_CONNECTIONS") {
            self.config.database.max_connections = max_conn
                .parse()
                .map_err(|_| ConfigError::EnvVarError("MAX_CONNECTIONS必须是数字".to_string()))?;
        }

        // 服务器配置
        if let Ok(host) = std::env::var("SERVER_HOST") {
            self.config.server.host = host;
        }

        if let Ok(port) = std::env::var("SERVER_PORT") {
            self.config.server.port = port
                .parse()
                .map_err(|_| ConfigError::EnvVarError("SERVER_PORT必须是数字".to_string()))?;
        }

        // 安全配置
        if let Ok(key) = std::env::var("ENCRYPTION_KEY") {
            self.config.security.encryption_key = key;
        }

        if let Ok(secret) = std::env::var("JWT_SECRET") {
            self.config.security.jwt_secret = secret;
        }

        Ok(())
    }

    /// 保存配置到文件
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let content = serde_json::to_string_pretty(&self.config)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        // 确保目录存在
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent).map_err(|e| ConfigError::WriteError(e.to_string()))?;
        }

        fs::write(path, content).map_err(|e| ConfigError::WriteError(e.to_string()))?;

        Ok(())
    }

    /// 获取配置的引用
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// 获取可修改的配置引用
    pub fn config_mut(&mut self) -> &mut AppConfig {
        &mut self.config
    }

    /// 验证配置
    pub fn validate(&self) -> Result<(), ConfigError> {
        // 验证必需的配置
        if self.config.security.encryption_key.is_empty() {
            return Err(ConfigError::ValidationError("加密密钥不能为空".to_string()));
        }

        if self.config.security.jwt_secret.is_empty() {
            return Err(ConfigError::ValidationError("JWT密钥不能为空".to_string()));
        }

        // 验证端口号
        if self.config.server.port == 0 {
            return Err(ConfigError::ValidationError(
                "服务器端口不能为0".to_string(),
            ));
        }

        Ok(())
    }
}

/// 获取默认配置文件路径
pub fn get_default_config_path() -> String {
    "config/app.json".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.max_connections, 10);
    }

    #[test]
    fn test_config_save_load() -> Result<(), ConfigError> {
        let mut manager = ConfigManager::new();

        // 修改一些配置
        manager.config_mut().server.port = 9999;
        manager.config_mut().app.name = "Test App".to_string();

        // 创建临时文件
        let mut temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // 保存和加载
        manager.save_to_file(path)?;
        let mut new_manager = ConfigManager::new();
        new_manager.load_from_file(path)?;

        // 验证
        assert_eq!(new_manager.config().server.port, 9999);
        assert_eq!(new_manager.config().app.name, "Test App");

        Ok(())
    }

    #[test]
    fn test_config_validation() {
        let mut manager = ConfigManager::new();

        // 无效配置（空密钥）
        assert!(manager.validate().is_err());

        // 有效配置
        manager.config_mut().security.encryption_key = "test_key_32_bytes_long!!".to_string();
        manager.config_mut().security.jwt_secret = "test_jwt_secret_32_bytes".to_string();
        assert!(manager.validate().is_ok());
    }
}
