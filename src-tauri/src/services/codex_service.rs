// Codex供应商业务服务
//
// 提供Codex供应商的业务逻辑处理，包括验证、规则执行等

use crate::crypto::CryptoService;
use crate::database::DatabaseManager;
use crate::models::{
    CodexProvider, CreateCodexProviderRequest, PagedResult, PaginationParams,
    UpdateCodexProviderRequest,
};
use crate::repositories::{BaseRepository, CodexProviderRepository};
use crate::{ValidationError, Validator};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Codex供应商业务错误
#[derive(Debug, thiserror::Error)]
pub enum CodexServiceError {
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

impl From<ValidationError> for CodexServiceError {
    fn from(error: ValidationError) -> Self {
        CodexServiceError::Validation(error.to_string())
    }
}

/// Codex供应商服务结果类型
pub type CodexServiceResult<T> = Result<T, CodexServiceError>;

/// Codex供应商业务服务
#[derive(Clone)]
pub struct CodexProviderService {
    repository: Arc<CodexProviderRepository>,
}

impl CodexProviderService {
    /// 创建新的Codex供应商服务实例
    pub fn new(db_manager: Arc<DatabaseManager>, crypto_service: Arc<CryptoService>) -> Self {
        Self {
            repository: Arc::new(CodexProviderRepository::new(&db_manager, &crypto_service)),
        }
    }

    /// 创建Codex供应商
    pub async fn create_provider(
        &self,
        request: &CreateCodexProviderRequest,
    ) -> CodexServiceResult<i64> {
        info!(
            name = %request.name,
            url = %request.url,
            "创建Codex供应商业务逻辑开始"
        );

        // 验证请求
        self.validate_create_request(request)?;

        // 检查名称唯一性
        if let Some(existing) = self.find_by_name(&request.name).await? {
            if existing.id != 0 {
                warn!(
                    name = %request.name,
                    "供应商名称已存在"
                );
                return Err(CodexServiceError::NameAlreadyExists(request.name.clone()));
            }
        }

        // 创建供应商记录
        let id = self.repository.create_codex_provider(request).await?;

        info!(
            id = %id,
            name = %request.name,
            "Codex供应商创建成功"
        );

        Ok(id)
    }

    /// 根据ID获取Codex供应商
    pub async fn get_provider(&self, id: i64) -> CodexServiceResult<Option<CodexProvider>> {
        debug!(
            id = %id,
            "获取Codex供应商"
        );

        Validator::validate_id(id, "id")?;

        let provider = self.repository.find_by_id_decrypted(id).await?;
        Ok(provider)
    }

    /// 更新Codex供应商
    pub async fn update_provider(
        &self,
        id: i64,
        request: UpdateCodexProviderRequest,
    ) -> CodexServiceResult<bool> {
        info!(
            id = %id,
            "更新Codex供应商业务逻辑开始"
        );

        Validator::validate_id(id, "id")?;

        // 验证请求
        self.validate_update_request(&request)?;

        // 检查供应商是否存在
        let existing = self.repository.find_by_id::<CodexProvider>(id).await?;
        if existing.is_none() {
            warn!(
                id = %id,
                "尝试更新不存在的供应商"
            );
            return Err(CodexServiceError::ProviderNotFound(id));
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
                    return Err(CodexServiceError::NameAlreadyExists(new_name.clone()));
                }
            }
        }

        // 执行更新
        let updated = self.repository.update_codex_provider(id, &request).await?;

        if updated {
            info!(
                id = %id,
                "Codex供应商更新成功"
            );
        } else {
            warn!(
                id = %id,
                "Codex供应商更新未影响任何记录"
            );
        }

