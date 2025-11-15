// MSW Server Setup
// 为测试环境设置Mock Service Worker

import { setupServer } from 'msw/node';
import allHandlers from './handlers';

// 创建MSW服务器实例
export const server = setupServer(...allHandlers);

// 服务器配置
server.listen({
  onUnhandledRequest: 'warn',
});

// 测试生命周期钩子
beforeAll(() => {
  // 在所有测试开始前启动服务器
  server.listen({
    onUnhandledRequest: 'error',
  });
});

afterEach(() => {
  // 在每个测试后重置handlers
  server.resetHandlers();
});

afterAll(() => {
  // 在所有测试结束后关闭服务器
  server.close();
});

// 导出server实例，方便在测试中使用
export default server;