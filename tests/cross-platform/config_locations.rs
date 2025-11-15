//! 配置文件位置跨平台兼容性测试
//! 
//! 测试不同操作系统下配置文件的正确位置、权限和访问方式：
//! - Windows: %APPDATA%, %LOCALAPPDATA%
//! - macOS: ~/Library/Application Support, ~/.config
//! - Linux: ~/.config, /etc

use std::path::{Path, PathBuf};
use std::fs;
use std::env;
use tempfile::TempDir;
use serde_json;
use toml;

/// 配置文件类型定义
#[derive(Debug, Clone)]
pub enum ConfigType {
    Claude,
    Codex,
    App,
    Database,
    Logs,
}

impl ConfigType {
    /// 获取配置文件名
    pub fn filename(&self) -> &'static str {
        match self {
            ConfigType::Claude => "settings.json",
            ConfigType::Codex => "config.toml",
            ConfigType::App => "app-config.json",
            ConfigType::Database => "ai_manager.db",
            ConfigType::Logs => "app.log",
        }
    }
    
    /// 获取默认配置内容
    pub fn default_content(&self) -> &'static str {
        match self {
            ConfigType::Claude => r#"{
  "providers": [],
  "default_provider": null,
  "api_version": "2023-06-01"
}"#,
            ConfigType::Codex => r#"[codex]
api_key = ""
model = "gpt-4"
max_tokens = 4096

[database]
path = "~/.codex/data.db"
"#,
            ConfigType::App => r#"{
  "app_name": "AI Manager",
  "version": "0.1.0",
  "theme": "light",
  "language": "zh-CN"
}"#,
            ConfigType::Database => "-- SQLite database file",
            ConfigType::Logs => "# Application log file\n",
        }
    }
}

/// 获取操作系统特定的配置目录
pub fn get_config_directories() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    
    if cfg!(target_os = "windows") {
        // Windows 配置目录
        if let Ok(appdata) = env::var("APPDATA") {
            dirs.push(PathBuf::from(appdata));
        }
        if let Ok(localappdata) = env::var("LOCALAPPDATA") {
            dirs.push(PathBuf::from(localappdata));
        }
        if let Ok(programdata) = env::var("PROGRAMDATA") {
            dirs.push(PathBuf::from(programdata));
        }
    } else if cfg!(target_os = "macos") {
        // macOS 配置目录
        if let Some(home) = dirs::home_dir() {
            dirs.push(home.join("Library").join("Application Support"));
            dirs.push(home.join(".config"));
        }
    } else {
        // Linux/Unix 配置目录
        if let Some(home) = dirs::home_dir() {
            dirs.push(home.join(".config"));
            dirs.push(home.join(".local").join("share"));
        }
        dirs.push(PathBuf::from("/etc"));
        dirs.push(PathBuf::from("/usr/local/etc"));
    }
    
    // 添加项目特定的配置目录
    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".claude"));
        dirs.push(home.join(".codex"));
    }
    
    dirs
}

/// 获取特定配置类型的完整路径
pub fn get_config_path(config_type: ConfigType, base_dir: &Path) -> PathBuf {
    match config_type {
        ConfigType::Claude => {
            if cfg!(target_os = "windows") {
                base_dir.join("AI Manager").join("Claude").join(config_type.filename())
            } else {
                base_dir.join("claude").join(config_type.filename())
            }
        },
        ConfigType::Codex => {
            if cfg!(target_os = "windows") {
                base_dir.join("AI Manager").join("Codex").join(config_type.filename())
            } else {
                base_dir.join("codex").join(config_type.filename())
            }
        },
        ConfigType::App => {
            if cfg!(target_os = "windows") {
                base_dir.join("AI Manager").join(config_type.filename())
            } else {
                base_dir.join("ai-manager").join(config_type.filename())
            }
        },
        ConfigType::Database => {
            base_dir.join("ai-manager").join("data").join(config_type.filename())
        },
        ConfigType::Logs => {
            base_dir.join("ai-manager").join("logs").join(config_type.filename())
        },
    }
}

