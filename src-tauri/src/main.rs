// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Tauri 基础命令
#[tauri::command]
fn greet(name: &str) -> String {
    format!("你好, {}! AI Manager 后端已就绪。", name)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用时出错");
}