//! 数据兼容性验证测试
//! 
//! 验证从原Python项目迁移到Rust项目的数据兼容性
//! 包括数据库schema、加密格式和数据完整性

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, SqlitePool};
use crate::crypto::{CryptoService, CryptoError};
use crate::models::*;

/// 测试错误类型
#[derive(Debug)]
pub enum DataCompatibilityError {
    Database(String),
    Encryption(String),
    DataValidation(String),
    FileSystem(String),
}

impl std::fmt::Display for DataCompatibilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataCompatibilityError::Database(msg) => write!(f, "数据库错误: {}", msg),
            DataCompatibilityError::Encryption(msg) => write!(f, "加密错误: {}", msg),
            DataCompatibilityError::DataValidation(msg) => write!(f, "数据验证错误: {}", msg),
            DataCompatibilityError::FileSystem(msg) => write!(f, "文件系统错误: {}", msg),
        }
    }
}

impl std::error::Error for DataCompatibilityError {}

/// Python版本数据模型（用于兼容性验证）
#[derive(Debug, Serialize, Deserialize)]
pub struct PythonClaudeProvider {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub token: String,
    pub timeout: i64,
    pub auto_update: i64,
    #[serde(rename = "type")]
    pub provider_type: String,
    pub enabled: i64,
    pub opus_model: Option<String>,
    pub sonnet_model: Option<String>,
    pub haiku_model: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PythonCodexProvider {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub token: String,
    #[serde(rename = "type")]
    pub provider_type: String,
    pub enabled: i64,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PythonAgentGuide {
    pub id: i64,
    pub name: String,
    #[serde(rename = "type")]
    pub guide_type: String,
    pub text: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PythonMcpServer {
    pub id: i64,
    pub name: String,
    #[serde(rename = "type")]
    pub server_type: Option<String>,
    pub timeout: i64,
    pub command: String,
    pub args: serde_json::Value,
    pub env: Option<serde_json::Value>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PythonCommonConfig {
    pub id: i64,
    pub key: String,
    pub value: String,
    pub description: Option<String>,
    pub category: String,
    pub is_active: i64,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// 完整的Python数据库数据
#[derive(Debug, Serialize, Deserialize)]
pub struct PythonDatabaseData {
    pub version: String,
    pub claude_providers: Vec<PythonClaudeProvider>,
    pub codex_providers: Vec<PythonCodexProvider>,
    pub agent_guides: Vec<PythonAgentGuide>,
    pub mcp_servers: Vec<PythonMcpServer>,
    pub common_configs: Vec<PythonCommonConfig>,
}

/// 数据兼容性验证器
pub struct DataCompatibilityValidator {
    pool: SqlitePool,
    crypto: CryptoService,
    test_data: PythonDatabaseData,
}

impl DataCompatibilityValidator {
    /// 创建新的验证器实例
    pub async fn new(database_url: &str, encryption_key: &str) -> Result<Self, DataCompatibilityError> {
        // 连接数据库
        let pool = SqlitePool::connect(database_url)
            .await
            .map_err(|e| DataCompatibilityError::Database(format!("数据库连接失败: {}", e)))?;

        // 创建加密服务
        let crypto = CryptoService::new(encryption_key)
            .map_err(|e| DataCompatibilityError::Encryption(format!("加密服务初始化失败: {}", e)))?;

        // 加载测试数据
        let test_data = Self::load_test_data()
            .await
            .map_err(|e| DataCompatibilityError::FileSystem(format!("测试数据加载失败: {}", e)))?;

        Ok(Self {
            pool,
            crypto,
            test_data,
        })
    }

    /// 从文件加载Python测试数据
    async fn load_test_data() -> Result<PythonDatabaseData, DataCompatibilityError> {
        let test_data_path = "tests/data/python_original_sample.json";
        let content = fs::read_to_string(test_data_path)
            .map_err(|e| DataCompatibilityError::FileSystem(format!("读取测试数据文件失败: {}", e)))?;

        serde_json::from_str(&content)
            .map_err(|e| DataCompatibilityError::DataValidation(format!("测试数据解析失败: {}", e)))
    }

    /// 验证数据库Schema兼容性
    pub async fn validate_schema_compatibility(&self) -> Result<bool, DataCompatibilityError> {
        println!("🔍 验证数据库Schema兼容性...");

        let mut all_compatible = true;

        // 验证Claude供应商表结构
        if let Err(e) = self.validate_claude_providers_schema().await {
            println!("❌ Claude供应商表Schema验证失败: {}", e);
            all_compatible = false;
        } else {
            println!("✅ Claude供应商表Schema兼容");
        }

        // 验证Codex供应商表结构
        if let Err(e) = self.validate_codex_providers_schema().await {
            println!("❌ Codex供应商表Schema验证失败: {}", e);
            all_compatible = false;
        } else {
            println!("✅ Codex供应商表Schema兼容");
        }

        // 验证Agent指导文件表结构
        if let Err(e) = self.validate_agent_guides_schema().await {
            println!("❌ Agent指导文件表Schema验证失败: {}", e);
            all_compatible = false;
        } else {
            println!("✅ Agent指导文件表Schema兼容");
        }

        // 验证MCP服务器表结构
        if let Err(e) = self.validate_mcp_servers_schema().await {
            println!("❌ MCP服务器表Schema验证失败: {}", e);
            all_compatible = false;
        } else {
            println!("✅ MCP服务器表Schema兼容");
        }

        // 验证通用配置表结构
        if let Err(e) = self.validate_common_configs_schema().await {
            println!("❌ 通用配置表Schema验证失败: {}", e);
            all_compatible = false;
        } else {
            println!("✅ 通用配置表Schema兼容");
        }

        Ok(all_compatible)
    }

    /// 验证Claude供应商表结构
    async fn validate_claude_providers_schema(&self) -> Result<(), DataCompatibilityError> {
        let query = r#"
            PRAGMA table_info(claude_providers)
        "#;

        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DataCompatibilityError::Database(format!("查询Claude供应商表结构失败: {}", e)))?;

        // 验证必需的字段存在
        let required_fields = vec![
            ("id", "INTEGER"),
            ("name", "TEXT"),
            ("url", "TEXT"),
            ("token", "TEXT"),
            ("timeout", "INTEGER"),
            ("auto_update", "INTEGER"),
            ("type", "TEXT"),
            ("enabled", "INTEGER"),
            ("opus_model", "TEXT"),
            ("sonnet_model", "TEXT"),
            ("haiku_model", "TEXT"),
            ("created_at", "TEXT"),
            ("updated_at", "TEXT"),
        ];

        for (field_name, field_type) in required_fields {
            let field_exists = rows.iter().any(|row| {
                let name: String = row.get("name");
                let dtype: String = row.get("type");
                name == field_name && dtype.contains(field_type)
            });

            if !field_exists {
                return Err(DataCompatibilityError::DataValidation(
                    format!("Claude供应商表缺少必需字段: {} ({})", field_name, field_type)
                ));
            }
        }

        Ok(())
    }

    /// 验证Codex供应商表结构
    async fn validate_codex_providers_schema(&self) -> Result<(), DataCompatibilityError> {
        let query = r#"
            PRAGMA table_info(codex_providers)
        "#;

        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DataCompatibilityError::Database(format!("查询Codex供应商表结构失败: {}", e)))?;

        let required_fields = vec![
            ("id", "INTEGER"),
            ("name", "TEXT"),
            ("url", "TEXT"),
            ("token", "TEXT"),
            ("type", "TEXT"),
            ("enabled", "INTEGER"),
            ("created_at", "TEXT"),
            ("updated_at", "TEXT"),
        ];

        for (field_name, field_type) in required_fields {
            let field_exists = rows.iter().any(|row| {
                let name: String = row.get("name");
                let dtype: String = row.get("type");
                name == field_name && dtype.contains(field_type)
            });

            if !field_exists {
                return Err(DataCompatibilityError::DataValidation(
                    format!("Codex供应商表缺少必需字段: {} ({})", field_name, field_type)
                ));
            }
        }

        Ok(())
    }

    /// 验证Agent指导文件表结构
    async fn validate_agent_guides_schema(&self) -> Result<(), DataCompatibilityError> {
        let query = r#"
            PRAGMA table_info(agent_guides)
        "#;

        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DataCompatibilityError::Database(format!("查询Agent指导文件表结构失败: {}", e)))?;

        let required_fields = vec![
            ("id", "INTEGER"),
            ("name", "TEXT"),
            ("type", "TEXT"),
            ("text", "TEXT"),
            ("created_at", "TEXT"),
            ("updated_at", "TEXT"),
        ];

        for (field_name, field_type) in required_fields {
            let field_exists = rows.iter().any(|row| {
                let name: String = row.get("name");
                let dtype: String = row.get("type");
                name == field_name && dtype.contains(field_type)
            });

            if !field_exists {
                return Err(DataCompatibilityError::DataValidation(
                    format!("Agent指导文件表缺少必需字段: {} ({})", field_name, field_type)
                ));
            }
        }

        Ok(())
    }

    /// 验证MCP服务器表结构
    async fn validate_mcp_servers_schema(&self) -> Result<(), DataCompatibilityError> {
        let query = r#"
            PRAGMA table_info(mcp_servers)
        "#;

        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DataCompatibilityError::Database(format!("查询MCP服务器表结构失败: {}", e)))?;

        let required_fields = vec![
            ("id", "INTEGER"),
            ("name", "TEXT"),
            ("type", "TEXT"),
            ("timeout", "INTEGER"),
            ("command", "TEXT"),
            ("args", "TEXT"),
            ("env", "TEXT"),
            ("created_at", "TEXT"),
            ("updated_at", "TEXT"),
        ];

        for (field_name, field_type) in required_fields {
            let field_exists = rows.iter().any(|row| {
                let name: String = row.get("name");
                let dtype: String = row.get("type");
                name == field_name && dtype.contains(field_type)
            });

            if !field_exists {
                return Err(DataCompatibilityError::DataValidation(
                    format!("MCP服务器表缺少必需字段: {} ({})", field_name, field_type)
                ));
            }
        }

        Ok(())
    }

    /// 验证通用配置表结构
    async fn validate_common_configs_schema(&self) -> Result<(), DataCompatibilityError> {
        let query = r#"
            PRAGMA table_info(common_configs)
        "#;

        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DataCompatibilityError::Database(format!("查询通用配置表结构失败: {}", e)))?;

        let required_fields = vec![
            ("id", "INTEGER"),
            ("key", "TEXT"),
            ("value", "TEXT"),
            ("description", "TEXT"),
            ("category", "TEXT"),
            ("is_active", "INTEGER"),
            ("created_at", "TEXT"),
            ("updated_at", "TEXT"),
        ];

        for (field_name, field_type) in required_fields {
            let field_exists = rows.iter().any(|row| {
                let name: String = row.get("name");
                let dtype: String = row.get("type");
                name == field_name && dtype.contains(field_type)
            });

            if !field_exists {
                return Err(DataCompatibilityError::DataValidation(
                    format!("通用配置表缺少必需字段: {} ({})", field_name, field_type)
                ));
            }
        }

        Ok(())
    }

