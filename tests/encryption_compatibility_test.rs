//! 加密兼容性测试
//! 
//! 验证Python版本的Fernet加密与Rust版本的加密完全兼容

use std::collections::HashMap;
use crate::crypto::{CryptoService, CryptoError};
use serde::{Deserialize, Serialize};

/// 加密兼容性测试错误类型
#[derive(Debug)]
pub enum EncryptionCompatibilityError {
    RustEncryption(String),
    PythonEncryption(String),
    Validation(String),
    FileSystem(String),
}

impl std::fmt::Display for EncryptionCompatibilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncryptionCompatibilityError::RustEncryption(msg) => write!(f, "Rust加密错误: {}", msg),
            EncryptionCompatibilityError::PythonEncryption(msg) => write!(f, "Python加密错误: {}", msg),
            EncryptionCompatibilityError::Validation(msg) => write!(f, "验证错误: {}", msg),
            EncryptionCompatibilityError::FileSystem(msg) => write!(f, "文件系统错误: {}", msg),
        }
    }
}

impl std::error::Error for EncryptionCompatibilityError {}

/// 加密测试用例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionTestCase {
    pub name: String,
    pub plaintext: String,
    pub description: String,
}

/// 加密兼容性验证器
pub struct EncryptionCompatibilityValidator {
    crypto_service: CryptoService,
    test_key: String,
    python_encrypted_data: Option<PythonEncryptedTestData>,
}

/// Python加密的测试数据
#[derive(Debug, Serialize, Deserialize)]
pub struct PythonEncryptedTestData {
    pub version: String,
    pub test_cases: Vec<PythonEncryptedTestCase>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PythonEncryptedTestCase {
    pub name: String,
    pub plaintext: String,
    pub encrypted: String,
    pub decrypted: String,
    pub success: bool,
}

impl EncryptionCompatibilityValidator {
    /// 创建新的验证器实例
    pub fn new(key: &str) -> Result<Self, EncryptionCompatibilityError> {
        let crypto_service = CryptoService::new(key)
            .map_err(|e| EncryptionCompatibilityError::RustEncryption(format!("加密服务初始化失败: {}", e)))?;

        Ok(Self {
            crypto_service,
            test_key: key.to_string(),
            python_encrypted_data: None,
        })
    }

    /// 加载Python加密的测试数据
    pub async fn load_python_encrypted_data(&mut self) -> Result<(), EncryptionCompatibilityError> {
        let test_data_path = "tests/unit/data/python_encrypted_tokens.json";
        
        // 如果Python加密数据不存在，生成它
        if !std::path::Path::new(test_data_path).exists() {
            self.generate_python_encrypted_data(test_data_path).await?;
        }

        let content = std::fs::read_to_string(test_data_path)
            .map_err(|e| EncryptionCompatibilityError::FileSystem(format!("读取Python加密数据失败: {}", e)))?;

        let data: PythonEncryptedTestData = serde_json::from_str(&content)
            .map_err(|e| EncryptionCompatibilityError::Validation(format!("Python加密数据解析失败: {}", e)))?;

        self.python_encrypted_data = Some(data);
        Ok(())
    }

    /// 生成Python加密的测试数据
    async fn generate_python_encrypted_data(&self, output_path: &str) -> Result<(), EncryptionCompatibilityError> {
        println!("🔐 生成Python加密测试数据...");

        let test_cases = self.get_test_cases();
        let mut python_data = PythonEncryptedTestData {
            version: "1.0.0".to_string(),
            test_cases: Vec::new(),
        };

        for case in &test_cases {
            // 使用Python加密（通过调用Python脚本）
            let encrypted = self.encrypt_with_python(&case.plaintext, &self.test_key)
                .await?;
            
            let decrypted = self.decrypt_with_python(&encrypted, &self.test_key)
                .await?;

            let python_case = PythonEncryptedTestCase {
                name: case.name.clone(),
                plaintext: case.plaintext.clone(),
                encrypted,
                decrypted,
                success: true,
            };

            python_data.test_cases.push(python_case);
        }

        // 保存到文件
        let content = serde_json::to_string_pretty(&python_data)
            .map_err(|e| EncryptionCompatibilityError::Validation(format!("序列化Python加密数据失败: {}", e)))?;

        std::fs::write(output_path, content)
            .map_err(|e| EncryptionCompatibilityError::FileSystem(format!("写入Python加密数据失败: {}", e)))?;

        println!("✅ Python加密测试数据已生成: {}", output_path);
        Ok(())
    }

