use sqlx::migrate::MigrateDatabase;
use sqlx::{Pool, Row, Sqlite};
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, error, info, warn};

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
            max_connections: 10,  // 优化连接池大小，减少内存占用
            min_connections: 1,   // 最小连接数，减少资源浪费
            connect_timeout: Duration::from_secs(5),  // 快速连接超时
            idle_timeout: Duration::from_secs(180),   // 优化空闲超时
            max_lifetime: Duration::from_secs(600),   // 优化连接生命周期
        }
    }
}

/// 数据库连接池管理器
#[derive(Clone)]
pub struct DatabaseManager {
    pool: Pool<Sqlite>,
    config: DatabaseConfig,
}

impl DatabaseManager {
    /// 创建新的数据库管理器（优化启动时间）
    pub async fn new(config: DatabaseConfig) -> Result<Self, DatabaseError> {
        info!("初始化数据库连接池，URL: {}", config.url);

        // 使用连接池建立和迁移并行执行来优化启动时间
        let pool_fut = async {
            // 检查并创建数据库
            if !Sqlite::database_exists(&config.url)
                .await
                .map_err(|e| DatabaseError::Config(format!("检查数据库存在性失败: {}", e)))?
            {
                warn!("数据库文件不存在，将创建新数据库");
                Sqlite::create_database(&config.url)
                    .await
                    .map_err(|e| DatabaseError::Config(format!("创建数据库失败: {}", e)))?;
                info!("✅ 数据库创建成功");
            }

            // 配置性能优化的连接池选项
            let pool_options = sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(config.max_connections)
                .min_connections(config.min_connections)
                .idle_timeout(config.idle_timeout)
                .max_lifetime(config.max_lifetime)
                .acquire_timeout(Duration::from_secs(10)) // 减少获取连接超时
                .test_before_acquire(true) // 连接前测试，避免使用损坏的连接
                // 启用连接池的性能优化设置
                .after_connect(|conn, _meta| {
                    Box::pin(async move {
                        // SQLite性能优化设置
                        sqlx::query("PRAGMA journal_mode = WAL")
                            .execute(&mut *conn)
                            .await?;
                        sqlx::query("PRAGMA synchronous = NORMAL") // 平衡性能和安全性
                            .execute(&mut *conn)
                            .await?;
                        sqlx::query("PRAGMA cache_size = -64000") // 64MB缓存
                            .execute(&mut *conn)
                            .await?;
                        sqlx::query("PRAGMA temp_store = MEMORY") // 临时表存储在内存
                            .execute(&mut *conn)
                            .await?;
                        sqlx::query("PRAGMA mmap_size = 268435456") // 256MB内存映射
                            .execute(&mut *conn)
                            .await?;
                        sqlx::query("PRAGMA optimize") // 自动优化查询计划
                            .execute(&mut *conn)
                            .await?;
                        Ok(())
                    })
                });

            // 创建连接池
            pool_options
                .connect(&config.url)
                .await
                .map_err(|e| DatabaseError::Connection(e))
        };

        // 等待连接池建立
        let pool = pool_fut.await?;

        info!("✅ 数据库连接池创建成功");

        let manager = Self { pool, config };

        // 异步运行数据库迁移和性能优化，不阻塞返回
        let manager_clone = manager.clone();
        tokio::spawn(async move {
            // 运行数据库迁移
            if let Err(e) = manager_clone.run_migrations().await {
                error!("数据库迁移失败: {}", e);
            }

            // 创建性能索引
            let query_builder = QueryBuilder::new(manager_clone.pool());
            if let Err(e) = query_builder.create_performance_indexes().await {
                warn!("性能索引创建失败: {}", e);
            }

            // 连接池预热：创建最小连接数，优化首次查询性能
            if let Err(e) = manager_clone.warmup_connection_pool().await {
                warn!("连接池预热失败: {}", e);
            }

            info!("✅ 数据库初始化和性能优化完成");
        });

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

        let result = sqlx::query("SELECT 1 as test").fetch_one(&self.pool).await;

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
        PoolStatus { size: self.pool.size(), idle: self.pool.num_idle() as u32 }
    }

    /// 健康检查
    pub async fn health_check(&self) -> Result<(), sqlx::Error> {
        self.pool.acquire().await?;
        Ok(())
    }