    /// 验证加密数据兼容性
    pub async fn validate_encryption_compatibility(&self) -> Result<bool, DataCompatibilityError> {
        println!("🔐 验证加密数据兼容性...");

        // 加载Python加密的测试数据
        let encrypted_data_path = "tests/data/python_encrypted_sample.json";
        let content = fs::read_to_string(encrypted_data_path)
            .map_err(|e| DataCompatibilityError::FileSystem(format!("读取加密测试数据失败: {}", e)))?;

        let encrypted_data: PythonDatabaseData = serde_json::from_str(&content)
            .map_err(|e| DataCompatibilityError::DataValidation(format!("加密测试数据解析失败: {}", e)))?;

        let mut all_compatible = true;

        // 验证Claude供应商token解密
        for provider in &encrypted_data.claude_providers {
            match self.crypto.decrypt(&provider.token) {
                Ok(decrypted) => {
                    println!("✅ Claude供应商 '{}' token解密成功", provider.name);
                }
                Err(e) => {
                    println!("❌ Claude供应商 '{}' token解密失败: {}", provider.name, e);
                    all_compatible = false;
                }
            }
        }

        // 验证Codex供应商token解密
        for provider in &encrypted_data.codex_providers {
            match self.crypto.decrypt(&provider.token) {
                Ok(decrypted) => {
                    println!("✅ Codex供应商 '{}' token解密成功", provider.name);
                }
                Err(e) => {
                    println!("❌ Codex供应商 '{}' token解密失败: {}", provider.name, e);
                    all_compatible = false;
                }
            }
        }

        Ok(all_compatible)
    }

