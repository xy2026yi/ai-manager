// 工作模式相关模型

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkModeConfig {
    pub id: Option<i64>,
    pub mode_name: String,
    pub active_claude_supplier_id: Option<i64>,
    pub active_codex_supplier_id: Option<i64>,
    pub mcp_template_ids: Option<String>, // JSON数组
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkModeRequest {
    pub mode_name: String,
    pub active_claude_supplier_id: Option<i64>,
    pub active_codex_supplier_id: Option<i64>,
    pub mcp_template_ids: Option<Vec<i64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWorkModeRequest {
    pub id: i64,
    pub mode_name: Option<String>,
    pub active_claude_supplier_id: Option<i64>,
    pub active_codex_supplier_id: Option<i64>,
    pub mcp_template_ids: Option<Vec<i64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkModeSwitchRequest {
    pub target_mode: String,
    pub claude_supplier_id: Option<i64>,
    pub codex_supplier_id: Option<i64>,
    pub mcp_template_ids: Option<Vec<i64>>,
    pub create_backup: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkModeStatus {
    pub current_mode: String,
    pub is_transitioning: bool,
    pub last_switch_time: Option<DateTime<Utc>>,
    pub active_claude_supplier: Option<String>,
    pub active_codex_supplier: Option<String>,
    pub active_mcp_templates: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkModeSwitchResult {
    pub success: bool,
    pub message: String,
    pub backup_id: Option<i64>,
    pub applied_at: Option<DateTime<Utc>>,
    pub steps_completed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkModeSwitchStep {
    pub id: String,
    pub name: String,
    pub status: String, // "pending", "in_progress", "completed", "failed"
    pub message: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl WorkModeConfig {
    pub async fn get_all(pool: &sqlx::SqlitePool) -> Result<Vec<Self>> {
        let modes = sqlx::query_as::<_, WorkModeConfig>(
            "SELECT * FROM work_mode_configs ORDER BY updated_at DESC"
        )
        .fetch_all(pool)
        .await?;
        
        Ok(modes)
    }

    pub async fn get_by_name(pool: &sqlx::SqlitePool, mode_name: &str) -> Result<Option<Self>> {
        let mode = sqlx::query_as::<_, WorkModeConfig>(
            "SELECT * FROM work_mode_configs WHERE mode_name = ?"
        )
        .bind(mode_name)
        .fetch_optional(pool)
        .await?;
        
        Ok(mode)
    }

    pub async fn create(pool: &sqlx::SqlitePool, request: CreateWorkModeRequest) -> Result<Self> {
        let mcp_template_ids_json = request.mcp_template_ids
            .map(|ids| serde_json::to_string(&ids).unwrap_or_default());

        let mode = sqlx::query_as::<_, WorkModeConfig>(
            r#"
            INSERT INTO work_mode_configs (mode_name, active_claude_supplier_id, active_codex_supplier_id, mcp_template_ids)
            VALUES (?, ?, ?, ?)
            RETURNING *
            "#
        )
        .bind(&request.mode_name)
        .bind(request.active_claude_supplier_id)
        .bind(request.active_codex_supplier_id)
        .bind(mcp_template_ids_json)
        .fetch_one(pool)
        .await?;
        
        Ok(mode)
    }

    pub async fn update(pool: &sqlx::SqlitePool, request: UpdateWorkModeRequest) -> Result<Option<Self>> {
        let existing = Self::get_by_id(pool, request.id).await?;
        if existing.is_none() {
            return Ok(None);
        }

        let mode_name = request.mode_name.or_else(|| existing.clone().map(|m| m.mode_name));
        let claude_id = request.active_claude_supplier_id.or_else(|| existing.clone().and_then(|m| m.active_claude_supplier_id));
        let codex_id = request.active_codex_supplier_id.or_else(|| existing.clone().and_then(|m| m.active_codex_supplier_id));
        
        let mcp_template_ids_json = if let Some(ids) = request.mcp_template_ids {
            Some(serde_json::to_string(&ids).unwrap_or_default())
        } else {
            existing.and_then(|m| m.mcp_template_ids)
        };

        let updated = sqlx::query_as::<_, WorkModeConfig>(
            r#"
            UPDATE work_mode_configs 
            SET mode_name = ?, active_claude_supplier_id = ?, active_codex_supplier_id = ?, 
                mcp_template_ids = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            RETURNING *
            "#
        )
        .bind(&mode_name.unwrap())
        .bind(claude_id)
        .bind(codex_id)
        .bind(mcp_template_ids_json)
        .bind(request.id)
        .fetch_one(pool)
        .await?;
        
        Ok(Some(updated))
    }

    pub async fn get_by_id(pool: &sqlx::SqlitePool, id: i64) -> Result<Option<Self>> {
        let mode = sqlx::query_as::<_, WorkModeConfig>(
            "SELECT * FROM work_mode_configs WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;
        
        Ok(mode)
    }

    pub async fn delete(pool: &sqlx::SqlitePool, id: i64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM work_mode_configs WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        
        Ok(result.rows_affected() > 0)
    }

    pub fn get_mcp_template_ids(&self) -> Vec<i64> {
        if let Some(ids_json) = &self.mcp_template_ids {
            serde_json::from_str(ids_json).unwrap_or_default()
        } else {
            vec![]
        }
    }
}