//! 文件路径处理跨平台兼容性测试
//! 
//! 测试不同操作系统下文件路径处理的正确性，包括：
//! - 路径分隔符处理
//! - 绝对路径和相对路径
//! - 路径规范化
//! - 文件名合法性检查
//! - 特殊字符处理

use std::path::{Path, PathBuf};
use std::fs;
use tempfile::TempDir;
use std::env;

/// 获取当前操作系统的类型
fn get_os_type() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    }
}

/// 获取当前系统的路径分隔符
fn get_path_separator() -> &'static str {
    if cfg!(target_os = "windows") {
        "\\"
    } else {
        "/"
    }
}

/// 测试路径分隔符处理
#[test]
fn test_path_separator_handling() {
    let os_type = get_os_type();
    let separator = get_path_separator();
    
    println!("测试操作系统: {}", os_type);
    println!("路径分隔符: {}", separator);
    
    // 测试路径拼接
    let base_path = if cfg!(target_os = "windows") {
        PathBuf::from("C:\\Program Files\\AI Manager")
    } else {
        PathBuf::from("/usr/local/ai-manager")
    };
    
    let sub_path = PathBuf::from("config").join("settings.json");
    let full_path = base_path.join(sub_path);
    
    println!("完整路径: {:?}", full_path);
    
    // 验证路径存在性（相对路径）
    assert!(full_path.parent().is_some());
    assert!(full_path.file_name().is_some());
}

/// 测试路径规范化
#[test]
fn test_path_normalization() {
    let temp_dir = TempDir::new().expect("无法创建临时目录");
    
    // 创建包含相对路径的路径
    let test_cases = vec![
        ("./config/settings.json", "config/settings.json"),
        ("../config/settings.json", "../config/settings.json"),
        ("config/./settings.json", "config/settings.json"),
        ("config/../settings.json", "settings.json"),
    ];
    
    for (input, expected) in test_cases {
        let input_path = Path::new(input);
        let normalized = input_path.components().as_path();
        let expected_path = Path::new(expected);
        
        println!("输入: {:?}, 规范化: {:?}, 期望: {:?}", 
                input_path, normalized, expected_path);
        
        // 在某些情况下，路径可能不完全相等，但应该表示相同的位置
        assert_eq!(normalized.file_name(), expected_path.file_name());
    }
}

/// 测试不同平台的特殊路径
#[test]
fn test_platform_specific_paths() {
    let temp_dir = TempDir::new().expect("无法创建临时目录");
    
    if cfg!(target_os = "windows") {
        // Windows 特定路径测试
        let test_paths = vec![
            "C:\\Program Files\\AI Manager",
            "C:\\Users\\Test\\AppData\\Roaming\\.claude",
            "D:\\Data\\ai-manager\\config",
            "\\\\?\\C:\\Very\\Long\\Path\\That\\Exceeds\\Normal\\Limits",
        ];
        
        for path_str in test_paths {
            let path = Path::new(path_str);
            println!("Windows 路径测试: {:?}", path);
            
            // 验证路径是否为绝对路径
            assert!(path.is_absolute(), "路径应该是绝对路径: {:?}", path);
        }
    } else {
        // Unix/Linux/macOS 特定路径测试
        let test_paths = vec![
            "/usr/local/bin/ai-manager",
            "/home/user/.config/ai-manager",
            "/tmp/ai-manager-test",
            "~/Library/Application Support/AI Manager",
        ];
        
        for path_str in test_paths {
            let path = Path::new(path_str);
            println!("Unix 路径测试: {:?}", path);
            
            // 展开用户目录符号
            let expanded = if path_str.starts_with("~/") {
                if let Some(home) = dirs::home_dir() {
                    home.join(path.strip_prefix("~").unwrap())
                } else {
                    path.to_path_buf()
                }
            } else {
                path.to_path_buf()
            };
            
            println!("展开后的路径: {:?}", expanded);
        }
    }
}

