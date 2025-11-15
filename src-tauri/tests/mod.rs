// 测试模块
//
// 组织所有集成测试和兼容性验证测试

pub mod api_test;
pub mod middleware_test;
pub mod crypto_test;
pub mod providers_test;
pub mod config_management_test;
pub mod database_test;

// 数据兼容性验证测试模块
pub mod migration_test;
pub mod migration_test_config;
pub mod encryption_compatibility_test;
pub mod data_compatibility_runner;