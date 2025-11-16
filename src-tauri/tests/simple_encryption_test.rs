//! 简化的加密兼容性测试
//!
//! 专注于验证与Python Fernet的兼容性

use migration_ai_manager_lib::crypto::{python_compatibility, CryptoService};

#[tokio::test]
async fn test_python_fernet_compatibility() {
    println!("🔐 测试Python Fernet兼容性...");

    // 验证Python兼容性
    let result = python_compatibility::verify_python_compatibility();
    assert!(result.is_ok(), "Python兼容性测试应该通过");

    println!("✅ Python Fernet兼容性测试通过");
}

#[tokio::test]
async fn test_encryption_roundtrip() {
    println!("🔄 测试加密往返...");

    let crypto_service = CryptoService::new("Jw4Ff1BWLnSykdfXDVOuEJCG6m9dyST5B1VhU_qg0fI=")
        .expect("加密服务创建应该成功");

    // 测试用例
    let _binding = "A".repeat(1000);
    let test_cases = vec![
        "sk-ant-api03-test-key-1",
        "sk-test-openai-key-1",
        "测试中文token",
        "🔒🔐🔑",
        "",
        "binding",
    ];

    for (i, original) in test_cases.iter().enumerate() {
        println!("测试用例 {}: {} chars ", i + 1, original.len());

        // 加密
        let encrypted = crypto_service.encrypt(original).expect("加密应该成功");

        // 验证加密格式
        assert!(encrypted.starts_with("gAAAA"), "加密结果应该以gAAAA开头");

        // 解密
        let decrypted = crypto_service.decrypt(&encrypted).expect("解密应该成功");

        // 验证往返一致性
        assert_eq!(original, &decrypted, "加密往返应该保持数据一致");

        println!("  ✅ 往返测试通过 ");
    }

    println!("✅ 加密往返测试通过 ");
}

#[tokio::test]
async fn test_cross_platform_vectors() {
    println!("🌐 测试跨平台加密向量...");

    // 生成测试向量
    let test_vectors = python_compatibility::generate_test_vectors();

    println!("生成了 {} 个测试向量", test_vectors.len());

    let crypto_service = CryptoService::new("Jw4Ff1BWLnSykdfXDVOuEJCG6m9dyST5B1VhU_qg0fI=")
        .expect("加密服务创建应该成功");

    for (i, (original, encrypted)) in test_vectors.iter().enumerate() {
        println!(
            "验证向量 {}: {} -> {} chars",
            i + 1,
            original.len(),
            encrypted.len()
        );

        // 验证可以解密预加密的数据
        let decrypted = crypto_service.decrypt(encrypted).expect("测试向量解密应该成功");

        assert_eq!(original, &decrypted, "测试向量解密结果应该一致");

        println!("  ✅ 向量验证通过");
    }

    println!("✅ 跨平台加密向量测试通过");
}

#[test]
fn test_encryption_performance() {
    println!("⚡ 测试加密性能...");

    let crypto_service = CryptoService::new("Jw4Ff1BWLnSykdfXDVOuEJCG6m9dyST5B1VhU_qg0fI=")
        .expect("加密服务创建应该成功");

    // 性能测试
    let test_data = "sk-ant-test-performance-key-1234567890";
    let iterations = 100;

    let start_time = std::time::Instant::now();

    for _ in 0..iterations {
        let encrypted = crypto_service.encrypt(test_data).expect("加密应该成功");
        let _decrypted = crypto_service.decrypt(&encrypted).expect("解密应该成功");
    }

    let duration = start_time.elapsed();
    let avg_time = duration.as_millis() / iterations as u128;

    println!("平均每次加密解密耗时: {}ms", avg_time);

    // 性能断言（应该足够快）
    assert!(avg_time < 10, "每次操作应该少于10ms");

    println!("✅ 加密性能测试通过");
}

#[test]
fn test_unicode_handling() {
    println!("🌍 测试Unicode处理...");

    let crypto_service = CryptoService::new("Jw4Ff1BWLnSykdfXDVOuEJCG6m9dyST5B1VhU_qg0fI=")
        .expect("加密服务创建应该成功");

    let unicode_cases = vec![
        "测试中文",
        "Test English",
        "Тест русский",
        "テスト日本語",
        "🔒🔐🔑",
        "Mixed 测试🔐English",
    ];

    for (i, test_case) in unicode_cases.iter().enumerate() {
        println!("Unicode测试 {}: {}", i + 1, test_case);

        let encrypted = crypto_service.encrypt(test_case).expect("Unicode加密应该成功");

        let decrypted = crypto_service.decrypt(&encrypted).expect("Unicode解密应该成功");

        assert_eq!(test_case, &decrypted, "Unicode应该正确处理");

        println!("  ✅ Unicode测试通过");
    }

    println!("✅ Unicode处理测试通过");
}