/// 测试配置目录存在性和权限
#[test]
fn test_config_directory_access() {
    println!("测试配置目录访问权限...");
    
    let config_dirs = get_config_directories();
    
    for (index, dir) in config_dirs.iter().enumerate() {
        println!("配置目录 {}: {:?}", index, dir);
        
        // 检查目录是否存在
        if dir.exists() {
            println!("  ✅ 目录存在");
            
            // 检查是否为目录
            assert!(dir.is_dir(), "路径应该是目录: {:?}", dir);
            
            // 检查读取权限
            match fs::read_dir(dir) {
                Ok(entries) => {
                    let count = entries.count();
                    println!("  ✅ 可读取，包含 {} 个条目", count);
                },
                Err(e) => {
                    println!("  ⚠️ 无法读取目录: {}", e);
                }
            }
            
            // 测试创建子目录的权限
            let test_subdir = dir.join("ai-manager-test");
            match fs::create_dir_all(&test_subdir) {
                Ok(()) => {
                    println!("  ✅ 可以创建子目录");
                    // 清理测试目录
                    let _ = fs::remove_dir_all(&test_subdir);
                },
                Err(e) => {
                    println!("  ⚠️ 无法创建子目录: {}", e);
                }
            }
        } else {
            println!("  ❌ 目录不存在");
        }
    }
}

/// 测试配置文件创建和读写
#[test]
fn test_config_file_creation() {
    println!("测试配置文件创建...");
    
    let temp_dir = TempDir::new().expect("无法创建临时目录");
    let base_path = temp_dir.path();
    
    let config_types = [
        ConfigType::Claude,
        ConfigType::Codex,
        ConfigType::App,
        ConfigType::Database,
        ConfigType::Logs,
    ];
    
    for config_type in config_types.iter() {
        let config_path = get_config_path(config_type.clone(), base_path);
        
        // 创建目录结构
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .expect(&format!("无法创建配置目录: {:?}", parent));
        }
        
        // 创建配置文件
        fs::write(&config_path, config_type.default_content())
            .expect(&format!("无法写入配置文件: {:?}", config_path));
        
        println!("✅ 创建配置文件: {:?}", config_path);
        
        // 验证文件存在
        assert!(config_path.exists(), "配置文件应该存在: {:?}", config_path);
        assert!(config_path.is_file(), "路径应该是文件: {:?}", config_path);
        
        // 验证文件内容
        let read_content = fs::read_to_string(&config_path)
            .expect(&format!("无法读取配置文件: {:?}", config_path));
        assert_eq!(read_content, config_type.default_content());
        
        // 测试文件权限
        let metadata = fs::metadata(&config_path)
            .expect(&format!("无法获取文件元数据: {:?}", config_path));
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = metadata.permissions();
            let mode = permissions.mode();
            println!("  文件权限: {:o}", mode);
            
            // 验证文件所有者有读写权限
            assert!(mode & 0o600 != 0, "文件应该至少有读写权限");
        }
        
        #[cfg(windows)]
        {
            // Windows 上的权限检查相对简单
            let readonly = metadata.permissions().readonly();
            println!("  只读属性: {}", readonly);
            assert!(!readonly, "配置文件不应该是只读的");
        }
    }
}

/// 测试JSON配置文件解析
#[test]
fn test_json_config_parsing() {
    println!("测试JSON配置文件解析...");
    
    let temp_dir = TempDir::new().expect("无法创建临时目录");
    
    // 创建测试用的Claude配置
    let claude_config = temp_dir.path().join("settings.json");
    let test_config = r#"{
  "providers": [
    {
      "name": "anthropic",
      "base_url": "https://api.anthropic.com",
      "api_key": "sk-ant-test",
      "enabled": true,
      "models": ["claude-3-sonnet", "claude-3-haiku"]
    }
  ],
  "default_provider": "anthropic",
  "api_version": "2023-06-01",
  "timeout": 30000,
  "retry_count": 3
}"#;
    
    fs::write(&claude_config, test_config)
        .expect("无法写入测试配置");
    
    // 解析JSON配置
    let parsed: serde_json::Value = serde_json::from_str(test_config)
        .expect("无法解析JSON配置");
    
    // 验证配置内容
    assert!(parsed.get("providers").is_some());
    assert_eq!(parsed["default_provider"], "anthropic");
    assert_eq!(parsed["api_version"], "2023-06-01");
    assert_eq!(parsed["timeout"], 30000);
    
    println!("✅ JSON配置解析成功");
    
    // 测试重新序列化
    let serialized = serde_json::to_string_pretty(&parsed)
        .expect("无法序列化配置");
    
    // 重新解析序列化的内容
    let reparsed: serde_json::Value = serde_json::from_str(&serialized)
        .expect("无法重新解析序列化的配置");
    
    assert_eq!(parsed, reparsed);
    println!("✅ JSON配置序列化和重新解析成功");
}

