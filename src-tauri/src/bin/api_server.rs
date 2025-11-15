// API服务器独立启动入口
//
// 可以独立运行的API服务器程序
// 用于测试和开发环境
// 支持命令行参数配置和优雅关闭

use clap::{Arg, Command};
use migration_ai_manager_lib::{api::server::ApiServerConfig, ApiServer};
use std::net::SocketAddr;
use tokio::signal;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 解析命令行参数
    let matches = Command::new("AI Manager API Server")
        .version("1.0.0")
        .about("AI Manager 数据管理API服务")
        .arg(
            Arg::new("host")
                .short('H')
                .long("host")
                .value_name("HOST")
                .help("服务器监听地址")
                .default_value("127.0.0.1"),
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .help("服务器监听端口")
                .value_parser(clap::value_parser!(u16))
                .default_value("8080"),
        )
        .arg(
            Arg::new("no-cors")
                .long("no-cors")
                .help("禁用CORS支持")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no-tracing")
                .long("no-tracing")
                .help("禁用请求追踪日志")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("log-level")
                .short('l')
                .long("log-level")
                .value_name("LEVEL")
                .help("日志级别")
                .value_parser(["trace", "debug", "info", "warn", "error"])
                .default_value("info"),
        )
        .get_matches();

    // 初始化日志系统
    let log_level = matches.get_one::<String>("log-level").unwrap();
    let level = match log_level.as_str() {
        "trace" => tracing::Level::TRACE,
        "debug" => tracing::Level::DEBUG,
        "info" => tracing::Level::INFO,
        "warn" => tracing::Level::WARN,
        "error" => tracing::Level::ERROR,
        _ => tracing::Level::INFO,
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                tracing_subscriber::EnvFilter::new(format!("migration_ai_manager_lib={}", level))
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 读取配置
    let host = matches.get_one::<String>("host").unwrap().clone();
    let port = *matches.get_one::<u16>("port").unwrap();
    let enable_cors = !matches.get_flag("no-cors");
    let enable_tracing = !matches.get_flag("no-tracing");

    // 验证配置
    let addr = format!("{}:{}", host, port)
        .parse::<SocketAddr>()
        .map_err(|e| format!("无效的服务器地址 {}: {}", host, e))?;

    info!("🚀 启动AI Manager API服务器");
    info!("📍 监听地址: http://{}", addr);
    info!("🔧 CORS支持: {}", if enable_cors { "启用" } else { "禁用" });
    info!(
        "📊 追踪日志: {}",
        if enable_tracing { "启用" } else { "禁用" }
    );

    // 创建API服务器配置
    let config = ApiServerConfig { host, port, enable_cors, enable_tracing };

    // 创建API服务器
    let server = ApiServer::with_config(config).await?;

    // 设置优雅关闭
    let shutdown_signal = async {
        signal::ctrl_c().await.expect("无法监听Ctrl+C信号");

        warn!("📡 收到终止信号，开始优雅关闭...");
    };

    // 启动服务器并等待关闭信号
    tokio::select! {
        result = server.run() => {
            if let Err(e) = result {
                error!("❌ 服务器运行出错: {}", e);
                return Err(e);
            }
        }
        _ = shutdown_signal => {
            info!("✅ 服务器已优雅关闭");
        }
    }

    Ok(())
}
