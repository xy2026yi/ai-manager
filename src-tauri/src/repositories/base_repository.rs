// 基础Repository抽象
//
// 提供通用的数据库访问操作，包括CRUD、分页、搜索等功能
// 支持加密数据的透明处理

use sqlx::{FromRow, SqlitePool};
use std::marker::PhantomData;
use thiserror::Error;
use tracing::{debug, error, info};

use crate::crypto::CryptoService;
use crate::database::DatabaseManager;
use crate::models::PagedResult;
use crate::models::PaginationParams;

/// Repository错误类型
#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),

    #[error("加密错误: {0}")]
    Crypto(#[from] crate::crypto::CryptoError),

    #[error("记录不存在: {0}")]
    NotFound(String),

    #[error("数据冲突: {0}")]
    Conflict(String),

    #[error("验证失败: {0}")]
    Validation(String),

    #[error("查询错误: {0}")]
    Query(String),

    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Repository结果类型
pub type RepositoryResult<T> = Result<T, RepositoryError>;

/// 基础Repository trait
#[allow(async_fn_in_trait)]
pub trait BaseRepository {
    /// 获取表名
    fn table_name() -> &'static str;

    /// 获取数据库连接池
    fn pool(&self) -> &SqlitePool;

    /// 获取加密服务
    fn crypto_service(&self) -> &CryptoService;

    /// 根据ID查找记录
    async fn find_by_id<T>(&self, id: i64) -> RepositoryResult<Option<T>>
    where
        T: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
        Self: Sized;

    /// 创建记录
    async fn create<T>(&self, data: &T) -> RepositoryResult<i64>
    where
        T: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
        Self: Sized;

    /// 更新记录
    async fn update<T>(&self, id: i64, data: &T) -> RepositoryResult<bool>
    where
        T: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
        Self: Sized;

    /// 删除记录
    async fn delete(&self, id: i64) -> RepositoryResult<bool>;

    /// 列出所有记录
    async fn list_all<T>(&self) -> RepositoryResult<Vec<T>>
    where
        T: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
        Self: Sized;

    /// 分页查询
    async fn paginate<T>(&self, params: &PaginationParams) -> RepositoryResult<PagedResult<T>>
    where
        T: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
        Self: Sized;

    /// 搜索记录
    async fn search<T>(
        &self,
        search_term: &str,
        search_fields: &[&str],
        limit: Option<i64>,
    ) -> RepositoryResult<Vec<T>>
    where
        T: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
        Self: Sized;

    /// 计算总数
    async fn count(&self) -> RepositoryResult<i64>;
}

/// 通用Repository实现
pub struct GenericRepository<T> {
    pool: SqlitePool,
    crypto_service: CryptoService,
    table_name: String,
    phantom: PhantomData<T>,
}

impl<T> GenericRepository<T> {
    /// 创建新的通用Repository实例
    pub fn new(
        db_manager: &DatabaseManager,
        table_name: &str,
        crypto_service: &CryptoService,
    ) -> Self {
        Self {
            pool: db_manager.pool().clone(),
            crypto_service: crypto_service.clone(),
            table_name: table_name.to_string(),
            phantom: PhantomData,
        }
    }

    /// 获取表名
    pub fn table_name(&self) -> &str {
        &self.table_name
    }
}

impl<T> BaseRepository for GenericRepository<T>
where
    T: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow>,
{
    fn table_name() -> &'static str {
        // 这个方法需要在具体实现中重写
        ""
    }

    fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    fn crypto_service(&self) -> &CryptoService {
        &self.crypto_service
    }

    async fn find_by_id<U>(&self, id: i64) -> RepositoryResult<Option<U>>
    where
        U: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
    {
        let query = format!("SELECT * FROM {} WHERE id = ?", self.table_name);

        debug!(
            table_name = %self.table_name,
            id = %id,
            "执行查询: {}",
            query
        );

        let result = sqlx::query_as::<_, U>(&query).bind(id).fetch_optional(self.pool()).await?;

        Ok(result)
    }

    async fn create<U>(&self, _data: &U) -> RepositoryResult<i64>
    where
        U: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
    {
        // 这里需要根据具体的结构体来构建INSERT语句
        // 由于这是一个通用实现，我们暂时返回错误
        Err(RepositoryError::Validation(
            "通用Repository需要具体实现create方法".to_string(),
        ))
    }

    async fn update<U>(&self, _id: i64, _data: &U) -> RepositoryResult<bool>
    where
        U: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
    {
        Err(RepositoryError::Validation(
            "通用Repository需要具体实现update方法".to_string(),
        ))
    }

    async fn delete(&self, id: i64) -> RepositoryResult<bool> {
        let query = format!("DELETE FROM {} WHERE id = ?", self.table_name);

        info!(
            table_name = %self.table_name,
            id = %id,
            "执行删除: {}",
            query
        );

        let result = sqlx::query(&query).bind(id).execute(self.pool()).await?;

        Ok(result.rows_affected() > 0)
    }

    async fn list_all<U>(&self) -> RepositoryResult<Vec<U>>
    where
        U: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
    {
        let query = format!("SELECT * FROM {} ORDER BY id DESC", self.table_name);

        debug!(
            table_name = %self.table_name,
            "执行查询: {}",
            query
        );

        let results = sqlx::query_as::<_, U>(&query).fetch_all(self.pool()).await?;

        Ok(results)
    }

    async fn paginate<U>(&self, params: &PaginationParams) -> RepositoryResult<PagedResult<U>>
    where
        U: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
    {
        let page = params.page.unwrap_or(1);
        let limit = params.limit.unwrap_or(20);
        let offset = params.offset.unwrap_or((page - 1) * limit);

        // 查询总数
        let count_query = format!("SELECT COUNT(*) FROM {}", self.table_name);
        let total: i64 = sqlx::query_scalar(&count_query).fetch_one(self.pool()).await?;

        // 查询分页数据
        let data_query = format!(
            "SELECT * FROM {} ORDER BY id DESC LIMIT ? OFFSET ?",
            self.table_name
        );

        debug!(
            table_name = %self.table_name,
            page = %page,
            limit = %limit,
            offset = %offset,
            "执行分页查询: {}",
            data_query
        );

        let data = sqlx::query_as::<_, U>(&data_query)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool())
            .await?;

        let _total_pages = (total + limit - 1) / limit;
        let paged_result = PagedResult::new(data, total, page, limit);

        Ok(paged_result)
    }

    async fn search<U>(
        &self,
        search_term: &str,
        search_fields: &[&str],
        limit: Option<i64>,
    ) -> RepositoryResult<Vec<U>>
    where
        U: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
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
            format!("SELECT * FROM {} ORDER BY id DESC LIMIT ?", self.table_name)
        } else {
            format!(
                "SELECT * FROM {} WHERE {} ORDER BY id DESC LIMIT ?",
                self.table_name,
                where_conditions.join(" OR ")
            )
        };

        debug!(
            table_name = %self.table_name,
            search_term = %search_term,
            search_fields = ?search_fields,
            query = %query,
            "执行搜索查询"
        );

        let mut query_builder = sqlx::query_as::<_, U>(&query);

        // 绑定搜索参数
        for condition in &where_conditions {
            query_builder = query_builder.bind(condition);
        }
        query_builder = query_builder.bind(limit);

        let results = query_builder.fetch_all(self.pool()).await?;

        Ok(results)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let query = format!("SELECT COUNT(*) FROM {}", self.table_name);

        debug!(
            table_name = %self.table_name,
            "执行计数查询: {}",
            query
        );

        let count: i64 = sqlx::query_scalar(&query).fetch_one(self.pool()).await?;

        Ok(count)
    }
}

