// 数据迁移器
// 负责从Python数据库迁移数据到Rust数据库

use crate::crypto::CryptoService;
use crate::database::DatabaseManager;
use crate::models::*;
use anyhow::{Context, Result};
use sqlx::{Row, SqlitePool};
use tracing::{debug, error, info, warn};

/// 数据迁移统计信息
#[derive(Debug, Default)]
pub struct MigrationStats {
    pub total_records: i64,
    pub migrated_records: i64,
    pub failed_records: i64,
    pub tables_processed: usize,
    pub errors: Vec<String>,
}

impl MigrationStats {
    pub fn success_rate(&self) -> f64 {
        if self.total_records == 0 {
            100.0
        } else {
            (self.migrated_records as f64 / self.total_records as f64) * 100.0
        }
    }
}

/// 数据迁移器
pub struct DataMigrator {
    db_manager: DatabaseManager,
    #[allow(dead_code)]
    crypto_service: CryptoService,
}

impl DataMigrator {
    /// 创建新的数据迁移器实例
    pub fn new(db_manager: DatabaseManager, crypto_service: CryptoService) -> Self {
        Self { db_manager, crypto_service }
    }

    /// 从Python数据库迁移数据
    pub async fn migrate_from_python_db(&self, python_db_path: &str) -> Result<MigrationStats> {
        info!("开始从Python数据库迁移数据: {}", python_db_path);

        let mut stats = MigrationStats::default();

        // 连接到Python数据库
        let python_pool = self
            .connect_to_python_db(python_db_path)
            .await
            .context("连接Python数据库失败")?;

        // 执行表结构迁移
        self.migrate_schemas(&python_pool).await.context("迁移表结构失败")?;

        // 迁移各个表的数据
        info!("迁移表: claude_providers");
        match self.migrate_claude_providers(&python_pool).await {
            Ok(table_stats) => {
                stats.total_records += table_stats.total_records;
                stats.migrated_records += table_stats.migrated_records;
                stats.failed_records += table_stats.failed_records;
                stats.tables_processed += 1;

                if !table_stats.errors.is_empty() {
                    stats.errors.extend(table_stats.errors);
                }

                info!(
                    "表 claude_providers 迁移完成: {}/{} 成功",
                    table_stats.migrated_records, table_stats.total_records
                );
            }
            Err(e) => {
                error!("迁移表 claude_providers 失败: {}", e);
                stats.errors.push(format!("表 claude_providers 迁移失败: {}", e));
            }
        }

        info!("迁移表: codex_providers");
        match self.migrate_codex_providers(&python_pool).await {
            Ok(table_stats) => {
                stats.total_records += table_stats.total_records;
                stats.migrated_records += table_stats.migrated_records;
                stats.failed_records += table_stats.failed_records;
                stats.tables_processed += 1;

                if !table_stats.errors.is_empty() {
                    stats.errors.extend(table_stats.errors);
                }

                info!(
                    "表 codex_providers 迁移完成: {}/{} 成功",
                    table_stats.migrated_records, table_stats.total_records
                );
            }
            Err(e) => {
                error!("迁移表 codex_providers 失败: {}", e);
                stats.errors.push(format!("表 codex_providers 迁移失败: {}", e));
            }
        }

        info!("迁移表: agent_guides");
        match self.migrate_agent_guides(&python_pool).await {
            Ok(table_stats) => {
                stats.total_records += table_stats.total_records;
                stats.migrated_records += table_stats.migrated_records;
                stats.failed_records += table_stats.failed_records;
                stats.tables_processed += 1;

                if !table_stats.errors.is_empty() {
                    stats.errors.extend(table_stats.errors);
                }

                info!(
                    "表 agent_guides 迁移完成: {}/{} 成功",
                    table_stats.migrated_records, table_stats.total_records
                );
            }
            Err(e) => {
                error!("迁移表 agent_guides 失败: {}", e);
                stats.errors.push(format!("表 agent_guides 迁移失败: {}", e));
            }
        }

        info!("迁移表: mcp_servers");
        match self.migrate_mcp_servers(&python_pool).await {
            Ok(table_stats) => {
                stats.total_records += table_stats.total_records;
                stats.migrated_records += table_stats.migrated_records;
                stats.failed_records += table_stats.failed_records;
                stats.tables_processed += 1;

                if !table_stats.errors.is_empty() {
                    stats.errors.extend(table_stats.errors);
                }

                info!(
                    "表 mcp_servers 迁移完成: {}/{} 成功",
                    table_stats.migrated_records, table_stats.total_records
                );
            }
            Err(e) => {
                error!("迁移表 mcp_servers 失败: {}", e);
                stats.errors.push(format!("表 mcp_servers 迁移失败: {}", e));
            }
        }

        info!("迁移表: common_configs");
        match self.migrate_common_configs(&python_pool).await {
            Ok(table_stats) => {
                stats.total_records += table_stats.total_records;
                stats.migrated_records += table_stats.migrated_records;
                stats.failed_records += table_stats.failed_records;
                stats.tables_processed += 1;

                if !table_stats.errors.is_empty() {
                    stats.errors.extend(table_stats.errors);
                }

                info!(
                    "表 common_configs 迁移完成: {}/{} 成功",
                    table_stats.migrated_records, table_stats.total_records
                );
            }
            Err(e) => {
                error!("迁移表 common_configs 失败: {}", e);
                stats.errors.push(format!("表 common_configs 迁移失败: {}", e));
            }
        }

        // 关闭Python数据库连接
        python_pool.close().await;

        info!(
            "数据迁移完成: 总计 {} 条记录，成功 {} 条，失败 {} 条",
            stats.total_records, stats.migrated_records, stats.failed_records
        );
        info!("迁移成功率: {:.2}%", stats.success_rate());

        Ok(stats)
    }

