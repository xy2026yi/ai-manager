// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Tauri imports
use tauri::State;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::path::PathBuf;

// Modules
mod commands;
mod models;
mod services;

// Commands
use commands::*;

// 应用状态
use commands::supplier::AppState;

#[tokio::main]
async fn main() {
    // 初始化数据库
    let app_data_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ai-tools-client");

    // 确保目录存在
    std::fs::create_dir_all(&app_data_dir)
        .expect("Failed to create app data directory");

    let db_path = app_data_dir.join("ai-tools.db");
    let db_url = format!("sqlite:{}", db_path.display());

    // 创建数据库连接池
    let database = services::database::Database::new(&db_url)
        .await
        .expect("Failed to initialize database");

    let app_state = AppState {
        db_pool: Arc::new(Mutex::new(database.pool.clone())),
    };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // Supplier commands
            list_suppliers,
            create_supplier,
            update_supplier,
            delete_supplier,
            get_supplier_by_id,
            set_active_supplier,
            test_supplier_connection,
            validate_supplier_config,
            get_supplier_stats,
            import_suppliers,
            export_suppliers,

            // MCP Template commands
            list_mcp_templates,
            create_mcp_template,
            update_mcp_template,
            delete_mcp_template,
            get_mcp_template_by_id,
            validate_mcp_template,
            get_mcp_template_categories,
            increment_template_usage,
            clone_mcp_template,
            get_mcp_template_stats,
            import_mcp_templates,
            export_mcp_templates,

            // Config commands
            backup_config,
            get_config_history,
            get_latest_config_backup,
            restore_config_from_backup,
            cleanup_old_config_history,
            delete_config_history,

            // Work mode commands
            get_work_mode_by_name,
            list_work_mode_configs,
            update_work_mode_by_id,
            switch_work_mode,
            get_work_mode_status,
            rollback_work_mode,

            // App state commands
            get_app_state,
            set_app_state,
            get_current_mode,
            get_all_app_states,

            // Database commands
            get_database_stats,
            test_database_connection,
            export_all_data,
        ])
        .setup(|_app| {
            // 数据库已在上面初始化
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}