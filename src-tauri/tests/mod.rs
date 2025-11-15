// 测试模块
//
// 组织所有集成测试和兼容性验证测试

// API 集成测试
pub mod agent_guide_api_test;
pub mod api_integration;
pub mod claude_provider_api_test;
pub mod codex_provider_api_test;
pub mod common_config_api_test;
pub mod mcp_server_api_test;

// 中间件和基础功能测试
pub mod middleware_test;
pub mod simple_encryption_test;

// 配置生成测试
pub mod config_generation_test;

// 数据兼容性验证测试模块
pub mod crypto_compatibility;
pub mod data_compatibility_runner;
pub mod data_integrity_validator;
pub mod encryption_compatibility_test;
pub mod migration_compatibility_test;
pub mod migration_test;
pub mod migration_test_config;
