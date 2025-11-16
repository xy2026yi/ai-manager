// 简单的配置文件生成测试
use serde_json;

fn main() {
    println!("🔧 测试配置文件生成功能...");

    // 测试Claude配置生成
    let claude_config = generate_claude_config();
    println!("✅ Claude配置文件生成成功:");
    println!("{}", claude_config);

    // 测试Codex配置生成
    let codex_auth = generate_codex_auth_config();
    println!("✅ Codex认证配置文件生成成功:");
    println!("{}", codex_auth);

    let codex_config = generate_codex_main_config();
    println!("✅ Codex主配置文件生成成功:");
    println!("{}", codex_config);

    // 验证JSON格式
    let _claude_parsed: serde_json::Value = serde_json::from_str(&claude_config).expect("Claude配置JSON格式有效");
    let _codex_auth_parsed: serde_json::Value = serde_json::from_str(&codex_auth).expect("Codex认证配置JSON格式有效");

    println!("🎉 所有配置文件生成测试通过！");
}

fn generate_claude_config() -> String {
    let config = serde_json::json!({
        "claude_api_key": "sk-ant-test-key-12345",
        "claude_api_url": "https://api.anthropic.com",
        "models": {
            "opus": "claude-3-opus-20240229",
            "sonnet": "claude-3-sonnet-20240229",
            "haiku": "claude-3-haiku-20240307"
        },
        "default_model": "claude-3-sonnet-20240229",
        "max_tokens": 4096,
        "temperature": 0.7,
        "timeout": 30000,
        "auto_update": true,
        "provider_type": "public_welfare"
    });

    serde_json::to_string_pretty(&config).unwrap()
}

fn generate_codex_auth_config() -> String {
    let config = serde_json::json!({
        "openai_api_key": "sk-test-openai-key-67890",
        "openai_api_base": "https://api.openai.com/v1",
        "organization": null,
        "project": null
    });

    serde_json::to_string_pretty(&config).unwrap()
}

fn generate_codex_main_config() -> String {
    r#"[openai]
api_key = "sk-test-openai-key-67890"
api_base = "https://api.openai.com/v1"
model = "gpt-4"
max_tokens = 4096
temperature = 0.7
timeout = 30000
provider_type = "official"

[general]
default_provider = "openai"
auto_save = true
save_interval = 300
"#.to_string()
}