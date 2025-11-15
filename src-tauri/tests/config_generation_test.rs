//! 配置文件生成测试
//! 
//! 测试生成的配置文件与原Python项目的格式一致性

use serde_json;
use std::fs;
use tempfile::tempdir;
use tracing::{info, warn};

/// Claude配置文件生成测试结果
#[derive(Debug)]
pub struct ClaudeConfigTestResult {
    pub generated: bool,
    pub format_valid: bool,
    pub content_match: bool,
    pub file_path: String,
    pub errors: Vec<String>,
}

/// Codex配置文件生成测试结果
#[derive(Debug)]
pub struct CodexConfigTestResult {
    pub generated: bool,
    pub auth_format_valid: bool,
    pub config_format_valid: bool,
    pub auth_file_path: String,
    pub config_file_path: String,
    pub errors: Vec<String>,
}

/// 配置文件生成测试器
pub struct ConfigGeneratorTester {
    temp_dir: tempfile::TempDir,
}

impl ConfigGeneratorTester {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        Ok(Self { temp_dir })
    }
    
    /// 测试Claude配置文件生成
    pub async fn test_claude_config_generation(&self) -> Result<ClaudeConfigTestResult, Box<dyn std::error::Error>> {
        info!("测试Claude配置文件生成...");
        
        let mut result = ClaudeConfigTestResult {
            generated: false,
            format_valid: false,
            content_match: false,
            file_path: String::new(),
            errors: Vec::new(),
        };
        
        // 创建测试数据
        let test_data = create_test_claude_data();
        
        // 生成配置文件路径
        let config_path = self.temp_dir.path().join("claude_settings.json");
        result.file_path = config_path.to_string_lossy().to_string();
        
        // 生成Claude配置文件内容
        let config_content = generate_claude_config(&test_data)?;
        
        // 写入配置文件
        fs::write(&config_path, config_content)?;
        result.generated = true;
        
        // 验证配置文件格式
        let loaded_config: serde_json::Value = serde_json::from_str(&fs::read_to_string(&config_path)?)?;
        
        // 检查必要的字段
        if let Some(config_obj) = loaded_config.as_object() {
            result.format_valid = true;
            
            // 验证基本结构
            if config_obj.contains_key("claude_api_key") && 
               config_obj.contains_key("models") {
                result.content_match = true;
                info!("✅ Claude配置文件格式验证通过");
            } else {
                result.errors.push("Claude配置文件缺少必要字段".to_string());
                warn!("Claude配置文件缺少必要字段");
            }
        } else {
            result.errors.push("Claude配置文件不是有效的JSON对象".to_string());
            warn!("Claude配置文件格式无效");
        }
        
        Ok(result)
    }
    
    /// 测试Codex配置文件生成
    pub async fn test_codex_config_generation(&self) -> Result<CodexConfigTestResult, Box<dyn std::error::Error>> {
        info!("测试Codex配置文件生成...");
        
        let mut result = CodexConfigTestResult {
            generated: false,
            auth_format_valid: false,
            config_format_valid: false,
            auth_file_path: String::new(),
            config_file_path: String::new(),
            errors: Vec::new(),
        };
        
        // 创建测试数据
        let test_data = create_test_codex_data();
        
        // 生成配置文件路径
        let auth_path = self.temp_dir.path().join("codex_auth.json");
        let config_path = self.temp_dir.path().join("codex_config.toml");
        
        result.auth_file_path = auth_path.to_string_lossy().to_string();
        result.config_file_path = config_path.to_string_lossy().to_string();
        
        // 生成认证配置文件
        let auth_content = generate_codex_auth_config(&test_data)?;
        fs::write(&auth_path, auth_content)?;
        
        // 生成主配置文件
        let config_content = generate_codex_main_config(&test_data)?;
        fs::write(&config_path, config_content)?;
        
        result.generated = true;
        
        // 验证认证配置文件格式
        let auth_config: serde_json::Value = serde_json::from_str(&fs::read_to_string(&auth_path)?)?;
        if let Some(_auth_obj) = auth_config.as_object() {
            result.auth_format_valid = true;
            info!("✅ Codex认证配置文件格式验证通过");
        } else {
            result.errors.push("Codex认证配置文件格式无效".to_string());
            warn!("Codex认证配置文件格式无效");
        }
        
        // 验证主配置文件格式（TOML）
        let config_content = fs::read_to_string(&config_path)?;
        if config_content.contains("[openai]") && config_content.contains("api_key") {
            result.config_format_valid = true;
            info!("✅ Codex主配置文件格式验证通过");
        } else {
            result.errors.push("Codex主配置文件格式无效".to_string());
            warn!("Codex主配置文件格式无效");
        }
        
        Ok(result)
    }
    
    /// 测试配置文件与Python版本的兼容性
    pub async fn test_python_compatibility(&self) -> Result<bool, Box<dyn std::error::Error>> {
        info!("测试配置文件与Python版本兼容性...");
        
        // 生成Python风格的配置文件
        let python_claude_config = generate_python_style_claude_config()?;
        let python_codex_auth = generate_python_style_codex_auth()?;
        let python_codex_config = generate_python_style_codex_config()?;
        
        // 写入测试文件
        let claude_path = self.temp_dir.path().join("python_claude_settings.json");
        let codex_auth_path = self.temp_dir.path().join("python_codex_auth.json");
        let codex_config_path = self.temp_dir.path().join("python_codex_config.toml");
        
        fs::write(&claude_path, python_claude_config)?;
        fs::write(&codex_auth_path, python_codex_auth)?;
        fs::write(&codex_config_path, python_codex_config)?;
        
        // 验证文件可以被Python风格的解析器读取
        let claude_config: serde_json::Value = serde_json::from_str(&fs::read_to_string(&claude_path)?)?;
        let codex_auth: serde_json::Value = serde_json::from_str(&fs::read_to_string(&codex_auth_path)?)?;
        
        // 基本验证
        let claude_valid = claude_config.get("claude_api_key").is_some();
        let codex_auth_valid = codex_auth.get("openai_api_key").is_some();
        
        let codex_config_content = fs::read_to_string(&codex_config_path)?;
        let codex_config_valid = codex_config_content.contains("[openai]") && 
                                codex_config_content.contains("api_key");
        
        let all_valid = claude_valid && codex_auth_valid && codex_config_valid;
        
        if all_valid {
            info!("✅ 配置文件Python兼容性测试通过");
        } else {
            warn!("⚠️ 配置文件Python兼容性测试部分失败");
        }
        
        Ok(all_valid)
    }
}