/// 加密数据辅助函数
pub struct EncryptedField;

impl EncryptedField {
    /// 加密字符串字段
    pub fn encrypt_field(
        encrypted_value: &str,
        crypto_service: &CryptoService,
    ) -> RepositoryResult<String> {
        crypto_service.encrypt(encrypted_value).map_err(RepositoryError::Crypto)
    }

    /// 解密字符串字段
    pub fn decrypt_field(
        encrypted_value: &str,
        crypto_service: &CryptoService,
    ) -> RepositoryResult<String> {
        crypto_service.decrypt(encrypted_value).map_err(RepositoryError::Crypto)
    }

    /// 可选地解密字段
    pub fn optional_decrypt_field(
        encrypted_value: Option<String>,
        crypto_service: &CryptoService,
    ) -> RepositoryResult<Option<String>> {
        match encrypted_value {
            Some(value) => {
                let decrypted = crypto_service.decrypt(&value)?;
                Ok(Some(decrypted))
            }
            None => Ok(None),
        }
    }

    /// 条件解密字段
    pub fn conditional_decrypt_field(
        should_decrypt: bool,
        encrypted_value: &str,
        crypto_service: &CryptoService,
    ) -> RepositoryResult<String> {
        if should_decrypt {
            Self::decrypt_field(encrypted_value, crypto_service)
        } else {
            Ok(encrypted_value.to_string())
        }
    }
}

