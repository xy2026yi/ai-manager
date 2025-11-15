//! 加密相关工具
//!
//! 提供常用的加密和安全处理函数

use crate::crypto::CryptoService;
use rand::{thread_rng, Rng};
use sha2::Digest;
use base64::{Engine as _, engine::general_purpose};
use std::collections::HashMap;

/// 生成随机盐值
pub fn generate_salt(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    let mut rng = thread_rng();

    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// 生成安全的随机ID
pub fn generate_secure_id(prefix: &str) -> String {
    let random_part = generate_salt(16);
    format!("{}_{}", prefix, random_part)
}

/// 计算字符串的SHA256哈希值
pub fn sha256_hash(input: &str) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// 验证哈希值
pub fn verify_hash(input: &str, hash: &str) -> bool {
    sha256_hash(input) == hash
}

/// 生成API密钥的哈希值（用于存储）
pub fn hash_api_key(api_key: &str, salt: &str) -> String {
    let combined = format!("{}:{}", api_key, salt);
    sha256_hash(&combined)
}

/// 验证API密钥
pub fn verify_api_key(api_key: &str, salt: &str, stored_hash: &str) -> bool {
    hash_api_key(api_key, salt) == stored_hash
}

/// 创建加密服务实例
pub fn create_crypto_service(key: &str) -> Result<CryptoService, String> {
    CryptoService::new(key).map_err(|e| format!("创建加密服务失败: {}", e))
}

/// 安全地比较两个字符串（防止时序攻击）
pub fn safe_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (byte_a, byte_b) in a.bytes().zip(b.bytes()) {
        result |= byte_a ^ byte_b;
    }

    result == 0
}

/// 生成临时密码
pub fn generate_temp_password(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789!@#$%^&*";
    let mut rng = thread_rng();

    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// 掩码Base64字符串
pub fn decode_base64(input: &str) -> Result<Vec<u8>, base64::DecodeError> {
    general_purpose::STANDARD.decode(input)
}

/// 编码Base64字符串
pub fn encode_base64(input: &[u8]) -> String {
    general_purpose::STANDARD.encode(input)
}

/// 创建安全的会话令牌
pub fn create_session_token(user_id: &str) -> String {
    let timestamp = chrono::Utc::now().timestamp();
    let random_part = generate_salt(32);
    let combined = format!("{}:{}:{}", user_id, timestamp, random_part);
    sha256_hash(&combined)
}

/// 验证会话令牌格式
pub fn validate_session_token(token: &str) -> bool {
    if token.len() != 64 {
        // SHA256哈希的标准长度
        return false;
    }

    // 检查是否只包含十六进制字符
    token.chars().all(|c| c.is_ascii_hexdigit())
}

/// 生成加密的数据映射
pub fn create_encrypted_map(
    crypto_service: &CryptoService,
    data: HashMap<String, String>,
) -> Result<HashMap<String, String>, String> {
    let mut encrypted_map = HashMap::new();

    for (key, value) in data {
        let encrypted_value =
            crypto_service.encrypt(&value).map_err(|e| format!("加密失败: {}", e))?;
        encrypted_map.insert(key, encrypted_value);
    }

    Ok(encrypted_map)
}

/// 解密数据映射
pub fn decrypt_map(
    crypto_service: &CryptoService,
    encrypted_data: HashMap<String, String>,
) -> Result<HashMap<String, String>, String> {
    let mut decrypted_map = HashMap::new();

    for (key, encrypted_value) in encrypted_data {
        let decrypted_value = crypto_service
            .decrypt(&encrypted_value)
            .map_err(|e| format!("解密失败: {}", e))?;
        decrypted_map.insert(key, decrypted_value);
    }

    Ok(decrypted_map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_salt() {
        let salt = generate_salt(16);
        assert_eq!(salt.len(), 16);
        assert!(salt.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_sha256_hash() {
        let input = "test";
        let hash1 = sha256_hash(input);
        let hash2 = sha256_hash(input);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA256输出长度
    }

    #[test]
    fn test_verify_hash() {
        let input = "test";
        let hash = sha256_hash(input);
        assert!(verify_hash(input, &hash));
        assert!(!verify_hash("different", &hash));
    }

    #[test]
    fn test_safe_compare() {
        assert!(safe_compare("hello", "hello"));
        assert!(!safe_compare("hello", "world"));
        assert!(!safe_compare("short", "longer"));
    }

    #[test]
    fn test_generate_temp_password() {
        let password = generate_temp_password(12);
        assert_eq!(password.len(), 12);
        assert!(password.chars().any(|c| "!@#$%^&*".contains(c)));
    }

    #[test]
    fn test_base64_encode_decode() {
        let original = b"hello world";
        let encoded = encode_base64(original);
        let decoded = decode_base64(&encoded).unwrap();
        assert_eq!(&original[..], &decoded[..]);
    }

    #[test]
    fn test_validate_session_token() {
        let valid_token = sha256_hash("test");
        assert!(validate_session_token(&valid_token));
        assert!(!validate_session_token("invalid"));
        assert!(!validate_session_token("short"));
    }
}
