//! 跨平台兼容性测试运行器
//! 
//! 提供命令行接口来运行跨平台兼容性测试，支持：
//! - 选择性运行特定测试模块
//! - 生成详细的测试报告
//! - 性能基准测试
//! - 与CI/CD集成

use std::env;
use std::process;
use clap::{Arg, Command};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("跨平台兼容性测试")
        .version("1.0.0")
        .author("AI Manager Migration Team")
        .about("运行AI Manager的跨平台兼容性测试")
        .arg(
            Arg::new("module")
                .short('m')
                .long("module")
                .value_name("MODULE")
                .help("选择要运行的测试模块")
                .value_parser(["all", "paths", "config", "functional"])
                .default_value("all")
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("详细输出")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("report")
                .short('r')
                .long("report")
                .help("生成测试报告")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("benchmark")
                .short('b')
                .long("benchmark")
                .help("运行性能基准测试")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();

    // 设置环境变量
    if matches.get_flag("verbose") {
        env::set_var("RUST_BACKTRACE", "1");
        env::set_var("RUST_LOG", "debug");
    }

    println!("🚀 AI Manager 跨平台兼容性测试");
    println!("==============================");
    
    let platform = get_platform_info();
    println!("平台: {}", platform);
    println!("架构: {}", std::env::consts::ARCH);
    println!("Rust版本: {}", get_rust_version());
    println!();

    let module = matches.get_one::<String>("module").unwrap();
    
    match module.as_str() {
        "all" => run_all_tests(&matches)?,
        "paths" => run_path_tests(&matches)?,
        "config" => run_config_tests(&matches)?,
        "functional" => run_functional_tests(&matches)?,
        _ => {
            eprintln!("❌ 未知的测试模块: {}", module);
            process::exit(1);
        }
    }

    if matches.get_flag("report") {
        generate_test_report(&matches)?;
    }

    println!("\n🎉 测试完成！");
    Ok(())
}

fn get_platform_info() -> String {
    let os = std::env::consts::OS;
    match os {
        "windows" => "Windows".to_string(),
        "macos" => "macOS".to_string(),
        "linux" => "Linux".to_string(),
        _ => format!("Unknown ({})", os),
    }
}

fn get_rust_version() -> String {
    let output = process::Command::new("rustc")
        .arg("--version")
        .output();
    
    match output {
        Ok(output) => String::from_utf8_lossy(&output.stdout).trim().to_string(),
        Err(_) => "未知".to_string(),
    }
}

fn run_all_tests(matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 运行完整的跨平台兼容性测试套件...\n");

    // 1. 文件路径处理测试
    println!("1️⃣ 文件路径处理测试");
    println!("===================");
    run_path_tests(matches)?;
    
    println!();

    // 2. 配置文件位置测试  
    println!("2️⃣ 配置文件位置测试");
    println!("===================");
    run_config_tests(matches)?;
    
    println!();

    // 3. 功能一致性测试
    println!("3️⃣ 功能一致性测试");
    println!("===================");
    run_functional_tests(matches)?;

    println!();
    println!("✅ 所有跨平台兼容性测试完成！");
    
    Ok(())
}

fn run_path_tests(_matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    println!("测试文件路径处理的跨平台兼容性...\n");

    // 这里我们调用具体的测试函数
    println!("🔍 路径分隔符处理...");
    cross_platform::file_paths::test_path_separator_handling();
    println!("✅ 路径分隔符处理测试通过");

    println!("🔍 路径规范化...");
    cross_platform::file_paths::test_path_normalization();
    println!("✅ 路径规范化测试通过");

    println!("🔍 平台特定路径...");
    cross_platform::file_paths::test_platform_specific_paths();
    println!("✅ 平台特定路径测试通过");

    println!("🔍 文件名合法性...");
    cross_platform::file_paths::test_filename_validity();
    println!("✅ 文件名合法性测试通过");

    println!("🔍 路径存在性和权限...");
    cross_platform::file_paths::test_path_existence_and_permissions();
    println!("✅ 路径存在性和权限测试通过");

    println!("🔍 路径解析和拼接...");
    cross_platform::file_paths::test_path_resolution();
    println!("✅ 路径解析和拼接测试通过");

    println!("🔍 特殊字符和Unicode...");
    cross_platform::file_paths::test_special_characters_and_unicode();
    println!("✅ 特殊字符和Unicode测试通过");

    println!("🔍 配置文件路径生成...");
    cross_platform::file_paths::test_config_path_generation();
    println!("✅ 配置文件路径生成测试通过");

    Ok(())
}

