// 数据兼容性验证测试运行器
//
// 统一执行所有数据迁移和加密兼容性测试

use std::path::Path;
use std::time::Instant;
use tokio::time::{sleep, Duration};

use migration_ai_manager_lib::crypto::CryptoService;
use migration_ai_manager_lib::database::{DatabaseManager, DatabaseConfig};

/// 测试运行结果
#[derive(Debug)]
pub struct TestRunnerResult {
    /// 总体是否通过
    pub passed: bool,
    /// 测试开始时间
    pub start_time: Instant,
    /// 测试结束时间
    pub end_time: Instant,
    /// 数据完整性测试结果
    pub data_integrity_result: Option<TestResult>,
    /// 加密兼容性测试结果
    pub encryption_result: Option<EncryptionTestResult>,
    /// 迁移功能测试结果
    pub migration_result: Option<MigrationTestResult>,
    /// 错误信息
    pub errors: Vec<String>,
    /// 警告信息
    pub warnings: Vec<String>,
}

/// 迁移功能测试结果
#[derive(Debug, Clone)]
pub struct MigrationTestResult {
    pub passed: bool,
    pub migrated_records: i64,
    pub failed_records: i64,
    pub duration_ms: u64,
    pub error_details: Vec<String>,
}

/// 数据兼容性验证测试运行器
pub struct DataCompatibilityTestRunner {
    config: TestConfig,
    verbose: bool,
    warnings: Vec<String>,
}

impl DataCompatibilityTestRunner {
    /// 创建新的测试运行器
    pub fn new(verbose: bool) -> Self {
        Self {
            config: TestConfig::new(),
            verbose,
            warnings: Vec::new(),
        }
    }