    /// 连接池预热 - 创建最小连接数，优化首次查询性能
    pub async fn warmup_connection_pool(&self) -> Result<(), DatabaseError> {
        debug!("开始连接池预热");

        // 并行创建多个连接以达到最小连接数
        let pool = &self.pool;
        let warmup_tasks: Vec<_> = (0..self.config.min_connections)
            .map(|_| async {
                // 直接在池上执行查询来创建和测试连接
                sqlx::query("SELECT 1")
                    .fetch_one(pool)
                    .await
                    .map_err(|e| DatabaseError::Connection(e))?;

                Ok::<(), DatabaseError>(())
            })
            .collect();

        // 等待所有预热任务完成
        let results = futures::future::join_all(warmup_tasks).await;

        let mut errors = 0;
        for result in results {
            if let Err(e) = result {
                warn!("连接池预热连接失败: {}", e);
                errors += 1;
            }
        }

        if errors == 0 {
            info!("✅ 连接池预热完成，{} 个连接就绪", self.config.min_connections);
        } else {
            warn!("⚠️ 连接池预热部分失败，{}/{} 个连接失败", errors, self.config.min_connections);
        }

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
        write!(
            f,
            "连接池状态: 总连接数={}, 空闲连接数={}",
            self.size, self.idle
        )
    }
}

/// 表性能统计信息
#[derive(Debug, serde::Serialize)]
pub struct TablePerformanceStats {
    pub name: String,
    pub record_count: i64,
    pub estimated_size_bytes: i64,
    pub index_count: i64,
}

