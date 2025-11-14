#!/usr/bin/env python3
"""
创建测试数据脚本
为数据迁移工具创建源数据库和测试数据
"""

import sqlite3
import json
from cryptography.fernet import Fernet
import os

def create_test_database():
    """创建包含测试数据的源数据库"""

    # 创建数据库
    conn = sqlite3.connect('test_source.db')
    cursor = conn.cursor()

    # 创建表结构（与原Python版本一致）
    cursor.execute('''
        CREATE TABLE IF NOT EXISTS "claude_providers" (
            "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
            "name" TEXT NOT NULL UNIQUE,
            "url" TEXT NOT NULL,
            "token" TEXT NOT NULL,
            "timeout" INTEGER DEFAULT 30000,
            "auto_update" INTEGER DEFAULT 1,
            "type" TEXT NOT NULL DEFAULT 'public_welfare',
            "enabled" INTEGER NOT NULL DEFAULT 0,
            "opus_model" TEXT,
            "sonnet_model" TEXT,
            "haiku_model" TEXT,
            "created_at" TEXT DEFAULT CURRENT_TIMESTAMP,
            "updated_at" TEXT DEFAULT CURRENT_TIMESTAMP
        )
    ''')

    cursor.execute('''
        CREATE TABLE IF NOT EXISTS "codex_providers" (
            "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
            "name" TEXT NOT NULL UNIQUE,
            "url" TEXT NOT NULL,
            "token" TEXT NOT NULL,
            "type" TEXT NOT NULL DEFAULT 'public_welfare',
            "enabled" INTEGER NOT NULL DEFAULT 0,
            "created_at" TEXT DEFAULT CURRENT_TIMESTAMP,
            "updated_at" TEXT DEFAULT CURRENT_TIMESTAMP
        )
    ''')

    cursor.execute('''
        CREATE TABLE IF NOT EXISTS "agent_guides" (
            "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
            "name" TEXT NOT NULL UNIQUE,
            "type" TEXT NOT NULL,
            "text" TEXT NOT NULL,
            "created_at" TEXT DEFAULT CURRENT_TIMESTAMP,
            "updated_at" TEXT DEFAULT CURRENT_TIMESTAMP
        )
    ''')

    cursor.execute('''
        CREATE TABLE IF NOT EXISTS "mcp_servers" (
            "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
            "name" TEXT NOT NULL UNIQUE,
            "type" TEXT,
            "timeout" INTEGER DEFAULT 30000,
            "command" TEXT NOT NULL,
            "args" TEXT NOT NULL,
            "env" TEXT,
            "created_at" TEXT DEFAULT CURRENT_TIMESTAMP,
            "updated_at" TEXT DEFAULT CURRENT_TIMESTAMP
        )
    ''')

    cursor.execute('''
        CREATE TABLE IF NOT EXISTS "common_configs" (
            "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
            "key" TEXT NOT NULL UNIQUE,
            "value" TEXT NOT NULL,
            "description" TEXT,
            "category" TEXT NOT NULL DEFAULT 'general',
            "is_active" INTEGER NOT NULL DEFAULT 1,
            "created_at" TEXT DEFAULT CURRENT_TIMESTAMP,
            "updated_at" TEXT DEFAULT CURRENT_TIMESTAMP
        )
    ''')

    # 生成加密密钥
    key = Fernet.generate_key().decode()

    # 保存密钥到环境文件
    with open('.env', 'w') as f:
        f.write(f'FERNET_KEY={key}\n')
        f.write(f'OLD_FERNET_KEY={key}\n')

    print(f"🔑 生成加密密钥: {key}")

    # 创建加密器
    fernet = Fernet(key)

    # 插入测试数据
    print("📝 插入测试数据...")

    # Claude供应商
    claude_data = [
        ("Test Claude Provider", "https://api.anthropic.com", "sk-ant-api03-test-key-123"),
        ("Anthropic Official", "https://api.anthropic.com", "sk-ant-api03-official-456"),
    ]

    for name, url, token in claude_data:
        encrypted_token = fernet.encrypt(token.encode()).decode()
        cursor.execute('''
            INSERT INTO claude_providers (name, url, token, type, enabled)
            VALUES (?, ?, ?, ?, ?)
        ''', (name, url, encrypted_token, 'public_welfare', 1))

    # Codex供应商
    codex_data = [
        ("Test Codex Provider", "https://api.openai.com", "sk-test-codex-789"),
        ("OpenAI Official", "https://api.openai.com", "sk-official-codex-101"),
    ]

    for name, url, token in codex_data:
        encrypted_token = fernet.encrypt(token.encode()).decode()
        cursor.execute('''
            INSERT INTO codex_providers (name, url, token, type, enabled)
            VALUES (?, ?, ?, ?, ?)
        ''', (name, url, encrypted_token, 'public_welfare', 1))

    # Agent指导文件
    agent_guides = [
        ("Web开发助手", "and", "这是一个专门用于Web开发的AI助手..."),
        ("数据分析专家", "only", "专注于数据分析和可视化..."),
        ("代码审查工具", "and", "帮助进行代码质量和安全审查..."),
    ]

    for name, guide_type, text in agent_guides:
        cursor.execute('''
            INSERT INTO agent_guides (name, type, text)
            VALUES (?, ?, ?)
        ''', (name, guide_type, text))

    # MCP服务器
    mcp_servers = [
        ("file-server", "stdio", 30000, "python3", '["/path/to/file_server.py"]', '{"PORT": "8080"}'),
        ("database-proxy", "stdio", 15000, "node", '["db-proxy.js", "--port", "5432"]', None),
    ]

    for name, server_type, timeout, command, args, env in mcp_servers:
        cursor.execute('''
            INSERT INTO mcp_servers (name, type, timeout, command, args, env)
            VALUES (?, ?, ?, ?, ?, ?)
        ''', (name, server_type, timeout, command, args, env))

    # 通用配置
    common_configs = [
        ("app_name", "AI Manager", "应用程序名称", "general"),
        ("version", "2.0.0", "当前版本号", "system"),
        ("max_tokens", "4096", "默认最大token数", "api"),
        ("theme", "light", "界面主题", "ui"),
    ]

    for key, value, description, category in common_configs:
        cursor.execute('''
            INSERT INTO common_configs (key, value, description, category)
            VALUES (?, ?, ?, ?)
        ''', (key, value, description, category))

    conn.commit()

    # 显示统计信息
    print("\n📊 数据库统计:")
    cursor.execute("SELECT COUNT(*) FROM claude_providers")
    print(f"  Claude供应商: {cursor.fetchone()[0]} 条")

    cursor.execute("SELECT COUNT(*) FROM codex_providers")
    print(f"  Codex供应商: {cursor.fetchone()[0]} 条")

    cursor.execute("SELECT COUNT(*) FROM agent_guides")
    print(f"  Agent指导文件: {cursor.fetchone()[0]} 条")

    cursor.execute("SELECT COUNT(*) FROM mcp_servers")
    print(f"  MCP服务器: {cursor.fetchone()[0]} 条")

    cursor.execute("SELECT COUNT(*) FROM common_configs")
    print(f"  通用配置: {cursor.fetchone()[0]} 条")

    conn.close()

    print(f"\n✅ 测试数据库创建完成: test_source.db")
    print(f"📝 加密密钥已保存到 .env 文件")

    return key

if __name__ == "__main__":
    create_test_database()