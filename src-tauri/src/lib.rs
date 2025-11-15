// AI Manager Library - 核心功能模块
mod common_validators;
mod logging_manager;
pub mod api;
pub mod crypto;
pub mod database;
pub mod logging;
pub mod models;
pub mod performance;
pub mod python_compatibility_test;
pub mod repositories;
pub mod services;
pub mod simple_migration;
pub mod utils;
pub mod migration_tool;
pub mod migration;

// 通用验证器
pub use common_validators::{Validator, ValidationError, ValidationResult};

// 重新导出主要功能
pub use api::{ApiError, ApiResponse, ApiResult, ApiServer, PagedResponse, RequestContext};
pub use crypto::{CryptoError, CryptoService};
pub use database::{DatabaseConfig, DatabaseError, DatabaseManager, PoolStatus, QueryBuilder};
pub use models::*;
pub use performance::{PerformanceMonitor, PerformanceMetric, PerformanceTimer, MetricType, PerformanceSummary};
pub use repositories::{BaseRepository, RepositoryResult};
pub use simple_migration::{SimpleExportData, SimpleMigrationError, SimpleMigrationTool};
pub use logging_manager::LoggingManager;
pub use logging_manager::LogConfig;
