#!/usr/bin/env node

/**
 * 数据兼容性验证测试运行器
 * 统一运行所有数据迁移和兼容性测试
 */

const { spawn } = require('child_process');
const fs = require('fs');
const path = require('path');
const os = require('os');

// 测试配置
const TEST_CONFIG = {
    // 测试数据目录
    testDataDir: path.join(__dirname, '..'),
    // Python验证脚本路径
    pythonValidatorScript: path.join(__dirname, 'migration_validator.py'),
    // 输出报告目录
    reportsDir: path.join(__dirname, '..', '..', 'reports'),
    // Rust测试二进制路径
    rustTestBinary: path.join(__dirname, '..', '..', 'src-tauri', 'target', 'debug', 'migration_ai_manager_tests'),
};

// 测试结果
const testResults = {
    pythonValidation: null,
    rustMigrationTests: null,
    rustEncryptionTests: null,
    summary: null,
};

// 颜色码输出
const colors = {
    reset: '\x1b[0m',
    red: '\x1b[31m',
    green: '\x1b[32m',
    yellow: '\x1b[33m',
    blue: '\x1b[34m',
    magenta: '\x1b[35m',
    cyan: '\x1b[36m',
    white: '\x1b[37m',
};

function log(message, color = 'white') {
    console.log(`${colors[color]}${message}${colors.reset}`);
}

function logSuccess(message) {
    log(`✅ ${message}`, 'green');
}

function logError(message) {
    log(`❌ ${message}`, 'red');
}

function logWarning(message) {
    log(`⚠️  ${message}`, 'yellow');
}

function logInfo(message) {
    log(`ℹ️  ${message}`, 'cyan');
}

// 创建测试数据库
async function createTestDatabase() {
    logInfo('创建测试数据库...');
    
    const testDbPath = path.join(TEST_CONFIG.testDataDir, 'python_original.db');
    
    // 如果测试数据库不存在，创建一个简单的测试数据库
    if (!fs.existsSync(testDbPath)) {
        log('创建测试数据库文件...', 'yellow');
        
        // 这里简化处理，实际项目中应该有真实的Python数据库
        // 创建基本的SQLite数据库结构
        const createTestDbScript = `
import sqlite3
import sys

def create_test_database(db_path):
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()
    
    # 创建Claude供应商表
    cursor.execute("""
        CREATE TABLE IF NOT EXISTS claude_providers (
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
    """)
    
    # 创建Codex供应商表
    cursor.execute("""
        CREATE TABLE IF NOT EXISTS codex_providers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            url TEXT NOT NULL,
            token TEXT NOT NULL,
            type TEXT,
            enabled INTEGER DEFAULT 1,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
    """)
    
    # 插入测试数据
    cursor.execute("""
        INSERT INTO claude_providers (name, url, token, enabled, description) 
        VALUES 
        ('测试Claude供应商1', 'https://api.openai.com', 'sk-test-token-1', 1, '测试用Claude供应商1'),
        ('测试Claude供应商2', 'https://api.anthropic.com', 'sk-test-token-2', 0, '测试用Claude供应商2')
    """)
    
    cursor.execute("""
        INSERT INTO codex_providers (name, url, token, type, enabled)
        VALUES 
        ('测试Codex供应商1', 'https://api.openai.com', 'sk-codex-token-1', 'gpt-4', 1),
        ('测试Codex供应商2', 'https://github.com', 'sk-codex-token-2', 'gpt-3.5-turbo', 0)
    """)
    
    conn.commit()
    conn.close()
    print(f"测试数据库创建成功: {db_path}")

if __name__ == '__main__':
    if len(sys.argv) > 1:
        create_test_database(sys.argv[1])
    else:
        create_test_database('test.db')
`;
        
        fs.writeFileSync(path.join(TEST_CONFIG.testDataDir, 'create_test_db.py'), createTestDbScript);
        
        // 运行Python脚本创建数据库
        return new Promise((resolve, reject) => {
            const python = spawn('python3', [path.join(TEST_CONFIG.testDataDir, 'create_test_db.py'), testDbPath]);
            
            python.on('close', (code) => {
                if (code === 0) {
                    logSuccess('测试数据库创建成功');
                    resolve();
                } else {
                    logError(`测试数据库创建失败，退出码: ${code}`);
                    reject(new Error('创建测试数据库失败'));
                }
            });
            
            python.on('error', (error) => {
                logError(`Python执行错误: ${error.message}`);
                reject(error);
            });
        });
    } else {
        logSuccess('测试数据库已存在');
        return Promise.resolve();
    }
}

