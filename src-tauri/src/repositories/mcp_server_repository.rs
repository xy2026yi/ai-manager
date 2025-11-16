// MCP服务器Repository实现
//
// 提供MCP服务器的特定数据访问操作

use crate::crypto::CryptoService;
use crate::database::DatabaseManager;
use crate::models::{CreateMcpServerRequest, McpServer, UpdateMcpServerRequest};
use crate::repositories::base_repository::{BaseRepository, RepositoryError, RepositoryResult};
use serde_json;
use sqlx::{FromRow, SqlitePool};

/// MCP服务器Repository
pub struct McpServerRepository {
    pool: SqlitePool,
    crypto_service: CryptoService,
}

impl McpServerRepository {
    /// 创建新的MCP服务器Repository实例
    pub fn new(db_manager: &DatabaseManager, crypto_service: &CryptoService) -> Self {
        Self {
            pool: db_manager.pool().clone(),
            crypto_service: crypto_service.clone(),
        }
    }

    /// 创建MCP服务器记录
    pub async fn create_mcp_server(
        &self,
        request: &CreateMcpServerRequest,
    ) -> RepositoryResult<i64> {
        // 将args和env序列化为JSON字符串
        let args_json = serde_json::to_string(&request.args)?;
        let env_json = request.env.as_ref().map(serde_json::to_string).transpose()?;

        let query = r#"
            INSERT INTO mcp_servers (
                name, type, timeout, command, args, env, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))
        "#;

        tracing::info!(
            name = %request.name,
            server_type = ?request.r#type,
            command = %request.command,
            "创建MCP服务器"
        );

        let result = sqlx::query(query)
            .bind(&request.name)
            .bind(&request.r#type)
            .bind(request.timeout)
            .bind(&request.command)
            .bind(args_json)
            .bind(env_json)
            .execute(&self.pool)
            .await?;

        Ok(result.last_insert_rowid())
    }

