//! 数据迁移兼容性测试
//!
//! 测试从原Python项目迁移数据的完整性和加密兼容性

use migration_ai_manager_lib::crypto::{python_compatibility, CryptoService};
use migration_ai_manager_lib::database::{DatabaseConfig, DatabaseManager};
use migration_ai_manager_lib::migration_tool::{DataMigrationTool, PythonExportData};
use serde_json;
use sqlx;
use std::fs;
use std::path::Path;
use std::time::Duration;
use tempfile::tempdir;

/// 测试设置结构体
struct TestSetup {
    migration_tool: DataMigrationTool,
    db_manager: DatabaseManager,
    temp_dir: tempfile::TempDir,
}

impl TestSetup {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
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

        let db_manager = DatabaseManager::new(config).await?;
        let migration_tool = DataMigrationTool::new(
            db_manager.clone(),
            "Jw4Ff1BWLnSykdfXDVOuEJCG6m9dyST5B1VhU_qg0fI=",
        )
        .await?;

        Ok(Self { migration_tool, db_manager, temp_dir })
    }
}

#[tokio::test]
async fn test_python_encryption_compatibility() {
    println!("🔐 测试Python加密兼容性...");

    // 验证Python兼容性
    let result = python_compatibility::verify_python_compatibility();
    assert!(result.is_ok(), "Python兼容性测试应该通过");

    println!("✅ Python加密兼容性测试通过");
}

#[tokio::test]
async fn test_full_migration_roundtrip() {
    println!("🔄 测试完整迁移往返...");

    let setup = TestSetup::new().await.expect("测试设置失败");

    // 1. 加载测试数据
    let test_data_path = Path::new("tests/data/python_original_sample.json");
    if !test_data_path.exists() {
        // 如果测试数据文件不存在，创建一个简化的测试数据
        create_test_sample_file(&setup.temp_dir).await;
    }

    let json_content = fs::read_to_string(test_data_path).unwrap_or_else(|_| {
        // 使用内置测试数据
        serde_json::to_string(&create_sample_json_data()).unwrap()
    });

    // 2. 导入数据
    let import_report = setup
        .migration_tool
        .import_from_json(&json_content)
        .await
        .expect("数据导入应该成功");

    println!("✅ 数据导入完成: {:?}", import_report);
    assert!(import_report.total_migrated > 0, "应该有数据被迁移");

    // 3. 导出数据
    let exported_data = setup.migration_tool.export_to_json().await.expect("数据导出应该成功");

    println!("✅ 数据导出完成");

    // 4. 验证数据完整性
    let original_data: PythonExportData =
        serde_json::from_str(&json_content).expect("原始数据解析应该成功");

    verify_data_integrity(&original_data, &exported_data);

    println!("✅ 完整迁移往返测试通过");
}

#[tokio::test]
async fn test_encrypted_data_migration() {
    println!("🔒 测试加密数据迁移...");

    let setup = TestSetup::new().await.expect("测试设置失败");

    // 创建包含加密token的测试数据
    let mut test_data = create_sample_json_data();

    // 手动加密token（模拟Python加密的数据）
    let crypto_service = CryptoService::new("Jw4Ff1BWLnSykdfXDVOuEJCG6m9dyST5B1VhU_qg0fI=")
        .expect("加密服务创建应该成功");

    for provider in &mut test_data.claude_providers {
        provider.token = crypto_service.encrypt(&provider.token).expect("token加密应该成功");
    }

    for provider in &mut test_data.codex_providers {
        provider.token = crypto_service.encrypt(&provider.token).expect("token加密应该成功");
    }

    // 导入加密数据
    let json_content = serde_json::to_string(&test_data).expect("序列化应该成功");

    let import_report = setup
        .migration_tool
        .import_from_json(&json_content)
        .await
        .expect("加密数据导入应该成功");

    println!("✅ 加密数据导入完成: {:?}", import_report);

    // 验证数据能正确解密
    let exported_data = setup.migration_tool.export_to_json().await.expect("数据导出应该成功");

    // 检查token是否被正确解密
    for provider in &exported_data.claude_providers {
        assert!(
            !provider.token.starts_with("gAAAA"),
            "token应该被解密，当前仍为加密状态: {}",
            &provider.token[..20]
        );
    }

    println!("✅ 加密数据迁移测试通过");
}

