// API响应时间性能测试
// 测试所有API端点的响应时间和并发处理能力

const axios = require('axios');
const fs = require('fs');
const path = require('path');

// API性能测试配置
const API_CONFIG = {
  // API基础URL
  BASE_URL: 'http://localhost:8080',
  // 响应时间目标：500ms
  RESPONSE_TIME_TARGET: 500,
  // 并发测试数量
  CONCURRENT_REQUESTS: 50,
  // 测试重复次数
  TEST_ITERATIONS: 10,
  // 超时时间（毫秒）
  TIMEOUT: 10000,
};

// API端点配置
const API_ENDPOINTS = [
  {
    name: '健康检查',
    method: 'GET',
    path: '/api/health',
    expectSuccess: true,
  },
  {
    name: '系统信息',
    method: 'GET',
    path: '/api/system/info',
    expectSuccess: true,
  },
  {
    name: '获取Claude供应商列表',
    method: 'GET',
    path: '/api/claude-providers',
    expectSuccess: true,
  },
  {
    name: '获取Codex供应商列表',
    method: 'GET',
    path: '/api/codex-providers',
    expectSuccess: true,
  },
  {
    name: '获取Agent指导列表',
    method: 'GET',
    path: '/api/agent-guides',
    expectSuccess: true,
  },
  {
    name: '获取MCP服务器列表',
    method: 'GET',
    path: '/api/mcp-servers',
    expectSuccess: true,
  },
  {
    name: '获取通用配置列表',
    method: 'GET',
    path: '/api/common-configs',
    expectSuccess: true,
  },
  {
    name: '创建Claude供应商',
    method: 'POST',
    path: '/api/claude-providers',
    expectSuccess: true,
    data: {
      name: '性能测试供应商',
      url: 'https://api.openai.com',
      token: 'sk-performance-test-token',
      max_tokens: 4096,
      temperature: 0.7,
      model: 'gpt-4',
      enabled: 1,
      description: '用于性能测试的供应商',
      timeout: 30,
      retry_count: 3,
    },
  },
];

// 测试结果收集
const testResults = {
  endpoints: {},
  concurrentTests: [],
  errors: [],
};

// 单个API请求测试
async function testSingleRequest(endpoint, iteration) {
  const startTime = Date.now();
  
  try {
    const config = {
      method: endpoint.method,
      url: `${API_CONFIG.BASE_URL}${endpoint.path}`,
      timeout: API_CONFIG.TIMEOUT,
      headers: {
        'Content-Type': 'application/json',
      },
    };

    if (endpoint.data) {
      config.data = endpoint.data;
    }

    const response = await axios(config);
    const endTime = Date.now();
    const responseTime = endTime - startTime;

    return {
      success: true,
      responseTime,
      statusCode: response.status,
      dataSize: JSON.stringify(response.data).length,
      iteration,
    };
    
  } catch (error) {
    const endTime = Date.now();
    const responseTime = endTime - startTime;

    return {
      success: false,
      responseTime,
      error: error.message,
      statusCode: error.response?.status || null,
      iteration,
    };
  }
}

// 运行单个端点的性能测试
async function runEndpointTest(endpoint) {
  console.log(`测试API端点: ${endpoint.name} (${endpoint.method} ${endpoint.path})`);
  
  const results = [];
  
  for (let i = 0; i < API_CONFIG.TEST_ITERATIONS; i++) {
    const result = await testSingleRequest(endpoint, i);
    results.push(result);
    
    if (result.success) {
      console.log(`  第 ${i + 1} 次测试: ${result.responseTime}ms ✅`);
    } else {
      console.log(`  第 ${i + 1} 次测试: ${result.responseTime}ms ❌ ${result.error}`);
    }
    
    // 在测试之间稍作等待
    if (i < API_CONFIG.TEST_ITERATIONS - 1) {
      await new Promise(resolve => setTimeout(resolve, 100));
    }
  }
  
  return results;
}

// 并发请求测试
async function runConcurrentTest(endpoint) {
  console.log(`运行并发测试: ${endpoint.name} (${API_CONFIG.CONCURRENT_REQUESTS} 个并发请求)`);
  
  const startTime = Date.now();
  
  // 创建并发请求
  const promises = Array.from({ length: API_CONFIG.CONCURRENT_REQUESTS }, (_, i) => 
    testSingleRequest(endpoint, i)
  );
  
  try {
    const results = await Promise.all(promises);
    const endTime = Date.now();
    const totalTime = endTime - startTime;
    
    const successful = results.filter(r => r.success);
    const failed = results.filter(r => !r.success);
    
    const testResult = {
      endpoint: endpoint.name,
      totalTime,
      concurrentRequests: API_CONFIG.CONCURRENT_REQUESTS,
      successful: successful.length,
      failed: failed.length,
      successRate: (successful.length / API_CONFIG.CONCURRENT_REQUESTS) * 100,
      averageResponseTime: successful.length > 0 ? 
        successful.reduce((sum, r) => sum + r.responseTime, 0) / successful.length : 0,
      minResponseTime: successful.length > 0 ? Math.min(...successful.map(r => r.responseTime)) : 0,
      maxResponseTime: successful.length > 0 ? Math.max(...successful.map(r => r.responseTime)) : 0,
      results,
    };
    
    console.log(`  总时间: ${totalTime}ms`);
    console.log(`  成功率: ${testResult.successRate.toFixed(2)}%`);
    console.log(`  平均响应时间: ${testResult.averageResponseTime.toFixed(2)}ms`);
    
    return testResult;
    
  } catch (error) {
    console.log(`  并发测试失败: ${error.message}`);
    return {
      endpoint: endpoint.name,
      error: error.message,
      success: false,
    };
  }
}