    /// 更新MCP服务器记录
    pub async fn update_mcp_server(
        &self,
        id: i64,
        request: &UpdateMcpServerRequest,
    ) -> RepositoryResult<bool> {
        // 获取现有记录
        let existing = self.find_by_id::<McpServer>(id).await?;
        if existing.is_none() {
            return Err(RepositoryError::NotFound(format!(
                "MCP服务器 ID {} 不存在",
                id
            )));
        }

        // 序列化更新的数据
        let args_json = if let Some(ref args) = request.args {
            Some(serde_json::to_string(args)?)
        } else {
            None
        };

        let env_json = if let Some(ref env_option) = request.env {
            env_option.as_ref().map(serde_json::to_string).transpose()?
        } else {
            None
        };

        let query = r#"
            UPDATE mcp_servers SET
                name = COALESCE(?, name),
                type = COALESCE(?, type),
                timeout = COALESCE(?, timeout),
                command = COALESCE(?, command),
                args = COALESCE(?, args),
                env = COALESCE(?, env),
                updated_at = datetime('now')
            WHERE id = ?
        "#;

        tracing::info!(
            id = %id,
            "更新MCP服务器"
        );

        let result = sqlx::query(query)
            .bind(&request.name)
            .bind(&request.r#type)
            .bind(request.timeout)
            .bind(&request.command)
            .bind(args_json)
            .bind(env_json)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 根据ID获取MCP服务器（解析JSON字段）
    pub async fn find_by_id_parsed(&self, id: i64) -> RepositoryResult<Option<McpServer>> {
        if let Some(server) = self.find_by_id::<McpServer>(id).await? {
            // 解析args字段
            if let Ok(_args_vec) = serde_json::from_str::<Vec<String>>(&server.args) {
                // 注意：这里保持原样，因为McpServer结构体中args是String
                // 如果需要的话，可以在这里添加转换逻辑
            }

            Ok(Some(server))
        } else {
            Ok(None)
        }
    }

    /// 搜索MCP服务器
    pub async fn search_mcp_servers(
        &self,
        search_term: &str,
        limit: Option<i64>,
    ) -> RepositoryResult<Vec<McpServer>> {
        let search_fields = vec!["name", "type", "command"];
        self.search::<McpServer>(search_term, &search_fields, limit).await
    }

    /// 根据类型获取MCP服务器
    pub async fn find_by_type(&self, server_type: &str) -> RepositoryResult<Vec<McpServer>> {
        let query = "SELECT * FROM mcp_servers WHERE type = ? ORDER BY id DESC";

        tracing::debug!(
            server_type = %server_type,
            "根据类型获取MCP服务器列表"
        );

        let results = sqlx::query_as::<_, McpServer>(query)
            .bind(server_type)
            .fetch_all(&self.pool)
            .await?;

        Ok(results)
    }

    /// 测试MCP服务器配置
    pub async fn test_server_config(&self, id: i64) -> RepositoryResult<bool> {
        let server = self.find_by_id_parsed(id).await?;
        if server.is_none() {
            return Err(RepositoryError::NotFound(format!(
                "MCP服务器 ID {} 不存在",
                id
            )));
        }

        let server = server.unwrap();

        tracing::info!(
            server_name = %server.name,
            "测试MCP服务器配置"
        );

        // 基本配置验证
        let is_valid = !server.command.trim().is_empty() && server.timeout.unwrap_or(30000) > 0;

        // 可以在这里添加更复杂的验证逻辑，比如：
        // - 检查命令是否存在
        // - 验证环境变量格式
        // - 测试服务器启动

        Ok(is_valid)
    }

    /// 统计MCP服务器数量
    pub async fn count_by_type(&self, server_type: &str) -> RepositoryResult<i64> {
        let query = "SELECT COUNT(*) FROM mcp_servers WHERE type = ?";

        let count: i64 = sqlx::query_scalar(query).bind(server_type).fetch_one(&self.pool).await?;

        Ok(count)
    }

    /// 获取活跃的MCP服务器（根据timeout判断）
    pub async fn list_active_servers(&self) -> RepositoryResult<Vec<McpServer>> {
        let query = "SELECT * FROM mcp_servers WHERE timeout > 0 ORDER BY id DESC";

        tracing::debug!("获取活跃的MCP服务器列表");

        let results = sqlx::query_as::<_, McpServer>(query).fetch_all(&self.pool).await?;

        Ok(results)
    }
}

impl BaseRepository for McpServerRepository {
    fn table_name() -> &'static str {
        "mcp_servers"
    }

    fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    fn crypto_service(&self) -> &CryptoService {
        &self.crypto_service
    }

    async fn find_by_id<T>(&self, id: i64) -> RepositoryResult<Option<T>>
    where
        T: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
    {
        let query = format!("SELECT * FROM {} WHERE id = ?", Self::table_name());

        tracing::debug!(
            table_name = %Self::table_name(),
            id = %id,
            "执行查询: {}",
            query
        );

        let result = sqlx::query_as::<_, T>(&query).bind(id).fetch_optional(self.pool()).await?;

        Ok(result)
    }

    async fn create<T>(&self, _data: &T) -> RepositoryResult<i64>
    where
        T: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
    {
        // 对于MCP服务器，使用专用的创建方法
        Err(RepositoryError::Validation(
            "请使用 create_mcp_server 方法".to_string(),
        ))
    }

    async fn update<T>(&self, _id: i64, _data: &T) -> RepositoryResult<bool>
    where
        T: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
    {
        // 对于MCP服务器，使用专用的更新方法
        Err(RepositoryError::Validation(
            "请使用 update_mcp_server 方法".to_string(),
        ))
    }

    async fn delete(&self, id: i64) -> RepositoryResult<bool> {
        let query = "DELETE FROM mcp_servers WHERE id = ?";

        tracing::info!(
            id = %id,
            "删除MCP服务器"
        );

        let result = sqlx::query(query).bind(id).execute(self.pool()).await?;

        Ok(result.rows_affected() > 0)
    }

    async fn list_all<T>(&self) -> RepositoryResult<Vec<T>>
    where
        T: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
    {
        let query = "SELECT * FROM mcp_servers ORDER BY id DESC";

        tracing::debug!("获取MCP服务器列表");

        let results = sqlx::query_as::<_, T>(query).fetch_all(self.pool()).await?;

        Ok(results)
    }

    async fn paginate<T>(
        &self,
        params: &crate::models::PaginationParams,
    ) -> RepositoryResult<crate::models::PagedResult<T>>
    where
        T: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
    {
        let page = params.page.unwrap_or(1);
        let limit = params.limit.unwrap_or(20);
        let offset = params.offset.unwrap_or((page - 1) * limit);

        // 查询总数
        let count_query = "SELECT COUNT(*) FROM mcp_servers";
        let total: i64 = sqlx::query_scalar(count_query).fetch_one(self.pool()).await?;

        // 查询分页数据
        let data_query = "SELECT * FROM mcp_servers ORDER BY id DESC LIMIT ? OFFSET ?";

        tracing::debug!(
            page = %page,
            limit = %limit,
            offset = %offset,
            "分页查询MCP服务器"
        );

        let data = sqlx::query_as::<_, T>(data_query)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool())
            .await?;

        let paged_result = crate::models::PagedResult::new(data, total, page, limit);

        Ok(paged_result)
    }

