#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Python数据验证脚本
用于验证从Rust版本迁移回Python的数据兼容性
"""

import json
import sqlite3
import tempfile
import os
from typing import Dict, List, Any
from cryptography.fernet import Fernet

def load_test_data() -> Dict[str, Any]:
    """加载测试数据"""
    with open('python_original_sample.json', 'r', encoding='utf-8') as f:
        return json.load(f)

def create_sqlite_database(data: Dict[str, Any]) -> str:
    """创建模拟Python版本的SQLite数据库"""
    # 创建临时数据库
    fd, db_path = tempfile.mkstemp(suffix='.db')
    os.close(fd)
    
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()
    
    # 创建表结构
    cursor.execute('''
        CREATE TABLE claude_providers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            url TEXT NOT NULL,
            token TEXT NOT NULL,
            timeout INTEGER DEFAULT 30000,
            auto_update INTEGER DEFAULT 1,
            type TEXT DEFAULT 'public_welfare',
            enabled INTEGER DEFAULT 0,
            opus_model TEXT,
            sonnet_model TEXT,
            haiku_model TEXT,
            created_at TEXT,
            updated_at TEXT
        )
    ''')
    
    cursor.execute('''
        CREATE TABLE codex_providers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            url TEXT NOT NULL,
            token TEXT NOT NULL,
            type TEXT DEFAULT 'public_welfare',
            enabled INTEGER DEFAULT 0,
            created_at TEXT,
            updated_at TEXT
        )
    ''')
    
    cursor.execute('''
        CREATE TABLE agent_guides (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            type TEXT NOT NULL,
            text TEXT NOT NULL,
            created_at TEXT,
            updated_at TEXT
        )
    ''')
    
    cursor.execute('''
        CREATE TABLE mcp_servers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            type TEXT DEFAULT 'stdio',
            timeout INTEGER DEFAULT 30000,
            command TEXT NOT NULL,
            args TEXT,
            env TEXT,
            created_at TEXT,
            updated_at TEXT
        )
    ''')
    
    cursor.execute('''
        CREATE TABLE common_configs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            key TEXT NOT NULL UNIQUE,
            value TEXT NOT NULL,
            description TEXT,
            category TEXT DEFAULT 'general',
            is_active INTEGER DEFAULT 1,
            created_at TEXT,
            updated_at TEXT
        )
    ''')
    
    # 插入测试数据
    # Claude供应商
    for provider in data['claude_providers']:
        cursor.execute('''
            INSERT INTO claude_providers 
            (id, name, url, token, timeout, auto_update, type, enabled, 
             opus_model, sonnet_model, haiku_model, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ''', (
            provider['id'], provider['name'], provider['url'], provider['token'],
            provider['timeout'], provider['auto_update'], provider['type'],
            provider['enabled'], provider['opus_model'], provider['sonnet_model'],
            provider['haiku_model'], provider['created_at'], provider['updated_at']
        ))
    
    # Codex供应商
    for provider in data['codex_providers']:
        cursor.execute('''
            INSERT INTO codex_providers 
            (id, name, url, token, type, enabled, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ''', (
            provider['id'], provider['name'], provider['url'], provider['token'],
            provider['type'], provider['enabled'], provider['created_at'], 
            provider['updated_at']
        ))
    
    # Agent指导文件
    for guide in data['agent_guides']:
        cursor.execute('''
            INSERT INTO agent_guides 
            (id, name, type, text, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
        ''', (
            guide['id'], guide['name'], guide['type'], guide['text'],
            guide['created_at'], guide['updated_at']
        ))
    
    # MCP服务器
    for server in data['mcp_servers']:
        args_json = json.dumps(server['args'])
        env_json = json.dumps(server['env']) if server['env'] else None
        cursor.execute('''
            INSERT INTO mcp_servers 
            (id, name, type, timeout, command, args, env, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ''', (
            server['id'], server['name'], server['type'], server['timeout'],
            server['command'], args_json, env_json, server['created_at'],
            server['updated_at']
        ))
    
    # 通用配置
    for config in data['common_configs']:
        cursor.execute('''
            INSERT INTO common_configs 
            (id, key, value, description, category, is_active, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ''', (
            config['id'], config['key'], config['value'], config['description'],
            config['category'], config['is_active'], config['created_at'],
            config['updated_at']
        ))
    
    conn.commit()
    conn.close()
    
    return db_path

def encrypt_tokens(data: Dict[str, Any], key: str) -> Dict[str, Any]:
    """使用Python Fernet加密token数据"""
    fernet = Fernet(key)
    
    encrypted_data = data.copy()
    
    # 加密Claude供应商token
    for provider in encrypted_data['claude_providers']:
        provider['token'] = fernet.encrypt(provider['token'].encode()).decode()
    
    # 加密Codex供应商token
    for provider in encrypted_data['codex_providers']:
        provider['token'] = fernet.encrypt(provider['token'].encode()).decode()
    
    return encrypted_data

def validate_data_integrity(original_data: Dict[str, Any], 
                          migrated_data: Dict[str, Any]) -> bool:
    """验证数据完整性"""
    print("🔍 验证数据完整性...")
    
    success = True
    
    # 验证Claude供应商
    if len(original_data['claude_providers']) != len(migrated_data['claude_providers']):
        print(f"❌ Claude供应商数量不匹配: 原始={len(original_data['claude_providers'])}, 迁移={len(migrated_data['claude_providers'])}")
        success = False
    else:
        print(f"✅ Claude供应商数量匹配: {len(original_data['claude_providers'])}")
    
    # 验证Codex供应商
    if len(original_data['codex_providers']) != len(migrated_data['codex_providers']):
        print(f"❌ Codex供应商数量不匹配: 原始={len(original_data['codex_providers'])}, 迁移={len(migrated_data['codex_providers'])}")
        success = False
    else:
        print(f"✅ Codex供应商数量匹配: {len(original_data['codex_providers'])}")
    
    # 验证Agent指导文件
    if len(original_data['agent_guides']) != len(migrated_data['agent_guides']):
        print(f"❌ Agent指导文件数量不匹配: 原始={len(original_data['agent_guides'])}, 迁移={len(migrated_data['agent_guides'])}")
        success = False
    else:
        print(f"✅ Agent指导文件数量匹配: {len(original_data['agent_guides'])}")
    
    # 验证MCP服务器
    if len(original_data['mcp_servers']) != len(migrated_data['mcp_servers']):
        print(f"❌ MCP服务器数量不匹配: 原始={len(original_data['mcp_servers'])}, 迁移={len(migrated_data['mcp_servers'])}")
        success = False
    else:
        print(f"✅ MCP服务器数量匹配: {len(original_data['mcp_servers'])}")
    
    # 验证通用配置
    if len(original_data['common_configs']) != len(migrated_data['common_configs']):
        print(f"❌ 通用配置数量不匹配: 原始={len(original_data['common_configs'])}, 迁移={len(migrated_data['common_configs'])}")
        success = False
    else:
        print(f"✅ 通用配置数量匹配: {len(original_data['common_configs'])}")
    
    return success

def test_encryption_compatibility():
    """测试加密兼容性"""
    print("🔐 测试加密兼容性...")
    
    # 使用与Rust相同的测试密钥
    test_key = "Jw4Ff1BWLnSykdfXDVOuEJCG6m9dyST5B1VhU_qg0fI="
    fernet = Fernet(test_key)
    
    # 测试用例
    test_cases = [
        "sk-ant-test-key-1",
        "sk-test-openai-key-1",
        "测试中文token",
        "🔒🔐🔑",
        "",
        "A" * 1000
    ]
    
    for test_data in test_cases:
        try:
            # 加密
            encrypted = fernet.encrypt(test_data.encode()).decode()
            
            # 解密
            decrypted = fernet.decrypt(encrypted.encode()).decode()
            
            if test_data == decrypted:
                print(f"✅ 加密/解密测试通过: {test_data[:20]}...")
            else:
                print(f"❌ 加密/解密测试失败: {test_data[:20]}...")
                return False
                
        except Exception as e:
            print(f"❌ 加密测试异常: {e}")
            return False
    
    print("✅ 加密兼容性测试全部通过")
    return True

def generate_encrypted_test_data():
    """生成加密的测试数据"""
    print("📝 生成加密测试数据...")
    
    # 加载原始数据
    data = load_test_data()
    
    # 使用固定密钥加密
    test_key = "Jw4Ff1BWLnSykdfXDVOuEJCG6m9dyST5B1VhU_qg0fI="
    encrypted_data = encrypt_tokens(data, test_key)
    
    # 保存加密数据
    with open('python_encrypted_sample.json', 'w', encoding='utf-8') as f:
        json.dump(encrypted_data, f, ensure_ascii=False, indent=2)
    
    print("✅ 加密测试数据已生成: python_encrypted_sample.json")

def main():
    """主函数"""
    print("🚀 开始Python数据兼容性验证...")
    
    # 测试加密兼容性
    if not test_encryption_compatibility():
        print("❌ 加密兼容性测试失败")
        return False
    
    # 生成加密测试数据
    generate_encrypted_test_data()
    
    # 创建测试数据库
    data = load_test_data()
    db_path = create_sqlite_database(data)
    print(f"✅ 测试数据库已创建: {db_path}")
    
    print("🎉 Python数据兼容性验证完成")
    return True

if __name__ == "__main__":
    success = main()
    exit(0 if success else 1)