// 负载测试
async function runLoadTest() {
  console.log('开始负载测试...');
  
  const loadTestResults = [];
  
  // 逐步增加负载
  const loadLevels = [10, 20, 30, 40, 50];
  
  for (const load of loadLevels) {
    console.log(`负载级别: ${load} 个并发请求`);
    
    const endpoint = API_ENDPOINTS[0]; // 使用健康检查端点
    const promises = Array.from({ length: load }, (_, i) => testSingleRequest(endpoint, i));
    
    const startTime = Date.now();
    const results = await Promise.allSettled(promises);
    const endTime = Date.now();
    
    const successful = results.filter(r => r.status === 'fulfilled' && r.value.success);
    const totalTime = endTime - startTime;
    
    const loadResult = {
      load,
      totalTime,
      successful: successful.length,
      failed: load - successful.length,
      successRate: (successful.length / load) * 100,
      throughput: (successful.length / totalTime) * 1000, // 每秒请求数
    };
    
    loadTestResults.push(loadResult);
    
    console.log(`  成功率: ${loadResult.successRate.toFixed(2)}%`);
    console.log(`  吞吐量: ${loadResult.throughput.toFixed(2)} 请求/秒`);
    
    // 在不同负载级别之间等待
    await new Promise(resolve => setTimeout(resolve, 2000));
  }
  
  return loadTestResults;
}

// 生成API性能报告
function generateApiPerformanceReport() {
  const report = {
    timestamp: new Date().toISOString(),
    config: API_CONFIG,
    endpoints: {},
    concurrentTests: testResults.concurrentTests,
    loadTests: testResults.loadTests || [],
    statistics: {},
    summary: {},
  };

  // 处理每个端点的结果
  for (const endpoint of API_ENDPOINTS) {
    const results = testResults.endpoints[endpoint.name];
    if (!results || results.length === 0) continue;
    
    const successful = results.filter(r => r.success);
    const failed = results.filter(r => !r.success);
    
    const responseTimes = successful.map(r => r.responseTime);
    
    report.endpoints[endpoint.name] = {
      method: endpoint.method,
      path: endpoint.path,
      totalTests: results.length,
      successful: successful.length,
      failed: failed.length,
      successRate: (successful.length / results.length) * 100,
      averageResponseTime: successful.length > 0 ? 
        responseTimes.reduce((sum, r) => sum + r, 0) / responseTimes.length : 0,
      minResponseTime: successful.length > 0 ? Math.min(...responseTimes) : 0,
      maxResponseTime: successful.length > 0 ? Math.max(...responseTimes) : 0,
      medianResponseTime: successful.length > 0 ? 
        responseTimes.sort((a, b) => a - b)[Math.floor(responseTimes.length / 2)] : 0,
      percentile95: successful.length > 0 ? 
        responseTimes.sort((a, b) => a - b)[Math.floor(responseTimes.length * 0.95)] : 0,
    };
  }

  // 计算整体统计信息
  const allEndpointStats = Object.values(report.endpoints);
  if (allEndpointStats.length > 0) {
    const allResponseTimes = allEndpointStats
      .filter(stat => stat.averageResponseTime > 0)
      .map(stat => stat.averageResponseTime);
    
    if (allResponseTimes.length > 0) {
      report.statistics.overall = {
        averageResponseTime: allResponseTimes.reduce((sum, r) => sum + r, 0) / allResponseTimes.length,
        minResponseTime: Math.min(...allResponseTimes),
        maxResponseTime: Math.max(...allResponseTimes),
        totalEndpoints: allEndpointStats.length,
        endpointsWith100PercentSuccess: allEndpointStats.filter(s => s.successRate === 100).length,
      };
    }
  }

  // 并发测试统计
  if (testResults.concurrentTests.length > 0) {
    const concurrentStats = testResults.concurrentTests.filter(t => t.success !== false);
    report.statistics.concurrent = {
      totalTests: concurrentStats.length,
      averageSuccessRate: concurrentStats.reduce((sum, t) => sum + t.successRate, 0) / concurrentStats.length,
      averageResponseTime: concurrentStats.reduce((sum, t) => sum + t.averageResponseTime, 0) / concurrentStats.length,
    };
  }

  // 生成摘要
  report.summary = {
    responseTimeTargetMet: report.statistics.overall ? 
      report.statistics.overall.averageResponseTime <= API_CONFIG.RESPONSE_TIME_TARGET : false,
    overallPerformance: report.statistics.overall ? 
      (report.statistics.overall.averageResponseTime <= API_CONFIG.RESPONSE_TIME_TARGET ? 'excellent' :
       report.statistics.overall.averageResponseTime <= API_CONFIG.RESPONSE_TIME_TARGET * 1.5 ? 'good' : 'needs_improvement') : 'unknown',
    endpointCount: allEndpointStats.length,
    successRate: allEndpointStats.length > 0 ? 
      allEndpointStats.reduce((sum, s) => sum + s.successRate, 0) / allEndpointStats.length : 0,
  };

  return report;
}

