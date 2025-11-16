// 应用启动时间性能测试
// 测试应用启动到完全可用的时间，包括数据库连接、前端渲染等

const puppeteer = require('puppeteer');
const path = require('path');
const fs = require('fs');

// 性能测试配置
const PERFORMANCE_CONFIG = {
  // 启动时间目标：2秒
  STARTUP_TIME_TARGET: 2000,
  // 测试重复次数
  TEST_ITERATIONS: 5,
  // 超时时间
  TIMEOUT: 10000,
  // 应用URL
  APP_URL: 'http://localhost:3000',
  // 后端API URL
  API_URL: 'http://localhost:8080',
};

// 测试结果收集
const testResults = {
  startupTimes: [],
  memoryUsage: [],
  errors: [],
};

// 等待应用启动
async function waitForAppStart(page) {
  const startTime = Date.now();
  
  try {
    // 等待页面加载
    await page.goto(PERFORMANCE_CONFIG.APP_URL, {
      waitUntil: 'domcontentloaded',
      timeout: PERFORMANCE_CONFIG.TIMEOUT,
    });

    // 等待关键元素加载
    await page.waitForSelector('[data-testid="app"]', { timeout: 5000 });
    await page.waitForSelector('[data-testid="navigation"]', { timeout: 3000 });
    await page.waitForSelector('[data-testid="main-content"]', { timeout: 3000 });

    // 检查API连接状态
    const apiStatus = await page.evaluate(async () => {
      try {
        const response = await fetch('http://localhost:8080/api/health');
        return response.ok;
      } catch (error) {
        return false;
      }
    });

    if (!apiStatus) {
      throw new Error('API服务器连接失败');
    }

    const endTime = Date.now();
    return endTime - startTime;
  } catch (error) {
    const endTime = Date.now();
    const duration = endTime - startTime;
    testResults.errors.push({
      iteration: testResults.startupTimes.length,
      error: error.message,
      duration,
    });
    throw error;
  }
}

// 获取内存使用情况
async function getMemoryUsage(page) {
  const memoryInfo = await page.evaluate(() => {
    if (performance && performance.memory) {
      return {
        usedJSHeapSize: performance.memory.usedJSHeapSize,
        totalJSHeapSize: performance.memory.totalJSHeapSize,
        jsHeapSizeLimit: performance.memory.jsHeapSizeLimit,
      };
    }
    return null;
  });

  return memoryInfo;
}

// 监控应用启动过程
async function monitorStartupProcess(page) {
  const events = [];
  
  // 监控关键事件
  page.on('response', (response) => {
    if (response.url().includes('/api/')) {
      events.push({
        type: 'api_call',
        url: response.url(),
        status: response.status(),
        timestamp: Date.now(),
      });
    }
  });

  page.on('load', () => {
    events.push({
      type: 'page_load',
      timestamp: Date.now(),
    });
  });

  page.on('domcontentloaded', () => {
    events.push({
      type: 'dom_loaded',
      timestamp: Date.now(),
    });
  });

  return events;
}

// 单次启动测试
async function runSingleStartupTest(browser, iteration) {
  console.log(`运行第 ${iteration + 1} 次启动测试...`);
  
  const page = await browser.newPage();
  
  // 启用性能监控
  await page.coverage.startJSCoverage();
  
  try {
    const monitorPromise = monitorStartupProcess(page);
    
    // 等待应用启动
    const startupTime = await waitForAppStart(page);
    testResults.startupTimes.push(startupTime);
    
    // 获取内存使用情况
    const memoryUsage = await getMemoryUsage(page);
    if (memoryUsage) {
      testResults.memoryUsage.push(memoryUsage);
    }

    // 等待监控完成
    const events = await monitorPromise;
    
    console.log(`第 ${iteration + 1} 次启动完成: ${startupTime}ms`);
    
    // 清理
    await page.coverage.stopJSCoverage();
    await page.close();
    
    return {
      startupTime,
      memoryUsage,
      events,
    };
    
  } catch (error) {
    await page.coverage.stopJSCoverage();
    await page.close();
    throw error;
  }
}

// 生成性能报告
function generatePerformanceReport() {
  const report = {
    timestamp: new Date().toISOString(),
    config: PERFORMANCE_CONFIG,
    results: {
      startupTimes: testResults.startupTimes,
      memoryUsage: testResults.memoryUsage,
      errors: testResults.errors,
    },
    statistics: {},
    summary: {},
  };

  if (testResults.startupTimes.length > 0) {
    const startupTimes = testResults.startupTimes;
    report.statistics.startup = {
      min: Math.min(...startupTimes),
      max: Math.max(...startupTimes),
      average: startupTimes.reduce((a, b) => a + b, 0) / startupTimes.length,
      median: startupTimes.sort((a, b) => a - b)[Math.floor(startupTimes.length / 2)],
      percentile95: startupTimes.sort((a, b) => a - b)[Math.floor(startupTimes.length * 0.95)],
    };

    report.summary.startupTargetMet = report.statistics.startup.average <= PERFORMANCE_CONFIG.STARTUP_TIME_TARGET;
    report.summary.startupPerformance = 
      report.statistics.startup.average <= PERFORMANCE_CONFIG.STARTUP_TIME_TARGET ? 'excellent' :
      report.statistics.startup.average <= PERFORMANCE_CONFIG.STARTUP_TIME_TARGET * 1.5 ? 'good' :
      'needs_improvement';
  }

  if (testResults.memoryUsage.length > 0) {
    const memoryUsages = testResults.memoryUsage.map(m => m.usedJSHeapSize);
    report.statistics.memory = {
      min: Math.min(...memoryUsages),
      max: Math.max(...memoryUsages),
      average: memoryUsages.reduce((a, b) => a + b, 0) / memoryUsages.length,
      // 转换为MB
      minMB: Math.min(...memoryUsages) / 1024 / 1024,
      maxMB: Math.max(...memoryUsages) / 1024 / 1024,
      averageMB: memoryUsages.reduce((a, b) => a + b, 0) / memoryUsages.length / 1024 / 1024,
    };

    report.summary.memoryTargetMet = report.statistics.memory.averageMB <= 100; // 100MB目标
  }

  report.summary.successRate = (testResults.startupTimes.length / PERFORMANCE_CONFIG.TEST_ITERATIONS) * 100;
  report.summary.errorCount = testResults.errors.length;

  return report;
}

