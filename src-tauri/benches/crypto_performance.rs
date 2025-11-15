// 加密性能基准测试
// 测试加密解密操作的性能和内存使用

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use migration_ai_manager_lib::crypto::CryptoService;
use std::time::Duration;

// 创建加密服务实例
fn create_crypto_service() -> CryptoService {
    CryptoService::new("benchmark_key_for_crypto_performance_tests").expect("Failed to create crypto service")
}

// 生成测试数据
fn generate_test_data(size: usize) -> String {
    let base_text = "这是一段用于加密性能测试的中文数据内容，包含各种中文字符和标点符号！@#$%^&*()_+-={}[]|:;\"'<>?,./";
    base_text.chars().cycle().take(size).collect()
}

// 基准测试：小数据加密性能
fn bench_small_data_encryption(c: &mut Criterion) {
    let crypto_service = create_crypto_service();
    let test_data = "小段测试数据";
    
    c.bench_function("small_data_encryption", |b| {
        b.iter(|| {
            let encrypted = crypto_service.encrypt(black_box(test_data));
            black_box(encrypted)
        });
    });
}

// 基准测试：小数据解密性能
fn bench_small_data_decryption(c: &mut Criterion) {
    let crypto_service = create_crypto_service();
    let test_data = "小段测试数据";
    let encrypted = crypto_service.encrypt(test_data).unwrap();
    
    c.bench_function("small_data_decryption", |b| {
        b.iter(|| {
            let decrypted = crypto_service.decrypt(black_box(&encrypted));
            black_box(decrypted)
        });
    });
}

// 基准测试：中等数据加密性能
fn bench_medium_data_encryption(c: &mut Criterion) {
    let crypto_service = create_crypto_service();
    let test_data = generate_test_data(1024); // 1KB数据
    
    c.bench_function("medium_data_encryption", |b| {
        b.iter(|| {
            let encrypted = crypto_service.encrypt(black_box(&test_data));
            black_box(encrypted)
        });
    });
}

// 基准测试：中等数据解密性能
fn bench_medium_data_decryption(c: &mut Criterion) {
    let crypto_service = create_crypto_service();
    let test_data = generate_test_data(1024); // 1KB数据
    let encrypted = crypto_service.encrypt(&test_data).unwrap();
    
    c.bench_function("medium_data_decryption", |b| {
        b.iter(|| {
            let decrypted = crypto_service.decrypt(black_box(&encrypted));
            black_box(decrypted)
        });
    });
}

// 基准测试：大数据加密性能
fn bench_large_data_encryption(c: &mut Criterion) {
    let crypto_service = create_crypto_service();
    let test_data = generate_test_data(10240); // 10KB数据
    
    c.bench_function("large_data_encryption", |b| {
        b.iter(|| {
            let encrypted = crypto_service.encrypt(black_box(&test_data));
            black_box(encrypted)
        });
    });
}

// 基准测试：大数据解密性能
fn bench_large_data_decryption(c: &mut Criterion) {
    let crypto_service = create_crypto_service();
    let test_data = generate_test_data(10240); // 10KB数据
    let encrypted = crypto_service.encrypt(&test_data).unwrap();
    
    c.bench_function("large_data_decryption", |b| {
        b.iter(|| {
            let decrypted = crypto_service.decrypt(black_box(&encrypted));
            black_box(decrypted)
        });
    });
}

// 基准测试：批量加密性能
fn bench_batch_encryption(c: &mut Criterion) {
    let crypto_service = create_crypto_service();
    let test_data_list: Vec<String> = (0..100).map(|i| format!("批量加密测试数据 {}", i)).collect();
    
    c.bench_function("batch_encryption", |b| {
        b.iter(|| {
            let results: Result<Vec<_>, _> = test_data_list
                .iter()
                .map(|data| crypto_service.encrypt(black_box(data)))
                .collect();
            black_box(results)
        });
    });
}

// 基准测试：批量解密性能
fn bench_batch_decryption(c: &mut Criterion) {
    let crypto_service = create_crypto_service();
    let test_data_list: Vec<String> = (0..100).map(|i| format!("批量解密测试数据 {}", i)).collect();
    let encrypted_list: Result<Vec<_>, _> = test_data_list
        .iter()
        .map(|data| crypto_service.encrypt(data))
        .collect();
    let encrypted_list = encrypted_list.unwrap();
    
    c.bench_function("batch_decryption", |b| {
        b.iter(|| {
            let results: Result<Vec<_>, _> = encrypted_list
                .iter()
                .map(|data| crypto_service.decrypt(black_box(data)))
                .collect();
            black_box(results)
        });
    });
}