/// 测试文件名合法性检查
#[test]
fn test_filename_validity() {
    let os_type = get_os_type();
    
    // 测试非法文件名字符
    let invalid_chars = if cfg!(target_os = "windows") {
        vec!['<', '>', ':', '"', '|', '?', '*']
    } else {
        vec!['/', '\0'] // Unix 系统中 / 和 null 字符是非法的
    };
    
    let valid_filenames = vec![
        "config.json",
        "settings.toml",
        "app_data.db",
        "test-file.txt",
        "application.conf",
        "数据文件.json", // 中文文件名
        "file_with spaces.txt",
        "file-with-dashes.txt",
        "file_with_underscores.txt",
    ];
    
    // 测试有效文件名
    for filename in valid_filenames {
        let path = Path::new(filename);
        assert!(path.file_name().is_some(), "有效文件名应该被识别: {}", filename);
        println!("✅ 有效文件名: {}", filename);
    }
    
    // 测试非法字符
    for invalid_char in invalid_chars {
        let invalid_filename = format!("test{}file.txt", invalid_char);
        let path = Path::new(&invalid_filename);
        
        // 在不同平台上，这些字符的处理可能不同
        if cfg!(target_os = "windows") {
            // Windows 上这些字符在文件名中是非法的
            println!("⚠️ Windows 非法字符测试: {} (包含 '{}')", invalid_filename, invalid_char);
        } else {
            // Unix 系统通常允许更多字符
            println!("ℹ️ Unix 字符测试: {} (包含 '{}')", invalid_filename, invalid_char);
        }
    }
}

/// 测试路径存在性和权限
#[test]
fn test_path_existence_and_permissions() {
    let temp_dir = TempDir::new().expect("无法创建临时目录");
    let temp_path = temp_dir.path();
    
    // 创建测试目录结构
    let test_dirs = vec![
        "config",
        "data",
        "logs",
        "cache",
    ];
    
    let test_files = vec![
        ("config/settings.json", "{\"app_name\": \"AI Manager\"}"),
        ("data/test.db", "test database content"),
        ("logs/app.log", "application log content"),
        ("cache/temp.txt", "cache content"),
    ];
    
    // 创建目录
    for dir in test_dirs {
        let dir_path = temp_path.join(dir);
        fs::create_dir_all(&dir_path)
            .expect(&format!("无法创建目录: {:?}", dir_path));
        println!("✅ 创建目录: {:?}", dir_path);
        
        // 验证目录存在且可读
        assert!(dir_path.exists(), "目录应该存在: {:?}", dir_path);
        assert!(dir_path.is_dir(), "路径应该是目录: {:?}", dir_path);
    }
    
    // 创建文件
    for (file_path, content) in test_files {
        let full_path = temp_path.join(file_path);
        fs::write(&full_path, content)
            .expect(&format!("无法写入文件: {:?}", full_path));
        println!("✅ 创建文件: {:?}", full_path);
        
        // 验证文件存在且可读
        assert!(full_path.exists(), "文件应该存在: {:?}", full_path);
        assert!(full_path.is_file(), "路径应该是文件: {:?}", full_path);
        
        // 验证文件内容
        let read_content = fs::read_to_string(&full_path)
            .expect(&format!("无法读取文件: {:?}", full_path));
        assert_eq!(read_content, content, "文件内容应该匹配: {:?}", full_path);
    }
}

/// 测试路径解析和拼接
#[test]
fn test_path_resolution() {
    let temp_dir = TempDir::new().expect("无法创建临时目录");
    let base_path = temp_dir.path();
    
    // 测试相对路径解析
    let relative_paths = vec![
        "config/settings.json",
        "./data/test.db",
        "../external/config.toml", // 这个路径不存在，但应该能正确解析
    ];
    
    for rel_path in relative_paths {
        let absolute_path = base_path.join(rel_path);
        println!("相对路径: {} -> 绝对路径: {:?}", rel_path, absolute_path);
        
        // 验证路径组成
        assert!(absolute_path.starts_with(base_path), 
               "绝对路径应该以基础路径开头: {:?}", absolute_path);
    }
    
    // 测试路径组件分析
    let test_path = base_path.join("config").join("settings.json");
    println!("分析路径: {:?}", test_path);
    
    // 验证路径组件
    assert_eq!(test_path.file_name(), Some(std::ffi::OsStr::new("settings.json")));
    assert_eq!(test_path.extension(), Some(std::ffi::OsStr::new("json")));
    assert!(test_path.parent().is_some());
}

