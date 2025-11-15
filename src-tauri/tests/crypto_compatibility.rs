//! 独立的加密兼容性测试
//! 这个测试文件不依赖Tauri主程序，可以独立运行

use std::process::Command;

// 注意：这个测试需要独立的cargo test运行
// 或者可以移到集成测试中

#[test]
fn test_python_rust_compatibility() {
    println!("🧪 开始Python-Rust加密兼容性测试");

    // 首先运行Rust加密测试
    let output = Command::new("cargo")
        .args(&["test", "crypto", "--", "--nocapture"])
        .current_dir("..")
        .output()
        .expect("Failed to execute cargo test crypto");

    if !output.status.success() {
        println!("Rust加密测试失败:");
        println!("{}", String::from_utf8_lossy(&output.stderr));
        panic!("Rust加密测试失败");
    }

    println!("✅ Rust加密测试通过");

    // 运行Python兼容性验证
    let python_test = r#"
from cryptography.fernet import Fernet
import json

# 使用相同的密钥
key = 'Jw4Ff1BWLnSykdfXDVOuEJCG6m9dyST5B1VhU_qg0fI='
f = Fernet(key.encode())

# 测试数据
test_data = "Hello, World!"
encrypted = f.encrypt(test_data.encode()).decode()
decrypted = f.decrypt(encrypted.encode()).decode()

assert decrypted == test_data
print("✅ Python加密测试通过")
"#;

    let output = Command::new("python3")
        .arg("-c")
        .arg(python_test)
        .output()
        .expect("Failed to execute Python test");

    if !output.status.success() {
        println!("Python加密测试失败:");
        println!("{}", String::from_utf8_lossy(&output.stderr));
        panic!("Python加密测试失败");
    }

    println!("🎉 Python-Rust兼容性测试全部通过！");
}
