//! 加密兼容性测试
//!
//! 验证与Python Fernet的完全兼容性，确保加密数据可以在两个平台间无缝迁移

use migration_ai_manager_lib::crypto::{python_compatibility, CryptoService};
use migration_ai_manager_lib::database::{DatabaseConfig, DatabaseManager};
use migration_ai_manager_lib::migration_tool::DataMigrationTool;
use serde_json;
use std::collections::HashMap;
use std::time::Duration;
use tempfile::tempdir;
use tracing::{error, info, warn};

#[tokio::test]
async fn test_python_fernet_compatibility() {
    println!("🔐 测试Python Fernet兼容性...");

    // 验证Python兼容性
    let result = python_compatibility::verify_python_compatibility();
    assert!(result.is_ok(), "Python Fernet兼容性测试应该通过");

    println!("✅ Python Fernet兼容性测试通过");
}

#[tokio::test]
async fn test_encrypted_token_roundtrip() {
    println!("🔄 测试加密token往返...");

    let crypto_service = CryptoService::new("Jw4Ff1BWLnSykdfXDVOuEJCG6m9dyST5B1VhU_qg0fI=")
        .expect("加密服务创建应该成功");

    // 模拟各种可能的token格式
    let test_tokens = vec![
        "sk-ant-api03-test-key-1".to_string(),
        "sk-test-openai-key-1".to_string(),
        "sk-1234567890abcdef".to_string(),
        "test-api-key-with-special-chars-!@#$%^&*()".to_string(),
        "测试中文token".to_string(),
        "🔒🔐🔑".to_string(),
        String::new(),               // 空token
        "A".repeat(1000), // 长token
    ];

    for (i, original_token) in test_tokens.iter().enumerate() {
        println!(
            "测试token {}: {}",
            i + 1,
            &original_token[..20.min(original_token.len())]
        );

        // 加密
        let encrypted_token = crypto_service.encrypt(original_token).expect("token加密应该成功");

        // 验证加密结果格式
        assert!(
            encrypted_token.starts_with("gAAAA"),
            "加密结果应该以gAAAA开头: {}",
            &encrypted_token[..10]
        );
        assert!(
            encrypted_token.len() > 100,
            "加密结果应该足够长: {}",
            encrypted_token.len()
        );

        // 解密
        let decrypted_token = crypto_service.decrypt(&encrypted_token).expect("token解密应该成功");

        // 验证往返一致性
        assert_eq!(
            original_token, &decrypted_token,
            "加密往返应该保持token不变"
        );

        println!("  ✅ 加密/解密往返成功");
    }

    println!("✅ 加密token往返测试通过");
}

#[tokio::test]
async fn test_cross_platform_encryption_vectors() {
    println!("🌐 测试跨平台加密向量...");

    // 生成与Python兼容的测试向量
    let test_vectors = python_compatibility::generate_test_vectors();

    println!("生成了 {} 个测试向量", test_vectors.len());

    for (i, (original, encrypted)) in test_vectors.iter().enumerate() {
        println!(
            "测试向量 {}: {} chars -> {} chars",
            i + 1,
            original.len(),
            encrypted.len()
        );

        // 验证加密向量可以被解密
        let crypto_service = CryptoService::new("Jw4Ff1BWLnSykdfXDVOuEJCG6m9dyST5B1VhU_qg0fI=")
            .expect("加密服务创建应该成功");

        let decrypted = crypto_service.decrypt(encrypted).expect("测试向量解密应该成功");

        assert_eq!(original, &decrypted, "测试向量解密结果应该与原始数据一致");

        println!("  ✅ 测试向量验证成功");
    }

    println!("✅ 跨平台加密向量测试通过");
}

