// 前端集成测试工具函数
// 提供通用的测试辅助函数和Mock工具

import React, { ReactElement } from 'react';
import { render, RenderOptions, RenderResult } from '@testing-library/react';
import { screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Provider } from 'jotai';
import { themeAtom, notificationsAtom } from '../../../src/stores';

// 测试组件渲染工具
export const renderWithProviders = (
  ui: ReactElement,
  options?: RenderOptions
): RenderResult => {
  return render(
    <Provider>
      {ui}
    </Provider>,
    options
  );
};

// 创建用户事件实例
export const createUserEvent = (options?: { delay?: number }) => {
  return userEvent.setup({
    ...options,
  });
};

// 测试数据生成器
export const createMockClaudeProvider = (overrides = {}) => ({
  id: 1,
  name: '测试Claude供应商',
  url: 'https://api.anthropic.com',
  token: 'sk-test-key',
  model: 'claude-3-sonnet-20240229',
  timeout: 30000,
  max_retries: 3,
  retry_delay: 1,
  auto_update: 1,
  opus_model: 'claude-3-opus-20240229',
  sonnet_model: 'claude-3-sonnet-20241022',
  haiku_model: 'claude-3-haiku-20240307',
  is_enabled: 1,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
  ...overrides
});

export const createMockCodexProvider = (overrides = {}) => ({
  id: 1,
  name: '测试Codex供应商',
  url: 'https://api.openai.com',
  token: 'sk-test-key',
  type: 'gpt-4',
  timeout: 60000,
  is_enabled: 1,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
  ...overrides
});

export const createMockAgentGuide = (overrides = {}) => ({
  id: 1,
  name: '测试指导文件',
  description: '这是一个测试指导文件',
  content: '# 测试指导文件\n\n这是测试内容。',
  version: '1.0.0',
  author: '测试作者',
  guide_type: '通用',
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
  ...overrides
});

export const createMockMcpServer = (overrides = {}) => ({
  id: 1,
  name: '测试MCP服务器',
  description: '这是一个测试MCP服务器',
  command: 'node',
  args: ['server.js'],
  env: { 'NODE_ENV': 'test' },
  cwd: '/workspace',
  connection_type: 'stdio',
  timeout: 60000,
  is_enabled: 1,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
  ...overrides
});

export const createMockCommonConfig = (overrides = {}) => ({
  id: 1,
  key: 'test.config',
  value: 'test-value',
  description: '测试配置',
  category: '测试类别',
  data_type: 'string',
  is_encrypted: 0,
  is_enabled: 1,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
  ...overrides
});

// Mock API响应数据
export const createMockApiResponse = (data, success = true, message = '操作成功') => ({
  success,
  data,
  message,
  timestamp: new Date().toISOString()
});

export const createMockPagedResult = (data, total = data.length, page = 1, limit = 20) => ({
  data,
  total,
  page,
  limit,
  total_pages: Math.ceil(total / limit)
});

// Mock API服务
export class MockApiService {
  private responses: Map<string, any>;

  constructor() {
    this.responses = new Map();
  }

  setResponse(key: string, response: any) {
    this.responses.set(key, response);
  }

  getResponse(key: string) {
    return this.responses.get(key);
  }

  clear() {
    this.responses.clear();
  }
}

// 创建全局Mock API服务实例
export const mockApiService = new MockApiService();

// 表单测试工具
export const fillForm = async (
  formFields: Record<string, string>,
  userEvent: ReturnType<typeof createUserEvent>
) => {
  for (const [selector, value] of Object.entries(formFields)) {
    const element = screen.getByLabelText(selector) || screen.getByPlaceholderText(selector);
    await userEvent.clear(element);
    await userEvent.type(element, value);
  }
};

// 模拟表单提交
export const submitForm = async (submitButtonText: string) => {
  const submitButton = screen.getByRole('button', { name: submitButtonText });
  await userEvent.click(submitButton);
};

// 等待API响应的工具
export const waitForApiResponse = async (timeout = 5000) => {
  await waitFor(() => {
    const errorElement = screen.queryByRole('alert');
    if (errorElement) {
      throw new Error(errorElement.textContent || '出现错误');
    }
  }, { timeout });
};

// 模拟网络延迟
export const delay = (ms: number): Promise<void> => {
  return new Promise(resolve => setTimeout(resolve, ms));
};

// 模拟异步操作
export const waitForAsync = async (condition: () => boolean, timeout = 5000): Promise<void> => {
  const startTime = Date.now();
  while (!condition() && Date.now() - startTime < timeout) {
    await delay(100);
  }
  if (!condition()) {
    throw new Error(`等待条件超时，等待时间: ${timeout}ms`);
  }
};

// 验证通知工具
export const expectNotification = (message: string, type = 'success') => {
  const notification = screen.getByText(message);
  expect(notification).toBeInTheDocument();
  
  if (type !== 'info') {
    const alertElement = screen.getByRole('alert');
    expect(alertElement).toHaveClass(`alert-${type}`);
  }
};

// 验证表单错误
export const expectFormError = (fieldLabel: string, errorMessage: string) => {
  const field = screen.getByLabelText(fieldLabel);
  const errorElement = field.closest('.form-field')?.querySelector('.error-message') ||
                     field.parentElement?.querySelector('.error-message');
  
  expect(errorElement).toBeInTheDocument();
  expect(errorElement).toHaveTextContent(errorMessage);
};

// 验证表格行数
export const expectTableRows = (rowCount: number) => {
  const rows = screen.getAllByRole('row');
  expect(rows.length).toBe(rowCount);
};

// 验证分页信息
export const expectPagination = (current: number, total: number, pageSize: number) => {
  expect(screen.getByText(`第 ${current} 页`)).toBeInTheDocument();
  expect(screen.getByText(`共 ${total} 条`)).toBeInTheDocument();
};

// 清理工具
export const cleanup = () => {
  jest.clearAllMocks();
  document.body.innerHTML = '';
};