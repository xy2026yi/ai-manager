// Jest测试环境设置
// 配置测试环境的全局设置和Mock

import '@testing-library/jest-dom';
import { TextEncoder, TextDecoder } from 'util';

// 全局变量Mock
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: jest.fn().mockImplementation(query => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: jest.fn(),
    removeListener: jest.fn(),
    addEventListener: jest.fn(),
    removeEventListener: jest.fn(),
    dispatchEvent: jest.fn(),
  })),
});

Object.defineProperty(window, 'ResizeObserver', {
  writable: true,
  value: jest.fn().mockImplementation(() => ({
    observe: jest.fn(),
    unobserve: jest.fn(),
    disconnect: jest.fn(),
  })),
});

// Tauri API Mock
jest.mock('@tauri-apps/api/tauri', () => ({
  invoke: jest.fn(),
  listen: jest.fn(),
  emit: jest.fn(),
}));

// 设置fetch Mock
global.fetch = jest.fn();

// 环境变量Mock
process.env.NODE_ENV = 'test';

// Mock localStorage
const localStorageMock = {
  getItem: jest.fn(),
  setItem: jest.fn(),
  removeItem: jest.fn(),
  clear: jest.fn(),
};
global.localStorage = localStorageMock;

// Mock sessionStorage
const sessionStorageMock = {
  getItem: jest.fn(),
  setItem: jest.fn(),
  removeItem: jest.fn(),
  clear: jest.fn(),
};
global.sessionStorage = sessionStorageMock;

// Mock URLSearchParams
global.URLSearchParams = jest.fn(() => ({
  get: jest.fn(),
  set: jest.fn(),
  has: jest.fn(),
  delete: jest.fn(),
  toString: jest.fn(),
}));

// 全局错误处理
process.on('unhandledRejection', (reason, promise) => {
  console.error('Unhandled Rejection at:', promise, 'reason:', reason);
});

// 测试工具函数
global.createMockFetch = (responses, options = {}) => {
  let callCount = 0;
  return jest.fn().mockImplementation((url, init) => {
    const response = responses[callCount] || responses[responses.length - 1];
    callCount++;
    
    return Promise.resolve({
      ok: true,
      status: response.status || 200,
      json: () => Promise.resolve(response.data),
      headers: new Map(response.headers || {}),
      url,
      init,
    });
  });
};

// 清理函数
afterEach(() => {
  jest.clearAllMocks();
  localStorageMock.clear();
  sessionStorageMock.clear();
});