// 数据迁移兼容性测试
// 验证从原Python项目迁移数据的完整性和格式一致性

use chrono::Utc;
use migration_ai_manager_lib::crypto::CryptoService;
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;

// 测试数据结构
#[derive(Debug)]
struct TestDataRecord {
    table_name: String,
    original_count: i64,
    migrated_count: i64,
    #[allow(dead_code)]
    mismatched_fields: Vec<String>,
    integrity_issues: Vec<String>,
}

// 数据完整性验证器
struct DataIntegrityValidator {
    original_db: SqlitePool,
    migrated_db: SqlitePool,
    #[allow(dead_code)]
    crypto_service: CryptoService,
}

impl DataIntegrityValidator {
    async fn new(
        original_db_path: &str,
        migrated_db_path: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let original_db = SqlitePool::connect(original_db_path).await?;
        let migrated_db = SqlitePool::connect(migrated_db_path).await?;

        // 使用与原Python项目相同的密钥
        let crypto_service = CryptoService::new("test_migration_key_32_bytes_long!")?;

        Ok(Self { original_db, migrated_db, crypto_service })
    }

    // 验证表结构一致性
    async fn verify_table_schemas(
        &self,
    ) -> Result<Vec<TestDataRecord>, Box<dyn std::error::Error>> {
        let tables = vec![
            "claude_providers",
            "codex_providers",
            "agent_guides",
            "mcp_servers",
            "common_configs",
        ];

        let mut results = Vec::new();

        for table in tables {
            let result = self.verify_single_table_schema(table).await?;
            results.push(result);
        }

        Ok(results)
    }

    // 验证单个表的Schema
    async fn verify_single_table_schema(
        &self,
        table_name: &str,
    ) -> Result<TestDataRecord, Box<dyn std::error::Error>> {
        let original_schema = self.get_table_schema(&self.original_db, table_name).await?;
        let migrated_schema = self.get_table_schema(&self.migrated_db, table_name).await?;

        let mut mismatched_fields = Vec::new();

        // 比较字段定义
        for (field_name, original_def) in &original_schema {
            match migrated_schema.get(field_name) {
                Some(migrated_def) => {
                    if original_def != migrated_def {
                        mismatched_fields.push(format!(
                            "字段 {}: 原始 '{}' vs 迁移后 '{}'",
                            field_name, original_def, migrated_def
                        ));
                    }
                }
                None => {
                    mismatched_fields.push(format!("迁移后缺少字段: {}", field_name));
                }
            }
        }

        // 检查新增字段
        for field_name in migrated_schema.keys() {
            if !original_schema.contains_key(field_name) {
                mismatched_fields.push(format!("新增字段: {}", field_name));
            }
        }

        Ok(TestDataRecord {
            table_name: table_name.to_string(),
            original_count: 0,
            migrated_count: 0,
            mismatched_fields,
            integrity_issues: Vec::new(),
        })
    }

    // 获取表结构信息
    async fn get_table_schema(
        &self,
        pool: &SqlitePool,
        table_name: &str,
    ) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        let query = format!("PRAGMA table_info({})", table_name);

        let rows = sqlx::query(&query).fetch_all(pool).await?;
        let mut schema = HashMap::new();

        for row in rows {
            let name: String = row.get("name");
            let type_name: String = row.get("type");
            let not_null: i32 = row.get("notnull");
            let default_value: Option<String> = row.get("dflt_value");
            let primary_key: i32 = row.get("pk");

            let def = format!(
                "TYPE:{} NOT_NULL:{} DEFAULT:{:?} PK:{}",
                type_name, not_null, default_value, primary_key
            );

            schema.insert(name, def);
        }