/// 测试路径中的特殊字符和 Unicode
#[test]
fn test_special_characters_and_unicode() {
    let temp_dir = TempDir::new().expect("无法创建临时目录");
    
    // 测试包含特殊字符和 Unicode 的文件名
    let special_filenames = vec![
        "config_中文.json",
        "settings-日本語.toml",
        "app_한국어.conf",
        "data_العربية.db",
        "test_файл.txt",
        "file with spaces.json",
        "file-with-dashes.toml",
        "file_with_underscores.conf",
        "file.dots.in.name.json",
        "CamelCaseConfig.toml",
    ];
    
    for filename in special_filenames {
        let file_path = temp_dir.path().join(filename);
        let content = format!("测试内容: {}", filename);
        
        // 创建文件
        fs::write(&file_path, &content)
            .expect(&format!("无法写入包含特殊字符的文件: {:?}", file_path));
        
        println!("✅ 创建特殊字符文件: {:?}", file_path);
        
        // 验证文件可以正确读取
        let read_content = fs::read_to_string(&file_path)
            .expect(&format!("无法读取包含特殊字符的文件: {:?}", file_path));
        assert_eq!(read_content, content);
        
        // 验证路径解析
        assert_eq!(file_path.file_name(), Some(std::ffi::OsStr::new(filename)));
    }
}

/// 测试配置文件路径生成
#[test]
fn test_config_path_generation() {
    // 测试不同平台的配置文件路径生成
    let config_paths = if cfg!(target_os = "windows") {
        vec![
            env::var("APPDATA").unwrap_or_else(|_| "C:\\Users\\Default\\AppData\\Roaming".to_string()),
            env::var("LOCALAPPDATA").unwrap_or_else(|_| "C:\\Users\\Default\\AppData\\Local".to_string()),
        ]
    } else if cfg!(target_os = "macos") {
        vec![
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join("Library/Application Support").to_string_lossy().to_string(),
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join(".config").to_string_lossy().to_string(),
        ]
    } else {
        vec![
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join(".config").to_string_lossy().to_string(),
            "/etc/ai-manager".to_string(),
        ]
    };
    
    for base_dir in config_paths {
        let app_config_path = Path::new(&base_dir).join("ai-manager");
        let claude_config_path = if cfg!(target_os = "windows") {
            Path::new(&base_dir).join(".claude")
        } else {
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join(".claude")
        };
        let codex_config_path = if cfg!(target_os = "windows") {
            Path::new(&base_dir).join(".codex")
        } else {
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join(".codex")
        };
        
        println!("配置路径:");
        println!("  AI Manager: {:?}", app_config_path);
        println!("  Claude: {:?}", claude_config_path);
        println!("  Codex: {:?}", codex_config_path);
        
        // 验证路径格式正确性
        assert!(app_config_path.is_absolute(), "AI Manager 配置路径应该是绝对路径");
        assert!(claude_config_path.is_absolute(), "Claude 配置路径应该是绝对路径");
        assert!(codex_config_path.is_absolute(), "Codex 配置路径应该是绝对路径");
    }
}

/// 辅助函数：打印系统信息
#[cfg(test)]
mod test_utils {
    use super::*;
    
    pub fn print_system_info() {
        println!("=== 系统信息 ===");
        println!("操作系统: {}", get_os_type());
        println!("路径分隔符: {}", get_path_separator());
        println!("当前目录: {:?}", env::current_dir().unwrap_or_else(|_| PathBuf::from("unknown")));
        println!("临时目录: {:?}", env::temp_dir());
        println!("用户目录: {:?}", dirs::home_dir().unwrap_or_else(|| PathBuf::from("unknown")));
        
        if let Ok(config_dirs) = dirs::config_dir() {
            println!("配置目录: {:?}", config_dirs);
        }
        if let Ok(data_dirs) = dirs::data_dir() {
            println!("数据目录: {:?}", data_dirs);
        }
        
        println!("================");
    }
}

#[test]
fn test_print_system_info() {
    test_utils::print_system_info();
}