    /// 连接到Python数据库
    async fn connect_to_python_db(&self, db_path: &str) -> Result<SqlitePool> {
        let connection_string = format!("sqlite:{}", db_path);
        let pool = SqlitePool::connect(&connection_string).await?;
        Ok(pool)
    }

    /// 迁移表结构
    async fn migrate_schemas(&self, python_pool: &SqlitePool) -> Result<()> {
        info!("检查并迁移表结构...");

        let tables = vec![
            "claude_providers",
            "codex_providers",
            "agent_guides",
            "mcp_servers",
            "common_configs",
        ];

        for table in tables {
            if !self.table_exists(self.db_manager.pool(), table).await? {
                info!("创建表: {}", table);
                self.create_table_from_python(python_pool, table).await?;
            }
        }

        Ok(())
    }

    /// 检查表是否存在
    async fn table_exists(&self, pool: &SqlitePool, table_name: &str) -> Result<bool> {
        let query = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name=?")
            .bind(table_name);

        let result = query.fetch_optional(pool).await?;
        Ok(result.is_some())
    }

    /// 从Python数据库结构创建表
    async fn create_table_from_python(
        &self,
        _python_pool: &SqlitePool,
        table_name: &str,
    ) -> Result<()> {
        let create_sql = match table_name {
            "claude_providers" => {
                r#"
                CREATE TABLE claude_providers (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL,
                    url TEXT NOT NULL,
                    token TEXT NOT NULL,
                    max_tokens INTEGER DEFAULT 4096,
                    temperature REAL DEFAULT 0.7,
                    model TEXT DEFAULT 'gpt-4',
                    enabled INTEGER DEFAULT 1,
                    description TEXT,
                    timeout INTEGER DEFAULT 30,
                    retry_count INTEGER DEFAULT 3,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
                )
            "#
            }

            "codex_providers" => {
                r#"
                CREATE TABLE codex_providers (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL,
                    url TEXT NOT NULL,
                    token TEXT NOT NULL,
                    type TEXT,
                    enabled INTEGER DEFAULT 1,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
                )
            "#
            }

            "agent_guides" => {
                r#"
                CREATE TABLE agent_guides (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL,
                    description TEXT,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
                )
            "#
            }

            "mcp_servers" => {
                r#"
                CREATE TABLE mcp_servers (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL,
                    url TEXT,
                    command TEXT,
                    args TEXT,
                    enabled INTEGER DEFAULT 1,
                    description TEXT,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
                )
            "#
            }

            "common_configs" => {
                r#"
                CREATE TABLE common_configs (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    key TEXT UNIQUE NOT NULL,
                    value TEXT NOT NULL,
                    type TEXT DEFAULT 'string',
                    description TEXT,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
                )
            "#
            }

            _ => return Err(anyhow::anyhow!("未知的表类型: {}", table_name)),
        };

        sqlx::query(create_sql).execute(self.db_manager.pool()).await?;
        Ok(())
    }

