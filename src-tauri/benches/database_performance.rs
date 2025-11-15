// 数据库性能基准测试
// 测试数据库查询、事务处理和连接池性能

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use migration_ai_manager_lib::{database::{DatabaseManager, DatabaseConfig}, repositories::*, crypto::CryptoService};
use tokio::runtime::Runtime;
use std::sync::Arc;
use migration_ai_manager_lib::models::*;

// 创建测试数据库和仓库
async fn create_test_database() -> (Arc<DatabaseManager>, Arc<CryptoService>, ClaudeProviderRepository) {
    let db_config = DatabaseConfig {
        url: "sqlite::memory:".to_string(),
        max_connections: 20,
        min_connections: 5,
        connect_timeout: std::time::Duration::from_secs(10),
        idle_timeout: std::time::Duration::from_secs(60),
        max_lifetime: std::time::Duration::from_secs(300),
    };

    let db_manager = Arc::new(DatabaseManager::new(db_config).await.expect("Failed to create database"));
    
    // 运行数据库迁移
    sqlx::migrate!("./migrations")
        .run(db_manager.pool())
        .await
        .expect("Failed to run migrations");
    
    let crypto_service = Arc::new(CryptoService::new("test_key_for_db_bench").expect("Failed to create crypto service"));
    let repository = ClaudeProviderRepository::new(&db_manager, &crypto_service);
    
    (db_manager, crypto_service, repository)
}

// 基准测试：批量插入性能
fn bench_batch_insert(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (_db, _crypto, repository) = rt.block_on(create_test_database());
    
    c.bench_function("batch_insert_claude_providers", |b| {
        b.to_async(&rt).iter(|| async {
            let mut handles = vec![];
            
            // 批量插入100个记录
            for i in 0..100 {
                let repo = repository.clone();
                let handle = tokio::spawn(async move {
                    let request = CreateClaudeProviderRequest {
                        name: format!("Batch Provider {}", i),
                        url: "https://api.openai.com".to_string(),
                        token: format!("sk-batch-token-{}", i),
                        // max_tokens: Some(4096),
                        // temperature: Some(0.7),
                        // model: Some("gpt-4".to_string()),
                        // enabled: Some(if i % 3 == 0 { 1 } else { 0 }),
                        // description: Some(format!("Batch insert provider {}", i)),
                        timeout: Some(30),
                        auto_update: todo!(),
                        r#type: todo!(),
                        opus_model: todo!(),
                        sonnet_model: todo!(),
                        haiku_model: todo!(),
                        // retry_count: Some(3),
                    };
                    
                    repo.create_claude_provider(&request).await
                });
                handles.push(handle);
            }
            
            let results = futures::future::join_all(handles).await;
            black_box(results)
        });
    });
}

// 基准测试：分页查询性能
fn bench_paginated_query(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (_db, _crypto, repository) = rt.block_on(create_test_database());
    
    // 预先插入测试数据
    rt.block_on(async {
        for i in 0..1000 {
            let request = CreateClaudeProviderRequest {
                name: format!("Query Provider {}", i),
                url: "https://api.openai.com".to_string(),
                token: format!("sk-query-token-{}", i),
                // max_tokens: Some(4096),
                // temperature: Some(0.7),
                // model: Some("gpt-4".to_string()),
                // enabled: Some(if i % 5 == 0 { 1 } else { 0 }),
                // description: Some(format!("Query test provider {}", i)),
                timeout: Some(30),
                auto_update: todo!(),
                r#type: todo!(),
                opus_model: todo!(),
                sonnet_model: todo!(),
                haiku_model: todo!(),
                // retry_count: Some(3),
            };
            repository.create_claude_provider(&request).await.unwrap();
        }
    });
    
    c.bench_function("paginated_query", |b| {
        b.to_async(&rt).iter(|| async {
            let params = PaginationParams {
                page: black_box(Some(5)), // 查询第5页
                limit: black_box(Some(20)),
                offset: todo!(),
            };
            
            let result = repository.paginate::<ClaudeProvider>(&params).await;
            black_box(result)
        });
    });
}

// 基准测试：搜索查询性能
fn bench_search_query(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (_db, _crypto, repository) = rt.block_on(create_test_database());
    
    // 预先插入测试数据
    rt.block_on(async {
        let search_terms = vec!["Alpha", "Beta", "Gamma", "Delta", "Epsilon"];
        for i in 0..500 {
            let term = search_terms[i % search_terms.len()];
            let request = CreateClaudeProviderRequest {
                name: format!("{} Provider {}", term, i),
                url: "https://api.openai.com".to_string(),
                token: format!("sk-search-token-{}", i),
                // max_tokens: Some(4096),
                // temperature: Some(0.7),
                // model: Some("gpt-4".to_string()),
                // enabled: Some(1),
                // description: Some(format!("Search test provider {} with term {}", i, term)),
                timeout: Some(30),
                auto_update: todo!(),
                r#type: todo!(),
                opus_model: todo!(),
                sonnet_model: todo!(),
                haiku_model: todo!(),
                // retry_count: Some(3),
            };
            repository.create_claude_provider(&request).await.unwrap();
        }
    });
    
    c.bench_function("search_query", |b| {
        b.to_async(&rt).iter(|| async {
            let search_term = black_box("Alpha");
            let result = repository.search_claude_providers(search_term, Some(10)).await;
            black_box(result)
        });
    });
}

