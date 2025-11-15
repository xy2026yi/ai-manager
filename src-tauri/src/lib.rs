// AI Manager Library - 核心功能模块
pub mod api;
mod common_validators;
pub mod crypto;
pub mod database;
pub mod logging;
mod logging_manager;
pub mod migration;
pub mod migration_tool;
pub mod models;
pub mod performance;
pub mod python_compatibility_test;
pub mod repositories;
pub mod services;
pub mod simple_migration;
pub mod utils;

// 通用验证器
pub use common_validators::{ValidationError, ValidationResult, Validator};

// 重新导出主要功能
pub use api::{ApiError, ApiResponse, ApiResult, ApiServer, PagedResponse, RequestContext};
pub use crypto::{CryptoError, CryptoService};
pub use database::{DatabaseConfig, DatabaseError, DatabaseManager, PoolStatus, QueryBuilder};
pub use logging_manager::LogConfig;
pub use logging_manager::LoggingManager;
pub use models::*;
pub use performance::{
    MetricType, PerformanceMetric, PerformanceMonitor, PerformanceSummary, PerformanceTimer,
};
pub use repositories::{BaseRepository, RepositoryResult};
pub use simple_migration::{SimpleExportData, SimpleMigrationError, SimpleMigrationTool};
