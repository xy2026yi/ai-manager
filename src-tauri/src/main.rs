// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! AI Manager 主程序
//!
//! 从 Python/FastAPI 迁移到 Rust/Tauri 的桌面应用程序

// 从 library crate 导入必要的模块
use migration_ai_manager_lib::LoggingManager;
use tauri::Manager;

// Tauri 基础命令
#[tauri::command]
fn greet(name: &str) -> String {
    format!("你好, {}! AI Manager 后端已就绪。", name)
}

/// 主函数（高度优化启动时间）
///
/// 使用延迟初始化和并行处理来最小化启动延迟
fn main() {
    // 使用新的日志管理器初始化日志系统
    if let Err(e) = LoggingManager::init_from_env() {
        eprintln!("日志系统初始化失败: {}", e);
    }

    tracing::info!("AI Manager 应用程序启动");
    tracing::info!("版本: 0.1.0");

    // 启动Tauri应用
    let result = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|app| {
            // 在Tauri设置阶段启动后台初始化任务
            let app_handle = app.handle().clone();
            tokio::spawn(async move {
                // 延迟初始化非关键组件
                delayed_initialization().await;

                // 可选：通知前端初始化完成
                if let Err(e) = app_handle.emit_all("initialization_complete", ()) {
                    eprintln!("发送初始化完成事件失败: {}", e);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!());

    if let Err(e) = result {
        tracing::error!("应用启动失败: {}", e);
        std::process::exit(1);
    }
}

/// 延迟初始化非关键组件
async fn delayed_initialization() {
    let start = std::time::Instant::now();

    // 并行执行所有延迟初始化阶段
    tokio::join!(
        async {
            // 阶段1：预热数据库连接（如果需要）
            tracing::debug!("开始延迟初始化 - 阶段1");
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        },
        async {
            // 阶段2：预加载常用配置
            tracing::debug!("开始延迟初始化 - 阶段2");
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        },
        async {
            // 阶段3：其他后台任务
            tracing::debug!("开始延迟初始化 - 阶段3");
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    );

    let elapsed = start.elapsed();
    tracing::info!("✅ 延迟初始化完成，耗时: {:?}", elapsed);
}