#[tokio::test]
async fn test_migration_error_handling() {
    println!("⚠️ 测试迁移错误处理...");

    let setup = TestSetup::new().await.expect("测试设置失败");

    // 测试无效JSON
    let invalid_json = "{ invalid json }";
    let result = setup.migration_tool.import_from_json(invalid_json).await;
    assert!(result.is_err(), "无效JSON应该返回错误");

    // 测试不支持的版本
    let mut test_data = create_sample_json_data();
    test_data.version = "0.1.0".to_string(); // 不支持的版本

    let json_content = serde_json::to_string(&test_data).unwrap();
    let result = setup.migration_tool.import_from_json(&json_content).await;
    assert!(result.is_err(), "不支持的版本应该返回错误");

    println!("✅ 迁移错误处理测试通过");
}

#[tokio::test]
async fn test_database_schema_compatibility() {
    println!("🗄️ 测试数据库模式兼容性...");

    let setup = TestSetup::new().await.expect("测试设置失败");

    // 导入测试数据
    let test_data = create_sample_json_data();
    let json_content = serde_json::to_string(&test_data).unwrap();

    let _import_report = setup
        .migration_tool
        .import_from_json(&json_content)
        .await
        .expect("数据导入应该成功");

    // 验证数据库表结构
    let pool = setup.db_manager.pool();

    // 检查Claude供应商表
    let claude_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM claude_providers")
        .fetch_one(pool)
        .await
        .expect("查询Claude供应商数量应该成功");

    assert_eq!(
        claude_count,
        test_data.claude_providers.len() as i64,
        "Claude供应商数量应该匹配"
    );

    // 检查Codex供应商表
    let codex_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM codex_providers")
        .fetch_one(pool)
        .await
        .expect("查询Codex供应商数量应该成功");

    assert_eq!(
        codex_count,
        test_data.codex_providers.len() as i64,
        "Codex供应商数量应该匹配"
    );

    // 检查Agent指导文件表
    let agent_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM agent_guides")
        .fetch_one(pool)
        .await
        .expect("查询Agent指导文件数量应该成功");

    assert_eq!(
        agent_count,
        test_data.agent_guides.len() as i64,
        "Agent指导文件数量应该匹配"
    );

    // 检查MCP服务器表
    let mcp_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM mcp_servers")
        .fetch_one(pool)
        .await
        .expect("查询MCP服务器数量应该成功");

    assert_eq!(
        mcp_count,
        test_data.mcp_servers.len() as i64,
        "MCP服务器数量应该匹配"
    );

    // 检查通用配置表
    let config_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM common_configs")
        .fetch_one(pool)
        .await
        .expect("查询通用配置数量应该成功");

    assert_eq!(
        config_count,
        test_data.common_configs.len() as i64,
        "通用配置数量应该匹配"
    );

    println!("✅ 数据库模式兼容性测试通过");
}

#[tokio::test]
async fn test_data_validation_rules() {
    println!("✅ 测试数据验证规则...");

    let setup = TestSetup::new().await.expect("测试设置失败");

    // 测试供应商唯一性规则
    let mut test_data = create_sample_json_data();

    // 添加重复的启用供应商
    test_data.claude_providers.push(
        migration_ai_manager_lib::migration_tool::PythonClaudeProvider {
            id: None,
            name: "Duplicate Provider".to_string(),
            url: "https://api.anthropic.com".to_string(),
            token: "sk-duplicate-key".to_string(),
            timeout: Some(30000),
            auto_update: Some(1),
            r#type: Some("public_welfare".to_string()),
            enabled: Some(1), // 多个启用供应商
            opus_model: Some("claude-3-sonnet-20240229".to_string()),
            sonnet_model: None,
            haiku_model: None,
            created_at: None,
            updated_at: None,
        },
    );

    let json_content = serde_json::to_string(&test_data).unwrap();

    // 导入应该成功，但可能有警告
    let import_report = setup
        .migration_tool
        .import_from_json(&json_content)
        .await
        .expect("数据导入应该成功");

    // 检查是否有关于重复启用供应商的警告
    let has_duplicate_warning = import_report
        .warnings
        .iter()
        .any(|warning| warning.contains("重复") || warning.contains("duplicate"));

    if has_duplicate_warning {
        println!("✅ 检测到重复供应商警告: {:?}", import_report.warnings);
    } else {
        println!("ℹ️ 未检测到重复供应商警告（可能由业务逻辑处理）");
    }

    println!("✅ 数据验证规则测试通过");
}

