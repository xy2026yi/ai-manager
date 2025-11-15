#!/usr/bin/env node

// 性能测试运行器
// 统一运行所有性能测试并生成综合报告

const { runStartupPerformanceTests } = require('../tests/performance/startup.test.js');
const { runMemoryUsageTests } = require('../tests/performance/memory.test.js');
const { runApiPerformanceTests } = require('../tests/performance/api_response.test.js');
const fs = require('fs');
const path = require('path');

// 性能测试配置
const PERFORMANCE_CONFIG = {
  // 启动时间目标：2秒
  STARTUP_TIME_TARGET: 2000,
  // 内存使用目标：100MB
  MEMORY_TARGET_MB: 100,
  // API响应时间目标：500ms
  API_RESPONSE_TIME_TARGET: 500,
};

// 综合报告
const comprehensiveReport = {
  timestamp: new Date().toISOString(),
  config: PERFORMANCE_CONFIG,
  testResults: {},
  summary: {},
  recommendations: [],
};

// 运行启动性能测试
async function runStartupTests() {
  console.log('🚀 启动性能测试');
  console.log('='.repeat(50));
  
  try {
    const result = await runStartupPerformanceTests();
    comprehensiveReport.testResults.startup = result;
    
    if (result.success) {
      console.log('✅ 启动性能测试完成\n');
      return {
        success: true,
        averageStartupTime: result.report.statistics.startup.average,
        startupTargetMet: result.report.summary.startupTargetMet,
      };
    } else {
      console.log('❌ 启动性能测试失败\n');
      return { success: false, error: result.error };
    }
  } catch (error) {
    console.log('❌ 启动性能测试异常:', error.message, '\n');
    return { success: false, error: error.message };
  }
}

// 运行内存使用测试
async function runMemoryTests() {
  console.log('💾 内存使用测试');
  console.log('='.repeat(50));
  
  try {
    const result = await runMemoryUsageTests();
    comprehensiveReport.testResults.memory = result;
    
    if (result.success) {
      console.log('✅ 内存使用测试完成\n');
      return {
        success: true,
        averageMemoryUsage: result.report.statistics.browser.averageMB,
        memoryTargetMet: result.report.summary.memoryTargetMet,
        hasMemoryLeaks: result.report.summary.hasMemoryLeaks,
      };
    } else {
      console.log('❌ 内存使用测试失败\n');
      return { success: false, error: result.error };
    }
  } catch (error) {
    console.log('❌ 内存使用测试异常:', error.message, '\n');
    return { success: false, error: error.message };
  }
}

// 运行API性能测试
async function runApiTests() {
  console.log('🌐 API性能测试');
  console.log('='.repeat(50));
  
  try {
    const result = await runApiPerformanceTests();
    comprehensiveReport.testResults.api = result;
    
    if (result.success) {
      console.log('✅ API性能测试完成\n');
      return {
        success: true,
        averageResponseTime: result.report.statistics.overall.averageResponseTime,
        responseTimeTargetMet: result.report.summary.responseTimeTargetMet,
        endpointCount: result.report.statistics.overall.totalEndpoints,
        overallSuccessRate: result.report.summary.successRate,
      };
    } else {
      console.log('❌ API性能测试失败\n');
      return { success: false, error: result.error };
    }
  } catch (error) {
    console.log('❌ API性能测试异常:', error.message, '\n');
    return { success: false, error: error.message };
  }
}

// 生成性能评分
function generatePerformanceScore(results) {
  let score = 100;
  const deductions = [];
  
  // 启动性能评分 (权重: 30%)
  if (results.startup && results.startup.success) {
    const startupScore = Math.max(0, 100 - (results.startup.averageStartupTime / PERFORMANCE_CONFIG.STARTUP_TIME_TARGET - 1) * 100);
    score = score * 0.7 + startupScore * 0.3;
    
    if (results.startup.averageStartupTime > PERFORMANCE_CONFIG.STARTUP_TIME_TARGET * 2) {
      deductions.push('启动时间严重超标');
    }
  } else {
    deductions.push('启动性能测试失败');
    score *= 0.7;
  }
  
  // 内存使用评分 (权重: 25%)
  if (results.memory && results.memory.success) {
    const memoryScore = Math.max(0, 100 - (results.memory.averageMemoryUsage / PERFORMANCE_CONFIG.MEMORY_TARGET_MB - 1) * 100);
    score = score * 0.75 + memoryScore * 0.25;
    
    if (results.memory.averageMemoryUsage > PERFORMANCE_CONFIG.MEMORY_TARGET_MB * 2) {
      deductions.push('内存使用严重超标');
    }
    
    if (results.memory.hasMemoryLeaks) {
      deductions.push('检测到内存泄漏');
      score -= 10;
    }
  } else {
    deductions.push('内存使用测试失败');
    score *= 0.75;
  }
  
  // API性能评分 (权重: 25%)
  if (results.api && results.api.success) {
    const apiScore = Math.max(0, 100 - (results.api.averageResponseTime / PERFORMANCE_CONFIG.API_RESPONSE_TIME_TARGET - 1) * 100);
    score = score * 0.75 + apiScore * 0.25;
    
    if (results.api.averageResponseTime > PERFORMANCE_CONFIG.API_RESPONSE_TIME_TARGET * 2) {
      deductions.push('API响应时间严重超标');
    }
    
    if (results.api.overallSuccessRate < 95) {
      deductions.push('API成功率过低');
      score -= 5;
    }
  } else {
    deductions.push('API性能测试失败');
    score *= 0.75;
  }
  
  // 确保分数在0-100范围内
  score = Math.max(0, Math.min(100, Math.round(score)));
  
  return { score, deductions };
}

