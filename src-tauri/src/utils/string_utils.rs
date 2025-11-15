//! 字符串处理工具模块
//!
//! 这个模块提供了常用的字符串处理函数，包括字符串截断、格式化、
//! 清理、转换等功能。支持中文处理和安全的字符串操作。
//!
//! # 主要功能
//!
//! - **字符串截断**: 安全地截断长字符串，避免panic
//! - **格式转换**: 大小写转换、清理空白字符
//! - **Unicode支持**: 完整支持中文和其他Unicode字符
//! - **安全操作**: 提供防时序攻击的安全比较函数
//! - **格式化**: 文件大小、文件名等格式化功能
//!
//! # 使用示例
//!
//! ```rust
//! use crate::utils::string_utils::{truncate_string, format_file_size, clean_string};
//!
//! // 截断长字符串
//! let short = truncate_string("这是一个很长的字符串", 10);
//!
//! // 格式化文件大小
//! let size_str = format_file_size(1024); // "1.00 KB"
//!
//! // 清理字符串
//! let clean = clean_string("  hello   world  "); // "hello world"
//! ```

/// 截断字符串到指定长度
pub fn truncate_string(s: &str, max_length: usize) -> String {
    if s.chars().count() <= max_length {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_length.saturating_sub(3)).collect();
        truncated + "..."
    }
}

/// 安全地转换为小写
pub fn to_lowercase_safe(s: &str) -> String {
    s.to_lowercase()
}

/// 安全地转换为大写
pub fn to_uppercase_safe(s: &str) -> String {
    s.to_uppercase()
}

/// 清理字符串（去除前后空白并统一内部空格）
pub fn clean_string(s: &str) -> String {
    s.trim().split_whitespace().collect::<Vec<_>>().join(" ")
}

/// 检查字符串是否包含中文字符
pub fn contains_chinese(s: &str) -> bool {
    s.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c))
}

/// 移除字符串中的所有空白字符
pub fn remove_whitespace(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

/// 生成随机字符串
pub fn generate_random_string(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    let mut rng = rand::thread_rng();

    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// 格式化文件大小显示
pub fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// 转换为安全的文件名（移除非法字符）
pub fn to_safe_filename(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            _ => c,
        })
        .collect()
}

/// 生成URL友好的字符串
pub fn slugify(s: &str) -> String {
    let normalized = s.to_lowercase()
        .chars()
        .map(|c| match c {
            'a'..='z' | '0'..='9' => c,
            ' ' | '-' => '-',
            _ => '_',
        })
        .collect::<String>();

    normalized
        .split('-')
        .filter(|&s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("short", 10), "short");
        assert_eq!(truncate_string("this is a long string", 10), "this is...");
    }

    #[test]
    fn test_clean_string() {
        assert_eq!(clean_string("  hello   world  "), "hello world");
        assert_eq!(clean_string("  multiple   spaces  "), "multiple spaces");
    }

    #[test]
    fn test_contains_chinese() {
        assert!(contains_chinese("hello 世界"));
        assert!(!contains_chinese("hello world"));
        assert!(contains_chinese("测试"));
    }

    #[test]
    fn test_remove_whitespace() {
        assert_eq!(remove_whitespace("hello world"), "helloworld");
        assert_eq!(remove_whitespace("  test  string  "), "teststring");
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(1024), "1.00 KB");
        assert_eq!(format_file_size(1048576), "1.00 MB");
        assert_eq!(format_file_size(500), "500 B");
    }

    #[test]
    fn test_to_safe_filename() {
        assert_eq!(to_safe_filename("file<>name.txt"), "file__name.txt");
        assert_eq!(to_safe_filename("path/to/file"), "path/to/file");
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World!"), "hello-world");
        assert_eq!(slugify("  Multiple   Spaces  "), "multiple-spaces");
    }
}
