//! 一次性数据迁移工具
//! 从原Python版本的AI Manager数据库迁移数据到新的Rust/Tauri版本

use migration_ai_manager_lib::{CryptoService, DatabaseManager};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::env;
use std::path::Path;
use tracing::{error, info, warn};

#[derive(Debug, Serialize, Deserialize)]
struct MigratedData {
    claude_providers: usize,
    codex_providers: usize,
    agent_guides: usize,
    mcp_servers: usize,
    common_configs: usize,
    errors: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    info!("🚀 开始AI Manager数据迁移...");

    // 获取命令行参数
    let args: Vec<String> = env::args().collect();

    // 检查帮助参数
    if args.len() < 2 || args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_usage();
        return Ok(());
    }

    let mut source_db_path: String = String::new();
    let mut target_db_path: String = "data/ai_manager.db".to_string();
    let mut dry_run = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--dry-run" => dry_run = true,
            arg if arg.starts_with('-') => {
                error!("未知参数: {}", arg);
                return Ok(());
            }
            _ => {
                if source_db_path.is_empty() {
                    source_db_path = args[i].clone();
                } else if !args[i].starts_with('-') {
                    target_db_path = args[i].clone();
                }
            }
        }
        i += 1;
    }

    if source_db_path.is_empty() {
        print_usage();
        return Ok(());
    }

    info!("源数据库: {}", source_db_path);
    info!("目标数据库: {}", target_db_path);
    if dry_run {
        info!("模式: 预览模式（不会实际修改数据）");
    }

    // 检查源数据库是否存在
    if !Path::new(&source_db_path).exists() {
        error!("❌ 源数据库文件不存在: {}", source_db_path);
        return Ok(());
    }

    // 执行迁移
    let result = migrate_data(&source_db_path, &target_db_path, dry_run).await;

    match result {
        Ok(data) => {
            info!("✅ 数据迁移完成！");
            info!("📊 迁移统计:");
            info!("  - Claude供应商: {} 条", data.claude_providers);
            info!("  - Codex供应商: {} 条", data.codex_providers);
            info!("  - Agent指导: {} 条", data.agent_guides);
            info!("  - MCP服务器: {} 条", data.mcp_servers);
            info!("  - 通用配置: {} 条", data.common_configs);

            if !data.errors.is_empty() {
                warn!("⚠️  迁移过程中遇到 {} 个问题:", data.errors.len());
                for error in &data.errors {
                    warn!("  - {}", error);
                }
            }

            if dry_run {
                info!("🔍 这是预览模式，没有实际修改数据");
            }
        }
        Err(e) => {
            error!("❌ 迁移失败: {}", e);
        }
    }

    Ok(())
}

fn print_usage() {
    println!("用法: migrate_data <源数据库路径> [目标数据库路径] [--dry-run]");
    println!();
    println!("示例:");
    println!("  migrate_data ../ai-manager/ai_manager.db");
    println!("  migrate_data ../ai-manager/ai_manager.db data/new_ai_manager.db");
    println!("  migrate_data ../ai-manager/ai_manager.db --dry-run");
    println!();
    println!("参数:");
    println!("  源数据库路径    - 原Python版本的数据库文件");
    println!("  目标数据库路径  - 新Rust版本的数据库文件（默认: data/ai_manager.db）");
    println!("  --dry-run        - 预览模式，不实际修改数据");
}