// 生成性能建议
function generateRecommendations(results) {
  const recommendations = [];
  
  // 启动性能建议
  if (results.startup && results.startup.success) {
    if (results.startup.averageStartupTime > PERFORMANCE_CONFIG.STARTUP_TIME_TARGET) {
      recommendations.push({
        category: '启动性能',
        priority: 'high',
        description: `启动时间 (${results.startup.averageStartupTime}ms) 超过目标值 (${PERFORMANCE_CONFIG.STARTUP_TIME_TARGET}ms)`,
        suggestions: [
          '优化应用初始化流程',
          '减少启动时的数据库查询',
          '实现懒加载机制',
          '优化依赖项加载顺序',
        ],
      });
    }
  } else {
    recommendations.push({
      category: '启动性能',
      priority: 'high',
      description: '启动性能测试失败，无法评估启动性能',
      suggestions: [
        '检查应用启动流程',
        '验证环境配置',
        '修复启动测试问题',
      ],
    });
  }
  
  // 内存使用建议
  if (results.memory && results.memory.success) {
    if (results.memory.averageMemoryUsage > PERFORMANCE_CONFIG.MEMORY_TARGET_MB) {
      recommendations.push({
        category: '内存使用',
        priority: 'medium',
        description: `平均内存使用 (${results.memory.averageMemoryUsage.toFixed(2)}MB) 超过目标值 (${PERFORMANCE_CONFIG.MEMORY_TARGET_MB}MB)`,
        suggestions: [
          '优化内存分配策略',
          '实现对象池或缓存机制',
          '检查内存泄漏问题',
          '优化数据结构使用',
        ],
      });
    }
    
    if (results.memory.hasMemoryLeaks) {
      recommendations.push({
        category: '内存使用',
        priority: 'high',
        description: '检测到内存泄漏问题',
        suggestions: [
          '使用内存分析工具定位泄漏点',
          '检查事件监听器和定时器清理',
          '验证闭包和循环引用',
          '优化组件生命周期管理',
        ],
      });
    }
  }
  
  // API性能建议
  if (results.api && results.api.success) {
    if (results.api.averageResponseTime > PERFORMANCE_CONFIG.API_RESPONSE_TIME_TARGET) {
      recommendations.push({
        category: 'API性能',
        priority: 'high',
        description: `API平均响应时间 (${results.api.averageResponseTime.toFixed(2)}ms) 超过目标值 (${PERFORMANCE_CONFIG.API_RESPONSE_TIME_TARGET}ms)`,
        suggestions: [
          '优化数据库查询性能',
          '实现API响应缓存',
          '减少序列化/反序列化开销',
          '优化网络传输数据量',
        ],
      });
    }
    
    if (results.api.overallSuccessRate < 99) {
      recommendations.push({
        category: 'API性能',
        priority: 'medium',
        description: `API成功率 (${results.api.overallSuccessRate.toFixed(2)}%) 低于理想值`,
        suggestions: [
          '改进错误处理机制',
          '增强API稳定性',
          '优化超时和重试策略',
          '完善监控和告警',
        ],
      });
    }
  }
  
  return recommendations;
}

// 保存综合报告
function saveComprehensiveReport(report) {
  const reportDir = path.join(__dirname, '../reports');
  if (!fs.existsSync(reportDir)) {
    fs.mkdirSync(reportDir, { recursive: true });
  }

  const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
  const reportFile = path.join(reportDir, `comprehensive-performance-${timestamp}.json`);
  
  fs.writeFileSync(reportFile, JSON.stringify(report, null, 2));
  console.log(`综合性能报告已保存到: ${reportFile}`);
  
  return reportFile;
}

