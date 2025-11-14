-- 添加性能优化索引
-- 为提高查询性能，在关键字段上创建索引

-- Claude供应商表索引
CREATE INDEX "idx_claude_providers_enabled" ON "claude_providers"("enabled");
CREATE INDEX "idx_claude_providers_type" ON "claude_providers"("type");
CREATE INDEX "idx_claude_providers_name" ON "claude_providers"("name");

-- Codex供应商表索引
CREATE INDEX "idx_codex_providers_enabled" ON "codex_providers"("enabled");
CREATE INDEX "idx_codex_providers_type" ON "codex_providers"("type");
CREATE INDEX "idx_codex_providers_name" ON "codex_providers"("name");

-- Agent指导文件表索引
CREATE INDEX "idx_agent_guides_type" ON "agent_guides"("type");
CREATE INDEX "idx_agent_guides_name" ON "agent_guides"("name");

-- MCP服务器表索引
CREATE INDEX "idx_mcp_servers_name" ON "mcp_servers"("name");
CREATE INDEX "idx_mcp_servers_type" ON "mcp_servers"("type");

-- 通用配置表索引
CREATE INDEX "idx_common_configs_key" ON "common_configs"("key");
CREATE INDEX "idx_common_configs_category" ON "common_configs"("category");
CREATE INDEX "idx_common_configs_is_active" ON "common_configs"("is_active");