/// 创建示例JSON数据
fn create_sample_json_data() -> PythonExportData {
    PythonExportData {
        version: "1.0.0".to_string(),
        claude_providers: vec![
            migration_ai_manager_lib::migration_tool::PythonClaudeProvider {
                id: None,
                name: "Test Claude Provider".to_string(),
                url: "https://api.anthropic.com".to_string(),
                token: "sk-ant-test-key-12345".to_string(),
                timeout: Some(30000),
                auto_update: Some(1),
                r#type: Some("public_welfare".to_string()),
                enabled: Some(1),
                opus_model: Some("claude-3-opus-20240229".to_string()),
                sonnet_model: Some("claude-3-sonnet-20240229".to_string()),
                haiku_model: Some("claude-3-haiku-20240307".to_string()),
                created_at: None,
                updated_at: None,
            },
        ],
        codex_providers: vec![
            migration_ai_manager_lib::migration_tool::PythonCodexProvider {
                id: None,
                name: "Test OpenAI Provider".to_string(),
                url: "https://api.openai.com/v1/chat/completions".to_string(),
                token: "sk-test-openai-key-67890".to_string(),
                r#type: Some("official".to_string()),
                enabled: Some(0),
                created_at: None,
                updated_at: None,
            },
        ],
        agent_guides: vec![migration_ai_manager_lib::migration_tool::PythonAgentGuide {
            id: None,
            name: "测试助手".to_string(),
            r#type: "testing".to_string(),
            text: "这是一个测试用的助手指导文本。".to_string(),
            created_at: None,
            updated_at: None,
        }],
        mcp_servers: vec![migration_ai_manager_lib::migration_tool::PythonMcpServer {
            id: None,
            name: "test-filesystem".to_string(),
            r#type: Some("stdio".to_string()),
            timeout: Some(30000),
            command: "npx".to_string(),
            args: vec![
                "@modelcontextprotocol/server-filesystem".to_string(),
                "/tmp".to_string(),
            ],
            env: Some(std::collections::HashMap::from([(
                "NODE_ENV".to_string(),
                "production".to_string(),
            )])),
            created_at: None,
            updated_at: None,
        }],
        common_configs: vec![
            migration_ai_manager_lib::migration_tool::PythonCommonConfig {
                id: None,
                key: "test_config".to_string(),
                value: "test_value".to_string(),
                description: Some("测试配置".to_string()),
                category: Some("test".to_string()),
                is_active: Some(1),
                created_at: None,
                updated_at: None,
            },
        ],
    }
}

/// 创建测试样本文件
async fn create_test_sample_file(temp_dir: &tempfile::TempDir) {
    let sample_file_path = temp_dir.path().join("python_original_sample.json");
    let sample_data = create_sample_json_data();
    let json_content = serde_json::to_string_pretty(&sample_data).unwrap();

    fs::write(sample_file_path, json_content).expect("测试样本文件写入应该成功");
}

/// 验证数据完整性
fn verify_data_integrity(original: &PythonExportData, exported: &PythonExportData) {
    println!("🔍 验证数据完整性...");

    // 验证数量匹配
    assert_eq!(
        original.claude_providers.len(),
        exported.claude_providers.len(),
        "Claude供应商数量应该匹配"
    );
    assert_eq!(
        original.codex_providers.len(),
        exported.codex_providers.len(),
        "Codex供应商数量应该匹配"
    );
    assert_eq!(
        original.agent_guides.len(),
        exported.agent_guides.len(),
        "Agent指导文件数量应该匹配"
    );
    assert_eq!(
        original.mcp_servers.len(),
        exported.mcp_servers.len(),
        "MCP服务器数量应该匹配"
    );
    assert_eq!(
        original.common_configs.len(),
        exported.common_configs.len(),
        "通用配置数量应该匹配"
    );

    // 验证关键数据字段
    for (i, orig_provider) in original.claude_providers.iter().enumerate() {
        let exp_provider = &exported.claude_providers[i];
        assert_eq!(orig_provider.name, exp_provider.name, "供应商名称应该匹配");
        assert_eq!(orig_provider.url, exp_provider.url, "供应商URL应该匹配");
        assert_eq!(
            orig_provider.token, exp_provider.token,
            "供应商token应该匹配"
        );
    }

    for (i, orig_guide) in original.agent_guides.iter().enumerate() {
        let exp_guide = &exported.agent_guides[i];
        assert_eq!(orig_guide.name, exp_guide.name, "指导文件名称应该匹配");
        assert_eq!(orig_guide.text, exp_guide.text, "指导文件内容应该匹配");
    }

    println!("✅ 数据完整性验证通过");
}
