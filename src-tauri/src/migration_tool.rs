//! 数据迁移工具
//!
//! 这个模块提供从Python版本AI Manager迁移数据到Rust版本的工具

use crate::crypto::{CryptoError, CryptoService};
use crate::database::{DatabaseManager, QueryBuilder};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::clone::Clone;
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;
use tracing::{debug, error, info, warn};

/// 迁移错误类型
#[derive(Error, Debug)]
pub enum MigrationError {
    #[error("加密错误: {0}")]
    Crypto(#[from] CryptoError),
    #[error("数据库错误: {0}")]
    Database(#[from] crate::database::DatabaseError),
    #[error("文件错误: {0}")]
    File(#[from] std::io::Error),
    #[error("JSON解析错误: {0}")]
    Json(#[from] serde_json::Error),
    #[error("数据验证错误: {0}")]
    Validation(String),
    #[error("版本不兼容: {0}")]
    VersionMismatch(String),
}

/// Python导出的数据格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonExportData {
    pub version: String,
    pub claude_providers: Vec<PythonClaudeProvider>,
    pub codex_providers: Vec<PythonCodexProvider>,
    pub agent_guides: Vec<PythonAgentGuide>,
    pub mcp_servers: Vec<PythonMcpServer>,
    pub common_configs: Vec<PythonCommonConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonClaudeProvider {
    pub id: Option<i64>,
    pub name: String,
    pub url: String,
    pub token: String,
    pub timeout: Option<i64>,
    pub auto_update: Option<i64>,
    pub r#type: Option<String>,
    pub enabled: Option<i64>,
    pub opus_model: Option<String>,
    pub sonnet_model: Option<String>,
    pub haiku_model: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonCodexProvider {
    pub id: Option<i64>,
    pub name: String,
    pub url: String,
    pub token: String,
    pub r#type: Option<String>,
    pub enabled: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonAgentGuide {
    pub id: Option<i64>,
    pub name: String,
    pub r#type: String,
    pub text: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonMcpServer {
    pub id: Option<i64>,
    pub name: String,
    pub r#type: Option<String>,
    pub timeout: Option<i64>,
    pub command: String,
    pub args: Vec<String>,
    pub env: Option<HashMap<String, String>>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonCommonConfig {
    pub id: Option<i64>,
    pub key: String,
    pub value: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub is_active: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// 迁移报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationReport {
    pub total_migrated: usize,
    pub claude_providers: usize,
    pub codex_providers: usize,
    pub agent_guides: usize,
    pub mcp_servers: usize,
    pub common_configs: usize,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub duration_secs: u64,
}

/// 数据迁移工具
pub struct DataMigrationTool {
    crypto_service: CryptoService,
    db_manager: DatabaseManager,
}

impl DataMigrationTool {
    /// 创建新的迁移工具实例
    pub async fn new(
        db_manager: DatabaseManager,
        encryption_key: &str,
    ) -> Result<Self, MigrationError> {
        let crypto_service = CryptoService::new(encryption_key)?;

        Ok(Self { crypto_service, db_manager })
    }

    /// 从JSON文件导入Python数据
    pub async fn import_from_json_file<P: AsRef<Path>>(
        &self,
        file_path: P,
    ) -> Result<MigrationReport, MigrationError> {
        let content = std::fs::read_to_string(file_path)?;
        self.import_from_json(&content).await
    }

    /// 从JSON字符串导入Python数据
    pub async fn import_from_json(
        &self,
        json_content: &str,
    ) -> Result<MigrationReport, MigrationError> {
        info!("开始从JSON导入数据...");
        let start_time = std::time::Instant::now();

        let python_data: PythonExportData = serde_json::from_str(json_content)?;

        // 验证版本兼容性
        self.validate_version(&python_data.version)?;

        let mut report = MigrationReport {
            total_migrated: 0,
            claude_providers: 0,
            codex_providers: 0,
            agent_guides: 0,
            mcp_servers: 0,
            common_configs: 0,
            errors: Vec::new(),
            warnings: Vec::new(),
            duration_secs: 0,
        };

        // 清空现有数据（可选）
        self.clear_existing_data(&mut report).await?;

        // 导入Claude供应商
        report.claude_providers =
            self.import_claude_providers(&python_data.claude_providers, &mut report).await?;

        // 导入Codex供应商
        report.codex_providers =
            self.import_codex_providers(&python_data.codex_providers, &mut report).await?;

        // 导入Agent指导文件
        report.agent_guides =
            self.import_agent_guides(&python_data.agent_guides, &mut report).await?;

        // 导入MCP服务器
        report.mcp_servers = self.import_mcp_servers(&python_data.mcp_servers, &mut report).await?;

        // 导入通用配置
        report.common_configs =
            self.import_common_configs(&python_data.common_configs, &mut report).await?;

        report.total_migrated = report.claude_providers
            + report.codex_providers
            + report.agent_guides
            + report.mcp_servers
            + report.common_configs;

        report.duration_secs = start_time.elapsed().as_secs();

        info!("✅ 数据迁移完成: {:?}", report);
        Ok(report)
    }

    /// 验证版本兼容性
    fn validate_version(&self, version: &str) -> Result<(), MigrationError> {
        // 这里可以添加版本兼容性检查逻辑
        match version {
            v if v.starts_with("1.") => {
                info!("✅ 版本 {} 兼容", version);
                Ok(())
            }
            _ => Err(MigrationError::VersionMismatch(format!(
                "不支持的版本: {}",
                version
            ))),
        }
    }

    /// 清空现有数据
    async fn clear_existing_data(
        &self,
        report: &mut MigrationReport,
    ) -> Result<(), MigrationError> {
        info!("清理现有数据...");

        let query_builder = QueryBuilder::new(self.db_manager.pool());

        // 清空各个表
        let tables = [
            "claude_providers",
            "codex_providers",
            "agent_guides",
            "mcp_servers",
            "common_configs",
        ];
        for table in tables {
            match query_builder.execute_raw(&format!("DELETE FROM {}", table), &[]).await {
                Ok(_) => debug!("清空表 {}", table),
                Err(e) => {
                    let msg = format!("清空表 {} 失败: {}", table, e);
                    warn!("{}", msg);
                    report.warnings.push(msg);
                }
            }
        }

        Ok(())
    }

    /// 导入Claude供应商
    async fn import_claude_providers(
        &self,
        providers: &[PythonClaudeProvider],
        report: &mut MigrationReport,
    ) -> Result<usize, MigrationError> {
        info!("导入 {} 个Claude供应商", providers.len());
        let mut imported = 0;

        for provider in providers {
            match self.import_claude_provider(provider).await {
                Ok(_) => {
                    imported += 1;
                    debug!("✅ 导入Claude供应商: {}", provider.name);
                }
                Err(e) => {
                    let msg = format!("导入Claude供应商失败 {}: {}", provider.name, e);
                    error!("{}", msg);
                    report.errors.push(msg);
                }
            }
        }

        Ok(imported)
    }

    /// 导入单个Claude供应商
    async fn import_claude_provider(
        &self,
        provider: &PythonClaudeProvider,
    ) -> Result<(), MigrationError> {
        // 加密token
        let encrypted_token = self.crypto_service.encrypt(&provider.token)?;

        let query = r#"
            INSERT INTO claude_providers
            (name, url, token, timeout, auto_update, type, enabled, opus_model, sonnet_model, haiku_model)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        let timeout_val = provider.timeout.unwrap_or(30000).to_string();
        let auto_update_val = provider.auto_update.unwrap_or(1).to_string();
        let type_val = provider.r#type.clone().unwrap_or_else(|| "public_welfare".to_string());
        let enabled_val = provider.enabled.unwrap_or(0).to_string();
        let opus_val = provider.opus_model.as_deref().unwrap_or("").to_string();
        let sonnet_val = provider.sonnet_model.as_deref().unwrap_or("").to_string();
        let haiku_val = provider.haiku_model.as_deref().unwrap_or("").to_string();

        let params = [
            &provider.name,
            &provider.url,
            &encrypted_token,
            &timeout_val,
            &auto_update_val,
            &type_val,
            &enabled_val,
            &opus_val,
            &sonnet_val,
            &haiku_val,
        ];

        QueryBuilder::new(self.db_manager.pool())
            .execute_raw(
                query,
                &params.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            )
            .await?;

        Ok(())
    }

    /// 导入Codex供应商
    async fn import_codex_providers(
        &self,
        providers: &[PythonCodexProvider],
        report: &mut MigrationReport,
    ) -> Result<usize, MigrationError> {
        info!("导入 {} 个Codex供应商", providers.len());
        let mut imported = 0;

        for provider in providers {
            match self.import_codex_provider(provider).await {
                Ok(_) => {
                    imported += 1;
                    debug!("✅ 导入Codex供应商: {}", provider.name);
                }
                Err(e) => {
                    let msg = format!("导入Codex供应商失败 {}: {}", provider.name, e);
                    error!("{}", msg);
                    report.errors.push(msg);
                }
            }
        }

        Ok(imported)
    }

    /// 导入单个Codex供应商
    async fn import_codex_provider(
        &self,
        provider: &PythonCodexProvider,
    ) -> Result<(), MigrationError> {
        let encrypted_token = self.crypto_service.encrypt(&provider.token)?;

        let query = r#"
            INSERT INTO codex_providers
            (name, url, token, type, enabled)
            VALUES (?, ?, ?, ?, ?)
        "#;

        let params = [
            &provider.name,
            &provider.url,
            &encrypted_token,
            &provider.r#type.clone().unwrap_or_else(|| "public_welfare".to_string()),
            &provider.enabled.unwrap_or(0).to_string(),
        ];

        QueryBuilder::new(self.db_manager.pool())
            .execute_raw(
                query,
                &params.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            )
            .await?;

        Ok(())
    }

    /// 导入Agent指导文件
    async fn import_agent_guides(
        &self,
        guides: &[PythonAgentGuide],
        report: &mut MigrationReport,
    ) -> Result<usize, MigrationError> {
        info!("导入 {} 个Agent指导文件", guides.len());
        let mut imported = 0;

        for guide in guides {
            match self.import_agent_guide(guide).await {
                Ok(_) => {
                    imported += 1;
                    debug!("✅ 导入Agent指导: {}", guide.name);
                }
                Err(e) => {
                    let msg = format!("导入Agent指导失败 {}: {}", guide.name, e);
                    error!("{}", msg);
                    report.errors.push(msg);
                }
            }
        }

        Ok(imported)
    }

    /// 导入单个Agent指导文件
    async fn import_agent_guide(&self, guide: &PythonAgentGuide) -> Result<(), MigrationError> {
        let query = r#"
            INSERT INTO agent_guides
            (name, type, text)
            VALUES (?, ?, ?)
        "#;

        let params = [
            guide.name.as_str(),
            guide.r#type.as_str(),
            guide.text.as_str(),
        ];

        QueryBuilder::new(self.db_manager.pool()).execute_raw(query, &params).await?;

        Ok(())
    }

    /// 导入MCP服务器
    async fn import_mcp_servers(
        &self,
        servers: &[PythonMcpServer],
        report: &mut MigrationReport,
    ) -> Result<usize, MigrationError> {
        info!("导入 {} 个MCP服务器", servers.len());
        let mut imported = 0;

        for server in servers {
            match self.import_mcp_server(server).await {
                Ok(_) => {
                    imported += 1;
                    debug!("✅ 导入MCP服务器: {}", server.name);
                }
                Err(e) => {
                    let msg = format!("导入MCP服务器失败 {}: {}", server.name, e);
                    error!("{}", msg);
                    report.errors.push(msg);
                }
            }
        }

        Ok(imported)
    }

    /// 导入单个MCP服务器
    async fn import_mcp_server(&self, server: &PythonMcpServer) -> Result<(), MigrationError> {
        let args_json = serde_json::to_string(&server.args)?;
        let env_json = server.env.as_ref().map(serde_json::to_string).transpose()?;

        let query = r#"
            INSERT INTO mcp_servers
            (name, type, timeout, command, args, env)
            VALUES (?, ?, ?, ?, ?, ?)
        "#;

        let server_type = server.r#type.as_ref().unwrap_or(&"stdio".to_string()).clone();
        let env_value = env_json.as_ref().unwrap_or(&"".to_string()).clone();

        let params = [
            &server.name,
            &server_type,
            &server.timeout.unwrap_or(30000).to_string(),
            &server.command,
            &args_json,
            &env_value,
        ];

        QueryBuilder::new(self.db_manager.pool())
            .execute_raw(
                query,
                &params.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            )
            .await?;

        Ok(())
    }

    /// 导入通用配置
    async fn import_common_configs(
        &self,
        configs: &[PythonCommonConfig],
        report: &mut MigrationReport,
    ) -> Result<usize, MigrationError> {
        info!("导入 {} 个通用配置", configs.len());
        let mut imported = 0;

        for config in configs {
            match self.import_common_config(config).await {
                Ok(_) => {
                    imported += 1;
                    debug!("✅ 导入配置: {}", config.key);
                }
                Err(e) => {
                    let msg = format!("导入配置失败 {}: {}", config.key, e);
                    error!("{}", msg);
                    report.errors.push(msg);
                }
            }
        }

        Ok(imported)
    }

    /// 导入单个通用配置
    async fn import_common_config(
        &self,
        config: &PythonCommonConfig,
    ) -> Result<(), MigrationError> {
        let query = r#"
            INSERT INTO common_configs
            (key, value, description, category, is_active)
            VALUES (?, ?, ?, ?, ?)
        "#;

        let description = config.description.as_ref().unwrap_or(&"".to_string()).clone();
        let category = config.category.as_ref().unwrap_or(&"general".to_string()).clone();

        let params = [
            config.key.as_str(),
            config.value.as_str(),
            description.as_str(),
            category.as_str(),
            &config.is_active.unwrap_or(1).to_string(),
        ];

        QueryBuilder::new(self.db_manager.pool()).execute_raw(query, &params).await?;

        Ok(())
    }

    /// 导出数据到JSON文件
    pub async fn export_to_json_file<P: AsRef<Path>>(
        &self,
        file_path: P,
    ) -> Result<(), MigrationError> {
        let data = self.export_to_json().await?;
        std::fs::write(file_path, serde_json::to_string_pretty(&data)?)?;
        Ok(())
    }

    /// 导出数据到JSON字符串
    pub async fn export_to_json(&self) -> Result<PythonExportData, MigrationError> {
        info!("开始导出数据...");

        let query_builder = QueryBuilder::new(self.db_manager.pool());

        // 导出Claude供应商
        let claude_providers = self.export_claude_providers(&query_builder).await?;
        let codex_providers = self.export_codex_providers(&query_builder).await?;
        let agent_guides = self.export_agent_guides(&query_builder).await?;
        let mcp_servers = self.export_mcp_servers(&query_builder).await?;
        let common_configs = self.export_common_configs(&query_builder).await?;

        Ok(PythonExportData {
            version: "2.0.0".to_string(), // Rust版本号
            claude_providers,
            codex_providers,
            agent_guides,
            mcp_servers,
            common_configs,
        })
    }

    /// 导出Claude供应商
    async fn export_claude_providers(
        &self,
        _query_builder: &QueryBuilder<'_>,
    ) -> Result<Vec<PythonClaudeProvider>, MigrationError> {
        let rows = sqlx::query("SELECT * FROM claude_providers")
            .fetch_all(self.db_manager.pool())
            .await
            .map_err(|e| {
                MigrationError::Database(crate::database::DatabaseError::Query(e.to_string()))
            })?;

        let mut providers = Vec::new();
        for row in rows {
            let token: String = row.get("token");
            let decrypted_token = match self.crypto_service.decrypt(&token) {
                Ok(t) => t,
                Err(_) => {
                    warn!("无法解密Claude供应商token，保持加密状态");
                    token
                }
            };

            providers.push(PythonClaudeProvider {
                id: Some(row.get("id")),
                name: row.get("name"),
                url: row.get("url"),
                token: decrypted_token,
                timeout: row.get("timeout"),
                auto_update: row.get("auto_update"),
                r#type: Some(row.get("type")),
                enabled: Some(row.get("enabled")),
                opus_model: row.get("opus_model"),
                sonnet_model: row.get("sonnet_model"),
                haiku_model: row.get("haiku_model"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }

        Ok(providers)
    }

    /// 导出Codex供应商
    async fn export_codex_providers(
        &self,
        _query_builder: &QueryBuilder<'_>,
    ) -> Result<Vec<PythonCodexProvider>, MigrationError> {
        let rows = sqlx::query("SELECT * FROM codex_providers")
            .fetch_all(self.db_manager.pool())
            .await
            .map_err(|e| {
                MigrationError::Database(crate::database::DatabaseError::Query(e.to_string()))
            })?;

        let mut providers = Vec::new();
        for row in rows {
            let token: String = row.get("token");
            let decrypted_token = match self.crypto_service.decrypt(&token) {
                Ok(t) => t,
                Err(_) => token,
            };

            providers.push(PythonCodexProvider {
                id: Some(row.get("id")),
                name: row.get("name"),
                url: row.get("url"),
                token: decrypted_token,
                r#type: Some(row.get("type")),
                enabled: Some(row.get("enabled")),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }

        Ok(providers)
    }

    /// 导出Agent指导文件
    async fn export_agent_guides(
        &self,
        _query_builder: &QueryBuilder<'_>,
    ) -> Result<Vec<PythonAgentGuide>, MigrationError> {
        let rows = sqlx::query("SELECT * FROM agent_guides")
            .fetch_all(self.db_manager.pool())
            .await
            .map_err(|e| {
                MigrationError::Database(crate::database::DatabaseError::Query(e.to_string()))
            })?;

        let mut guides = Vec::new();
        for row in rows {
            guides.push(PythonAgentGuide {
                id: Some(row.get("id")),
                name: row.get("name"),
                r#type: row.get("type"),
                text: row.get("text"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }

        Ok(guides)
    }

    /// 导出MCP服务器
    async fn export_mcp_servers(
        &self,
        _query_builder: &QueryBuilder<'_>,
    ) -> Result<Vec<PythonMcpServer>, MigrationError> {
        let rows = sqlx::query("SELECT * FROM mcp_servers")
            .fetch_all(self.db_manager.pool())
            .await
            .map_err(|e| {
                MigrationError::Database(crate::database::DatabaseError::Query(e.to_string()))
            })?;

        let mut servers = Vec::new();
        for row in rows {
            let args_str: String = row.get("args");
            let args: Vec<String> = serde_json::from_str(&args_str).unwrap_or_default();

            let env_str: Option<String> = row.get("env");
            let env = env_str.and_then(|s| serde_json::from_str(&s).ok());

            servers.push(PythonMcpServer {
                id: Some(row.get("id")),
                name: row.get("name"),
                r#type: row.get("type"),
                timeout: row.get("timeout"),
                command: row.get("command"),
                args,
                env,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }

        Ok(servers)
    }

    /// 导出通用配置
    async fn export_common_configs(
        &self,
        _query_builder: &QueryBuilder<'_>,
    ) -> Result<Vec<PythonCommonConfig>, MigrationError> {
        let rows = sqlx::query("SELECT * FROM common_configs")
            .fetch_all(self.db_manager.pool())
            .await
            .map_err(|e| {
                MigrationError::Database(crate::database::DatabaseError::Query(e.to_string()))
            })?;

        let mut configs = Vec::new();
        for row in rows {
            configs.push(PythonCommonConfig {
                id: Some(row.get("id")),
                key: row.get("key"),
                value: row.get("value"),
                description: row.get("description"),
                category: Some(row.get("category")),
                is_active: Some(row.get("is_active")),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }

        Ok(configs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::DatabaseConfig;
    use std::time::Duration;
    use tempfile::tempdir;

    async fn create_test_migration_tool() -> (DataMigrationTool, DatabaseManager) {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_migration.db");
        let db_url = format!("sqlite:{}", db_path.display());

        let config = DatabaseConfig {
            url: db_url,
            max_connections: 5,
            min_connections: 1,
            connect_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(60),
            max_lifetime: Duration::from_secs(300),
        };

        let db_manager = DatabaseManager::new(config).await.unwrap();
        let migration_tool = DataMigrationTool::new(
            db_manager.clone(),
            "Jw4Ff1BWLnSykdfXDVOuEJCG6m9dyST5B1VhU_qg0fI=",
        )
        .await
        .unwrap();

        (migration_tool, db_manager)
    }

    #[tokio::test]
    async fn test_migration_tool_creation() {
        let (_, _) = create_test_migration_tool().await;

        // 测试迁移工具创建成功
        assert!(true); // 如果到这里没有panic，说明创建成功
        println!("✅ 迁移工具创建测试通过");
    }

    #[tokio::test]
    async fn test_roundtrip_migration() {
        let (migration_tool, _) = create_test_migration_tool().await;

        // 创建测试数据
        let test_data = PythonExportData {
            version: "1.0.0".to_string(),
            claude_providers: vec![PythonClaudeProvider {
                id: None,
                name: "Test Claude Provider".to_string(),
                url: "https://api.anthropic.com".to_string(),
                token: "sk-ant-test-key".to_string(),
                timeout: Some(30000),
                auto_update: Some(1),
                r#type: Some("public_welfare".to_string()),
                enabled: Some(1),
                opus_model: Some("claude-3-opus-20240229".to_string()),
                sonnet_model: Some("claude-3-sonnet-20240229".to_string()),
                haiku_model: Some("claude-3-haiku-20240307".to_string()),
                created_at: None,
                updated_at: None,
            }],
            codex_providers: vec![],
            agent_guides: vec![],
            mcp_servers: vec![],
            common_configs: vec![],
        };

        // 导入数据
        let json = serde_json::to_string(&test_data).unwrap();
        let report = migration_tool.import_from_json(&json).await.unwrap();

        assert_eq!(report.claude_providers, 1);
        assert_eq!(report.total_migrated, 1);
        println!("✅ 数据导入测试通过: {:?}", report);

        // 导出数据
        let exported_data = migration_tool.export_to_json().await.unwrap();

        assert_eq!(exported_data.claude_providers.len(), 1);
        assert_eq!(
            exported_data.claude_providers[0].name,
            "Test Claude Provider"
        );
        println!("✅ 数据导出测试通过");
    }
}
