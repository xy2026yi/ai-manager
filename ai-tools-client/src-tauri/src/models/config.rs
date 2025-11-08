use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConfigHistory {
    pub id: Option<i64>,
    pub config_type: String,
    pub config_path: String,
    pub backup_content: String,
    pub operation_type: String,
    pub operation_time: Option<DateTime<Utc>>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkModeConfig {
    pub id: Option<i64>,
    pub mode_name: String,
    pub active_claude_supplier_id: Option<i64>,
    pub active_codex_supplier_id: Option<i64>,
    pub mcp_template_ids: Option<String>, // JSON array
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AppState {
    pub key: String,
    pub value: String,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWorkModeRequest {
    pub mode_name: String,
    pub active_claude_supplier_id: Option<i64>,
    pub active_codex_supplier_id: Option<i64>,
    pub mcp_template_ids: Option<Vec<i64>>,
}

impl ConfigHistory {
    /// 创建配置历史记录
    pub async fn create(
        pool: &SqlitePool,
        config_type: &str,
        config_path: &str,
        backup_content: &str,
        operation_type: &str,
        description: Option<&str>,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, ConfigHistory>(
            r#"
            INSERT INTO config_history (config_type, config_path, backup_content, operation_type, description)
            VALUES (?, ?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(config_type)
        .bind(config_path)
        .bind(backup_content)
        .bind(operation_type)
        .bind(description)
        .fetch_one(pool)
        .await
    }

    /// 获取配置历史列表
    pub async fn get_by_type(
        pool: &SqlitePool,
        config_type: &str,
        limit: Option<i64>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let query = if let Some(limit) = limit {
            sqlx::query_as::<_, ConfigHistory>(
                "SELECT * FROM config_history WHERE config_type = ? ORDER BY operation_time DESC LIMIT ?"
            )
            .bind(config_type)
            .bind(limit)
        } else {
            sqlx::query_as::<_, ConfigHistory>(
                "SELECT * FROM config_history WHERE config_type = ? ORDER BY operation_time DESC"
            )
            .bind(config_type)
        };

        query.fetch_all(pool).await
    }

    /// 获取最近的配置历史
    pub async fn get_latest(
        pool: &SqlitePool,
        config_type: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, ConfigHistory>(
            "SELECT * FROM config_history WHERE config_type = ? ORDER BY operation_time DESC LIMIT 1"
        )
        .bind(config_type)
        .fetch_optional(pool)
        .await
    }

    /// 删除旧的配置历史（保留最近的N条）
    pub async fn cleanup_old(
        pool: &SqlitePool,
        config_type: &str,
        keep_count: i64,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM config_history
            WHERE id NOT IN (
                SELECT id FROM config_history
                WHERE config_type = ?
                ORDER BY operation_time DESC
                LIMIT ?
            )
            "#,
        )
        .bind(config_type)
        .bind(keep_count)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    /// 根据ID获取配置历史
    pub async fn get_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, ConfigHistory>("SELECT * FROM config_history WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    /// 删除配置历史
    pub async fn delete(pool: &SqlitePool, id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM config_history WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}

impl WorkModeConfig {
    /// 获取工作模式配置
    pub async fn get_by_mode(
        pool: &SqlitePool,
        mode_name: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, WorkModeConfig>("SELECT * FROM work_mode_configs WHERE mode_name = ?")
            .bind(mode_name)
            .fetch_optional(pool)
            .await
    }

    /// 获取所有工作模式配置
    pub async fn get_all(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, WorkModeConfig>("SELECT * FROM work_mode_configs ORDER BY mode_name")
            .fetch_all(pool)
            .await
    }

    /// 更新工作模式配置
    pub async fn update(
        pool: &SqlitePool,
        request: UpdateWorkModeRequest,
    ) -> Result<Self, sqlx::Error> {
        let now = Utc::now();
        let templates_json = request.mcp_template_ids.as_ref().map(|ids| serde_json::to_string(ids).unwrap_or_default());

        sqlx::query_as::<_, WorkModeConfig>(
            r#"
            INSERT INTO work_mode_configs (mode_name, active_claude_supplier_id, active_codex_supplier_id, mcp_template_ids, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(mode_name) DO UPDATE SET
                active_claude_supplier_id = excluded.active_claude_supplier_id,
                active_codex_supplier_id = excluded.active_codex_supplier_id,
                mcp_template_ids = excluded.mcp_template_ids,
                updated_at = excluded.updated_at
            RETURNING *
            "#,
        )
        .bind(&request.mode_name)
        .bind(request.active_claude_supplier_id)
        .bind(request.active_codex_supplier_id)
        .bind(&templates_json)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await
    }

    /// 获取MCP模板ID列表
    pub fn get_mcp_template_ids(&self) -> Vec<i64> {
        match &self.mcp_template_ids {
            Some(json) => {
                match serde_json::from_str::<Vec<i64>>(json) {
                    Ok(ids) => ids,
                    Err(_) => Vec::new(),
                }
            }
            None => Vec::new(),
        }
    }

    /// 设置MCP模板ID列表
    pub fn set_mcp_template_ids(&mut self, ids: Vec<i64>) {
        match serde_json::to_string(&ids) {
            Ok(json) => self.mcp_template_ids = Some(json),
            Err(_) => self.mcp_template_ids = None,
        }
    }

    /// 验证工作模式配置
    pub fn validate(&self) -> Result<(), String> {
        match self.mode_name.as_str() {
            "claude_only" | "codex_only" | "claude_codex" => Ok(()),
            _ => Err("无效的工作模式名称".to_string()),
        }
    }
}

impl AppState {
    /// 获取应用状态
    pub async fn get(pool: &SqlitePool, key: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, AppState>("SELECT * FROM app_state WHERE key = ?")
            .bind(key)
            .fetch_optional(pool)
            .await
    }

    /// 设置应用状态
    pub async fn set(
        pool: &SqlitePool,
        key: &str,
        value: &str,
    ) -> Result<Self, sqlx::Error> {
        let now = Utc::now();

        sqlx::query_as::<_, AppState>(
            r#"
            INSERT INTO app_state (key, value, updated_at)
            VALUES (?, ?, ?)
            ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                updated_at = excluded.updated_at
            RETURNING *
            "#,
        )
        .bind(key)
        .bind(value)
        .bind(now)
        .fetch_one(pool)
        .await
    }

    /// 删除应用状态
    pub async fn delete(pool: &SqlitePool, key: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM app_state WHERE key = ?")
            .bind(key)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 获取所有应用状态
    pub async fn get_all(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, AppState>("SELECT * FROM app_state ORDER BY key")
            .fetch_all(pool)
            .await
    }

    /// 获取当前工作模式
    pub async fn get_current_mode(pool: &SqlitePool) -> Result<String, sqlx::Error> {
        match Self::get(pool, "current_mode").await? {
            Some(state) => Ok(state.value),
            None => {
                // 如果没有设置，返回默认值
                Self::set(pool, "current_mode", "claude_only").await?;
                Ok("claude_only".to_string())
            }
        }
    }

    /// 设置当前工作模式
    pub async fn set_current_mode(pool: &SqlitePool, mode: &str) -> Result<Self, sqlx::Error> {
        Self::set(pool, "current_mode", mode).await
    }

    /// 获取应用版本
    pub async fn get_app_version(pool: &SqlitePool) -> Result<String, sqlx::Error> {
        match Self::get(pool, "app_version").await? {
            Some(state) => Ok(state.value),
            None => Ok("0.1.0".to_string()),
        }
    }

    /// 设置应用版本
    pub async fn set_app_version(pool: &SqlitePool, version: &str) -> Result<Self, sqlx::Error> {
        Self::set(pool, "app_version", version).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;
    use tempfile::tempdir;

    async fn create_test_pool() -> SqlitePool {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db_url = format!("sqlite:{}", db_path.display());

        crate::database::Database::new(&db_url).await.unwrap();
        SqlitePool::connect(&db_url).await.unwrap()
    }

    #[tokio::test]
    async fn test_app_state() {
        let pool = create_test_pool().await;

        // 测试设置和获取应用状态
        let state = AppState::set(&pool, "test_key", "test_value").await.unwrap();
        assert_eq!(state.key, "test_key");
        assert_eq!(state.value, "test_value");

        let retrieved = AppState::get(&pool, "test_key").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().value, "test_value");

        // 测试当前工作模式
        let mode = AppState::get_current_mode(&pool).await.unwrap();
        assert_eq!(mode, "claude_only");

        AppState::set_current_mode(&pool, "codex_only").await.unwrap();
        let updated_mode = AppState::get_current_mode(&pool).await.unwrap();
        assert_eq!(updated_mode, "codex_only");
    }

    #[tokio::test]
    async fn test_work_mode_config() {
        let pool = create_test_pool().await;

        let request = UpdateWorkModeRequest {
            mode_name: "claude_only".to_string(),
            active_claude_supplier_id: Some(1),
            active_codex_supplier_id: None,
            mcp_template_ids: Some(vec![1, 2, 3]),
        };

        let config = WorkModeConfig::update(&pool, request).await.unwrap();
        assert_eq!(config.mode_name, "claude_only");
        assert_eq!(config.active_claude_supplier_id, Some(1));
        assert_eq!(config.active_codex_supplier_id, None);

        let templates = config.get_mcp_template_ids();
        assert_eq!(templates, vec![1, 2, 3]);

        // 验证工作模式
        assert!(config.validate().is_ok());
    }

    #[tokio::test]
    async fn test_config_history() {
        let pool = create_test_pool().await;

        let history = ConfigHistory::create(
            &pool,
            "claude",
            "/home/user/.claude.json",
            "{\"test\": true}",
            "backup",
            Some("测试备份"),
        )
        .await
        .unwrap();

        assert_eq!(history.config_type, "claude");
        assert_eq!(history.operation_type, "backup");

        let retrieved = ConfigHistory::get_by_id(&pool, history.id.unwrap()).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().config_type, "claude");
    }
}