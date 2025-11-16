// 内存使用监控测试
// 监控应用在运行时的内存使用情况，检测内存泄漏和高内存使用

const puppeteer = require('puppeteer');
const path = require('path');
const fs = require('fs');

// 内存测试配置
const MEMORY_CONFIG = {
  // 内存使用目标：100MB
  MEMORY_TARGET_MB: 100,
  // 测试持续时间（分钟）
  TEST_DURATION_MINUTES: 5,
  // 采样间隔（毫秒）
  SAMPLE_INTERVAL_MS: 1000,
  // 应用URL
  APP_URL: 'http://localhost:3000',
  // API URL
  API_URL: 'http://localhost:8080',
};

// 内存样本数据
const memorySamples = [];
let isMonitoring = false;

// 获取页面内存信息
async function getPageMemoryInfo(page) {
  const memoryInfo = await page.evaluate(() => {
    if (performance && performance.memory) {
      return {
        usedJSHeapSize: performance.memory.usedJSHeapSize,
        totalJSHeapSize: performance.memory.totalJSHeapSize,
        jsHeapSizeLimit: performance.memory.jsHeapSizeLimit,
        timestamp: Date.now(),
      };
    }
    return null;
  });

  // 获取进程内存信息（如果可用）
  const processInfo = await page.evaluate(() => {
    if (process && process.memoryUsage) {
      return {
        rss: process.memoryUsage().rss,
        heapTotal: process.memoryUsage().heapTotal,
        heapUsed: process.memoryUsage().heapUsed,
        external: process.memoryUsage().external,
        timestamp: Date.now(),
      };
    }
    return null;
  });

  return {
    browser: memoryInfo,
    process: processInfo,
    timestamp: Date.now(),
  };
}

// 执行内存密集型操作
async function performMemoryIntensiveOperations(page) {
  const operations = [
    // 1. 导航到不同页面
    async () => {
      await page.goto(`${MEMORY_CONFIG.APP_URL}/claude-providers`, { waitUntil: 'domcontentloaded' });
    },
    
    // 2. 创建大量数据
    async () => {
      await page.evaluate(() => {
        // 创建大量DOM元素
        const container = document.createElement('div');
        container.style.display = 'none';
        for (let i = 0; i < 1000; i++) {
          const div = document.createElement('div');
          div.textContent = `测试数据 ${i}`;
          container.appendChild(div);
        }
        document.body.appendChild(container);
      });
    },
    
    // 3. API调用测试
    async () => {
      await page.evaluate(async () => {
        // 模拟多个API调用
        const promises = [];
        for (let i = 0; i < 10; i++) {
          promises.push(
            fetch('http://localhost:8080/api/claude-providers')
              .then(response => response.json())
              .catch(() => ({}))
          );
        }
        return Promise.all(promises);
      });
    },
    
    // 4. 创建和销毁组件
    async () => {
      await page.evaluate(() => {
        // 创建和销毁React组件的模拟
        const components = [];
        for (let i = 0; i < 100; i++) {
          components.push({
            id: i,
            data: new Array(1000).fill(0).map((_, j) => ({ index: j, value: Math.random() })),
            timestamp: Date.now(),
          });
        }
        // 清理引用
        components.length = 0;
      });
    },
    
    // 5. 返回主页
    async () => {
      await page.goto(MEMORY_CONFIG.APP_URL, { waitUntil: 'domcontentloaded' });
    },
  ];

  // 随机执行操作
  const randomOperation = operations[Math.floor(Math.random() * operations.length)];
  await randomOperation();
}

// 监控内存使用
async function monitorMemoryUsage(page) {
  if (!isMonitoring) return;
  
  const memoryInfo = await getPageMemoryInfo(page);
  memorySamples.push(memoryInfo);
  
  // 输出当前内存使用情况
  if (memoryInfo.browser) {
    const usedMB = memoryInfo.browser.usedJSHeapSize / 1024 / 1024;
    const totalMB = memoryInfo.browser.totalJSHeapSize / 1024 / 1024;
    console.log(`内存使用: ${usedMB.toFixed(2)}MB (总计: ${totalMB.toFixed(2)}MB)`);
  }
}

