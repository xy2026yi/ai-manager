// API服务层
//
// 统一处理与后端API的通信，包括请求封装、错误处理和重试机制

import { invoke } from '@tauri-apps/api/tauri';
import type {
  ApiResponse,
  ApiErrorResponse,
  ClaudeProvider,
  CodexProvider,
  AgentGuide,
  McpServer,
  CommonConfig,
  CreateClaudeProviderRequest,
  UpdateClaudeProviderRequest,
  CreateCodexProviderRequest,
  UpdateCodexProviderRequest,
  CreateAgentGuideRequest,
  UpdateAgentGuideRequest,
  CreateMcpServerRequest,
  UpdateMcpServerRequest,
  CreateCommonConfigRequest,
  UpdateCommonConfigRequest,
  PaginationParams,
  PagedResult,
  ClaudeProviderStats,
  CodexProviderStats,
  AgentGuideStats,
  McpServerStats,
  CommonConfigStats,
  BatchUpdateRequest
} from '../types';

// API基础配置
const API_BASE_URL = 'http://127.0.0.1:8080/api/v1';

// 自定义错误类
export class ApiError extends Error {
  constructor(
    public code: string,
    message: string,
    public details?: any
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

// 请求选项接口
interface RequestOptions {
  method?: 'GET' | 'POST' | 'PUT' | 'DELETE';
  body?: any;
  headers?: Record<string, string>;
  timeout?: number;
}

// API响应处理工具
class ApiResponseHandler {
  static async handleResponse<T>(response: Response): Promise<T> {
    if (!response.ok) {
      const errorData: ApiErrorResponse = await response.json().catch(() => ({
        success: false,
        error: {
          code: response.status.toString(),
          message: response.statusText || '网络请求失败',
        },
        timestamp: new Date().toISOString(),
      }));
      
      throw new ApiError(
        errorData.error.code,
        errorData.error.message,
        errorData.error.details
      );
    }

    const result: ApiResponse<T> = await response.json();
    
    if (!result.success) {
      throw new ApiError(
        'API_ERROR',
        result.message || '请求失败',
        result
      );
    }

    return result.data;
  }

  static handleError(error: any): never {
    if (error instanceof ApiError) {
      throw error;
    }

    if (error.name === 'TypeError' && error.message.includes('fetch')) {
      throw new ApiError('NETWORK_ERROR', '网络连接失败，请检查网络设置');
    }

    if (error.name === 'AbortError') {
      throw new ApiError('TIMEOUT', '请求超时，请稍后重试');
    }

    console.error('API请求未知错误:', error);
    throw new ApiError('UNKNOWN_ERROR', '未知错误，请稍后重试', error);
  }
}

// 通用API请求方法
class BaseApiService {
  private baseUrl: string;
  private defaultTimeout: number;

  constructor(baseUrl: string = API_BASE_URL, timeout: number = 30000) {
    this.baseUrl = baseUrl;
    this.defaultTimeout = timeout;
  }

  private async request<T>(endpoint: string, options: RequestOptions = {}): Promise<T> {
    const url = `${this.baseUrl}${endpoint}`;
    const controller = new AbortController();
    
    // 设置超时
    const timeoutId = setTimeout(() => controller.abort(), options.timeout || this.defaultTimeout);

    try {
      const response = await fetch(url, {
        method: options.method || 'GET',
        headers: {
          'Content-Type': 'application/json',
          ...options.headers,
        },
        body: options.body ? JSON.stringify(options.body) : undefined,
        signal: controller.signal,
      });

      clearTimeout(timeoutId);
      return ApiResponseHandler.handleResponse<T>(response);
    } catch (error) {
      clearTimeout(timeoutId);
      return ApiResponseHandler.handleError(error);
    }
  }

  protected get<T>(endpoint: string, params?: Record<string, any>): Promise<T> {
    let url = endpoint;
    if (params) {
      const searchParams = new URLSearchParams();
      Object.entries(params).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          searchParams.append(key, String(value));
        }
      });
      url += `?${searchParams.toString()}`;
    }
    return this.request<T>(url);
  }

  protected post<T>(endpoint: string, data?: any): Promise<T> {
    return this.request<T>(endpoint, { method: 'POST', body: data });
  }

  protected put<T>(endpoint: string, data?: any): Promise<T> {
    return this.request<T>(endpoint, { method: 'PUT', body: data });
  }

  protected delete<T>(endpoint: string): Promise<T> {
    return this.request<T>(endpoint, { method: 'DELETE' });
  }
}

// Claude供应商API服务
export class ClaudeProviderService extends BaseApiService {
  async list(params?: PaginationParams & { search?: string; active_only?: boolean }): Promise<PagedResult<ClaudeProvider>> {
    return this.get<PagedResult<ClaudeProvider>>('/claude-providers', params);
  }

  async get(id: number): Promise<ClaudeProvider> {
    return this.get<ClaudeProvider>(`/claude-providers/${id}`);
  }

  async create(data: CreateClaudeProviderRequest): Promise<ClaudeProvider> {
    return this.post<ClaudeProvider>('/claude-providers', data);
  }

  async update(id: number, data: UpdateClaudeProviderRequest): Promise<ClaudeProvider> {
    return this.put<ClaudeProvider>(`/claude-providers/${id}`, data);
  }

  async delete(id: number): Promise<void> {
    return this.delete<void>(`/claude-providers/${id}`);
  }

  async testConnection(id: number): Promise<boolean> {
    return this.get<boolean>(`/claude-providers/${id}/test`);
  }

  async getStats(): Promise<ClaudeProviderStats> {
    return this.get<ClaudeProviderStats>('/claude-providers/stats');
  }
}

