// 工作模式专用类型定义

// 基础工作模式类型
export type WorkModeType = 'claude_only' | 'codex_only' | 'claude_codex'

// 工作模式切换步骤
export interface WorkModeSwitchStep {
  id: string
  name: string
  status: WorkModeStepStatus
  message?: string
  startedAt?: string
  completedAt?: string
  error?: string
}

// 工作模式步骤状态枚举
export type WorkModeStepStatus = 'pending' | 'in_progress' | 'completed' | 'failed' | 'skipped'

// 工作模式操作类型
export type WorkModeOperationType = 'switch' | 'create' | 'update' | 'delete' | 'rollback' | 'validate'

// 工作模式进度信息
export interface WorkModeProgress {
  totalSteps: number
  completedSteps: number
  currentStep?: WorkModeSwitchStep
  overallProgress: number // 0-100
  estimatedTimeRemaining?: number // 秒
  startTime: string
  isCompleted: boolean
  hasError: boolean
}

// 工作模式切换进度
export interface WorkModeSwitchProgress {
  progress: WorkModeProgress
  steps: WorkModeSwitchStep[]
  currentMode: string
  targetMode: string
  canCancel: boolean
  canRollback: boolean
}

// 工作模式验证结果
export interface WorkModeValidationResult {
  valid: boolean
  errors: string[]
  warnings: string[]
  suggestions: string[]
  validationDetails: {
    hasClaudeSupplier: boolean
    hasCodexSupplier: boolean
    hasMcpTemplates: boolean
    supplierConnectivity: boolean
    templateValidity: boolean
  }
}

// 工作模式历史记录
export interface WorkModeHistoryRecord {
  id: string
  operation: WorkModeOperationType
  fromMode?: string
  toMode?: string
  timestamp: string
  success: boolean
  error?: string
  duration?: number
  backupId?: number
  userId?: string
}

// 工作模式配置模板
export interface WorkModeTemplate {
  id: string
  name: string
  description: string
  modeType: WorkModeType
  claudeSupplierId?: number
  codexSupplierId?: number
  mcpTemplateIds: number[]
  variables: Record<string, string>
  isBuiltIn: boolean
  createdAt: string
  updatedAt: string
}

// 工作模式快照
export interface WorkModeSnapshot {
  id: string
  modeName: string
  timestamp: string
  config: any // WorkModeConfig 从config.ts导入
  appliedAt?: string
  isActive: boolean
  description?: string
}

// 工作模式统计信息
export interface WorkModeStats {
  totalModes: number
  activeMode: string
  modeUsageCount: Record<string, number>
  lastSwitchTime?: string
  totalSwitches: number
  failedSwitches: number
  averageSwitchTime?: number
  supplierUsageStats: {
    claude: number
    codex: number
  }
  templateUsageStats: Record<string, number>
}

// 工作模式推荐配置
export interface WorkModeRecommendation {
  modeType: WorkModeType
  reason: string
  confidence: number // 0-100
  suggestedSuppliers: {
    claude?: number
    codex?: number
  }
  suggestedTemplates: number[]
  estimatedBenefits: string[]
  potentialIssues: string[]
}

// 工作模式通知
export interface WorkModeNotification {
  id: string
  type: 'info' | 'warning' | 'error' | 'success'
  title: string
  message: string
  timestamp: string
  read: boolean
  actionUrl?: string
  metadata?: Record<string, any>
}

// 工作模式设置
export interface WorkModeSettings {
  autoBackupBeforeSwitch: boolean
  requireConfirmation: boolean
  showProgressNotifications: boolean
  enableAutoRecovery: boolean
  maxBackupHistory: number
  defaultTimeout: number // 秒
  enableRollbackConfirmation: boolean
  logLevel: 'debug' | 'info' | 'warn' | 'error'
}

// 工作模式事件类型
export type WorkModeEventType = 
  | 'mode_switch_started'
  | 'mode_switch_completed'
  | 'mode_switch_failed'
  | 'mode_switch_cancelled'
  | 'backup_created'
  | 'backup_restored'
  | 'validation_started'
  | 'validation_completed'
  | 'step_started'
  | 'step_completed'
  | 'step_failed'
  | 'progress_updated'

// 工作模式事件
export interface WorkModeEvent {
  id: string
  type: WorkModeEventType
  timestamp: string
  data: any
  source: string
  userId?: string
}

// 工作模式错误类型
export type WorkModeErrorType = 
  | 'validation_failed'
  | 'supplier_not_found'
  | 'template_not_found'
  | 'backup_failed'
  | 'restore_failed'
  | 'timeout'
  | 'network_error'
  | 'permission_denied'
  | 'invalid_configuration'
  | 'unknown_error'

// 工作模式错误
export interface WorkModeError {
  type: WorkModeErrorType
  message: string
  details?: any
  timestamp: string
  stepId?: string
  recoverable: boolean
  suggestions?: string[]
}

// 工作模式状态机
export interface WorkModeStateMachine {
  currentState: string
  availableTransitions: string[]
  isTransitioning: boolean
  lastTransitionTime?: string
  transitionHistory: Array<{
    from: string
    to: string
    timestamp: string
    success: boolean
  }>
}

// 工作模式依赖关系
export interface WorkModeDependency {
  id: string
  type: 'supplier' | 'template' | 'config' | 'service'
  name: string
  required: boolean
  status: 'available' | 'unavailable' | 'error'
  errorMessage?: string
  lastChecked: string
}

// 工作模式性能指标
export interface WorkModePerformanceMetrics {
  switchTime: number // 毫秒
  memoryUsage: number // MB
  cpuUsage: number // 百分比
  networkRequests: number
  errorCount: number
  userSatisfaction?: number // 1-5
  timestamp: string
}