// 测试运行器工具
// 提供测试执行和报告生成的辅助功能

import { configure } from '@testing-library/react';
import { server } from '../mocks/server';

// 配置Testing Library
configure({
  testIdAttribute: 'data-testid',
  asyncUtilTimeout: 5000,
});

// 全局测试设置
beforeAll(() => {
  // 设置全局测试环境
  process.env.NODE_ENV = 'test';
  
  // 启动MSW服务器
  server.listen({
    onUnhandledRequest: 'error',
  });
});

afterEach(() => {
  // 每个测试后重置状态
  server.resetHandlers();
  
  // 清理localStorage
  localStorage.clear();
  
  // 清理sessionStorage
  sessionStorage.clear();
  
  // 重置所有mocks
  jest.clearAllMocks();
});

afterAll(() => {
  // 所有测试结束后清理
  server.close();
});

// 测试工具函数
export const createTestSuite = (name: string, tests: () => void) => {
  describe(name, tests);
};

export const describeFeature = (feature: string, tests: () => void) => {
  describe(`功能: ${feature}`, tests);
};

export const describeWorkflow = (workflow: string, tests: () => void) => {
  describe(`工作流: ${workflow}`, tests);
};

// 性能测试工具
export const measurePerformance = async (name: string, fn: () => Promise<void>) => {
  const start = performance.now();
  await fn();
  const end = performance.now();
  console.log(`${name} 执行时间: ${end - start}ms`);
  return end - start;
};

// 覆盖率报告工具
export const generateCoverageReport = () => {
  console.log('生成测试覆盖率报告...');
  // 这里可以集成覆盖率报告工具
};

// 测试数据快照工具
export const createSnapshot = (name: string, data: any) => {
  console.log(`创建测试快照: ${name}`);
  // 这里可以实现数据快照功能
};