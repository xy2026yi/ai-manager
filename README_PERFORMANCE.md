# 性能基准测试框架

本文档描述了AI Manager项目的完整性能基准测试体系，包括启动时间、内存占用、API响应时间和并发处理能力的全面性能监控。

## 性能目标

基于项目迁移要求，我们设定了以下性能基准目标：

- **启动时间**: < 2秒
- **内存占用**: < 100MB
- **API响应时间**: < 500ms
- **应用体积**: < 15MB（相比Python方案减小90%+）
- **测试覆盖率**: > 80%

## 性能测试架构

### 1. Rust后端性能测试

**位置**: `src-tauri/benches/`

使用Criterion库进行Rust代码的性能基准测试：

#### api_performance.rs
- Claude供应商创建/查询性能
- 数据库连接池性能
- 加密解密操作性能
- JSON序列化/反序列化性能
- 并发请求处理性能
- 内存分配性能

#### database_performance.rs
- 批量数据插入性能
- 分页查询性能
- 搜索查询性能
- 数据库连接池压力测试
- 事务处理性能
- 复杂查询性能
- 索引查询性能

#### crypto_performance.rs
- 不同数据规模的加密/解密性能
- 批量加密操作性能
- 并发加密操作性能
- 不同密钥长度的性能影响
- 内存使用效率测试
- 错误处理性能开销

### 2. 前端性能测试

**位置**: `tests/performance/`

使用Puppeteer进行浏览器端性能测试：

#### startup.test.js
- 应用启动到完全可用的时间测试
- 关键元素加载时间监控
- API连接状态验证
- 启动过程事件监控
- 多次启动测试稳定性
- 内存使用情况统计

#### memory.test.js
- 实时内存使用监控
- 内存泄漏检测
- 内存使用趋势分析
- 内存密集型操作测试
- 长时间运行稳定性测试
- 内存使用模式分析

#### api_response.test.js
- 所有API端点的响应时间测试
- 并发请求处理能力测试
- API成功率统计
- 负载测试（逐步增加并发量）
- 吞吐量测试
- 错误处理性能测试

## 使用指南

### 运行Rust性能基准测试

```bash
# 运行所有基准测试
cargo bench

# 运行特定的基准测试
cargo bench api_performance
cargo bench database_performance
cargo bench crypto_performance

# 生成HTML报告
cargo bench -- --output-format html
```

### 运行前端性能测试

```bash
# 首先安装依赖
npm install puppeteer

# 运行启动性能测试
node tests/performance/startup.test.js

# 运行内存使用测试
node tests/performance/memory.test.js

# 运行API性能测试
node tests/performance/api_response.test.js
```

### 运行完整性能测试套件

```bash
# 运行综合性能测试
node scripts/run-performance-tests.js
```

## 性能监控指标

### 1. 启动性能指标

- **冷启动时间**: 应用首次启动到完全可用的时间
- **热启动时间**: 重新启动应用的时间
- **关键元素加载时间**: 导航栏、主内容区域等的加载时间
- **API连接时间**: 后端服务连接建立时间

### 2. 内存性能指标

- **初始内存占用**: 应用启动后的内存使用量
- **峰值内存使用**: 运行过程中的最大内存使用量
- **内存增长率**: 长时间运行时的内存增长趋势
- **内存泄漏检测**: 监控是否有未释放的内存

### 3. API性能指标

- **响应时间**: 各个API端点的平均响应时间
- **并发处理能力**: 同时处理的请求数量
- **成功率**: API调用的成功百分比
- **吞吐量**: 每秒处理的请求数量

### 4. 用户体验指标

- **首屏渲染时间**: 页面首次内容渲染的时间
- **交互响应时间**: 用户操作到系统响应的时间
- **页面切换时间**: 不同页面间的切换延迟

## 性能报告

### 报告结构

每次测试运行后，系统会生成详细的性能报告，包含：

```json
{
  "timestamp": "2024-01-01T00:00:00.000Z",
  "config": {
    "STARTUP_TIME_TARGET": 2000,
    "MEMORY_TARGET_MB": 100,
    "API_RESPONSE_TIME_TARGET": 500
  },
  "testResults": {
    "startup": { /* 启动测试结果 */ },
    "memory": { /* 内存测试结果 */ },
    "api": { /* API测试结果 */ }
  },
  "statistics": {
    "overallScore": 85,
    "successfulTests": 3,
    "totalTests": 3
  },
  "recommendations": [
    {
      "category": "启动性能",
      "priority": "high",
      "description": "启动时间超过目标值",
      "suggestions": ["优化初始化流程", "实现懒加载"]
    }
  ]
}
```

### 报告存储

报告自动保存在 `reports/` 目录下：