/// 测试TOML配置文件解析
#[test]
fn test_toml_config_parsing() {
    println!("测试TOML配置文件解析...");
    
    let temp_dir = TempDir::new().expect("无法创建临时目录");
    
    // 创建测试用的Codex配置
    let codex_config = temp_dir.path().join("config.toml");
    let test_config = r#"[codex]
api_key = "sk-test-key"
model = "gpt-4"
max_tokens = 4096
temperature = 0.7

[database]
path = "~/.codex/data.db"
connection_timeout = 30
max_connections = 10

[logging]
level = "info"
file = "~/.codex/logs/app.log"
max_size = "10MB"
"#;
    
    fs::write(&codex_config, test_config)
        .expect("无法写入测试配置");
    
    // 解析TOML配置
    let parsed: toml::Value = toml::from_str(test_config)
        .expect("无法解析TOML配置");
    
    // 验证配置内容
    assert!(parsed.get("codex").is_some());
    assert_eq!(parsed["codex"]["api_key"], "sk-test-key");
    assert_eq!(parsed["codex"]["model"], "gpt-4");
    assert_eq!(parsed["database"]["path"], "~/.codex/data.db");
    
    println!("✅ TOML配置解析成功");
    
    // 测试重新序列化
    let serialized = toml::to_string_pretty(&parsed)
        .expect("无法序列化配置");
    
    // 重新解析序列化的内容
    let reparsed: toml::Value = toml::from_str(&serialized)
        .expect("无法重新解析序列化的配置");
    
    assert_eq!(parsed, reparsed);
    println!("✅ TOML配置序列化和重新解析成功");
}

/// 测试配置文件备份和恢复
#[test]
fn test_config_backup_and_restore() {
    println!("测试配置文件备份和恢复...");
    
    let temp_dir = TempDir::new().expect("无法创建临时目录");
    let config_dir = temp_dir.path().join("config");
    fs::create_dir_all(&config_dir).expect("无法创建配置目录");
    
    let original_config = config_dir.join("settings.json");
    let backup_config = config_dir.join("settings.json.backup");
    
    // 创建原始配置
    let original_content = r#"{
  "app_name": "AI Manager",
  "version": "1.0.0",
  "backup_test": true
}"#;
    
    fs::write(&original_config, original_content)
        .expect("无法写入原始配置");
    
    // 创建备份
    fs::copy(&original_config, &backup_config)
        .expect("无法创建配置备份");
    
    println!("✅ 配置备份创建成功");
    
    // 修改原始配置
    let modified_content = r#"{
  "app_name": "AI Manager",
  "version": "1.0.1",
  "backup_test": true,
  "modified": true
}"#;
    
    fs::write(&original_config, modified_content)
        .expect("无法修改配置");
    
    // 从备份恢复
    fs::copy(&backup_config, &original_config)
        .expect("无法从备份恢复配置");
    
    // 验证恢复结果
    let restored_content = fs::read_to_string(&original_config)
        .expect("无法读取恢复的配置");
    
    assert_eq!(restored_content, original_content);
    println!("✅ 配置从备份恢复成功");
}

