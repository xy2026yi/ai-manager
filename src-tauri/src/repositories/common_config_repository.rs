// 通用配置Repository实现
//
// 提供通用配置的特定数据访问操作

use sqlx::{FromRow, SqlitePool};
use crate::repositories::base_repository::{BaseRepository, RepositoryResult, RepositoryError};
use crate::crypto::CryptoService;
use crate::database::DatabaseManager;
use crate::models::{CommonConfig, CreateCommonConfigRequest, UpdateCommonConfigRequest};

/// 通用配置Repository
pub struct CommonConfigRepository {
    pool: SqlitePool,
    crypto_service: CryptoService,
}

impl CommonConfigRepository {
    /// 创建新的通用配置Repository实例
    pub fn new(db_manager: &DatabaseManager, crypto_service: &CryptoService) -> Self {
        Self {
            pool: db_manager.pool().clone(),
            crypto_service: crypto_service.clone(),
        }
    }

    /// 创建通用配置记录
    pub async fn create_common_config(&self, request: &CreateCommonConfigRequest) -> RepositoryResult<i64> {
        let query = r#"
            INSERT INTO common_configs (
                key, value, description, category, is_active, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, datetime('now'), datetime('now'))
        "#;

        tracing::info!(
            key = %request.key,
            category = ?request.category,
            "创建通用配置"
        );

        let result = sqlx::query(query)
            .bind(&request.key)
            .bind(&request.value)
            .bind(&request.description)
            .bind(request.category.as_deref().unwrap_or("default"))
            .bind(request.is_active.unwrap_or(1))
            .execute(&self.pool)
            .await?;

        Ok(result.last_insert_rowid())
    }

