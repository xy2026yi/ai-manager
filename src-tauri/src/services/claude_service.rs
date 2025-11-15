// Claude供应商业务服务
//
// 提供Claude供应商的业务逻辑处理，包括验证、规则执行等

use crate::crypto::CryptoService;
use crate::database::DatabaseManager;
use crate::models::{
    ClaudeProvider, CreateClaudeProviderRequest, PagedResult, PaginationParams,
    UpdateClaudeProviderRequest,
};
use crate::repositories::{BaseRepository, ClaudeProviderRepository};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Claude供应商业务错误
#[derive(Debug, thiserror::Error)]
pub enum ClaudeServiceError {
    #[error("验证失败: {0}")]
    Validation(String),

    #[error("业务规则冲突: {0}")]
    BusinessRule(String),

    #[error("数据访问错误: {0}")]
    Repository(#[from] crate::repositories::base_repository::RepositoryError),

    #[error("供应商不存在: {0}")]
    ProviderNotFound(i64),

    #[error("供应商名称已存在: {0}")]
    NameAlreadyExists(String),

    #[error("没有启用的供应商")]
    NoActiveProvider,
}

/// Claude供应商服务结果类型
pub type ClaudeServiceResult<T> = Result<T, ClaudeServiceError>;

/// Claude供应商业务服务
#[derive(Clone)]
pub struct ClaudeProviderService {
    repository: Arc<ClaudeProviderRepository>,
}

impl ClaudeProviderService {
    /// 创建新的Claude供应商服务实例
    pub fn new(db_manager: Arc<DatabaseManager>, crypto_service: Arc<CryptoService>) -> Self {
        Self {
            repository: Arc::new(ClaudeProviderRepository::new(&db_manager, &crypto_service)),
        }
    }

    /// 创建Claude供应商
    pub async fn create_provider(
        &self,
        request: CreateClaudeProviderRequest,
    ) -> ClaudeServiceResult<i64> {
        info!(
            name = %request.name,
            url = %request.url,
            "创建Claude供应商业务逻辑开始"
        );

        // 验证请求
        self.validate_create_request(&request)?;

        // 检查名称唯一性
        if let Some(existing) = self.find_by_name(&request.name).await? {
            if existing.id != 0 {
                warn!(
                    name = %request.name,
                    "供应商名称已存在"
                );
                return Err(ClaudeServiceError::NameAlreadyExists(request.name));
            }
        }

        // 创建供应商记录
        let id = self.repository.create_claude_provider(&request).await?;

        info!(
            id = %id,
            name = %request.name,
            "Claude供应商创建成功"
        );

        Ok(id)
    }

    /// 根据ID获取Claude供应商
    pub async fn get_provider(&self, id: i64) -> ClaudeServiceResult<Option<ClaudeProvider>> {
        debug!(
            id = %id,
            "获取Claude供应商"
        );

        if id <= 0 {
            return Err(ClaudeServiceError::Validation("无效的供应商ID".to_string()));
        }

        let provider = self.repository.find_by_id_decrypted(id).await?;
        Ok(provider)
    }

    /// 更新Claude供应商
    pub async fn update_provider(
        &self,
        id: i64,
        request: UpdateClaudeProviderRequest,
    ) -> ClaudeServiceResult<bool> {
        info!(
            id = %id,
            "更新Claude供应商业务逻辑开始"
        );

        if id <= 0 {
            return Err(ClaudeServiceError::Validation("无效的供应商ID".to_string()));
        }

        // 验证请求
        self.validate_update_request(&request)?;

        // 检查供应商是否存在
        let existing = self.repository.find_by_id::<ClaudeProvider>(id).await?;
        if existing.is_none() {
            warn!(
                id = %id,
                "尝试更新不存在的供应商"
            );
            return Err(ClaudeServiceError::ProviderNotFound(id));
        }

        // 如果更新名称，检查唯一性
        if let Some(ref new_name) = request.name {
            if let Some(existing) = self.find_by_name(new_name).await? {
                if existing.id != id {
                    warn!(
                        id = %id,
                        new_name = %new_name,
                        "供应商名称已存在"
                    );
                    return Err(ClaudeServiceError::NameAlreadyExists(new_name.clone()));
                }
            }
        }

        // 执行更新
        let updated = self.repository.update_claude_provider(id, &request).await?;

        if updated {
            info!(
                id = %id,
                "Claude供应商更新成功"
            );
        } else {
            warn!(
                id = %id,
                "Claude供应商更新未影响任何记录"
            );
        }

        Ok(updated)
    }

