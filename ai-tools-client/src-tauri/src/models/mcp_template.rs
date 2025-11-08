use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::{FromRow, SqlitePool};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct McpTemplate {
    pub id: Option<i64>,
    pub name: String,
    pub version: String,
    pub ai_type: String, // "claude" or "codex"
    pub platform_type: String, // "unix" or "windows"
    pub config_content: String,
    pub description: Option<String>,
    pub is_builtin: Option<i64>, // SQLite uses INTEGER for boolean
    pub category: Option<String>,
    pub tags: Option<String>, // JSON string
    pub usage_count: Option<i64>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMcpTemplateRequest {
    pub name: String,
    pub version: Option<String>,
    pub ai_type: String,
    pub platform_type: String,
    pub config_content: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateMcpTemplateRequest {
    pub id: i64,
    pub name: Option<String>,
    pub version: Option<String>,
    pub config_content: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpTemplateValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct McpTemplateCategory {
    pub name: String,
    pub count: i64,
    pub description: Option<String>,
}

impl McpTemplate {
    /// 创建新MCP模板
    pub async fn create(
        pool: &SqlitePool,
        request: CreateMcpTemplateRequest,
    ) -> Result<Self, sqlx::Error> {
        let now = Utc::now();
        let version = request.version.unwrap_or_else(|| "1.0.0".to_string());
        let tags_json = request.tags.as_ref().map(|tags| serde_json::to_string(tags).unwrap_or_default());

        sqlx::query_as::<_, McpTemplate>(
            r#"
            INSERT INTO mcp_templates (
                name, version, ai_type, platform_type, config_content,
                description, is_builtin, category, tags, usage_count,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, 0, ?, ?, 0, ?, ?)
            RETURNING *
            "#,
        )
        .bind(&request.name)
        .bind(&version)
        .bind(&request.ai_type)
        .bind(&request.platform_type)
        .bind(&request.config_content)
        .bind(&request.description)
        .bind(&request.category)
        .bind(&tags_json)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await
    }

    /// 获取所有MCP模板
    pub async fn get_all(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, McpTemplate>(
            "SELECT * FROM mcp_templates ORDER BY is_builtin DESC, category ASC, name ASC"
        )
        .fetch_all(pool)
        .await
    }

    /// 根据AI类型获取模板
    pub async fn get_by_ai_type(
        pool: &SqlitePool,
        ai_type: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, McpTemplate>(
            "SELECT * FROM mcp_templates WHERE ai_type = ? ORDER BY is_builtin DESC, category ASC, name ASC"
        )
        .bind(ai_type)
        .fetch_all(pool)
        .await
    }

    /// 根据平台类型获取模板
    pub async fn get_by_platform_type(
        pool: &SqlitePool,
        platform_type: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, McpTemplate>(
            "SELECT * FROM mcp_templates WHERE platform_type = ? ORDER BY is_builtin DESC, category ASC, name ASC"
        )
        .bind(platform_type)
        .fetch_all(pool)
        .await
    }

    /// 根据AI类型和平台类型获取模板
    pub async fn get_by_filters(
        pool: &SqlitePool,
        ai_type: &str,
        platform_type: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, McpTemplate>(
            "SELECT * FROM mcp_templates WHERE ai_type = ? AND platform_type = ? ORDER BY is_builtin DESC, category ASC, name ASC"
        )
        .bind(ai_type)
        .bind(platform_type)
        .fetch_all(pool)
        .await
    }

    /// 根据ID获取模板
    pub async fn get_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, McpTemplate>("SELECT * FROM mcp_templates WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    /// 更新MCP模板
    pub async fn update(
        pool: &SqlitePool,
        request: UpdateMcpTemplateRequest,
    ) -> Result<Option<Self>, sqlx::Error> {
        let now = Utc::now();
        let tags_json = request.tags.as_ref().map(|tags| serde_json::to_string(tags).unwrap_or_default());

        let result = sqlx::query_as::<_, McpTemplate>(
            r#"
            UPDATE mcp_templates SET
                name = COALESCE(?, name),
                version = COALESCE(?, version),
                config_content = COALESCE(?, config_content),
                description = COALESCE(?, description),
                category = COALESCE(?, category),
                tags = COALESCE(?, tags),
                updated_at = ?
            WHERE id = ?
            RETURNING *
            "#,
        )
        .bind(&request.name)
        .bind(&request.version)
        .bind(&request.config_content)
        .bind(&request.description)
        .bind(&request.category)
        .bind(&tags_json)
        .bind(now)
        .bind(request.id)
        .fetch_optional(pool)
        .await?;

        Ok(result)
    }

    /// 删除MCP模板
    pub async fn delete(pool: &SqlitePool, id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM mcp_templates WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 增加使用计数
    pub async fn increment_usage_count(pool: &SqlitePool, id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("UPDATE mcp_templates SET usage_count = usage_count + 1 WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 获取模板分类统计
    pub async fn get_categories(pool: &SqlitePool) -> Result<Vec<McpTemplateCategory>, sqlx::Error> {
        sqlx::query_as::<_, McpTemplateCategory>(
            r#"
            SELECT
                category as name,
                COUNT(*) as count,
                NULL as description
            FROM mcp_templates
            WHERE category IS NOT NULL AND category != ''
            GROUP BY category
            ORDER BY count DESC, name ASC
            "#,
        )
        .fetch_all(pool)
        .await
    }

    /// 验证模板配置
    pub fn validate_config(&self) -> McpTemplateValidationResult {
        let mut result = McpTemplateValidationResult {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        // 基本验证
        if self.name.trim().is_empty() {
            result.errors.push("模板名称不能为空".to_string());
            result.valid = false;
        }

        if self.config_content.trim().is_empty() {
            result.errors.push("配置内容不能为空".to_string());
            result.valid = false;
        }

        if self.ai_type != "claude" && self.ai_type != "codex" {
            result.errors.push("AI类型必须是 'claude' 或 'codex'".to_string());
            result.valid = false;
        }

        if self.platform_type != "unix" && self.platform_type != "windows" {
            result.errors.push("平台类型必须是 'unix' 或 'windows'".to_string());
            result.valid = false;
        }

        // 格式验证
        if self.ai_type == "claude" {
            // Claude使用JSON格式
            match serde_json::from_str::<serde_json::Value>(&self.config_content) {
                Ok(_) => {
                    // JSON格式正确，可以进一步验证字段
                }
                Err(_) => {
                    result.errors.push("Claude模板配置必须是有效的JSON格式".to_string());
                    result.valid = false;
                }
            }
        } else if self.ai_type == "codex" {
            // Codex使用TOML格式
            match toml::from_str::<toml::Value>(&self.config_content) {
                Ok(_) => {
                    // TOML格式正确
                }
                Err(_) => {
                    result.errors.push("Codex模板配置必须是有效的TOML格式".to_string());
                    result.valid = false;
                }
            }
        }

        result
    }

    /// 获取标签列表
    pub fn get_tags(&self) -> Vec<String> {
        match &self.tags {
            Some(tags_json) => {
                match serde_json::from_str::<Vec<String>>(tags_json) {
                    Ok(tags) => tags,
                    Err(_) => Vec::new(),
                }
            }
            None => Vec::new(),
        }
    }

    /// 设置标签列表
    pub fn set_tags(&mut self, tags: Vec<String>) {
        match serde_json::to_string(&tags) {
            Ok(tags_json) => self.tags = Some(tags_json),
            Err(_) => self.tags = None,
        }
    }

    /// 获取预览配置（用于显示）
    pub fn get_preview_config(&self) -> String {
        // 截取前200个字符作为预览
        if self.config_content.len() > 200 {
            format!("{}...", &self.config_content[..200])
        } else {
            self.config_content.clone()
        }
    }

    /// 检查是否为内置模板
    pub fn is_builtin(&self) -> bool {
        self.is_builtin.unwrap_or(0) == 1
    }

    /// 克隆模板
    pub async fn clone_template(
        pool: &SqlitePool,
        id: i64,
        new_name: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        if let Some(original) = Self::get_by_id(pool, id).await? {
            let request = CreateMcpTemplateRequest {
                name: new_name.to_string(),
                version: Some("1.0.0".to_string()),
                ai_type: original.ai_type.clone(),
                platform_type: original.platform_type.clone(),
                config_content: original.config_content.clone(),
                description: Some(format!("克隆自: {}", original.name)),
                category: original.category.clone(),
                tags: Some(original.get_tags()),
            };

            Self::create(pool, request).await.map(Some)
        } else {
            Ok(None)
        }
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
    async fn test_create_mcp_template() {
        let pool = create_test_pool().await;

        let request = CreateMcpTemplateRequest {
            name: "Test Template".to_string(),
            version: Some("1.0.0".to_string()),
            ai_type: "claude".to_string(),
            platform_type: "unix".to_string(),
            config_content: r#"{"type": "stdio", "command": "npx", "args": ["-y", "example"]}"#.to_string(),
            description: Some("Test template".to_string()),
            category: Some("test".to_string()),
            tags: Some(vec!["test".to_string(), "example".to_string()]),
        };

        let template = McpTemplate::create(&pool, request).await.unwrap();
        assert!(template.id.is_some());
        assert_eq!(template.name, "Test Template");
        assert_eq!(template.ai_type, "claude");
        assert_eq!(template.platform_type, "unix");
    }

    #[tokio::test]
    async fn test_validate_claude_template() {
        let template = McpTemplate {
            id: None,
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            ai_type: "claude".to_string(),
            platform_type: "unix".to_string(),
            config_content: r#"{"type": "stdio", "command": "npx"}"#.to_string(),
            description: None,
            is_builtin: Some(0),
            category: None,
            tags: None,
            usage_count: Some(0),
            created_at: None,
            updated_at: None,
        };

        let result = template.validate_config();
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[tokio::test]
    async fn test_validate_invalid_json() {
        let template = McpTemplate {
            id: None,
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            ai_type: "claude".to_string(),
            platform_type: "unix".to_string(),
            config_content: r#"{"type": "stdio", "command": "npx""#.to_string(), // Invalid JSON
            description: None,
            is_builtin: Some(0),
            category: None,
            tags: None,
            usage_count: Some(0),
            created_at: None,
            updated_at: None,
        };

        let result = template.validate_config();
        assert!(!result.valid);
        assert!(!result.errors.is_empty());
    }
}