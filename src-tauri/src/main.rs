// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! AI Manager 主程序
//!
//! 从 Python/FastAPI 迁移到 Rust/Tauri 的桌面应用程序

mod crypto;
mod models;
mod python_compatibility_test;
mod test_deps;

// 导入模块
mod api;
mod database;
mod logging;
mod migration_tool;
mod repositories;
mod services;

use logging::init_development;

// Tauri 基础命令
#[tauri::command]
fn greet(name: &str) -> String {
    format!("你好, {}! AI Manager 后端已就绪。", name)
}

/// 主函数（优化启动时间）
///
/// 初始化日志系统并启动 Tauri 应用程序
fn main() {
    // 使用异步运行时来优化启动时间
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create async runtime");

    rt.block_on(async {
        // 异步初始化日志系统，不阻塞主线程
        let logging_fut = async {
            if let Err(e) = init_development() {
                eprintln!("日志系统初始化失败: {}", e);
            }
        };

        // 异步记录应用启动信息
        let startup_info_fut = async {
            tracing::info!("AI Manager 应用程序启动");
            tracing::info!("版本: 0.1.0");
            tracing::info!("环境: 开发模式");
        };

        // 并行执行非关键启动任务
        tokio::join!(logging_fut, startup_info_fut);

        // 构建并运行 Tauri 应用
        let result = tauri::Builder::default()
            .invoke_handler(tauri::generate_handler![greet])
            .run(tauri::generate_context!());

        if let Err(e) = result {
            tracing::error!("应用启动失败: {}", e);
            return Err(e);
        }

        Ok(())
    })
    .expect("Failed to run async main");
}
