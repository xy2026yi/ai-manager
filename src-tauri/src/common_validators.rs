//! 通用验证器
//!
//! 提供统一的输入验证功能，减少重复代码

/// 验证函数结果类型
pub type ValidationResult<T> = Result<T, ValidationError>;

/// 验证错误类型
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub message: String,
    pub field: Option<String>,
}

impl ValidationError {
    /// 创建新的验证错误
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            field: None,
        }
    }

    /// 创建带字段的验证错误
    pub fn with_field(message: impl Into<String>, field: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            field: Some(field.into()),
        }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.field {
            Some(field) => write!(f, "[字段: {}] {}", field, self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for ValidationError {}

/// 通用验证器
pub struct Validator;

impl Validator {
    /// 验证ID是否有效（正整数）
    pub fn validate_id(id: i64, field_name: &str) -> ValidationResult<i64> {
        if id <= 0 {
            return Err(ValidationError::with_field(
                "无效的ID，必须为正整数".to_string(),
                field_name,
            ));
        }
        Ok(id)
    }

    /// 验证字符串不为空
    pub fn validate_non_empty<'a>(value: &'a str, field_name: &str) -> ValidationResult<&'a str> {
        if value.trim().is_empty() {
            return Err(ValidationError::with_field(
                format!("{}不能为空", field_name),
                field_name,
            ));
        }
        Ok(value)
    }

    /// 验证字符串长度
    pub fn validate_string_length<'a>(
        value: &'a str,
        field_name: &str,
        min: usize,
        max: usize,
    ) -> ValidationResult<&'a str> {
        let len = value.len();
        if len < min {
            return Err(ValidationError::with_field(
                format!("{}长度不能少于{}个字符", field_name, min),
                field_name,
            ));
        }
        if len > max {
            return Err(ValidationError::with_field(
                format!("{}长度不能超过{}个字符", field_name, max),
                field_name,
            ));
        }
        Ok(value)
    }

    /// 验证搜索词
    pub fn validate_search_term<'a>(value: &'a str) -> ValidationResult<&'a str> {
        Self::validate_non_empty(value, "搜索词")
            .and_then(|term| Self::validate_string_length(term, "搜索词", 1, 100))
    }

    /// 验证供应商名称
    pub fn validate_provider_name<'a>(value: &'a str) -> ValidationResult<&'a str> {
        Self::validate_non_empty(value, "供应商名称")
            .and_then(|name| Self::validate_string_length(name, "供应商名称", 1, 100))
    }

    /// 验证Agent指导文件名称
    pub fn validate_agent_guide_name<'a>(value: &'a str) -> ValidationResult<&'a str> {
        Self::validate_non_empty(value, "指导文件名称")
            .and_then(|name| Self::validate_string_length(name, "指导文件名称", 1, 200))
    }

    /// 验证Agent指导文件内容
    pub fn validate_agent_guide_content<'a>(value: &'a str) -> ValidationResult<&'a str> {
        Self::validate_non_empty(value, "指导文件内容")
            .and_then(|content| Self::validate_string_length(content, "指导文件内容", 1, 100000))
    }

    /// 验证配置键名
    pub fn validate_config_key<'a>(value: &'a str) -> ValidationResult<&'a str> {
        Self::validate_non_empty(value, "配置键名")
            .and_then(|key| Self::validate_string_length(key, "配置键名", 1, 100))
    }

    /// 验证配置值
    pub fn validate_config_value<'a>(value: &'a str) -> ValidationResult<&'a str> {
        Self::validate_string_length(value, "配置值", 0, 10000)
    }

    /// 验证配置类别
    pub fn validate_config_category<'a>(value: &'a str) -> ValidationResult<&'a str> {
        Self::validate_non_empty(value, "配置类别")
            .and_then(|category| Self::validate_string_length(category, "配置类别", 1, 50))
    }

    /// 验证服务器名称
    pub fn validate_server_name<'a>(value: &'a str) -> ValidationResult<&'a str> {
        Self::validate_non_empty(value, "服务器名称")
            .and_then(|name| Self::validate_string_length(name, "服务器名称", 1, 100))
    }

    /// 验证URL格式
    pub fn validate_url<'a>(value: &'a str) -> ValidationResult<&'a str> {
        if value.trim().is_empty() {
            return Ok(value); // 允许空URL
        }

        // 简单的URL验证
        if !value.starts_with("http://") && !value.starts_with("https://") {
            return Err(ValidationError::with_field(
                "URL必须以http://或https://开头".to_string(),
                "url",
            ));
        }

        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_id() {
        assert!(Validator::validate_id(1, "id").is_ok());
        assert!(Validator::validate_id(0, "id").is_err());
        assert!(Validator::validate_id(-1, "id").is_err());
    }

    #[test]
    fn test_validate_non_empty() {
        assert!(Validator::validate_non_empty("test", "field").is_ok());
        assert!(Validator::validate_non_empty("", "field").is_err());
        assert!(Validator::validate_non_empty("   ", "field").is_err());
    }

    #[test]
    fn test_validate_string_length() {
        assert!(Validator::validate_string_length("test", "field", 1, 10).is_ok());
        assert!(Validator::validate_string_length("", "field", 1, 10).is_err());
        assert!(Validator::validate_string_length("very long string", "field", 1, 5).is_err());
    }

    #[test]
    fn test_validate_url() {
        assert!(Validator::validate_url("https://example.com").is_ok());
        assert!(Validator::validate_url("http://example.com").is_ok());
        assert!(Validator::validate_url("ftp://example.com").is_err());
        assert!(Validator::validate_url("").is_ok()); // 允许空
    }
}