    /// 验证数据迁移完整性
    pub async fn validate_migration_integrity(&self) -> Result<bool, DataCompatibilityError> {
        println!("🔄 验证数据迁移完整性...");

        let mut all_valid = true;

        // 验证Claude供应商数据
        if let Err(e) = self.validate_claude_providers_data().await {
            println!("❌ Claude供应商数据验证失败: {}", e);
            all_valid = false;
        } else {
            println!("✅ Claude供应商数据验证通过");
        }

        // 验证Codex供应商数据
        if let Err(e) = self.validate_codex_providers_data().await {
            println!("❌ Codex供应商数据验证失败: {}", e);
            all_valid = false;
        } else {
            println!("✅ Codex供应商数据验证通过");
        }

        // 验证Agent指导文件数据
        if let Err(e) = self.validate_agent_guides_data().await {
            println!("❌ Agent指导文件数据验证失败: {}", e);
            all_valid = false;
        } else {
            println!("✅ Agent指导文件数据验证通过");
        }

        // 验证MCP服务器数据
        if let Err(e) = self.validate_mcp_servers_data().await {
            println!("❌ MCP服务器数据验证失败: {}", e);
            all_valid = false;
        } else {
            println!("✅ MCP服务器数据验证通过");
        }

        // 验证通用配置数据
        if let Err(e) = self.validate_common_configs_data().await {
            println!("❌ 通用配置数据验证失败: {}", e);
            all_valid = false;
        } else {
            println!("✅ 通用配置数据验证通过");
        }

        Ok(all_valid)
    }