    async fn search<T>(
        &self,
        search_term: &str,
        search_fields: &[&str],
        limit: Option<i64>,
    ) -> RepositoryResult<Vec<T>>
    where
        T: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
    {
        let limit = limit.unwrap_or(50);

        // 构建搜索查询
        let mut where_conditions = Vec::new();
        let mut search_keywords: Vec<String> = Vec::new();

        // 分割搜索词
        for word in search_term.split_whitespace() {
            if word.is_empty() {
                continue;
            }
            search_keywords.push(format!("%{}%", word));
        }

        // 为每个搜索字段构建条件
        for field in search_fields {
            for _keyword in &search_keywords {
                where_conditions.push(format!("{} LIKE ?", field));
            }
        }

        let query = if where_conditions.is_empty() {
            "SELECT * FROM mcp_servers ORDER BY id DESC LIMIT ?".to_string()
        } else {
            format!(
                "SELECT * FROM mcp_servers WHERE {} ORDER BY id DESC LIMIT ?",
                where_conditions.join(" OR ")
            )
        };

        tracing::debug!(
            search_term = %search_term,
            query = %query,
            "搜索MCP服务器"
        );

        let mut query_builder = sqlx::query_as::<_, T>(&query);

        // 绑定搜索参数
        for _condition in &where_conditions {
            query_builder = query_builder.bind(search_term);
        }
        query_builder = query_builder.bind(limit);

        let results = query_builder.fetch_all(self.pool()).await?;

        Ok(results)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let query = "SELECT COUNT(*) FROM mcp_servers";

        tracing::debug!("统计MCP服务器总数");

        let count: i64 = sqlx::query_scalar(query).fetch_one(self.pool()).await?;

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::DatabaseConfig;
    use std::collections::HashMap;
    use tempfile::tempdir;

    async fn create_test_repository() -> McpServerRepository {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_mcp_server.db");
        let db_url = format!("sqlite:{}", db_path.display());

        let config = DatabaseConfig {
            url: db_url,
            max_connections: 5,
            min_connections: 1,
            connect_timeout: std::time::Duration::from_secs(10),
            idle_timeout: std::time::Duration::from_secs(60),
            max_lifetime: std::time::Duration::from_secs(300),
        };

        let db_manager = DatabaseManager::new(config).await.unwrap();
        let crypto_service = crate::crypto::CryptoService::new("test_key_for_mcp_server").unwrap();

        McpServerRepository::new(&db_manager, &crypto_service)
    }

    #[tokio::test]
    async fn test_mcp_server_crud() {
        let repo = create_test_repository().await;

        // 测试创建
        let mut env = HashMap::new();
        env.insert("NODE_ENV".to_string(), "production".to_string());

        let create_request = CreateMcpServerRequest {
            name: "测试MCP服务器".to_string(),
            r#type: Some("stdio".to_string()),
            timeout: Some(30000),
            command: "npx".to_string(),
            args: vec!["@modelcontextprotocol/server".to_string()],
            env: Some(env),
        };

        let id = repo.create_mcp_server(&create_request).await.unwrap();
        assert!(id > 0);

        // 测试查找
        let server = repo.find_by_id_parsed(id).await.unwrap();
        assert!(server.is_some());
        let server = server.unwrap();
        assert_eq!(server.name, "测试MCP服务器");

        // 测试更新
        let update_request = UpdateMcpServerRequest {
            name: Some("更新后的MCP服务器".to_string()),
            r#type: Some("sse".to_string()),
            timeout: Some(60000),
            command: None,
            args: Some(vec!["--port".to_string(), "8080".to_string()]),
            env: None,
        };

        let updated = repo.update_mcp_server(id, &update_request).await.unwrap();
        assert!(updated);

        // 验证更新
        let updated_server = repo.find_by_id_parsed(id).await.unwrap();
        assert!(updated_server.is_some());
        let updated_server = updated_server.unwrap();
        assert_eq!(updated_server.name, "更新后的MCP服务器");

        // 测试删除
        let deleted = repo.delete(id).await.unwrap();
        assert!(deleted);

        // 验证删除
        let deleted_server = repo.find_by_id_parsed(id).await.unwrap();
        assert!(deleted_server.is_none());
    }

    #[tokio::test]
    async fn test_server_config_validation() {
        let repo = create_test_repository().await;

        // 创建有效服务器
        let create_request = CreateMcpServerRequest {
            name: "有效服务器".to_string(),
            r#type: Some("stdio".to_string()),
            timeout: Some(30000),
            command: "python".to_string(),
            args: vec!["server.py".to_string()],
            env: None,
        };

        let id = repo.create_mcp_server(&create_request).await.unwrap();
        let is_valid = repo.test_server_config(id).await.unwrap();
        assert!(is_valid);

        // 创建无效服务器（空命令）
        let create_request_invalid = CreateMcpServerRequest {
            name: "无效服务器".to_string(),
            r#type: Some("stdio".to_string()),
            timeout: Some(30000),
            command: "".to_string(),
            args: vec![],
            env: None,
        };

        let id_invalid = repo.create_mcp_server(&create_request_invalid).await.unwrap();
        let is_valid_invalid = repo.test_server_config(id_invalid).await.unwrap();
        assert!(!is_valid_invalid);
    }
}
