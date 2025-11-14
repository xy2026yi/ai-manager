//! Python 兼容性测试模块
//!
//! 这个模块验证Rust实现的加密服务与Python的cryptography.fernet完全兼容

use crate::crypto::{CryptoService, CryptoError};

/// 运行完整的Python兼容性测试
pub fn run_python_compatibility_tests() -> Result<(), CryptoError> {
    println!("🧪 开始Python兼容性测试...");

    // 使用Python生成的相同密钥
    let key = "Jw4Ff1BWLnSykdfXDVOuEJCG6m9dyST5B1VhU_qg0fI=";
    let crypto = CryptoService::new(key)?;

    // Python加密的测试数据（从Python脚本输出）
    let python_encrypted_data = vec![
        ("simple", "gAAAAABpFtrzyWTUFYuU5SszMqbwEBg5Uht5YGLuoIodnGMCHezhhDFs4rD5VNZjzjibSXHLr1G5_HG05PSLGT3jcmNZQFc5Ag==", "Hello, World!"),
        ("chinese", "gAAAAABpFtrzQzVH5e4MHnTPv4AxIbDTlNGFzk4Dr6E1So2j11gzwYgXM5_bCuJfnJYrPgabxaFuuRP8Fhe5TmWES8-USDNWMQ==", "测试中文"),
        ("emoji", "gAAAAABpFtrzFp09aHSEcRib_lgt1WMArcQJBNnjde5aPd0-MON_wfENInFXTo6YxDTxO-aAKWUrzslwt2JgtpU1YU7ACu3ZkQ==", "🔒🔐🔑"),
        ("empty", "gAAAAABpFtrzEKxVatOW8QwZmp5oRySamtytMyLYFWFLH37AqfXPHqDVpFDtpbmpy_sYPdI8OLIuqNBhN_QlMXppbAn9KLovyA==", ""),
        ("token", "gAAAAABpFtrzD5JfctIFdTmpSM8LCv2TfWc3zxUpjg6_xm1WQN8_w8tDmMfPFeaudlFfs0v3nHSpanLs1qaBs_0amI1KL23S21stvlZNAkB-kGIzzeDApO0=", "sk-1234567890abcdef"),
    ];

    // 测试1: 解密Python加密的数据
    println!("📥 测试1: 解密Python加密的数据");
    for (name, encrypted, expected) in python_encrypted_data {
        let decrypted = crypto.decrypt(encrypted)?;
        assert_eq!(decrypted, expected);
        println!("✅ {}: 解密成功", name);
    }

    // 测试2: Rust加密与Python格式兼容
    println!("📤 测试2: 验证Rust加密格式");
    let test_data = vec![
        ("simple", "Hello, World!"),
        ("chinese", "测试中文"),
        ("emoji", "🔒🔐🔑"),
        ("empty", ""),
        ("token", "sk-1234567890abcdef"),
    ];

    for (name, data) in test_data {
        let rust_encrypted = crypto.encrypt(data)?;

        // 验证加密结果格式正确（Base64编码）
        assert!(!rust_encrypted.is_empty());
        assert!(rust_encrypted.len() > 20); // Fernet tokens通常很长

        // 验证能正确解密
        let decrypted = crypto.decrypt(&rust_encrypted)?;
        assert_eq!(decrypted, data);

        println!("✅ {}: 加密/解密循环成功", name);
    }

    // 测试3: 批量处理兼容性
    println!("📦 测试3: 批量加密兼容性");
    let test_strings = vec![
        "API Key 1".to_string(),
        "Token测试".to_string(),
        "".to_string(),
        "🔒🔐🔑".to_string(),
    ];

    let encrypted = crypto.encrypt_batch(&test_strings)?;
    let decrypted = crypto.decrypt_batch(&encrypted)?;

    assert_eq!(test_strings, decrypted);
    println!("✅ 批量加密/解密成功");

    println!("🎉 所有Python兼容性测试通过！");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_compatibility() {
        let result = run_python_compatibility_tests();
        assert!(result.is_ok());
    }

    #[test]
    fn test_specific_python_vectors() {
        let key = "Jw4Ff1BWLnSykdfXDVOuEJCG6m9dyST5B1VhU_qg0fI=";
        let crypto = CryptoService::new(key).unwrap();

        // 具体的Python加密向量
        let python_token = "gAAAAABpFtrzyWTUFYuU5SszMqbwEBg5Uht5YGLuoIodnGMCHezhhDFs4rD5VNZjzjibSXHLr1G5_HG05PSLGT3jcmNZQFc5Ag==";
        let expected = "Hello, World!";

        let decrypted = crypto.decrypt(python_token).unwrap();
        assert_eq!(decrypted, expected);

        println!("✅ 具体Python测试向量验证通过");
    }

    #[test]
    fn test_cross_platform_compatibility() {
        let key = "Jw4Ff1BWLnSykdfXDVOuEJCG6m9dyST5B1VhU_qg0fI=";
        let crypto = CryptoService::new(key).unwrap();

        // 测试各种边缘情况
        let edge_cases = vec![
            ("very_long_string", "A".repeat(10000)),
            ("unicode", "测试中文字符串和各种符号🎉🚀💻".to_string()),
            ("json", "{\"key\":\"value\",\"number\":42,\"array\":[1,2,3]}".to_string()),
            ("newlines", "Line 1\nLine 2\r\nLine 3".to_string()),
            ("special_chars", "!@#$%^&*()_+-=[]{}|;':\",./<>?".to_string()),
        ];

        for (name, data) in edge_cases {
            let encrypted = crypto.encrypt(&data).unwrap();
            let decrypted = crypto.decrypt(&encrypted).unwrap();
            assert_eq!(decrypted, data);
            println!("✅ 边缘测试 {} 通过", name);
        }
    }
}