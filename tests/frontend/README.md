# 前端集成测试框架

本文档描述了AI Manager项目的前端集成测试框架，使用Jest + React Testing Library + MSW技术栈。

## 框架概述

### 技术栈
- **Jest**: 测试运行器和断言库
- **React Testing Library**: React组件测试工具
- **MSW (Mock Service Worker)**: API mock服务
- **Jotai**: 状态管理测试
- **TypeScript**: 类型安全的测试代码

### 测试策略
1. **组件级测试**: 测试单个组件的功能和交互
2. **页面级测试**: 测试完整页面的用户工作流
3. **集成测试**: 测试前端与后端API的交互
4. **工作流测试**: 测试跨页面的完整用户流程

## 目录结构

```
tests/frontend/
├── components/          # 组件测试
├── pages/              # 页面测试
├── integration/        # 集成测试
├── utils/              # 测试工具
├── mocks/              # API mocks
├── jest.config.js      # Jest配置
├── jest.setup.js       # Jest环境设置
└── README.md          # 本文档
```

## 核心功能

### 1. API Mock系统

**位置**: `tests/frontend/mocks/handlers.ts`

覆盖所有主要API端点：
- Claude供应商管理 (`/api/claude-providers/*`)
- Codex供应商管理 (`/api/codex-providers/*`)
- Agent指导管理 (`/api/agent-guides/*`)
- MCP服务器管理 (`/api/mcp-servers/*`)
- 通用配置管理 (`/api/common-configs/*`)
- 系统信息 (`/api/system/*`)

### 2. 测试工具库

**位置**: `tests/frontend/utils/testUtils.tsx`

提供以下工具：
- `renderWithProviders`: 带状态管理的组件渲染
- `createMock*`: Mock数据生成器
- `assertLoadingState`: 加载状态断言
- `assertErrorState`: 错误状态断言
- 表单测试工具
- 用户交互模拟工具

### 3. 测试配置

**Jest配置**: `tests/frontend/jest.config.js`
- TypeScript支持
- 模块路径映射
- 覆盖率阈值设置
- 测试文件模式匹配

**环境设置**: `tests/frontend/jest.setup.js`
- Tauri API mocks
- 浏览器环境API mocks
- 全局测试工具设置

## 使用指南

### 运行测试

```bash
# 运行所有前端测试
npm test -- tests/frontend

# 运行特定测试文件
npm test -- tests/frontend/pages/ClaudeProviders.test.tsx

# 运行测试并生成覆盖率报告
npm test -- --coverage tests/frontend

# 监视模式运行测试
npm test -- --watch tests/frontend
```

### 编写新测试

1. **组件测试示例**:

```typescript
import { render, screen } from '@testing-library/react';
import { renderWithProviders } from '../utils/testUtils';
import MyComponent from '../../../src/components/MyComponent';

describe('MyComponent', () => {
  it('应该正确渲染', () => {
    renderWithProviders(<MyComponent />);
    expect(screen.getByText('预期文本')).toBeInTheDocument();
  });
});
```

2. **页面集成测试示例**:

```typescript
import { screen, waitFor, fireEvent } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { server } from '../mocks/server';
import { rest } from 'msw';
import { renderWithProviders } from '../utils/testUtils';

describe('页面集成测试', () => {
  it('应该完成完整的用户工作流', async () => {
    // Mock API响应
    server.use(
      rest.get('/api/data', (req, res, ctx) => {
        return res(ctx.status(200), ctx.json({ data: 'test' }));
      })
    );

    renderWithProviders(<TestedPage />);
    
    // 测试用户交互
    const button = screen.getByRole('button');
    await userEvent.click(button);
    
    // 验证结果
    await waitFor(() => {
      expect(screen.getByText('预期结果')).toBeInTheDocument();
    });
  });
});
```

### Mock API数据

使用现有的Mock数据生成器：

```typescript
import { 
  createMockClaudeProvider,
  createMockPagedResult 
} from '../utils/testUtils';

const mockProvider = createMockClaudeProvider({
  name: 'Test Provider',
  enabled: 1,
});

const mockResponse = createMockPagedResult({
  items: [mockProvider],
  total: 1,
});
```