    /// 验证Claude供应商数据
    async fn validate_claude_providers_data(&self) -> Result<(), DataCompatibilityError> {
        let query = "SELECT COUNT(*) as count FROM claude_providers";
        let row = sqlx::query(query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DataCompatibilityError::Database(format!("查询Claude供应商数量失败: {}", e)))?;

        let count: i64 = row.get("count");
        let expected_count = self.test_data.claude_providers.len() as i64;

        if count != expected_count {
            return Err(DataCompatibilityError::DataValidation(
                format!("Claude供应商数量不匹配: 实际={}, 期望={}", count, expected_count)
            ));
        }

        // 验证数据一致性
        for python_provider in &self.test_data.claude_providers {
            let query = "SELECT * FROM claude_providers WHERE id = ?";
            let row = sqlx::query(query)
                .bind(python_provider.id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| DataCompatibilityError::Database(format!("查询Claude供应商数据失败: {}", e)))?;

            let name: String = row.get("name");
            let url: String = row.get("url");
            let enabled: i64 = row.get("enabled");

            if name != python_provider.name {
                return Err(DataCompatibilityError::DataValidation(
                    format!("Claude供应商名称不匹配: 实际={}, 期望={}", name, python_provider.name)
                ));
            }

            if url != python_provider.url {
                return Err(DataCompatibilityError::DataValidation(
                    format!("Claude供应商URL不匹配: 实际={}, 期望={}", url, python_provider.url)
                ));
            }

            if enabled != python_provider.enabled {
                return Err(DataCompatibilityError::DataValidation(
                    format!("Claude供应商启用状态不匹配: 实际={}, 期望={}", enabled, python_provider.enabled)
                ));
            }
        }

        Ok(())
    }