// 主测试运行函数
async function runAllPerformanceTests() {
  console.log('🎯 AI Manager 性能基准测试套件');
  console.log('='.repeat(60));
  console.log(`启动时间目标: ${PERFORMANCE_CONFIG.STARTUP_TIME_TARGET}ms`);
  console.log(`内存使用目标: ${PERFORMANCE_CONFIG.MEMORY_TARGET_MB}MB`);
  console.log(`API响应时间目标: ${PERFORMANCE_CONFIG.API_RESPONSE_TIME_TARGET}ms`);
  console.log('='.repeat(60));
  console.log('');
  
  const startTime = Date.now();
  
  try {
    // 运行各项性能测试
    const startupResults = await runStartupTests();
    const memoryResults = await runMemoryTests();
    const apiResults = await runApiTests();
    
    const allResults = {
      startup: startupResults,
      memory: memoryResults,
      api: apiResults,
    };
    
    // 生成性能评分
    const { score, deductions } = generatePerformanceScore(allResults);
    
    // 生成建议
    const recommendations = generateRecommendations(allResults);
    
    // 完善综合报告
    comprehensiveReport.testResults = allResults;
    comprehensiveReport.summary = {
      overallScore: score,
      testDuration: Date.now() - startTime,
      successfulTests: Object.values(allResults).filter(r => r.success).length,
      totalTests: Object.keys(allResults).length,
      deductions,
    };
    comprehensiveReport.recommendations = recommendations;
    
    // 保存报告
    const reportFile = saveComprehensiveReport(comprehensiveReport);
    
    // 输出结果摘要
    console.log('📊 性能测试结果摘要');
    console.log('='.repeat(50));
    console.log(`总体评分: ${score}/100`);
    console.log(`测试通过率: ${comprehensiveReport.summary.successfulTests}/${comprehensiveReport.summary.totalTests}`);
    console.log(`总测试时间: ${(comprehensiveReport.summary.testDuration / 1000).toFixed(2)} 秒`);
    
    if (startupResults.success) {
      console.log(`启动时间: ${startupResults.averageStartupTime}ms ${startupResults.startupTargetMet ? '✅' : '❌'}`);
    }
    
    if (memoryResults.success) {
      console.log(`内存使用: ${memoryResults.averageMemoryUsage.toFixed(2)}MB ${memoryResults.memoryTargetMet ? '✅' : '❌'}`);
      console.log(`内存泄漏: ${memoryResults.hasMemoryLeaks ? '❌ 检测到泄漏' : '✅ 无泄漏'}`);
    }
    
    if (apiResults.success) {
      console.log(`API响应时间: ${apiResults.averageResponseTime.toFixed(2)}ms ${apiResults.responseTimeTargetMet ? '✅' : '❌'}`);
      console.log(`API成功率: ${apiResults.overallSuccessRate.toFixed(2)}%`);
    }
    
    // 输出性能等级
    let performanceGrade;
    if (score >= 90) {
      performanceGrade = 'A+ (优秀)';
    } else if (score >= 80) {
      performanceGrade = 'B+ (良好)';
    } else if (score >= 70) {
      performanceGrade = 'C+ (一般)';
    } else if (score >= 60) {
      performanceGrade = 'D+ (需要改进)';
    } else {
      performanceGrade = 'F (不合格)';
    }
    
    console.log(`\n🏆 性能等级: ${performanceGrade}`);
    
    // 输出建议摘要
    if (recommendations.length > 0) {
      console.log('\n💡 性能优化建议:');
      recommendations.slice(0, 3).forEach((rec, index) => {
        console.log(`${index + 1}. [${rec.category}] ${rec.description}`);
      });
      
      if (recommendations.length > 3) {
        console.log(`   ... 还有 ${recommendations.length - 3} 项建议，详见报告`);
      }
    }
    
    return {
      success: true,
      score,
      performanceGrade,
      recommendations,
      reportFile,
      comprehensiveReport,
    };
    
  } catch (error) {
    console.error('性能测试套件执行失败:', error);
    return {
      success: false,
      error: error.message,
    };
  }
}

// 如果直接运行此文件
if (require.main === module) {
  runAllPerformanceTests()
    .then((result) => {
      if (result.success) {
        console.log('\n✅ 性能基准测试套件完成');
        if (result.score >= 80) {
          console.log('🎉 性能表现良好！');
        } else {
          console.log('⚠️  建议进行性能优化');
        }
        process.exit(0);
      } else {
        console.log('\n❌ 性能基准测试套件失败');
        process.exit(1);
      }
    })
    .catch((error) => {
      console.error('测试执行错误:', error);
      process.exit(1);
    });
}

module.exports = {
  runAllPerformanceTests,
  PERFORMANCE_CONFIG,
};