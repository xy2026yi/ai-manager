// MSW API Handlers
// 为前端集成测试提供完整的API mock支持

import { rest } from 'msw';
import { 
  createMockClaudeProvider, 
  createMockCodexProvider, 
  createMockAgentGuide, 
  createMockMcpServer, 
  createMockCommonConfig,
  createMockPagedResult 
} from '../utils/testUtils';

// Mock数据存储
let mockClaudeProviders = [
  createMockClaudeProvider({ id: 1, name: 'Test Claude 1', enabled: 1 }),
  createMockClaudeProvider({ id: 2, name: 'Test Claude 2', enabled: 0 }),
];

let mockCodexProviders = [
  createMockCodexProvider({ id: 1, name: 'Test Codex 1', enabled: 1 }),
];

let mockAgentGuides = [
  createMockAgentGuide({ id: 1, name: 'Test Guide 1' }),
];

let mockMcpServers = [
  createMockMcpServer({ id: 1, name: 'Test MCP Server 1', enabled: 1 }),
];

let mockCommonConfigs = [
  createMockCommonConfig({ id: 1, key: 'test_config', value: 'test_value' }),
];

// Claude Providers API Handlers
export const claudeProvidersHandlers = [
  // 获取Claude供应商列表
  rest.get('/api/claude-providers', (req, res, ctx) => {
    const page = parseInt(req.url.searchParams.get('page') || '1');
    const limit = parseInt(req.url.searchParams.get('limit') || '10');
    const search = req.url.searchParams.get('search');

    let filteredProviders = [...mockClaudeProviders];

    // 搜索过滤
    if (search) {
      filteredProviders = filteredProviders.filter(provider =>
        provider.name.toLowerCase().includes(search.toLowerCase())
      );
    }

    // 分页
    const startIndex = (page - 1) * limit;
    const endIndex = startIndex + limit;
    const paginatedProviders = filteredProviders.slice(startIndex, endIndex);

    const response = createMockPagedResult({
      items: paginatedProviders,
      total: filteredProviders.length,
      page,
      limit,
    });

    return res(
      ctx.status(200),
      ctx.json(response)
    );
  }),

  // 获取单个Claude供应商
  rest.get('/api/claude-providers/:id', (req, res, ctx) => {
    const id = parseInt(req.params.id as string);
    const provider = mockClaudeProviders.find(p => p.id === id);

    if (!provider) {
      return res(
        ctx.status(404),
        ctx.json({ error: 'Provider not found' })
      );
    }

    return res(
      ctx.status(200),
      ctx.json(provider)
    );
  }),

  // 创建Claude供应商
  rest.post('/api/claude-providers', async (req, res, ctx) => {
    const providerData = await req.json();
    
    const newProvider = createMockClaudeProvider({
      ...providerData,
      id: Math.max(...mockClaudeProviders.map(p => p.id)) + 1,
    });

    mockClaudeProviders.push(newProvider);

    return res(
      ctx.status(201),
      ctx.json({ id: newProvider.id })
    );
  }),

  // 更新Claude供应商
  rest.put('/api/claude-providers/:id', async (req, res, ctx) => {
    const id = parseInt(req.params.id as string);
    const updateData = await req.json();
    
    const index = mockClaudeProviders.findIndex(p => p.id === id);
    if (index === -1) {
      return res(
        ctx.status(404),
        ctx.json({ error: 'Provider not found' })
      );
    }

    mockClaudeProviders[index] = { ...mockClaudeProviders[index], ...updateData };

    return res(
      ctx.status(200),
      ctx.json({ success: true })
    );
  }),

  // 删除Claude供应商
  rest.delete('/api/claude-providers/:id', (req, res, ctx) => {
    const id = parseInt(req.params.id as string);
    const index = mockClaudeProviders.findIndex(p => p.id === id);
    
    if (index === -1) {
      return res(
        ctx.status(404),
        ctx.json({ error: 'Provider not found' })
      );
    }

    mockClaudeProviders.splice(index, 1);

    return res(
      ctx.status(200),
      ctx.json({ success: true })
    );
  }),

  // 测试Claude供应商连接
  rest.get('/api/claude-providers/:id/test', (req, res, ctx) => {
    const id = parseInt(req.params.id as string);
    const provider = mockClaudeProviders.find(p => p.id === id);

    if (!provider) {
      return res(
        ctx.status(404),
        ctx.json({ error: 'Provider not found' })
      );
    }

    // 模拟测试结果
    const success = Math.random() > 0.2; // 80% 成功率

    if (success) {
      return res(
        ctx.status(200),
        ctx.json({ success: true, message: 'Connection test successful' })
      );
    } else {
      return res(
        ctx.status(400),
        ctx.json({ success: false, message: 'Connection failed' })
      );
    }
  }),
];

