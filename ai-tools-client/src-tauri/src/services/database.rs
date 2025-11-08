use sqlx::{SqlitePool, Row};
use anyhow::Result;
use std::collections::HashMap;

pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;

        // 运行数据库迁移
        Self::run_migrations(&pool).await?;

        Ok(Database { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    async fn run_migrations(pool: &SqlitePool) -> Result<()> {
        // 创建suppliers表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS suppliers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                type TEXT NOT NULL,
                name TEXT NOT NULL,
                base_url TEXT NOT NULL,
                api_key TEXT,
                max_tokens TEXT,
                temperature TEXT,
                is_active INTEGER DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(pool)
        .await?;

        // 创建mcp_templates表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS mcp_templates (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                version TEXT NOT NULL,
                ai_type TEXT NOT NULL,
                platform_type TEXT NOT NULL,
                config_content TEXT NOT NULL,
                description TEXT,
                is_builtin INTEGER DEFAULT 0,
                category TEXT,
                tags TEXT,
                usage_count INTEGER DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(name, version)
            )
            "#,
        )
        .execute(pool)
        .await?;

        // 创建work_mode_configs表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS work_mode_configs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mode_name TEXT NOT NULL UNIQUE,
                active_claude_supplier_id INTEGER,
                active_codex_supplier_id INTEGER,
                mcp_template_ids TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (active_claude_supplier_id) REFERENCES suppliers(id),
                FOREIGN KEY (active_codex_supplier_id) REFERENCES suppliers(id)
            )
            "#,
        )
        .execute(pool)
        .await?;

        // 创建config_history表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS config_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                config_type TEXT NOT NULL,
                config_path TEXT NOT NULL,
                backup_content TEXT NOT NULL,
                operation_type TEXT NOT NULL,
                operation_time DATETIME DEFAULT CURRENT_TIMESTAMP,
                description TEXT
            )
            "#,
        )
        .execute(pool)
        .await?;

        // 创建app_state表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS app_state (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_stats(&self) -> Result<HashMap<String, i64>> {
        let mut stats = HashMap::new();

        // 获取各表的记录数
        let suppliers_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM suppliers")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);
        stats.insert("suppliers".to_string(), suppliers_count);

        let templates_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM mcp_templates")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);
        stats.insert("mcp_templates".to_string(), templates_count);

        let work_modes_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM work_mode_configs")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);
        stats.insert("work_mode_configs".to_string(), work_modes_count);

        let config_history_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM config_history")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);
        stats.insert("config_history".to_string(), config_history_count);

        let app_state_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM app_state")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);
        stats.insert("app_state".to_string(), app_state_count);

        Ok(stats)
    }

    pub async fn test_connection(&self) -> Result<bool> {
        // 测试数据库连接
        let result: i64 = sqlx::query_scalar("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

        Ok(result == 1)
    }

    /// 静态方法：获取数据库统计信息
    pub async fn get_db_stats(pool: &SqlitePool) -> Result<HashMap<String, i64>> {
        let mut stats = HashMap::new();

        // 获取各表的记录数
        let suppliers_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM suppliers")
            .fetch_one(pool)
            .await
            .unwrap_or(0);
        stats.insert("suppliers".to_string(), suppliers_count);

        let templates_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM mcp_templates")
            .fetch_one(pool)
            .await
            .unwrap_or(0);
        stats.insert("mcp_templates".to_string(), templates_count);

        let work_modes_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM work_mode_configs")
            .fetch_one(pool)
            .await
            .unwrap_or(0);
        stats.insert("work_mode_configs".to_string(), work_modes_count);

        let config_history_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM config_history")
            .fetch_one(pool)
            .await
            .unwrap_or(0);
        stats.insert("config_history".to_string(), config_history_count);

        let app_state_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM app_state")
            .fetch_one(pool)
            .await
            .unwrap_or(0);
        stats.insert("app_state".to_string(), app_state_count);

        Ok(stats)
    }

    /// 静态方法：测试数据库连接
    pub async fn test_db_connection(pool: &SqlitePool) -> Result<bool> {
        // 测试数据库连接
        let result: i64 = sqlx::query_scalar("SELECT 1")
            .fetch_one(pool)
            .await
            .unwrap_or(0);

        Ok(result == 1)
    }
}