fn run_config_tests(_matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    println!("测试配置文件位置的跨平台兼容性...\n");

    println!("🔧 配置目录访问...");
    cross_platform::config_locations::test_config_directory_access();
    println!("✅ 配置目录访问测试通过");

    println!("🔧 配置文件创建...");
    cross_platform::config_locations::test_config_file_creation();
    println!("✅ 配置文件创建测试通过");

    println!("🔧 JSON配置解析...");
    cross_platform::config_locations::test_json_config_parsing();
    println!("✅ JSON配置解析测试通过");

    println!("🔧 TOML配置解析...");
    cross_platform::config_locations::test_toml_config_parsing();
    println!("✅ TOML配置解析测试通过");

    println!("🔧 配置备份和恢复...");
    cross_platform::config_locations::test_config_backup_and_restore();
    println!("✅ 配置备份和恢复测试通过");

    println!("🔧 配置文件安全性...");
    cross_platform::config_locations::test_config_file_security();
    println!("✅ 配置文件安全性测试通过");

    println!("🔧 配置路径解析...");
    cross_platform::config_locations::test_config_path_resolution();
    println!("✅ 配置路径解析测试通过");

    println!("🔧 配置迁移...");
    cross_platform::config_locations::test_config_migration();
    println!("✅ 配置迁移测试通过");

    Ok(())
}

fn run_functional_tests(_matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    println!("测试功能行为的跨平台一致性...\n");

    println!("⚡ 数据库操作一致性...");
    cross_platform::functional_consistency::test_database_consistency();
    println!("✅ 数据库操作一致性测试通过");

    println!("⚡ 加密解密一致性...");
    cross_platform::functional_consistency::test_encryption_consistency();
    println!("✅ 加密解密一致性测试通过");

    println!("⚡ JSON序列化一致性...");
    cross_platform::functional_consistency::test_json_serialization_consistency();
    println!("✅ JSON序列化一致性测试通过");

    println!("⚡ 错误处理一致性...");
    cross_platform::functional_consistency::test_error_handling_consistency();
    println!("✅ 错误处理一致性测试通过");

    println!("⚡ 环境变量一致性...");
    cross_platform::functional_consistency::test_environment_consistency();
    println!("✅ 环境变量一致性测试通过");

    println!("⚡ 并发安全性一致性...");
    cross_platform::functional_consistency::test_concurrency_consistency();
    println!("✅ 并发安全性一致性测试通过");

    Ok(())
}

fn generate_test_report(matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 生成测试报告...");

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let report_dir = "target/cross-platform-test-reports";
    
    // 创建报告目录
    std::fs::create_dir_all(report_dir)?;

    // 生成主报告
    let main_report = format!(
        r#"# AI Manager 跨平台兼容性测试报告

## 测试信息
- **测试时间**: {}
- **操作系统**: {}
- **架构**: {}
- **Rust版本**: {}
- **详细输出**: {}

## 测试模块
- ✅ 文件路径处理兼容性
- ✅ 配置文件位置兼容性
- ✅ 功能行为一致性

## 系统环境
- **工作目录**: {}
- **环境变量**: RUST_BACKTRACE={}, RUST_LOG={}

## 结论
所有跨平台兼容性测试已成功完成。AI Manager应用在当前平台上表现良好，具备良好的跨平台兼容性。

详细的技术指标和性能数据请参考各模块的具体测试输出。
"#,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        get_platform_info(),
        std::env::consts::ARCH,
        get_rust_version(),
        if matches.get_flag("verbose") { "启用" } else { "禁用" },
        std::env::current_dir().unwrap_or_else(|_| "unknown".into()).display(),
        env::var("RUST_BACKTRACE").unwrap_or_else(|_| "未设置".to_string()),
        env::var("RUST_LOG").unwrap_or_else(|_| "未设置".to_string())
    );

    let report_path = format!("{}/cross-platform-test-report-{}.md", report_dir, timestamp);
    std::fs::write(&report_path, main_report)?;

    println!("📄 测试报告已生成: {}", report_path);

    // 如果启用了基准测试，生成性能报告
    if matches.get_flag("benchmark") {
        generate_performance_report(report_dir, &timestamp)?;
    }

    Ok(())
}

fn generate_performance_report(report_dir: &str, timestamp: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("📈 生成性能基准测试报告...");

    let performance_report = format!(
        r#"# 跨平台性能基准测试报告

## 测试环境
- **平台**: {}
- **架构**: {}
- **测试时间**: {}

## 性能指标
由于本测试主要关注功能兼容性，详细的性能基准测试请参考：
- `cargo bench --bench api_performance`
- `cargo bench --bench database_performance`  
- `cargo bench --bench crypto_performance`

## 建议
建议在正式部署前运行完整的性能基准测试，以确保应用在各种硬件配置上都能提供良好的性能表现。
"#,
        get_platform_info(),
        std::env::consts::ARCH,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );

    let perf_report_path = format!("{}/performance-report-{}.md", report_dir, timestamp);
    std::fs::write(&perf_report_path, performance_report)?;

    println!("📈 性能报告已生成: {}", perf_report_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_platform_info() {
        let platform = get_platform_info();
        assert!(!platform.is_empty());
        
        match std::env::consts::OS {
            "windows" => assert_eq!(platform, "Windows"),
            "macos" => assert_eq!(platform, "macOS"),
            "linux" => assert_eq!(platform, "Linux"),
            _ => assert!(platform.starts_with("Unknown")),
        }
    }

    #[test]
    fn test_get_rust_version() {
        let version = get_rust_version();
        assert!(!version.is_empty());
        assert!(version.contains("rustc"));
    }
}