    /// 删除Claude供应商
    pub async fn delete_provider(&self, id: i64) -> ClaudeServiceResult<bool> {
        info!(
            id = %id,
            "删除Claude供应商业务逻辑开始"
        );

        if id <= 0 {
            return Err(ClaudeServiceError::Validation("无效的供应商ID".to_string()));
        }

        // 检查供应商是否存在
        let existing = self.repository.find_by_id::<ClaudeProvider>(id).await?;
        if existing.is_none() {
            warn!(
                id = %id,
                "尝试删除不存在的供应商"
            );
            return Err(ClaudeServiceError::ProviderNotFound(id));
        }

        // 删除供应商
        let deleted = self.repository.delete(id).await?;

        if deleted {
            info!(
                id = %id,
                "Claude供应商删除成功"
            );
        } else {
            warn!(
                id = %id,
                "Claude供应商删除未影响任何记录"
            );
        }

        Ok(deleted)
    }

    /// 获取Claude供应商列表
    pub async fn list_providers(
        &self,
        params: PaginationParams,
    ) -> ClaudeServiceResult<PagedResult<ClaudeProvider>> {
        debug!(
            page = ?params.page,
            limit = ?params.limit,
            "获取Claude供应商列表"
        );

        let result = self.repository.paginate::<ClaudeProvider>(&params).await?;
        Ok(result)
    }

    /// 搜索Claude供应商（内存优化版本）
    pub async fn search_providers(
        &self,
        search_term: &str,
        limit: Option<i64>,
    ) -> ClaudeServiceResult<Vec<ClaudeProvider>> {
        let trimmed_term = search_term.trim();
        
        debug!(
            search_term = %trimmed_term,
            limit = ?limit,
            "搜索Claude供应商"
        );

        if trimmed_term.is_empty() {
            return Err(ClaudeServiceError::Validation("搜索词不能为空".to_string()));
        }

        // 避免不必要的字符串分配，直接传递引用
        let providers = self.repository.search_claude_providers(trimmed_term, limit).await?;
        Ok(providers)
    }

    /// 获取活跃的Claude供应商（优化版本，使用索引查询）
    pub async fn list_active_providers(&self) -> ClaudeServiceResult<Vec<ClaudeProvider>> {
        debug!("获取活跃的Claude供应商列表");

        // 直接使用优化的索引查询，避免多次数据库访问
        let query = "SELECT * FROM claude_providers WHERE enabled = 1 ORDER BY id DESC";
        
        let providers = sqlx::query_as::<_, ClaudeProvider>(query)
            .fetch_all(&self.repository.pool)
            .await
            .map_err(|e| ClaudeServiceError::Repository(e.into()))?;
        
        Ok(providers)
    }

    /// 获取当前启用的供应商
    pub async fn get_current_provider(&self) -> ClaudeServiceResult<Option<ClaudeProvider>> {
        debug!("获取当前启用的Claude供应商");

        let active_providers = self.list_active_providers().await?;

        // 根据业务规则，应该只有一个启用的供应商
        match active_providers.len() {
            0 => {
                debug!("没有找到启用的供应商");
                Ok(None)
            }
            1 => {
                debug!("找到唯一的启用供应商");
                Ok(Some(active_providers.into_iter().next().unwrap()))
            }
            _ => {
                warn!(
                    count = %active_providers.len(),
                    "发现多个启用的供应商，这不符合业务规则"
                );
                // 返回第一个，但记录警告
                Ok(Some(active_providers.into_iter().next().unwrap()))
            }
        }
    }

    /// 启用供应商（同时禁用其他供应商）
    pub async fn enable_provider(&self, id: i64) -> ClaudeServiceResult<bool> {
        info!(
            id = %id,
            "启用Claude供应商"
        );

        if id <= 0 {
            return Err(ClaudeServiceError::Validation("无效的供应商ID".to_string()));
        }

        // 检查供应商是否存在
        let provider = self.repository.find_by_id::<ClaudeProvider>(id).await?;
        if provider.is_none() {
            return Err(ClaudeServiceError::ProviderNotFound(id));
        }

        // 禁用所有其他供应商
        self.disable_all_providers().await?;

        // 启用指定供应商
        let update_request = UpdateClaudeProviderRequest {
            name: None,
            url: None,
            token: None,
            timeout: None,
            auto_update: None,
            r#type: None,
            enabled: Some(1),
            opus_model: None,
            sonnet_model: None,
            haiku_model: None,
        };

        let enabled = self.repository.update_claude_provider(id, &update_request).await?;

        if enabled {
            info!(
                id = %id,
                "Claude供应商启用成功"
            );
        }

        Ok(enabled)
    }