## 测试覆盖范围

### 已覆盖的测试场景

1. **Claude供应商管理**
   - 页面加载和数据展示
   - 创建新供应商
   - 编辑现有供应商
   - 删除供应商
   - 测试供应商连接
   - 搜索和过滤
   - 错误处理
   - 状态管理

2. **Codex供应商管理**
   - 基本CRUD操作
   - API错误处理

3. **用户工作流**
   - 应用启动和导航
   - 跨页面数据管理
   - 错误处理和恢复
   - 状态持久化
   - 并发操作处理
   - 性能测试

### 测试覆盖指标

- **功能覆盖率**: > 85%
- **API覆盖率**: 100% (所有主要端点)
- **错误场景覆盖率**: > 80%
- **用户工作流覆盖率**: > 90%

## 最佳实践

### 1. 测试命名
- 使用描述性的测试名称
- 采用"应该 + 预期行为"的格式
- 使用中文描述业务场景

### 2. 测试结构
- Arrange（准备）: 设置测试数据和mocks
- Act（执行）: 执行用户操作
- Assert（断言）: 验证结果

### 3. 异步测试
- 使用`waitFor`等待异步操作完成
- 设置合适的超时时间
- 避免使用`setTimeout`进行硬编码等待

### 4. Mock策略
- 每个测试只mock必要的数据
- 在测试后重置mock状态
- 使用真实的业务数据进行测试

### 5. 状态管理测试
- 测试Jotai原子的初始状态
- 验证状态更新的正确性
- 测试状态派生和计算

## 故障排除

### 常见问题

1. **MSW服务器未启动**
   - 确保在测试文件中导入了`server`配置
   - 检查`jest.setup.js`中的服务器配置

2. **Mock数据不匹配**
   - 验证API路径是否正确
   - 检查Mock数据的格式和类型

3. **异步测试超时**
   - 增加`asyncUtilTimeout`配置
   - 使用`waitFor`替代固定延迟

4. **组件mock问题**
   - 检查jest.mock的路径是否正确
   - 确保mock组件返回有效的React元素

### 调试技巧

1. **使用screen.debug()**
   ```typescript
   renderWithProviders(<Component />);
   screen.debug(); // 打印当前DOM结构
   ```

2. **查看所有可用角色**
   ```typescript
   const { getByRole } = renderWithProviders(<Component />);
   console.log(screen.getByRole('button', { name: /test/i }));
   ```

3. **检查Mock状态**
   ```typescript
   // 打印当前server状态
   console.log(server.listHandlers());
   ```

## 持续集成

### CI/CD集成
```yaml
# GitHub Actions示例
- name: 运行前端测试
  run: |
    npm ci
    npm run test:frontend -- --coverage --watchAll=false
```

### 覆盖率报告
- 生成HTML格式的覆盖率报告
- 集成到代码质量检查流程
- 设置覆盖率阈值警告

## 扩展指南

### 添加新的页面测试
1. 在`tests/frontend/pages/`下创建测试文件
2. 使用现有的测试工具和mocks
3. 确保测试覆盖主要的用户场景

### 添加新的API mock
1. 在`tests/frontend/mocks/handlers.ts`中添加新的handler
2. 更新相关的测试工具函数
3. 确保mock数据与真实API响应格式一致

### 添加新的测试工具
1. 在`tests/frontend/utils/`下创建工具文件
2. 导出必要的函数和类型
3. 更新本README文档

## 相关资源

- [Jest官方文档](https://jestjs.io/docs/getting-started)
- [React Testing Library文档](https://testing-library.com/docs/react-testing-library/intro/)
- [MSW文档](https://mswjs.io/docs/)
- [Jotai文档](https://jotai.org/docs/introduction)

## 维护说明

定期维护任务：
- 更新依赖包版本
- 清理过时的测试代码
- 检查覆盖率报告
- 更新mock数据以匹配API变更
- 审查和优化测试性能