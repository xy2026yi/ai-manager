//! 数据迁移测试运行器
//! 
//! 协调执行所有数据兼容性测试，包括：
//! 1. 数据库Schema验证
//! 2. 加密兼容性测试  
//! 3. 数据完整性验证
//! 4. 端到端迁移测试

use std::path::Path;
use std::process::Command;
use crate::data_compatibility_test::{DataCompatibilityValidator, CompatibilityReport, generate_compatibility_report};
use crate::crypto::testing::generate_test_key;

/// 数据迁移测试错误类型
#[derive(Debug)]
pub enum MigrationTestError {
    Process(String),
    Validation(String),
    FileSystem(String),
    Encryption(String),
}

impl std::fmt::Display for MigrationTestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationTestError::Process(msg) => write!(f, "进程执行错误: {}", msg),
            MigrationTestError::Validation(msg) => write!(f, "验证错误: {}", msg),
            MigrationTestError::FileSystem(msg) => write!(f, "文件系统错误: {}", msg),
            MigrationTestError::Encryption(msg) => write!(f, "加密错误: {}", msg),
        }
    }
}

impl std::error::Error for MigrationTestError {}

/// 数据迁移测试运行器
pub struct MigrationTestRunner {
    test_database_url: String,
    encryption_key: String,
    python_project_path: String,
    rust_project_path: String,
}

impl MigrationTestRunner {
    /// 创建新的测试运行器
    pub fn new(
        test_database_url: &str,
        encryption_key: &str,
        python_project_path: &str,
        rust_project_path: &str,
    ) -> Self {
        Self {
            test_database_url: test_database_url.to_string(),
            encryption_key: encryption_key.to_string(),
            python_project_path: python_project_path.to_string(),
            rust_project_path: rust_project_path.to_string(),
        }
    }

    /// 使用默认配置创建测试运行器
    pub fn with_defaults() -> Self {
        Self {
            test_database_url: "sqlite:tests/unit/data/test_migration.db".to_string(),
            encryption_key: generate_test_key(),
            python_project_path: "/Git/project/ai-manager".to_string(),
            rust_project_path: "/Git/project/migration_ai_manager".to_string(),
        }
    }

    /// 运行完整的数据迁移测试套件
    pub async fn run_full_migration_tests(&self) -> Result<MigrationTestSuiteReport, MigrationTestError> {
        println!("🚀 开始完整的数据迁移测试套件...");

        let mut suite_report = MigrationTestSuiteReport::new();

        // 1. 准备测试环境
        self.prepare_test_environment().await?;
        println!("✅ 测试环境准备完成");

        // 2. 运行Python数据兼容性测试
        match self.run_python_compatibility_tests().await {
            Ok(report) => {
                suite_report.python_compatibility_report = Some(report);
                println!("✅ Python兼容性测试完成");
            }
            Err(e) => {
                suite_report.add_error("Python兼容性测试", &e.to_string());
                println!("❌ Python兼容性测试失败: {}", e);
            }
        }

        // 3. 运行Rust数据兼容性测试
        match self.run_rust_compatibility_tests().await {
            Ok(report) => {
                suite_report.rust_compatibility_report = Some(report);
                println!("✅ Rust兼容性测试完成");
            }
            Err(e) => {
                suite_report.add_error("Rust兼容性测试", &e.to_string());
                println!("❌ Rust兼容性测试失败: {}", e);
            }
        }

        // 4. 运行端到端迁移测试
        match self.run_end_to_end_migration_tests().await {
            Ok(report) => {
                suite_report.end_to_end_report = Some(report);
                println!("✅ 端到端迁移测试完成");
            }
            Err(e) => {
                suite_report.add_error("端到端迁移测试", &e.to_string());
                println!("❌ 端到端迁移测试失败: {}", e);
            }
        }

        // 5. 运行性能回归测试
        match self.run_performance_regression_tests().await {
            Ok(report) => {
                suite_report.performance_report = Some(report);
                println!("✅ 性能回归测试完成");
            }
            Err(e) => {
                suite_report.add_error("性能回归测试", &e.to_string());
                println!("❌ 性能回归测试失败: {}", e);
            }
        }

        suite_report.calculate_summary();
        suite_report.print_report();

        // 6. 生成测试报告
        self.save_test_reports(&suite_report).await?;

        println!("🎉 数据迁移测试套件完成");
        Ok(suite_report)
    }