    /// 使用Python进行加密
    async fn encrypt_with_python(&self, plaintext: &str, key: &str) -> Result<String, EncryptionCompatibilityError> {
        let python_script = r#"
import sys
import json
from cryptography.fernet import Fernet

try:
    key = sys.argv[1]
    plaintext = sys.argv[2]
    
    fernet = Fernet(key)
    encrypted = fernet.encrypt(plaintext.encode()).decode()
    
    result = {"success": True, "encrypted": encrypted}
    print(json.dumps(result))
except Exception as e:
    result = {"success": False, "error": str(e)}
    print(json.dumps(result))
"#;

        // 创建临时Python脚本文件
        let script_path = "tests/unit/data/temp_encrypt.py";
        std::fs::write(script_path, python_script)
            .map_err(|e| EncryptionCompatibilityError::FileSystem(format!("创建Python脚本失败: {}", e)))?;

        // 执行Python脚本
        let output = std::process::Command::new("python3")
            .args(&[script_path, key, plaintext])
            .output()
            .map_err(|e| EncryptionCompatibilityError::PythonEncryption(format!("Python加密脚本执行失败: {}", e)))?;

        // 清理临时文件
        let _ = std::fs::remove_file(script_path);

        if !output.status.success() {
            return Err(EncryptionCompatibilityError::PythonEncryption(format!(
                "Python加密脚本执行失败: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let result: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
            .map_err(|e| EncryptionCompatibilityError::Validation(format!("Python加密结果解析失败: {}", e)))?;

        if let Some(success) = result.get("success").and_then(|v| v.as_bool()) {
            if success {
                if let Some(encrypted) = result.get("encrypted").and_then(|v| v.as_str()) {
                    return Ok(encrypted.to_string());
                }
            }
        }

        if let Some(error) = result.get("error").and_then(|v| v.as_str()) {
            return Err(EncryptionCompatibilityError::PythonEncryption(format!("Python加密失败: {}", error)));
        }

        Err(EncryptionCompatibilityError::PythonEncryption("Python加密返回无效结果".to_string()))
    }

    /// 使用Python进行解密
    async fn decrypt_with_python(&self, ciphertext: &str, key: &str) -> Result<String, EncryptionCompatibilityError> {
        let python_script = r#"
import sys
import json
from cryptography.fernet import Fernet

try:
    key = sys.argv[1]
    ciphertext = sys.argv[2]
    
    fernet = Fernet(key)
    decrypted = fernet.decrypt(ciphertext.encode()).decode()
    
    result = {"success": True, "decrypted": decrypted}
    print(json.dumps(result))
except Exception as e:
    result = {"success": False, "error": str(e)}
    print(json.dumps(result))
"#;

        // 创建临时Python脚本文件
        let script_path = "tests/unit/data/temp_decrypt.py";
        std::fs::write(script_path, python_script)
            .map_err(|e| EncryptionCompatibilityError::FileSystem(format!("创建Python脚本失败: {}", e)))?;

        // 执行Python脚本
        let output = std::process::Command::new("python3")
            .args(&[script_path, key, ciphertext])
            .output()
            .map_err(|e| EncryptionCompatibilityError::PythonEncryption(format!("Python解密脚本执行失败: {}", e)))?;

        // 清理临时文件
        let _ = std::fs::remove_file(script_path);

        if !output.status.success() {
            return Err(EncryptionCompatibilityError::PythonEncryption(format!(
                "Python解密脚本执行失败: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let result: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
            .map_err(|e| EncryptionCompatibilityError::Validation(format!("Python解密结果解析失败: {}", e)))?;

        if let Some(success) = result.get("success").and_then(|v| v.as_bool()) {
            if success {
                if let Some(decrypted) = result.get("decrypted").and_then(|v| v.as_str()) {
                    return Ok(decrypted.to_string());
                }
            }
        }

        if let Some(error) = result.get("error").and_then(|v| v.as_str()) {
            return Err(EncryptionCompatibilityError::PythonEncryption(format!("Python解密失败: {}", error)));
        }

        Err(EncryptionCompatibilityError::PythonEncryption("Python解密返回无效结果".to_string()))
    }

    /// 获取测试用例
    fn get_test_cases(&self) -> Vec<EncryptionTestCase> {
        vec![
            EncryptionTestCase {
                name: "简单英文".to_string(),
                plaintext: "Hello, World!".to_string(),
                description: "基本的英文文本".to_string(),
            },
            EncryptionTestCase {
                name: "中文字符".to_string(),
                plaintext: "测试中文字符串加密".to_string(),
                description: "包含中文字符的文本".to_string(),
            },
            EncryptionTestCase {
                name: "API Token".to_string(),
                plaintext: "sk-ant-api03-test-key-1234567890".to_string(),
                description: "模拟API Token".to_string(),
            },
            EncryptionTestCase {
                name: "JSON数据".to_string(),
                plaintext: r#"{"model": "claude-3-sonnet", "max_tokens": 4096}"#.to_string(),
                description: "JSON格式的配置数据".to_string(),
            },
            EncryptionTestCase {
                name: "特殊字符".to_string(),
                plaintext: "🔒🔐🔑 特殊符号 !@#$%^&*()".to_string(),
                description: "包含Emoji和特殊符号".to_string(),
            },
            EncryptionTestCase {
                name: "空字符串".to_string(),
                plaintext: "".to_string(),
                description: "空字符串测试".to_string(),
            },
            EncryptionTestCase {
                name: "长文本".to_string(),
                plaintext: "A".repeat(1000),
                description: "长文本数据测试".to_string(),
            },
            EncryptionTestCase {
                name: "URL".to_string(),
                plaintext: "https://api.anthropic.com/v1/messages".to_string(),
                description: "URL格式数据".to_string(),
            },
        ]
    }

    /// 执行加密兼容性测试
    pub async fn run_compatibility_tests(&self) -> Result<EncryptionCompatibilityReport, EncryptionCompatibilityError> {
        println!("🔐 开始加密兼容性测试...");

        let mut report = EncryptionCompatibilityReport::new();

        let test_cases = self.get_test_cases();

        // 1. Rust加密 -> Rust解密测试
        println!("🦀 测试Rust加密/解密循环...");
        match self.test_rust_encryption_roundtrip(&test_cases).await {
            Ok(results) => {
                report.rust_roundtrip_results = results;
                println!("✅ Rust加密/解密测试完成");
            }
            Err(e) => {
                report.add_error("Rust加密/解密测试", &e.to_string());
                println!("❌ Rust加密/解密测试失败: {}", e);
            }
        }

        // 2. Python加密 -> Rust解密测试
        println!("🐍🦀 测试Python加密 -> Rust解密...");
        match self.test_python_to_rust_compatibility(&test_cases).await {
            Ok(results) => {
                report.python_to_rust_results = results;
                println!("✅ Python加密 -> Rust解密测试完成");
            }
            Err(e) => {
                report.add_error("Python加密 -> Rust解密测试", &e.to_string());
                println!("❌ Python加密 -> Rust解密测试失败: {}", e);
            }
        }

        // 3. Rust加密 -> Python解密测试
        println!("🦀🐍 测试Rust加密 -> Python解密...");
        match self.test_rust_to_python_compatibility(&test_cases).await {
            Ok(results) => {
                report.rust_to_python_results = results;
                println!("✅ Rust加密 -> Python解密测试完成");
            }
            Err(e) => {
                report.add_error("Rust加密 -> Python解密测试", &e.to_string());
                println!("❌ Rust加密 -> Python解密测试失败: {}", e);
            }
        }

        // 4. 加密数据兼容性验证
        if let Some(ref python_data) = self.python_encrypted_data {
            println!("🔍 验证Python加密数据兼容性...");
            match self.validate_python_encrypted_data(python_data).await {
                Ok(compatible) => {
                    report.python_data_compatible = compatible;
                    if compatible {
                        println!("✅ Python加密数据兼容性验证通过");
                    } else {
                        println!("❌ Python加密数据兼容性验证失败");
                    }
                }
                Err(e) => {
                    report.add_error("Python加密数据验证", &e.to_string());
                    println!("❌ Python加密数据验证失败: {}", e);
                }
            }
        }

        // 5. 性能对比测试
        println!("⚡ 执行加密性能对比测试...");
        match self.test_encryption_performance(&test_cases).await {
            Ok(results) => {
                report.performance_results = Some(results);
                println!("✅ 加密性能测试完成");
            }
            Err(e) => {
                report.add_error("加密性能测试", &e.to_string());
                println!("❌ 加密性能测试失败: {}", e);
            }
        }

        report.calculate_summary();
        report.print_report();

        Ok(report)
    }

    /// 测试Rust加密/解密循环
    async fn test_rust_encryption_roundtrip(&self, test_cases: &[EncryptionTestCase]) -> Vec<EncryptionTestResult> {
        let mut results = Vec::new();

        for case in test_cases {
            let start_time = std::time::Instant::now();

            match self.crypto_service.encrypt(&case.plaintext) {
                Ok(encrypted) => {
                    match self.crypto_service.decrypt(&encrypted) {
                        Ok(decrypted) => {
                            let success = decrypted == case.plaintext;
                            let duration = start_time.elapsed();

                            results.push(EncryptionTestResult {
                                test_name: case.name.clone(),
                                plaintext: case.plaintext.clone(),
                                encrypted: Some(encrypted),
                                decrypted: Some(decrypted),
                                success,
                                error_message: if success { None } else { Some("解密结果与原文不匹配".to_string()) },
                                duration_ms: duration.as_millis() as f64,
                            });
                        }
                        Err(e) => {
                            results.push(EncryptionTestResult {
                                test_name: case.name.clone(),
                                plaintext: case.plaintext.clone(),
                                encrypted: None,
                                decrypted: None,
                                success: false,
                                error_message: Some(format!("Rust解密失败: {}", e)),
                                duration_ms: start_time.elapsed().as_millis() as f64,
                            });
                        }
                    }
                }
                Err(e) => {
                    results.push(EncryptionTestResult {
                        test_name: case.name.clone(),
                        plaintext: case.plaintext.clone(),
                        encrypted: None,
                        decrypted: None,
                        success: false,
                        error_message: Some(format!("Rust加密失败: {}", e)),
                        duration_ms: start_time.elapsed().as_millis() as f64,
                    });
                }
            }
        }

        results
    }

    /// 测试Python加密 -> Rust解密兼容性
    async fn test_python_to_rust_compatibility(&self, test_cases: &[EncryptionTestCase]) -> Vec<EncryptionTestResult> {
        let mut results = Vec::new();

        for case in test_cases {
            let start_time = std::time::Instant::now();

            // 使用Python加密
            match self.encrypt_with_python(&case.plaintext, &self.test_key).await {
                Ok(encrypted) => {
                    // 使用Rust解密
                    match self.crypto_service.decrypt(&encrypted) {
                        Ok(decrypted) => {
                            let success = decrypted == case.plaintext;
                            let duration = start_time.elapsed();

                            results.push(EncryptionTestResult {
                                test_name: case.name.clone(),
                                plaintext: case.plaintext.clone(),
                                encrypted: Some(encrypted),
                                decrypted: Some(decrypted),
                                success,
                                error_message: if success { None } else { Some("Python加密->Rust解密结果不匹配".to_string()) },
                                duration_ms: duration.as_millis() as f64,
                            });
                        }
                        Err(e) => {
                            results.push(EncryptionTestResult {
                                test_name: case.name.clone(),
                                plaintext: case.plaintext.clone(),
                                encrypted: Some(encrypted),
                                decrypted: None,
                                success: false,
                                error_message: Some(format!("Rust解密Python加密数据失败: {}", e)),
                                duration_ms: start_time.elapsed().as_millis() as f64,
                            });
                        }
                    }
                }
                Err(e) => {
                    results.push(EncryptionTestResult {
                        test_name: case.name.clone(),
                        plaintext: case.plaintext.clone(),
                        encrypted: None,
                        decrypted: None,
                        success: false,
                        error_message: Some(format!("Python加密失败: {}", e)),
                        duration_ms: start_time.elapsed().as_millis() as f64,
                    });
                }
            }
        }

        results
    }

    /// 测试Rust加密 -> Python解密兼容性
    async fn test_rust_to_python_compatibility(&self, test_cases: &[EncryptionTestCase]) -> Vec<EncryptionTestResult> {
        let mut results = Vec::new();

        for case in test_cases {
            let start_time = std::time::Instant::now();

            // 使用Rust加密
            match self.crypto_service.encrypt(&case.plaintext) {
                Ok(encrypted) => {
                    // 使用Python解密
                    match self.decrypt_with_python(&encrypted, &self.test_key).await {
                        Ok(decrypted) => {
                            let success = decrypted == case.plaintext;
                            let duration = start_time.elapsed();

                            results.push(EncryptionTestResult {
                                test_name: case.name.clone(),
                                plaintext: case.plaintext.clone(),
                                encrypted: Some(encrypted),
                                decrypted: Some(decrypted),
                                success,
                                error_message: if success { None } else { Some("Rust加密->Python解密结果不匹配".to_string()) },
                                duration_ms: duration.as_millis() as f64,
                            });
                        }
                        Err(e) => {
                            results.push(EncryptionTestResult {
                                test_name: case.name.clone(),
                                plaintext: case.plaintext.clone(),
                                encrypted: Some(encrypted),
                                decrypted: None,
                                success: false,
                                error_message: Some(format!("Python解密Rust加密数据失败: {}", e)),
                                duration_ms: start_time.elapsed().as_millis() as f64,
                            });
                        }
                    }
                }
                Err(e) => {
                    results.push(EncryptionTestResult {
                        test_name: case.name.clone(),
                        plaintext: case.plaintext.clone(),
                        encrypted: None,
                        decrypted: None,
                        success: false,
                        error_message: Some(format!("Rust加密失败: {}", e)),
                        duration_ms: start_time.elapsed().as_millis() as f64,
                    });
                }
            }
        }

        results
    }

    /// 验证Python加密数据
    async fn validate_python_encrypted_data(&self, python_data: &PythonEncryptedTestData) -> Result<bool, EncryptionCompatibilityError> {
        for case in &python_data.test_cases {
            if !case.success {
                return Err(EncryptionCompatibilityError::Validation(format!(
                    "Python加密测试用例 '{}' 失败", case.name
                )));
            }

            // 使用Rust解密验证
            match self.crypto_service.decrypt(&case.encrypted) {
                Ok(decrypted) => {
                    if decrypted != case.plaintext {
                        return Err(EncryptionCompatibilityError::Validation(format!(
                            "Rust解密Python加密数据不匹配: 测试用例 '{}'", case.name
                        )));
                    }
                }
                Err(e) => {
                    return Err(EncryptionCompatibilityError::Validation(format!(
                        "Rust无法解密Python加密数据: 测试用例 '{}', 错误: {}", case.name, e
                    )));
                }
            }
        }

        Ok(true)
    }

    /// 测试加密性能
    async fn test_encryption_performance(&self, test_cases: &[EncryptionTestCase]) -> Result<EncryptionPerformanceResults, EncryptionCompatibilityError> {
        let iterations = 100;
        let mut rust_total_time = std::time::Duration::new(0, 0);
        let mut python_total_time = std::time::Duration::new(0, 0);

        // 测试Rust加密性能
        for _ in 0..iterations {
            for case in test_cases {
                let start = std::time::Instant::now();
                let _ = self.crypto_service.encrypt(&case.plaintext);
                rust_total_time += start.elapsed();
            }
        }

        // 测试Python加密性能
        for _ in 0..iterations {
            for case in test_cases {
                let start = std::time::Instant::now();
                let _ = self.encrypt_with_python(&case.plaintext, &self.test_key).await;
                python_total_time += start.elapsed();
            }
        }

        let rust_ops_per_sec = (iterations * test_cases.len()) as f64 / rust_total_time.as_secs_f64();
        let python_ops_per_sec = (iterations * test_cases.len()) as f64 / python_total_time.as_secs_f64();

        Ok(EncryptionPerformanceResults {
            rust_operations_per_second: rust_ops_per_sec,
            python_operations_per_second: python_ops_per_sec,
            rust_avg_time_ms: rust_total_time.as_millis() as f64 / (iterations * test_cases.len()) as f64,
            python_avg_time_ms: python_total_time.as_millis() as f64 / (iterations * test_cases.len()) as f64,
            performance_ratio: rust_ops_per_sec / python_ops_per_sec,
        })
    }
}

/// 加密测试结果
#[derive(Debug, serde::Serialize)]
pub struct EncryptionTestResult {
    pub test_name: String,
    pub plaintext: String,
    pub encrypted: Option<String>,
    pub decrypted: Option<String>,
    pub success: bool,
    pub error_message: Option<String>,
    pub duration_ms: f64,
}

/// 加密兼容性报告
#[derive(Debug, serde::Serialize)]
pub struct EncryptionCompatibilityReport {
    pub rust_roundtrip_results: Vec<EncryptionTestResult>,
    pub python_to_rust_results: Vec<EncryptionTestResult>,
    pub rust_to_python_results: Vec<EncryptionTestResult>,
    pub python_data_compatible: bool,
    pub performance_results: Option<EncryptionPerformanceResults>,
    pub errors: Vec<String>,
    pub test_summary: EncryptionTestSummary,
}

/// 加密性能测试结果
#[derive(Debug, serde::Serialize)]
pub struct EncryptionPerformanceResults {
    pub rust_operations_per_second: f64,
    pub python_operations_per_second: f64,
    pub rust_avg_time_ms: f64,
    pub python_avg_time_ms: f64,
    pub performance_ratio: f64,
}

/// 加密测试统计
#[derive(Debug, serde::Serialize)]
pub struct EncryptionTestSummary {
    pub total_tests: usize,
    pub rust_roundtrip_passed: usize,
    pub python_to_rust_passed: usize,
    pub rust_to_python_passed: usize,
    pub overall_success_rate: f64,
    pub performance_acceptable: bool,
}

impl EncryptionCompatibilityReport {
    pub fn new() -> Self {
        Self {
            rust_roundtrip_results: Vec::new(),
            python_to_rust_results: Vec::new(),
            rust_to_python_results: Vec::new(),
            python_data_compatible: false,
            performance_results: None,
            errors: Vec::new(),
            test_summary: EncryptionTestSummary {
                total_tests: 0,
                rust_roundtrip_passed: 0,
                python_to_rust_passed: 0,
                rust_to_python_passed: 0,
                overall_success_rate: 0.0,
                performance_acceptable: false,
            },
        }
    }

    pub fn add_error(&mut self, test_name: &str, error: &str) {
        self.errors.push(format!("{}: {}", test_name, error));
    }

    pub fn calculate_summary(&mut self) {
        self.test_summary.rust_roundtrip_passed = self.rust_roundtrip_results.iter().filter(|r| r.success).count();
        self.test_summary.python_to_rust_passed = self.python_to_rust_results.iter().filter(|r| r.success).count();
        self.test_summary.rust_to_python_passed = self.rust_to_python_results.iter().filter(|r| r.success).count();

        self.test_summary.total_tests = self.rust_roundtrip_results.len()
            + self.python_to_rust_results.len()
            + self.rust_to_python_results.len();

        let total_passed = self.test_summary.rust_roundtrip_passed
            + self.test_summary.python_to_rust_passed
            + self.test_summary.rust_to_python_passed;

        if self.test_summary.total_tests > 0 {
            self.test_summary.overall_success_rate = (total_passed as f64) / (self.test_summary.total_tests as f64) * 100.0;
        }

        self.test_summary.performance_acceptable = self.performance_results.as_ref()
            .map_or(false, |p| p.performance_ratio >= 0.5); // Rust性能不低于Python的50%
    }

    pub fn is_successful(&self) -> bool {
        self.test_summary.overall_success_rate >= 95.0 && self.python_data_compatible
    }

    pub fn print_report(&self) {
        println!("\n🔐 加密兼容性测试报告");
        println!("========================");
        
        println!("\n📊 测试结果统计:");
        println!("Rust加密/解密: {}/{} 通过", self.test_summary.rust_roundtrip_passed, self.rust_roundtrip_results.len());
        println!("Python加密->Rust解密: {}/{} 通过", self.test_summary.python_to_rust_passed, self.python_to_rust_results.len());
        println!("Rust加密->Python解密: {}/{} 通过", self.test_summary.rust_to_python_passed, self.rust_to_python_results.len());
        println!("Python数据兼容性: {}", if self.python_data_compatible { "✅ 通过" } else { "❌ 失败" });

        println!("\n📈 总体统计:");
        println!("总测试数: {}", self.test_summary.total_tests);
        println!("通过测试数: {}", 
            self.test_summary.rust_roundtrip_passed + self.test_summary.python_to_rust_passed + self.test_summary.rust_to_python_passed);
        println!("成功率: {:.1}%", self.test_summary.overall_success_rate);

        if let Some(ref perf) = self.performance_results {
            println!("\n⚡ 性能对比:");
            println!("Rust: {:.1} ops/sec (平均 {:.2}ms)", perf.rust_operations_per_second, perf.rust_avg_time_ms);
            println!("Python: {:.1} ops/sec (平均 {:.2}ms)", perf.python_operations_per_second, perf.python_avg_time_ms);
            println!("性能比: {:.2}x", perf.performance_ratio);
        }

        if !self.errors.is_empty() {
            println!("\n❌ 错误详情:");
            for error in &self.errors {
                println!("  - {}", error);
            }
        }

        println!("\n🏆 总体结果: {}", 
            if self.is_successful() { "✅ 加密兼容性测试全部通过" } 
            else { "❌ 加密兼容性测试存在问题" }
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::testing::generate_test_key;

    #[tokio::test]
    async fn test_encryption_compatibility() {
        let key = generate_test_key();
        let mut validator = EncryptionCompatibilityValidator::new(&key).unwrap();
        
        // 运行兼容性测试
        let report = validator.run_compatibility_tests().await;
        assert!(report.is_ok());
        
        let report = report.unwrap();
        report.print_report();
        
        // 验证报告
        assert!(report.is_successful());
    }
}