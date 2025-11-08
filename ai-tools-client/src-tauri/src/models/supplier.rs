use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::{FromRow, SqlitePool};

// 健康状态枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

// 故障转移触发条件类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionType {
    ConsecutiveFailures,
    ResponseTime,
    SuccessRate,
}

// 供应商切换原因枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SwitchReason {
    Manual,
    AutoFailover,
    HealthCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Supplier {
    pub id: Option<i64>,
    pub r#type: String, // "claude" or "codex"
    pub name: String,
    pub base_url: String,
    pub auth_token: String,
    pub timeout_ms: Option<i64>,
    pub auto_update: Option<i64>, // SQLite uses INTEGER for boolean
    pub opus_model: Option<String>,
    pub sonnet_model: Option<String>,
    pub haiku_model: Option<String>,
    pub is_active: Option<i64>, // SQLite uses INTEGER for boolean
    pub sort_order: Option<i64>,
    // 健康检查相关字段
    pub is_healthy: Option<i64>, // SQLite uses INTEGER for boolean
    pub last_check_time: Option<DateTime<Utc>>,
    pub response_time: Option<i64>, // 毫秒
    pub consecutive_failures: Option<i64>, // 连续失败次数
    pub uptime_percentage: Option<f64>, // 运行时间百分比
    pub total_requests: Option<i64>, // 总请求数
    pub failed_requests: Option<i64>, // 失败请求数
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSupplierRequest {
    pub r#type: String,
    pub name: String,
    pub base_url: String,
    pub auth_token: String,
    pub timeout_ms: Option<i64>,
    pub auto_update: Option<bool>,
    pub opus_model: Option<String>,
    pub sonnet_model: Option<String>,
    pub haiku_model: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSupplierRequest {
    pub id: i64,
    pub name: Option<String>,
    pub base_url: Option<String>,
    pub auth_token: Option<String>,
    pub timeout_ms: Option<i64>,
    pub auto_update: Option<bool>,
    pub opus_model: Option<String>,
    pub sonnet_model: Option<String>,
    pub haiku_model: Option<String>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionTestResult {
    pub success: bool,
    pub response_time: Option<i64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierHealth {
    pub supplier_id: i64,
    pub is_healthy: bool,
    pub last_check_time: DateTime<Utc>,
    pub response_time: i64,
    pub consecutive_failures: i64,
    pub uptime_percentage: f64,
    pub total_requests: i64,
    pub failed_requests: i64,
    pub status: HealthStatus,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SupplierSwitchProgress {
    pub total_steps: u8,
    pub completed_steps: u8,
    pub overall_progress: u8, // 0-100
    pub current_step: String,
    pub from_supplier: i64,
    pub to_supplier: i64,
    pub start_time: DateTime<Utc>,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub rollback_available: bool,
    pub is_completed: bool,
    pub has_error: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FailoverConfig {
    pub enabled: bool,
    pub trigger_conditions: Vec<FailoverTrigger>,
    pub auto_rollback: bool,
    pub rollback_delay_seconds: u32,
    pub max_consecutive_failures: u32,
    pub max_response_time_ms: u32,
    pub min_success_rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FailoverTrigger {
    pub condition_type: ConditionType,
    pub threshold: f64,
    pub evaluation_window_minutes: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SupplierSwitchRequest {
    pub from_supplier_id: i64,
    pub to_supplier_id: i64,
    pub switch_reason: SwitchReason,
    pub create_backup: bool,
    pub rollback_on_failure: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SupplierSwitchResult {
    pub success: bool,
    pub message: String,
    pub from_supplier_id: i64,
    pub to_supplier_id: i64,
    pub switch_time: DateTime<Utc>,
    pub rollback_available: bool,
    pub backup_id: Option<i64>,
    pub error: Option<String>,
}

impl Supplier {
    /// 创建新供应商
    pub async fn create(
        pool: &SqlitePool,
        request: CreateSupplierRequest,
    ) -> Result<Self, sqlx::Error> {
        let now = Utc::now();

        sqlx::query_as::<_, Supplier>(
            r#"
            INSERT INTO suppliers (
                type, name, base_url, auth_token, timeout_ms, auto_update,
                opus_model, sonnet_model, haiku_model, is_active, sort_order,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 1, 0, ?, ?)
            RETURNING *
            "#,
        )
        .bind(&request.r#type)
        .bind(&request.name)
        .bind(&request.base_url)
        .bind(&request.auth_token)
        .bind(request.timeout_ms)
        .bind(request.auto_update.map(|b| if b { 1 } else { 0 }))
        .bind(&request.opus_model)
        .bind(&request.sonnet_model)
        .bind(&request.haiku_model)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await
    }

    /// 获取所有供应商
    pub async fn get_all(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Supplier>(
            "SELECT * FROM suppliers ORDER BY sort_order ASC, name ASC"
        )
        .fetch_all(pool)
        .await
    }

    /// 根据类型获取供应商
    pub async fn get_by_type(
        pool: &SqlitePool,
        supplier_type: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Supplier>(
            "SELECT * FROM suppliers WHERE type = ? ORDER BY sort_order ASC, name ASC"
        )
        .bind(supplier_type)
        .fetch_all(pool)
        .await
    }

    /// 获取激活的供应商
    pub async fn get_active(
        pool: &SqlitePool,
        supplier_type: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Supplier>(
            "SELECT * FROM suppliers WHERE type = ? AND is_active = 1 LIMIT 1"
        )
        .bind(supplier_type)
        .fetch_optional(pool)
        .await
    }

    /// 根据ID获取供应商
    pub async fn get_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Supplier>("SELECT * FROM suppliers WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    /// 更新供应商
    pub async fn update(
        pool: &SqlitePool,
        request: UpdateSupplierRequest,
    ) -> Result<Option<Self>, sqlx::Error> {
        let now = Utc::now();

        let result = sqlx::query_as::<_, Supplier>(
            r#"
            UPDATE suppliers SET
                name = COALESCE(?, name),
                base_url = COALESCE(?, base_url),
                auth_token = COALESCE(?, auth_token),
                timeout_ms = COALESCE(?, timeout_ms),
                auto_update = COALESCE(?, auto_update),
                opus_model = COALESCE(?, opus_model),
                sonnet_model = COALESCE(?, sonnet_model),
                haiku_model = COALESCE(?, haiku_model),
                is_active = COALESCE(?, is_active),
                sort_order = COALESCE(?, sort_order),
                updated_at = ?
            WHERE id = ?
            RETURNING *
            "#,
        )
        .bind(&request.name)
        .bind(&request.base_url)
        .bind(&request.auth_token)
        .bind(request.timeout_ms)
        .bind(request.is_active.map(|b| if b { 1 } else { 0 }))
        .bind(&request.opus_model)
        .bind(&request.sonnet_model)
        .bind(&request.haiku_model)
        .bind(request.is_active.map(|b| if b { 1 } else { 0 }))
        .bind(request.sort_order)
        .bind(now)
        .bind(request.id)
        .fetch_optional(pool)
        .await?;

        Ok(result)
    }

    /// 删除供应商
    pub async fn delete(pool: &SqlitePool, id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM suppliers WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 设置激活状态
    pub async fn set_active(
        pool: &SqlitePool,
        id: i64,
        is_active: bool,
    ) -> Result<bool, sqlx::Error> {
        // 先将该类型的所有供应商设为非激活
        sqlx::query("UPDATE suppliers SET is_active = 0 WHERE type = (SELECT type FROM suppliers WHERE id = ?)")
            .bind(id)
            .execute(pool)
            .await?;

        // 设置指定供应商为激活
        let result = sqlx::query("UPDATE suppliers SET is_active = ? WHERE id = ?")
            .bind(if is_active { 1 } else { 0 })
            .bind(id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 测试供应商连接
    pub async fn test_connection(&self) -> ConnectionTestResult {
        let start = std::time::Instant::now();

        // TODO: 实现实际的连接测试逻辑
        // 这里暂时模拟一个简单的测试
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        let elapsed = start.elapsed();

        // 模拟连接测试（总是成功）
        ConnectionTestResult {
            success: true,
            response_time: Some(elapsed.as_millis() as i64),
            error: None,
        }
    }

    /// 验证供应商配置
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("供应商名称不能为空".to_string());
        }

        if self.base_url.trim().is_empty() {
            return Err("访问URL不能为空".to_string());
        }

        if self.auth_token.trim().is_empty() {
            return Err("访问密钥不能为空".to_string());
        }

        // 验证URL格式
        if !self.base_url.starts_with("http://") && !self.base_url.starts_with("https://") {
            return Err("访问URL格式不正确".to_string());
        }

        // 验证类型
        if self.r#type != "claude" && self.r#type != "codex" {
            return Err("供应商类型必须是 'claude' 或 'codex'".to_string());
        }

        Ok(())
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
    async fn test_create_supplier() {
        let pool = create_test_pool().await;

        let request = CreateSupplierRequest {
            r#type: "claude".to_string(),
            name: "Test Supplier".to_string(),
            base_url: "https://api.example.com".to_string(),
            auth_token: "test_token".to_string(),
            timeout_ms: Some(30000),
            auto_update: Some(true),
            opus_model: Some("claude-3-opus-20240229".to_string()),
            sonnet_model: Some("claude-3-sonnet-20240229".to_string()),
            haiku_model: Some("claude-3-haiku-20240307".to_string()),
        };

        let supplier = Supplier::create(&pool, request).await.unwrap();
        assert!(supplier.id.is_some());
        assert_eq!(supplier.name, "Test Supplier");
        assert_eq!(supplier.r#type, "claude");
    }

    #[tokio::test]
    async fn test_get_suppliers() {
        let pool = create_test_pool().await;

        let request = CreateSupplierRequest {
            r#type: "claude".to_string(),
            name: "Test Supplier".to_string(),
            base_url: "https://api.example.com".to_string(),
            auth_token: "test_token".to_string(),
            timeout_ms: None,
            auto_update: None,
            opus_model: None,
            sonnet_model: None,
            haiku_model: None,
        };

        Supplier::create(&pool, request).await.unwrap();

        let suppliers = Supplier::get_all(&pool).await.unwrap();
        assert_eq!(suppliers.len(), 1);

        let claude_suppliers = Supplier::get_by_type(&pool, "claude").await.unwrap();
        assert_eq!(claude_suppliers.len(), 1);

        let codex_suppliers = Supplier::get_by_type(&pool, "codex").await.unwrap();
        assert_eq!(codex_suppliers.len(), 0);
    }
}