    /// 验证Codex供应商数据
    async fn validate_codex_providers_data(&self) -> Result<(), DataCompatibilityError> {
        let query = "SELECT COUNT(*) as count FROM codex_providers";
        let row = sqlx::query(query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DataCompatibilityError::Database(format!("查询Codex供应商数量失败: {}", e)))?;

        let count: i64 = row.get("count");
        let expected_count = self.test_data.codex_providers.len() as i64;

        if count != expected_count {
            return Err(DataCompatibilityError::DataValidation(
                format!("Codex供应商数量不匹配: 实际={}, 期望={}", count, expected_count)
            ));
        }

        Ok(())
    }

    /// 验证Agent指导文件数据
    async fn validate_agent_guides_data(&self) -> Result<(), DataCompatibilityError> {
        let query = "SELECT COUNT(*) as count FROM agent_guides";
        let row = sqlx::query(query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DataCompatibilityError::Database(format!("查询Agent指导文件数量失败: {}", e)))?;

        let count: i64 = row.get("count");
        let expected_count = self.test_data.agent_guides.len() as i64;

        if count != expected_count {
            return Err(DataCompatibilityError::DataValidation(
                format!("Agent指导文件数量不匹配: 实际={}, 期望={}", count, expected_count)
            ));
        }

        Ok(())
    }

    /// 验证MCP服务器数据
    async fn validate_mcp_servers_data(&self) -> Result<(), DataCompatibilityError> {
        let query = "SELECT COUNT(*) as count FROM mcp_servers";
        let row = sqlx::query(query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DataCompatibilityError::Database(format!("查询MCP服务器数量失败: {}", e)))?;

        let count: i64 = row.get("count");
        let expected_count = self.test_data.mcp_servers.len() as i64;

        if count != expected_count {
            return Err(DataCompatibilityError::DataValidation(
                format!("MCP服务器数量不匹配: 实际={}, 期望={}", count, expected_count)
            ));
        }

        Ok(())
    }

    /// 验证通用配置数据
    async fn validate_common_configs_data(&self) -> Result<(), DataCompatibilityError> {
        let query = "SELECT COUNT(*) as count FROM common_configs";
        let row = sqlx::query(query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DataCompatibilityError::Database(format!("查询通用配置数量失败: {}", e)))?;

        let count: i64 = row.get("count");
        let expected_count = self.test_data.common_configs.len() as i64;

        if count != expected_count {
            return Err(DataCompatibilityError::DataValidation(
                format!("通用配置数量不匹配: 实际={}, 期望={}", count, expected_count)
            ));
        }

        Ok(())
    }

    /// 执行完整的数据兼容性验证
    pub async fn run_full_compatibility_test(&self) -> Result<CompatibilityReport, DataCompatibilityError> {
        println!("🚀 开始完整的数据兼容性验证...");

        let mut report = CompatibilityReport::new();

        // 1. 验证Schema兼容性
        match self.validate_schema_compatibility().await {
            Ok(compatible) => {
                report.schema_compatible = compatible;
                if compatible {
                    println!("✅ 数据库Schema兼容性验证通过");
                } else {
                    println!("❌ 数据库Schema兼容性验证失败");
                }
            }
            Err(e) => {
                report.add_error("Schema验证", &e.to_string());
                println!("❌ Schema验证异常: {}", e);
            }
        }

        // 2. 验证加密兼容性
        match self.validate_encryption_compatibility().await {
            Ok(compatible) => {
                report.encryption_compatible = compatible;
                if compatible {
                    println!("✅ 加密数据兼容性验证通过");
                } else {
                    println!("❌ 加密数据兼容性验证失败");
                }
            }
            Err(e) => {
                report.add_error("加密验证", &e.to_string());
                println!("❌ 加密验证异常: {}", e);
            }
        }

        // 3. 验证迁移完整性
        match self.validate_migration_integrity().await {
            Ok(valid) => {
                report.data_integrity_valid = valid;
                if valid {
                    println!("✅ 数据迁移完整性验证通过");
                } else {
                    println!("❌ 数据迁移完整性验证失败");
                }
            }
            Err(e) => {
                report.add_error("完整性验证", &e.to_string());
                println!("❌ 完整性验证异常: {}", e);
            }
        }

        report.completed = true;
        println!("🎉 数据兼容性验证完成");

        Ok(report)
    }
}

