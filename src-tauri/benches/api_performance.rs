// API性能基准测试
// 测试所有API端点的响应时间和并发处理能力

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use migration_ai_manager_lib::{api, services::*, database::DatabaseManager, crypto::CryptoService};
use tokio::runtime::Runtime;
use std::sync::Arc;
use serde_json::json;

// 创建测试服务
async fn create_test_services() -> (Arc<DatabaseManager>, Arc<CryptoService>, ClaudeProviderService) {
    let db_config = migration_ai_manager_lib::database::DatabaseConfig {
        url: "sqlite::memory:".to_string(),
        max_connections: 10,
        min_connections: 1,
        connect_timeout: std::time::Duration::from_secs(5),
        idle_timeout: std::time::Duration::from_secs(60),
        max_lifetime: std::time::Duration::from_secs(300),
    };

    let db_manager = Arc::new(DatabaseManager::new(db_config).await.expect("Failed to create database"));
    let crypto_service = Arc::new(CryptoService::new("test_key_for_benchmarks").expect("Failed to create crypto service"));
    let claude_service = ClaudeProviderService::new(db_manager.clone(), crypto_service.clone());
    
    (db_manager, crypto_service, claude_service)
}

// 基准测试：Claude供应商创建性能
fn bench_claude_provider_creation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (_db, _crypto, service) = rt.block_on(create_test_services());
    
    c.bench_function("claude_provider_creation", |b| {
        b.to_async(&rt).iter(|| async {
            let request = migration_ai_manager_lib::models::CreateClaudeProviderRequest {
                name: "Benchmark Provider".to_string(),
                url: "https://api.openai.com".to_string(),
                token: "sk-benchmark-token".to_string(),
                max_tokens: Some(4096),
                temperature: Some(0.7),
                model: Some("gpt-4".to_string()),
                enabled: Some(1),
                description: Some("Benchmark test provider".to_string()),
                timeout: Some(30),
                retry_count: Some(3),
            };
            
            let result = service.create_provider(black_box(&request)).await;
            black_box(result)
        });
    });
}

// 基准测试：Claude供应商查询性能
fn bench_claude_provider_query(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (_db, _crypto, service) = rt.block_on(create_test_services());
    
    // 先创建一个供应商用于查询
    rt.block_on(async {
        let request = migration_ai_manager_lib::models::CreateClaudeProviderRequest {
            name: "Query Test Provider".to_string(),
            url: "https://api.openai.com".to_string(),
            token: "sk-query-token".to_string(),
            max_tokens: None,
            temperature: None,
            model: None,
            enabled: Some(1),
            description: None,
            timeout: None,
            retry_count: None,
        };
        service.create_provider(&request).await.unwrap();
    });
    
    c.bench_function("claude_provider_query", |b| {
        b.to_async(&rt).iter(|| async {
            let params = migration_ai_manager_lib::models::PaginationParams {
                page: 1,
                limit: 10,
            };
            
            let result = service.list_providers(black_box(params)).await;
            black_box(result)
        });
    });
}

// 基准测试：数据库连接池性能
fn bench_database_connection_pool(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (db_manager, _crypto, _service) = rt.block_on(create_test_services());
    
    c.bench_function("database_connection_pool", |b| {
        b.to_async(&rt).iter(|| async {
            // 模拟频繁的数据库连接获取和释放
            let result = db_manager.health_check().await;
            black_box(result)
        });
    });
}

// 基准测试：加密解密性能
fn bench_crypto_operations(c: &mut Criterion) {
    let crypto_service = CryptoService::new("benchmark_key").expect("Failed to create crypto service");
    let test_data = "这是一段需要进行性能测试的敏感数据内容，包含中文字符和特殊符号！@#$%^&*()";
    
    c.bench_function("encryption", |b| {
        b.iter(|| {
            let encrypted = crypto_service.encrypt(black_box(test_data));
            black_box(encrypted)
        });
    });
    
    c.bench_function("decryption", |b| {
        let encrypted = crypto_service.encrypt(test_data).unwrap();
        b.iter(|| {
            let decrypted = crypto_service.decrypt(black_box(&encrypted));
            black_box(decrypted)
        });
    });
}

// 基准测试：JSON序列化/反序列化性能
fn bench_json_serialization(c: &mut Criterion) {
    let claude_provider = migration_ai_manager_lib::models::ClaudeProvider {
        id: 1,
        name: "JSON Test Provider".to_string(),
        url: "https://api.openai.com".to_string(),
        token: "sk-json-test-token".to_string(),
        max_tokens: 4096,
        temperature: 0.7,
        model: "gpt-4".to_string(),
        enabled: 1,
        description: Some("JSON序列化测试供应商".to_string()),
        timeout: 30,
        retry_count: 3,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    c.bench_function("json_serialization", |b| {
        b.iter(|| {
            let serialized = serde_json::to_string(black_box(&claude_provider));
            black_box(serialized)
        });
    });
    
    let json_string = serde_json::to_string(&claude_provider).unwrap();
    c.bench_function("json_deserialization", |b| {
        b.iter(|| {
            let deserialized: Result<migration_ai_manager_lib::models::ClaudeProvider, _> = 
                serde_json::from_str(black_box(&json_string));
            black_box(deserialized)
        });
    });
}

// 基准测试：并发API请求处理
fn bench_concurrent_requests(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (_db, _crypto, service) = rt.block_on(create_test_services());
    
    c.bench_function("concurrent_claude_creation", |b| {
        b.to_async(&rt).iter(|| async {
            let mut handles = vec![];
            
            // 创建10个并发任务
            for i in 0..10 {
                let service_clone = service.clone();
                let handle = tokio::spawn(async move {
                    let request = migration_ai_manager_lib::models::CreateClaudeProviderRequest {
                        name: format!("Concurrent Provider {}", i),
                        url: "https://api.openai.com".to_string(),
                        token: format!("sk-concurrent-token-{}", i),
                        max_tokens: Some(4096),
                        temperature: Some(0.7),
                        model: Some("gpt-4".to_string()),
                        enabled: Some(1),
                        description: Some(format!("Concurrent test provider {}", i)),
                        timeout: Some(30),
                        retry_count: Some(3),
                    };
                    
                    service_clone.create_provider(&request).await
                });
                handles.push(handle);
            }
            
            // 等待所有任务完成
            let results = futures::future::join_all(handles).await;
            black_box(results)
        });
    });
}

// 基准测试：内存分配性能
fn bench_memory_allocation(c: &mut Criterion) {
    c.bench_function("vector_creation", |b| {
        b.iter(|| {
            let providers: Vec<migration_ai_manager_lib::models::ClaudeProvider> = (0..1000)
                .map(|i| migration_ai_manager_lib::models::ClaudeProvider {
                    id: i,
                    name: format!("Provider {}", i),
                    url: "https://api.openai.com".to_string(),
                    token: format!("sk-token-{}", i),
                    max_tokens: 4096,
                    temperature: 0.7,
                    model: "gpt-4".to_string(),
                    enabled: if i % 2 == 0 { 1 } else { 0 },
                    description: Some(format!("Test provider number {}", i)),
                    timeout: 30,
                    retry_count: 3,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                })
                .collect();
            black_box(providers)
        });
    });
}

criterion_group!(
    benches,
    bench_claude_provider_creation,
    bench_claude_provider_query,
    bench_database_connection_pool,
    bench_crypto_operations,
    bench_json_serialization,
    bench_concurrent_requests,
    bench_memory_allocation
);
criterion_main!(benches);