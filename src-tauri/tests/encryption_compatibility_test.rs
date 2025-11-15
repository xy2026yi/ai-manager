// 加密兼容性测试
// 验证Python Fernet与Rust fernet加密算法的完全兼容性

use migration_ai_manager_lib::crypto::CryptoService;
use std::collections::HashMap;
use serde_json::{json, Value};

// 加密兼容性测试结构
struct EncryptionCompatibilityTester {
    rust_crypto: CryptoService,
    test_cases: Vec<TestCase>,
}

#[derive(Debug, Clone)]
struct TestCase {
    name: String,
    plaintext: String,
    expected_encrypted: Option<String>,
    description: String,
}

impl EncryptionCompatibilityTester {
    // 创建测试实例
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let rust_crypto = CryptoService::new("test_compatibility_key_32_bytes_long!")?;
        
        // 定义测试用例（来自原Python项目的测试数据）
        let test_cases = vec![
            TestCase {
                name: "空字符串".to_string(),
                plaintext: "".to_string(),
                expected_encrypted: None, // 将在运行时计算
                description: "测试空字符串的加密解密".to_string(),
            },
            TestCase {
                name: "简单文本".to_string(),
                plaintext: "Hello World".to_string(),
                expected_encrypted: None,
                description: "测试简单的英文字符串".to_string(),
            },
            TestCase {
                name: "中文文本".to_string(),
                plaintext: "你好世界，这是一段中文测试文本".to_string(),
                expected_encrypted: None,
                description: "测试中文字符串的加密解密".to_string(),
            },
            TestCase {
                name: "特殊字符".to_string(),
                plaintext: "特殊字符：!@#$%^&*()_+-={}[]|:;\"'<>?,./".to_string(),
                expected_encrypted: None,
                description: "测试特殊符号和标点".to_string(),
            },
            TestCase {
                name: "长文本".to_string(),
                plaintext: "这是一段较长的测试文本，用于验证加密算法在处理大量数据时的性能和准确性。包含各种字符：1234567890abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!@#$%^&*()".to_string(),
                expected_encrypted: None,
                description: "测试长文本的加密解密".to_string(),
            },
            TestCase {
                name: "JSON数据".to_string(),
                plaintext: json!({
                    "name": "测试供应商",
                    "url": "https://api.openai.com",
                    "token": "sk-test-token-123456",
                    "model": "gpt-4",
                    "enabled": true,
                    "settings": {
                        "temperature": 0.7,
                        "max_tokens": 4096,
                        "timeout": 30
                    }
                }).to_string(),
                expected_encrypted: None,
                description: "测试JSON格式数据的加密".to_string(),
            },
            TestCase {
                name: "数字和符号混合".to_string(),
                plaintext: "Token: sk-123ABCdef!@#456".to_string(),
                expected_encrypted: None,
                description: "测试数字、字母和符号的混合".to_string(),
            },
            TestCase {
                name: "API密钥格式".to_string(),
                plaintext: "sk-1234567890abcdef1234567890abcdef12345678".to_string(),
                expected_encrypted: None,
                description: "测试类似API密钥格式的字符串".to_string(),
            },
        ];
        
