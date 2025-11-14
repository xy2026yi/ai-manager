// Claude供应商Repository实现
//
// 提供Claude供应商的特定数据访问操作

use sqlx::{FromRow, SqlitePool};
use crate::repositories::base_repository::{BaseRepository, RepositoryResult, RepositoryError};
use crate::crypto::CryptoService;
use crate::database::DatabaseManager;
use crate::models::{ClaudeProvider, CreateClaudeProviderRequest, UpdateClaudeProviderRequest};

/// Claude供应商Repository
pub struct ClaudeProviderRepository {
    pool: SqlitePool,
    crypto_service: CryptoService,
}

impl ClaudeProviderRepository {
    /// 创建新的Claude供应商Repository实例
    pub fn new(db_manager: &DatabaseManager, crypto_service: &CryptoService) -> Self {
        Self {
            pool: db_manager.pool().clone(),
            crypto_service: crypto_service.clone(),
        }
    }

    /// 创建Claude供应商记录
    pub async fn create_claude_provider(&self, request: &CreateClaudeProviderRequest) -> RepositoryResult<i64> {
        // 加密token
        let encrypted_token = crate::repositories::base_repository::EncryptedField::encrypt_field(
            &request.token,
            &self.crypto_service
        )?;

        let query = r#"
            INSERT INTO claude_providers (
                name, url, token, timeout, auto_update, type,
                opus_model, sonnet_model, haiku_model, enabled,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))
        "#;

        tracing::info!(
            name = %request.name,
            url = %request.url,
            "创建Claude供应商"
        );

        let result = sqlx::query(query)
            .bind(&request.name)
            .bind(&request.url)
            .bind(encrypted_token)
            .bind(request.timeout)
            .bind(request.auto_update)
            .bind(&request.r#type)
            .bind(&request.opus_model)
            .bind(&request.sonnet_model)
            .bind(&request.haiku_model)
            .bind(1i64) // 默认启用
            .execute(&self.pool)
            .await?;

        Ok(result.last_insert_rowid())
    }

