// 全局状态管理
//
// 使用Jotai进行原子化状态管理，包括所有业务实体和应用状态

import { atom } from 'jotai';
import { atomWithImmer } from 'jotai-immer';
import { atomWithStorage } from 'jotai/utils';
import type {
  ClaudeProvider,
  CodexProvider,
  AgentGuide,
  McpServer,
  CommonConfig,
  PaginationParams,
  PagedResult,
  ClaudeProviderStats,
  CodexProviderStats,
  AgentGuideStats,
  McpServerStats,
  CommonConfigStats,
  Notification
} from '../types';

// ==================== 应用级状态 ====================

// 主题设置
export const themeAtom = atomWithStorage<'light' | 'dark'>('theme', 'light');

// 侧边栏折叠状态
export const sidebarCollapsedAtom = atom(false);

// 加载状态
export const globalLoadingAtom = atom(false);

// 通知列表
export const notificationsAtom = atom<Notification[]>([]);

// 当前用户信息（如果需要）
export const currentUserAtom = atom<{
  id: string;
  name: string;
  role: string;
} | null>(null);

// ==================== 通用状态 ====================

// 通用加载状态
export const loadingAtom = atom(false);

// 通用错误状态
export const errorAtom = atom<string | null>(null);

// 数据刷新触发器
export const refreshTriggerAtom = atom(0);

// ==================== Claude供应商状态 ====================

// Claude供应商列表
export const claudeProvidersAtom = atom<PagedResult<ClaudeProvider> | null>(null);

// 当前选中的Claude供应商
export const selectedClaudeProviderAtom = atom<ClaudeProvider | null>(null);

// Claude供应商加载状态
export const claudeProvidersLoadingAtom = atom(false);

// Claude供应商搜索参数
export const claudeProvidersSearchAtom = atom<PaginationParams & { search?: string; active_only?: boolean }>({
  page: 1,
  limit: 20,
  search: '',
  active_only: false
});

// Claude供应商统计信息
export const claudeProviderStatsAtom = atom<ClaudeProviderStats | null>(null);

// ==================== Codex供应商状态 ====================

// Codex供应商列表
export const codexProvidersAtom = atom<PagedResult<CodexProvider> | null>(null);

// 当前选中的Codex供应商
export const selectedCodexProviderAtom = atom<CodexProvider | null>(null);

// Codex供应商加载状态
export const codexProvidersLoadingAtom = atom(false);

// Codex供应商搜索参数
export const codexProvidersSearchAtom = atom<PaginationParams & { search?: string; active_only?: boolean }>({
  page: 1,
  limit: 20,
  search: '',
  active_only: false
});

// Codex供应商统计信息
export const codexProviderStatsAtom = atom<CodexProviderStats | null>(null);

// ==================== Agent指导文件状态 ====================

// Agent指导文件列表
export const agentGuidesAtom = atom<PagedResult<AgentGuide> | null>(null);

// 当前选中的Agent指导文件
export const selectedAgentGuideAtom = atom<AgentGuide | null>(null);

// Agent指导文件加载状态
export const agentGuidesLoadingAtom = atom(false);

// Agent指导文件搜索参数
export const agentGuidesSearchAtom = atom<PaginationParams & { search?: string; guide_type?: string }>({
  page: 1,
  limit: 20,
  search: '',
  guide_type: ''
});

// Agent指导文件统计信息
export const agentGuideStatsAtom = atom<AgentGuideStats | null>(null);

// ==================== MCP服务器状态 ====================

// MCP服务器列表
export const mcpServersAtom = atom<PagedResult<McpServer> | null>(null);

// 当前选中的MCP服务器
export const selectedMcpServerAtom = atom<McpServer | null>(null);

// MCP服务器加载状态
export const mcpServersLoadingAtom = atom(false);

// MCP服务器搜索参数
export const mcpServersSearchAtom = atom<PaginationParams & { search?: string; server_type?: string; active_only?: boolean }>({
  page: 1,
  limit: 20,
  search: '',
  server_type: '',
  active_only: false
});

// MCP服务器统计信息
export const mcpServerStatsAtom = atom<McpServerStats | null>(null);

// ==================== 通用配置状态 ====================

// 通用配置列表
export const commonConfigsAtom = atom<PagedResult<CommonConfig> | null>(null);

// 当前选中的通用配置
export const selectedCommonConfigAtom = atom<CommonConfig | null>(null);

