#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Python数据验证脚本
用于验证原Python项目数据库的数据完整性和格式
"""

import sqlite3
import json
import sys
from datetime import datetime
from typing import Dict, List, Any, Optional
import os

class PythonDataValidator:
    """Python项目数据验证器"""
    
    def __init__(self, db_path: str):
        """初始化验证器"""
        self.db_path = db_path
        self.conn = None
        self.validation_results = {
            'schema_validation': {},
            'data_validation': {},
            'integrity_check': {},
            'summary': {}
        }
    
    def connect(self) -> bool:
        """连接数据库"""
        try:
            self.conn = sqlite3.connect(self.db_path)
            self.conn.row_factory = sqlite3.Row
            return True
        except Exception as e:
            print(f"数据库连接失败: {e}")
            return False
    
    def disconnect(self):
        """断开数据库连接"""
        if self.conn:
            self.conn.close()
    
    def get_table_schema(self, table_name: str) -> Dict[str, Any]:
        """获取表结构信息"""
        try:
            cursor = self.conn.cursor()
            cursor.execute(f"PRAGMA table_info({table_name})")
            columns = cursor.fetchall()
            
            schema = {}
            for col in columns:
                schema[col['name']] = {
                    'type': col['type'],
                    'notnull': col['notnull'],
                    'default': col['dflt_value'],
                    'primary_key': col['pk']
                }
            return schema
        except Exception as e:
            print(f"获取表结构失败 {table_name}: {e}")
            return {}
    
    def get_table_row_count(self, table_name: str) -> int:
        """获取表的行数"""
        try:
            cursor = self.conn.cursor()
            cursor.execute(f"SELECT COUNT(*) as count FROM {table_name}")
            result = cursor.fetchone()
            return result['count'] if result else 0
        except Exception as e:
            print(f"获取行数失败 {table_name}: {e}")
            return 0
    
    def validate_table_schemas(self) -> Dict[str, Any]:
        """验证所有表的Schema"""
        tables = [
            'claude_providers',
            'codex_providers', 
            'agent_guides',
            'mcp_servers',
            'common_configs'
        ]
        
        schema_results = {}
        
        for table in tables:
            try:
                schema = self.get_table_schema(table)
                row_count = self.get_table_row_count(table)
                
                schema_results[table] = {
                    'exists': len(schema) > 0,
                    'columns': len(schema),
                    'row_count': row_count,
                    'schema': schema,
                    'issues': []
                }
                
                # 检查必要的字段
                required_fields = self.get_required_fields(table)
                for field in required_fields:
                    if field not in schema:
                        schema_results[table]['issues'].append(f"缺少必要字段: {field}")
                
                print(f"✓ 表 {table}: {len(schema)} 列, {row_count} 行")
                
            except Exception as e:
                schema_results[table] = {
                    'exists': False,
                    'error': str(e),
                    'issues': [f"表验证失败: {e}"]
                }
                print(f"❌ 表 {table}: 验证失败 - {e}")
        
        self.validation_results['schema_validation'] = schema_results
        return schema_results
    
    def get_required_fields(self, table_name: str) -> List[str]:
        """获取表的必要字段"""
        required_fields = {
            'claude_providers': [
                'id', 'name', 'url', 'token', 'max_tokens', 
                'temperature', 'model', 'enabled', 'description', 
                'timeout', 'retry_count', 'created_at', 'updated_at'
            ],
            'codex_providers': [
                'id', 'name', 'url', 'token', 'type', 'enabled', 
                'created_at', 'updated_at'
            ],
            'agent_guides': [
                'id', 'name', 'description', 'created_at', 'updated_at'
            ],
            'mcp_servers': [
                'id', 'name', 'url', 'command', 'args', 'enabled', 
                'description', 'created_at', 'updated_at'
            ],
            'common_configs': [
                'id', 'key', 'value', 'type', 'description', 
                'created_at', 'updated_at'
            ]
        }
        
        return required_fields.get(table_name, [])
    
    def validate_data_integrity(self) -> Dict[str, Any]:
        """验证数据完整性"""
        integrity_results = {}
        
        # 检查Claude供应商数据
        integrity_results['claude_providers'] = self.validate_claude_providers()
        
        # 检查Codex供应商数据
        integrity_results['codex_providers'] = self.validate_codex_providers()
        
        # 检查Agent指导数据
        integrity_results['agent_guides'] = self.validate_agent_guides()
        
        # 检查MCP服务器数据
        integrity_results['mcp_servers'] = self.validate_mcp_servers()
        
        # 检查通用配置数据
        integrity_results['common_configs'] = self.validate_common_configs()
        
        self.validation_results['integrity_check'] = integrity_results
        return integrity_results
    
    def validate_claude_providers(self) -> Dict[str, Any]:
        """验证Claude供应商数据完整性"""
        try:
            cursor = self.conn.cursor()
            cursor.execute("""
                SELECT id, name, url, max_tokens, temperature, model, 
                       enabled, description, timeout, retry_count
                FROM claude_providers
                ORDER BY id
            """)
            
            providers = cursor.fetchall()
            issues = []
            
            for provider in providers:
                # 检查必要字段
                if not provider['name'] or provider['name'].strip() == '':
                    issues.append(f"ID {provider['id']}: name字段为空")
                
                if not provider['url'] or provider['url'].strip() == '':
                    issues.append(f"ID {provider['id']}: url字段为空")
                
                if not provider['url'].startswith(('http://', 'https://')):
                    issues.append(f"ID {provider['id']}: url格式无效")
                
                if provider['enabled'] not in [0, 1]:
                    issues.append(f"ID {provider['id']}: enabled字段值无效")
                
                if provider['max_tokens'] and (provider['max_tokens'] < 1 or provider['max_tokens'] > 100000):
                    issues.append(f"ID {provider['id']}: max_tokens超出合理范围")
                
                if provider['temperature'] and (provider['temperature'] < 0.0 or provider['temperature'] > 2.0):
                    issues.append(f"ID {provider['id']}: temperature超出合理范围")
            
            # 检查启用供应商数量
            enabled_count = len([p for p in providers if p['enabled'] == 1])
            if enabled_count > 1:
                issues.append(f"多个启用供应商: {enabled_count}个")
            
            return {
                'total_count': len(providers),
                'enabled_count': enabled_count,
                'issues': issues,
                'success': len(issues) == 0
            }
            
        except Exception as e:
            return {'error': str(e), 'success': False}
    
    def validate_codex_providers(self) -> Dict[str, Any]:
        """验证Codex供应商数据完整性"""
        try:
            cursor = self.conn.cursor()
            cursor.execute("""
                SELECT id, name, url, type, enabled
                FROM codex_providers
                ORDER BY id
            """)
            
            providers = cursor.fetchall()
            issues = []
            
            for provider in providers:
                if not provider['name'] or provider['name'].strip() == '':
                    issues.append(f"ID {provider['id']}: name字段为空")
                
                if not provider['url'] or provider['url'].strip() == '':
                    issues.append(f"ID {provider['id']}: url字段为空")
                
                if not provider['url'].startswith(('http://', 'https://')):
                    issues.append(f"ID {provider['id']}: url格式无效")
                
                if provider['enabled'] not in [0, 1]:
                    issues.append(f"ID {provider['id']}: enabled字段值无效")
            
            return {
                'total_count': len(providers),
                'issues': issues,
                'success': len(issues) == 0
            }
            
        except Exception as e:
            return {'error': str(e), 'success': False}
    
    def validate_agent_guides(self) -> Dict[str, Any]:
        """验证Agent指导数据完整性"""
        try:
            cursor = self.conn.cursor()
            cursor.execute("""
                SELECT id, name, description
                FROM agent_guides
                ORDER BY id
            """)
            
            guides = cursor.fetchall()
            issues = []
            
            for guide in guides:
                if not guide['name'] or guide['name'].strip() == '':
                    issues.append(f"ID {guide['id']}: name字段为空")
                
                if not guide['description'] or guide['description'].strip() == '':
                    issues.append(f"ID {guide['id']}: description字段为空")
            
            return {
                'total_count': len(guides),
                'issues': issues,
                'success': len(issues) == 0
            }
            
        except Exception as e:
            return {'error': str(e), 'success': False}
    
    def validate_mcp_servers(self) -> Dict[str, Any]:
        """验证MCP服务器数据完整性"""
        try:
            cursor = self.conn.cursor()
            cursor.execute("""
                SELECT id, name, url, command, args, enabled
                FROM mcp_servers
                ORDER BY id
            """)
            
            servers = cursor.fetchall()
            issues = []
            
            for server in servers:
                if not server['name'] or server['name'].strip() == '':
                    issues.append(f"ID {server['id']}: name字段为空")
                
                if not server['command'] or server['command'].strip() == '':
                    issues.append(f"ID {server['id']}: command字段为空")
                
                if server['enabled'] not in [0, 1]:
                    issues.append(f"ID {server['id']}: enabled字段值无效")
            
            return {
                'total_count': len(servers),
                'issues': issues,
                'success': len(issues) == 0
            }
            
        except Exception as e:
            return {'error': str(e), 'success': False}
    
    def validate_common_configs(self) -> Dict[str, Any]:
        """验证通用配置数据完整性"""
        try:
            cursor = self.conn.cursor()
            cursor.execute("""
                SELECT id, key, value, type
                FROM common_configs
                ORDER BY id
            """)
            
            configs = cursor.fetchall()
            issues = []
            
            for config in configs:
                if not config['key'] or config['key'].strip() == '':
                    issues.append(f"ID {config['id']}: key字段为空")
                
                if not config['value'] or config['value'].strip() == '':
                    issues.append(f"ID {config['id']}: value字段为空")
                
                # 检查key的唯一性
                cursor.execute("SELECT COUNT(*) as count FROM common_configs WHERE key = ?", (config['key'],))
                duplicate_count = cursor.fetchone()['count']
                if duplicate_count > 1:
                    issues.append(f"Key '{config['key']}' 重复: {duplicate_count} 次")
            
            return {
                'total_count': len(configs),
                'issues': issues,
                'success': len(issues) == 0
            }
            
        except Exception as e:
            return {'error': str(e), 'success': False}
    
    def generate_sample_encrypted_data(self) -> Dict[str, str]:
        """生成加密数据样本（用于测试Rust解密兼容性）"""
        sample_data = {
            "simple_text": "Hello, World!",
            "chinese_text": "你好世界，这是中文测试数据",
            "api_token": "sk-1234567890abcdef1234567890abcdef12345678",
            "json_data": json.dumps({
                "name": "测试供应商",
                "url": "https://api.openai.com",
                "token": "sk-test-token",
                "model": "gpt-4",
                "enabled": True
            }, ensure_ascii=False)
        }
        
        # 这里只是返回原始数据，实际加密会在Python端完成
        return sample_data
    
    def run_full_validation(self) -> Dict[str, Any]:
        """运行完整的数据验证"""
        print("开始Python项目数据验证...")
        print("=" * 50)
        
        # 连接数据库
        if not self.connect():
            return {'success': False, 'error': '数据库连接失败'}
        
        try:
            # Schema验证
            print("1. 验证数据库表结构...")
            schema_results = self.validate_table_schemas()
            
            # 数据完整性验证
            print("\n2. 验证数据完整性...")
            integrity_results = self.validate_data_integrity()
            
            # 生成样本数据
            print("\n3. 生成测试数据样本...")
            sample_data = self.generate_sample_encrypted_data()
            
            # 汇总结果
            total_tables = len(schema_results)
            valid_tables = len([t for t in schema_results.values() if t.get('exists', False)])
            total_integrity_checks = len(integrity_results)
            passed_integrity_checks = len([t for t in integrity_results.values() if t.get('success', False)])
            
            overall_success = (valid_tables == total_tables and 
                              passed_integrity_checks == total_integrity_checks)
            
            summary = {
                'database_path': self.db_path,
                'validation_time': datetime.now().isoformat(),
                'schema_validation': {
                    'total_tables': total_tables,
                    'valid_tables': valid_tables,
                    'success': valid_tables == total_tables
                },
                'integrity_validation': {
                    'total_checks': total_integrity_checks,
                    'passed_checks': passed_integrity_checks,
                    'success': passed_integrity_checks == total_integrity_checks
                },
                'sample_data': sample_data,
                'overall_success': overall_success,
                'detailed_results': {
                    'schema': schema_results,
                    'integrity': integrity_results
                }
            }
            
            self.validation_results['summary'] = summary
            
            print(f"\n验证完成!")
            print(f"数据库: {self.db_path}")
            print(f"表结构验证: {valid_tables}/{total_tables}")
            print(f"数据完整性验证: {passed_integrity_checks}/{total_integrity_checks}")
            print(f"总体结果: {'✅ 通过' if overall_success else '❌ 失败'}")
            
            return summary
            
        finally:
            self.disconnect()
    
    def save_validation_report(self, output_path: str):
        """保存验证报告"""
        try:
            with open(output_path, 'w', encoding='utf-8') as f:
                json.dump(self.validation_results, f, indent=2, ensure_ascii=False, default=str)
            print(f"验证报告已保存到: {output_path}")
        except Exception as e:
            print(f"保存报告失败: {e}")

def main():
    """主函数"""
    if len(sys.argv) != 2:
        print("用法: python migration_validator.py <数据库文件路径>")
        sys.exit(1)
    
    db_path = sys.argv[1]
    
    if not os.path.exists(db_path):
        print(f"数据库文件不存在: {db_path}")
        sys.exit(1)
    
    # 创建验证器
    validator = PythonDataValidator(db_path)
    
    # 运行验证
    results = validator.run_full_validation()
    
    # 生成报告文件路径
    script_dir = os.path.dirname(os.path.abspath(__file__))
    report_path = os.path.join(script_dir, "python_validation_report.json")
    
    # 保存报告
    validator.save_validation_report(report_path)
    
    # 设置退出码
    sys.exit(0 if results['overall_success'] else 1)

if __name__ == "__main__":
    main()