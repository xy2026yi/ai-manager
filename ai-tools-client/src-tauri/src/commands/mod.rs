// 命令模块声明
pub mod supplier;
pub mod mcp_template;
pub mod config;
pub mod mode;

// 重新导出所有命令函数
pub use supplier::*;
pub use mcp_template::*;
pub use config::*;
pub use mode::*;