// 通用配置加载状态
export const commonConfigsLoadingAtom = atom(false);

// 通用配置搜索参数
export const commonConfigsSearchAtom = atom<PaginationParams & { search?: string; category?: string; active_only?: boolean }>({
  page: 1,
  limit: 20,
  search: '',
  category: '',
  active_only: false
});

// 通用配置统计信息
export const commonConfigStatsAtom = atom<CommonConfigStats | null>(null);

// ==================== 表单和模态框状态 ====================

// Claude供应商表单模态框状态
export const claudeProviderModalAtom = atom<{
  isOpen: boolean;
  mode: 'create' | 'edit';
  data?: ClaudeProvider;
}>({
  isOpen: false,
  mode: 'create'
});

// Codex供应商表单模态框状态
export const codexProviderModalAtom = atom<{
  isOpen: boolean;
  mode: 'create' | 'edit';
  data?: CodexProvider;
}>({
  isOpen: false,
  mode: 'create'
});

// Agent指导文件表单模态框状态
export const agentGuideModalAtom = atom<{
  isOpen: boolean;
  mode: 'create' | 'edit';
  data?: AgentGuide;
}>({
  isOpen: false,
  mode: 'create'
});

// MCP服务器表单模态框状态
export const mcpServerModalAtom = atom<{
  isOpen: boolean;
  mode: 'create' | 'edit';
  data?: McpServer;
}>({
  isOpen: false,
  mode: 'create'
});

// 通用配置表单模态框状态
export const commonConfigModalAtom = atom<{
  isOpen: boolean;
  mode: 'create' | 'edit';
  data?: CommonConfig;
}>({
  isOpen: false,
  mode: 'create'
});

// ==================== 缓存状态 ====================

// API响应缓存（简单的内存缓存）
interface CacheEntry<T> {
  data: T;
  timestamp: number;
  expiry: number;
}

export const apiCacheAtom = atomWithImmer<Map<string, CacheEntry<any>>>(new Map());

// 缓存工具函数
export const cacheUtils = {
  set: <T>(cache: Map<string, CacheEntry<any>>, key: string, data: T, expiryMs: number = 5 * 60 * 1000) => {
    cache.set(key, {
      data,
      timestamp: Date.now(),
      expiry: Date.now() + expiryMs
    });
  },

  get: <T>(cache: Map<string, CacheEntry<any>>, key: string): T | null => {
    const entry = cache.get(key);
    if (!entry) return null;

    if (Date.now() > entry.expiry) {
      cache.delete(key);
      return null;
    }

    return entry.data;
  },

  clear: (cache: Map<string, CacheEntry<any>>, pattern?: string) => {
    if (pattern) {
      for (const key of cache.keys()) {
        if (key.includes(pattern)) {
          cache.delete(key);
        }
      }
    } else {
      cache.clear();
    }
  }
};

// ==================== 导出所有原子 ====================

export const atoms = {
  // 应用级
  themeAtom,
  sidebarCollapsedAtom,
  globalLoadingAtom,
  notificationsAtom,
  currentUserAtom,

  // Claude供应商
  claudeProvidersAtom,
  selectedClaudeProviderAtom,
  claudeProvidersLoadingAtom,
  claudeProvidersSearchAtom,
  claudeProviderStatsAtom,

  // Codex供应商
  codexProvidersAtom,
  selectedCodexProviderAtom,
  codexProvidersLoadingAtom,
  codexProvidersSearchAtom,
  codexProviderStatsAtom,

  // Agent指导文件
  agentGuidesAtom,
  selectedAgentGuideAtom,
  agentGuidesLoadingAtom,
  agentGuidesSearchAtom,
  agentGuideStatsAtom,

  // MCP服务器
  mcpServersAtom,
  selectedMcpServerAtom,
  mcpServersLoadingAtom,
  mcpServersSearchAtom,
  mcpServerStatsAtom,

  // 通用配置
  commonConfigsAtom,
  selectedCommonConfigAtom,
  commonConfigsLoadingAtom,
  commonConfigsSearchAtom,
  commonConfigStatsAtom,

  // 模态框
  claudeProviderModalAtom,
  codexProviderModalAtom,
  agentGuideModalAtom,
  mcpServerModalAtom,
  commonConfigModalAtom,

  // 缓存
  apiCacheAtom
};

// 类型导出
export type AtomsType = typeof atoms;