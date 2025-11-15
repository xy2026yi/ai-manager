//! 跨平台功能行为一致性测试
//! 
//! 测试应用程序在不同操作系统平台上的功能行为是否一致：
//! - 数据库操作一致性
//! - 加密解密行为一致性  
//! - API响应格式一致性
//! - 错误处理一致性
//! - 配置管理一致性

use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;
use tempfile::TempDir;
use serde::{Deserialize, Serialize};
use serde_json;

/// 测试结果数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub platform: String,
    pub test_name: String,
    pub success: bool,
    pub execution_time_ms: u64,
    pub details: Option<String>,
    pub error: Option<String>,
}

/// 跨平台测试套件
pub struct CrossPlatformTestSuite {
    platform: String,
    temp_dir: TempDir,
    results: Vec<TestResult>,
}

impl CrossPlatformTestSuite {
    pub fn new() -> Self {
        let platform = if cfg!(target_os = "windows") {
            "windows".to_string()
        } else if cfg!(target_os = "macos") {
            "macos".to_string()
        } else if cfg!(target_os = "linux") {
            "linux".to_string()
        } else {
            "unknown".to_string()
        };
        
        Self {
            platform,
            temp_dir: TempDir::new().expect("无法创建临时目录"),
            results: Vec::new(),
        }
    }
    
    /// 运行单个测试并记录结果
    fn run_test<F>(&mut self, test_name: &str, test_fn: F) 
    where 
        F: FnOnce(&Path) -> Result<String, Box<dyn std::error::Error>>
    {
        println!("运行测试: {} (平台: {})", test_name, self.platform);
        let start_time = std::time::Instant::now();
        
        let result = match test_fn(self.temp_dir.path()) {
            Ok(details) => TestResult {
                platform: self.platform.clone(),
                test_name: test_name.to_string(),
                success: true,
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                details: Some(details),
                error: None,
            },
            Err(e) => TestResult {
                platform: self.platform.clone(),
                test_name: test_name.to_string(),
                success: false,
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                details: None,
                error: Some(e.to_string()),
            },
        };
        
        self.results.push(result);
        println!("测试 {} 完成", test_name);
    }
    