// 保存API性能报告
function saveApiPerformanceReport(report) {
  const reportDir = path.join(__dirname, '../reports');
  if (!fs.existsSync(reportDir)) {
    fs.mkdirSync(reportDir, { recursive: true });
  }

  const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
  const reportFile = path.join(reportDir, `api-performance-${timestamp}.json`);
  
  fs.writeFileSync(reportFile, JSON.stringify(report, null, 2));
  console.log(`API性能报告已保存到: ${reportFile}`);
  
  return reportFile;
}

// 主API性能测试函数
async function runApiPerformanceTests() {
  console.log('开始API响应时间性能测试...');
  console.log(`响应时间目标: ${API_CONFIG.RESPONSE_TIME_TARGET}ms`);
  console.log(`测试重复次数: ${API_CONFIG.TEST_ITERATIONS}`);
  console.log(`并发请求数: ${API_CONFIG.CONCURRENT_REQUESTS}`);
  
  try {
    // 首先检查API服务器是否可用
    try {
      await axios.get(`${API_CONFIG.BASE_URL}/api/health`, { timeout: 5000 });
      console.log('✅ API服务器连接正常');
    } catch (error) {
      console.error('❌ API服务器连接失败:', error.message);
      console.error('请确保API服务器在', API_CONFIG.BASE_URL, '运行');
      return {
        success: false,
        error: 'API服务器不可用',
      };
    }

    // 测试每个API端点
    for (const endpoint of API_ENDPOINTS) {
      const results = await runEndpointTest(endpoint);
      testResults.endpoints[endpoint.name] = results;
      
      console.log(`  平均响应时间: ${results.filter(r => r.success).length > 0 ? 
        (results.filter(r => r.success).reduce((sum, r) => sum + r.responseTime, 0) / 
         results.filter(r => r.success).length).toFixed(2) : 'N/A'}ms\n`);
    }

    // 并发测试
    console.log('\n开始并发测试...');
    for (const endpoint of API_ENDPOINTS.slice(0, 3)) { // 只测试前3个端点
      const concurrentResult = await runConcurrentTest(endpoint);
      testResults.concurrentTests.push(concurrentResult);
      console.log('');
    }

    // 负载测试
    console.log('\n开始负载测试...');
    const loadTestResults = await runLoadTest();
    testResults.loadTests = loadTestResults;

    // 生成报告
    const report = generateApiPerformanceReport();
    const reportFile = saveApiPerformanceReport(report);

    // 输出结果摘要
    console.log('\n=== API性能测试结果 ===');
    console.log(`测试端点数量: ${Object.keys(report.endpoints).length}`);
    
    if (report.statistics.overall) {
      console.log(`平均响应时间: ${report.statistics.overall.averageResponseTime.toFixed(2)}ms`);
      console.log(`最快响应时间: ${report.statistics.overall.minResponseTime}ms`);
      console.log(`最慢响应时间: ${report.statistics.overall.maxResponseTime}ms`);
      console.log(`100%成功率端点: ${report.statistics.overall.endpointsWith100PercentSuccess}/${report.statistics.overall.totalEndpoints}`);
      console.log(`响应时间目标达成: ${report.summary.responseTimeTargetMet ? '✅' : '❌'}`);
      console.log(`整体性能评级: ${report.summary.overallPerformance || 'N/A'}`);
    }
    
    console.log(`平均成功率: ${report.summary.successRate.toFixed(2)}%`);

    if (report.statistics.concurrent) {
      console.log(`并发测试平均成功率: ${report.statistics.concurrent.averageSuccessRate.toFixed(2)}%`);
    }

    return {
      success: true,
      report,
      reportFile,
    };

  } catch (error) {
    console.error('API性能测试失败:', error);
    return {
      success: false,
      error: error.message,
    };
  }
}

// 如果直接运行此文件
if (require.main === module) {
  runApiPerformanceTests()
    .then((result) => {
      if (result.success) {
        console.log('\n✅ API性能测试完成');
        process.exit(0);
      } else {
        console.log('\n❌ API性能测试失败');
        process.exit(1);
      }
    })
    .catch((error) => {
      console.error('测试执行错误:', error);
      process.exit(1);
    });
}

module.exports = {
  runApiPerformanceTests,
  API_CONFIG,
  API_ENDPOINTS,
};