// 基准测试：并发加密性能
fn bench_concurrent_encryption(c: &mut Criterion) {
    let crypto_service = create_crypto_service();
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("concurrent_encryption", |b| {
        b.to_async(&rt).iter(|| async {
            let mut handles = vec![];
            
            // 创建20个并发加密任务
            for i in 0..20 {
                let service = crypto_service.clone();
                let data = format!("并发加密测试数据 {}", i);
                
                let handle = tokio::task::spawn_blocking(move || {
                    service.encrypt(&data)
                });
                handles.push(handle);
            }
            
            let results = futures::future::join_all(handles).await;
            black_box(results)
        });
    });
}

// 基准测试：内存使用效率
fn bench_memory_efficiency(c: &mut Criterion) {
    let crypto_service = create_crypto_service();
    
    c.bench_function("memory_efficiency", |b| {
        b.iter(|| {
            // 创建大量加密实例来测试内存使用
            let mut encrypted_data = Vec::new();
            
            for i in 0..100 {
                let data = format!("内存效率测试数据 {}", i);
                let encrypted = crypto_service.encrypt(&data).unwrap();
                encrypted_data.push(encrypted);
            }
            
            let encrypted_count = encrypted_data.len();

            // 解密所有数据
            let mut decrypted_data = Vec::new();
            for encrypted in encrypted_data {
                let decrypted = crypto_service.decrypt(&encrypted).unwrap();
                decrypted_data.push(decrypted);
            }
            
            black_box((encrypted_count, decrypted_data.len()))
        });
    });
}

// 基准测试：不同密钥长度的性能
fn bench_different_key_sizes(c: &mut Criterion) {
    let short_key_service = CryptoService::new("short").expect("Failed to create short key service");
    let medium_key_service = CryptoService::new("medium_key_for_testing").expect("Failed to create medium key service");
    let long_key_service = CryptoService::new("this_is_a_very_long_key_for_testing_purposes").expect("Failed to create long key service");
    
    let test_data = "密钥长度性能测试数据";
    
    c.bench_function("short_key_encryption", |b| {
        b.iter(|| {
            let encrypted = short_key_service.encrypt(black_box(test_data));
            black_box(encrypted)
        });
    });
    
    c.bench_function("medium_key_encryption", |b| {
        b.iter(|| {
            let encrypted = medium_key_service.encrypt(black_box(test_data));
            black_box(encrypted)
        });
    });
    
    c.bench_function("long_key_encryption", |b| {
        b.iter(|| {
            let encrypted = long_key_service.encrypt(black_box(test_data));
            black_box(encrypted)
        });
    });
}

// 基准测试：错误处理性能
fn bench_error_handling(c: &mut Criterion) {
    let crypto_service = create_crypto_service();
    let valid_encrypted = crypto_service.encrypt("valid data").unwrap();
    let invalid_encrypted = "invalid_encrypted_data_that_will_fail_to_decrypt";
    
    c.bench_function("valid_decryption", |b| {
        b.iter(|| {
            let result = crypto_service.decrypt(black_box(&valid_encrypted));
            black_box(result)
        });
    });
    
    c.bench_function("invalid_decryption", |b| {
        b.iter(|| {
            let result = crypto_service.decrypt(black_box(invalid_encrypted));
            black_box(result)
        });
    });
}

// 基准测试：吞吐量测试
fn bench_throughput(c: &mut Criterion) {
    let crypto_service = create_crypto_service();
    let test_data = generate_test_data(512); // 512字节数据
    
    let mut group = c.benchmark_group("throughput");
    group.throughput(criterion::Throughput::Bytes(test_data.len() as u64));
    
    group.bench_function("encryption_throughput", |b| {
        b.iter(|| {
            let encrypted = crypto_service.encrypt(black_box(&test_data));
            black_box(encrypted)
        });
    });
    
    group.bench_function("decryption_throughput", |b| {
        let encrypted = crypto_service.encrypt(&test_data).unwrap();
        b.iter(|| {
            let decrypted = crypto_service.decrypt(black_box(&encrypted));
            black_box(decrypted)
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_small_data_encryption,
    bench_small_data_decryption,
    bench_medium_data_encryption,
    bench_medium_data_decryption,
    bench_large_data_encryption,
    bench_large_data_decryption,
    bench_batch_encryption,
    bench_batch_decryption,
    bench_concurrent_encryption,
    bench_memory_efficiency,
    bench_different_key_sizes,
    bench_error_handling,
    bench_throughput
);
criterion_main!(benches);