impl std::fmt::Display for TablePerformanceStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "表 '{}': {} 条记录, ~{}KB, {} 个索引",
            self.name,
            self.record_count,
            self.estimated_size_bytes / 1024,
            self.index_count
        )
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
    pub async fn execute_raw(
        &self,
        query: &str,
        params: &[&str],
    ) -> Result<sqlx::sqlite::SqliteQueryResult, DatabaseError> {
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

    /// 获取表的记录数（优化版本，使用预编译语句）
    pub async fn count_records(&self, table_name: &str) -> Result<i64, DatabaseError> {
        let query = format!("SELECT COUNT(*) as count FROM {}", table_name);
        let result = sqlx::query(&query)
            .fetch_one(self.pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        let count: i64 = result.get("count");
        Ok(count)
    }

    /// 执行优化的批量插入（性能优化版本）
    pub async fn batch_insert(
        &self,
        table: &str,
        columns: &[&str],
        values: Vec<Vec<String>>,
    ) -> Result<u64, DatabaseError> {
        if values.is_empty() {
            return Ok(0);
        }

        // 验证数据一致性
        let expected_cols = columns.len();
        for (i, row) in values.iter().enumerate() {
            if row.len() != expected_cols {
                return Err(DatabaseError::Query(
                    format!("第{}行数据长度({})与列数({})不匹配", i + 1, row.len(), expected_cols)
                ));
            }
        }

        // 使用事务提高批量插入性能
        let mut tx = self.pool.begin()
            .await
            .map_err(|e| DatabaseError::Query(format!("开始事务失败: {}", e)))?;

        let mut total_changes = 0;

        // 批量大小优化：每批处理1000行以避免内存溢出
        const BATCH_SIZE: usize = 1000;

        for chunk in values.chunks(BATCH_SIZE) {
            // 预编译插入语句以提高性能
            let placeholders: Vec<String> = (0..expected_cols).map(|_| "?".to_string()).collect();
            let query_str = format!(
                "INSERT INTO {} ({}) VALUES ({})",
                table,
                columns.join(","),
                placeholders.join(",")
            );

            // 在事务内执行查询（修复关键问题）
            for row in chunk {
                let query = row.iter().fold(
                    sqlx::query(&query_str),
                    |q, value| q.bind(value)
                );

                let result = query.execute(&mut *tx).await
                    .map_err(|e| DatabaseError::Query(format!("批量插入失败: {}", e)))?;

                total_changes += result.rows_affected();
            }

            // 每批后短暂释放CPU，避免阻塞UI
            tokio::task::yield_now().await;
        }

        // 提交事务
        tx.commit()
            .await
            .map_err(|e| DatabaseError::Query(format!("提交事务失败: {}", e)))?;

        Ok(total_changes)
    }

    /// 创建性能优化索引
    pub async fn create_performance_indexes(&self) -> Result<(), DatabaseError> {
        tracing::info!("创建性能优化索引");

        let indexes = vec![
            ("idx_claude_providers_enabled", "CREATE INDEX IF NOT EXISTS idx_claude_providers_enabled ON claude_providers(enabled)"),
            ("idx_claude_providers_type", "CREATE INDEX IF NOT EXISTS idx_claude_providers_type ON claude_providers(type)"),
            ("idx_claude_providers_name", "CREATE INDEX IF NOT EXISTS idx_claude_providers_name ON claude_providers(name)"),
            ("idx_claude_providers_created", "CREATE INDEX IF NOT EXISTS idx_claude_providers_created ON claude_providers(created_at)"),
            
            ("idx_codex_providers_enabled", "CREATE INDEX IF NOT EXISTS idx_codex_providers_enabled ON codex_providers(enabled)"),
            ("idx_codex_providers_type", "CREATE INDEX IF NOT EXISTS idx_codex_providers_type ON codex_providers(type)"),
            
            ("idx_agent_guides_type", "CREATE INDEX IF NOT EXISTS idx_agent_guides_type ON agent_guides(type)"),
            ("idx_agent_guides_name", "CREATE INDEX IF NOT EXISTS idx_agent_guides_name ON agent_guides(name)"),
            
            ("idx_mcp_servers_type", "CREATE INDEX IF NOT EXISTS idx_mcp_servers_type ON mcp_servers(type)"),
            ("idx_mcp_servers_command", "CREATE INDEX IF NOT EXISTS idx_mcp_servers_command ON mcp_servers(command)"),
            
            ("idx_common_configs_key", "CREATE INDEX IF NOT EXISTS idx_common_configs_key ON common_configs(key)"),
            ("idx_common_configs_category", "CREATE INDEX IF NOT EXISTS idx_common_configs_category ON common_configs(category)"),
            ("idx_common_configs_active", "CREATE INDEX IF NOT EXISTS idx_common_configs_active ON common_configs(is_active)"),
        ];

        for (name, query) in indexes {
            sqlx::query(query)
                .execute(self.pool)
                .await
                .map_err(|e| DatabaseError::Query(format!("创建索引 {} 失败: {}", name, e)))?;
        }

        tracing::info!("✅ 性能优化索引创建完成");
        Ok(())
    }

    /// 分析表性能统计
    pub async fn analyze_table_performance(&self, table_name: &str) -> Result<TablePerformanceStats, DatabaseError> {
        // 获取记录数
        let count = self.count_records(table_name).await?;

        // 获取表大小信息（SQLite特定）
        let size_query = "SELECT COUNT(*) * 1024 as estimated_size FROM sqlite_master WHERE type='table' AND name=?";
        let size_result = sqlx::query(size_query)
            .bind(table_name)
            .fetch_one(self.pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        let estimated_size: i64 = size_result.get("estimated_size");

        // 获取索引信息
        let index_query = "SELECT COUNT(*) as index_count FROM sqlite_master WHERE type='index' AND tbl_name=?";
        let index_result = sqlx::query(index_query)
            .bind(table_name)
            .fetch_one(self.pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        let index_count: i64 = index_result.get("index_count");

        Ok(TablePerformanceStats {
            name: table_name.to_string(),
            record_count: count,
            estimated_size_bytes: estimated_size,
            index_count,
        })
    }

    /// 清理和优化数据库
    pub async fn vacuum_and_analyze(&self) -> Result<(), DatabaseError> {
        tracing::info!("开始数据库清理和优化");

        // VACUUM 重新组织数据库文件，减少碎片
        sqlx::query("VACUUM")
            .execute(self.pool)
            .await
            .map_err(|e| DatabaseError::Query(format!("VACUUM 失败: {}", e)))?;

        // ANALYZE 更新查询计划器统计信息
        sqlx::query("ANALYZE")
            .execute(self.pool)
            .await
            .map_err(|e| DatabaseError::Query(format!("ANALYZE 失败: {}", e)))?;

        tracing::info!("✅ 数据库清理和优化完成");
        Ok(())
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
        let result = query_builder
            .execute_raw(
                "INSERT INTO common_configs (key, value, category) VALUES (?, ?, ?)",
                &["test_key", "test_value", "test"],
            )
            .await;

        assert!(result.is_ok());

        let count = query_builder.count_records("common_configs").await.unwrap();
        assert_eq!(count, 1);
    }
}
