// Agent指导文件Repository实现
//
// 提供Agent指导文件的特定数据访问操作

use crate::crypto::CryptoService;
use crate::database::DatabaseManager;
use crate::models::{AgentGuide, CreateAgentGuideRequest, UpdateAgentGuideRequest};
use crate::repositories::base_repository::{BaseRepository, RepositoryError, RepositoryResult};
use sqlx::{FromRow, SqlitePool};

/// Agent指导文件Repository
pub struct AgentGuideRepository {
    pool: SqlitePool,
    crypto_service: CryptoService,
}

impl AgentGuideRepository {
    /// 创建新的Agent指导文件Repository实例
    pub fn new(db_manager: &DatabaseManager, crypto_service: &CryptoService) -> Self {
        Self {
            pool: db_manager.pool().clone(),
            crypto_service: crypto_service.clone(),
        }
    }

    /// 创建Agent指导文件记录
    pub async fn create_agent_guide(
        &self,
        request: &CreateAgentGuideRequest,
    ) -> RepositoryResult<i64> {
        let query = r#"
            INSERT INTO agent_guides (
                name, type, text, created_at, updated_at
            ) VALUES (?, ?, ?, datetime('now'), datetime('now'))
        "#;

        tracing::info!(
            name = %request.name,
            guide_type = %request.r#type,
            "创建Agent指导文件"
        );

        let result = sqlx::query(query)
            .bind(&request.name)
            .bind(&request.r#type)
            .bind(&request.text)
            .execute(&self.pool)
            .await?;

        Ok(result.last_insert_rowid())
    }

    /// 更新Agent指导文件记录
    pub async fn update_agent_guide(
        &self,
        id: i64,
        request: &UpdateAgentGuideRequest,
    ) -> RepositoryResult<bool> {
        // 获取现有记录
        let existing = self.find_by_id::<AgentGuide>(id).await?;
        if existing.is_none() {
            return Err(RepositoryError::NotFound(format!(
                "Agent指导文件 ID {} 不存在",
                id
            )));
        }

        let query = r#"
            UPDATE agent_guides SET
                name = COALESCE(?, name),
                type = COALESCE(?, type),
                text = COALESCE(?, text),
                updated_at = datetime('now')
            WHERE id = ?
        "#;

        tracing::info!(
            id = %id,
            "更新Agent指导文件"
        );

        let result = sqlx::query(query)
            .bind(&request.name)
            .bind(&request.r#type)
            .bind(&request.text)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 根据ID获取Agent指导文件
    pub async fn find_by_id_decrypted(&self, id: i64) -> RepositoryResult<Option<AgentGuide>> {
        self.find_by_id::<AgentGuide>(id).await
    }

    /// 搜索Agent指导文件
    pub async fn search_agent_guides(
        &self,
        search_term: &str,
        limit: Option<i64>,
    ) -> RepositoryResult<Vec<AgentGuide>> {
        let search_fields = vec!["name", "type", "text"];
        self.search::<AgentGuide>(search_term, &search_fields, limit).await
    }

    /// 根据类型获取Agent指导文件
    pub async fn find_by_type(&self, guide_type: &str) -> RepositoryResult<Vec<AgentGuide>> {
        let query = "SELECT * FROM agent_guides WHERE type = ? ORDER BY id DESC";

        tracing::debug!(
            guide_type = %guide_type,
            "根据类型获取Agent指导文件列表"
        );

        let results = sqlx::query_as::<_, AgentGuide>(query)
            .bind(guide_type)
            .fetch_all(&self.pool)
            .await?;

        Ok(results)
    }

    /// 测试Agent指导文件内容有效性
    pub async fn validate_guide_content(&self, id: i64) -> RepositoryResult<bool> {
        let guide = self.find_by_id_decrypted(id).await?;
        if guide.is_none() {
            return Err(RepositoryError::NotFound(format!(
                "Agent指导文件 ID {} 不存在",
                id
            )));
        }

        let guide = guide.unwrap();

        tracing::info!(
            guide_name = %guide.name,
            "验证Agent指导文件内容"
        );

        // 基本内容验证：检查是否为空或仅包含空白字符
        let is_valid = !guide.text.trim().is_empty();

        Ok(is_valid)
    }

    /// 统计Agent指导文件数量
    pub async fn count_by_type(&self, guide_type: &str) -> RepositoryResult<i64> {
        let query = "SELECT COUNT(*) FROM agent_guides WHERE type = ?";

        let count: i64 = sqlx::query_scalar(query).bind(guide_type).fetch_one(&self.pool).await?;

        Ok(count)
    }
}

impl BaseRepository for AgentGuideRepository {
    fn table_name() -> &'static str {
        "agent_guides"
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
        // 对于Agent指导文件，使用专用的创建方法
        Err(RepositoryError::Validation(
            "请使用 create_agent_guide 方法".to_string(),
        ))
    }

    async fn update<T>(&self, _id: i64, _data: &T) -> RepositoryResult<bool>
    where
        T: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
    {
        // 对于Agent指导文件，使用专用的更新方法
        Err(RepositoryError::Validation(
            "请使用 update_agent_guide 方法".to_string(),
        ))
    }

    async fn delete(&self, id: i64) -> RepositoryResult<bool> {
        let query = "DELETE FROM agent_guides WHERE id = ?";

        tracing::info!(
            id = %id,
            "删除Agent指导文件"
        );

        let result = sqlx::query(query).bind(id).execute(self.pool()).await?;

        Ok(result.rows_affected() > 0)
    }

    async fn list_all<T>(&self) -> RepositoryResult<Vec<T>>
    where
        T: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
    {
        let query = "SELECT * FROM agent_guides ORDER BY id DESC";

        tracing::debug!("获取Agent指导文件列表");

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
        let count_query = "SELECT COUNT(*) FROM agent_guides";
        let total: i64 = sqlx::query_scalar(count_query).fetch_one(self.pool()).await?;

        // 查询分页数据
        let data_query = "SELECT * FROM agent_guides ORDER BY id DESC LIMIT ? OFFSET ?";

        tracing::debug!(
            page = %page,
            limit = %limit,
            offset = %offset,
            "分页查询Agent指导文件"
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
            "SELECT * FROM agent_guides ORDER BY id DESC LIMIT ?".to_string()
        } else {
            format!(
                "SELECT * FROM agent_guides WHERE {} ORDER BY id DESC LIMIT ?",
                where_conditions.join(" OR ")
            )
        };

        tracing::debug!(
            search_term = %search_term,
            query = %query,
            "搜索Agent指导文件"
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
        let query = "SELECT COUNT(*) FROM agent_guides";

        tracing::debug!("统计Agent指导文件总数");

        let count: i64 = sqlx::query_scalar(query).fetch_one(self.pool()).await?;

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::DatabaseConfig;
    use tempfile::tempdir;

    async fn create_test_repository() -> AgentGuideRepository {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_agent_guide.db");
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
        let crypto_service = crate::crypto::CryptoService::new("test_key_for_agent_guide").unwrap();

        AgentGuideRepository::new(&db_manager, &crypto_service)
    }

    #[tokio::test]
    async fn test_agent_guide_crud() {
        let repo = create_test_repository().await;

        // 测试创建
        let create_request = CreateAgentGuideRequest {
            name: "测试Agent指导".to_string(),
            r#type: "only".to_string(),
            text: "这是一个测试指导文件的内容".to_string(),
        };

        let id = repo.create_agent_guide(&create_request).await.unwrap();
        assert!(id > 0);

        // 测试查找
        let guide = repo.find_by_id_decrypted(id).await.unwrap();
        assert!(guide.is_some());
        let guide = guide.unwrap();
        assert_eq!(guide.name, "测试Agent指导");
        assert_eq!(guide.r#type, "only");

        // 测试更新
        let update_request = UpdateAgentGuideRequest {
            name: Some("更新后的Agent指导".to_string()),
            r#type: Some("and".to_string()),
            text: Some("这是更新后的指导文件内容".to_string()),
        };

        let updated = repo.update_agent_guide(id, &update_request).await.unwrap();
        assert!(updated);

        // 验证更新
        let updated_guide = repo.find_by_id_decrypted(id).await.unwrap();
        assert!(updated_guide.is_some());
        let updated_guide = updated_guide.unwrap();
        assert_eq!(updated_guide.name, "更新后的Agent指导");
        assert_eq!(updated_guide.r#type, "and");

        // 测试删除
        let deleted = repo.delete(id).await.unwrap();
        assert!(deleted);

        // 验证删除
        let deleted_guide = repo.find_by_id_decrypted(id).await.unwrap();
        assert!(deleted_guide.is_none());
    }

    #[tokio::test]
    async fn test_validate_guide_content() {
        let repo = create_test_repository().await;

        // 创建有效指导文件
        let create_request = CreateAgentGuideRequest {
            name: "有效指导".to_string(),
            r#type: "only".to_string(),
            text: "这是有效的内容".to_string(),
        };

        let id = repo.create_agent_guide(&create_request).await.unwrap();
        let is_valid = repo.validate_guide_content(id).await.unwrap();
        assert!(is_valid);

        // 创建无效指导文件（空内容）
        let create_request_empty = CreateAgentGuideRequest {
            name: "无效指导".to_string(),
            r#type: "and".to_string(),
            text: "   ".to_string(), // 仅空白字符
        };

        let id_empty = repo.create_agent_guide(&create_request_empty).await.unwrap();
        let is_valid_empty = repo.validate_guide_content(id_empty).await.unwrap();
        assert!(!is_valid_empty);
    }
}