// Codex供应商API服务
export class CodexProviderService extends BaseApiService {
  async list(params?: PaginationParams & { search?: string; active_only?: boolean }): Promise<PagedResult<CodexProvider>> {
    return this.get<PagedResult<CodexProvider>>('/codex-providers', params);
  }

  async get(id: number): Promise<CodexProvider> {
    return this.get<CodexProvider>(`/codex-providers/${id}`);
  }

  async create(data: CreateCodexProviderRequest): Promise<CodexProvider> {
    return this.post<CodexProvider>('/codex-providers', data);
  }

  async update(id: number, data: UpdateCodexProviderRequest): Promise<CodexProvider> {
    return this.put<CodexProvider>(`/codex-providers/${id}`, data);
  }

  async delete(id: number): Promise<void> {
    return this.delete<void>(`/codex-providers/${id}`);
  }

  async testConnection(id: number): Promise<boolean> {
    return this.get<boolean>(`/codex-providers/${id}/test`);
  }

  async getStats(): Promise<CodexProviderStats> {
    return this.get<CodexProviderStats>('/codex-providers/stats');
  }
}

// Agent指导文件API服务
export class AgentGuideService extends BaseApiService {
  async list(params?: PaginationParams & { search?: string; guide_type?: string }): Promise<PagedResult<AgentGuide>> {
    return this.get<PagedResult<AgentGuide>>('/agent-guides', params);
  }

  async get(id: number): Promise<AgentGuide> {
    return this.get<AgentGuide>(`/agent-guides/${id}`);
  }

  async create(data: CreateAgentGuideRequest): Promise<AgentGuide> {
    return this.post<AgentGuide>('/agent-guides', data);
  }

  async update(id: number, data: UpdateAgentGuideRequest): Promise<AgentGuide> {
    return this.put<AgentGuide>(`/agent-guides/${id}`, data);
  }

  async delete(id: number): Promise<void> {
    return this.delete<void>(`/agent-guides/${id}`);
  }

  async validate(id: number): Promise<boolean> {
    return this.get<boolean>(`/agent-guides/${id}/validate`);
  }

  async getStats(): Promise<AgentGuideStats> {
    return this.get<AgentGuideStats>('/agent-guides/stats');
  }
}

// MCP服务器API服务
export class McpServerService extends BaseApiService {
  async list(params?: PaginationParams & { search?: string; server_type?: string; active_only?: boolean }): Promise<PagedResult<McpServer>> {
    return this.get<PagedResult<McpServer>>('/mcp-servers', params);
  }

  async get(id: number): Promise<McpServer> {
    return this.get<McpServer>(`/mcp-servers/${id}`);
  }

  async create(data: CreateMcpServerRequest): Promise<McpServer> {
    return this.post<McpServer>('/mcp-servers', data);
  }

  async update(id: number, data: UpdateMcpServerRequest): Promise<McpServer> {
    return this.put<McpServer>(`/mcp-servers/${id}`, data);
  }

  async delete(id: number): Promise<void> {
    return this.delete<void>(`/mcp-servers/${id}`);
  }

  async test(id: number): Promise<boolean> {
    return this.get<boolean>(`/mcp-servers/${id}/test`);
  }

  async getStats(): Promise<McpServerStats> {
    return this.get<McpServerStats>('/mcp-servers/stats');
  }
}

// 通用配置API服务
export class CommonConfigService extends BaseApiService {
  async list(params?: PaginationParams & { search?: string; category?: string; active_only?: boolean }): Promise<PagedResult<CommonConfig>> {
    return this.get<PagedResult<CommonConfig>>('/common-configs', params);
  }

  async get(id: number): Promise<CommonConfig> {
    return this.get<CommonConfig>(`/common-configs/${id}`);
  }

  async getByKey(key: string): Promise<CommonConfig> {
    return this.get<CommonConfig>(`/common-configs/key/${encodeURIComponent(key)}`);
  }

  async create(data: CreateCommonConfigRequest): Promise<CommonConfig> {
    return this.post<CommonConfig>('/common-configs', data);
  }

  async update(id: number, data: UpdateCommonConfigRequest): Promise<CommonConfig> {
    return this.put<CommonConfig>(`/common-configs/${id}`, data);
  }

  async delete(id: number): Promise<void> {
    return this.delete<void>(`/common-configs/${id}`);
  }

  async batchUpdate(data: BatchUpdateRequest): Promise<number> {
    return this.post<number>('/common-configs/batch', data);
  }

  async validate(id: number): Promise<boolean> {
    return this.get<boolean>(`/common-configs/${id}/validate`);
  }

  async getStats(): Promise<CommonConfigStats> {
    return this.get<CommonConfigStats>('/common-configs/stats');
  }
}

// 系统API服务
export class SystemService extends BaseApiService {
  async getHealth(): Promise<void> {
    return this.get<void>('/health');
  }

  async getInfo(): Promise<any> {
    return this.get<any>('/info');
  }
}

// 导出服务实例
export const claudeProviderService = new ClaudeProviderService();
export const codexProviderService = new CodexProviderService();
export const agentGuideService = new AgentGuideService();
export const mcpServerService = new McpServerService();
export const commonConfigService = new CommonConfigService();
export const systemService = new SystemService();

// Tauri命令服务（用于文件操作等）
export class TauriCommandService {
  async openPath(path: string): Promise<void> {
    await invoke('open_path', { path });
  }

  async saveFile(path: string, content: string): Promise<void> {
    await invoke('save_file', { path, content });
  }

  async readFile(path: string): Promise<string> {
    return await invoke<string>('read_file', { path });
  }

  async showInFolder(path: string): Promise<void> {
    await invoke('show_in_folder', { path });
  }
}

export const tauriCommandService = new TauriCommandService();