#[tokio::test]
async fn test_migration_with_encrypted_data() {
    println!("📦 测试加密数据迁移...");

    // 设置测试环境
    let temp_dir = tempdir().expect("临时目录创建失败");
    let db_path = temp_dir.path().join("test_encrypted_migration.db");
    let db_url = format!("sqlite:{}", db_path.display());

    let config = DatabaseConfig {
        url: db_url,
        max_connections: 5,
        min_connections: 1,
        connect_timeout: Duration::from_secs(10),
        idle_timeout: Duration::from_secs(60),
        max_lifetime: Duration::from_secs(300),
    };

    let db_manager = DatabaseManager::new(config).await.expect("数据库管理器创建失败");

    let migration_tool = DataMigrationTool::new(
        db_manager.clone(),
        "Jw4Ff1BWLnSykdfXDVOuEJCG6m9dyST5B1VhU_qg0fI=",
    )
    .await
    .expect("迁移工具创建失败");

    // 创建包含预加密token的测试数据（模拟从Python导出的数据）
    let crypto_service = CryptoService::new("Jw4Ff1BWLnSykdfXDVOuEJCG6m9dyST5B1VhU_qg0fI=")
        .expect("加密服务创建失败");

    let mut test_data = create_encrypted_test_data(&crypto_service);

    // 导入预加密的数据
    let json_content = serde_json::to_string(&test_data).expect("JSON序列化失败");

    let import_report = migration_tool
        .import_from_json(&json_content)
        .await
        .expect("加密数据导入应该成功");

    println!("✅ 加密数据导入完成: {:?}", import_report);
    assert!(import_report.total_migrated > 0, "应该有数据被迁移");

    // 导出并验证数据能正确解密
    let exported_data = migration_tool.export_to_json().await.expect("数据导出应该成功");

    // 验证token被正确解密
    for provider in &exported_data.claude_providers {
        assert!(
            !provider.token.starts_with("gAAAA"),
            "Claude供应商token应该被解密: {}...",
            &provider.token[..20]
        );
    }

    for provider in &exported_data.codex_providers {
        assert!(
            !provider.token.starts_with("gAAAA"),
            "Codex供应商token应该被解密: {}...",
            &provider.token[..20]
        );
    }

    println!("✅ 加密数据迁移测试通过");
}

#[tokio::test]
async fn test_encryption_performance() {
    println!("⚡ 测试加密性能...");

    let crypto_service = CryptoService::new("Jw4Ff1BWLnSykdfXDVOuEJCG6m9dyST5B1VhU_qg0fI=")
        .expect("加密服务创建应该成功");

    // 测试批量加密性能
    let test_data: Vec<String> =
        (0..100).map(|i| format!("test-token-{:04}-sk-1234567890abcdef", i)).collect();

    println!("测试 {} 个token的批量加密性能...", test_data.len());

    let start_time = std::time::Instant::now();

    // 批量加密
    let encrypted_data = crypto_service.encrypt_batch(&test_data).expect("批量加密应该成功");

    let encrypt_duration = start_time.elapsed();
    println!("批量加密耗时: {:?}", encrypt_duration);

    // 批量解密
    let start_time = std::time::Instant::now();
    let decrypted_data = crypto_service.decrypt_batch(&encrypted_data).expect("批量解密应该成功");

    let decrypt_duration = start_time.elapsed();
    println!("批量解密耗时: {:?}", decrypt_duration);

    // 验证数据一致性
    assert_eq!(test_data, decrypted_data, "批量加密解密应该保持数据一致");

    // 性能基准
    let encrypt_per_item = encrypt_duration.as_millis() / test_data.len() as u128;
    let decrypt_per_item = decrypt_duration.as_millis() / test_data.len() as u128;

    println!("每个token加密耗时: {}ms", encrypt_per_item);
    println!("每个token解密耗时: {}ms", decrypt_per_item);

    // 性能断言（应该足够快）
    assert!(encrypt_per_item < 10, "每个token加密应该少于10ms");
    assert!(decrypt_per_item < 10, "每个token解密应该少于10ms");

    println!("✅ 加密性能测试通过");
}

#[tokio::test]
async fn test_encryption_error_handling() {
    println!("⚠️ 测试加密错误处理...");

    let crypto_service = CryptoService::new("Jw4Ff1BWLnSykdfXDVOuEJCG6m9dyST5B1VhU_qg0fI=")
        .expect("加密服务创建应该成功");

    // 测试无效密钥
    let invalid_crypto_result = CryptoService::new("invalid_key");
    assert!(invalid_crypto_result.is_err(), "无效密钥应该返回错误");

    // 测试解密无效数据
    let invalid_encrypted_data = vec![
        "invalid_encrypted_data".to_string(),
        "gAAAA".to_string(),                    // 太短
        "gAAAAinvaliddata".to_string(),         // 格式错误
        format!("gAAAA{}", "A".repeat(100)), // 长度正确但内容无效
    ];

    for invalid_data in invalid_encrypted_data {
        let decrypt_result = crypto_service.decrypt(&invalid_data);
        assert!(
            decrypt_result.is_err(),
            "解密无效数据应该返回错误: {}",
            invalid_data
        );
        println!(
            "  ✅ 无效数据正确拒绝: {}",
            &invalid_data[..20.min(invalid_data.len())]
        );
    }

    println!("✅ 加密错误处理测试通过");
}