// 检测内存泄漏
function detectMemoryLeaks() {
  if (memorySamples.length < 10) return { hasLeaks: false, analysis: '样本数量不足' };
  
  const browserMemory = memorySamples
    .filter(s => s.browser)
    .map(s => s.browser.usedJSHeapSize);
  
  if (browserMemory.length === 0) return { hasLeaks: false, analysis: '没有有效的内存数据' };
  
  // 计算内存增长趋势
  const firstHalf = browserMemory.slice(0, Math.floor(browserMemory.length / 2));
  const secondHalf = browserMemory.slice(Math.floor(browserMemory.length / 2));
  
  const firstHalfAvg = firstHalf.reduce((a, b) => a + b, 0) / firstHalf.length;
  const secondHalfAvg = secondHalf.reduce((a, b) => a + b, 0) / secondHalf.length;
  
  const growthRate = ((secondHalfAvg - firstHalfAvg) / firstHalfAvg) * 100;
  
  // 检测是否持续增长
  let consecutiveIncreases = 0;
  let maxConsecutiveIncreases = 0;
  
  for (let i = 1; i < browserMemory.length; i++) {
    if (browserMemory[i] > browserMemory[i - 1]) {
      consecutiveIncreases++;
      maxConsecutiveIncreases = Math.max(maxConsecutiveIncreases, consecutiveIncreases);
    } else {
      consecutiveIncreases = 0;
    }
  }
  
  const hasLeaks = growthRate > 20 || maxConsecutiveIncreases > browserMemory.length * 0.7;
  
  return {
    hasLeaks,
    growthRate,
    maxConsecutiveIncreases,
    analysis: hasLeaks ? 
      `检测到可能的内存泄漏，增长率: ${growthRate.toFixed(2)}%，最大连续增长: ${maxConsecutiveIncreases}次` :
      `内存使用正常，增长率: ${growthRate.toFixed(2)}%`,
  };
}

// 生成内存测试报告
function generateMemoryReport(testDuration) {
  const report = {
    timestamp: new Date().toISOString(),
    config: MEMORY_CONFIG,
    testDuration,
    samples: memorySamples,
    statistics: {},
    leakDetection: {},
    summary: {},
  };

  if (memorySamples.length > 0) {
    const browserMemory = memorySamples
      .filter(s => s.browser)
      .map(s => s.browser.usedJSHeapSize);
    
    if (browserMemory.length > 0) {
      const memoryMB = browserMemory.map(bytes => bytes / 1024 / 1024);
      
      report.statistics.browser = {
        minMB: Math.min(...memoryMB),
        maxMB: Math.max(...memoryMB),
        averageMB: memoryMB.reduce((a, b) => a + b, 0) / memoryMB.length,
        currentMB: memoryMB[memoryMB.length - 1],
        samples: browserMemory.length,
      };
      
      report.summary.memoryTargetMet = report.statistics.browser.averageMB <= MEMORY_CONFIG.MEMORY_TARGET_MB;
      report.summary.memoryPerformance = 
        report.statistics.browser.averageMB <= MEMORY_CONFIG.MEMORY_TARGET_MB ? 'excellent' :
        report.statistics.browser.averageMB <= MEMORY_CONFIG.MEMORY_TARGET_MB * 1.5 ? 'good' :
        'needs_improvement';
    }
  }

  // 内存泄漏检测
  report.leakDetection = detectMemoryLeaks();
  report.summary.hasMemoryLeaks = report.leakDetection.hasLeaks;

  return report;
}

// 保存内存报告
function saveMemoryReport(report) {
  const reportDir = path.join(__dirname, '../reports');
  if (!fs.existsSync(reportDir)) {
    fs.mkdirSync(reportDir, { recursive: true });
  }

  const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
  const reportFile = path.join(reportDir, `memory-usage-${timestamp}.json`);
  
  fs.writeFileSync(reportFile, JSON.stringify(report, null, 2));
  console.log(`内存测试报告已保存到: ${reportFile}`);
  
  return reportFile;
}