// Codex Providers API Handlers
export const codexProvidersHandlers = [
  rest.get('/api/codex-providers', (req, res, ctx) => {
    const response = createMockPagedResult({
      items: mockCodexProviders,
      total: mockCodexProviders.length,
    });
    return res(ctx.status(200), ctx.json(response));
  }),

  rest.post('/api/codex-providers', async (req, res, ctx) => {
    const providerData = await req.json();
    const newProvider = createMockCodexProvider({
      ...providerData,
      id: Math.max(...mockCodexProviders.map(p => p.id)) + 1,
    });
    mockCodexProviders.push(newProvider);
    return res(ctx.status(201), ctx.json({ id: newProvider.id }));
  }),
];

// Agent Guides API Handlers
export const agentGuidesHandlers = [
  rest.get('/api/agent-guides', (req, res, ctx) => {
    const response = createMockPagedResult({
      items: mockAgentGuides,
      total: mockAgentGuides.length,
    });
    return res(ctx.status(200), ctx.json(response));
  }),

  rest.post('/api/agent-guides', async (req, res, ctx) => {
    const guideData = await req.json();
    const newGuide = createMockAgentGuide({
      ...guideData,
      id: Math.max(...mockAgentGuides.map(g => g.id)) + 1,
    });
    mockAgentGuides.push(newGuide);
    return res(ctx.status(201), ctx.json({ id: newGuide.id }));
  }),
];

// MCP Servers API Handlers
export const mcpServersHandlers = [
  rest.get('/api/mcp-servers', (req, res, ctx) => {
    const response = createMockPagedResult({
      items: mockMcpServers,
      total: mockMcpServers.length,
    });
    return res(ctx.status(200), ctx.json(response));
  }),

  rest.post('/api/mcp-servers', async (req, res, ctx) => {
    const serverData = await req.json();
    const newServer = createMockMcpServer({
      ...serverData,
      id: Math.max(...mockMcpServers.map(s => s.id)) + 1,
    });
    mockMcpServers.push(newServer);
    return res(ctx.status(201), ctx.json({ id: newServer.id }));
  }),
];

// Common Configs API Handlers
export const commonConfigsHandlers = [
  rest.get('/api/common-configs', (req, res, ctx) => {
    const response = createMockPagedResult({
      items: mockCommonConfigs,
      total: mockCommonConfigs.length,
    });
    return res(ctx.status(200), ctx.json(response));
  }),

  rest.post('/api/common-configs', async (req, res, ctx) => {
    const configData = await req.json();
    const newConfig = createMockCommonConfig({
      ...configData,
      id: Math.max(...mockCommonConfigs.map(c => c.id)) + 1,
    });
    mockCommonConfigs.push(newConfig);
    return res(ctx.status(201), ctx.json({ id: newConfig.id }));
  }),
];

// System API Handlers
export const systemHandlers = [
  // 系统信息
  rest.get('/api/system/info', (req, res, ctx) => {
    return res(
      ctx.status(200),
      ctx.json({
        version: '1.0.0-test',
        buildTime: '2024-01-01T00:00:00Z',
        environment: 'test',
        uptime: 3600,
      })
    );
  }),

  // 健康检查
  rest.get('/api/health', (req, res, ctx) => {
    return res(
      ctx.status(200),
      ctx.json({
        status: 'healthy',
        timestamp: new Date().toISOString(),
      })
    );
  }),
];

// 错误处理Handlers (用于测试错误场景)
export const errorHandlers = [
  // 500错误
  rest.get('/api/error/server-error', (req, res, ctx) => {
    return res(
      ctx.status(500),
      ctx.json({ error: 'Internal server error' })
    );
  }),

  // 404错误
  rest.get('/api/error/not-found', (req, res, ctx) => {
    return res(
      ctx.status(404),
      ctx.json({ error: 'Resource not found' })
    );
  }),

  // 网络超时
  rest.get('/api/error/timeout', (req, res, ctx) => {
    return res(
      ctx.delay(10000), // 10秒延迟模拟超时
      ctx.status(200),
      ctx.json({ message: 'This should timeout' })
    );
  }),
];

// 整合所有Handlers
export const allHandlers = [
  ...claudeProvidersHandlers,
  ...codexProvidersHandlers,
  ...agentGuidesHandlers,
  ...mcpServersHandlers,
  ...commonConfigsHandlers,
  ...systemHandlers,
  ...errorHandlers,
];

// 默认导出，方便在测试中使用
export default allHandlers;