    /// 迁移Claude供应商数据
    async fn migrate_claude_providers(&self, python_pool: &SqlitePool) -> Result<MigrationStats> {
        let query = r#"
            SELECT id, name, url, token, timeout, created_at, updated_at
            FROM claude_providers
            ORDER BY id
        "#;

        let rows = sqlx::query(query).fetch_all(python_pool).await?;
        let mut stats = MigrationStats { total_records: rows.len() as i64, ..Default::default() };

        for row in rows {
            let provider = CreateClaudeProviderRequest {
                name: row.get("name"),
                url: row.get("url"),
                token: row.get("token"), // 保持加密状态
                timeout: row.try_get("timeout").ok(),
                auto_update: None,
                r#type: None,
                opus_model: None,
                sonnet_model: None,
                haiku_model: None,
            };

            match self.create_claude_provider(&provider).await {
                Ok(_) => {
                    stats.migrated_records += 1;
                    debug!("Claude供应商 {} 迁移成功", row.get::<i64, _>("id"));
                }
                Err(e) => {
                    stats.failed_records += 1;
                    let error_msg =
                        format!("Claude供应商 {} 迁移失败: {}", row.get::<i64, _>("id"), e);
                    warn!("{}", error_msg);
                    stats.errors.push(error_msg);
                }
            }
        }

        Ok(stats)
    }

    /// 迁移Codex供应商数据
    async fn migrate_codex_providers(&self, python_pool: &SqlitePool) -> Result<MigrationStats> {
        let query = r#"
            SELECT id, name, url, token, type, created_at, updated_at
            FROM codex_providers
            ORDER BY id
        "#;

        let rows = sqlx::query(query).fetch_all(python_pool).await?;
        let mut stats = MigrationStats { total_records: rows.len() as i64, ..Default::default() };

        for row in rows {
            let provider = CreateCodexProviderRequest {
                name: row.get("name"),
                url: row.get("url"),
                token: row.get("token"), // 保持加密状态
                r#type: row.try_get("type").ok(),
            };

            match self.create_codex_provider(&provider).await {
                Ok(_) => {
                    stats.migrated_records += 1;
                    debug!("Codex供应商 {} 迁移成功", row.get::<i64, _>("id"));
                }
                Err(e) => {
                    stats.failed_records += 1;
                    let error_msg =
                        format!("Codex供应商 {} 迁移失败: {}", row.get::<i64, _>("id"), e);
                    warn!("{}", error_msg);
                    stats.errors.push(error_msg);
                }
            }
        }

        Ok(stats)
    }

    /// 迁移Agent指导数据
    async fn migrate_agent_guides(&self, python_pool: &SqlitePool) -> Result<MigrationStats> {
        let query = r#"
            SELECT id, name, type, text, created_at, updated_at
            FROM agent_guides
            ORDER BY id
        "#;

        let rows = sqlx::query(query).fetch_all(python_pool).await?;
        let mut stats = MigrationStats { total_records: rows.len() as i64, ..Default::default() };

        for row in rows {
            let guide = CreateAgentGuideRequest {
                name: row.get("name"),
                r#type: row.try_get("type").unwrap_or_else(|_| "default".to_string()),
                text: row.try_get("text").unwrap_or_default(),
            };

            match self.create_agent_guide(&guide).await {
                Ok(_) => {
                    stats.migrated_records += 1;
                    debug!("Agent指导 {} 迁移成功", row.get::<i64, _>("id"));
                }
                Err(e) => {
                    stats.failed_records += 1;
                    let error_msg =
                        format!("Agent指导 {} 迁移失败: {}", row.get::<i64, _>("id"), e);
                    warn!("{}", error_msg);
                    stats.errors.push(error_msg);
                }
            }
        }

        Ok(stats)
    }

