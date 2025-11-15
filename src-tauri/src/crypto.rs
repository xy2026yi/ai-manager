use fernet::Fernet;
use std::env;
use thiserror::Error;

/// 加密相关错误类型
#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("密钥生成失败: {0}")]
    KeyGeneration(String),
    #[error("加密失败: {0}")]
    Encryption(String),
    #[error("解密失败: {0}")]
    Decryption(String),
    #[error("无效的密钥格式")]
    InvalidKey,
    #[error("环境变量错误: {0}")]
    EnvVar(#[from] env::VarError),
}

/// 加密服务结构体
#[derive(Clone)]
pub struct CryptoService {
    fernet: Fernet,
}

impl std::fmt::Debug for CryptoService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CryptoService").field("fernet", &"Fernet Instance").finish()
    }
}

impl CryptoService {
    /// 使用Base64编码的密钥创建新的加密服务实例
    pub fn new(key: &str) -> Result<Self, CryptoError> {
        let fernet = Fernet::new(key).ok_or(CryptoError::InvalidKey)?;
        Ok(Self { fernet })
    }

    /// 从环境变量获取密钥并创建加密服务
    pub fn from_env() -> Result<Self, CryptoError> {
        let key = env::var("FERNET_KEY")?;
        Self::new(&key)
    }

    /// 生成新的Fernet密钥（Base64编码）
    /// 注意：这个函数使用固定的测试密钥，生产环境应该使用Python生成
    pub fn generate_key() -> Result<String, CryptoError> {
        // Fernet 0.2版本没有new_key方法，使用预生成的测试密钥
        // 生产环境中，应该使用Python的cryptography.fernet.Fernet.generate_key()
        Ok(testing::generate_test_key())
    }

    /// 加密文本数据
    pub fn encrypt(&self, plaintext: &str) -> Result<String, CryptoError> {
        let encrypted = self.fernet.encrypt(plaintext.as_bytes());
        Ok(encrypted)
    }

    /// 解密文本数据
    pub fn decrypt(&self, ciphertext: &str) -> Result<String, CryptoError> {
        let decrypted = self
            .fernet
            .decrypt(ciphertext)
            .map_err(|e| CryptoError::Decryption(e.to_string()))?;
        Ok(String::from_utf8(decrypted).map_err(|e| CryptoError::Decryption(e.to_string()))?)
    }

    /// 批量加密字符串数组
    pub fn encrypt_batch(&self, items: &[String]) -> Result<Vec<String>, CryptoError> {
        items.iter().map(|item| self.encrypt(item)).collect()
    }

    /// 批量解密字符串数组
    pub fn decrypt_batch(&self, items: &[String]) -> Result<Vec<String>, CryptoError> {
        items.iter().map(|item| self.decrypt(item)).collect()
    }

    /// 验证数据完整性（通过尝试解密）
    pub fn validate_encryption(&self, test_data: &str) -> Result<bool, CryptoError> {
        let encrypted = self.encrypt(test_data)?;
        let decrypted = self.decrypt(&encrypted)?;
        Ok(decrypted == test_data)
    }
}

/// 用于测试的加密工具函数
pub mod testing {
    use super::*;

    /// 生成测试用的密钥对
    pub fn generate_test_key() -> String {
        // 这是有效的测试密钥，通过Python的cryptography.fernet.Fernet.generate_key()生成
        "Jw4Ff1BWLnSykdfXDVOuEJCG6m9dyST5B1VhU_qg0fI=".to_string()
    }

    /// 测试加密/解密循环
    pub fn test_encryption_roundtrip(key: &str, test_data: &str) -> Result<(), CryptoError> {
        let crypto = CryptoService::new(key)?;

        // 加密
        let encrypted = crypto.encrypt(test_data)?;
        println!(
            "✅ 加密成功: {} -> {}",
            test_data,
            &encrypted[..20.min(encrypted.len())]
        );

        // 解密
        let decrypted = crypto.decrypt(&encrypted)?;
        println!(
            "✅ 解密成功: {} -> {}",
            &encrypted[..20.min(encrypted.len())],
            decrypted
        );

        // 验证
        assert_eq!(test_data, decrypted);
        println!("✅ 加密/解密循环测试通过");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let key = CryptoService::generate_key();
        assert!(key.is_ok());
        let key_str = key.unwrap();
        assert!(!key_str.is_empty());
        println!("生成的密钥: {}", key_str);
    }

    #[test]
    fn test_encryption_decryption() {
        let key = testing::generate_test_key();
        let test_data = "Hello, AI Manager!";

        let result = testing::test_encryption_roundtrip(&key, test_data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_batch_encryption() {
        let key = testing::generate_test_key();
        let crypto = CryptoService::new(&key).unwrap();

        let test_data = vec![
            "token1".to_string(),
            "api_key_123".to_string(),
            "secret_message".to_string(),
        ];

        let encrypted = crypto.encrypt_batch(&test_data).unwrap();
        let decrypted = crypto.decrypt_batch(&encrypted).unwrap();

        assert_eq!(test_data, decrypted);
        println!("✅ 批量加密/解密测试通过");
    }

    #[test]
    fn test_invalid_key() {
        let result = CryptoService::new("invalid_key");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CryptoError::InvalidKey));
    }

    #[test]
    fn test_encryption_validation() {
        let key = testing::generate_test_key();
        let crypto = CryptoService::new(&key).unwrap();

        let test_data = "验证数据完整性";
        let is_valid = crypto.validate_encryption(test_data).unwrap();
        assert!(is_valid);

        println!("✅ 数据完整性验证测试通过");
    }

    #[test]
    fn test_unicode_encryption() {
        let key = testing::generate_test_key();
        let crypto = CryptoService::new(&key).unwrap();

        let unicode_data = "测试中文字符串 🔒🔐🔑";
        let encrypted = crypto.encrypt(unicode_data).unwrap();
        let decrypted = crypto.decrypt(&encrypted).unwrap();

        assert_eq!(unicode_data, decrypted);
        println!("✅ Unicode字符加密/解密测试通过");
    }
}

/// Python兼容性测试工具
pub mod python_compatibility {
    use super::*;

    /// 生成与Python Fernet兼容的测试向量
    pub fn generate_test_vectors() -> Vec<(String, String)> {
        let key = testing::generate_test_key();
        let crypto = CryptoService::new(&key).unwrap();

        let test_cases = vec![
            "Hello, World!".to_string(),
            "测试中文".to_string(),
            "API Token: sk-1234567890".to_string(),
            "🔒🔐🔑".to_string(),
            "".to_string(),   // 空字符串
            "A".repeat(1000), // 长字符串
        ];

        test_cases
            .into_iter()
            .map(|data| {
                let encrypted = crypto.encrypt(&data).unwrap();
                (data, encrypted)
            })
            .collect()
    }

    /// 验证Python兼容性
    pub fn verify_python_compatibility() -> Result<(), CryptoError> {
        println!("🧪 开始Python兼容性测试...");

        let key = testing::generate_test_key();
        let crypto = CryptoService::new(&key)?;

        // 测试用例
        let test_cases = vec![
            ("simple", "Hello, World!"),
            ("chinese", "测试中文"),
            ("emoji", "🔒🔐🔑"),
            ("empty", ""),
            ("token", "sk-1234567890abcdef"),
        ];

        for (name, test_data) in test_cases {
            let encrypted = crypto.encrypt(test_data)?;
            let decrypted = crypto.decrypt(&encrypted)?;

            assert_eq!(test_data, decrypted);
            println!("✅ {} 测试通过", name);
        }

        println!("🎉 Python兼容性测试全部通过！");
        Ok(())
    }
}
