-- 初始化数据库架构
-- 创建5个核心表：claude_providers, codex_providers, agent_guides, mcp_servers, common_configs

-- Claude供应商表
CREATE TABLE "claude_providers" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "name" TEXT NOT NULL UNIQUE,
    "url" TEXT NOT NULL,
    "token" TEXT NOT NULL,  -- 加密存储
    "timeout" INTEGER DEFAULT 30000,
    "auto_update" INTEGER DEFAULT 1,  -- 1-禁用遥测，0-启用遥测
    "type" TEXT NOT NULL DEFAULT 'public_welfare',  -- paid 或 public_welfare
    "enabled" INTEGER NOT NULL DEFAULT 0,  -- 0-未启用，1-启用
    "opus_model" TEXT,  -- 可选模型名称
    "sonnet_model" TEXT,  -- 可选模型名称
    "haiku_model" TEXT,  -- 可选模型名称
    "created_at" TEXT DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Codex供应商表
CREATE TABLE "codex_providers" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "name" TEXT NOT NULL UNIQUE,
    "url" TEXT NOT NULL,
    "token" TEXT NOT NULL,  -- 加密存储
    "type" TEXT NOT NULL DEFAULT 'public_welfare',  -- paid 或 public_welfare
    "enabled" INTEGER NOT NULL DEFAULT 0,  -- 0-未启用，1-启用
    "created_at" TEXT DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Agent指导文件表
CREATE TABLE "agent_guides" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "name" TEXT NOT NULL UNIQUE,
    "type" TEXT NOT NULL,  -- 'only' 或 'and'
    "text" TEXT NOT NULL,  -- 文件完整内容
    "created_at" TEXT DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TEXT DEFAULT CURRENT_TIMESTAMP
);

-- MCP服务器配置表
CREATE TABLE "mcp_servers" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "name" TEXT NOT NULL UNIQUE,
    "type" TEXT,  -- stdio, sse等
    "timeout" INTEGER DEFAULT 30000,  -- 默认30000ms
    "command" TEXT NOT NULL,  -- 命令，如npx, uvx, python等
    "args" TEXT NOT NULL,  -- 命令参数，存储为JSON字符串
    "env" TEXT,  -- 环境变量，存储为JSON字符串
    "created_at" TEXT DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TEXT DEFAULT CURRENT_TIMESTAMP
);

-- 通用配置表
CREATE TABLE "common_configs" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "key" TEXT NOT NULL UNIQUE,
    "value" TEXT NOT NULL,  -- 支持环境变量替换，如 ${HOME}
    "description" TEXT,
    "category" TEXT NOT NULL DEFAULT 'general',  -- 配置分类
    "is_active" INTEGER NOT NULL DEFAULT 1,  -- 是否启用：1-启用，0-禁用
    "created_at" TEXT DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TEXT DEFAULT CURRENT_TIMESTAMP
);

-- 创建触发器，自动更新 updated_at 字段
CREATE TRIGGER "update_claude_providers_updated_at"
    AFTER UPDATE ON "claude_providers"
    FOR EACH ROW
BEGIN
    UPDATE "claude_providers"
    SET "updated_at" = CURRENT_TIMESTAMP
    WHERE "id" = NEW."id";
END;

CREATE TRIGGER "update_codex_providers_updated_at"
    AFTER UPDATE ON "codex_providers"
    FOR EACH ROW
BEGIN
    UPDATE "codex_providers"
    SET "updated_at" = CURRENT_TIMESTAMP
    WHERE "id" = NEW."id";
END;

CREATE TRIGGER "update_agent_guides_updated_at"
    AFTER UPDATE ON "agent_guides"
    FOR EACH ROW
BEGIN
    UPDATE "agent_guides"
    SET "updated_at" = CURRENT_TIMESTAMP
    WHERE "id" = NEW."id";
END;

CREATE TRIGGER "update_mcp_servers_updated_at"
    AFTER UPDATE ON "mcp_servers"
    FOR EACH ROW
BEGIN
    UPDATE "mcp_servers"
    SET "updated_at" = CURRENT_TIMESTAMP
    WHERE "id" = NEW."id";
END;

CREATE TRIGGER "update_common_configs_updated_at"
    AFTER UPDATE ON "common_configs"
    FOR EACH ROW
BEGIN
    UPDATE "common_configs"
    SET "updated_at" = CURRENT_TIMESTAMP
    WHERE "id" = NEW."id";
END;