/// 构建查询条件的辅助函数
pub struct QueryBuilder;

impl QueryBuilder {
    /// 创建WHERE条件
    pub fn where_equals(field: &str, value: &str) -> String {
        format!("{} = '{}'", field, value)
    }

    /// 创建LIKE条件
    pub fn where_like(field: &str, value: &str) -> String {
        format!("{} LIKE '%{}%'", field, value)
    }

    /// 创建IN条件
    pub fn where_in(field: &str, values: &[&str]) -> String {
        let in_values = values.iter().map(|v| format!("'{}'", v)).collect::<Vec<_>>().join(", ");
        format!("{} IN ({})", field, in_values)
    }

    /// 创建条件组合
    pub fn and_conditions(conditions: &[&str]) -> String {
        if conditions.is_empty() {
            String::new()
        } else {
            conditions.join(" AND ")
        }
    }

    /// 创建OR条件组合
    pub fn or_conditions(conditions: &[&str]) -> String {
        if conditions.is_empty() {
            String::new()
        } else {
            conditions.join(" OR ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::DatabaseConfig;
    use tempfile::tempdir;

    #[allow(dead_code)]
    async fn create_test_repository() -> GenericRepository<serde_json::Value> {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
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
        let crypto_service = crate::crypto::CryptoService::new("test_key").unwrap();

        GenericRepository::new(&db_manager, "test_table", &crypto_service)
    }

    #[tokio::test]
    async fn test_encrypted_field_operations() {
        let crypto_service = crate::crypto::CryptoService::new("test_key").unwrap();
        let plaintext = "test_data";

        // 测试加密
        let encrypted = EncryptedField::encrypt_field(plaintext, &crypto_service).unwrap();
        assert_ne!(encrypted, plaintext);
        assert!(!encrypted.is_empty());

        // 测试解密
        let decrypted = EncryptedField::decrypt_field(&encrypted, &crypto_service).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[tokio::test]
    async fn test_query_builder() {
        // 测试条件构建
        let eq_condition = QueryBuilder::where_equals("name", "test");
        assert_eq!(eq_condition, "name = 'test'");

        let like_condition = QueryBuilder::where_like("description", "search");
        assert_eq!(like_condition, "description LIKE '%search%'");

        let in_condition = QueryBuilder::where_in("status", &["active", "inactive"]);
        assert_eq!(in_condition, "status IN ('active', 'inactive')");

        let and_condition = QueryBuilder::and_conditions(&["name = 'test'", "status = 'active'"]);
        assert_eq!(and_condition, "name = 'test' AND status = 'active'");

        let or_condition = QueryBuilder::or_conditions(&["name = 'test'", "status = 'active'"]);
        assert_eq!(or_condition, "name = 'test' OR status = 'active'");
    }
}
