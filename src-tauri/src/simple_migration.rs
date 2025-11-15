//! 简化版数据迁移工具
//!
//! 提供基本的数据导入导出功能

use crate::crypto::CryptoService;
use crate::database::DatabaseManager;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use thiserror::Error;
use tracing::{info, warn};
use std::collections::HashMap;

/// 简化迁移错误
#[derive(Error, Debug)]
pub enum SimpleMigrationError {
    #[error("加密错误: {0}")]
    Crypto(String),
    #[error("数据库错误: {0}")]
    Database(String),
    #[error("JSON错误: {0}")]
    Json(#[from] serde_json::Error),
}

/// 简化版导出数据结构
#[derive(Debug, Serialize, Deserialize)]
pub struct SimpleExportData {
    pub version: String,
    pub claude_providers: Vec<HashMap<String, serde_json::Value>>,
    pub common_configs: Vec<HashMap<String, serde_json::Value>>,
}

/// 简化版迁移工具
pub struct SimpleMigrationTool {
    crypto_service: CryptoService,
    db_manager: DatabaseManager,
}

impl SimpleMigrationTool {
    pub async fn new(
        db_manager: DatabaseManager,
        encryption_key: &str,
    ) -> Result<Self, SimpleMigrationError> {
        let crypto_service = CryptoService::new(encryption_key)
            .map_err(|e| SimpleMigrationError::Crypto(e.to_string()))?;

        Ok(Self { crypto_service, db_manager })
    }

    /// 导出基本数据到JSON
    pub async fn export_to_json(&self) -> Result<String, SimpleMigrationError> {
        info!("开始简化版数据导出...");

        let mut claude_providers = Vec::new();
        let mut common_configs = Vec::new();

        // 导出Claude供应商
        let provider_rows = sqlx::query("SELECT * FROM claude_providers")
            .fetch_all(self.db_manager.pool())
            .await
            .map_err(|e| SimpleMigrationError::Database(e.to_string()))?;

        for row in provider_rows {
            let mut provider = HashMap::new();
            provider.insert(
                "id".to_string(),
                serde_json::Value::Number(serde_json::Number::from(row.get::<i64, _>("id"))),
            );
            provider.insert(
                "name".to_string(),
                serde_json::Value::String(row.get::<String, _>("name")),
            );
            provider.insert(
                "url".to_string(),
                serde_json::Value::String(row.get::<String, _>("url")),
            );

            // 解密token
            let encrypted_token: String = row.get("token");
            let decrypted_token = match self.crypto_service.decrypt(&encrypted_token) {
                Ok(token) => token,
                Err(_) => {
                    warn!("无法解密token，保持加密状态");
                    encrypted_token
                }
            };
            provider.insert(
                "token".to_string(),
                serde_json::Value::String(decrypted_token),
            );

            claude_providers.push(provider);
        }

        // 导出通用配置
        let config_rows = sqlx::query("SELECT * FROM common_configs")
            .fetch_all(self.db_manager.pool())
            .await
            .map_err(|e| SimpleMigrationError::Database(e.to_string()))?;

        for row in config_rows {
            let mut config = HashMap::new();
            config.insert(
                "key".to_string(),
                serde_json::Value::String(row.get::<String, _>("key")),
            );
            config.insert(
                "value".to_string(),
                serde_json::Value::String(row.get::<String, _>("value")),
            );
            config.insert(
                "category".to_string(),
                serde_json::Value::String(row.get::<String, _>("category")),
            );
            common_configs.push(config);
        }

        let export_data = SimpleExportData {
            version: "2.0.0".to_string(),
            claude_providers,
            common_configs,
        };

        serde_json::to_string_pretty(&export_data).map_err(Into::into)
    }

    /// 从JSON导入基本数据
    pub async fn import_from_json(&self, json_str: &str) -> Result<(), SimpleMigrationError> {
        info!("开始简化版数据导入...");

        let export_data: SimpleExportData = serde_json::from_str(json_str)?;

        // 导入Claude供应商
        for provider_data in export_data.claude_providers {
            let name = provider_data.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
                SimpleMigrationError::Json(serde_json::Error::io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Expected string for name field",
                )))
            })?;

            let url = provider_data.get("url").and_then(|v| v.as_str()).unwrap_or("");

            let token = provider_data.get("token").and_then(|v| v.as_str()).unwrap_or("");

            // 加密token
            let encrypted_token = self
                .crypto_service
                .encrypt(token)
                .map_err(|e| SimpleMigrationError::Crypto(e.to_string()))?;

            // 插入数据库
            sqlx::query(
                "INSERT OR REPLACE INTO claude_providers (name, url, token) VALUES (?, ?, ?)",
            )
            .bind(name)
            .bind(url)
            .bind(encrypted_token)
            .execute(self.db_manager.pool())
            .await
            .map_err(|e| SimpleMigrationError::Database(e.to_string()))?;

            info!("✅ 导入Claude供应商: {}", name);
        }

        // 导入通用配置
        for config_data in export_data.common_configs {
            let key = config_data.get("key").and_then(|v| v.as_str()).ok_or_else(|| {
                SimpleMigrationError::Json(serde_json::Error::io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Expected string for key field",
                )))
            })?;

            let value = config_data.get("value").and_then(|v| v.as_str()).unwrap_or("");

            let category =
                config_data.get("category").and_then(|v| v.as_str()).unwrap_or("general");

            // 插入数据库
            sqlx::query(
                "INSERT OR REPLACE INTO common_configs (key, value, category) VALUES (?, ?, ?)",
            )
            .bind(key)
            .bind(value)
            .bind(category)
            .execute(self.db_manager.pool())
            .await
            .map_err(|e| SimpleMigrationError::Database(e.to_string()))?;

            info!("✅ 导入配置: {}", key);
        }

        info!("✅ 简化版数据导入完成");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::DatabaseConfig;
    use std::time::Duration;
    use tempfile::tempdir;

    async fn create_test_simple_migration() -> SimpleMigrationTool {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_simple.db");
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
        SimpleMigrationTool::new(db_manager, "Jw4Ff1BWLnSykdfXDVOuEJCG6m9dyST5B1VhU_qg0fI=")
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_simple_migration_tool_creation() {
        let migration_tool = create_test_simple_migration().await;
        println!("✅ 简化版迁移工具创建测试通过");
    }

    #[tokio::test]
    async fn test_simple_export_import() {
        let migration_tool = create_test_simple_migration().await;

        // 创建测试JSON数据
        let test_json = r#"{
            "version": "2.0.0",
            "claude_providers": [
                {
                    "id": 1,
                    "name": "Test Provider",
                    "url": "https://api.test.com",
                    "token": "sk-ant-test-12345"
                }
            ],
            "common_configs": [
                {
                    "key": "test_key",
                    "value": "test_value",
                    "category": "test_category"
                }
            ]
        }"#;

        // 导入数据
        let result = migration_tool.import_from_json(test_json).await;
        assert!(result.is_ok(), "导入失败: {:?}", result);

        // 导出数据
        let exported_json = migration_tool.export_to_json().await.unwrap();
        println!("导出的JSON: {}", exported_json);

        // 验证导出的JSON包含我们的数据
        assert!(exported_json.contains("Test Provider"));
        assert!(exported_json.contains("test_key"));

        println!("✅ 简化版导入导出测试通过");
    }
}
