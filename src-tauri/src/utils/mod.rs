//! 通用工具模块
//!
//! 这个模块包含了应用程序中常用的工具函数和帮助类，
//! 涵盖验证、字符串处理、日期时间、加密安全和配置管理等方面。
//!
//! # 模块结构
//!
//! - **validation**: 数据验证工具，包括字符串、邮箱、URL等验证
//! - **string_utils**: 字符串处理工具，包括截断、格式化、清理等
//! - **date_time**: 日期时间处理工具，包括格式化、计算、验证等
//! - **crypto_utils**: 加密相关工具，包括哈希、安全比较、令牌生成等
//! - **config_utils**: 配置文件管理工具，包括读取、写入、验证等
//!
//! # 设计原则
//!
//! - **简洁易用**: 提供简单直观的API接口
//! - **类型安全**: 使用强类型确保参数安全
//! - **错误处理**: 提供详细的错误信息
//! - **性能优化**: 避免不必要的内存分配和计算
//!
//! # 使用示例
//!
//! ```rust
//! use crate::utils::{validate_email, format_file_size};
//!
//! // 验证邮箱
//! let result = validate_email("user@example.com");
//!
//! // 格式化文件大小
//! let size_str = format_file_size(1024); // "1.00 KB"
//! ```

pub mod config_utils;
pub mod crypto_utils;
pub mod date_time;
pub mod string_utils;
pub mod validation;

// 重新导出常用函数
pub use config_utils::*;
pub use crypto_utils::*;
pub use date_time::*;
pub use string_utils::*;
pub use validation::*;
