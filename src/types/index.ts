// 应用类型定义
//
// 定义所有数据模型和API接口类型

// 通用分页参数
export interface PaginationParams {
  page?: number;
  limit?: number;
  offset?: number;
  search?: string;
}

// 通用分页结果
export interface PagedResult<T> {
  data: T[];
  total: number;
  page: number;
  limit: number;
  total_pages: number;
}

// API响应格式
export interface ApiResponse<T> {
  success: boolean;
  data: T;
  message?: string | null;
  timestamp: string;
}

// API错误响应
export interface ApiErrorResponse {
  success: false;
  error: {
    code: string;
    message: string;
    details?: any;
  };
  timestamp: string;
}

// Claude供应商相关类型
export interface ClaudeProvider {
  id: number;
  name: string;
  url: string;
  token: string; // 加密存储
  timeout?: number;
  auto_update?: number; // 1-禁用遥测，0-启用遥测
  type: string; // paid 或 public_welfare
  enabled: number; // 0-未启用，1-启用
  opus_model?: string;
  sonnet_model?: string;
  haiku_model?: string;
  created_at?: string;
  updated_at?: string;
}

export interface CreateClaudeProviderRequest {
  name: string;
  url: string;
  token: string;
  timeout?: number;
  auto_update?: number;
  type?: string;
  opus_model?: string;
  sonnet_model?: string;
  haiku_model?: string;
}

export interface UpdateClaudeProviderRequest {
  name?: string;
  url?: string;
  token?: string;
  timeout?: number;
  auto_update?: number;
  type?: string;
  enabled?: number;
  opus_model?: string;
  sonnet_model?: string;
  haiku_model?: string;
}

// Codex供应商相关类型
export interface CodexProvider {
  id: number;
  name: string;
  url: string;
  token: string; // 加密存储
  type: string; // paid 或 public_welfare
  enabled: number; // 0-未启用，1-启用
  created_at?: string;
  updated_at?: string;
}

export interface CreateCodexProviderRequest {
  name: string;
  url: string;
  token: string;
  type?: string;
}

export interface UpdateCodexProviderRequest {
  name?: string;
  url?: string;
  token?: string;
  type?: string;
  enabled?: number;
}

// Agent指导文件相关类型
export interface AgentGuide {
  id: number;
  name: string;
  type: string; // 'only' 或 'and'
  text: string; // 文件完整内容
  created_at?: string;
  updated_at?: string;
}

export interface CreateAgentGuideRequest {
  name: string;
  type: string;
  text: string;
}

export interface UpdateAgentGuideRequest {
  name?: string;
  type?: string;
  text?: string;
}

// MCP服务器相关类型
export interface McpServer {
  id: number;
  name: string;
  type?: string; // stdio, sse等
  timeout?: number; // 默认30000ms
  command: string; // 命令，如npx, uvx, python等
  args: string; // 命令参数，存储为JSON字符串
  env?: string; // 环境变量，存储为JSON字符串
  created_at?: string;
  updated_at?: string;
}

export interface CreateMcpServerRequest {
  name: string;
  type?: string;
  timeout?: number;
  command: string;
  args: string[];
  env?: Record<string, string>;
}

export interface UpdateMcpServerRequest {
  name?: string;
  type?: string;
  timeout?: number;
  command?: string;
  args?: string[];
  env?: Record<string, string> | null;
}

// 通用配置相关类型
export interface CommonConfig {
  id: number;
  key: string;
  value: string; // 支持环境变量替换，如 ${HOME}
  description?: string;
  category: string; // 配置分类
  is_active: number; // 是否启用：1-启用，0-禁用
  created_at?: string;
  updated_at?: string;
}

export interface CreateCommonConfigRequest {
  key: string;
  value: string;
  description?: string;
  category?: string;
  is_active?: number;
}

export interface UpdateCommonConfigRequest {
  key?: string;
  value?: string;
  description?: string;
  category?: string;
  is_active?: number;
}

// 批量更新配置请求
export interface BatchUpdateRequest {
  configs: ConfigItem[];
}

export interface ConfigItem {
  key: string;
  value: string;
}

// 统计信息类型
export interface ClaudeProviderStats {
  total: number;
  enabled_count: number;
  disabled_count: number;
  paid_type: number;
  public_welfare_type: number;
  enabled_rate: number;
}

export interface CodexProviderStats {
  total: number;
  enabled_count: number;
  disabled_count: number;
  paid_type: number;
  public_welfare_type: number;
  enabled_rate: number;
}

export interface AgentGuideStats {
  total: number;
  only_type: number;
  and_type: number;
  only_type_rate: number;
  and_type_rate: number;
}

export interface McpServerStats {
  total: number;
  stdio_type: number;
  sse_type: number;
  websocket_type: number;
  active_count: number;
  inactive_count: number;
  stdio_type_rate: number;
  sse_type_rate: number;
  websocket_type_rate: number;
  active_rate: number;
}

export interface CommonConfigStats {
  total: number;
  active: number;
  inactive: number;
  active_rate: number;
  categories: string[];
  category_count: number;
}

// UI相关类型
export interface TableColumn<T> {
  key: keyof T;
  title: string;
  sortable?: boolean;
  width?: string;
  render?: (value: any, record: T) => React.ReactNode;
}

export interface ActionItem {
  label: string;
  icon?: string;
  onClick: () => void;
  variant?: 'primary' | 'secondary' | 'danger';
}

// 表单相关类型
export interface FormFieldConfig {
  name: string;
  label: string;
  type: 'text' | 'textarea' | 'select' | 'number' | 'checkbox' | 'password';
  placeholder?: string;
  required?: boolean;
  options?: { label: string; value: string }[];
  validation?: {
    min?: number;
    max?: number;
    pattern?: string;
    message?: string;
  };
}

// 通知类型
export interface Notification {
  id: string;
  type: 'success' | 'error' | 'warning' | 'info';
  title: string;
  message: string;
  duration?: number;
  persistent?: boolean;
}

// 导航相关类型
export interface MenuItem {
  id: string;
  label: string;
  icon?: string;
  path?: string;
  children?: MenuItem[];
  badge?: string | number;
}