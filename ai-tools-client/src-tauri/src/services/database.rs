use anyhow::Result;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let connect_options = SqliteConnectOptions::from_str(database_url)?
            .create_if_missing(true)
            .foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(connect_options)
            .await?;

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
                auth_token TEXT NOT NULL DEFAULT '',
                timeout_ms INTEGER,
                auto_update INTEGER DEFAULT 0,
                opus_model TEXT,
                sonnet_model TEXT,
                haiku_model TEXT,
                is_active INTEGER DEFAULT 0,
                sort_order INTEGER DEFAULT 0,
                is_healthy INTEGER DEFAULT 1,
                last_check_time DATETIME,
                response_time INTEGER,
                consecutive_failures INTEGER DEFAULT 0,
                uptime_percentage REAL DEFAULT 100.0,
                total_requests INTEGER DEFAULT 0,
                failed_requests INTEGER DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(pool)
        .await?;
        Self::ensure_suppliers_schema(pool).await?;

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

    async fn ensure_suppliers_schema(pool: &SqlitePool) -> Result<()> {
        let rows = sqlx::query("PRAGMA table_info('suppliers')")
            .fetch_all(pool)
            .await?;

        let existing: HashSet<String> = rows
            .into_iter()
            .map(|row| row.get::<String, _>("name"))
            .collect();

        let required = [
            ("auth_token", "TEXT NOT NULL DEFAULT ''"),
            ("timeout_ms", "INTEGER"),
            ("auto_update", "INTEGER DEFAULT 0"),
            ("opus_model", "TEXT"),
            ("sonnet_model", "TEXT"),
            ("haiku_model", "TEXT"),
            ("sort_order", "INTEGER DEFAULT 0"),
            ("is_healthy", "INTEGER DEFAULT 1"),
            ("last_check_time", "DATETIME"),
            ("response_time", "INTEGER"),
            ("consecutive_failures", "INTEGER DEFAULT 0"),
            ("uptime_percentage", "REAL DEFAULT 100.0"),
            ("total_requests", "INTEGER DEFAULT 0"),
            ("failed_requests", "INTEGER DEFAULT 0"),
        ];

        for (name, definition) in required {
            if !existing.contains(name) {
                let sql = format!("ALTER TABLE suppliers ADD COLUMN {} {}", name, definition);
                sqlx::query(&sql).execute(pool).await?;
            }
        }

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

#[cfg(test)]
mod tests {
    use super::Database;
    use std::fs;
    use uuid::Uuid;

    #[tokio::test]
    async fn creates_sqlite_file_if_missing() {
        let temp_root = std::env::temp_dir().join(format!("ai-tools-test-{}", Uuid::new_v4()));
        fs::create_dir_all(&temp_root).unwrap();
        let db_path = temp_root.join("test-db.sqlite");
        let db_url = format!("sqlite://{}", db_path.to_string_lossy());

        assert!(!db_path.exists());

        {
            let db = Database::new(&db_url).await.unwrap();
            db.test_connection().await.unwrap();
        }

        assert!(db_path.exists());

        let _ = fs::remove_file(&db_path);
        let _ = fs::remove_dir_all(&temp_root);
    }
}
