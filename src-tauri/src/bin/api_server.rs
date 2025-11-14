// API服务器独立启动入口
//
// 可以独立运行的API服务器程序
// 用于测试和开发环境

use migration_ai_manager_lib::ApiServer;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🚀 启动AI Manager API服务器...");

    // 创建API服务器
    let server = ApiServer::new().await?;

    // 启动服务器
    server.run().await?;

    Ok(())
}