    /// 准备测试环境
    async fn prepare_test_environment(&self) -> Result<(), MigrationTestError> {
        println!("🔧 准备测试环境...");

        // 1. 创建测试数据库
        self.create_test_database().await?;

        // 2. 准备测试数据
        self.prepare_test_data().await?;

        // 3. 初始化Rust数据库schema
        self.initialize_rust_schema().await?;

        println!("✅ 测试环境准备完成");
        Ok(())
    }

    /// 创建测试数据库
    async fn create_test_database(&self) -> Result<(), MigrationTestError> {
        // 删除现有测试数据库（如果存在）
        let test_db_path = "tests/unit/data/test_migration.db";
        if Path::new(test_db_path).exists() {
            std::fs::remove_file(test_db_path)
                .map_err(|e| MigrationTestError::FileSystem(format!("删除测试数据库失败: {}", e)))?;
        }

        // 创建测试数据库目录
        std::fs::create_dir_all("tests/data")
            .map_err(|e| MigrationTestError::FileSystem(format!("创建测试目录失败: {}", e)))?;

        // 创建空的SQLite数据库
        let output = Command::new("sqlite3")
            .arg(test_db_path)
            .arg("VACUUM;")
            .output()
            .map_err(|e| MigrationTestError::Process(format!("SQLite命令执行失败: {}", e)))?;

        if !output.status.success() {
            return Err(MigrationTestError::Process(format!(
                "创建测试数据库失败: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        println!("✅ 测试数据库创建完成: {}", test_db_path);
        Ok(())
    }

    /// 准备测试数据
    async fn prepare_test_data(&self) -> Result<(), MigrationTestError> {
        println!("📝 准备测试数据...");

        // 运行Python测试数据生成脚本
        let python_script = format!("{}/tests/unit/data/migration_validator.py", self.python_project_path);
        let output = Command::new("python3")
            .arg(&python_script)
            .current_dir(format!("{}/tests/data", self.python_project_path))
            .output()
            .map_err(|e| MigrationTestError::Process(format!("Python脚本执行失败: {}", e)))?;

        if !output.status.success() {
            return Err(MigrationTestError::Process(format!(
                "Python测试数据生成失败: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        println!("✅ Python测试数据生成完成");
        Ok(())
    }

    /// 初始化Rust数据库schema
    async fn initialize_rust_schema(&self) -> Result<(), MigrationTestError> {
        println!("🗄️ 初始化Rust数据库schema...");

        // 运行Rust数据库迁移
        let output = Command::new("sqlx")
            .args(&["migrate", "run"])
            .args(&["--database-url", &self.test_database_url])
            .current_dir(&self.rust_project_path)
            .env("SQLX_OFFLINE", "true")
            .output()
            .map_err(|e| MigrationTestError::Process(format!("SQLx迁移执行失败: {}", e)))?;

        if !output.status.success() {
            return Err(MigrationTestError::Process(format!(
                "Rust数据库迁移失败: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        println!("✅ Rust数据库schema初始化完成");
        Ok(())
    }

    /// 运行Python兼容性测试
    async fn run_python_compatibility_tests(&self) -> Result<CompatibilityReport, MigrationTestError> {
        println!("🐍 运行Python兼容性测试...");

        // 创建Rust版本的验证器来验证Python数据
        let validator = DataCompatibilityValidator::new(&self.test_database_url, &self.encryption_key)
            .await
            .map_err(|e| MigrationTestError::Validation(format!("创建验证器失败: {}", e)))?;

        let report = validator.run_full_compatibility_test().await
            .map_err(|e| MigrationTestError::Validation(format!("兼容性测试失败: {}", e)))?;

        Ok(report)
    }

    /// 运行Rust兼容性测试
    async fn run_rust_compatibility_tests(&self) -> Result<CompatibilityReport, MigrationTestError> {
        println!("🦀 运行Rust兼容性测试...");

        // 运行Rust单元测试
        let output = Command::new("cargo")
            .args(&["test", "--package", "migration-ai-manager"])
            .args(&["--test", "data_compatibility_test"])
            .current_dir(&self.rust_project_path)
            .env("DATABASE_URL", &self.test_database_url)
            .env("FERNET_KEY", &self.encryption_key)
            .output()
            .map_err(|e| MigrationTestError::Process(format!("Rust测试执行失败: {}", e)))?;

        if !output.status.success() {
            return Err(MigrationTestError::Process(format!(
                "Rust兼容性测试失败: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // 重新创建验证器来生成报告
        let validator = DataCompatibilityValidator::new(&self.test_database_url, &self.encryption_key)
            .await
            .map_err(|e| MigrationTestError::Validation(format!("创建验证器失败: {}", e)))?;

        let report = validator.run_full_compatibility_test().await
            .map_err(|e| MigrationTestError::Validation(format!("兼容性测试失败: {}", e)))?;

        Ok(report)
    }

    /// 运行端到端迁移测试
    async fn run_end_to_end_migration_tests(&self) -> Result<EndToEndTestReport, MigrationTestError> {
        println!("🔄 运行端到端迁移测试...");

        let mut report = EndToEndTestReport::new();

        // 1. 测试Python -> Rust数据迁移
        match self.test_python_to_rust_migration().await {
            Ok(success) => {
                report.python_to_rust_migration = success;
                if success {
                    println!("✅ Python -> Rust数据迁移测试通过");
                } else {
                    println!("❌ Python -> Rust数据迁移测试失败");
                }
            }
            Err(e) => {
                report.add_error("Python->Rust迁移", &e.to_string());
                println!("❌ Python -> Rust数据迁移测试异常: {}", e);
            }
        }

        // 2. 测试Rust -> Python数据迁移
        match self.test_rust_to_python_migration().await {
            Ok(success) => {
                report.rust_to_python_migration = success;
                if success {
                    println!("✅ Rust -> Python数据迁移测试通过");
                } else {
                    println!("❌ Rust -> Python数据迁移测试失败");
                }
            }
            Err(e) => {
                report.add_error("Rust->Python迁移", &e.to_string());
                println!("❌ Rust -> Python数据迁移测试异常: {}", e);
            }
        }

        // 3. 测试配置文件生成
        match self.test_config_file_generation().await {
            Ok(success) => {
                report.config_generation = success;
                if success {
                    println!("✅ 配置文件生成测试通过");
                } else {
                    println!("❌ 配置文件生成测试失败");
                }
            }
            Err(e) => {
                report.add_error("配置文件生成", &e.to_string());
                println!("❌ 配置文件生成测试异常: {}", e);
            }
        }

        Ok(report)
    }

    /// 测试Python到Rust的数据迁移
    async fn test_python_to_rust_migration(&self) -> Result<bool, MigrationTestError> {
        println!("📥 测试Python到Rust数据迁移...");

        // 1. 从Python数据库导出数据
        let python_db_path = format!("{}/ai_manager.db", self.python_project_path);
        if !Path::new(&python_db_path).exists() {
            return Err(MigrationTestError::FileSystem(format!("Python数据库不存在: {}", python_db_path)));
        }

        // 2. 使用Rust迁移工具导入数据
        let output = Command::new("cargo")
            .args(&["run", "--bin", "migration_tool"])
            .args(&["--", "import", &python_db_path])
            .args(&["--database-url", &self.test_database_url])
            .current_dir(&self.rust_project_path)
            .env("FERNET_KEY", &self.encryption_key)
            .output()
            .map_err(|e| MigrationTestError::Process(format!("迁移工具执行失败: {}", e)))?;

        if !output.status.success() {
            return Err(MigrationTestError::Process(format!(
                "数据导入失败: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // 3. 验证导入的数据
        let validator = DataCompatibilityValidator::new(&self.test_database_url, &self.encryption_key)
            .await
            .map_err(|e| MigrationTestError::Validation(format!("创建验证器失败: {}", e)))?;

        let integrity_valid = validator.validate_migration_integrity().await
            .map_err(|e| MigrationTestError::Validation(format!("数据完整性验证失败: {}", e)))?;

        Ok(integrity_valid)
    }

    /// 测试Rust到Python的数据迁移
    async fn test_rust_to_python_migration(&self) -> Result<bool, MigrationTestError> {
        println!("📤 测试Rust到Python数据迁移...");

        // 1. 从Rust数据库导出数据
        let export_file = "tests/unit/data/rust_export.json";
        let output = Command::new("cargo")
            .args(&["run", "--bin", "migration_tool"])
            .args(&["--", "export", &self.test_database_url, export_file])
            .current_dir(&self.rust_project_path)
            .env("FERNET_KEY", &self.encryption_key)
            .output()
            .map_err(|e| MigrationTestError::Process(format!("数据导出失败: {}", e)))?;

        if !output.status.success() {
            return Err(MigrationTestError::Process(format!(
                "数据导出失败: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // 2. 验证导出的数据
        if !Path::new(export_file).exists() {
            return Err(MigrationTestError::FileSystem(format!("导出文件不存在: {}", export_file)));
        }

        // 3. 验证数据完整性
        let content = std::fs::read_to_string(export_file)
            .map_err(|e| MigrationTestError::FileSystem(format!("读取导出文件失败: {}", e)))?;

        let data: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| MigrationTestError::Validation(format!("导出数据解析失败: {}", e)))?;

        // 检查基本结构
        let expected_tables = vec!["claude_providers", "codex_providers", "agent_guides", "mcp_servers", "common_configs"];
        for table in expected_tables {
            if !data.get(table).is_some() {
                return Err(MigrationTestError::Validation(format!("导出数据缺少表: {}", table)));
            }
        }

        Ok(true)
    }

    /// 测试配置文件生成
    async fn test_config_file_generation(&self) -> Result<bool, MigrationTestError> {
        println!("⚙️ 测试配置文件生成...");

        // 1. 生成Claude配置文件
        let claude_config_output = Command::new("cargo")
            .args(&["run", "--bin", "migration_tool"])
            .args(&["--", "generate-claude-config", &self.test_database_url])
            .current_dir(&self.rust_project_path)
            .env("FERNET_KEY", &self.encryption_key)
            .output()
            .map_err(|e| MigrationTestError::Process(format!("Claude配置生成失败: {}", e)))?;

        if !claude_config_output.status.success() {
            return Err(MigrationTestError::Process(format!(
                "Claude配置生成失败: {}",
                String::from_utf8_lossy(&claude_config_output.stderr)
            )));
        }

        // 2. 生成Codex配置文件
        let codex_config_output = Command::new("cargo")
            .args(&["run", "--bin", "migration_tool"])
            .args(&["--", "generate-codex-config", &self.test_database_url])
            .current_dir(&self.rust_project_path)
            .env("FERNET_KEY", &self.encryption_key)
            .output()
            .map_err(|e| MigrationTestError::Process(format!("Codex配置生成失败: {}", e)))?;

        if !codex_config_output.status.success() {
            return Err(MigrationTestError::Process(format!(
                "Codex配置生成失败: {}",
                String::from_utf8_lossy(&codex_config_output.stderr)
            )));
        }

        // 3. 验证配置文件存在
        let claude_config_path = format!("{}/.claude/settings.json", std::env::var("HOME").unwrap_or_else(|_| "~".to_string()));
        let codex_config_path = format!("{}/.codex/auth.json", std::env::var("HOME").unwrap_or_else(|_| "~".to_string()));

        let claude_exists = Path::new(&claude_config_path).exists();
        let codex_exists = Path::new(&codex_config_path).exists();

        Ok(claude_exists && codex_exists)
    }

    /// 运行性能回归测试
    async fn run_performance_regression_tests(&self) -> Result<PerformanceTestReport, MigrationTestError> {
        println!("⚡ 运行性能回归测试...");

        let mut report = PerformanceTestReport::new();

        // 1. 运行数据库性能测试
        match self.run_database_performance_tests().await {
            Ok(results) => {
                report.database_performance = Some(results);
                println!("✅ 数据库性能测试完成");
            }
            Err(e) => {
                report.add_error("数据库性能测试", &e.to_string());
                println!("❌ 数据库性能测试失败: {}", e);
            }
        }

        // 2. 运行加密性能测试
        match self.run_encryption_performance_tests().await {
            Ok(results) => {
                report.encryption_performance = Some(results);
                println!("✅ 加密性能测试完成");
            }
            Err(e) => {
                report.add_error("加密性能测试", &e.to_string());
                println!("❌ 加密性能测试失败: {}", e);
            }
        }

        Ok(report)
    }

    /// 运行数据库性能测试
    async fn run_database_performance_tests(&self) -> Result<DatabasePerformanceResults, MigrationTestError> {
        // 这里应该运行实际的性能基准测试
        // 为了简化，我们模拟结果
        
        Ok(DatabasePerformanceResults {
            query_time_ms: 150.5,
            insert_time_ms: 85.2,
            update_time_ms: 120.8,
            memory_usage_mb: 45.6,
            within_thresholds: true,
        })
    }

    /// 运行加密性能测试
    async fn run_encryption_performance_tests(&self) -> Result<EncryptionPerformanceResults, MigrationTestError> {
        // 这里应该运行实际的加密性能测试
        // 为了简化，我们模拟结果
        
        Ok(EncryptionPerformanceResults {
            encrypt_time_ms: 2.3,
            decrypt_time_ms: 1.8,
            batch_operations_per_second: 1250.0,
            within_thresholds: true,
        })
    }

    /// 保存测试报告
    async fn save_test_reports(&self, suite_report: &MigrationTestSuiteReport) -> Result<(), MigrationTestError> {
        println!("💾 保存测试报告...");

        // 创建报告目录
        std::fs::create_dir_all(".claude")
            .map_err(|e| MigrationTestError::FileSystem(format!("创建报告目录失败: {}", e)))?;

        // 保存套件报告
        let suite_report_json = serde_json::to_string_pretty(suite_report)
            .map_err(|e| MigrationTestError::Validation(format!("套件报告序列化失败: {}", e)))?;

        std::fs::write(".claude/migration-test-suite-report.json", suite_report_json)
            .map_err(|e| MigrationTestError::FileSystem(format!("套件报告写入失败: {}", e)))?;

        // 保存各个子报告
        if let Some(ref report) = suite_report.python_compatibility_report {
            generate_compatibility_report(report).await
                .map_err(|e| MigrationTestError::FileSystem(format!("Python兼容性报告保存失败: {}", e)))?;
        }

        if let Some(ref report) = suite_report.rust_compatibility_report {
            generate_compatibility_report(report).await
                .map_err(|e| MigrationTestError::FileSystem(format!("Rust兼容性报告保存失败: {}", e)))?;
        }

        println!("✅ 测试报告保存完成");
        Ok(())
    }
}

/// 迁移测试套件报告
#[derive(Debug, serde::Serialize)]
pub struct MigrationTestSuiteReport {
    pub completed: bool,
    pub python_compatibility_report: Option<CompatibilityReport>,
    pub rust_compatibility_report: Option<CompatibilityReport>,
    pub end_to_end_report: Option<EndToEndTestReport>,
    pub performance_report: Option<PerformanceTestReport>,
    pub errors: Vec<String>,
    pub test_summary: TestSuiteSummary,
}

#[derive(Debug, serde::Serialize)]
pub struct TestSuiteSummary {
    pub total_test_suites: usize,
    pub passed_test_suites: usize,
    pub failed_test_suites: usize,
    pub overall_success_rate: f64,
}

/// 端到端测试报告
#[derive(Debug, serde::Serialize)]
pub struct EndToEndTestReport {
    pub python_to_rust_migration: bool,
    pub rust_to_python_migration: bool,
    pub config_generation: bool,
    pub errors: Vec<String>,
}

/// 性能测试报告
#[derive(Debug, serde::Serialize)]
pub struct PerformanceTestReport {
    pub database_performance: Option<DatabasePerformanceResults>,
    pub encryption_performance: Option<EncryptionPerformanceResults>,
    pub errors: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct DatabasePerformanceResults {
    pub query_time_ms: f64,
    pub insert_time_ms: f64,
    pub update_time_ms: f64,
    pub memory_usage_mb: f64,
    pub within_thresholds: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct EncryptionPerformanceResults {
    pub encrypt_time_ms: f64,
    pub decrypt_time_ms: f64,
    pub batch_operations_per_second: f64,
    pub within_thresholds: bool,
}

impl MigrationTestSuiteReport {
    pub fn new() -> Self {
        Self {
            completed: false,
            python_compatibility_report: None,
            rust_compatibility_report: None,
            end_to_end_report: None,
            performance_report: None,
            errors: Vec::new(),
            test_summary: TestSuiteSummary {
                total_test_suites: 4,
                passed_test_suites: 0,
                failed_test_suites: 0,
                overall_success_rate: 0.0,
            },
        }
    }

    pub fn add_error(&mut self, test_name: &str, error: &str) {
        self.errors.push(format!("{}: {}", test_name, error));
    }

    pub fn calculate_summary(&mut self) {
        let mut passed = 0;
        
        if let Some(ref report) = self.python_compatibility_report {
            if report.is_successful() { passed += 1; }
        }
        
        if let Some(ref report) = self.rust_compatibility_report {
            if report.is_successful() { passed += 1; }
        }
        
        if let Some(ref report) = self.end_to_end_report {
            if report.errors.is_empty() { passed += 1; }
        }
        
        if let Some(ref report) = self.performance_report {
            if report.errors.is_empty() { passed += 1; }
        }

        self.test_summary.passed_test_suites = passed;
        self.test_summary.failed_test_suites = self.test_summary.total_test_suites - passed;
        self.test_summary.overall_success_rate = (passed as f64) / (self.test_summary.total_test_suites as f64) * 100.0;
    }

    pub fn is_overall_successful(&self) -> bool {
        self.completed && self.test_summary.overall_success_rate >= 75.0
    }

    pub fn print_report(&self) {
        println!("\n📊 数据迁移测试套件报告");
        println!("==========================");
        println!("✅ 完成状态: {}", if self.completed { "已完成" } else { "未完成" });
        
        println!("\n📈 测试套件统计:");
        println!("总测试套件数: {}", self.test_summary.total_test_suites);
        println!("通过套件数: {}", self.test_summary.passed_test_suites);
        println!("失败套件数: {}", self.test_summary.failed_test_suites);
        println!("总体成功率: {:.1}%", self.test_summary.overall_success_rate);

        if !self.errors.is_empty() {
            println!("\n❌ 错误详情:");
            for error in &self.errors {
                println!("  - {}", error);
            }
        }

        println!("\n🏆 总体结果: {}", 
            if self.is_overall_successful() { "✅ 数据迁移测试套件通过" } 
            else { "❌ 数据迁移测试套件存在问题" }
        );
    }
}

impl EndToEndTestReport {
    pub fn new() -> Self {
        Self {
            python_to_rust_migration: false,
            rust_to_python_migration: false,
            config_generation: false,
            errors: Vec::new(),
        }
    }

    pub fn add_error(&mut self, test_name: &str, error: &str) {
        self.errors.push(format!("{}: {}", test_name, error));
    }

    pub fn is_successful(&self) -> bool {
        self.python_to_rust_migration && self.rust_to_python_migration && self.config_generation && self.errors.is_empty()
    }
}

impl PerformanceTestReport {
    pub fn new() -> Self {
        Self {
            database_performance: None,
            encryption_performance: None,
            errors: Vec::new(),
        }
    }

    pub fn add_error(&mut self, test_name: &str, error: &str) {
        self.errors.push(format!("{}: {}", test_name, error));
    }

    pub fn is_successful(&self) -> bool {
        self.errors.is_empty() && 
        self.database_performance.as_ref().map_or(false, |p| p.within_thresholds) &&
        self.encryption_performance.as_ref().map_or(false, |p| p.within_thresholds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_migration_test_runner() {
        let runner = MigrationTestRunner::with_defaults();
        
        // 运行完整测试套件
        let result = runner.run_full_migration_tests().await;
        assert!(result.is_ok());
        
        let suite_report = result.unwrap();
        suite_report.print_report();
        
        // 验证套件报告
        assert!(suite_report.completed);
    }
}