// 主内存测试函数
async function runMemoryUsageTests() {
  console.log('开始内存使用监控测试...');
  console.log(`测试持续时间: ${MEMORY_CONFIG.TEST_DURATION_MINUTES} 分钟`);
  console.log(`采样间隔: ${MEMORY_CONFIG.SAMPLE_INTERVAL_MS}ms`);
  console.log(`内存使用目标: ${MEMORY_CONFIG.MEMORY_TARGET_MB}MB`);
  
  let browser;
  let page;
  let monitoringInterval;
  
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

    page = await browser.newPage();
    
    // 导航到应用
    await page.goto(MEMORY_CONFIG.APP_URL, { waitUntil: 'domcontentloaded' });
    
    console.log('应用加载完成，开始内存监控...');
    isMonitoring = true;
    
    // 开始内存监控
    monitoringInterval = setInterval(() => {
      monitorMemoryUsage(page).catch(console.error);
    }, MEMORY_CONFIG.SAMPLE_INTERVAL_MS);
    
    // 在测试期间执行各种操作
    const testStartTime = Date.now();
    const testEndTime = testStartTime + (MEMORY_CONFIG.TEST_DURATION_MINUTES * 60 * 1000);
    
    while (Date.now() < testEndTime) {
      try {
        await performMemoryIntensiveOperations(page);
        await new Promise(resolve => setTimeout(resolve, 5000)); // 等待5秒
      } catch (error) {
        console.warn('执行内存密集型操作时出错:', error.message);
      }
    }
    
    // 停止监控
    isMonitoring = false;
    if (monitoringInterval) {
      clearInterval(monitoringInterval);
    }
    
    const testDuration = Date.now() - testStartTime;
    
    // 生成报告
    const report = generateMemoryReport(testDuration);
    const reportFile = saveMemoryReport(report);

    // 输出结果摘要
    console.log('\n=== 内存使用测试结果 ===');
    console.log(`测试持续时间: ${(testDuration / 1000).toFixed(2)} 秒`);
    console.log(`样本数量: ${memorySamples.length}`);
    
    if (report.statistics.browser) {
      console.log(`平均内存使用: ${report.statistics.browser.averageMB.toFixed(2)}MB`);
      console.log(`最小内存使用: ${report.statistics.browser.minMB.toFixed(2)}MB`);
      console.log(`最大内存使用: ${report.statistics.browser.maxMB.toFixed(2)}MB`);
      console.log(`当前内存使用: ${report.statistics.browser.currentMB.toFixed(2)}MB`);
      console.log(`内存目标达成: ${report.summary.memoryTargetMet ? '✅' : '❌'}`);
      console.log(`内存性能评级: ${report.summary.memoryPerformance || 'N/A'}`);
    }
    
    console.log(`内存泄漏检测: ${report.summary.hasMemoryLeaks ? '❌ 检测到泄漏' : '✅ 无泄漏'}`);
    console.log(report.leakDetection.analysis);

    return {
      success: true,
      report,
      reportFile,
    };

  } catch (error) {
    console.error('内存测试失败:', error);
    return {
      success: false,
      error: error.message,
    };
  } finally {
    isMonitoring = false;
    if (monitoringInterval) {
      clearInterval(monitoringInterval);
    }
    if (page) {
      await page.close();
    }
    if (browser) {
      await browser.close();
    }
  }
}

// 如果直接运行此文件
if (require.main === module) {
  runMemoryUsageTests()
    .then((result) => {
      if (result.success) {
        console.log('\n✅ 内存使用测试完成');
        process.exit(0);
      } else {
        console.log('\n❌ 内存使用测试失败');
        process.exit(1);
      }
    })
    .catch((error) => {
      console.error('测试执行错误:', error);
      process.exit(1);
    });
}

module.exports = {
  runMemoryUsageTests,
  MEMORY_CONFIG,
  memorySamples,
};