    /// 运行所有兼容性测试
    pub async fn run_all_tests(&mut self) -> TestRunnerResult {
        let start_time = Instant::now();
        println!("🚀 开始数据兼容性验证测试...");
        println!("📅 测试时间: {:?}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
        println!();

        let mut result = TestRunnerResult {
            passed: true,
            start_time,
            end_time: start_time,
            data_integrity_result: None,
            encryption_result: None,
            migration_result: None,
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        // 1. 环境准备检查
        if let Err(e) = self.prepare_environment().await {
            result.errors.push(format!("环境准备失败: {}", e));
            result.passed = false;
            result.end_time = Instant::now();
            return result;
        }

        // 2. 加密兼容性测试
        println!("📊 步骤 1/3: 加密兼容性测试");
        match self.run_encryption_compatibility_test().await {
            Ok(test_result) => {
                result.encryption_result = Some(test_result.clone());
                if !test_result.passed {
                    result.passed = false;
                    result.errors.extend(test_result.failures);
                }
                if self.verbose {
                    self.print_encryption_test_summary(&test_result);
                }
            }
            Err(e) => {
                result.errors.push(format!("加密兼容性测试失败: {}", e));
                result.passed = false;
            }
        }
        println!();

        // 3. 数据迁移功能测试
        println!("📊 步骤 2/3: 数据迁移功能测试");
        match self.run_migration_test().await {
            Ok(test_result) => {
                result.migration_result = Some(test_result.clone());
                if !test_result.passed {
                    result.passed = false;
                    result.errors.extend(test_result.error_details.clone());
                }
                if self.verbose {
                    self.print_migration_test_summary(&test_result);
                }
            }
            Err(e) => {
                result.errors.push(format!("数据迁移测试失败: {}", e));
                result.passed = false;
            }
        }
        println!();

        // 4. 数据完整性验证测试
        println!("📊 步骤 3/3: 数据完整性验证测试");
        match self.run_data_integrity_test().await {
            Ok(test_result) => {
                result.data_integrity_result = Some(test_result.clone());
                if !test_result.all_passed() {
                    result.passed = false;
                    // 添加失败的测试到错误列表
                    for (table_name, table_result) in test_result.table_results {
                        if !table_result.records_match {
                            result.errors.push(format!("表 {} 记录数量不匹配", table_name));
                        }
                        if !table_result.content_match {
                            result.errors.push(format!("表 {} 内容不匹配", table_name));
                        }
                    }
                }
                if self.verbose {
                    self.print_integrity_test_summary(&test_result);
                }
            }
            Err(e) => {
                result.errors.push(format!("数据完整性测试失败: {}", e));
                result.passed = false;
            }
        }
        println!();

        result.end_time = Instant::now();
        self.print_final_summary(&result);

        result
    }

    /// 准备测试环境
    async fn prepare_environment(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔧 准备测试环境...");

        let preparer = DatabasePreparer::new(self.config.clone());
        preparer.prepare_test_environment().await?;

        // 检查关键文件和目录
        if !self.config.original_db_exists() {
            return Err("原始数据库文件不存在".into());
        }

        if !self.config.test_data_exists() {
            self.warnings.push("测试数据目录不存在，将自动创建".to_string());
        }

        println!("✅ 测试环境准备完成");
        Ok(())
    }

    /// 运行加密兼容性测试
    async fn run_encryption_compatibility_test(&mut self) -> Result<EncryptionTestResult, Box<dyn std::error::Error>> {
        let crypto_service = CryptoService::new("test_key_for_migration_32bytes!!")?;
        let tester = EncryptionCompatibilityTester::new(crypto_service);
        
        let test_result = tester.run_comprehensive_tests().await?;
        
        if test_result.passed {
            println!("✅ 加密兼容性测试通过");
        } else {
            println!("❌ 加密兼容性测试失败");
        }

        Ok(test_result)
    }

    /// 运行数据迁移测试
    async fn run_migration_test(&mut self) -> Result<MigrationTestResult, Box<dyn std::error::Error>> {
        // 准备临时数据库
        let preparer = DatabasePreparer::new(self.config.clone());
        preparer.copy_original_to_temp().await?;

        // 创建数据库管理器和加密服务
        let db_config = crate::database::DatabaseConfig {
            url: self.config.temp_db_url(),
            max_connections: 5,
            min_connections: 1,
            connect_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(60),
            max_lifetime: Duration::from_secs(300),
        };

        let db_manager = DatabaseManager::new(db_config).await?;
        let crypto_service = CryptoService::new("test_key_for_migration_32bytes!!")?;

        // 创建数据迁移器
        let migrator = DataMigrator::new(db_manager, crypto_service);
        let start_time = Instant::now();

        // 执行迁移
        let migration_stats = migrator.migrate_all_data().await?;
        let duration = start_time.elapsed();

        let test_result = MigrationTestResult {
            passed: migration_stats.failed_records == 0,
            migrated_records: migration_stats.migrated_records,
            failed_records: migration_stats.failed_records,
            duration_ms: duration.as_millis() as u64,
            error_details: migration_stats.error_messages,
        };

        if test_result.passed {
            println!("✅ 数据迁移测试通过 - 迁移了 {} 条记录", test_result.migrated_records);
        } else {
            println!("❌ 数据迁移测试失败 - {} 条记录迁移失败", test_result.failed_records);
        }

        Ok(test_result)
    }

    /// 运行数据完整性测试
    async fn run_data_integrity_test(&mut self) -> Result<TestResult, Box<dyn std::error::Error>> {
        // 创建原始数据库连接池
        let original_pool = self.config.create_original_db_pool().await?;
        let migrated_pool = self.config.create_temp_db_pool().await?;

        let crypto_service = CryptoService::new("test_key_for_migration_32bytes!!")?;
        let validator = DataIntegrityValidator::new(original_pool, migrated_pool, crypto_service);

        let test_result = validator.validate_all_data().await?;

        if test_result.all_passed() {
            println!("✅ 数据完整性验证通过");
        } else {
            println!("❌ 数据完整性验证失败");
        }

        Ok(test_result)
    }

    /// 打印加密测试摘要
    fn print_encryption_test_summary(&self, result: &EncryptionTestResult) {
        println!("   - 往返加密测试: {} 个通过", result.round_trip_passed);
        println!("   - 格式兼容测试: {} 个通过", result.format_passed);
        println!("   - 跨密钥测试: {} 个通过", result.cross_key_passed);
        println!("   - 总测试数: {}", result.total_tests);
        if !result.failures.is_empty() {
            println!("   - 失败数: {}", result.failures.len());
        }
    }

    /// 打印迁移测试摘要
    fn print_migration_test_summary(&self, result: &MigrationTestResult) {
        println!("   - 成功迁移记录: {}", result.migrated_records);
        println!("   - 失败记录: {}", result.failed_records);
        println!("   - 耗时: {}ms", result.duration_ms);
    }

    /// 打印完整性测试摘要
    fn print_integrity_test_summary(&self, result: &TestResult) {
        println!("   - 验证表数量: {}", result.table_results.len());
        let passed_tables = result.table_results.values()
            .filter(|t| t.records_match && t.content_match)
            .count();
        println!("   - 通过表数量: {}", passed_tables);
        
        for (table_name, table_result) in &result.table_results {
            let status = if table_result.records_match && table_result.content_match {
                "✅"
            } else {
                "❌"
            };
            println!("   - {}: {}", table_name, status);
        }
    }

    /// 打印最终测试摘要
    fn print_final_summary(&self, result: &TestRunnerResult) {
        let duration = result.end_time.duration_since(result.start_time);
        
        println!("🏁 数据兼容性验证测试完成");
        println!("⏱️  总耗时: {:?}", duration);
        println!();
        
        if result.passed {
            println!("🎉 所有测试通过！数据兼容性验证成功。");
        } else {
            println!("❌ 测试失败！发现以下问题:");
            for error in &result.errors {
                println!("   ❌ {}", error);
            }
        }
        
        if !result.warnings.is_empty() {
            println!();
            println!("⚠️  警告信息:");
            for warning in &result.warnings {
                println!("   ⚠️  {}", warning);
            }
        }
        
        // 生成详细报告
        if let Err(e) = self.generate_detailed_report(result) {
            println!("⚠️  生成详细报告失败: {}", e);
        }
    }

    /// 生成详细测试报告
    fn generate_detailed_report(&self, result: &TestRunnerResult) -> Result<(), Box<dyn std::error::Error>> {
        let report_path = Path::new("test_compatibility_report.md");
        let mut content = String::new();

        content.push_str("# 数据兼容性验证测试报告\n\n");
        content.push_str(&format!("**测试时间**: {:?}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
        content.push_str(&format!("**总耗时**: {:?}\n\n", result.end_time.duration_since(result.start_time)));
        content.push_str(&format!("**测试结果**: {}\n\n", if result.passed { "通过 ✅" } else { "失败 ❌" }));

        // 加密兼容性测试结果
        if let Some(enc_result) = &result.encryption_result {
            content.push_str("## 加密兼容性测试\n\n");
            content.push_str(&format!("- 总测试数: {}\n", enc_result.total_tests));
            content.push_str(&format!("- 往返加密通过: {}\n", enc_result.round_trip_passed));
            content.push_str(&format!("- 格式兼容通过: {}\n", enc_result.format_passed));
            content.push_str(&format!("- 跨密钥测试通过: {}\n", enc_result.cross_key_passed));
            if !enc_result.failures.is_empty() {
                content.push_str("- 失败详情:\n");
                for failure in &enc_result.failures {
                    content.push_str(&format!("  - {}\n", failure));
                }
            }
            content.push_str("\n");
        }

        // 数据迁移测试结果
        if let Some(mig_result) = &result.migration_result {
            content.push_str("## 数据迁移测试\n\n");
            content.push_str(&format!("- 成功迁移记录: {}\n", mig_result.migrated_records));
            content.push_str(&format!("- 失败记录: {}\n", mig_result.failed_records));
            content.push_str(&format!("- 迁移耗时: {}ms\n", mig_result.duration_ms));
            if !mig_result.error_details.is_empty() {
                content.push_str("- 错误详情:\n");
                for error in &mig_result.error_details {
                    content.push_str(&format!("  - {}\n", error));
                }
            }
            content.push_str("\n");
        }

        // 数据完整性测试结果
        if let Some(int_result) = &result.data_integrity_result {
            content.push_str("## 数据完整性测试\n\n");
            content.push_str(&format!("- 验证表数量: {}\n", int_result.table_results.len()));
            for (table_name, table_result) in &int_result.table_results {
                let status = if table_result.records_match && table_result.content_match {
                    "通过 ✅"
                } else {
                    "失败 ❌"
                };
                content.push_str(&format!("- {}: {}\n", table_name, status));
            }
            content.push_str("\n");
        }

        // 错误和警告
        if !result.errors.is_empty() {
            content.push_str("## 错误信息\n\n");
            for error in &result.errors {
                content.push_str(&format!("- {}\n", error));
            }
            content.push_str("\n");
        }

        if !result.warnings.is_empty() {
            content.push_str("## 警告信息\n\n");
            for warning in &result.warnings {
                content.push_str(&format!("- {}\n", warning));
            }
            content.push_str("\n");
        }

        std::fs::write(report_path, content)?;
        println!("📄 详细报告已生成: {:?}", report_path);

        Ok(())
    }
}

/// 运行所有兼容性测试的便捷函数
pub async fn run_compatibility_tests(verbose: bool) -> Result<TestRunnerResult, Box<dyn std::error::Error>> {
    let mut runner = DataCompatibilityTestRunner::new(verbose);
    Ok(runner.run_all_tests().await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_runner_creation() {
        let runner = DataCompatibilityTestRunner::new(true);
        assert_eq!(runner.verbose, true);
    }

    #[test]
    fn test_result_structures() {
        let start_time = Instant::now();
        let result = TestRunnerResult {
            passed: true,
            start_time,
            end_time: start_time,
            data_integrity_result: None,
            encryption_result: None,
            migration_result: None,
            errors: Vec::new(),
            warnings: Vec::new(),
        };
        
        assert!(result.passed);
        assert_eq!(result.errors.len(), 0);
    }
}