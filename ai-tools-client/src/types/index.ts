// 通用类型定义
export interface ApiResponse<T = unknown> {
  success: boolean
  data?: T
  message?: string
}

// 分页相关
export interface PaginationParams {
  page: number
  pageSize: number
}

export interface PaginationResponse<T> {
  items: T[]
  total: number
  page: number
  pageSize: number
  totalPages: number
}

// 操作结果
export interface OperationResult {
  success: boolean
  message: string
  details?: unknown
}

// 配置相关类型
export type PlatformType = 'windows' | 'macos' | 'linux'
export type AiType = 'claude' | 'codex'
export type WorkMode = 'claude_only' | 'codex_only' | 'claude_codex'

// 重新导出所有类型
export * from './supplier'
export * from './mcpTemplate'
export * from './config'
export * from './mode'