// 运行Python数据验证
async function runPythonValidation() {
    logInfo('运行Python数据验证...');
    
    const pythonDbPath = path.join(TEST_CONFIG.testDataDir, 'python_original.db');
    
    if (!fs.existsSync(pythonDbPath)) {
        logWarning('Python数据库文件不存在，跳过Python验证');
        return { success: true, skipped: true, message: '数据库文件不存在' };
    }
    
    return new Promise((resolve, reject) => {
        const python = spawn('python3', [TEST_CONFIG.pythonValidatorScript, pythonDbPath]);
        let output = '';
        let errorOutput = '';
        
        python.stdout.on('data', (data) => {
            output += data.toString();
        });
        
        python.stderr.on('data', (data) => {
            errorOutput += data.toString();
        });
        
        python.on('close', (code) => {
            if (code === 0) {
                logSuccess('Python数据验证完成');
                
                // 尝试解析生成的报告文件
                const reportPath = path.join(TEST_CONFIG.testDataDir, 'python_validation_report.json');
                if (fs.existsSync(reportPath)) {
                    try {
                        const reportData = JSON.parse(fs.readFileSync(reportPath, 'utf8'));
                        resolve({ 
                            success: reportData.overall_success, 
                            skipped: false, 
                            report: reportData 
                        });
                    } catch (e) {
                        logWarning('无法解析Python验证报告，使用输出结果');
                        resolve({ 
                            success: code === 0, 
                            skipped: false, 
                            output: output,
                            error: errorOutput 
                        });
                    }
                } else {
                    resolve({ 
                        success: code === 0, 
                        skipped: false, 
                        output: output,
                        error: errorOutput 
                    });
                }
            } else {
                logError(`Python验证失败，退出码: ${code}`);
                reject(new Error(`Python验证失败: ${errorOutput || output}`));
                }
            });
            
            python.on('error', (error) => {
                logError(`Python执行错误: ${error.message}`);
                reject(error);
            });
        });
    });
}

// 检查Rust测试二进制是否存在
function checkRustBinary() {
    const binaryPath = TEST_CONFIG.rustBinary;
    
    if (!fs.existsSync(binaryPath)) {
        logWarning('Rust测试二进制不存在，需要先构建');
        logInfo('运行以下命令构建测试二进制:');
        logInfo('  cd src-tauri && cargo test --bin migration_ai_manager_tests');
        return false;
    }
    
    return true;
}

// 构建Rust测试二进制
async function buildRustTests() {
    logInfo('构建Rust测试二进制...');
    
    return new Promise((resolve, reject) => {
        const cargo = spawn('cargo', ['test', '--bin', 'migration_ai_manager_tests'], {
            cwd: path.join(TEST_CONFIG.testDataDir, '..', 'src-tauri'),
            stdio: 'inherit'
        });
        
        cargo.on('close', (code) => {
            if (code === 0) {
                logSuccess('Rust测试二进制构建成功');
                resolve();
            } else {
                logError(`Rust测试二进制构建失败，退出码: ${code}`);
                reject(new Error('构建失败'));
            }
        });
        
        cargo.on('error', (error) => {
            logError(`构建错误: ${error.message}`);
            reject(error);
        });
    });
}

