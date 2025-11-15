// ClaudeProviders页面集成测试
// 测试完整的用户工作流程：页面加载→数据获取→表单提交→状态更新→错误处理

import React from 'react';
import { screen, waitFor, fireEvent, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { rest } from 'msw';
import { server } from '../mocks/server';
import { 
  renderWithProviders, 
  createMockClaudeProvider,
  createMockPagedResult,
  assertLoadingState,
  assertErrorState
} from '../utils/testUtils';
import ClaudeProviders from '../../../src/pages/ClaudeProviders';

// Mock导入的组件
jest.mock('../../../src/components/common/Button', () => ({
  Button: ({ children, onClick, variant, disabled, ...props }: any) => (
    <button 
      onClick={onClick} 
      data-variant={variant} 
      disabled={disabled}
      className={`btn btn-${variant || 'primary'}`}
      {...props}
    >
      {children}
    </button>
  ),
}));

jest.mock('../../../src/components/common/Card', () => ({
  Card: ({ children, className }: any) => (
    <div className={className || 'card'} data-testid="card">
      {children}
    </div>
  ),
  Card: Object.assign(
    ({ children, className }: any) => (
      <div className={className || 'card'} data-testid="card">
        {children}
      </div>
    ),
    {
      Header: ({ children }: any) => <div className="card-header">{children}</div>,
      Title: ({ children }: any) => <div className="card-title">{children}</div>,
      Content: ({ children }: any) => <div className="card-content">{children}</div>,
      Footer: ({ children }: any) => <div className="card-footer">{children}</div>,
    }
  ),
}));

jest.mock('../../../src/components/common/Input', () => ({
  Input: ({ onChange, value, placeholder, error, ...props }: any) => (
    <input 
      onChange={onChange} 
      value={value} 
      placeholder={placeholder}
      data-error={error}
      {...props}
    />
  ),
}));

describe('ClaudeProviders集成测试', () => {
  const user = userEvent.setup();

  beforeEach(() => {
    // 重置所有mocks
    jest.clearAllMocks();
  });

  describe('页面加载和初始数据获取', () => {
    it('应该成功加载页面并显示Claude供应商列表', async () => {
      const mockProviders = [
        createMockClaudeProvider({ id: 1, name: 'Claude Provider 1', enabled: 1 }),
        createMockClaudeProvider({ id: 2, name: 'Claude Provider 2', enabled: 0 }),
      ];

      const mockResponse = createMockPagedResult({
        items: mockProviders,
        total: mockProviders.length,
      });

      // Mock API响应
      server.use(
        rest.get('/api/claude-providers', (req, res, ctx) => {
          return res(ctx.status(200), ctx.json(mockResponse));
        })
      );

      renderWithProviders(<ClaudeProviders />);

      // 验证页面标题
      expect(screen.getByText('Claude 供应商')).toBeInTheDocument();

      // 验证加载状态
      assertLoadingState();

      // 等待数据加载完成
      await waitFor(() => {
        expect(screen.getByText('Claude Provider 1')).toBeInTheDocument();
        expect(screen.getByText('Claude Provider 2')).toBeInTheDocument();
      });

      // 验证表格结构
      const table = screen.getByRole('table');
      expect(table).toBeInTheDocument();
      
      // 验证数据行数
      const rows = within(table).getAllByRole('row');
      expect(rows.length).toBe(mockProviders.length + 1); // +1 for header row
    });

    it('应该处理空数据状态', async () => {
      const mockResponse = createMockPagedResult({
        items: [],
        total: 0,
      });

      server.use(
        rest.get('/api/claude-providers', (req, res, ctx) => {
          return res(ctx.status(200), ctx.json(mockResponse));
        })
      );

      renderWithProviders(<ClaudeProviders />);

      await waitFor(() => {
        expect(screen.getByText(/暂无数据/)).toBeInTheDocument();
      });
    });

    it('应该处理API错误', async () => {
      server.use(
        rest.get('/api/claude-providers', (req, res, ctx) => {
          return res(
            ctx.status(500),
            ctx.json({ error: 'Internal server error' })
          );
        })
      );

      renderWithProviders(<ClaudeProviders />);

      await waitFor(() => {
        assertErrorState('Internal server error');
      });
    });

    it('应该处理网络超时', async () => {
      server.use(
        rest.get('/api/claude-providers', (req, res, ctx) => {
          return res(
            ctx.delay(6000), // 超过默认超时时间
            ctx.status(200),
            ctx.json({})
          );
        })
      );

      renderWithProviders(<ClaudeProviders />);

      await waitFor(() => {
        assertErrorState('请求超时');
      }, { timeout: 10000 });
    });
  });

  describe('创建新的Claude供应商', () => {
    it('应该成功创建新的Claude供应商', async () => {
      const newProvider = createMockClaudeProvider({
        name: 'New Claude Provider',
        url: 'https://api.new-provider.com',
        token: 'sk-new-token',
      });

      const existingProviders = [
        createMockClaudeProvider({ id: 1, name: 'Existing Provider' }),
      ];

      // Mock初始数据
      server.use(
        rest.get('/api/claude-providers', (req, res, ctx) => {
          const response = createMockPagedResult({
            items: existingProviders,
            total: existingProviders.length,
          });
          return res(ctx.status(200), ctx.json(response));
        }),

        // Mock创建API
        rest.post('/api/claude-providers', (req, res, ctx) => {
          return res(ctx.status(201), ctx.json({ id: newProvider.id }));
        }),

        // Mock创建后的数据获取
        rest.get('/api/claude-providers', (req, res, ctx) => {
          const response = createMockPagedResult({
            items: [...existingProviders, newProvider],
            total: existingProviders.length + 1,
          });
          return res(ctx.status(200), ctx.json(response));
        })
      );

      renderWithProviders(<ClaudeProviders />);

      // 等待页面加载
      await waitFor(() => {
        expect(screen.getByText('Existing Provider')).toBeInTheDocument();
      });

      // 点击添加按钮
      const addButton = screen.getByRole('button', { name: /添加|新建/ });
      await user.click(addButton);

      // 验证模态框打开
      await waitFor(() => {
        expect(screen.getByText(/创建.*供应商/)).toBeInTheDocument();
      });

      // 填写表单
      const nameInput = screen.getByLabelText(/名称/);
      const urlInput = screen.getByLabelText(/URL/);
      const tokenInput = screen.getByLabelText(/Token/);

      await user.type(nameInput, newProvider.name);
      await user.type(urlInput, newProvider.url);
      await user.type(tokenInput, newProvider.token);

      // 提交表单
      const submitButton = screen.getByRole('button', { name: /确认|保存/ });
      await user.click(submitButton);

      // 验证创建成功
      await waitFor(() => {
        expect(screen.getByText(newProvider.name)).toBeInTheDocument();
      });

      // 验证模态框关闭
      expect(screen.queryByText(/创建.*供应商/)).not.toBeInTheDocument();
    });

    it('应该验证表单输入', async () => {
      renderWithProviders(<ClaudeProviders />);

      // 打开创建模态框
      const addButton = screen.getByRole('button', { name: /添加|新建/ });
      await user.click(addButton);

      // 尝试提交空表单
      const submitButton = screen.getByRole('button', { name: /确认|保存/ });
      await user.click(submitButton);

      // 验证错误提示
      await waitFor(() => {
        expect(screen.getByText(/名称.*必填/)).toBeInTheDocument();
        expect(screen.getByText(/URL.*必填/)).toBeInTheDocument();
        expect(screen.getByText(/Token.*必填/)).toBeInTheDocument();
      });
    });

    it('应该处理创建失败', async () => {
      server.use(
        rest.get('/api/claude-providers', (req, res, ctx) => {
          const response = createMockPagedResult({ items: [], total: 0 });
          return res(ctx.status(200), ctx.json(response));
        }),

        rest.post('/api/claude-providers', (req, res, ctx) => {
          return res(
            ctx.status(400),
            ctx.json({ error: '创建失败：名称已存在' })
          );
        })
      );

      renderWithProviders(<ClaudeProviders />);

      // 打开创建模态框
      const addButton = screen.getByRole('button', { name: /添加|新建/ });
      await user.click(addButton);

      // 填写表单
      const nameInput = screen.getByLabelText(/名称/);
      await user.type(nameInput, 'Duplicate Name');

      // 提交表单
      const submitButton = screen.getByRole('button', { name: /确认|保存/ });
      await user.click(submitButton);

      // 验证错误显示
      await waitFor(() => {
        expect(screen.getByText('创建失败：名称已存在')).toBeInTheDocument();
      });
    });
  });

  describe('编辑Claude供应商', () => {
    it('应该成功编辑现有供应商', async () => {
      const existingProvider = createMockClaudeProvider({
        id: 1,
        name: 'Original Name',
        url: 'https://original-url.com',
      });

      const updatedProvider = {
        ...existingProvider,
        name: 'Updated Name',
        url: 'https://updated-url.com',
      };

      server.use(
        rest.get('/api/claude-providers', (req, res, ctx) => {
          const response = createMockPagedResult({
            items: [existingProvider],
            total: 1,
          });
          return res(ctx.status(200), ctx.json(response));
        }),

        rest.put('/api/claude-providers/1', (req, res, ctx) => {
          return res(ctx.status(200), ctx.json({ success: true }));
        }),

        rest.get('/api/claude-providers', (req, res, ctx) => {
          const response = createMockPagedResult({
            items: [updatedProvider],
            total: 1,
          });
          return res(ctx.status(200), ctx.json(response));
        })
      );

      renderWithProviders(<ClaudeProviders />);

      // 等待数据显示
      await waitFor(() => {
        expect(screen.getByText('Original Name')).toBeInTheDocument();
      });

      // 点击编辑按钮
      const editButton = screen.getByRole('button', { name: /编辑/ });
      await user.click(editButton);

      // 验证编辑模态框打开并预填充数据
      await waitFor(() => {
        expect(screen.getByText(/编辑.*供应商/)).toBeInTheDocument();
        
        const nameInput = screen.getByLabelText(/名称/) as HTMLInputElement;
        const urlInput = screen.getByLabelText(/URL/) as HTMLInputElement;
        
        expect(nameInput.value).toBe('Original Name');
        expect(urlInput.value).toBe('https://original-url.com');
      });

      // 修改数据
      const nameInput = screen.getByLabelText(/名称/);
      const urlInput = screen.getByLabelText(/URL/);

      await user.clear(nameInput);
      await user.type(nameInput, 'Updated Name');

      await user.clear(urlInput);
      await user.type(urlInput, 'https://updated-url.com');

      // 提交修改
      const submitButton = screen.getByRole('button', { name: /保存|更新/ });
      await user.click(submitButton);

      // 验证更新成功
      await waitFor(() => {
        expect(screen.getByText('Updated Name')).toBeInTheDocument();
        expect(screen.getByText('https://updated-url.com')).toBeInTheDocument();
      });
    });
  });

  describe('删除Claude供应商', () => {
    it('应该成功删除供应商', async () => {
      const providerToDelete = createMockClaudeProvider({
        id: 1,
        name: 'Provider To Delete',
      });

      server.use(
        rest.get('/api/claude-providers', (req, res, ctx) => {
          const response = createMockPagedResult({
            items: [providerToDelete],
            total: 1,
          });
          return res(ctx.status(200), ctx.json(response));
        }),

        rest.delete('/api/claude-providers/1', (req, res, ctx) => {
          return res(ctx.status(200), ctx.json({ success: true }));
        }),

        rest.get('/api/claude-providers', (req, res, ctx) => {
          const response = createMockPagedResult({ items: [], total: 0 });
          return res(ctx.status(200), ctx.json(response));
        })
      );

      renderWithProviders(<ClaudeProviders />);

      // 等待数据显示
      await waitFor(() => {
        expect(screen.getByText('Provider To Delete')).toBeInTheDocument();
      });

      // 点击删除按钮
      const deleteButton = screen.getByRole('button', { name: /删除/ });
      await user.click(deleteButton);

      // 确认删除
      await waitFor(() => {
        expect(screen.getByText(/确认删除/)).toBeInTheDocument();
      });

      const confirmButton = screen.getByRole('button', { name: /确认/ });
      await user.click(confirmButton);

      // 验证删除成功
      await waitFor(() => {
        expect(screen.queryByText('Provider To Delete')).not.toBeInTheDocument();
        expect(screen.getByText(/暂无数据/)).toBeInTheDocument();
      });
    });
  });

  describe('测试供应商连接', () => {
    it('应该成功测试连接', async () => {
      const provider = createMockClaudeProvider({ id: 1 });

      server.use(
        rest.get('/api/claude-providers', (req, res, ctx) => {
          const response = createMockPagedResult({ items: [provider], total: 1 });
          return res(ctx.status(200), ctx.json(response));
        }),

        rest.get('/api/claude-providers/1/test', (req, res, ctx) => {
          return res(
            ctx.status(200),
            ctx.json({ success: true, message: 'Connection test successful' })
          );
        })
      );

      renderWithProviders(<ClaudeProviders />);

      await waitFor(() => {
        expect(screen.getByText(provider.name)).toBeInTheDocument();
      });

      // 点击测试连接按钮
      const testButton = screen.getByRole('button', { name: /测试连接/ });
      await user.click(testButton);

      // 验证成功消息
      await waitFor(() => {
        expect(screen.getByText('Connection test successful')).toBeInTheDocument();
      });
    });

    it('应该处理连接测试失败', async () => {
      const provider = createMockClaudeProvider({ id: 1 });

      server.use(
        rest.get('/api/claude-providers', (req, res, ctx) => {
          const response = createMockPagedResult({ items: [provider], total: 1 });
          return res(ctx.status(200), ctx.json(response));
        }),

        rest.get('/api/claude-providers/1/test', (req, res, ctx) => {
          return res(
            ctx.status(400),
            ctx.json({ success: false, message: 'Connection failed' })
          );
        })
      );

      renderWithProviders(<ClaudeProviders />);

      await waitFor(() => {
        expect(screen.getByText(provider.name)).toBeInTheDocument();
      });

      // 点击测试连接按钮
      const testButton = screen.getByRole('button', { name: /测试连接/ });
      await user.click(testButton);

      // 验证失败消息
      await waitFor(() => {
        expect(screen.getByText('Connection failed')).toBeInTheDocument();
      });
    });
  });

  describe('搜索和过滤', () => {
    it('应该支持按名称搜索供应商', async () => {
      const providers = [
        createMockClaudeProvider({ id: 1, name: 'Claude Provider Alpha' }),
        createMockClaudeProvider({ id: 2, name: 'Claude Provider Beta' }),
        createMockClaudeProvider({ id: 3, name: 'Other Provider' }),
      ];

      server.use(
        rest.get('/api/claude-providers', (req, res, ctx) => {
          const search = req.url.searchParams.get('search');
          let filteredProviders = providers;

          if (search) {
            filteredProviders = providers.filter(p =>
              p.name.toLowerCase().includes(search.toLowerCase())
            );
          }

          const response = createMockPagedResult({
            items: filteredProviders,
            total: filteredProviders.length,
          });
          return res(ctx.status(200), ctx.json(response));
        })
      );

      renderWithProviders(<ClaudeProviders />);

      // 等待初始数据加载
      await waitFor(() => {
        expect(screen.getByText('Claude Provider Alpha')).toBeInTheDocument();
        expect(screen.getByText('Claude Provider Beta')).toBeInTheDocument();
        expect(screen.getByText('Other Provider')).toBeInTheDocument();
      });

      // 搜索 "Claude"
      const searchInput = screen.getByPlaceholderText(/搜索/);
      await user.type(searchInput, 'Claude');

      // 验证搜索结果
      await waitFor(() => {
        expect(screen.getByText('Claude Provider Alpha')).toBeInTheDocument();
        expect(screen.getByText('Claude Provider Beta')).toBeInTheDocument();
        expect(screen.queryByText('Other Provider')).not.toBeInTheDocument();
      });
    });
  });

  describe('状态管理测试', () => {
    it('应该正确管理加载状态', async () => {
      server.use(
        rest.get('/api/claude-providers', (req, res, ctx) => {
          // 延迟响应以测试加载状态
          return res(
            ctx.delay(1000),
            ctx.status(200),
            ctx.json(createMockPagedResult({ items: [], total: 0 }))
          );
        })
      );

      renderWithProviders(<ClaudeProviders />);

      // 验证初始加载状态
      expect(screen.getByText(/加载中/)).toBeInTheDocument();

      // 等待加载完成
      await waitFor(() => {
        expect(screen.queryByText(/加载中/)).not.toBeInTheDocument();
      }, { timeout: 2000 });
    });

    it('应该正确管理错误状态', async () => {
      server.use(
        rest.get('/api/claude-providers', (req, res, ctx) => {
          return res(
            ctx.status(500),
            ctx.json({ error: 'API Error' })
          );
        })
      );

      renderWithProviders(<ClaudeProviders />);

      await waitFor(() => {
        expect(screen.getByText(/API Error/)).toBeInTheDocument();
      });
    });
  });
});