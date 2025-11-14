// 测试所有依赖是否正确导入和工作的临时文件
use serde::{Deserialize, Serialize};
use fernet::Fernet;
use handlebars::Handlebars;
use thiserror::Error;
use tracing::{info, error};
use tracing_subscriber;

// 测试数据结构
#[derive(Debug, Serialize, Deserialize)]
struct TestItem {
    id: i64,
    name: String,
    token: String,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),
    #[error("加密错误: {0}")]
    Crypto(String),
    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("模板错误: {0}")]
    Template(String),
}

// 测试加密功能
fn test_crypto() -> Result<(), AppError> {
    let key = "dGVzdCBrZXkgZm9yIGZlcm5ldA=="; // Base64 编码的测试密钥
    let fernet = Fernet::new(key).ok_or_else(|| AppError::Crypto("无效的密钥格式".to_string()))?;

    let plaintext = "Hello, AI Manager!";
    let encrypted = fernet.encrypt(plaintext.as_bytes());
    let decrypted = fernet.decrypt(&encrypted).map_err(|e| AppError::Crypto(e.to_string()))?;

    assert_eq!(plaintext.as_bytes(), decrypted);
    info!("✅ 加密测试通过");

    Ok(())
}

// 测试模板功能
fn test_template() -> Result<(), AppError> {
    let mut handlebars = Handlebars::new();

    let template_str = "你好, {{name}}! 欢迎使用 {{app_name}}。";
    handlebars.register_template_string("greeting", template_str)
        .map_err(|e| AppError::Template(e.to_string()))?;

    let data = serde_json::json!({
        "name": "用户",
        "app_name": "AI Manager"
    });

    let result = handlebars.render("greeting", &data)
        .map_err(|e| AppError::Template(e.to_string()))?;

    assert!(result.contains("你好, 用户!"));
    info!("✅ 模板测试通过: {}", result);

    Ok(())
}

// 测试序列化功能
fn test_serialization() -> Result<(), AppError> {
    let item = TestItem {
        id: 1,
        name: "测试供应商".to_string(),
        token: "加密的令牌".to_string(),
    };

    let json = serde_json::to_string(&item)?;
    let deserialized: TestItem = serde_json::from_str(&json)?;

    assert_eq!(item.id, deserialized.id);
    assert_eq!(item.name, deserialized.name);
    info!("✅ 序列化测试通过");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_dependencies() {
        // 初始化日志
        tracing_subscriber::fmt().init();

        // 运行所有测试
        test_crypto().expect("加密测试失败");
        test_template().expect("模板测试失败");
        test_serialization().expect("序列化测试失败");

        println!("✅ 所有依赖测试通过!");
    }
}