use sqlx::{Row, Sqlite, Pool};
use sqlx::migrate::MigrateDatabase;
use std::time::Duration;
use thiserror::Error;
use tracing::{info, warn, error, debug};

/// 数据库相关错误类型
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("数据库连接失败: {0}")]
    Connection(#[from] sqlx::Error),
    #[error("数据库迁移失败: {0}")]
    Migration(String),
    #[error("数据库查询失败: {0}")]
    Query(String),
    #[error("数据库配置错误: {0}")]
    Config(String),
}

/// 数据库配置
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite:data/ai_manager.db".to_string(),
            max_connections: 10,
            min_connections: 1,
            connect_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
        }
    }
}

/// 数据库连接池管理器
pub struct DatabaseManager {
    pool: Pool<Sqlite>,
    config: DatabaseConfig,
}

impl DatabaseManager {
    /// 创建新的数据库管理器
    pub async fn new(config: DatabaseConfig) -> Result<Self, DatabaseError> {
        info!("初始化数据库连接池，URL: {}", config.url);

        // 检查并创建数据库
        if !Sqlite::database_exists(&config.url).await.map_err(|e| {
            DatabaseError::Config(format!("检查数据库存在性失败: {}", e))
        })? {
            warn!("数据库文件不存在，将创建新数据库");
            Sqlite::create_database(&config.url).await.map_err(|e| {
                DatabaseError::Config(format!("创建数据库失败: {}", e))
            })?;
            info!("✅ 数据库创建成功");
        }

        // 配置连接池选项
        let pool_options = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .idle_timeout(config.idle_timeout)
            .max_lifetime(config.max_lifetime)
            .acquire_timeout(Duration::from_secs(30));

        // 创建连接池
        let pool = pool_options.connect(&config.url).await
            .map_err(|e| DatabaseError::Connection(e))?;

        info!("✅ 数据库连接池创建成功");

        let manager = Self {
            pool,
            config,
        };

        // 运行数据库迁移
        manager.run_migrations().await?;

        Ok(manager)
    }

    /// 使用默认配置创建数据库管理器
    pub async fn new_default() -> Result<Self, DatabaseError> {
        Self::new(DatabaseConfig::default()).await
    }

    /// 运行数据库迁移
    async fn run_migrations(&self) -> Result<(), DatabaseError> {
        info!("开始运行数据库迁移");

        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

        info!("✅ 数据库迁移完成");
        Ok(())
    }

    /// 获取连接池引用
    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    /// 测试数据库连接
    pub async fn test_connection(&self) -> Result<(), DatabaseError> {
        debug!("测试数据库连接");

        let result = sqlx::query("SELECT 1 as test")
            .fetch_one(&self.pool)
            .await;

        match result {
            Ok(row) => {
                let test_val: i64 = row.get("test");
                if test_val == 1 {
                    info!("✅ 数据库连接测试成功");
                    Ok(())
                } else {
                    Err(DatabaseError::Query("测试查询返回意外结果".to_string()))
                }
            }
            Err(e) => {
                error!("❌ 数据库连接测试失败: {}", e);
                Err(DatabaseError::Connection(e))
            }
        }
    }

    /// 获取连接池状态信息
    pub async fn pool_status(&self) -> PoolStatus {
        PoolStatus {
            size: self.pool.size(),
            idle: self.pool.num_idle() as u32,
        }
    }

    /// 健康检查
    pub async fn health_check(&self) -> Result<(), sqlx::Error> {
        self.pool.acquire().await?;
        Ok(())
    }

    /// 关闭连接池
    pub async fn close(self) {
        info!("关闭数据库连接池");
        self.pool.close().await;
        info!("✅ 数据库连接池已关闭");
    }
}

/// 连接池状态信息
#[derive(Debug)]
pub struct PoolStatus {
    pub size: u32,
    pub idle: u32,
}

impl std::fmt::Display for PoolStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "连接池状态: 总连接数={}, 空闲连接数={}", self.size, self.idle)
    }
}

/// 数据库查询构建器
pub struct QueryBuilder<'a> {
    pool: &'a Pool<Sqlite>,
}

impl<'a> QueryBuilder<'a> {
    pub fn new(pool: &'a Pool<Sqlite>) -> Self {
        Self { pool }
    }

    /// 执行原始SQL查询（简单版本，只支持字符串参数）
    pub async fn execute_raw(&self, query: &str, params: &[&str]) -> Result<sqlx::sqlite::SqliteQueryResult, DatabaseError> {
        let mut query_builder = sqlx::query(query);

        for param in params {
            query_builder = query_builder.bind(param);
        }

        query_builder
            .execute(self.pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))
    }

    /// 检查表是否存在
    pub async fn table_exists(&self, table_name: &str) -> Result<bool, DatabaseError> {
        let result = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name=?")
            .bind(table_name)
            .fetch_optional(self.pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(result.is_some())
    }

    /// 获取表的记录数
    pub async fn count_records(&self, table_name: &str) -> Result<i64, DatabaseError> {
        let query = format!("SELECT COUNT(*) as count FROM {}", table_name);
        let result = sqlx::query(&query)
            .fetch_one(self.pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        let count: i64 = result.get("count");
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    async fn create_test_database() -> DatabaseManager {
        let temp_file = NamedTempFile::new().unwrap();
        let db_url = temp_file.path().to_str().unwrap().to_string();

        // 保持文件不被删除，通过复制到新路径
        let persistent_db = format!("{}_test.db", db_url);
        std::fs::copy(&db_url, &persistent_db).unwrap();

        let config = DatabaseConfig {
            url: persistent_db,
            max_connections: 5,
            min_connections: 1,
            connect_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(60),
            max_lifetime: Duration::from_secs(300),
        };

        DatabaseManager::new(config).await.unwrap()
    }

    #[tokio::test]
    async fn test_database_creation() {
        let db_manager = create_test_database().await;

        // 测试连接
        match db_manager.test_connection().await {
            Ok(_) => println!("✅ 数据库连接测试成功"),
            Err(e) => {
                println!("❌ 数据库连接测试失败: {:?}", e);
                panic!("数据库连接测试失败");
            }
        }

        // 测试表存在检查
        let query_builder = QueryBuilder::new(db_manager.pool());
        assert!(query_builder.table_exists("claude_providers").await.unwrap());
        assert!(query_builder.table_exists("codex_providers").await.unwrap());
        assert!(query_builder.table_exists("agent_guides").await.unwrap());
        assert!(query_builder.table_exists("mcp_servers").await.unwrap());
        assert!(query_builder.table_exists("common_configs").await.unwrap());

        // 测试记录计数
        let count = query_builder.count_records("claude_providers").await.unwrap();
        assert_eq!(count, 0); // 应该是空表
    }

    #[tokio::test]
    async fn test_pool_status() {
        let db_manager = create_test_database().await;
        let status = db_manager.pool_status().await;
        println!("📊 {}", status);

        assert!(status.size <= 5); // 不应超过最大连接数
    }

    #[tokio::test]
    async fn test_query_builder() {
        let db_manager = create_test_database().await;
        let query_builder = QueryBuilder::new(db_manager.pool());

        // 测试插入和查询
        let result = query_builder.execute_raw(
            "INSERT INTO common_configs (key, value, category) VALUES (?, ?, ?)",
            &["test_key", "test_value", "test"]
        ).await;

        assert!(result.is_ok());

        let count = query_builder.count_records("common_configs").await.unwrap();
        assert_eq!(count, 1);
    }
}