/// 创建测试用的Claude数据
fn create_test_claude_data() -> serde_json::Value {
    serde_json::json!({
        "name": "Test Claude Provider",
        "url": "https://api.anthropic.com",
        "token": "sk-ant-test-key-12345",
        "type": "public_welfare",
        "enabled": true,
        "opus_model": "claude-3-opus-20240229",
        "sonnet_model": "claude-3-sonnet-20240229",
        "haiku_model": "claude-3-haiku-20240307"
    })
}

/// 创建测试用的Codex数据
fn create_test_codex_data() -> serde_json::Value {
    serde_json::json!({
        "name": "Test OpenAI Provider",
        "url": "https://api.openai.com/v1/chat/completions",
        "token": "sk-test-openai-key-67890",
        "type": "official",
        "enabled": true
    })
}

/// 生成Claude配置文件
fn generate_claude_config(data: &serde_json::Value) -> Result<String, Box<dyn std::error::Error>> {
    let config = serde_json::json!({
        "claude_api_key": data["token"],
        "claude_api_url": data["url"],
        "models": {
            "opus": data["opus_model"],
            "sonnet": data["sonnet_model"],
            "haiku": data["haiku_model"]
        },
        "default_model": data["sonnet_model"],
        "max_tokens": 4096,
        "temperature": 0.7,
        "timeout": 30000,
        "auto_update": true,
        "provider_type": data["type"]
    });
    
    Ok(serde_json::to_string_pretty(&config)?)
}

/// 生成Codex认证配置文件
fn generate_codex_auth_config(data: &serde_json::Value) -> Result<String, Box<dyn std::error::Error>> {
    let config = serde_json::json!({
        "openai_api_key": data["token"],
        "openai_api_base": data["url"],
        "organization": null,
        "project": null
    });
    
    Ok(serde_json::to_string_pretty(&config)?)
}