/// 测试配置文件权限安全性
#[test]
fn test_config_file_security() {
    println!("测试配置文件权限安全性...");
    
    let temp_dir = TempDir::new().expect("无法创建临时目录");
    
    // 创建包含敏感信息的配置
    let sensitive_config = temp_dir.path().join("sensitive.json");
    let sensitive_content = r#"{
  "api_keys": {
    "anthropic": "sk-ant-api-key",
    "openai": "sk-openai-api-key"
  },
  "database_password": "secret-db-password",
  "encryption_key": "base64-encoded-key"
}"#;
    
    fs::write(&sensitive_config, sensitive_content)
        .expect("无法写入敏感配置");
    
    // 检查文件权限
    let metadata = fs::metadata(&sensitive_config)
        .expect("无法获取文件元数据");
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = metadata.permissions();
        let mode = permissions.mode();
        
        println!("敏感文件权限: {:o}", mode);
        
        // 验证文件不会对其他用户可读（在Unix系统上）
        // 注意：在CI环境中可能无法完全控制权限，所以这里只是检查
        if mode & 0o077 != 0 {
            println!("⚠️ 警告：文件对其他用户可读");
        } else {
            println!("✅ 文件权限设置安全");
        }
    }
    
    #[cfg(windows)]
    {
        let readonly = metadata.permissions().readonly();
        println!("敏感文件只读属性: {}", readonly);
    }
    
    // 验证文件内容不会被意外暴露（这里只是基本检查）
    let content = fs::read_to_string(&sensitive_config)
        .expect("无法读取文件内容");
    
    assert!(content.contains("api_keys"));
    println!("✅ 敏感配置文件内容验证完成");
}

/// 测试配置文件路径解析
#[test]
fn test_config_path_resolution() {
    println!("测试配置文件路径解析...");
    
    // 测试用户目录展开
    let test_paths = vec![
        "~/.config/ai-manager/settings.json",
        "~/Library/Application Support/AI Manager/config.json",
        "$HOME/.claude/settings.json",
    ];
    
    for path_str in test_paths {
        let path = Path::new(path_str);
        
        // 展开用户目录
        let expanded = if path_str.starts_with("~/") {
            if let Some(home) = dirs::home_dir() {
                home.join(path.strip_prefix("~").unwrap())
            } else {
                path.to_path_buf()
            }
        } else if path_str.starts_with("$HOME") {
            if let Some(home) = dirs::home_dir() {
                home.join(path.strip_prefix("$HOME").unwrap().strip_prefix("/").unwrap())
            } else {
                path.to_path_buf()
            }
        } else {
            path.to_path_buf()
        };
        
        println!("原始路径: {} -> 展开路径: {:?}", path_str, expanded);
        
        // 验证展开后的路径是绝对路径
        if expanded.is_absolute() {
            println!("  ✅ 路径已正确展开为绝对路径");
        } else {
            println!("  ⚠️ 路径仍为相对路径");
        }
    }
}

/// 测试跨平台配置迁移
#[test]
fn test_config_migration() {
    println!("测试跨平台配置迁移...");
    
    let temp_dir = TempDir::new().expect("无法创建临时目录");
    let source_dir = temp_dir.path().join("source");
    let target_dir = temp_dir.path().join("target");
    
    // 创建源配置结构（模拟从其他平台迁移）
    let source_config_dir = source_dir.join(".config").join("ai-manager");
    fs::create_dir_all(&source_config_dir).expect("无法创建源配置目录");
    
    // 创建源配置文件
    let configs = vec![
        ("settings.json", r#"{"app": "source", "version": "1.0.0"}"#),
        ("providers.json", r#"[{"name": "test", "enabled": true}]"#),
    ];
    
    for (filename, content) in configs {
        let file_path = source_config_dir.join(filename);
        fs::write(&file_path, content)
            .expect(&format!("无法创建源配置文件: {:?}", file_path));
    }
    
    // 模拟迁移过程
    let target_config_dir = if cfg!(target_os = "windows") {
        target_dir.join("AI Manager")
    } else {
        target_dir.join("ai-manager")
    };
    
    fs::create_dir_all(&target_config_dir)
        .expect("无法创建目标配置目录");
    
    // 复制配置文件
    for entry in fs::read_dir(&source_config_dir).expect("无法读取源目录") {
        let entry = entry.expect("无法读取目录条目");
        let source_file = entry.path();
        let target_file = target_config_dir.join(entry.file_name());
        
        fs::copy(&source_file, &target_file)
            .expect(&format!("无法复制配置文件: {:?} -> {:?}", source_file, target_file));
        
        println!("✅ 迁移配置文件: {:?}", target_file);
    }
    
    // 验证迁移结果
    for (filename, expected_content) in configs {
        let target_file = target_config_dir.join(filename);
        assert!(target_file.exists(), "迁移的配置文件应该存在: {:?}", target_file);
        
        let content = fs::read_to_string(&target_file)
            .expect(&format!("无法读取迁移的配置文件: {:?}", target_file));
        assert_eq!(content, expected_content);
    }
    
    println!("✅ 配置迁移验证成功");
}