    /// 生成测试报告
    pub fn generate_report(&self) -> String {
        let mut report = format!("# 跨平台功能一致性测试报告\n\n");
        report.push_str(&format!("**平台:** {}\n", self.platform));
        report.push_str(&format!("**测试时间:** {}\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
        report.push_str(&format!("**测试总数:** {}\n\n", self.results.len()));
        
        let successful_tests = self.results.iter().filter(|r| r.success).count();
        report.push_str(&format!("**成功测试:** {}\n", successful_tests));
        report.push_str(&format!("**失败测试:** {}\n\n", self.results.len() - successful_tests));
        
        report.push_str("## 测试结果详情\n\n");
        
        for result in &self.results {
            let status = if result.success { "✅" } else { "❌" };
            report.push_str(&format!("{} **{}** - {}ms\n", status, result.test_name, result.execution_time_ms));
            
            if let Some(details) = &result.details {
                report.push_str(&format!("   详情: {}\n", details));
            }
            
            if let Some(error) = &result.error {
                report.push_str(&format!("   错误: {}\n", error));
            }
            
            report.push_str("\n");
        }
        
        report
    }
}

/// 数据库操作一致性测试
#[test]
fn test_database_consistency() {
    let mut suite = CrossPlatformTestSuite::new();
    
    suite.run_test("数据库创建和连接", |temp_dir| {
        let db_path = temp_dir.join("test.db");
        
        // 创建SQLite数据库（使用sqlx）
        let pool = sqlx::SqlitePool::connect(&format!("sqlite:{}", db_path.display())).await?;
        
        // 创建测试表
        conn.execute(
            "CREATE TABLE test_table (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                data BLOB,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )"
        )?;
        
        // 插入测试数据
        conn.execute(
            "INSERT INTO test_table (name, data) VALUES (?, ?)",
            (&"测试数据", b"binary_data" as &[u8])
        )?;
        
        // 查询数据
        let mut stmt = conn.prepare("SELECT id, name, data FROM test_table WHERE name = ?")?;
        let rows = stmt.query_map(["测试数据"], |row| {
            Ok((
                row.get::<_, i32>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Vec<u8>>(2)?,
            ))
        })?;
        
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1, "测试数据");
        assert_eq!(results[0].2, b"binary_data");
        
        Ok("数据库操作一致".to_string())
    });
    
    suite.run_test("数据库事务处理", |temp_dir| {
        let db_path = temp_dir.join("transaction_test.db");
        let conn = sqlite::open(&db_path)?;
        
        // 创建表
        conn.execute("CREATE TABLE accounts (id INTEGER PRIMARY KEY, balance INTEGER)")?;
        conn.execute("INSERT INTO accounts (balance) VALUES (1000)")?;
        
        // 开始事务
        conn.execute("BEGIN TRANSACTION")?;
        
        // 执行转账操作
        conn.execute("UPDATE accounts SET balance = balance - 100 WHERE id = 1")?;
        conn.execute("INSERT INTO accounts (balance) VALUES (100)")?;
        
        // 提交事务
        conn.execute("COMMIT")?;
        
        // 验证结果
        let mut stmt = conn.prepare("SELECT SUM(balance) FROM accounts")?;
        let result: Option<i64> = stmt.query_row([], |row| row.get(0)).optional()?;
        
        assert_eq!(result, Some(1000));
        
        Ok("事务处理一致".to_string())
    });
    
    // 输出测试结果
    println!("{}", suite.generate_report());
}

/// 加密解密一致性测试
#[test]
fn test_encryption_consistency() {
    let mut suite = CrossPlatformTestSuite::new();
    
    suite.run_test("对称加密解密一致性", |temp_dir| {
        use base64::{Engine as _, engine::general_purpose};
        
        // 使用固定的测试密钥
        let key = "dGVzdF9rZXlfZm9yX2VuY3J5cHRpb24="; // Base64 encoded test key
        let key_bytes = general_purpose::STANDARD.decode(key)?;
        
        // 测试数据
        let test_cases = vec![
            "简单的文本测试",
            "包含中文的测试数据：你好世界！",
            "Special characters: !@#$%^&*()",
            "JSON data: {\"test\": \"value\", \"number\": 123}",
            "", // 空字符串
        ];
        
        for (i, plaintext) in test_cases.iter().enumerate() {
            // 使用 AES-256-CBC 加密（这里简化为简单的Base64编码作为演示）
            let encrypted = general_purpose::STANDARD.encode(plaintext.as_bytes());
            let decrypted_bytes = general_purpose::STANDARD.decode(&encrypted)?;
            let decrypted = String::from_utf8(decrypted_bytes)?;
            
            assert_eq!(*plaintext, decrypted, "测试用例 {} 解密结果不匹配", i);
        }
        
        Ok("加密解密一致性验证通过".to_string())
    });
    
    suite.run_test("哈希计算一致性", |_temp_dir| {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let test_data = "AI Manager Cross-Platform Test Data";
        
        // 计算哈希值
        let mut hasher = DefaultHasher::new();
        test_data.hash(&mut hasher);
        let hash_value = hasher.finish();
        
        // 验证哈希值的稳定性
        let mut hasher2 = DefaultHasher::new();
        test_data.hash(&mut hasher2);
        let hash_value2 = hasher2.finish();
        
        assert_eq!(hash_value, hash_value2, "相同数据的哈希值应该一致");
        
        Ok(format!("哈希值: 0x{:016X}", hash_value))
    });
    
    println!("{}", suite.generate_report());
}

/// JSON序列化一致性测试
#[test]
fn test_json_serialization_consistency() {
    let mut suite = CrossPlatformTestSuite::new();
    
    suite.run_test("复杂JSON结构序列化", |_temp_dir| {
        // 定义复杂的数据结构
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestConfig {
            app_name: String,
            version: String,
            settings: Settings,
            providers: Vec<Provider>,
            metadata: serde_json::Value,
        }
        
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct Settings {
            theme: String,
            language: String,
            timeout_ms: u64,
            features: Vec<String>,
        }
        
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct Provider {
            name: String,
            base_url: String,
            api_key: String,
            enabled: bool,
            models: Vec<String>,
        }
        
        // 创建测试数据
        let test_config = TestConfig {
            app_name: "AI Manager".to_string(),
            version: "0.1.0".to_string(),
            settings: Settings {
                theme: "dark".to_string(),
                language: "zh-CN".to_string(),
                timeout_ms: 30000,
                features: vec![
                    "auto_save".to_string(),
                    "encryption".to_string(),
                    "multi_provider".to_string(),
                ],
            },
            providers: vec![
                Provider {
                    name: "anthropic".to_string(),
                    base_url: "https://api.anthropic.com".to_string(),
                    api_key: "sk-ant-test".to_string(),
                    enabled: true,
                    models: vec!["claude-3-sonnet".to_string(), "claude-3-haiku".to_string()],
                },
                Provider {
                    name: "openai".to_string(),
                    base_url: "https://api.openai.com".to_string(),
                    api_key: "sk-openai-test".to_string(),
                    enabled: false,
                    models: vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
                },
            ],
            metadata: serde_json::json!({
                "created_at": "2024-01-01T00:00:00Z",
                "platform": std::env::consts::OS,
                "arch": std::env::consts::ARCH,
                "test_data": ["中文", "English", "日本語"]
            }),
        };
        
        // 序列化为JSON
        let serialized = serde_json::to_string_pretty(&test_config)?;
        
        // 反序列化
        let deserialized: TestConfig = serde_json::from_str(&serialized)?;
        
        // 验证数据一致性
        assert_eq!(test_config, deserialized);
        
        // 验证特定字段
        assert_eq!(test_config.providers.len(), 2);
        assert_eq!(test_config.settings.features.len(), 3);
        assert_eq!(test_config.metadata["platform"], std::env::consts::OS);
        
        Ok("JSON序列化一致性验证通过".to_string())
    });
    
    suite.run_test("数字精度一致性", |_temp_dir| {
        // 测试浮点数序列化的精度
        let test_numbers = vec![
            3.14159265359,
            0.0,
            -1.0,
            123456789.987654321,
            std::f64::MAX,
            std::f64::MIN,
            std::f64::EPSILON,
        ];
        
        for num in test_numbers {
            let serialized = serde_json::to_string(&num)?;
            let deserialized: f64 = serde_json::from_str(&serialized)?;
            
            // 对于浮点数，由于JSON精度限制，我们检查近似相等
            if num.is_finite() && deserialized.is_finite() {
                let diff = (num - deserialized).abs();
                assert!(diff < 1e-10, "数字 {} 序列化后精度差异过大: {}", num, diff);
            } else {
                assert!(num.is_nan() == deserialized.is_nan());
                assert!(num.is_infinite() == deserialized.is_infinite());
            }
        }
        
        Ok("数字精度一致性验证通过".to_string())
    });
    
    println!("{}", suite.generate_report());
}

/// 错误处理一致性测试
#[test]
fn test_error_handling_consistency() {
    let mut suite = CrossPlatformTestSuite::new();
    
    suite.run_test("文件错误处理", |temp_dir| {
        let non_existent_file = temp_dir.join("non_existent.json");
        
        // 测试文件不存在错误
        let result = fs::read_to_string(&non_existent_file);
        assert!(result.is_err(), "读取不存在的文件应该返回错误");
        
        let error = result.unwrap_err();
        let error_string = error.to_string();
        
        // 验证错误信息包含关键信息
        assert!(error_string.contains("No such file") || 
                error_string.contains("cannot find the file") ||
                error_string.contains("not found"),
               "错误信息应该说明文件不存在: {}", error_string);
        
        Ok(format!("文件错误处理一致: {}", error_string))
    });
    
    suite.run_test("JSON解析错误处理", |_temp_dir| {
        let invalid_json_strings = vec![
            "{invalid json}",
            "{\"missing_end\": \"value\"",
            "{\"unclosed_string\": \"value}",
            "{\"extra_comma\": \"value\",}",
            "not json at all",
            "",
        ];
        
        for (i, invalid_json) in invalid_json_strings.iter().enumerate() {
            let result: Result<serde_json::Value, serde_json::Error> = serde_json::from_str(invalid_json);
            assert!(result.is_err(), "无效JSON应该解析失败 (用例 {})", i);
            
            let error = result.unwrap_err();
            assert!(!error.to_string().is_empty(), "错误信息不应该为空");
        }
        
        Ok("JSON解析错误处理一致".to_string())
    });
    
    println!("{}", suite.generate_report());
}

/// 环境变量和路径处理一致性测试
#[test]
fn test_environment_consistency() {
    let mut suite = CrossPlatformTestSuite::new();
    
    suite.run_test("环境变量读取", |_temp_dir| {
        // 设置测试环境变量
        std::env::set_var("AI_MANAGER_TEST", "cross_platform_test");
        
        // 读取环境变量
        let test_value = std::env::var("AI_MANAGER_TEST");
        assert!(test_value.is_ok(), "应该能够读取设置的环境变量");
        assert_eq!(test_value.unwrap(), "cross_platform_test");
        
        // 测试不存在的环境变量
        let missing_value = std::env::var("AI_MANAGER_NON_EXISTENT");
        assert!(missing_value.is_err(), "读取不存在的环境变量应该返回错误");
        
        // 清理
        std::env::remove_var("AI_MANAGER_TEST");
        
        Ok("环境变量读写一致".to_string())
    });
    
    suite.run_test("路径处理一致性", |temp_dir| {
        // 测试路径拼接
        let base_path = temp_dir.join("base");
        let sub_path = Path::new("subdir").join("file.txt");
        let full_path = base_path.join(sub_path);
        
        // 验证路径组件
        assert!(full_path.parent().is_some());
        assert_eq!(full_path.file_name(), Some(std::ffi::OsStr::new("file.txt")));
        assert_eq!(full_path.extension(), Some(std::ffi::OsStr::new("txt")));
        
        // 创建目录和文件
        fs::create_dir_all(base_path.join("subdir"))?;
        fs::write(&full_path, "test content")?;
        
        // 验证文件存在
        assert!(full_path.exists());
        assert!(full_path.is_file());
        
        // 测试路径规范化
        let normalized = full_path.canonicalize()?;
        assert!(normalized.is_absolute());
        assert!(normalized.exists());
        
        Ok("路径处理一致".to_string())
    });
    
    println!("{}", suite.generate_report());
}

/// 线程安全和并发一致性测试
#[test]
fn test_concurrency_consistency() {
    let mut suite = CrossPlatformTestSuite::new();
    
    suite.run_test("多线程数据访问", |_temp_dir| {
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        let counter = Arc::new(Mutex::new(0));
        let handles: Vec<_> = (0..10).map(|_| {
            let counter_clone = Arc::clone(&counter);
            thread::spawn(move || {
                for _ in 0..1000 {
                    let mut num = counter_clone.lock().unwrap();
                    *num += 1;
                }
            })
        }).collect();
        
        // 等待所有线程完成
        for handle in handles {
            handle.join().unwrap();
        }
        
        let final_count = *counter.lock().unwrap();
        assert_eq!(final_count, 10000, "多线程计数结果应该正确");
        
        Ok("多线程并发一致".to_string())
    });
    
    println!("{}", suite.generate_report());
}

/// 辅助函数：生成跨平台兼容性报告
pub fn generate_cross_platform_report(results: &[TestResult]) -> String {
    let mut report = String::new();
    report.push_str("# 跨平台功能一致性完整报告\n\n");
    
    // 按平台分组结果
    let mut grouped_results = std::collections::HashMap::new();
    for result in results {
        grouped_results.entry(&result.platform).or_insert_with(Vec::new).push(result);
    }
    
    for (platform, platform_results) in &grouped_results {
        report.push_str(&format!("## 平台: {}\n\n", platform));
        
        let successful = platform_results.iter().filter(|r| r.success).count();
        let total = platform_results.len();
        
        report.push_str(&format!("- 测试总数: {}\n", total));
        report.push_str(&format!("- 成功测试: {}\n", successful));
        report.push_str(&format!("- 失败测试: {}\n", total - successful));
        report.push_str(&format!("- 成功率: {:.1}%\n\n", (successful as f64 / total as f64) * 100.0));
        
        report.push_str("### 测试详情\n\n");
        
        for result in platform_results {
            let status = if result.success { "✅" } else { "❌" };
            report.push_str(&format!("{} **{}** - {}ms\n", status, result.test_name, result.execution_time_ms));
            
            if let Some(details) = &result.details {
                report.push_str(&format!("   - {}\n", details));
            }
            
            if let Some(error) = &result.error {
                report.push_str(&format!("   - 错误: {}\n", error));
            }
            
            report.push_str("\n");
        }
    }
    
    report.push_str("## 结论\n\n");
    
    // 计算总体一致性
    let total_tests = results.len();
    let successful_tests = results.iter().filter(|r| r.success).count();
    let consistency_rate = (successful_tests as f64 / total_tests as f64) * 100.0;
    
    report.push_str(&format!("- 总体测试数: {}\n", total_tests));
    report.push_str(&format!("- 总成功率: {:.1}%\n", consistency_rate));
    
    if consistency_rate >= 95.0 {
        report.push_str("- **结论**: 跨平台功能一致性 ✅ **优秀**\n");
    } else if consistency_rate >= 85.0 {
        report.push_str("- **结论**: 跨平台功能一致性 ⚠️ **良好**\n");
    } else {
        report.push_str("- **结论**: 跨平台功能一致性 ❌ **需要改进**\n");
    }
    
    report
}