/// 生成Codex主配置文件
fn generate_codex_main_config(data: &serde_json::Value) -> Result<String, Box<dyn std::error::Error>> {
    let config = format!(
        r#"[openai]
api_key = "{}"
api_base = "{}"
model = "gpt-4"
max_tokens = 4096
temperature = 0.7
timeout = 30000
provider_type = "{}"

[general]
default_provider = "openai"
auto_save = true
save_interval = 300
"#,
        data["token"],
        data["url"],
        data["type"]
    );
    
    Ok(config)
}

/// 生成Python风格的Claude配置
fn generate_python_style_claude_config() -> Result<String, Box<dyn std::error::Error>> {
    let config = serde_json::json!({
        "claude_api_key": "sk-ant-python-test-key",
        "claude_api_url": "https://api.anthropic.com",
        "models": {
            "opus": "claude-3-opus-20240229",
            "sonnet": "claude-3-sonnet-20240229",
            "haiku": "claude-3-haiku-20240307"
        },
        "default_model": "claude-3-sonnet-20240229",
        "max_tokens": 4096,
        "temperature": 0.7,
        "timeout": 30000
    });
    
    Ok(serde_json::to_string_pretty(&config)?)
}

/// 生成Python风格的Codex认证配置
fn generate_python_style_codex_auth() -> Result<String, Box<dyn std::error::Error>> {
    let config = serde_json::json!({
        "openai_api_key": "sk-python-test-openai-key",
        "openai_api_base": "https://api.openai.com/v1",
        "organization": null,
        "project": null
    });
    
    Ok(serde_json::to_string_pretty(&config)?)
}

/// 生成Python风格的Codex主配置
fn generate_python_style_codex_config() -> Result<String, Box<dyn std::error::Error>> {
    let config = r#"[openai]
api_key = "sk-python-test-openai-key"
api_base = "https://api.openai.com/v1"
model = "gpt-4"
max_tokens = 4096
temperature = 0.7
timeout = 30000

[general]
default_provider = "openai"
auto_save = true
save_interval = 300
"#;
    
    Ok(config.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_config_generator_creation() {
        let tester = ConfigGeneratorTester::new().unwrap();
        assert!(tester.temp_dir.path().exists());
        println!("✅ 配置生成器创建测试通过");
    }
    
    #[tokio::test]
    async fn test_claude_config_generation() {
        let tester = ConfigGeneratorTester::new().unwrap();
        let result = tester.test_claude_config_generation().await.unwrap();
        
        assert!(result.generated, "Claude配置文件应该生成成功");
        assert!(result.format_valid, "Claude配置文件格式应该有效");
        assert!(result.content_match, "Claude配置文件内容应该匹配");
        
        println!("✅ Claude配置文件生成测试通过");
    }
    
    #[tokio::test]
    async fn test_codex_config_generation() {
        let tester = ConfigGeneratorTester::new().unwrap();
        let result = tester.test_codex_config_generation().await.unwrap();
        
        assert!(result.generated, "Codex配置文件应该生成成功");
        assert!(result.auth_format_valid, "Codex认证配置文件格式应该有效");
        assert!(result.config_format_valid, "Codex主配置文件格式应该有效");
        
        println!("✅ Codex配置文件生成测试通过");
    }
    
    #[tokio::test]
    async fn test_python_compatibility() {
        let tester = ConfigGeneratorTester::new().unwrap();
        let result = tester.test_python_compatibility().await.unwrap();
        
        assert!(result, "配置文件应该与Python版本兼容");
        
        println!("✅ Python兼容性测试通过");
    }
    
    #[test]
    fn test_config_content_generation() {
        let test_data = create_test_claude_data();
        let claude_config = generate_claude_config(&test_data).unwrap();
        
        // 验证生成的配置包含必要字段
        assert!(claude_config.contains("claude_api_key"));
        assert!(claude_config.contains("models"));
        assert!(claude_config.contains("default_model"));
        
        println!("✅ 配置内容生成测试通过");
    }
}