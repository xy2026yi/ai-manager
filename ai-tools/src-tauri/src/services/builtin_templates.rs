use crate::models::mcp_template::CreateMcpTemplateRequest;
use anyhow::Result;

pub struct BuiltinTemplates;

impl BuiltinTemplates {
    /// 获取所有内置MCP模板
    pub fn get_all_templates() -> Vec<CreateMcpTemplateRequest> {
        vec![
            // Claude模板 - Unix平台
            CreateMcpTemplateRequest {
                name: "Context7".to_string(),
                version: Some("1.0.0".to_string()),
                ai_type: "claude".to_string(),
                platform_type: "unix".to_string(),
                config_content: r#"{
  "context7": {
    "type": "stdio",
    "command": "npx",
    "args": [
      "-y",
      "@upstash/context7-mcp"
    ],
    "env": {}
  }
}"#.to_string(),
                description: Some("Context7 - 编程库文档聚合服务".to_string()),
                category: Some("documentation".to_string()),
                tags: Some(vec!["docs".to_string(), "search".to_string()]),
            },
            CreateMcpTemplateRequest {
                name: "Sequential Thinking".to_string(),
                version: Some("1.0.0".to_string()),
                ai_type: "claude".to_string(),
                platform_type: "unix".to_string(),
                config_content: r#"{
  "sequential-thinking": {
    "type": "stdio",
    "command": "npx",
    "args": [
      "-y",
      "@modelcontextprotocol/server-sequential-thinking"
    ],
    "env": {}
  }
}"#.to_string(),
                description: Some("Sequential Thinking - 结构化思维工具".to_string()),
                category: Some("tools".to_string()),
                tags: Some(vec!["thinking".to_string(), "analysis".to_string()]),
            },
            CreateMcpTemplateRequest {
                name: "Memory".to_string(),
                version: Some("1.0.0".to_string()),
                ai_type: "claude".to_string(),
                platform_type: "unix".to_string(),
                config_content: r#"{
  "memory": {
    "type": "stdio",
    "command": "npx",
    "args": [
      "-y",
      "@modelcontextprotocol/server-memory"
    ],
    "env": {}
  }
}"#.to_string(),
                description: Some("Memory - 记忆管理服务".to_string()),
                category: Some("tools".to_string()),
                tags: Some(vec!["memory".to_string(), "storage".to_string()]),
            },
            CreateMTemplateRequest {
                name: "Playwright".to_string(),
                version: Some("1.0.0".to_string()),
                ai_type: "claude".to_string(),
                platform_type: "unix".to_string(),
                config_content: r#"{
  "playwright": {
    "type": "stdio",
    "command": "npx",
    "args": [
      "-y",
      "@executeautomation/playwright-mcp-server"
    ],
    "env": {}
  }
}"#.to_string(),
                description: Some("Playwright - Web自动化测试工具".to_string()),
                category: Some("testing".to_string()),
                tags: Some(vec!["testing".to_string(), "automation".to_string()]),
            },
            CreateMcpTemplateRequest {
                name: "Shrimp Task Manager".to_string(),
                version: Some("1.0.0".to_string()),
                ai_type: "claude".to_string(),
                platform_type: "unix".to_string(),
                config_content: r#"{
  "shrimp-task-manager": {
    "command": "npx",
    "args": [
      "-y",
      "mcp-shrimp-task-manager"
    ],
    "env": {
      "DATA_DIR": ".shrimp",
      "TEMPLATES_USE": "zh",
      "ENABLE_GUI": "false"
    }
  }
}"#.to_string(),
                description: Some("Shrimp Task Manager - 任务管理工具".to_string()),
                category: Some("productivity".to_string()),
                tags: Some(vec!["task".to_string(), "management".to_string()]),
            },
            CreateMcpTemplateRequest {
                name: "Desktop Commander".to_string(),
                version: Some("1.0.0".to_string()),
                ai_type: "claude".to_string(),
                platform_type: "unix".to_string(),
                config_content: r#"{
  "desktop-commander": {
    "type": "stdio",
    "command": "npx",
    "args": [
      "-y",
      "@wonderwhy-er/desktop-commander"
    ],
    "env": {}
  }
}"#.to_string(),
                description: Some("Desktop Commander - 桌面文件和进程管理".to_string()),
                category: Some("productivity".to_string()),
                tags: Some(vec!["desktop".to_string(), "files".to_string()]),
            },

            // Claude模板 - Windows平台
            CreateMcpTemplateRequest {
                name: "Context7".to_string(),
                version: Some("1.0.0".to_string()),
                ai_type: "claude".to_string(),
                platform_type: "windows".to_string(),
                config_content: r#"{
  "context7": {
    "type": "stdio",
    "command": "/usr/bin/context7-mcp",
    "args": [],
    "env": {}
  }
}"#.to_string(),
                description: Some("Context7 - 编程库文档聚合服务 (Windows)".to_string()),
                category: Some("documentation".to_string()),
                tags: Some(vec!["docs".to_string(), "search".to_string()]),
            },
            CreateMcpTemplateRequest {
                name: "Sequential Thinking".to_string(),
                version: Some("1.0.0".to_string()),
                ai_type: "claude".to_string(),
                platform_type: "windows".to_string(),
                config_content: r#"{
  "sequential-thinking": {
    "type": "stdio",
    "command": "/usr/bin/sequential-thinking",
    "args": [],
    "env": {}
  }
}"#.to_string(),
                description: Some("Sequential Thinking - 结构化思维工具 (Windows)".to_string()),
                category: Some("tools".to_string()),
                tags: Some(vec!["thinking".to_string(), "analysis".to_string()]),
            },
            CreateMcpTemplateRequest {
                name: "Memory".to_string(),
                version: Some("1.0.0".to_string()),
                ai_type: "claude".to_string(),
                platform_type: "windows".to_string(),
                config_content: r#"{
  "memory": {
    "type": "stdio",
    "command": "/usr/bin/memory",
    "args": [],
    "env": {}
  }
}"#.to_string(),
                description: Some("Memory - 记忆管理服务 (Windows)".to_string()),
                category: Some("tools".to_string()),
                tags: Some(vec!["memory".to_string(), "storage".to_string()]),
            },

            // Codex模板 - Unix平台
            CreateMcpTemplateRequest {
                name: "Context7".to_string(),
                version: Some("1.0.0".to_string()),
                ai_type: "codex".to_string(),
                platform_type: "unix".to_string(),
                config_content: r#"[mcp_servers.context7]
type = "stdio"
command = "npx"
args = [ "-y", "@upstash/context7-mcp" ]
env = {}"#.to_string(),
                description: Some("Context7 - 编程库文档聚合服务".to_string()),
                category: Some("documentation".to_string()),
                tags: Some(vec!["docs".to_string(), "search".to_string()]),
            },
            CreateMcpTemplateRequest {
                name: "Sequential Thinking".to_string(),
                version: Some("1.0.0".to_string()),
                ai_type: "codex".to_string(),
                platform_type: "unix".to_string(),
                config_content: r#"[mcp_servers.sequential-thinking]
type = "stdio"
command = "npx"
args = [ "-y", "@modelcontextprotocol/server-sequential-thinking" ]"#.to_string(),
                description: Some("Sequential Thinking - 结构化思维工具".to_string()),
                category: Some("tools".to_string()),
                tags: Some(vec!["thinking".to_string(), "analysis".to_string()]),
            },
            CreateMcpTemplateRequest {
                name: "Chrome DevTools".to_string(),
                version: Some("1.0.0".to_string()),
                ai_type: "codex".to_string(),
                platform_type: "unix".to_string(),
                config_content: r#"[mcp_servers.chrome-devtools]
type = "stdio"
command = "npx"
args = [ "chrome-devtools-mcp@latest" ]"#.to_string(),
                description: Some("Chrome DevTools - Chrome开发者工具".to_string()),
                category: Some("development".to_string()),
                tags: Some(vec!["chrome".to_string(), "debugging".to_string()]),
            },
            CreateMcpTemplateRequest {
                name: "Exa".to_string(),
                version: Some("1.0.0".to_string()),
                ai_type: "codex".to_string(),
                platform_type: "unix".to_string(),
                config_content: r#"[mcp_servers.exa]
type = "stdio"
command = "npx"
args = [ "-y", "@smithery/cli@latest", "run", "exa", "--key", "1778bbf9-08f1-4d2e-9999-4cb4ba664dd9" ]
env = {}"#.to_string(),
                description: Some("Exa - 搜索和网络信息获取".to_string()),
                category: Some("search".to_string()),
                tags: Some(vec!["search".to_string(), "web".to_string()]),
            },
            CreateMcpTemplateRequest {
                name: "MCP DeepWiki".to_string(),
                version: Some("1.0.0".to_string()),
                ai_type: "codex".to_string(),
                platform_type: "unix".to_string(),
                config_content: r#"[mcp_servers.mcp-deepwiki]
startup_timeout_ms = 20000
type = "stdio"
command = "npx"
args = [ "-y", "mcp-deepwiki@latest" ]"#.to_string(),
                description: Some("MCP DeepWiki - 深度知识聚合".to_string()),
                category: Some("knowledge".to_string()),
                tags: Some(vec!["wiki".to_string(), "research".to_string()]),
            },
            CreateMcpTemplateRequest {
                name: "Spec Workflow".to_string(),
                version: Some("1.0.0".to_string()),
                ai_type: "codex".to_string(),
                platform_type: "unix".to_string(),
                config_content: r#"[mcp_servers.spec-workflow]
startup_timeout_ms = 20000
type = "stdio"
command = "npx"
args = [ "-y", "@pimzino/spec-workflow-mcp@latest" ]"#.to_string(),
                description: Some("Spec Workflow - 规范工作流工具".to_string()),
                category: Some("workflow".to_string()),
                tags: Some(vec!["workflow".to_string(), "spec".to_string()]),
            },
            CreateMcpRequest {
                name: "Serena".to_string(),
                version: Some("1.0.0".to_string()),
                ai_type: "codex".to_string(),
                platform_type: "unix".to_string(),
                config_content: r#"[mcp_servers.serena]
startup_timeout_ms = 20000
args = [
    "--from",
    "git+https://github.com/oraios/serena",
    "serena",
    "start-mcp-server",
    "--context",
    "codex",
    "--enable-web-dashboard",
    "False",
]
command = "uvx"
type = "stdio""#.to_string(),
                description: Some("Serena - 代码语义检索工具".to_string()),
                category: Some("development".to_string()),
                tags: Some(vec!["code".to_string(), "search".to_string()]),
            },

            // Codex模板 - Windows平台
            CreateMcpTemplateRequest {
                name: "Context7".to_string(),
                version: Some("1.0.0".to_string()),
                ai_type: "codex".to_string(),
                platform_type: "windows".to_string(),
                config_content: r#"[mcp_servers.context7]
type = "stdio"
command = "/usr/bin/context7-mcp"
args = []"#.to_string(),
                description: Some("Context7 - 编程库文档聚合服务 (Windows)".to_string()),
                category: Some("documentation".to_string()),
                tags: Some(vec!["docs".to_string(), "search".to_string()]),
            },
            CreateMcpTemplateRequest {
                name: "Sequential Thinking".to_string(),
                version: Some("1.0.0".to_string()),
                ai_type: "codex".to_string(),
                platform_type: "windows".to_string(),
                config_content: r#"[mcp_servers.sequential-thinking]
type = "stdio"
command = "/usr/bin/sequential-thinking"
args = []"#.to_string(),
                description: Some("Sequential Thinking - 结构化思维工具 (Windows)".to_string()),
                category: Some("tools".to_string()),
                tags: Some(vec!["thinking".to_string(), "analysis".to_string()]),
            },
            CreateMcpTemplateRequest {
                name: "Chrome DevTools".to_string(),
                version: Some("1.0.0".to_string()),
                ai_type: "codex".to_string(),
                platform_type: "windows".to_string(),
                config_content: r#"[mcp_servers.chrome-devtools]
type = "stdio"
command = "/usr/bin/npx"
args = [ "chrome-devtools-mcp@latest" ]"#.to_string(),
                description: Some("Chrome DevTools - Chrome开发者工具 (Windows)".to_string()),
                category: Some("development".to_string()),
                tags: Some(vec!["chrome".to_string(), "debugging".to_string()]),
            },
            CreateMcpTemplateRequest {
                name: "Exa".to_string(),
                version: Some("1.0.0".to_string()),
                ai_type: "codex".to_string(),
                platform_type: "windows".to_string(),
                config_content: r#"[mcp_servers.exa]
type = "stdio"
command = "/usr/bin/npx"
args = [ "-y", "@smithery/cli@latest", "run", "exa", "--key", "1778bbf9-08f1-4d2e-9999-4cb4ba664dd9" ]
"#.to_string(),
                description: Some("Exa - 搜索和网络信息获取 (Windows)".to_string()),
                category: Some("search".to_string()),
                tags: Some(vec!["search".to_string(), "web".to_string()]),
            },
        ]
    }

    /// 初始化内置模板到数据库
    pub async fn initialize_builtin_templates(pool: &sqlx::SqlitePool) -> Result<()> {
        let templates = Self::get_all_templates();

        for template in templates {
            // 检查是否已存在
            let existing = sqlx::query_scalar(
                "SELECT COUNT(*) FROM mcp_templates WHERE name = ? AND version = ?"
            )
            .bind(&template.name)
            .bind(template.version.as_ref().unwrap_or(&"1.0.0".to_string()))
            .fetch_one(pool)
            .await
            .unwrap_or(0);

            if existing == 0 {
                // 标记为内置模板
                let request = CreateMcpTemplateRequest {
                    name: template.name,
                    version: template.version,
                    ai_type: template.ai_type,
                    platform_type: template.platform_type,
                    config_content: template.config_content,
                    description: template.description,
                    category: template.category,
                    tags: template.tags,
                };

                // 创建模板
                let created = crate::models::mcp_template::McpTemplate::create(pool, request)
                    .await
                    .map_err(|e| anyhow::anyhow!("创建内置模板 '{}' 失败: {}", template.name, e))?;

                if let Some(created) = {
                    log::info!("创建内置模板: {} (ID: {})", created.name, created.id.unwrap_or(0));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;
    use tempfile::tempdir;

    async fn create_test_pool() -> SqlitePool {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db_url = format!("sqlite:{}", db_path.display());

        crate::database::Database::new(&db_url).await.unwrap();
        SqlitePool::connect(&db_url).await.unwrap()
    }

    #[tokio::test]
    async fn test_builtin_templates() {
        let templates = BuiltinTemplates::get_all_templates();
        assert!(!templates.is_empty());

        // 验证模板配置格式
        for template in &templates {
            if template.ai_type == "claude" {
                assert!(serde_json::from_str::<serde_json::Value>(&template.config_content).is_ok());
            } else if template.ai_type == "codex" {
                assert!(toml::from_str::<toml::Value>(&template.config_content).is_ok());
            }
        }
    }

    #[tokio::test]
    async fn test_initialize_builtin_templates() {
        let pool = create_test_pool().await;

        BuiltinTemplates::initialize_builtin_templates(&pool).await.unwrap();

        // 验证模板已创建
        for template in BuiltinTemplates::get_all_templates() {
            let count = sqlx::query_scalar("SELECT COUNT(*) FROM mcp_templates WHERE name = ? AND version = ?")
                .bind(&template.name)
                .bind(template.version.as_ref().unwrap_or(&"1.0.0".to_string()))
                .fetch_one(&pool)
                .await
                .unwrap_or(0);

            assert!(count > 0, "模板 {} 应该已创建", template.name);
        }
    }
}