    /// 禁用供应商
    pub async fn disable_provider(&self, id: i64) -> ClaudeServiceResult<bool> {
        info!(
            id = %id,
            "禁用Claude供应商"
        );

        if id <= 0 {
            return Err(ClaudeServiceError::Validation("无效的供应商ID".to_string()));
        }

        // 检查供应商是否存在
        let provider = self.repository.find_by_id::<ClaudeProvider>(id).await?;
        if provider.is_none() {
            return Err(ClaudeServiceError::ProviderNotFound(id));
        }

        let update_request = UpdateClaudeProviderRequest {
            name: None,
            url: None,
            token: None,
            timeout: None,
            auto_update: None,
            r#type: None,
            enabled: Some(0),
            opus_model: None,
            sonnet_model: None,
            haiku_model: None,
        };

        let disabled = self.repository.update_claude_provider(id, &update_request).await?;

        if disabled {
            info!(
                id = %id,
                "Claude供应商禁用成功"
            );
        }

        Ok(disabled)
    }

    /// 测试供应商连接
    pub async fn test_provider_connection(&self, id: i64) -> ClaudeServiceResult<bool> {
        debug!(
            id = %id,
            "测试Claude供应商连接"
        );

        if id <= 0 {
            return Err(ClaudeServiceError::Validation("无效的供应商ID".to_string()));
        }

        // 检查供应商是否存在
        let _provider = self
            .repository
            .find_by_id::<ClaudeProvider>(id)
            .await?
            .ok_or(ClaudeServiceError::ProviderNotFound(id))?;

        // 执行连接测试
        let result = self.repository.test_connection(id).await?;

        info!(
            id = %id,
            success = %result,
            "Claude供应商连接测试完成"
        );

        Ok(result)
    }

    /// 获取供应商统计信息
    pub async fn get_provider_stats(&self) -> ClaudeServiceResult<serde_json::Value> {
        debug!("获取Claude供应商统计信息");

        let total = self.repository.count().await?;
        let active_count = self.repository.count_by_status(true).await?;
        let inactive_count = self.repository.count_by_status(false).await?;

        let stats = serde_json::json!({
            "total": total,
            "active": active_count,
            "inactive": inactive_count,
            "active_rate": if total > 0 {
                (active_count as f64 / total as f64 * 100.0).round()
            } else {
                0.0
            }
        });

        info!(
            total = %total,
            active = %active_count,
            inactive = %inactive_count,
            "Claude供应商统计信息获取完成"
        );

        Ok(stats)
    }

    // 私有辅助方法

    /// 根据名称查找供应商
    async fn find_by_name(&self, name: &str) -> ClaudeServiceResult<Option<ClaudeProvider>> {
        let search_results = self.repository.search_claude_providers(name, Some(1)).await?;
        Ok(search_results.into_iter().next())
    }

    /// 禁用所有供应商
    async fn disable_all_providers(&self) -> ClaudeServiceResult<()> {
        debug!("禁用所有Claude供应商");

        // 获取所有启用的供应商
        let active_providers = self.list_active_providers().await?;

        for provider in active_providers {
            let update_request = UpdateClaudeProviderRequest {
                name: None,
                url: None,
                token: None,
                timeout: None,
                auto_update: None,
                r#type: None,
                enabled: Some(0),
                opus_model: None,
                sonnet_model: None,
                haiku_model: None,
            };

            let _ = self.repository.update_claude_provider(provider.id, &update_request).await?;
        }

        info!("已禁用所有Claude供应商");
        Ok(())
    }