// 保存报告到文件
function saveReport(report) {
  const reportDir = path.join(__dirname, '../reports');
  if (!fs.existsSync(reportDir)) {
    fs.mkdirSync(reportDir, { recursive: true });
  }

  const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
  const reportFile = path.join(reportDir, `startup-performance-${timestamp}.json`);
  
  fs.writeFileSync(reportFile, JSON.stringify(report, null, 2));
  console.log(`性能报告已保存到: ${reportFile}`);
  
  return reportFile;
}

// 主测试函数
async function runStartupPerformanceTests() {
  console.log('开始应用启动性能测试...');
  console.log(`目标启动时间: ${PERFORMANCE_CONFIG.STARTUP_TIME_TARGET}ms`);
  console.log(`测试次数: ${PERFORMANCE_CONFIG.TEST_ITERATIONS}`);
  
  let browser;
  try {
    // 启动浏览器
    browser = await puppeteer.launch({
      headless: true,
      args: [
        '--no-sandbox',
        '--disable-setuid-sandbox',
        '--disable-dev-shm-usage',
        '--disable-accelerated-2d-canvas',
        '--no-first-run',
        '--no-zygote',
        '--disable-gpu',
      ],
    });

    // 检查应用是否可用
    const testPage = await browser.newPage();
    try {
      await testPage.goto(PERFORMANCE_CONFIG.APP_URL, { timeout: 5000 });
    } catch (error) {
      console.error('应用不可用，请确保应用在 ', PERFORMANCE_CONFIG.APP_URL, ' 运行');
      console.error('错误:', error.message);
      process.exit(1);
    }
    await testPage.close();

    // 运行多次测试
    for (let i = 0; i < PERFORMANCE_CONFIG.TEST_ITERATIONS; i++) {
      try {
        await runSingleStartupTest(browser, i);
        
        // 在测试之间稍作等待
        if (i < PERFORMANCE_CONFIG.TEST_ITERATIONS - 1) {
          await new Promise(resolve => setTimeout(resolve, 1000));
        }
      } catch (error) {
        console.error(`第 ${i + 1} 次测试失败:`, error.message);
        // 继续进行下一次测试
      }
    }

    // 生成并保存报告
    const report = generatePerformanceReport();
    const reportFile = saveReport(report);

    // 输出结果摘要
    console.log('\n=== 启动性能测试结果 ===');
    console.log(`成功率: ${report.summary.successRate}%`);
    console.log(`平均启动时间: ${report.statistics.startup?.average || 'N/A'}ms`);
    console.log(`最快启动时间: ${report.statistics.startup?.min || 'N/A'}ms`);
    console.log(`最慢启动时间: ${report.statistics.startup?.max || 'N/A'}ms`);
    console.log(`目标达成: ${report.summary.startupTargetMet ? '✅' : '❌'}`);
    console.log(`性能评级: ${report.summary.startupPerformance || 'N/A'}`);
    
    if (report.statistics.memory) {
      console.log(`平均内存使用: ${report.statistics.memory.averageMB.toFixed(2)}MB`);
      console.log(`内存目标达成: ${report.summary.memoryTargetMet ? '✅' : '❌'}`);
    }
    
    if (report.summary.errorCount > 0) {
      console.log(`错误次数: ${report.summary.errorCount}`);
    }

    // 返回测试结果
    return {
      success: true,
      report,
      reportFile,
    };

  } catch (error) {
    console.error('启动性能测试失败:', error);
    return {
      success: false,
      error: error.message,
    };
  } finally {
    if (browser) {
      await browser.close();
    }
  }
}

// 如果直接运行此文件
if (require.main === module) {
  runStartupPerformanceTests()
    .then((result) => {
      if (result.success) {
        console.log('\n✅ 启动性能测试完成');
        process.exit(0);
      } else {
        console.log('\n❌ 启动性能测试失败');
        process.exit(1);
      }
    })
    .catch((error) => {
      console.error('测试执行错误:', error);
      process.exit(1);
    });
}

module.exports = {
  runStartupPerformanceTests,
  PERFORMANCE_CONFIG,
};