//! 数据验证工具模块
//!
//! 这个模块提供了常用的数据验证函数，用于验证用户输入的合法性。
//! 包括字符串长度、邮箱格式、URL格式、API Token格式等验证功能。
//!
//! # 主要功能
//!
//! - **字符串验证**: 空值检查、长度验证、空白字符处理
//! - **格式验证**: 邮箱、URL、API Token等标准格式验证
//! - **数值验证**: 端口号范围验证、模型名称格式验证
//! - **自定义验证**: 支持业务逻辑相关的自定义验证规则
//!
//! # 使用示例
//!
//! ```rust
//! use crate::utils::validation::{validate_email, validate_api_token, validate_port};
//!
//! // 验证邮箱
//! let email_result = validate_email("user@example.com");
//!
//! // 验证API Token
//! let token_result = validate_api_token("sk-1234567890");
//!
//! // 验证端口号
//! let port_result = validate_port(8080);
//! ```

/// 验证字符串是否为空或只包含空白字符
pub fn is_empty_or_whitespace(s: &str) -> bool {
    s.trim().is_empty()
}

/// 验证字符串长度是否在指定范围内
pub fn validate_length(s: &str, min: usize, max: usize) -> Result<(), String> {
    let len = s.chars().count();
    if len < min {
        return Err(format!("长度不能少于 {} 个字符", min));
    }
    if len > max {
        return Err(format!("长度不能超过 {} 个字符", max));
    }
    Ok(())
}

/// 验证邮箱地址格式
pub fn validate_email(email: &str) -> Result<(), String> {
    if email.trim().is_empty() {
        return Err("邮箱地址不能为空".to_string());
    }

    // 简单的邮箱验证
    if !email.contains('@') || !email.contains('.') {
        return Err("邮箱地址格式无效".to_string());
    }

    Ok(())
}

/// 验证URL格式
pub fn validate_url(url: &str) -> Result<(), String> {
    if url.trim().is_empty() {
        return Err("URL不能为空".to_string());
    }

    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("URL必须以 http:// 或 https:// 开头".to_string());
    }

    Ok(())
}

/// 验证API Token格式
pub fn validate_api_token(token: &str) -> Result<(), String> {
    if token.trim().is_empty() {
        return Err("API Token不能为空".to_string());
    }

    if token.len() < 10 {
        return Err("API Token长度不能少于10个字符".to_string());
    }

    // 检查是否包含基本的token前缀
    if !token.starts_with("sk-") && !token.starts_with("pk-") && !token.starts_with("ghp_") {
        return Err("API Token格式可能无效，请检查token前缀".to_string());
    }

    Ok(())
}

/// 验证端口号
pub fn validate_port(port: u16) -> Result<(), String> {
    if port == 0 {
        return Err("端口号不能为0".to_string());
    }

    if port < 1024 {
        return Err("建议使用1024以上的端口号以避免权限问题".to_string());
    }

    if port > 65535 {
        return Err("端口号不能超过65535".to_string());
    }

    Ok(())
}

/// 验证模型名称
pub fn validate_model_name(model: &str) -> Result<(), String> {
    if model.trim().is_empty() {
        return Err("模型名称不能为空".to_string());
    }

    // 常见的模型名称模式
    let valid_patterns = ["claude-3", "gpt-", "text-", "davinci-", "babbage-"];

    let model_lower = model.to_lowercase();
    if !valid_patterns.iter().any(|pattern| model_lower.contains(pattern)) {
        return Err("模型名称格式可能无效".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_empty_or_whitespace() {
        assert!(is_empty_or_whitespace(""));
        assert!(is_empty_or_whitespace("   "));
        assert!(!is_empty_or_whitespace("test"));
        assert!(!is_empty_or_whitespace("  test  "));
    }

    #[test]
    fn test_validate_length() {
        assert!(validate_length("test", 3, 10).is_ok());
        assert!(validate_length("test", 5, 10).is_err()); // 太短
        assert!(validate_length("this is a very long string", 1, 10).is_err()); // 太长
    }

    #[test]
    fn test_validate_email() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("invalid-email").is_err());
        assert!(validate_email("").is_err());
    }

    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://api.example.com").is_ok());
        assert!(validate_url("http://localhost:8080").is_ok());
        assert!(validate_url("ftp://example.com").is_err());
        assert!(validate_url("").is_err());
    }

    #[test]
    fn test_validate_api_token() {
        assert!(validate_api_token("sk-1234567890abcdef").is_ok());
        assert!(validate_api_token("pk-1234567890").is_ok());
        assert!(validate_api_token("invalid-token").is_err());
        assert!(validate_api_token("short").is_err());
    }

    #[test]
    fn test_validate_port() {
        assert!(validate_port(8080).is_ok());
        assert!(validate_port(3000).is_ok());
        assert!(validate_port(0).is_err());
        assert!(validate_port(1023).is_err()); // 警告
        assert!(validate_port(65535).is_err());
    }

    #[test]
    fn test_validate_model_name() {
        assert!(validate_model_name("claude-3-sonnet-20240229").is_ok());
        assert!(validate_model_name("gpt-4").is_ok());
        assert!(validate_model_name("").is_err());
        assert!(validate_model_name("invalid-model").is_err());
    }
}