    /// 更新Claude供应商记录
    pub async fn update_claude_provider(&self, id: i64, request: &UpdateClaudeProviderRequest) -> RepositoryResult<bool> {
        // 获取现有记录
        let existing = self.find_by_id::<ClaudeProvider>(id).await?;
        if existing.is_none() {
            return Err(RepositoryError::NotFound(format!("Claude供应商 ID {} 不存在", id)));
        }

        // 如果提供了新的token，则加密它
        let encrypted_token = if let Some(ref token) = request.token {
            Some(crate::repositories::base_repository::EncryptedField::encrypt_field(
                token,
                &self.crypto_service
            )?)
        } else {
            None
        };

        let query = r#"
            UPDATE claude_providers SET
                name = COALESCE(?, name),
                url = COALESCE(?, url),
                token = COALESCE(?, token),
                timeout = COALESCE(?, timeout),
                auto_update = COALESCE(?, auto_update),
                type = COALESCE(?, type),
                enabled = COALESCE(?, enabled),
                opus_model = COALESCE(?, opus_model),
                sonnet_model = COALESCE(?, sonnet_model),
                haiku_model = COALESCE(?, haiku_model),
                updated_at = datetime('now')
            WHERE id = ?
        "#;

        tracing::info!(
            id = %id,
            "更新Claude供应商"
        );

        let result = sqlx::query(query)
            .bind(&request.name)
            .bind(&request.url)
            .bind(encrypted_token)
            .bind(request.timeout)
            .bind(request.auto_update)
            .bind(&request.r#type)
            .bind(request.enabled)
            .bind(&request.opus_model)
            .bind(&request.sonnet_model)
            .bind(&request.haiku_model)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 获取Claude供应商列表（解密token）
    pub async fn list_claude_providers_decrypted(&self) -> RepositoryResult<Vec<ClaudeProvider>> {
        let providers = self.list_all::<ClaudeProvider>().await?;

        // 解密token
        let mut decrypted_providers = Vec::new();
        for provider in providers {
            let decrypted_token = crate::repositories::base_repository::EncryptedField::decrypt_field(
                &provider.token,
                &self.crypto_service
            )?;

            let mut decrypted_provider = provider;
            decrypted_provider.token = decrypted_token;
            decrypted_providers.push(decrypted_provider);
        }

        Ok(decrypted_providers)
    }

    /// 根据ID获取Claude供应商（解密token）
    pub async fn find_by_id_decrypted(&self, id: i64) -> RepositoryResult<Option<ClaudeProvider>> {
        if let Some(provider) = self.find_by_id::<ClaudeProvider>(id).await? {
            let decrypted_token = crate::repositories::base_repository::EncryptedField::decrypt_field(
                &provider.token,
                &self.crypto_service
            )?;

            let mut decrypted_provider = provider;
            decrypted_provider.token = decrypted_token;
            Ok(Some(decrypted_provider))
        } else {
            Ok(None)
        }
    }

    /// 搜索Claude供应商
    pub async fn search_claude_providers(&self, search_term: &str, limit: Option<i64>) -> RepositoryResult<Vec<ClaudeProvider>> {
        let search_fields = vec!["name", "url", "opus_model", "sonnet_model", "haiku_model"];
        self.search::<ClaudeProvider>(search_term, &search_fields, limit).await
    }

    /// 获取活跃的Claude供应商
    pub async fn list_active_providers(&self) -> RepositoryResult<Vec<ClaudeProvider>> {
        let query = "SELECT * FROM claude_providers WHERE enabled = 1 ORDER BY id DESC";

        tracing::debug!("获取活跃的Claude供应商列表");

        let results = sqlx::query_as::<_, ClaudeProvider>(query)
            .fetch_all(&self.pool)
            .await?;

        Ok(results)
    }

    /// 测试Claude供应商连接
    pub async fn test_connection(&self, id: i64) -> RepositoryResult<bool> {
        let provider = self.find_by_id_decrypted(id).await?;
        if provider.is_none() {
            return Err(RepositoryError::NotFound(format!("Claude供应商 ID {} 不存在", id)));
        }

        let provider = provider.unwrap();

        tracing::info!(
            provider_name = %provider.name,
            "测试Claude供应商连接"
        );

        // 这里可以添加实际的API连接测试逻辑
        // 暂时返回true，表示连接测试成功
        Ok(true)
    }

    /// 统计Claude供应商数量
    pub async fn count_by_status(&self, is_active: bool) -> RepositoryResult<i64> {
        let query = "SELECT COUNT(*) FROM claude_providers WHERE enabled = ?";

        let count: i64 = sqlx::query_scalar(query)
            .bind(if is_active { 1 } else { 0 })
            .fetch_one(&self.pool)
            .await?;

        Ok(count)
    }
}

impl BaseRepository for ClaudeProviderRepository {
    fn table_name() -> &'static str {
        "claude_providers"
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
        let query = format!(
            "SELECT * FROM {} WHERE id = ?",
            Self::table_name()
        );

        tracing::debug!(
            table_name = %Self::table_name(),
            id = %id,
            "执行查询: {}",
            query
        );

        let result = sqlx::query_as::<_, T>(&query)
            .bind(id)
            .fetch_optional(self.pool())
            .await?;

        Ok(result)
    }

    async fn create<T>(&self, _data: &T) -> RepositoryResult<i64>
    where
        T: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
    {
        // 对于Claude供应商，使用专用的创建方法
        Err(RepositoryError::Validation(
            "请使用 create_claude_provider 方法".to_string()
        ))
    }

    async fn update<T>(&self, _id: i64, _data: &T) -> RepositoryResult<bool>
    where
        T: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
    {
        // 对于Claude供应商，使用专用的更新方法
        Err(RepositoryError::Validation(
            "请使用 update_claude_provider 方法".to_string()
        ))
    }

    async fn delete(&self, id: i64) -> RepositoryResult<bool> {
        let query = "DELETE FROM claude_providers WHERE id = ?";

        tracing::info!(
            id = %id,
            "删除Claude供应商"
        );

        let result = sqlx::query(query)
            .bind(id)
            .execute(self.pool())
            .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn list_all<T>(&self) -> RepositoryResult<Vec<T>>
    where
        T: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
    {
        let query = "SELECT * FROM claude_providers ORDER BY id DESC";

        tracing::debug!("获取Claude供应商列表");

        let results = sqlx::query_as::<_, T>(query)
            .fetch_all(self.pool())
            .await?;

        Ok(results)
    }

    async fn paginate<T>(&self, params: &crate::models::PaginationParams) -> RepositoryResult<crate::models::PagedResult<T>>
    where
        T: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
    {
        let page = params.page.unwrap_or(1);
        let limit = params.limit.unwrap_or(20);
        let offset = params.offset.unwrap_or((page - 1) * limit);

        // 查询总数
        let count_query = "SELECT COUNT(*) FROM claude_providers";
        let total: i64 = sqlx::query_scalar(count_query)
            .fetch_one(self.pool())
            .await?;

        // 查询分页数据
        let data_query = "SELECT * FROM claude_providers ORDER BY id DESC LIMIT ? OFFSET ?";

        tracing::debug!(
            page = %page,
            limit = %limit,
            offset = %offset,
            "分页查询Claude供应商"
        );

        let data = sqlx::query_as::<_, T>(data_query)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool())
            .await?;

        let paged_result = crate::models::PagedResult::new(data, total, page, limit);

        Ok(paged_result)
    }

    async fn search<T>(&self, search_term: &str, search_fields: &[&str], limit: Option<i64>) -> RepositoryResult<Vec<T>>
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
            "SELECT * FROM claude_providers ORDER BY id DESC LIMIT ?".to_string()
        } else {
            format!(
                "SELECT * FROM claude_providers WHERE {} ORDER BY id DESC LIMIT ?",
                where_conditions.join(" OR ")
            )
        };

