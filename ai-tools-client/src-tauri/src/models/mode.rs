// 工作模式相关模型

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkMode {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub supplier_config: Option<String>, // JSON配置
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkModeRequest {
    pub name: String,
    pub description: Option<String>,
    pub supplier_config: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWorkModeRequest {
    pub id: i64,
    pub name: Option<String>,
    pub description: Option<String>,
    pub supplier_config: Option<String>,
}