        Ok(updated)
    }

    /// 删除Codex供应商
    pub async fn delete_provider(&self, id: i64) -> CodexServiceResult<bool> {
        info!(
            id = %id,
            "删除Codex供应商业务逻辑开始"
        );

        Validator::validate_id(id, "id")?;

        // 检查供应商是否存在
        let existing = self.repository.find_by_id::<CodexProvider>(id).await?;
        if existing.is_none() {
            warn!(
                id = %id,
                "尝试删除不存在的供应商"
            );
            return Err(CodexServiceError::ProviderNotFound(id));
        }

        // 删除供应商
        let deleted = self.repository.delete(id).await?;

        if deleted {
            info!(
                id = %id,
                "Codex供应商删除成功"
            );
        } else {
            warn!(
                id = %id,
                "Codex供应商删除未影响任何记录"
            );
        }

        Ok(deleted)
    }

    /// 获取Codex供应商列表
    pub async fn list_providers(
        &self,
        params: PaginationParams,
    ) -> CodexServiceResult<PagedResult<CodexProvider>> {
        debug!(
            page = ?params.page,
            limit = ?params.limit,
            "获取Codex供应商列表"
        );

        let result = self.repository.paginate::<CodexProvider>(&params).await?;
        Ok(result)
    }

    /// 搜索Codex供应商
    pub async fn search_providers(
        &self,
        search_term: &str,
        limit: Option<i64>,
    ) -> CodexServiceResult<Vec<CodexProvider>> {
        debug!(
            search_term = %search_term,
            limit = ?limit,
            "搜索Codex供应商"
        );

        // 使用统一验证器验证搜索词
        Validator::validate_search_term(search_term.trim())?;

        let providers = self.repository.search_codex_providers(search_term, limit).await?;
        Ok(providers)
    }

    /// 获取活跃的Codex供应商
    pub async fn list_active_providers(&self) -> CodexServiceResult<Vec<CodexProvider>> {
        debug!("获取活跃的Codex供应商列表");

        let providers = self.repository.list_active_providers().await?;
        Ok(providers)
    }

    /// 获取当前启用的供应商
    pub async fn get_current_provider(&self) -> CodexServiceResult<Option<CodexProvider>> {
        debug!("获取当前启用的Codex供应商");

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
    pub async fn enable_provider(&self, id: i64) -> CodexServiceResult<bool> {
        info!(
            id = %id,
            "启用Codex供应商"
        );

        Validator::validate_id(id, "id")?;

        // 检查供应商是否存在
        let provider = self.repository.find_by_id::<CodexProvider>(id).await?;
        if provider.is_none() {
            return Err(CodexServiceError::ProviderNotFound(id));
        }

        // 禁用所有其他供应商
        self.disable_all_providers().await?;

        // 启用指定供应商
        let update_request = UpdateCodexProviderRequest {
            name: None,
            url: None,
            token: None,
            r#type: None,
            enabled: Some(1),
        };

        let enabled = self.repository.update_codex_provider(id, &update_request).await?;

        if enabled {
            info!(
                id = %id,
                "Codex供应商启用成功"
            );
        }

        Ok(enabled)
    }

    /// 禁用供应商
    pub async fn disable_provider(&self, id: i64) -> CodexServiceResult<bool> {
        info!(
            id = %id,
            "禁用Codex供应商"
        );

        Validator::validate_id(id, "id")?;

        // 检查供应商是否存在
        let provider = self.repository.find_by_id::<CodexProvider>(id).await?;
        if provider.is_none() {
            return Err(CodexServiceError::ProviderNotFound(id));
        }

        let update_request = UpdateCodexProviderRequest {
            name: None,
            url: None,
            token: None,
            r#type: None,
            enabled: Some(0),
        };

        let disabled = self.repository.update_codex_provider(id, &update_request).await?;

        if disabled {
            info!(
                id = %id,
                "Codex供应商禁用成功"
            );
        }

        Ok(disabled)
    }

    /// 测试供应商连接
    pub async fn test_provider_connection(&self, id: i64) -> CodexServiceResult<bool> {
        debug!(
            id = %id,
            "测试Codex供应商连接"
        );

        Validator::validate_id(id, "id")?;

        // 检查供应商是否存在
        let _provider = self
            .repository
            .find_by_id::<CodexProvider>(id)
            .await?
            .ok_or(CodexServiceError::ProviderNotFound(id))?;

        // 执行连接测试
        let result = self.repository.test_connection(id).await?;

        info!(
            id = %id,
            success = %result,
            "Codex供应商连接测试完成"
        );

        Ok(result)
    }

    /// 获取供应商统计信息
    pub async fn get_provider_stats(&self) -> CodexServiceResult<serde_json::Value> {
        debug!("获取Codex供应商统计信息");

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
            "Codex供应商统计信息获取完成"
        );

        Ok(stats)
    }

    // 私有辅助方法

    /// 根据名称查找供应商
    async fn find_by_name(&self, name: &str) -> CodexServiceResult<Option<CodexProvider>> {
        let search_results = self.repository.search_codex_providers(name, Some(1)).await?;
        Ok(search_results.into_iter().next())
    }

    /// 禁用所有供应商
    async fn disable_all_providers(&self) -> CodexServiceResult<()> {
        debug!("禁用所有Codex供应商");

        // 获取所有启用的供应商
        let active_providers = self.list_active_providers().await?;

        for provider in active_providers {
            let update_request = UpdateCodexProviderRequest {
                name: None,
                url: None,
                token: None,
                r#type: None,
                enabled: Some(0),
            };

            let _ = self.repository.update_codex_provider(provider.id, &update_request).await?;
        }

        info!("已禁用所有Codex供应商");
        Ok(())
    }

    /// 验证创建请求
    fn validate_create_request(
        &self,
        request: &CreateCodexProviderRequest,
    ) -> CodexServiceResult<()> {
        // 使用统一验证器验证基本字段
        Validator::validate_provider_name(&request.name)?;
        Validator::validate_url(&request.url)?;

        if request.token.trim().is_empty() {
            return Err(CodexServiceError::Validation(
                "供应商Token不能为空".to_string(),
            ));
        }

        Ok(())
    }

    /// 验证更新请求
    fn validate_update_request(
        &self,
        request: &UpdateCodexProviderRequest,
    ) -> CodexServiceResult<()> {
        if let Some(ref name) = request.name {
            if name.trim().is_empty() {
                return Err(CodexServiceError::Validation(
                    "供应商名称不能为空".to_string(),
                ));
            }
        }

        if let Some(ref url) = request.url {
            if url.trim().is_empty() {
                return Err(CodexServiceError::Validation(
                    "供应商URL不能为空".to_string(),
                ));
            }

            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err(CodexServiceError::Validation(
                    "供应商URL必须以http://或https://开头".to_string(),
                ));
            }
        }

        if let Some(ref token) = request.token {
            if token.trim().is_empty() {
                return Err(CodexServiceError::Validation(
                    "供应商Token不能为空".to_string(),
                ));
            }
        }

        if let Some(enabled) = request.enabled {
            if enabled != 0 && enabled != 1 {
                return Err(CodexServiceError::Validation(
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

    async fn create_test_service() -> CodexProviderService {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_codex_service.db");
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
        let crypto_service = Arc::new(CryptoService::new("test_key_for_codex_service").unwrap());

        CodexProviderService::new(db_manager, crypto_service)
    }

    #[tokio::test]
    async fn test_create_provider() {
        let service = create_test_service().await;

        let create_request = CreateCodexProviderRequest {
            name: "测试Codex".to_string(),
            url: "https://api.openai.com".to_string(),
            token: "sk-test-api-key".to_string(),
            r#type: Some("gpt-4".to_string()),
        };

        let id = service.create_provider(&create_request).await.unwrap();
        assert!(id > 0);

        // 验证可以获取创建的供应商
        let provider = service.get_provider(id).await.unwrap();
        assert!(provider.is_some());
        let provider = provider.unwrap();
        assert_eq!(provider.name, "测试Codex");
        assert_eq!(provider.url, "https://api.openai.com");
        assert_eq!(provider.enabled, 1); // 默认启用
    }

    #[tokio::test]
    async fn test_validation() {
        let service = create_test_service().await;

        // 测试空名称
        let create_request = CreateCodexProviderRequest {
            name: "".to_string(),
            url: "https://api.openai.com".to_string(),
            token: "sk-test-api-key".to_string(),
            r#type: None,
        };

        let result = service.create_provider(&create_request).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CodexServiceError::Validation(_)
        ));
    }

    #[tokio::test]
    async fn test_enable_disable_provider() {
        let service = create_test_service().await;

        // 创建供应商
        let create_request = CreateCodexProviderRequest {
            name: "测试供应商".to_string(),
            url: "https://api.openai.com".to_string(),
            token: "sk-test-api-key".to_string(),
            r#type: None,
        };

        let id = service.create_provider(&create_request).await.unwrap();

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