    /// 更新通用配置记录
    pub async fn update_common_config(&self, id: i64, request: &UpdateCommonConfigRequest) -> RepositoryResult<bool> {
        // 获取现有记录
        let existing = self.find_by_id::<CommonConfig>(id).await?;
        if existing.is_none() {
            return Err(RepositoryError::NotFound(format!("通用配置 ID {} 不存在", id)));
        }

        let query = r#"
            UPDATE common_configs SET
                key = COALESCE(?, key),
                value = COALESCE(?, value),
                description = COALESCE(?, description),
                category = COALESCE(?, category),
                is_active = COALESCE(?, is_active),
                updated_at = datetime('now')
            WHERE id = ?
        "#;

        tracing::info!(
            id = %id,
            "更新通用配置"
        );

        let result = sqlx::query(query)
            .bind(&request.key)
            .bind(&request.value)
            .bind(&request.description)
            .bind(&request.category)
            .bind(request.is_active)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 根据ID获取通用配置
    pub async fn find_by_id_decrypted(&self, id: i64) -> RepositoryResult<Option<CommonConfig>> {
        self.find_by_id::<CommonConfig>(id).await
    }

    /// 根据key获取配置
    pub async fn find_by_key(&self, key: &str) -> RepositoryResult<Option<CommonConfig>> {
        let query = "SELECT * FROM common_configs WHERE key = ?";

        tracing::debug!(
            key = %key,
            "根据key获取配置"
        );

        let result = sqlx::query_as::<_, CommonConfig>(query)
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;

        Ok(result)
    }

    /// 根据类别获取配置列表
    pub async fn find_by_category(&self, category: &str) -> RepositoryResult<Vec<CommonConfig>> {
        let query = "SELECT * FROM common_configs WHERE category = ? ORDER BY key ASC";

        tracing::debug!(
            category = %category,
            "根据类别获取配置列表"
        );

        let results = sqlx::query_as::<_, CommonConfig>(query)
            .bind(category)
            .fetch_all(&self.pool)
            .await?;

        Ok(results)
    }

    /// 获取活跃配置
    pub async fn list_active_configs(&self) -> RepositoryResult<Vec<CommonConfig>> {
        let query = "SELECT * FROM common_configs WHERE is_active = 1 ORDER BY category ASC, key ASC";

        tracing::debug!("获取活跃配置列表");

        let results = sqlx::query_as::<_, CommonConfig>(query)
            .fetch_all(&self.pool)
            .await?;

        Ok(results)
    }

    /// 搜索通用配置
    pub async fn search_common_configs(&self, search_term: &str, limit: Option<i64>) -> RepositoryResult<Vec<CommonConfig>> {
        let search_fields = vec!["key", "value", "description", "category"];
        self.search::<CommonConfig>(search_term, &search_fields, limit).await
    }

    /// 根据key更新配置值（便捷方法）
    pub async fn update_config_value(&self, key: &str, value: &str) -> RepositoryResult<bool> {
        let query = "UPDATE common_configs SET value = ?, updated_at = datetime('now') WHERE key = ?";

        tracing::info!(
            key = %key,
            "根据key更新配置值"
        );

        let result = sqlx::query(query)
            .bind(value)
            .bind(key)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 批量更新配置
    pub async fn batch_update_configs(&self, configs: &[(String, String)]) -> RepositoryResult<usize> {
        let mut updated_count = 0;

        for (key, value) in configs {
            if self.update_config_value(key, value).await? {
                updated_count += 1;
            }
        }

        tracing::info!(
            updated_count = %updated_count,
            total_configs = %configs.len(),
            "批量更新配置完成"
        );

        Ok(updated_count)
    }

    /// 验证配置值
    pub async fn validate_config_value(&self, id: i64) -> RepositoryResult<bool> {
        let config = self.find_by_id_decrypted(id).await?;
        if config.is_none() {
            return Err(RepositoryError::NotFound(format!("通用配置 ID {} 不存在", id)));
        }

        let config = config.unwrap();

        tracing::info!(
            config_key = %config.key,
            "验证通用配置值"
        );

        // 基本验证：检查是否为空
        let is_valid = !config.value.trim().is_empty();

        // 可以在这里添加更复杂的验证逻辑，比如：
        // - 验证环境变量格式
        // - 检查数值范围
        // - 验证URL格式
        // - 检查文件路径是否存在

        Ok(is_valid)
    }

    /// 统计通用配置数量
    pub async fn count_by_category(&self, category: &str) -> RepositoryResult<i64> {
        let query = "SELECT COUNT(*) FROM common_configs WHERE category = ?";

        let count: i64 = sqlx::query_scalar(query)
            .bind(category)
            .fetch_one(&self.pool)
            .await?;

        Ok(count)
    }

    /// 统计活跃配置数量
    pub async fn count_active(&self) -> RepositoryResult<i64> {
        let query = "SELECT COUNT(*) FROM common_configs WHERE is_active = 1";

        let count: i64 = sqlx::query_scalar(query)
            .fetch_one(&self.pool)
            .await?;

        Ok(count)
    }

    /// 获取所有配置类别
    pub async fn get_all_categories(&self) -> RepositoryResult<Vec<String>> {
        let query = "SELECT DISTINCT category FROM common_configs ORDER BY category ASC";

        tracing::debug!("获取所有配置类别");

        let results = sqlx::query_scalar(query)
            .fetch_all(&self.pool)
            .await?;

        Ok(results)
    }
}

impl BaseRepository for CommonConfigRepository {
    fn table_name() -> &'static str {
        "common_configs"
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
        // 对于通用配置，使用专用的创建方法
        Err(RepositoryError::Validation(
            "请使用 create_common_config 方法".to_string()
        ))
    }

    async fn update<T>(&self, _id: i64, _data: &T) -> RepositoryResult<bool>
    where
        T: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
    {
        // 对于通用配置，使用专用的更新方法
        Err(RepositoryError::Validation(
            "请使用 update_common_config 方法".to_string()
        ))
    }

    async fn delete(&self, id: i64) -> RepositoryResult<bool> {
        let query = "DELETE FROM common_configs WHERE id = ?";

        tracing::info!(
            id = %id,
            "删除通用配置"
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
        let query = "SELECT * FROM common_configs ORDER BY category ASC, key ASC";

        tracing::debug!("获取通用配置列表");

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
        let count_query = "SELECT COUNT(*) FROM common_configs";
        let total: i64 = sqlx::query_scalar(count_query)
            .fetch_one(self.pool())
            .await?;

        // 查询分页数据
        let data_query = "SELECT * FROM common_configs ORDER BY category ASC, key ASC LIMIT ? OFFSET ?";

        tracing::debug!(
            page = %page,
            limit = %limit,
            offset = %offset,
            "分页查询通用配置"
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
            "SELECT * FROM common_configs ORDER BY category ASC, key ASC LIMIT ?".to_string()
        } else {
            format!(
                "SELECT * FROM common_configs WHERE {} ORDER BY category ASC, key ASC LIMIT ?",
                where_conditions.join(" OR ")
            )
        };

        tracing::debug!(
            search_term = %search_term,
            query = %query,
            "搜索通用配置"
        );

        let mut query_builder = sqlx::query_as::<_, T>(&query);

        // 绑定搜索参数
        for _condition in &where_conditions {
            query_builder = query_builder.bind(search_term);
        }
        query_builder = query_builder.bind(limit);

        let results = query_builder
            .fetch_all(self.pool())
            .await?;

        Ok(results)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let query = "SELECT COUNT(*) FROM common_configs";

        tracing::debug!("统计通用配置总数");

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

    async fn create_test_repository() -> CommonConfigRepository {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_common_config.db");
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
        let crypto_service = crate::crypto::CryptoService::new("test_key_for_common_config").unwrap();

        CommonConfigRepository::new(&db_manager, &crypto_service)
    }

    #[tokio::test]
    async fn test_common_config_crud() {
        let repo = create_test_repository().await;

        // 测试创建
        let create_request = CreateCommonConfigRequest {
            key: "test.config".to_string(),
            value: "test_value".to_string(),
            description: Some("测试配置".to_string()),
            category: Some("test".to_string()),
            is_active: Some(1),
        };

        let id = repo.create_common_config(&create_request).await.unwrap();
        assert!(id > 0);

        // 测试查找
        let config = repo.find_by_id_decrypted(id).await.unwrap();
        assert!(config.is_some());
        let config = config.unwrap();
        assert_eq!(config.key, "test.config");
        assert_eq!(config.value, "test_value");

        // 测试根据key查找
        let config_by_key = repo.find_by_key("test.config").await.unwrap();
        assert!(config_by_key.is_some());
        assert_eq!(config_by_key.unwrap().id, id);

        // 测试更新
        let update_request = UpdateCommonConfigRequest {
            key: None,
            value: Some("updated_value".to_string()),
            description: Some("更新后的配置".to_string()),
            category: None,
            is_active: Some(0),
        };

        let updated = repo.update_common_config(id, &update_request).await.unwrap();
        assert!(updated);

        // 验证更新
        let updated_config = repo.find_by_id_decrypted(id).await.unwrap();
        assert!(updated_config.is_some());
        let updated_config = updated_config.unwrap();
        assert_eq!(updated_config.value, "updated_value");
        assert_eq!(updated_config.is_active, 0);

        // 测试删除
        let deleted = repo.delete(id).await.unwrap();
        assert!(deleted);

        // 验证删除
        let deleted_config = repo.find_by_id_decrypted(id).await.unwrap();
        assert!(deleted_config.is_none());
    }

    #[tokio::test]
    async fn test_batch_update() {
        let repo = create_test_repository().await;

        // 创建多个配置
        let configs = vec![
            ("config1".to_string(), "value1".to_string()),
            ("config2".to_string(), "value2".to_string()),
            ("config3".to_string(), "value3".to_string()),
        ];

        for (key, value) in &configs {
            let create_request = CreateCommonConfigRequest {
                key: key.clone(),
                value: value.clone(),
                description: None,
                category: Some("batch_test".to_string()),
                is_active: Some(1),
            };
            repo.create_common_config(&create_request).await.unwrap();
        }

        // 批量更新
        let updates = vec![
            ("config1".to_string(), "new_value1".to_string()),
            ("config2".to_string(), "new_value2".to_string()),
        ];

        let updated_count = repo.batch_update_configs(&updates).await.unwrap();
        assert_eq!(updated_count, 2);

        // 验证更新
        let config1 = repo.find_by_key("config1").await.unwrap().unwrap();
        assert_eq!(config1.value, "new_value1");

        let config3 = repo.find_by_key("config3").await.unwrap().unwrap();
        assert_eq!(config3.value, "value3"); // 未更新
    }

    #[tokio::test]
    async fn test_config_validation() {
        let repo = create_test_repository().await;

        // 创建有效配置
        let create_request = CreateCommonConfigRequest {
            key: "valid.config".to_string(),
            value: "valid_value".to_string(),
            description: None,
            category: Some("validation_test".to_string()),
            is_active: Some(1),
        };

        let id = repo.create_common_config(&create_request).await.unwrap();
        let is_valid = repo.validate_config_value(id).await.unwrap();
        assert!(is_valid);

        // 创建无效配置（空值）
        let create_request_empty = CreateCommonConfigRequest {
            key: "invalid.config".to_string(),
            value: "".to_string(),
            description: None,
            category: Some("validation_test".to_string()),
            is_active: Some(1),
        };

        let id_empty = repo.create_common_config(&create_request_empty).await.unwrap();
        let is_valid_empty = repo.validate_config_value(id_empty).await.unwrap();
        assert!(!is_valid_empty);
    }
}