/// 兼容性验证报告
#[derive(Debug, Serialize)]
pub struct CompatibilityReport {
    pub completed: bool,
    pub schema_compatible: bool,
    pub encryption_compatible: bool,
    pub data_integrity_valid: bool,
    pub errors: Vec<String>,
    pub test_summary: TestSummary,
}

#[derive(Debug, Serialize)]
pub struct TestSummary {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub success_rate: f64,
}

impl CompatibilityReport {
    pub fn new() -> Self {
        Self {
            completed: false,
            schema_compatible: false,
            encryption_compatible: false,
            data_integrity_valid: false,
            errors: Vec::new(),
            test_summary: TestSummary {
                total_tests: 3,
                passed_tests: 0,
                failed_tests: 0,
                success_rate: 0.0,
            },
        }
    }

    pub fn add_error(&mut self, test_name: &str, error: &str) {
        self.errors.push(format!("{}: {}", test_name, error));
    }

    pub fn calculate_summary(&mut self) {
        let mut passed = 0;
        if self.schema_compatible { passed += 1; }
        if self.encryption_compatible { passed += 1; }
        if self.data_integrity_valid { passed += 1; }

        self.test_summary.passed_tests = passed;
        self.test_summary.failed_tests = self.test_summary.total_tests - passed;
        self.test_summary.success_rate = (passed as f64) / (self.test_summary.total_tests as f64) * 100.0;
    }

    pub fn is_successful(&self) -> bool {
        self.completed && self.schema_compatible && self.encryption_compatible && self.data_integrity_valid
    }

    pub fn print_report(&self) {
        println!("\n📊 数据兼容性验证报告");
        println!("========================");
        println!("✅ 完成状态: {}", if self.completed { "已完成" } else { "未完成" });
        println!("🔍 Schema兼容性: {}", if self.schema_compatible { "✅ 通过" } else { "❌ 失败" });
        println!("🔐 加密兼容性: {}", if self.encryption_compatible { "✅ 通过" } else { "❌ 失败" });
        println!("🔄 数据完整性: {}", if self.data_integrity_valid { "✅ 通过" } else { "❌ 失败" });
        
        println!("\n📈 测试统计:");
        println!("总测试数: {}", self.test_summary.total_tests);
        println!("通过测试: {}", self.test_summary.passed_tests);
        println!("失败测试: {}", self.test_summary.failed_tests);
        println!("成功率: {:.1}%", self.test_summary.success_rate);

        if !self.errors.is_empty() {
            println!("\n❌ 错误详情:");
            for error in &self.errors {
                println!("  - {}", error);
            }
        }

        println!("\n🏆 总体结果: {}", 
            if self.is_successful() { "✅ 数据兼容性验证全部通过" } 
            else { "❌ 数据兼容性验证存在问题" }
        );
    }
}

/// 生成兼容性验证报告并保存到文件
pub async fn generate_compatibility_report(report: &CompatibilityReport) -> Result<(), DataCompatibilityError> {
    let report_json = serde_json::to_string_pretty(report)
        .map_err(|e| DataCompatibilityError::DataValidation(format!("报告序列化失败: {}", e)))?;

    let report_path = ".claude/data-compatibility-report.json";
    fs::write(report_path, report_json)
        .map_err(|e| DataCompatibilityError::FileSystem(format!("报告写入失败: {}", e)))?;

    println!("📄 兼容性验证报告已保存: {}", report_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::testing::generate_test_key;

    #[tokio::test]
    async fn test_data_compatibility_validation() {
        // 使用测试数据库
        let database_url = "sqlite::memory:";
        let encryption_key = generate_test_key();

        // 创建验证器
        let validator = DataCompatibilityValidator::new(database_url, encryption_key).await;
        assert!(validator.is_ok());

        let validator = validator.unwrap();

        // 运行兼容性验证
        let report = validator.run_full_compatibility_test().await;
        assert!(report.is_ok());

        let report = report.unwrap();
        report.print_report();

        // 验证报告是否成功
        assert!(report.completed);
    }
}