// 基准测试：数据库连接池性能
fn bench_connection_pool(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (db_manager, _crypto, _repository) = rt.block_on(create_test_database());
    
    c.bench_function("connection_pool_stress", |b| {
        b.to_async(&rt).iter(|| async {
            let mut handles = vec![];
            
            // 并发获取50个连接
            for _ in 0..50 {
                let db = db_manager.clone();
                let handle = tokio::spawn(async move {
                    // 执行简单查询
                    let result = sqlx::query("SELECT 1")
                        .fetch_one(db.pool())
                        .await;
                    black_box(result)
                });
                handles.push(handle);
            }
            
            let results = futures::future::join_all(handles).await;
            black_box(results)
        });
    });
}

// 基准测试：事务处理性能
fn bench_transaction_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (db_manager, _crypto, _repository) = rt.block_on(create_test_database());
    
    c.bench_function("transaction_batch_insert", |b| {
        b.to_async(&rt).iter(|| async {
            let mut tx = db_manager.pool().begin().await.unwrap();
            
            // 在事务中插入10条记录
            for i in 0..10 {
                let query = sqlx::query(
                    "INSERT INTO claude_providers (name, url, token, enabled) VALUES (?, ?, ?, ?)"
                )
                .bind(format!("Tx Provider {}", i))
                .bind("https://api.openai.com")
                .bind(format!("sk-tx-token-{}", i))
                .bind(1);
                
                black_box(query.execute(&mut *tx).await.unwrap());
            }
            
            let result = tx.commit().await;
            black_box(result)
        });
    });
}

// 基准测试：复杂查询性能
fn bench_complex_query(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (_db, _crypto, repository) = rt.block_on(create_test_database());
    
    // 预先插入测试数据
    rt.block_on(async {
        for i in 0..200 {
            let request = CreateClaudeProviderRequest {
                name: format!("Complex Provider {}", i),
                url: if i % 3 == 0 { "https://api.openai.com".to_string() } else { "https://api.anthropic.com".to_string() },
                token: format!("sk-complex-token-{}", i),
                // max_tokens: Some(1024 * (i % 4 + 1)),
                // temperature: Some(0.1 * (i % 10) as f64),
                // model: Some(if i % 2 == 0 { "gpt-4" } else { "gpt-3.5-turbo" }),
                // enabled: Some(if i % 4 == 0 { 1 } else { 0 }),
                // description: Some(format!("Complex query test provider {}", i)),
                timeout: Some(10 + i % 50),
                auto_update: todo!(),
                r#type: todo!(),
                opus_model: todo!(),
                sonnet_model: todo!(),
                haiku_model: todo!(),
                // retry_count: Some(i % 5),
            };
            repository.create_claude_provider(&request).await.unwrap();
        }
    });
    
    c.bench_function("complex_query", |b| {
        b.to_async(&rt).iter(|| async {
            // 执行复杂查询：查找启用的、token长度>15的、按创建时间排序
            let result = sqlx::query_as::<_, ClaudeProvider>(
                r#"
                SELECT id, name, url, token, max_tokens, temperature, model, 
                       enabled, description, timeout, retry_count, created_at, updated_at
                FROM claude_providers 
                WHERE enabled = 1 AND length(token) > 15
                ORDER BY created_at DESC
                LIMIT 20
                "#
            )
            .fetch_all(&repository.pool)
            .await;
            black_box(result)
        });
    });
}

// 基准测试：索引查询性能
fn bench_indexed_query(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (_db, _crypto, repository) = rt.block_on(create_test_database());
    
    // 创建索引
    rt.block_on(async {
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_claude_providers_enabled ON claude_providers(enabled)")
            .execute(&repository.pool)
            .await
            .unwrap();
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_claude_providers_name ON claude_providers(name)")
            .execute(&repository.pool)
            .await
            .unwrap();
    });
    
    // 预先插入测试数据
    rt.block_on(async {
        for i in 0..1000 {
            let request = CreateClaudeProviderRequest {
                name: format!("Indexed Provider {}", i),
                url: "https://api.openai.com".to_string(),
                token: format!("sk-indexed-token-{}", i),
                // max_tokens: Some(4096),
                // temperature: Some(0.7),
                // model: Some("gpt-4".to_string()),
                // enabled: Some(if i % 10 == 0 { 1 } else { 0 }), // 10%启用
                // description: Some(format!("Indexed query test provider {}", i)),
                timeout: Some(30),
                auto_update: todo!(),
                r#type: todo!(),
                opus_model: todo!(),
                sonnet_model: todo!(),
                haiku_model: todo!(),
                // retry_count: Some(3),
            };
            repository.create_claude_provider(&request).await.unwrap();
        }
    });
    
    c.bench_function("indexed_query", |b| {
        b.to_async(&rt).iter(|| async {
            // 使用索引查询启用的供应商
            let result = repository.list_active_providers().await;
            black_box(result)
        });
    });
}

criterion_group!(
    benches,
    bench_batch_insert,
    bench_paginated_query,
    bench_search_query,
    bench_connection_pool,
    bench_transaction_performance,
    bench_complex_query,
    bench_indexed_query
);
criterion_main!(benches);