// 运行Rust迁移测试
async function runRustMigrationTests() {
    logInfo('运行Rust迁移测试...');
    
    const testDbPath = path.join(TEST_CONFIG.testDataDir, 'python_original.db');
    const rustDbPath = path.join(TEST_CONFIG.testDataDir, 'rust_migrated.db');
    
    if (!fs.existsSync(testDbPath)) {
        logWarning('Python测试数据库不存在，跳过Rust迁移测试');
        return { success: true, skipped: true, message: '源数据库不存在' };
    }
    
    // 创建环境变量文件
    const envFile = path.join(TEST_CONFIG.testDataDir, '..', 'src-tauri', '.env.test');
    const envContent = `
TEST_MODE=migration
PYTHON_DB_PATH=${testDbPath}
RUST_DB_PATH=${rustDbPath}
ENCRYPTION_KEY=test_migration_key_32_bytes_long!
`;
    
    fs.writeFileSync(envFile, envContent);
    
    return new Promise((resolve, reject) => {
        const rustTest = spawn(TEST_CONFIG.rustBinary, [], {
            cwd: path.join(TEST_CONFIG.testDataDir, '..', 'src-tauri'),
            env: { ...process.env, RUST_LOG: 'debug' }
        });
        
        let output = '';
        let errorOutput = '';
        
        rustTest.stdout.on('data', (data) => {
            output += data.toString();
        });
        
        rustTest.stderr.on('data', (data) => {
            errorOutput += data.toString();
        });
        
        rustTest.on('close', (code) => {
            // 清理环境变量文件
            if (fs.existsSync(envFile)) {
                fs.unlinkSync(envFile);
            }
            
            if (code === 0) {
                logSuccess('Rust迁移测试完成');
                resolve({ 
                    success: true, 
                    skipped: false, 
                    output: output,
                    error: errorOutput 
                });
            } else {
                logError(`Rust迁移测试失败，退出码: ${code}`);
                reject(new Error(`Rust迁移测试失败: ${errorOutput || output}`));
                }
            });
            
            rustTest.on('error', (error) => {
                // 清理环境变量文件
                if (fs.existsSync(envFile)) {
                    fs.unlinkSync(envFile);
                }
                
                logError(`Rust测试执行错误: ${error.message}`);
                reject(error);
            });
        });
    });
}

// 运行Rust加密兼容性测试
async function runRustEncryptionTests() {
    logInfo('运行Rust加密兼容性测试...');
    
    return new Promise((resolve, reject) => {
        const rustTest = spawn(TEST_CONFIG.rustBinary, ['encryption_compatibility'], {
            cwd: path.join(TEST_CONFIG.testDataDir, '..', 'src-tauri'),
            env: { ...process.env, RUST_LOG: 'info' }
        });
        
        let output = '';
        let errorOutput = '';
        
        rustTest.stdout.on('data', (data) => {
            output += data.toString();
        });
        
        rustTest.stderr.on('data', (data) => {
            errorOutput += data.toString();
        });
        
        rustTest.on('close', (code) => {
            if (code === 0) {
                logSuccess('Rust加密兼容性测试完成');
                resolve({ 
                    success: true, 
                    skipped: false, 
                    output: output,
                    error: errorOutput 
                });
            } else {
                logError(`Rust加密兼容性测试失败，退出码: ${code}`);
                reject(new Error(`Rust加密兼容性测试失败: ${errorOutput || output}`));
                }
            });
            
            rustTest.on('error', (error) => {
                logError(`Rust测试执行错误: ${error.message}`);
                reject(error);
            });
        });
    });
}

// 生成综合报告
function generateComprehensiveReport() {
    logInfo('生成综合兼容性验证报告...');
    
    const reportDir = TEST_CONFIG.reportsDir;
    if (!fs.existsSync(reportDir)) {
        fs.mkdirSync(reportDir, { recursive: true });
    }
    
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
    const reportFile = path.join(reportDir, `data-compatibility-${timestamp}.json`);
    
    const comprehensiveReport = {
        timestamp: new Date().toISOString(),
        test_environment: {
            node_version: process.version,
            platform: os.platform(),
            arch: os.arch(),
        },
        test_results: testResults,
        summary: {
            python_validation: testResults.pythonValidation ? testResults.pythonValidation.success : null,
            rust_migration: testResults.rustMigrationTests ? testResults.rustMigrationTests.success : null,
            rust_encryption: testResults.rustEncryptionTests ? testResults.rustEncryptionTests.success : null,
            overall_success: true,
        },
        recommendations: []
    };
    
    // 计算总体成功状态
    const tests = [testResults.python_validation, testResults.rustMigrationTests, testResults.rustEncryptionTests];
    const successfulTests = tests.filter(t => t && t.success).length;
    const totalTests = tests.filter(t => t && !t.skipped).length;
    
    if (totalTests > 0) {
        comprehensiveReport.summary.overall_success = successfulTests === totalTests;
    }
    
    // 生成建议
    if (!comprehensiveReport.summary.overall_success) {
        comprehensiveReport.recommendations.push('检查并修复测试失败的问题');
        comprehensiveReport.recommendations.push('确保数据迁移的完整性和加密兼容性');
    } else {
        comprehensiveReport.recommendations.push('数据兼容性验证通过，可以进行完整的数据迁移');
        comprehensiveReport.recommendations.push('建议在生产环境运行一次完整的数据迁移测试');
    }
    
    fs.writeFileSync(reportFile, JSON.stringify(comprehensiveReport, null, 2));
    
    logSuccess(`综合报告已保存到: ${reportFile}`);
    return reportFile;
}