#[tokio::test]
async fn test_unicode_encryption() {
    println!("🌍 测试Unicode加密...");

    let crypto_service = CryptoService::new("Jw4Ff1BWLnSykdfXDVOuEJCG6m9dyST5B1VhU_qg0fI=")
        .expect("加密服务创建应该成功");

    // 测试各种Unicode字符
    let unicode_test_cases = vec![
        "测试中文",
        "Test English",
        "Тест русский",
        "テスト日本語",
        "🔒🔐🔑",
        "Mixed 测试🔐English",
        "Special chars: !@#$%^&*()_+-=[]{}|;':\",./<>?",
        "Emojis: 😊😎🤖💻📱",
        "数学符号: ∑∏∫∆∇∂∞",
        "Currency: $¥€£₹₽₩",
    ];

    for (i, test_case) in unicode_test_cases.iter().enumerate() {
        println!("Unicode测试 {}: {}", i + 1, test_case);

        // 加密
        let encrypted = crypto_service.encrypt(test_case).expect("Unicode加密应该成功");

        // 解密
        let decrypted = crypto_service.decrypt(&encrypted).expect("Unicode解密应该成功");

        // 验证一致性
        assert_eq!(test_case, &decrypted, "Unicode加密解密应该保持数据一致");

        println!("  ✅ Unicode测试通过");
    }

    println!("✅ Unicode加密测试通过");
}

/// 创建包含加密token的测试数据
fn create_encrypted_test_data(crypto_service: &CryptoService) -> serde_json::Value {
    let mut test_data = serde_json::json!({
        "version": "1.0.0",
        "claude_providers": [
            {
                "id": null,
                "name": "Encrypted Claude Provider",
                "url": "https://api.anthropic.com",
                "token": "", // 将被加密
                "timeout": 30000,
                "auto_update": 1,
                "type": "public_welfare",
                "enabled": 1,
                "opus_model": "claude-3-opus-20240229",
                "sonnet_model": "claude-3-sonnet-20240229",
                "haiku_model": "claude-3-haiku-20240307",
                "created_at": null,
                "updated_at": null
            }
        ],
        "codex_providers": [
            {
                "id": null,
                "name": "Encrypted OpenAI Provider",
                "url": "https://api.openai.com/v1/chat/completions",
                "token": "", // 将被加密
                "type": "official",
                "enabled": 0,
                "created_at": null,
                "updated_at": null
            }
        ],
        "agent_guides": [
            {
                "id": null,
                "name": "测试助手",
                "type": "testing",
                "text": "这是一个测试用的助手指导文本，包含中文内容。",
                "created_at": null,
                "updated_at": null
            }
        ],
        "mcp_servers": [],
        "common_configs": []
    });

    // 加密token
    if let Some(claude_providers) = test_data["claude_providers"].as_array_mut() {
        if let Some(provider) = claude_providers.get_mut(0) {
            if let Some(token_obj) = provider.get_mut("token") {
                let original_token = "sk-ant-encrypted-test-key-12345";
                let encrypted_token =
                    crypto_service.encrypt(original_token).expect("token加密应该成功");
                *token_obj = serde_json::Value::String(encrypted_token);
            }
        }
    }

    if let Some(codex_providers) = test_data["codex_providers"].as_array_mut() {
        if let Some(provider) = codex_providers.get_mut(0) {
            if let Some(token_obj) = provider.get_mut("token") {
                let original_token = "sk-openai-encrypted-test-key-67890";
                let encrypted_token =
                    crypto_service.encrypt(original_token).expect("token加密应该成功");
                *token_obj = serde_json::Value::String(encrypted_token);
            }
        }
    }

    test_data
}