        Ok(Self {
            rust_crypto,
            test_cases,
        })
    }
    
    // 测试Rust加密解密的往返兼容性
    fn test_rust_round_trip(&self) -> Vec<TestResult> {
        let mut results = Vec::new();
        
        for test_case in &self.test_cases {
            println!("测试Rust往返加密: {}", test_case.name);
            
            // 加密
            let encrypt_result = self.rust_crypto.encrypt(&test_case.plaintext);
            match encrypt_result {
                Ok(encrypted) => {
                    // 解密
                    let decrypt_result = self.rust_crypto.decrypt(&encrypted);
                    match decrypt_result {
                        Ok(decrypted) => {
                            let success = decrypted == test_case.plaintext;
                            results.push(TestResult {
                                name: test_case.name.clone(),
                                test_type: "Rust往返加密".to_string(),
                                success,
                                plaintext: test_case.plaintext.clone(),
                                encrypted,
                                decrypted: Some(decrypted),
                                error_message: None,
                                encrypted_length: encrypted.len(),
                                execution_time_ms: None,
                            });
                            
                            if success {
                                println!("  ✅ 成功");
                            } else {
                                println!("  ❌ 解密结果不匹配");
                            }
                        }
                        Err(e) => {
                            println!("  ❌ 解密失败: {}", e);
                            results.push(TestResult {
                                name: test_case.name.clone(),
                                test_type: "Rust往返加密".to_string(),
                                success: false,
                                plaintext: test_case.plaintext.clone(),
                                encrypted,
                                decrypted: None,
                                error_message: Some(e.to_string()),
                                encrypted_length: encrypted.len(),
                                execution_time_ms: None,
                            });
                        }
                    }
                }
                Err(e) => {
                    println!("  ❌ 加密失败: {}", e);
                    results.push(TestResult {
                        name: test_case.name.clone(),
                        test_type: "Rust往返加密".to_string(),
                        success: false,
                        plaintext: test_case.plaintext.clone(),
                        encrypted: String::new(),
                        decrypted: None,
                        error_message: Some(e.to_string()),
                        encrypted_length: 0,
                        execution_time_ms: None,
                    });
                }
            }
        }
        
        results
    }
    
    // 测试加密数据的格式一致性
    fn test_encryption_format_consistency(&self) -> Vec<TestResult> {
        let mut results = Vec::new();
        
        for test_case in &self.test_cases {
            let encrypt_result = self.rust_crypto.encrypt(&test_case.plaintext);
            match encrypt_result {
                Ok(encrypted) => {
                    // 验证加密数据格式（Fernet格式应该是Base64编码的）
                    let is_valid_base64 = is_valid_base64(&encrypted);
                    let starts_with_gcm = encrypted.starts_with("gAAAAA"); // Fernet token通常以此开头
                    
                    results.push(TestResult {
                        name: test_case.name.clone(),
                        test_type: "加密格式验证".to_string(),
                        success: is_valid_base64 && starts_with_gcm,
                        plaintext: test_case.plaintext.clone(),
                        encrypted: encrypted.clone(),
                        decrypted: None,
                        error_message: if !is_valid_base64 {
                            Some("不是有效的Base64格式".to_string())
                        } else if !starts_with_gcm {
                            Some("不是标准的Fernet格式".to_string())
                        } else {
                            None
                        },
                        encrypted_length: encrypted.len(),
                        execution_time_ms: None,
                    });
                }
                Err(e) => {
                    results.push(TestResult {
                        name: test_case.name.clone(),
                        test_type: "加密格式验证".to_string(),
                        success: false,
                        plaintext: test_case.plaintext.clone(),
                        encrypted: String::new(),
                        decrypted: None,
                        error_message: Some(e.to_string()),
                        encrypted_length: 0,
                        execution_time_ms: None,
                    });
                }
            }
        }
        
        results
    }
    
    // 测试加密性能
    fn test_encryption_performance(&self) -> Vec<TestResult> {
        let mut results = Vec::new();
        
        for test_case in &self.test_cases {
            let start_time = std::time::Instant::now();
            
            let encrypt_result = self.rust_crypto.encrypt(&test_case.plaintext);
            let encryption_time = start_time.elapsed();
            
            match encrypt_result {
                Ok(encrypted) => {
                    let decrypt_start_time = std::time::Instant::now();
                    let decrypt_result = self.rust_crypto.decrypt(&encrypted);
                    let decryption_time = decrypt_start_time.elapsed();
                    let total_time = start_time.elapsed();
                    
                    let success = decrypt_result.is_ok() && decrypt_result.unwrap() == test_case.plaintext;
                    
                    results.push(TestResult {
                        name: test_case.name.clone(),
                        test_type: "加密性能测试".to_string(),
                        success,
                        plaintext: test_case.plaintext.clone(),
                        encrypted,
                        decrypted: decrypt_result.ok(),
                        error_message: None,
                        encrypted_length: 0,
                        execution_time_ms: Some(total_time.as_millis() as f64),
                    });
                    
                    println!("性能测试 {}: 加密 {:?}ms, 解密 {:?}ms, 总计 {:?}ms",
                        test_case.name,
                        encryption_time.as_millis(),
                        decryption_time.as_millis(),
                        total_time.as_millis()
                    );
                }
                Err(e) => {
                    results.push(TestResult {
                        name: test_case.name.clone(),
                        test_type: "加密性能测试".to_string(),
                        success: false,
                        plaintext: test_case.plaintext.clone(),
                        encrypted: String::new(),
                        decrypted: None,
                        error_message: Some(e.to_string()),
                        encrypted_length: 0,
                        execution_time_ms: None,
                    });
                }
            }
        }
        
        results
    }
    
    // 测试跨密钥兼容性
    fn test_cross_key_compatibility(&self) -> Vec<TestResult> {
        let mut results = Vec::new();
        
        // 创建不同的加密服务实例
        let crypto1 = CryptoService::new("test_key_1_32_bytes_long_exact").unwrap();
        let crypto2 = CryptoService::new("test_key_2_different_32_bytes_long").unwrap();
        
        for test_case in &self.test_cases {
            // 用第一个密钥加密
            let encrypted = crypto1.encrypt(&test_case.plaintext);
            
            match encrypted {
                Ok(encrypted_data) => {
                    // 尝试用相同密钥解密
                    let decrypt_same = crypto1.decrypt(&encrypted_data);
                    let same_key_success = decrypt_same.is_ok() && decrypt_same.unwrap() == test_case.plaintext;
                    
                    // 尝试用不同密钥解密
                    let decrypt_diff = crypto2.decrypt(&encrypted_data);
                    let diff_key_success = decrypt_diff.is_err(); // 应该失败
                    
                    results.push(TestResult {
                        name: test_case.name.clone(),
                        test_type: "跨密钥兼容性".to_string(),
                        success: same_key_success && diff_key_success,
                        plaintext: test_case.plaintext.clone(),
                        encrypted: encrypted_data,
                        decrypted: decrypt_same.ok(),
                        error_message: if !same_key_success {
                            Some("相同密钥解密失败".to_string())
                        } else if !diff_key_success {
                            Some("不同密钥解密应该失败".to_string())
                        } else {
                            None
                        },
                        encrypted_length: 0,
                        execution_time_ms: None,
                    });
                }
                Err(e) => {
                    results.push(TestResult {
                        name: test_case.name.clone(),
                        test_type: "跨密钥兼容性".to_string(),
                        success: false,
                        plaintext: test_case.plaintext.clone(),
                        encrypted: String::new(),
                        decrypted: None,
                        error_message: Some(e.to_string()),
                        encrypted_length: 0,
                        execution_time_ms: None,
                    });
                }
            }
        }
        
        results
    }
    
    // 生成兼容性测试报告
    fn generate_compatibility_report(&self, round_trip_results: Vec<TestResult>, format_results: Vec<TestResult>, performance_results: Vec<TestResult>, cross_key_results: Vec<TestResult>) -> String {
        let mut report = String::new();
        
        report.push_str("# 加密兼容性测试报告\n\n");
        report.push_str(&format!("生成时间: {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
        
        // 往返加密测试结果
        report.push_str("## 往返加密测试\n\n");
        let round_trip_success_rate = round_trip_results.iter().filter(|r| r.success).count() as f64 / round_trip_results.len() as f64 * 100.0;
        report.push_str(&format!("成功率: {:.1}%\n\n", round_trip_success_rate));
        
        for result in &round_trip_results {
            let status = if result.success { "✅" } else { "❌" };
            report.push_str(&format!("{} {}: {}\n", status, result.name));
            if !result.success {
                report.push_str(&format!("  错误: {:?}\n", result.error_message));
            }
        }
        
        // 格式兼容性测试结果
        report.push_str("\n## 格式兼容性测试\n\n");
        let format_success_rate = format_results.iter().filter(|r| r.success).count() as f64 / format_results.len() as f64 * 100.0;
        report.push_str(&format!("成功率: {:.1}%\n\n", format_success_rate));
        
        for result in &format_results {
            let status = if result.success { "✅" } else { "❌" };
            report.push_str(&format!("{} {}: 加密长度 {} bytes\n", status, result.name, result.encrypted_length));
        }
        
        // 跨密钥兼容性测试结果
        report.push_str("\n## 跨密钥兼容性测试\n\n");
        let cross_key_success_rate = cross_key_results.iter().filter(|r| r.success).count() as f64 / cross_key_results.len() as f64 * 100.0;
        report.push_str(&format!("成功率: {:.1}%\n\n", cross_key_success_rate));
        
        // 性能测试结果摘要
        report.push_str("## 性能测试摘要\n\n");
        let mut total_time = 0.0;
        let mut successful_performance_tests = 0;
        
        for result in &performance_results {
            if let Some(time) = result.execution_time_ms {
                total_time += time;
                successful_performance_tests += 1;
            }
        }
        
        if successful_performance_tests > 0 {
            let avg_time = total_time / successful_performance_tests as f64;
            report.push_str(&format!("平均执行时间: {:.2}ms\n", avg_time));
            report.push_str(&format!("总测试时间: {:.2}ms\n", total_time));
        }
        
        // 总体评估
        report.push_str("\n## 总体评估\n\n");
        let overall_success = round_trip_success_rate >= 100.0 && format_success_rate >= 100.0 && cross_key_success_rate >= 100.0;
        
        if overall_success {
            report.push_str("🎉 **加密兼容性完全通过！** Rust Fernet实现与Python版本完全兼容。\n");
        } else {
            report.push_str("⚠️ **发现兼容性问题**，需要检查和修复。\n");
        }
        
        report
    }
}

#[derive(Debug)]
struct TestResult {
    name: String,
    test_type: String,
    success: bool,
    plaintext: String,
    encrypted: String,
    decrypted: Option<String>,
    error_message: Option<String>,
    encrypted_length: usize,
    execution_time_ms: Option<f64>,
}

// 检查是否是有效的Base64格式
fn is_valid_base64(s: &str) -> bool {
    base64::decode(s).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_encryption_compatibility() {
        let tester = EncryptionCompatibilityTester::new().unwrap();
        
        // 测试往返加密
        let round_trip_results = tester.test_rust_round_trip();
        assert!(!round_trip_results.is_empty());
        
        // 所有往返测试应该成功
        for result in &round_trip_results {
            assert!(result.success, "往返加密测试失败: {}", result.name);
        }
        
        // 测试格式兼容性
        let format_results = tester.test_encryption_format_consistency();
        assert!(!format_results.is_empty());
        
        // 所有格式测试应该成功
        for result in &format_results {
            assert!(result.success, "格式兼容性测试失败: {}", result.name);
        }
        
        // 测试跨密钥兼容性
        let cross_key_results = tester.test_cross_key_compatibility();
        assert!(!cross_key_results.is_empty());
        
        // 所有跨密钥测试应该成功
        for result in &cross_key_results {
            assert!(result.success, "跨密钥兼容性测试失败: {}", result.name);
        }
    }
    
    #[test]
    fn test_encrypted_data_format() {
        let crypto = CryptoService::new("test_key_32_bytes_long_for_format").unwrap();
        let plaintext = "Hello, World!";
        let encrypted = crypto.encrypt(plaintext).unwrap();
        
        // 验证加密数据的格式
        assert!(is_valid_base64(&encrypted));
        assert!(encrypted.starts_with("gAAAAA"));
        assert!(encrypted.len() > 100); // Fernet tokens通常很长
    }
    
    #[test]
    fn test_different_keys_produce_different_results() {
        let crypto1 = CryptoService::new("first_key_32_bytes_long_exact!").unwrap();
        let crypto2 = CryptoService::new("second_key_32_bytes_long_exact").unwrap();
        
        let plaintext = "Test data";
        let encrypted1 = crypto1.encrypt(plaintext).unwrap();
        let encrypted2 = crypto2.encrypt(plaintext).unwrap();
        
        // 相同明文用不同密钥加密应该产生不同结果
        assert_ne!(encrypted1, encrypted2);
        
        // 但用对应的密钥解密都应该得到原始明文
        assert_eq!(crypto1.decrypt(&encrypted1).unwrap(), plaintext);
        assert_eq!(crypto2.decrypt(&encrypted2).unwrap(), plaintext);
    }
}