// 主测试运行函数
async function runDataCompatibilityTests() {
    console.log('🔍 数据兼容性验证测试套件');
    console.log('='.repeat(60));
    console.log('验证从Python项目到Rust项目的数据迁移兼容性');
    console.log('='.repeat(60));
    console.log('');
    
    try {
        // 确保报告目录存在
        const reportDir = TEST_CONFIG.reportsDir;
        if (!fs.existsSync(reportDir)) {
            fs.mkdirSync(reportDir, { recursive: true });
        }
        
        // 步骤1: 创建测试数据库
        logInfo('步骤 1/5: 准备测试环境');
        await createTestDatabase();
        
        // 步骤2: 运行Python数据验证
        logInfo('步骤 2/5: 运行Python数据验证');
        testResults.pythonValidation = await runPythonValidation();
        
        // 步骤3: 检查并构建Rust测试
        logInfo('步骤 3/5: 检查Rust测试环境');
        let rustBinaryAvailable = checkRustBinary();
        if (!rustBinaryAvailable) {
            logInfo('构建Rust测试二进制...');
            await buildRustTests();
            rustBinaryAvailable = true;
        }
        
        // 步骤4: 运行Rust迁移测试
        if (rustBinaryAvailable) {
            logInfo('步骤 4/5: 运行Rust迁移测试');
            testResults.rustMigrationTests = await runRustMigrationTests();
            
            // 步骤5: 运行Rust加密兼容性测试
            logInfo('步骤 5/5: 运行Rust加密兼容性测试');
            testResults.rustEncryptionTests = await runRustEncryptionTests();
        } else {
            logWarning('跳过Rust测试（二进制不可用）');
            testResults.rustMigrationTests = { success: false, skipped: true, message: '二进制不可用' };
            testResults.rustEncryptionTests = { success: false, skipped: true, message: '二进制不可用' };
        }
        
        // 生成综合报告
        const reportFile = generateComprehensiveReport();
        
        // 输出结果摘要
        console.log('\n📊 数据兼容性验证结果摘要');
        console.log('='.repeat(50));
        
        const pythonStatus = testResults.pythonValidation && testResults.pythonValidation.success;
        const rustMigrationStatus = testResults.rustMigrationTests && testResults.rustMigrationTests.success;
        const rustEncryptionStatus = testResults.rustEncryptionTests && testResults.rustEncryptionTests.success;
        
        console.log(`Python数据验证: ${pythonStatus ? '✅ 通过' : (testResults.pythonValidation?.skipped ? '⏭ 跳过' : '❌ 失败')}`);
        console.log(`Rust数据迁移测试: ${rustMigrationStatus ? '✅ 通过' : (testResults.rustMigrationTests?.skipped ? '⏭ 跳过' : '❌ 失败')}`);
        console.log(`Rust加密兼容性测试: ${rustEncryptionStatus ? '✅ 通过' : (testResults.rustEncryptionTests?.skipped ? '⏭ 跳过' : '❌ 失败')}`);
        
        const overallStatus = comprehensiveReport.summary.overall_success;
        console.log(`\n🏆 总体验证结果: ${overallStatus ? '✅ 通过' : '❌ 失败'}`);
        
        if (overallStatus) {
            console.log('\n🎉 数据兼容性验证完全通过！');
            console.log('可以安全进行完整的数据迁移。');
        } else {
            console.log('\n⚠️ 数据兼容性验证发现问题，需要修复后再进行迁移。');
        }
        
        console.log(`\n📄 详细报告: ${reportFile}`);
        
        return {
            success: overallStatus,
            reportFile,
            testResults: comprehensiveReport,
        };
        
    } catch (error) {
        console.error('\n❌ 数据兼容性验证失败:', error.message);
        return {
            success: false,
            error: error.message,
        };
    }
}

// 如果直接运行此文件
if (require.main === module) {
    runDataCompatibilityTests()
        .then((result) => {
            process.exit(result.success ? 0 : 1);
        })
        .catch((error) => {
            console.error('测试执行错误:', error);
            process.exit(1);
        });
}

module.exports = {
    runDataCompatibilityTests,
    TEST_CONFIG,
    testResults,
};