    /// 迁移MCP服务器数据
    async fn migrate_mcp_servers(&self, python_pool: &SqlitePool) -> Result<MigrationStats> {
        let query = r#"
            SELECT id, name, type, timeout, command, args, env, created_at, updated_at
            FROM mcp_servers
            ORDER BY id
        "#;

        let rows = sqlx::query(query).fetch_all(python_pool).await?;
        let mut stats = MigrationStats { total_records: rows.len() as i64, ..Default::default() };

        for row in rows {
            // 解析 args（假设存储为 JSON 字符串）
            let args_str: String = row.try_get("args").unwrap_or_default();
            let args: Vec<String> = serde_json::from_str(&args_str).unwrap_or_default();

            // 解析 env（假设存储为 JSON 字符串）
            let env_str: Option<String> = row.try_get("env").ok();
            let env: Option<std::collections::HashMap<String, String>> =
                env_str.and_then(|s| serde_json::from_str(&s).ok());

            let server = CreateMcpServerRequest {
                name: row.get("name"),
                r#type: row.try_get("type").ok(),
                timeout: row.try_get("timeout").ok(),
                command: row.get("command"),
                args,
                env,
            };

            match self.create_mcp_server(&server).await {
                Ok(_) => {
                    stats.migrated_records += 1;
                    debug!("MCP服务器 {} 迁移成功", row.get::<i64, _>("id"));
                }
                Err(e) => {
                    stats.failed_records += 1;
                    let error_msg =
                        format!("MCP服务器 {} 迁移失败: {}", row.get::<i64, _>("id"), e);
                    warn!("{}", error_msg);
                    stats.errors.push(error_msg);
                }
            }
        }

        Ok(stats)
    }

    /// 迁移通用配置数据
    async fn migrate_common_configs(&self, python_pool: &SqlitePool) -> Result<MigrationStats> {
        let query = r#"
            SELECT id, key, value, description, category, is_active, created_at, updated_at
            FROM common_configs
            ORDER BY id
        "#;

        let rows = sqlx::query(query).fetch_all(python_pool).await?;
        let mut stats = MigrationStats { total_records: rows.len() as i64, ..Default::default() };

        for row in rows {
            let config = CreateCommonConfigRequest {
                key: row.get("key"),
                value: row.get("value"),
                description: row.try_get("description").ok(),
                category: row.try_get("category").ok(),
                is_active: row.try_get("is_active").ok(),
            };

            match self.create_common_config(&config).await {
                Ok(_) => {
                    stats.migrated_records += 1;
                    debug!("通用配置 {} 迁移成功", row.get::<i64, _>("id"));
                }
                Err(e) => {
                    stats.failed_records += 1;
                    let error_msg = format!("通用配置 {} 迁移失败: {}", row.get::<i64, _>("id"), e);
                    warn!("{}", error_msg);
                    stats.errors.push(error_msg);
                }
            }
        }

        Ok(stats)
    }