    /// 验证创建请求
    fn validate_create_request(
        &self,
        request: &CreateClaudeProviderRequest,
    ) -> ClaudeServiceResult<()> {
        if request.name.trim().is_empty() {
            return Err(ClaudeServiceError::Validation(
                "供应商名称不能为空".to_string(),
            ));
        }

        if request.url.trim().is_empty() {
            return Err(ClaudeServiceError::Validation(
                "供应商URL不能为空".to_string(),
            ));
        }

        if !request.url.starts_with("http://") && !request.url.starts_with("https://") {
            return Err(ClaudeServiceError::Validation(
                "供应商URL必须以http://或https://开头".to_string(),
            ));
        }

        if request.token.trim().is_empty() {
            return Err(ClaudeServiceError::Validation(
                "供应商Token不能为空".to_string(),
            ));
        }

        if let Some(timeout) = request.timeout {
            if timeout <= 0 {
                return Err(ClaudeServiceError::Validation(
                    "超时时间必须大于0".to_string(),
                ));
            }
        }

        if let Some(ref r#type) = request.r#type {
            if r#type != "paid" && r#type != "public_welfare" {
                return Err(ClaudeServiceError::Validation(
                    "供应商类型必须是'paid'或'public_welfare'".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// 验证更新请求
    fn validate_update_request(
        &self,
        request: &UpdateClaudeProviderRequest,
    ) -> ClaudeServiceResult<()> {
        if let Some(ref name) = request.name {
            if name.trim().is_empty() {
                return Err(ClaudeServiceError::Validation(
                    "供应商名称不能为空".to_string(),
                ));
            }
        }

        if let Some(ref url) = request.url {
            if url.trim().is_empty() {
                return Err(ClaudeServiceError::Validation(
                    "供应商URL不能为空".to_string(),
                ));
            }

            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err(ClaudeServiceError::Validation(
                    "供应商URL必须以http://或https://开头".to_string(),
                ));
            }
        }

        if let Some(ref token) = request.token {
            if token.trim().is_empty() {
                return Err(ClaudeServiceError::Validation(
                    "供应商Token不能为空".to_string(),
                ));
            }
        }

        if let Some(timeout) = request.timeout {
            if timeout <= 0 {
                return Err(ClaudeServiceError::Validation(
                    "超时时间必须大于0".to_string(),
                ));
            }
        }

        if let Some(ref r#type) = request.r#type {
            if r#type != "paid" && r#type != "public_welfare" {
                return Err(ClaudeServiceError::Validation(
                    "供应商类型必须是'paid'或'public_welfare'".to_string(),
                ));
            }
        }

        if let Some(enabled) = request.enabled {
            if enabled != 0 && enabled != 1 {
                return Err(ClaudeServiceError::Validation(
                    "启用状态必须是0或1".to_string(),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::DatabaseConfig;
    use tempfile::tempdir;

    async fn create_test_service() -> ClaudeProviderService {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_claude_service.db");
        let db_url = format!("sqlite:{}", db_path.display());

        let config = DatabaseConfig {
            url: db_url,
            max_connections: 5,
            min_connections: 1,
            connect_timeout: std::time::Duration::from_secs(10),
            idle_timeout: std::time::Duration::from_secs(60),
            max_lifetime: std::time::Duration::from_secs(300),
        };

        let db_manager = Arc::new(DatabaseManager::new(config).await.unwrap());
        let crypto_service = Arc::new(CryptoService::new("test_key_for_claude_service").unwrap());

        ClaudeProviderService::new(db_manager, crypto_service)
    }

    #[tokio::test]
    async fn test_create_provider() {
        let service = create_test_service().await;

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

        let id = service.create_provider(create_request).await.unwrap();
        assert!(id > 0);

        // 验证可以获取创建的供应商
        let provider = service.get_provider(id).await.unwrap();
        assert!(provider.is_some());
        let provider = provider.unwrap();
        assert_eq!(provider.name, "测试Claude");
        assert_eq!(provider.url, "https://api.anthropic.com");
        assert_eq!(provider.enabled, 1); // 默认启用
    }

    #[tokio::test]
    async fn test_validation() {
        let service = create_test_service().await;

        // 测试空名称
        let create_request = CreateClaudeProviderRequest {
            name: "".to_string(),
            url: "https://api.anthropic.com".to_string(),
            token: "sk-test-api-key".to_string(),
            timeout: None,
            auto_update: None,
            r#type: None,
            opus_model: None,
            sonnet_model: None,
            haiku_model: None,
        };

        let result = service.create_provider(create_request).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ClaudeServiceError::Validation(_)
        ));
    }

    #[tokio::test]
    async fn test_enable_disable_provider() {
        let service = create_test_service().await;

        // 创建供应商
        let create_request = CreateClaudeProviderRequest {
            name: "测试供应商".to_string(),
            url: "https://api.anthropic.com".to_string(),
            token: "sk-test-api-key".to_string(),
            timeout: None,
            auto_update: None,
            r#type: None,
            opus_model: None,
            sonnet_model: None,
            haiku_model: None,
        };

        let id = service.create_provider(create_request).await.unwrap();

        // 测试禁用
        let disabled = service.disable_provider(id).await.unwrap();
        assert!(disabled);

        let provider = service.get_provider(id).await.unwrap().unwrap();
        assert_eq!(provider.enabled, 0);

        // 测试启用
        let enabled = service.enable_provider(id).await.unwrap();
        assert!(enabled);

        let provider = service.get_provider(id).await.unwrap().unwrap();
        assert_eq!(provider.enabled, 1);
    }
}
