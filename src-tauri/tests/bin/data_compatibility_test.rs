// 数据兼容性验证测试可执行程序
//
// 独立运行数据迁移和加密兼容性验证测试

use std::env;

// 将测试模块作为路径引入
#[path = "../data_compatibility_runner.rs"]
mod data_compatibility_runner;

use data_compatibility_runner::run_compatibility_tests;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 解析命令行参数
    let args: Vec<String> = env::args().collect();
    let verbose = args.contains(&"-v".to_string()) || args.contains(&"--verbose".to_string());

    println!("🔬 AI Manager 数据兼容性验证测试");
    println!("=====================================");
    println!();

    // 运行兼容性测试
    let result = run_compatibility_tests(verbose).await?;

    // 根据测试结果设置退出码
    if result.passed {
        println!();
        println!("✅ 所有兼容性测试通过！数据迁移验证成功。");
        std::process::exit(0);
    } else {
        println!();
        println!("❌ 兼容性测试失败！请检查上述错误信息。");
        std::process::exit(1);
    }
}
