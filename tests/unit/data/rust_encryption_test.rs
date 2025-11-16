
use fernet::Fernet;

fn main() {
    // 使用与Python相同的测试密钥
    let key = "Jw4Ff1BWLnSykdfXDVOuEJCG6m9dyST5B1VhU_qg0fI=";
    let fernet = Fernet::new(key).expect("Invalid key");
    
    let test_cases = vec![
        "Hello, World!",
        "测试中文",
        "sk-ant-api03-test-key-123",
        "🔒🔐🔑",
        "",
    ];
    
    let mut success = true;
    let mut total_time = std::time::Duration::new(0, 0);
    
    for (i, test_data) in test_cases.iter().enumerate() {
        let start = std::time::Instant::now();
        
        // 加密
        let encrypted = fernet.encrypt(test_data.as_bytes());
        
        // 解密
        let decrypted = fernet.decrypt(&encrypted);
        
        let elapsed = start.elapsed();
        total_time += elapsed;
        
        match decrypted {
            Ok(decrypted_bytes) => {
                let decrypted_str = String::from_utf8(decrypted_bytes).unwrap_or_default();
                if decrypted_str == *test_data {
                    println!("✅ Test {}: '{}', Time: {:?}", i, test_data, elapsed);
                } else {
                    println!("❌ Test {}: '{}', Decrypted: '{}'", i, test_data, decrypted_str);
                    success = false;
                }
            }
            Err(e) => {
                println!("❌ Test {}: '{}', Error: {}", i, test_data, e);
                success = false;
            }
        }
    }
    
    println!("Average time per operation: {:?}", total_time / test_cases.len() as u32);
    
    if success {
        println!("🎉 所有加密兼容性测试通过!");
        std::process::exit(0);
    } else {
        println!("❌ 加密兼容性测试失败!");
        std::process::exit(1);
    }
}
            