- `startup-performance-{timestamp}.json`: 启动性能报告
- `memory-usage-{timestamp}.json`: 内存使用报告
- `api-performance-{timestamp}.json`: API性能报告
- `comprehensive-performance-{timestamp}.json`: 综合性能报告

### 性能评分系统

系统会根据测试结果自动计算性能评分：

- **90-100分**: A+ (优秀) - 性能表现优异，满足所有目标
- **80-89分**: B+ (良好) - 性能表现良好，大部分目标达成
- **70-79分**: C+ (一般) - 性能表现一般，需要部分优化
- **60-69分**: D+ (需要改进) - 性能不达标，需要重点优化
- **0-59分**: F (不合格) - 性能严重不达标，需要紧急优化

## 持续集成集成

### GitHub Actions配置

```yaml
name: Performance Tests
on: [push, pull_request]

jobs:
  performance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Setup Node.js
        uses: actions/setup-node@v2
        with:
          node-version: '18'
      - name: Install dependencies
        run: |
          npm install
          cargo install cargo-criterion
      - name: Run Rust benchmarks
        run: cargo bench -- --output-format json > benchmark-results.json
      - name: Run frontend performance tests
        run: node scripts/run-performance-tests.js
      - name: Upload performance reports
        uses: actions/upload-artifact@v2
        with:
          name: performance-reports
          path: reports/
```

### 性能回归检测

设置性能基准，当性能下降超过阈值时自动告警：

- 启动时间增加超过20%
- 内存使用增加超过15%
- API响应时间增加超过25%
- 成功率下降到95%以下

## 性能优化建议

### 1. 启动性能优化

- **代码分割**: 实现按需加载，减少初始加载量
- **预加载**: 关键资源预加载，提升用户体验
- **缓存策略**: 合理使用缓存，减少重复计算
- **数据库优化**: 优化启动时的数据库查询

### 2. 内存使用优化

- **对象池**: 复用对象，减少垃圾回收压力
- **内存泄漏检测**: 定期检查和修复内存泄漏
- **数据结构优化**: 选择合适的数据结构，减少内存占用
- **图片优化**: 压缩和优化图片资源

### 3. API性能优化

- **查询优化**: 优化数据库查询，添加适当索引
- **缓存机制**: 实现API响应缓存
- **连接池**: 优化数据库连接池配置
- **异步处理**: 使用异步操作，提升并发能力

### 4. 前端性能优化

- **代码压缩**: 压缩JavaScript和CSS代码
- **图片优化**: 使用现代图片格式和压缩
- **懒加载**: 实现组件和资源的懒加载
- **Bundle优化**: 优化Webpack打包配置

## 故障排除

### 常见问题

1. **测试环境问题**
   - 确保应用在 `localhost:3000` 运行
   - 确保API服务器在 `localhost:8080` 运行
   - 检查网络连接和防火墙设置

2. **Puppeteer问题**
   - 安装问题：使用 `npm install puppeteer --ignore-scripts`
   - 权限问题：确保有足够权限启动无头浏览器
   - 依赖问题：安装所需的系统依赖

3. **内存测试不准确**
   - 关闭其他占用内存的应用程序
   - 多次运行测试取平均值
   - 检查系统内存使用情况

4. **API测试超时**
   - 检查API服务器是否正常运行
   - 增加超时时间配置
   - 验证网络连接稳定性

### 调试技巧

1. **启用详细日志**
   ```javascript
   // 在测试文件中启用调试
   const DEBUG = true;
   if (DEBUG) console.log('调试信息');
   ```

2. **单独运行测试**
   ```bash
   # 只运行启动测试
   node tests/performance/startup.test.js
   ```

3. **检查测试环境**
   ```javascript
   // 检查应用是否可用
   await page.goto('http://localhost:3000', { timeout: 5000 });
   ```

## 最佳实践

### 1. 测试环境准备
- 使用专用的测试环境
- 确保测试数据的可重复性
- 定期清理测试产生的临时数据

### 2. 测试执行
- 在相同环境下运行多次测试
- 记录测试时的系统状态
- 定期更新性能基准值

### 3. 结果分析
- 关注趋势而非单次结果
- 对比历史数据识别性能回归
- 结合用户反馈验证性能改进效果

### 4. 持续改进
- 定期更新测试用例
- 优化测试执行效率
- 扩展性能监控范围

## 相关资源

- [Criterion.rs文档](https://bheisler.github.io/criterion.rs/book/)
- [Puppeteer文档](https://pptr.dev/)
- [Web性能测试最佳实践](https://web.dev/performance/)
- [Tauri性能优化指南](https://tauri.app/v1/guides/performance/)

## 维护说明

定期维护任务：
- 更新性能基准值以反映当前硬件水平
- 清理旧的性能报告文件
- 优化测试脚本的执行效率
- 检查和更新依赖项版本
- 扩展新的性能测试场景