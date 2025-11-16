//! 跨平台兼容性测试模块
//! 
//! 提供全面的跨平台兼容性测试，确保AI Manager应用在Windows、macOS和Linux上
//! 的行为一致性和稳定性。

pub mod file_paths;
pub mod config_locations;
pub mod functional_consistency;

// 重新导出主要测试功能
pub use file_paths::*;
pub use config_locations::*;
pub use functional_consistency::*;

/// 跨平台测试套件
pub struct CrossPlatformTestSuite {
    test_results: Vec<functional_consistency::TestResult>,
}

impl CrossPlatformTestSuite {
    /// 创建新的测试套件
    pub fn new() -> Self {
        Self {
            test_results: Vec::new(),
        }
    }
    
    /// 运行所有跨平台测试
    pub fn run_all_tests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("开始运行跨平台兼容性测试套件...");
        
        // 1. 运行文件路径处理测试
        self.run_file_path_tests()?;
        
        // 2. 运行配置文件位置测试
        self.run_config_location_tests()?;
        
        // 3. 运行功能一致性测试
        self.run_functional_consistency_tests()?;
        
        // 4. 生成综合报告
        self.generate_comprehensive_report()?;
        
        Ok(())
    }
    
    /// 运行文件路径处理测试
    fn run_file_path_tests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🔍 运行文件路径处理测试...");
        
        // 这里我们调用具体的测试函数
        file_paths::test_path_separator_handling();
        file_paths::test_path_normalization();
        file_paths::test_platform_specific_paths();
        file_paths::test_filename_validity();
        file_paths::test_path_existence_and_permissions();
        file_paths::test_path_resolution();
        file_paths::test_special_characters_and_unicode();
        file_paths::test_config_path_generation();
        
        println!("✅ 文件路径处理测试完成");
        Ok(())
    }
    
    /// 运行配置文件位置测试
    fn run_config_location_tests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🔧 运行配置文件位置测试...");
        
        config_locations::test_config_directory_access();
        config_locations::test_config_file_creation();
        config_locations::test_json_config_parsing();
        config_locations::test_toml_config_parsing();
        config_locations::test_config_backup_and_restore();
        config_locations::test_config_file_security();
        config_locations::test_config_path_resolution();
        config_locations::test_config_migration();
        
        println!("✅ 配置文件位置测试完成");
        Ok(())
    }
    
    /// 运行功能一致性测试
    fn run_functional_consistency_tests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n⚡ 运行功能一致性测试...");
        
        functional_consistency::test_database_consistency();
        functional_consistency::test_encryption_consistency();
        functional_consistency::test_json_serialization_consistency();
        functional_consistency::test_error_handling_consistency();
        functional_consistency::test_environment_consistency();
        functional_consistency::test_concurrency_consistency();
        
        println!("✅ 功能一致性测试完成");
        Ok(())
    }
    
    /// 生成综合测试报告
    fn generate_comprehensive_report(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n📊 生成跨平台兼容性综合报告...");
        
        let report = format!(
            r#"# AI Manager 跨平台兼容性测试报告

## 测试概述
- **测试时间**: {}
- **操作系统**: {}
- **架构**: {}
- **Rust版本**: {}
- **测试总数**: {}

## 测试模块
1. ✅ 文件路径处理兼容性
2. ✅ 配置文件位置兼容性  
3. ✅ 功能行为一致性

## 结论
所有跨平台兼容性测试已通过验证，AI Manager应用在当前平台上表现良好。

## 详细报告
详细的测试结果和性能数据请参考各个测试模块的输出。
"#,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            std::env::consts::OS,
            std::env::consts::ARCH,
            rustc_version(),
            self.test_results.len()
        );
        
        // 将报告写入文件
        let report_path = std::path::Path::new("target").join("cross-platform-test-report.md");
        if let Some(parent) = report_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&report_path, report)?;
        
        println!("✅ 综合报告已生成: {:?}", report_path);
        Ok(())
    }
}

/// 获取Rust版本信息
fn rustc_version() -> String {
    let output = std::process::Command::new("rustc")
        .arg("--version")
        .output();
    
    match output {
        Ok(output) => String::from_utf8_lossy(&output.stdout).trim().to_string(),
        Err(_) => "未知".to_string(),
    }
}

/// 运行完整的跨平台兼容性测试套件
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn run_complete_cross_platform_tests() {
        let mut suite = CrossPlatformTestSuite::new();
        let result = suite.run_all_tests();
        
        match result {
            Ok(()) => println!("🎉 所有跨平台兼容性测试通过！"),
            Err(e) => panic!("❌ 跨平台兼容性测试失败: {}", e),
        }
    }
}

/// 命令行工具：运行跨平台兼容性测试
#[cfg_attr(test, allow(dead_code))]
pub fn run_cross_platform_tests_cli() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 启动AI Manager跨平台兼容性测试...\n");
    
    // 设置环境变量
    std::env::set_var("RUST_BACKTRACE", "1");
    std::env::set_var("RUST_LOG", "debug");
    
    let mut suite = CrossPlatformTestSuite::new();
    suite.run_all_tests()?;
    
    println!("\n🎯 跨平台兼容性测试完成！");
    println!("📁 详细报告请查看: target/cross-platform-test-report.md");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cross_platform_test_suite_creation() {
        let suite = CrossPlatformTestSuite::new();
        assert_eq!(suite.test_results.len(), 0);
    }
}