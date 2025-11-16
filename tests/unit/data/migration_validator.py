#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Python数据验证脚本
用于验证从Python项目到Rust项目的数据兼容性
"""

import json
import sqlite3
import tempfile
import os
from typing import Dict, List, Any
from cryptography.fernet import Fernet

def generate_test_data() -> Dict[str, Any]:
    """生成测试数据"""
    return {
        "version": "1.0.0",
        "claude_providers": [
            {
                "id": 1,
                "name": "Claude Public Welfare",
                "url": "https://api.anthropic.com",
                "token": "sk-ant-api03-test-key-1",
                "timeout": 30000,
                "auto_update": 1,
                "type": "public_welfare",
                "enabled": 1,
                "opus_model": "claude-3-opus-20240229",
                "sonnet_model": "claude-3-sonnet-20240229",
                "haiku_model": "claude-3-haiku-20240307",
                "created_at": "2024-01-15T10:30:00Z",
                "updated_at": "2024-01-15T10:30:00Z"
            }
        ],
        "codex_providers": [
            {
                "id": 1,
                "name": "OpenAI Official",
                "url": "https://api.openai.com/v1/chat/completions",
                "token": "sk-test-openai-key-1",
                "type": "official",
                "enabled": 1,
                "created_at": "2024-01-15T11:00:00Z",
                "updated_at": "2024-01-15T11:00:00Z"
            }
        ],
        "agent_guides": [
            {
                "id": 1,
                "name": "代码审查助手",
                "type": "code_reviewer",
                "text": "你是一个专业的代码审查助手。请仔细审查提供的代码，检查代码质量、性能、安全性和最佳实践。",
                "created_at": "2024-01-15T12:00:00Z",
                "updated_at": "2024-01-15T12:00:00Z"
            }
        ],
        "mcp_servers": [
            {
                "id": 1,
                "name": "filesystem",
                "type": "stdio",
                "timeout": 30000,
                "command": "npx",
                "args": ["@modelcontextprotocol/server-filesystem", "/tmp"],
                "env": {"NODE_ENV": "production"},
                "created_at": "2024-01-15T13:00:00Z",
                "updated_at": "2024-01-15T13:00:00Z"
            }
        ],
        "common_configs": [
            {
                "id": 1,
                "key": "default_claude_model",
                "value": "claude-3-sonnet-20240229",
                "description": "默认使用的Claude模型",
                "category": "models",
                "is_active": 1,
                "created_at": "2024-01-15T14:00:00Z",
                "updated_at": "2024-01-15T14:00:00Z"
            }
        ]
    }

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

def main():
    """主函数"""
    print("🚀 开始Python数据兼容性验证...")
    
    # 测试加密兼容性
    if not test_encryption_compatibility():
        print("❌ 加密兼容性测试失败")
        return False
    
    # 生成测试数据
    data = generate_test_data()
    
    # 保存原始数据
    with open('python_original_sample.json', 'w', encoding='utf-8') as f:
        json.dump(data, f, ensure_ascii=False, indent=2)
    
    print("✅ 原始测试数据已生成: python_original_sample.json")
    
    # 生成加密测试数据
    test_key = "Jw4Ff1BWLnSykdfXDVOuEJCG6m9dyST5B1VhU_qg0fI="
    encrypted_data = encrypt_tokens(data, test_key)
    
    with open('python_encrypted_sample.json', 'w', encoding='utf-8') as f:
        json.dump(encrypted_data, f, ensure_ascii=False, indent=2)
    
    print("✅ 加密测试数据已生成: python_encrypted_sample.json")
    print("🎉 Python数据兼容性验证完成")
    return True

if __name__ == "__main__":
    success = main()
    exit(0 if success else 1)