        Ok(schema)
    }

    // 验证数据行数一致性
    async fn verify_row_counts(&self) -> Result<Vec<TestDataRecord>, Box<dyn std::error::Error>> {
        let tables = vec![
            "claude_providers",
            "codex_providers",
            "agent_guides",
            "mcp_servers",
            "common_configs",
        ];

        let mut results = Vec::new();

        for table in tables {
            let original_count = self.get_row_count(&self.original_db, table).await?;
            let migrated_count = self.get_row_count(&self.migrated_db, table).await?;

            let mut integrity_issues = Vec::new();
            if original_count != migrated_count {
                integrity_issues.push(format!(
                    "行数不匹配: 原始 {} vs 迁移后 {}",
                    original_count, migrated_count
                ));
            }

            results.push(TestDataRecord {
                table_name: table.to_string(),
                original_count,
                migrated_count,
                mismatched_fields: Vec::new(),
                integrity_issues,
            });
        }

        Ok(results)
    }

    // 获取表的行数
    async fn get_row_count(
        &self,
        pool: &SqlitePool,
        table_name: &str,
    ) -> Result<i64, Box<dyn std::error::Error>> {
        let query = format!("SELECT COUNT(*) as count FROM {}", table_name);
        let row = sqlx::query(&query).fetch_one(pool).await?;
        Ok(row.get("count"))
    }

    // 验证数据内容一致性（非加密字段）
    async fn verify_unencrypted_data(
        &self,
    ) -> Result<Vec<TestDataRecord>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();

        // 验证Claude供应商（非加密字段）
        let claude_result = self.verify_claude_providers().await?;
        results.push(claude_result);

        // 验证Codex供应商（非加密字段）
        let codex_result = self.verify_codex_providers().await?;
        results.push(codex_result);

        // 验证Agent指导（非加密字段）
        let guide_result = self.verify_agent_guides().await?;
        results.push(guide_result);

        // 验证MCP服务器（非加密字段）
        let server_result = self.verify_mcp_servers().await?;
        results.push(server_result);

        // 验证通用配置（非加密字段）
        let config_result = self.verify_common_configs().await?;
        results.push(config_result);

        Ok(results)
    }

    // 验证Claude供应商数据
    async fn verify_claude_providers(&self) -> Result<TestDataRecord, Box<dyn std::error::Error>> {
        let original_query = "SELECT id, name, url, max_tokens, temperature, model, enabled, description, timeout, retry_count, created_at, updated_at FROM claude_providers ORDER BY id";
        let migrated_query = "SELECT id, name, url, max_tokens, temperature, model, enabled, description, timeout, retry_count, created_at, updated_at FROM claude_providers ORDER BY id";

        let original_rows = sqlx::query(original_query).fetch_all(&self.original_db).await?;
        let migrated_rows = sqlx::query(migrated_query).fetch_all(&self.migrated_db).await?;

        let mut integrity_issues = Vec::new();

        if original_rows.len() != migrated_rows.len() {
            integrity_issues.push(format!(
                "Claude供应商记录数不匹配: 原始 {} vs 迁移后 {}",
                original_rows.len(),
                migrated_rows.len()
            ));
        }

        for (i, (orig_row, mig_row)) in original_rows.iter().zip(migrated_rows.iter()).enumerate() {
            let orig_id: i64 = orig_row.get("id");
            let mig_id: i64 = mig_row.get("id");

            if orig_id != mig_id {
                integrity_issues.push(format!(
                    "记录 {} ID不匹配: 原始 {} vs 迁移后 {}",
                    i, orig_id, mig_id
                ));
                continue;
            }

            // 验证非加密字段
            let fields_to_check = vec![
                "name",
                "url",
                "max_tokens",
                "temperature",
                "model",
                "enabled",
                "description",
                "timeout",
                "retry_count",
            ];

            for field in fields_to_check {
                let orig_val: Option<String> = orig_row.try_get(field).unwrap_or(None);
                let mig_val: Option<String> = mig_row.try_get(field).unwrap_or(None);

                if orig_val != mig_val {
                    integrity_issues.push(format!(
                        "记录ID {} 字段 '{}' 不匹配: 原始 {:?} vs 迁移后 {:?}",
                        orig_id, field, orig_val, mig_val
                    ));
                }
            }
        }

        Ok(TestDataRecord {
            table_name: "claude_providers".to_string(),
            original_count: original_rows.len() as i64,
            migrated_count: migrated_rows.len() as i64,
            mismatched_fields: Vec::new(),
            integrity_issues,
        })
    }

    // 验证Codex供应商数据
    async fn verify_codex_providers(&self) -> Result<TestDataRecord, Box<dyn std::error::Error>> {
        let query = "SELECT id, name, url, type, enabled FROM codex_providers ORDER BY id";

        let original_rows = sqlx::query(query).fetch_all(&self.original_db).await?;
        let migrated_rows = sqlx::query(query).fetch_all(&self.migrated_db).await?;

        let mut integrity_issues = Vec::new();

        if original_rows.len() != migrated_rows.len() {
            integrity_issues.push(format!(
                "Codex供应商记录数不匹配: 原始 {} vs 迁移后 {}",
                original_rows.len(),
                migrated_rows.len()
            ));
        }

        for (i, (orig_row, mig_row)) in original_rows.iter().zip(migrated_rows.iter()).enumerate() {
            let orig_id: i64 = orig_row.get("id");
            let mig_id: i64 = mig_row.get("id");

            if orig_id != mig_id {
                integrity_issues.push(format!("记录ID {} 不匹配", i));
                continue;
            }

            // 验证字段
            let fields = vec!["name", "url", "type", "enabled"];
            for field in fields {
                let orig_val: Option<String> = orig_row.try_get(field).unwrap_or(None);
                let mig_val: Option<String> = mig_row.try_get(field).unwrap_or(None);

                if orig_val != mig_val {
                    integrity_issues
                        .push(format!("Codex供应商ID {} 字段 '{}' 不匹配", orig_id, field));
                }
            }
        }

        Ok(TestDataRecord {
            table_name: "codex_providers".to_string(),
            original_count: original_rows.len() as i64,
            migrated_count: migrated_rows.len() as i64,
            mismatched_fields: Vec::new(),
            integrity_issues,
        })
    }

    // 验证Agent指导数据
    async fn verify_agent_guides(&self) -> Result<TestDataRecord, Box<dyn std::error::Error>> {
        let query =
            "SELECT id, name, description, created_at, updated_at FROM agent_guides ORDER BY id";

        let original_rows = sqlx::query(query).fetch_all(&self.original_db).await?;
        let migrated_rows = sqlx::query(query).fetch_all(&self.migrated_db).await?;

        let mut integrity_issues = Vec::new();

        if original_rows.len() != migrated_rows.len() {
            integrity_issues.push(format!(
                "Agent指导记录数不匹配: 原始 {} vs 迁移后 {}",
                original_rows.len(),
                migrated_rows.len()
            ));
        }

        for (i, (orig_row, mig_row)) in original_rows.iter().zip(migrated_rows.iter()).enumerate() {
            let orig_id: i64 = orig_row.get("id");
            let mig_id: i64 = mig_row.get("id");

            if orig_id != mig_id {
                integrity_issues.push(format!("Agent指导记录ID {} 不匹配", i));
                continue;
            }

            // 验证字段
            let fields = vec!["name", "description"];
            for field in fields {
                let orig_val: Option<String> = orig_row.try_get(field).unwrap_or(None);
                let mig_val: Option<String> = mig_row.try_get(field).unwrap_or(None);

                if orig_val != mig_val {
                    integrity_issues
                        .push(format!("Agent指导ID {} 字段 '{}' 不匹配", orig_id, field));
                }
            }
        }

        Ok(TestDataRecord {
            table_name: "agent_guides".to_string(),
            original_count: original_rows.len() as i64,
            migrated_count: migrated_rows.len() as i64,
            mismatched_fields: Vec::new(),
            integrity_issues,
        })
    }

    // 验证MCP服务器数据
    async fn verify_mcp_servers(&self) -> Result<TestDataRecord, Box<dyn std::error::Error>> {
        let query = "SELECT id, name, url, command, args, enabled, description FROM mcp_servers ORDER BY id";

        let original_rows = sqlx::query(query).fetch_all(&self.original_db).await?;
        let migrated_rows = sqlx::query(query).fetch_all(&self.migrated_db).await?;

        let mut integrity_issues = Vec::new();

        if original_rows.len() != migrated_rows.len() {
            integrity_issues.push(format!(
                "MCP服务器记录数不匹配: 原始 {} vs 迁移后 {}",
                original_rows.len(),
                migrated_rows.len()
            ));
        }

        for (i, (orig_row, mig_row)) in original_rows.iter().zip(migrated_rows.iter()).enumerate() {
            let orig_id: i64 = orig_row.get("id");
            let mig_id: i64 = mig_row.get("id");

            if orig_id != mig_id {
                integrity_issues.push(format!("MCP服务器记录ID {} 不匹配", i));
                continue;
            }

            // 验证字段
            let fields = vec!["name", "url", "command", "args", "enabled", "description"];
            for field in fields {
                let orig_val: Option<String> = orig_row.try_get(field).unwrap_or(None);
                let mig_val: Option<String> = mig_row.try_get(field).unwrap_or(None);

                if orig_val != mig_val {
                    integrity_issues
                        .push(format!("MCP服务器ID {} 字段 '{}' 不匹配", orig_id, field));
                }
            }
        }

        Ok(TestDataRecord {
            table_name: "mcp_servers".to_string(),
            original_count: original_rows.len() as i64,
            migrated_count: migrated_rows.len() as i64,
            mismatched_fields: Vec::new(),
            integrity_issues,
        })
    }

    // 验证通用配置数据
    async fn verify_common_configs(&self) -> Result<TestDataRecord, Box<dyn std::error::Error>> {
        let query = "SELECT id, key, value, type, description, created_at, updated_at FROM common_configs ORDER BY id";

        let original_rows = sqlx::query(query).fetch_all(&self.original_db).await?;
        let migrated_rows = sqlx::query(query).fetch_all(&self.migrated_db).await?;

        let mut integrity_issues = Vec::new();

        if original_rows.len() != migrated_rows.len() {
            integrity_issues.push(format!(
                "通用配置记录数不匹配: 原始 {} vs 迁移后 {}",
                original_rows.len(),
                migrated_rows.len()
            ));
        }

        for (i, (orig_row, mig_row)) in original_rows.iter().zip(migrated_rows.iter()).enumerate() {
            let orig_id: i64 = orig_row.get("id");
            let mig_id: i64 = mig_row.get("id");

            if orig_id != mig_id {
                integrity_issues.push(format!("通用配置记录ID {} 不匹配", i));
                continue;
            }

            // 验证字段
            let fields = vec!["key", "value", "type", "description"];
            for field in fields {
                let orig_val: Option<String> = orig_row.try_get(field).unwrap_or(None);
                let mig_val: Option<String> = mig_row.try_get(field).unwrap_or(None);

                if orig_val != mig_val {
                    integrity_issues
                        .push(format!("通用配置ID {} 字段 '{}' 不匹配", orig_id, field));
                }
            }
        }

        Ok(TestDataRecord {
            table_name: "common_configs".to_string(),
            original_count: original_rows.len() as i64,
            migrated_count: migrated_rows.len() as i64,
            mismatched_fields: Vec::new(),
            integrity_issues,
        })
    }

    // 生成数据完整性报告
    #[allow(dead_code)]
    fn generate_integrity_report(
        &self,
        schema_results: Vec<TestDataRecord>,
        count_results: Vec<TestDataRecord>,
        data_results: Vec<TestDataRecord>,
    ) -> String {
        let mut report = String::new();

        report.push_str("# 数据迁移完整性验证报告\n\n");
        report.push_str(&format!(
            "生成时间: {}\n\n",
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));

        // 表结构验证结果
        report.push_str("## 表结构验证\n\n");
        for result in &schema_results {
            report.push_str(&format!("### {}\n", result.table_name));
            if result.mismatched_fields.is_empty() {
                report.push_str("✅ 表结构完全一致\n\n");
            } else {
                report.push_str("❌ 发现字段不匹配:\n");
                for field in &result.mismatched_fields {
                    report.push_str(&format!("- {}\n", field));
                }
                report.push_str("\n");
            }
        }

        // 数据行数验证结果
        report.push_str("## 数据行数验证\n\n");
        for result in &count_results {
            report.push_str(&format!("### {}\n", result.table_name));
            if result.integrity_issues.is_empty() {
                report.push_str(&format!(
                    "✅ 行数一致: {} -> {}\n\n",
                    result.original_count, result.migrated_count
                ));
            } else {
                report.push_str("❌ 发现行数问题:\n");
                for issue in &result.integrity_issues {
                    report.push_str(&format!("- {}\n", issue));
                }
                report.push_str("\n");
            }
        }

        // 数据内容验证结果
        report.push_str("## 数据内容验证\n\n");
        for result in &data_results {
            report.push_str(&format!("### {}\n", result.table_name));
            if result.integrity_issues.is_empty() {
                report.push_str("✅ 数据内容完全一致\n\n");
            } else {
                report.push_str("❌ 发现数据不一致:\n");
                for issue in &result.integrity_issues {
                    report.push_str(&format!("- {}\n", issue));
                }
                report.push_str("\n");
            }
        }

        // 总体评估
        let total_issues = schema_results
            .iter()
            .map(|r| r.mismatched_fields.len())
            .chain(count_results.iter().map(|r| r.integrity_issues.len()))
            .chain(data_results.iter().map(|r| r.integrity_issues.len()))
            .sum::<usize>();

        report.push_str("## 总体评估\n\n");
        if total_issues == 0 {
            report.push_str("🎉 **迁移完美成功！** 所有数据验证均通过，无任何问题发现。\n");
        } else {
            report.push_str(&format!(
                "⚠️ **发现问题**: 共发现 {} 个问题，需要修复后才能完成迁移。\n",
                total_issues
            ));
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // 创建测试用的原始数据库
    async fn create_test_original_database(
        db_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let pool = SqlitePool::connect(db_path).await?;

        // 创建表结构
        sqlx::query(
            r#"
            CREATE TABLE claude_providers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                url TEXT NOT NULL,
                token TEXT NOT NULL,
                max_tokens INTEGER DEFAULT 4096,
                temperature REAL DEFAULT 0.7,
                model TEXT DEFAULT 'gpt-4',
                enabled INTEGER DEFAULT 1,
                description TEXT,
                timeout INTEGER DEFAULT 30,
                retry_count INTEGER DEFAULT 3,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&pool)
        .await?;

        // 插入测试数据
        sqlx::query(
            r#"
            INSERT INTO claude_providers (name, url, token, enabled, description) 
            VALUES ('Test Provider', 'https://api.test.com', 'encrypted_token_data', 1, 'Test description')
            "#
        ).execute(&pool).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_data_integrity_validation() {
        // 创建临时目录
        let temp_dir = tempdir().unwrap();
        let original_db_path = temp_dir.path().join("original.db");
        let migrated_db_path = temp_dir.path().join("migrated.db");

        // 创建原始数据库
        create_test_original_database(original_db_path.to_str().unwrap()).await.unwrap();

        // 创建空的迁移数据库（模拟迁移后的状态）
        let migrated_pool = SqlitePool::connect(migrated_db_path.to_str().unwrap()).await.unwrap();

        // 运行迁移过程（这里简化为直接复制表结构和数据）
        sqlx::query(
            r#"
            CREATE TABLE claude_providers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                url TEXT NOT NULL,
                token TEXT NOT NULL,
                max_tokens INTEGER DEFAULT 4096,
                temperature REAL DEFAULT 0.7,
                model TEXT DEFAULT 'gpt-4',
                enabled INTEGER DEFAULT 1,
                description TEXT,
                timeout INTEGER DEFAULT 30,
                retry_count INTEGER DEFAULT 3,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&migrated_pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            INSERT INTO claude_providers (name, url, token, enabled, description) 
            VALUES ('Test Provider', 'https://api.test.com', 'encrypted_token_data', 1, 'Test description')
            "#
        ).execute(&migrated_pool).await.unwrap();

        // 验证数据完整性
        let validator = DataIntegrityValidator::new(
            original_db_path.to_str().unwrap(),
            migrated_db_path.to_str().unwrap(),
        )
        .await
        .unwrap();

        // 验证表结构
        let schema_results = validator.verify_table_schemas().await.unwrap();
        assert_eq!(schema_results.len(), 5); // 5个主要表

        // 验证行数
        let count_results = validator.verify_row_counts().await.unwrap();
        assert_eq!(count_results.len(), 5);

        // 验证数据内容
        let data_results = validator.verify_unencrypted_data().await.unwrap();
        assert_eq!(data_results.len(), 5);

        // 检查Claude供应商表的验证结果
        let claude_result =
            data_results.iter().find(|r| r.table_name == "claude_providers").unwrap();
        assert_eq!(claude_result.original_count, 1);
        assert_eq!(claude_result.migrated_count, 1);
        assert!(claude_result.integrity_issues.is_empty());
    }
}