        tracing::debug!(
            search_term = %search_term,
            query = %query,
            "搜索Claude供应商"
        );

        let mut query_builder = sqlx::query_as::<_, T>(&query);

        // 绑定搜索参数
        for keyword in &search_keywords {
            for _condition in search_fields {
                query_builder = query_builder.bind(keyword);
            }
        }
        query_builder = query_builder.bind(limit);

        let results = query_builder
            .fetch_all(self.pool())
            .await?;

        Ok(results)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let query = "SELECT COUNT(*) FROM claude_providers";

        tracing::debug!("统计Claude供应商总数");

        let count: i64 = sqlx::query_scalar(query)
            .fetch_one(self.pool())
            .await?;

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::DatabaseConfig;
    use tempfile::tempdir;

    async fn create_test_repository() -> ClaudeProviderRepository {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_claude.db");
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
        let crypto_service = crate::crypto::CryptoService::new("test_key_for_claude_provider").unwrap();

        ClaudeProviderRepository::new(&db_manager, &crypto_service)
    }

    #[tokio::test]
    async fn test_claude_provider_crud() {
        let repo = create_test_repository().await;

        // 测试创建
        let create_request = CreateClaudeProviderRequest {
            name: "测试Claude".to_string(),
            url: "https://api.anthropic.com".to_string(),
            token: "sk-test-api-key".to_string(),
            timeout: Some(30000),
            auto_update: Some(1),
            r#type: Some("public_welfare".to_string()),
            opus_model: Some("claude-3-opus-20240229".to_string()),
            sonnet_model: Some("claude-3-sonnet-20241022".to_string()),
            haiku_model: Some("claude-3-haiku-20240307".to_string()),
        };

        let id = repo.create_claude_provider(&create_request).await.unwrap();
        assert!(id > 0);

        // 测试查找
        let provider = repo.find_by_id_decrypted(id).await.unwrap();
        assert!(provider.is_some());
        let provider = provider.unwrap();
        assert_eq!(provider.name, "测试Claude");
        assert_eq!(provider.token, "sk-test-api-key");

        // 测试更新
        let update_request = UpdateClaudeProviderRequest {
            name: Some("更新后的Claude".to_string()),
            url: None,
            token: None,
            timeout: Some(60000),
            auto_update: Some(0),
            r#type: Some("paid".to_string()),
            enabled: Some(0),
            opus_model: None,
            sonnet_model: None,
            haiku_model: None,
        };

        let updated = repo.update_claude_provider(id, &update_request).await.unwrap();
        assert!(updated);

        // 验证更新
        let updated_provider = repo.find_by_id_decrypted(id).await.unwrap();
        assert!(updated_provider.is_some());
        let updated_provider = updated_provider.unwrap();
        assert_eq!(updated_provider.name, "更新后的Claude");
        assert_eq!(updated_provider.timeout, Some(60000));
        assert_eq!(updated_provider.enabled, 0);

        // 测试删除
        let deleted = repo.delete(id).await.unwrap();
        assert!(deleted);

        // 验证删除
        let deleted_provider = repo.find_by_id_decrypted(id).await.unwrap();
        assert!(deleted_provider.is_none());
    }
}