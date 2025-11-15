// 数据迁移测试配置
//
// 提供测试数据库连接和配置参数

use std::path::PathBuf;
use sqlx::SqlitePool;

/// 测试配置
pub struct TestConfig {
    /// 原始数据库路径（Python项目）
    pub original_db_path: PathBuf,
    /// 迁移后数据库路径（Rust项目）
    pub migrated_db_path: PathBuf,
    /// 测试用临时数据库路径
    pub temp_db_path: PathBuf,
    /// 测试数据目录
    pub test_data_dir: PathBuf,
    /// Python项目路径
    pub python_project_path: PathBuf,
}

impl TestConfig {
    /// 创建测试配置
    pub fn new() -> Self {
        let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .parent()
            .unwrap_or_else(|| PathBuf::from("."));
        
        Self {
            original_db_path: project_root.join("../ai-manager/ai_manager.db"),
            migrated_db_path: project_root.join("src-tauri/ai_manager.db"),
            temp_db_path: project_root.join("src-tauri/test_migration.db"),
            test_data_dir: project_root.join("tests/data"),
            python_project_path: project_root.join("../ai-manager"),
        }
    }

    /// 获取原始数据库连接字符串
    pub fn original_db_url(&self) -> String {
        format!("sqlite:{}", self.original_db_path.display())
    }

    /// 获取迁移后数据库连接字符串
    pub fn migrated_db_url(&self) -> String {
        format!("sqlite:{}", self.migrated_db_path.display())
    }

    /// 获取测试数据库连接字符串
    pub fn temp_db_url(&self) -> String {
        format!("sqlite:{}", self.temp_db_path.display())
    }

    /// 创建测试数据库连接池
    pub async fn create_original_db_pool(&self) -> Result<SqlitePool, sqlx::Error> {
        SqlitePool::connect(&self.original_db_url()).await
    }

    /// 创建迁移后数据库连接池
    pub async fn create_migrated_db_pool(&self) -> Result<SqlitePool, sqlx::Error> {
        SqlitePool::connect(&self.migrated_db_url()).await
    }

    /// 创建测试数据库连接池
    pub async fn create_temp_db_pool(&self) -> Result<SqlitePool, sqlx::Error> {
        SqlitePool::connect(&self.temp_db_url()).await
    }

    /// 检查原始数据库是否存在
    pub fn original_db_exists(&self) -> bool {
        self.original_db_path.exists()
    }

    /// 检查迁移后数据库是否存在
    pub fn migrated_db_exists(&self) -> bool {
        self.migrated_db_path.exists()
    }

    /// 检查测试数据目录是否存在
    pub fn test_data_exists(&self) -> bool {
        self.test_data_dir.exists()
    }

    /// 创建测试数据目录
    pub fn ensure_test_data_dir(&self) -> Result<(), std::io::Error> {
        std::fs::create_dir_all(&self.test_data_dir)
    }

    /// 删除测试数据库
    pub fn cleanup_temp_db(&self) -> Result<(), std::io::Error> {
        if self.temp_db_path.exists() {
            std::fs::remove_file(&self.temp_db_path)?;
        }
        Ok(())
    }
}

impl Default for TestConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// 测试数据库准备工具
pub struct DatabasePreparer {
    config: TestConfig,
}

impl DatabasePreparer {
    pub fn new(config: TestConfig) -> Self {
        Self { config }
    }

    /// 准备测试环境
    pub async fn prepare_test_environment(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔧 准备测试环境...");

        // 确保测试数据目录存在
        self.config.ensure_test_data_dir()?;
        
        // 清理临时数据库
        let _ = self.config.cleanup_temp_db();

        // 检查原始数据库
        if !self.config.original_db_exists() {
            return Err(format!("原始数据库不存在: {:?}", self.config.original_db_path).into());
        }

        println!("✅ 测试环境准备完成");
        Ok(())
    }

    /// 复制原始数据库到测试位置
    pub async fn copy_original_to_temp(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.config.original_db_exists() {
            return Err("原始数据库不存在".into());
        }

        // 删除现有的临时数据库
        let _ = self.config.cleanup_temp_db();

        // 复制数据库文件
        std::fs::copy(&self.config.original_db_path, &self.config.temp_db_path)?;

        println!("✅ 已复制原始数据库到测试位置");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = TestConfig::new();
        
        // 验证路径格式正确
        assert!(config.original_db_url().starts_with("sqlite:"));
        assert!(config.migrated_db_url().starts_with("sqlite:"));
        assert!(config.temp_db_url().starts_with("sqlite:"));
    }

    #[tokio::test]
    async fn test_database_pool_creation() {
        let config = TestConfig::new();
        
        // 测试临时数据库连接池创建
        let _temp_pool = config.create_temp_db_pool().await;
        
        // 注意：这个测试可能会失败，因为测试数据库文件可能不存在
        // 在实际使用中，我们应该先准备测试环境
    }

    #[test]
    fn test_database_preparer() {
        let config = TestConfig::new();
        let preparer = DatabasePreparer::new(config);
        
        // 基本功能测试
        assert_eq!(preparer.config.original_db_path, TestConfig::new().original_db_path);
    }
}