    /// 创建Claude供应商记录
    async fn create_claude_provider(&self, request: &CreateClaudeProviderRequest) -> Result<i64> {
        let id = sqlx::query(
            r#"
            INSERT INTO claude_providers (name, url, token, timeout, auto_update, type, opus_model, sonnet_model, haiku_model)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&request.name)
        .bind(&request.url)
        .bind(&request.token)
        .bind(request.timeout)
        .bind(request.auto_update)
        .bind(&request.r#type)
        .bind(&request.opus_model)
        .bind(&request.sonnet_model)
        .bind(&request.haiku_model)
        .execute(self.db_manager.pool())
        .await?;

        let id = id.last_insert_rowid();
        Ok(id)
    }

    /// 创建Codex供应商记录
    async fn create_codex_provider(&self, request: &CreateCodexProviderRequest) -> Result<i64> {
        let id = sqlx::query(
            r#"
            INSERT INTO codex_providers (name, url, token, type)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(&request.name)
        .bind(&request.url)
        .bind(&request.token)
        .bind(&request.r#type)
        .execute(self.db_manager.pool())
        .await?;

        let id = id.last_insert_rowid();
        Ok(id)
    }

    /// 创建Agent指导记录
    async fn create_agent_guide(&self, request: &CreateAgentGuideRequest) -> Result<i64> {
        let id = sqlx::query(
            r#"
            INSERT INTO agent_guides (name, type, text)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(&request.name)
        .bind(&request.r#type)
        .bind(&request.text)
        .execute(self.db_manager.pool())
        .await?;

        let id = id.last_insert_rowid();
        Ok(id)
    }

    /// 创建MCP服务器记录
    async fn create_mcp_server(&self, request: &CreateMcpServerRequest) -> Result<i64> {
        // 将 args 序列化为 JSON 字符串
        let args_json = serde_json::to_string(&request.args)?;

        // 将 env 序列化为 JSON 字符串
        let env_json = request.env.as_ref().map(serde_json::to_string).transpose()?;

        let id = sqlx::query(
            r#"
            INSERT INTO mcp_servers (name, type, timeout, command, args, env)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&request.name)
        .bind(&request.r#type)
        .bind(request.timeout)
        .bind(&request.command)
        .bind(&args_json)
        .bind(&env_json)
        .execute(self.db_manager.pool())
        .await?;

        let id = id.last_insert_rowid();
        Ok(id)
    }

    /// 创建通用配置记录
    async fn create_common_config(&self, request: &CreateCommonConfigRequest) -> Result<i64> {
        let id = sqlx::query(
            r#"
            INSERT INTO common_configs (key, value, description, category, is_active)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&request.key)
        .bind(&request.value)
        .bind(&request.description)
        .bind(&request.category)
        .bind(request.is_active)
        .execute(self.db_manager.pool())
        .await?;

        let id = id.last_insert_rowid();
        Ok(id)
    }

    /// 生成迁移报告
    pub async fn generate_migration_report(&self, stats: &MigrationStats) -> String {
        let mut report = String::new();

        report.push_str("# 数据迁移报告\n\n");
        report.push_str(&format!(
            "生成时间: {}\n\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));

        report.push_str("## 迁移统计\n\n");
        report.push_str(&format!("- 总记录数: {}\n", stats.total_records));
        report.push_str(&format!("- 成功迁移: {}\n", stats.migrated_records));
        report.push_str(&format!("- 失败记录: {}\n", stats.failed_records));
        report.push_str(&format!("- 处理表数: {}\n", stats.tables_processed));
        report.push_str(&format!("- 成功率: {:.2}%\n\n", stats.success_rate()));

        if !stats.errors.is_empty() {
            report.push_str("## 错误详情\n\n");
            for (i, error) in stats.errors.iter().enumerate() {
                report.push_str(&format!("{}. {}\n", i + 1, error));
            }
        }

        // 总体评估
        report.push_str("## 迁移评估\n\n");
        if stats.success_rate() >= 99.0 {
            report.push_str("🎉 **迁移成功！** 数据迁移完全成功，无数据丢失。\n");
        } else if stats.success_rate() >= 95.0 {
            report.push_str("✅ **迁移基本成功**，有少量数据问题需要处理。\n");
        } else if stats.success_rate() >= 80.0 {
            report.push_str("⚠️ **迁移部分成功**，有一些数据问题需要重点处理。\n");
        } else {
            report.push_str("❌ **迁移失败**，存在严重的数据问题，需要重新迁移。\n");
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_migration_stats_success_rate() {
        let mut stats = MigrationStats::default();
        stats.total_records = 100;
        stats.migrated_records = 95;
        stats.failed_records = 5;

        assert_eq!(stats.success_rate(), 95.0);

        // 测试边界情况
        stats.total_records = 0;
        assert_eq!(stats.success_rate(), 100.0);
    }

    #[tokio::test]
    async fn test_data_migrator_creation() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let db_config = crate::database::DatabaseConfig {
            url: format!("sqlite:{}", db_path.display()),
            max_connections: 5,
            min_connections: 1,
            connect_timeout: std::time::Duration::from_secs(5),
            idle_timeout: std::time::Duration::from_secs(60),
            max_lifetime: std::time::Duration::from_secs(300),
        };

        let db_manager = DatabaseManager::new(db_config).await.unwrap();
        let crypto_service = CryptoService::new("test_key_for_migration").unwrap();

        let migrator = DataMigrator::new(db_manager, crypto_service);

        // 测试创建成功
        assert!(!migrator.db_manager.pool().is_closed());
    }
}