async fn migrate_data(
    source_db_path: &str,
    target_db_path: &str,
    dry_run: bool,
) -> Result<MigratedData, Box<dyn std::error::Error>> {
    let mut migrated_data = MigratedData {
        claude_providers: 0,
        codex_providers: 0,
        agent_guides: 0,
        mcp_servers: 0,
        common_configs: 0,
        errors: Vec::new(),
    };

    // 连接源数据库
    info!("📖 连接源数据库...");
    let source_pool = sqlx::SqlitePool::connect(&format!("sqlite:{}", source_db_path)).await?;
    info!("✅ 源数据库连接成功");

    // 连接目标数据库
    info!("💾 连接目标数据库...");
    let target_db;
    if dry_run {
        info!("🔍 预览模式：跳过目标数据库连接");
        target_db = None;
    } else {
        // 创建目标数据库目录
        if let Some(parent) = Path::new(target_db_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let target_config = migration_ai_manager_lib::DatabaseConfig {
            url: format!("sqlite:{}", target_db_path),
            max_connections: 10,
            min_connections: 1,
            connect_timeout: std::time::Duration::from_secs(30),
            idle_timeout: std::time::Duration::from_secs(600),
            max_lifetime: std::time::Duration::from_secs(1800),
        };

        let db = DatabaseManager::new(target_config).await?;
        info!("✅ 目标数据库连接成功");
        target_db = Some(db);
    }

    // 获取加密密钥（这里使用固定的测试密钥，实际使用中应该从.env读取）
    let old_key = get_old_encryption_key();
    let new_key = get_new_encryption_key();

    let old_crypto = CryptoService::new(&old_key)?;
    let new_crypto = CryptoService::new(&new_key)?;

    info!("🔐 开始迁移数据...");

    // 迁移Claude供应商
    if let Some(ref db) = target_db {
        migrated_data.claude_providers =
            migrate_claude_providers(&source_pool, db, &old_crypto, &new_crypto, dry_run)
                .await
                .unwrap_or_else(|e| {
                    migrated_data.errors.push(format!("Claude供应商迁移失败: {}", e));
                    0
                });

        // 迁移Codex供应商
        migrated_data.codex_providers =
            migrate_codex_providers(&source_pool, db, &old_crypto, &new_crypto, dry_run)
                .await
                .unwrap_or_else(|e| {
                    migrated_data.errors.push(format!("Codex供应商迁移失败: {}", e));
                    0
                });

        // 迁移Agent指导文件
        migrated_data.agent_guides =
            migrate_agent_guides(&source_pool, db, dry_run).await.unwrap_or_else(|e| {
                migrated_data.errors.push(format!("Agent指导迁移失败: {}", e));
                0
            });

        // 迁移MCP服务器
        migrated_data.mcp_servers =
            migrate_mcp_servers(&source_pool, db, dry_run).await.unwrap_or_else(|e| {
                migrated_data.errors.push(format!("MCP服务器迁移失败: {}", e));
                0
            });

        // 迁移通用配置
        migrated_data.common_configs =
            migrate_common_configs(&source_pool, db, dry_run).await.unwrap_or_else(|e| {
                migrated_data.errors.push(format!("通用配置迁移失败: {}", e));
                0
            });
    } else {
        // 预览模式，只读取源数据数量
        migrated_data.claude_providers = sqlx::query("SELECT COUNT(*) FROM claude_providers")
            .fetch_one(&source_pool)
            .await?
            .get::<i64, _>(0) as usize;
        migrated_data.codex_providers = sqlx::query("SELECT COUNT(*) FROM codex_providers")
            .fetch_one(&source_pool)
            .await?
            .get::<i64, _>(0) as usize;
        migrated_data.agent_guides = sqlx::query("SELECT COUNT(*) FROM agent_guides")
            .fetch_one(&source_pool)
            .await?
            .get::<i64, _>(0) as usize;
        migrated_data.mcp_servers = sqlx::query("SELECT COUNT(*) FROM mcp_servers")
            .fetch_one(&source_pool)
            .await?
            .get::<i64, _>(0) as usize;
        migrated_data.common_configs = sqlx::query("SELECT COUNT(*) FROM common_configs")
            .fetch_one(&source_pool)
            .await?
            .get::<i64, _>(0) as usize;
    }

    Ok(migrated_data)
}

fn get_old_encryption_key() -> String {
    // 尝试从环境变量获取，如果没有则使用测试密钥
    env::var("OLD_FERNET_KEY").unwrap_or_else(|_| {
        warn!("未找到OLD_FERNET_KEY环境变量，使用默认测试密钥");
        "dGVzdCBrZXkgZm9yIGZlcm5ldCB0ZXN0aW5nIHVuaXQgdGVzdHM=".to_string() // 测试密钥
    })
}

fn get_new_encryption_key() -> String {
    // 尝试从环境变量获取，如果没有则使用测试密钥
    env::var("FERNET_KEY").unwrap_or_else(|_| {
        warn!("未找到FERNET_KEY环境变量，使用默认测试密钥");
        "Jw4Ff1BWLnSykdfXDVOuEJCG6m9dyST5B1VhU_qg0fI=".to_string() // 测试密钥
    })
}

async fn migrate_claude_providers(
    source_pool: &sqlx::SqlitePool,
    target_db: &DatabaseManager,
    old_crypto: &CryptoService,
    new_crypto: &CryptoService,
    dry_run: bool,
) -> Result<usize, Box<dyn std::error::Error>> {
    info!("🔄 迁移Claude供应商...");

    let rows = sqlx::query("SELECT * FROM claude_providers").fetch_all(source_pool).await?;

    info!("找到 {} 个Claude供应商", rows.len());

    let mut count = 0;
    for row in rows {
        let name: String = row.get("name");
        let _url: String = row.get("url");
        let encrypted_token: String = row.get("token");

        // 解密原始token
        let token = match old_crypto.decrypt(&encrypted_token) {
            Ok(t) => t,
            Err(e) => {
                warn!("无法解密Claude供应商 '{}' 的token: {}，跳过", name, e);
                continue;
            }
        };

        // 用新密钥加密token
        let new_encrypted_token = new_crypto.encrypt(&token)?;

        info!("  ✅ Claude供应商: {}", name);

        if !dry_run {
            // 插入到目标数据库
            sqlx::query(r#"
                INSERT INTO claude_providers (name, url, token, timeout, auto_update, type, enabled, opus_model, sonnet_model, haiku_model)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#)
            .bind(name.clone())
            .bind(row.get::<Option<String>, _>("url").unwrap_or_else(|| "https://api.anthropic.com".to_string()))
            .bind(new_encrypted_token)
            .bind(row.get::<Option<i64>, _>("timeout").unwrap_or(30000))
            .bind(row.get::<Option<i64>, _>("auto_update").unwrap_or(1))
            .bind(row.get::<Option<String>, _>("type").unwrap_or_else(|| "public_welfare".to_string()))
            .bind(row.get::<Option<i64>, _>("enabled").unwrap_or(0))
            .bind(row.get::<Option<String>, _>("opus_model"))
            .bind(row.get::<Option<String>, _>("sonnet_model"))
            .bind(row.get::<Option<String>, _>("haiku_model"))
            .execute(target_db.pool())
            .await?;
            count += 1;
        } else {
            count += 1;
        }
    }

    Ok(count)
}

async fn migrate_codex_providers(
    source_pool: &sqlx::SqlitePool,
    target_db: &DatabaseManager,
    old_crypto: &CryptoService,
    new_crypto: &CryptoService,
    dry_run: bool,
) -> Result<usize, Box<dyn std::error::Error>> {
    info!("🔄 迁移Codex供应商...");

    let rows = sqlx::query("SELECT * FROM codex_providers").fetch_all(source_pool).await?;

    info!("找到 {} 个Codex供应商", rows.len());

    let mut count = 0;
    for row in rows {
        let name: String = row.get("name");
        let encrypted_token: String = row.get("token");

        let token = match old_crypto.decrypt(&encrypted_token) {
            Ok(t) => t,
            Err(e) => {
                warn!("无法解密Codex供应商 '{}' 的token: {}，跳过", name, e);
                continue;
            }
        };

        let new_encrypted_token = new_crypto.encrypt(&token)?;
        info!("  ✅ Codex供应商: {}", name);

        if !dry_run {
            // 插入到目标数据库
            sqlx::query(
                r#"
                INSERT INTO codex_providers (name, url, token, type, enabled)
                VALUES (?, ?, ?, ?, ?)
            "#,
            )
            .bind(name.clone())
            .bind(
                row.get::<Option<String>, _>("url")
                    .unwrap_or_else(|| "https://api.openai.com".to_string()),
            )
            .bind(new_encrypted_token)
            .bind(
                row.get::<Option<String>, _>("type")
                    .unwrap_or_else(|| "public_welfare".to_string()),
            )
            .bind(row.get::<Option<i64>, _>("enabled").unwrap_or(0))
            .execute(target_db.pool())
            .await?;
            count += 1;
        } else {
            count += 1;
        }
    }

    Ok(count)
}

async fn migrate_agent_guides(
    source_pool: &sqlx::SqlitePool,
    target_db: &DatabaseManager,
    dry_run: bool,
) -> Result<usize, Box<dyn std::error::Error>> {
    info!("🔄 迁移Agent指导文件...");

    let rows = sqlx::query("SELECT * FROM agent_guides").fetch_all(source_pool).await?;

    info!("找到 {} 个Agent指导文件", rows.len());

    let mut count = 0;
    for row in rows {
        let name: String = row.get("name");
        let guide_type: String = row.get("type");
        let text: String = row.get("text");

        info!("  ✅ Agent指导: {}", name);

        if !dry_run {
            // 插入到目标数据库
            sqlx::query(
                r#"
                INSERT INTO agent_guides (name, type, text)
                VALUES (?, ?, ?)
            "#,
            )
            .bind(name.clone())
            .bind(guide_type)
            .bind(text)
            .execute(target_db.pool())
            .await?;
            count += 1;
        } else {
            count += 1;
        }
    }

    Ok(count)
}

async fn migrate_mcp_servers(
    source_pool: &sqlx::SqlitePool,
    target_db: &DatabaseManager,
    dry_run: bool,
) -> Result<usize, Box<dyn std::error::Error>> {
    info!("🔄 迁移MCP服务器...");

    let rows = sqlx::query("SELECT * FROM mcp_servers").fetch_all(source_pool).await?;

    info!("找到 {} 个MCP服务器", rows.len());

    let mut count = 0;
    for row in rows {
        let name: String = row.get("name");
        let server_type: Option<String> = row.get("type");
        let timeout: Option<i64> = row.get("timeout");
        let command: String = row.get("command");
        let args: String = row.get("args");
        let env: Option<String> = row.get("env");

        info!("  ✅ MCP服务器: {}", name);

        if !dry_run {
            // 插入到目标数据库
            sqlx::query(
                r#"
                INSERT INTO mcp_servers (name, type, timeout, command, args, env)
                VALUES (?, ?, ?, ?, ?, ?)
            "#,
            )
            .bind(name.clone())
            .bind(server_type)
            .bind(timeout.unwrap_or(30000))
            .bind(command)
            .bind(args)
            .bind(env)
            .execute(target_db.pool())
            .await?;
            count += 1;
        } else {
            count += 1;
        }
    }

    Ok(count)
}

async fn migrate_common_configs(
    source_pool: &sqlx::SqlitePool,
    target_db: &DatabaseManager,
    dry_run: bool,
) -> Result<usize, Box<dyn std::error::Error>> {
    info!("🔄 迁移通用配置...");

    let rows = sqlx::query("SELECT * FROM common_configs").fetch_all(source_pool).await?;

    info!("找到 {} 个通用配置", rows.len());

    let mut count = 0;
    for row in rows {
        let key: String = row.get("key");
        let value: String = row.get("value");
        let description: Option<String> = row.get("description");
        let category: Option<String> = row.get("category");
        let is_active: Option<i64> = row.get("is_active");

        info!("  ✅ 配置项: {}", key);

        if !dry_run {
            // 插入到目标数据库
            sqlx::query(
                r#"
                INSERT INTO common_configs (key, value, description, category, is_active)
                VALUES (?, ?, ?, ?, ?)
            "#,
            )
            .bind(key.clone())
            .bind(value)
            .bind(description)
            .bind(category.unwrap_or_else(|| "general".to_string()))
            .bind(is_active.unwrap_or(1))
            .execute(target_db.pool())
            .await?;
            count += 1;
        } else {
            count += 1;
        }
    }

    Ok(count)
}
