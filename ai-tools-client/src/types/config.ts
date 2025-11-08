import type { WorkMode, Supplier, McpTemplate } from './index'

// 配置文件类型
export type ConfigType = 'claude' | 'codex' | 'settings' | 'auth'

// 配置备份信息
export interface ConfigBackup {
  id: number
  configType: ConfigType
  configPath: string
  backupContent: string
  operationType: 'backup' | 'update' | 'restore'
  operationTime: string
  description?: string
}

// Claude配置结构
export interface ClaudeConfig {
  numStartups?: number
  installMethod?: 'global' | 'local'
  autoUpdates?: boolean
  tipsHistory?: Record<string, number>
  cachedStatsigGates?: Record<string, boolean>
  cachedDynamicConfigs?: Record<string, unknown>
  firstStartTime?: string
  userID?: string
  sonnet45MigrationComplete?: boolean
  changelogLastFetched?: number
  iterm2SetupInProgress?: boolean
  iterm2BackupPath?: string
  shiftEnterKeyBindingInstalled?: boolean
  hasCompletedOnboarding?: boolean
  lastOnboardingVersion?: string
  hasOpusPlanDefault?: boolean
  lastReleaseNotesSeen?: string
  projects?: Record<string, unknown>
  mcpServers?: Record<string, ClaudeMcpServer>
  env?: Record<string, string>
}

export interface ClaudeMcpServer {
  type: 'stdio' | 'websocket'
  command?: string
  args?: string[]
  env?: Record<string, string>
}

// Codex配置结构
export interface CodexConfig {
  model?: string
  modelReasoningEffort?: 'low' | 'medium' | 'high'
  disableResponseStorage?: boolean
  preferredAuthMethod?: 'apikey' | 'bearer'
  windowsWslSetupAcknowledged?: boolean
  modelProvider?: string
  modelProviders?: Record<string, CodexModelProvider>
  mcpServers?: Record<string, CodexMcpServer>
}

export interface CodexModelProvider {
  name: string
  baseUrl: string
  wireApi?: 'responses' | 'chat'
  requiresOpenaiAuth?: boolean
}

export interface CodexMcpServer {
  type: 'stdio' | 'websocket'
  command?: string
  args?: string[]
  env?: Record<string, string>
  startupTimeoutMs?: number
}

// 工作模式状态
export interface WorkModeState {
  currentMode: WorkMode
  previousMode?: WorkMode
  isTransitioning: boolean
  activeClaudeSupplier?: number
  activeCodexSupplier?: number
  activeMcpTemplates: number[]
}

// 配置更新请求
export interface UpdateConfigRequest {
  configType: ConfigType
  content: string
  createBackup?: boolean
  description?: string
}

// 配置差异对比结果
export interface ConfigDiffResult {
  hasChanges: boolean
  additions: string[]
  removals: string[]
  modifications: Array<{
    line: number
    old: string
    new: string
  }>
}

// 配置模板变量
export interface ConfigVariable {
  name: string
  value: string
  description?: string
}

// 模板生成结果
export interface TemplateGenerationResult {
  success: boolean
  content: string
  format: 'json' | 'toml'
  errors?: string[]
}

// 配置操作结果
export interface ConfigOperationResult {
  success: boolean
  message: string
  data?: any
}

// 配置预览参数
export interface ConfigPreviewParams {
  configType: ConfigType
  claudeSupplierId?: number
  codexSupplierId?: number
  mcpTemplateIds?: number[]
  variables?: Record<string, string>
}

// 配置验证结果
export interface ConfigValidationResult {
  valid: boolean
  error?: string
  warnings?: string[]
}

// 配置应用结果
export interface ConfigApplyResult {
  success: boolean
  message: string
  configPath?: string
  backupId?: number
  appliedAt?: string
}

// 配置文件路径映射
export interface ConfigPathMap {
  [key: string]: string
}

// 配置模板引擎接口
export interface IConfigTemplateEngine {
  generateClaudeConfig(
    claudeSupplier: Supplier,
    templates: McpTemplate[],
    variables?: Record<string, string>
  ): Promise<TemplateGenerationResult>

  generateCodexConfig(
    codexSupplier: Supplier,
    templates: McpTemplate[],
    variables?: Record<string, string>
  ): Promise<TemplateGenerationResult>

  replaceVariables(content: string, variables: Record<string, string>): string

  extractVariables(content: string): string[]

  validateConfig(content: string, configType: ConfigType): ConfigValidationResult
}

// 配置文件管理器接口
export interface IConfigFileManager {
  backupConfig(
    configType: ConfigType,
    content: string,
    description?: string
  ): Promise<ConfigOperationResult>

  getConfigHistory(configType: ConfigType, limit?: number): Promise<ConfigBackup[]>

  getLatestConfigBackup(configType: ConfigType): Promise<ConfigBackup | null>

  restoreConfigFromBackup(backupId: number): Promise<ConfigOperationResult>

  applyConfig(
    configType: ConfigType,
    content: string,
    createBackup?: boolean
  ): Promise<ConfigApplyResult>

  previewConfig(params: ConfigPreviewParams): Promise<ConfigOperationResult>

  validateConfigFormat(content: string, configType: ConfigType): ConfigValidationResult

  compareConfigs(oldContent: string, newContent: string): ConfigDiffResult

  deleteConfigHistory(backupId: number): Promise<ConfigOperationResult>

  cleanupOldConfigHistory(configType: ConfigType, keepCount: number): Promise<ConfigOperationResult>

  getConfigPath(configType: ConfigType): string

  getSupportedConfigTypes(): ConfigType[]
}

// 配置历史查询参数
export interface ConfigHistoryQuery {
  configType?: ConfigType
  operationType?: 'backup' | 'update' | 'restore'
  startDate?: string
  endDate?: string
  limit?: number
  offset?: number
}

// 配置历史统计
export interface ConfigHistoryStats {
  totalBackups: number
  totalRestores: number
  lastBackupTime?: string
  oldestBackupTime?: string
  configTypes: Record<ConfigType, number>
}

// 批量配置操作
export interface BatchConfigOperation {
  operationType: 'backup' | 'restore' | 'apply' | 'delete'
  configTypes: ConfigType[]
  targetIds?: number[]
  options?: {
    createBackup?: boolean
    description?: string
    variables?: Record<string, string>
  }
}

// 批量操作结果
export interface BatchOperationResult {
  success: boolean
  totalProcessed: number
  successCount: number
  failureCount: number
  results: Array<{
    configType: ConfigType
    success: boolean
    message?: string
    data?: any
  }>
  errors?: string[]
}

// 配置导出/导入
export interface ConfigExportData {
  version: string
  exportTime: string
  configs: Array<{
    configType: ConfigType
    content: string
    description?: string
    variables?: Record<string, string>
  }>
  history: ConfigBackup[]
  metadata: {
    platform: string
    appVersion: string
    totalConfigs: number
  }
}

export interface ConfigImportData {
  configs: Array<{
    configType: ConfigType
    content: string
    description?: string
  }>
  importHistory?: boolean
  overwriteExisting?: boolean
}

// 配置同步状态
export interface ConfigSyncStatus {
  lastSyncTime?: string
  syncInProgress: boolean
  hasUnsavedChanges: boolean
  conflictResolution?: 'local' | 'remote' | 'manual'
}

// 配置设置
export interface ConfigSettings {
  autoBackup: boolean
  backupRetentionDays: number
  maxBackupCount: number
  enableSync: boolean
  syncInterval: number // 分钟
  validationLevel: 'strict' | 'normal' | 'relaxed'
  formatOnSave: boolean
}

// 配置事件
export interface ConfigEvent {
  id: string
  type: 'backup' | 'restore' | 'apply' | 'validate' | 'sync'
  configType: ConfigType
  timestamp: string
  userId?: string
  details?: Record<string, any>
  status: 'pending' | 'success' | 'failed'
  message?: string
}

// 配置事件监听器
export interface ConfigEventListener {
  (event: ConfigEvent): void
}

// 工作模式配置
export interface WorkModeConfig {
  id?: number
  modeName: string
  activeClaudeSupplierId?: number
  activeCodexSupplierId?: number
  mcpTemplateIds?: number[]
  createdAt?: string
  updatedAt?: string
}

// 工作模式创建请求
export interface CreateWorkModeRequest {
  modeName: string
  activeClaudeSupplierId?: number
  activeCodexSupplierId?: number
  mcpTemplateIds?: number[]
}

// 工作模式更新请求
export interface UpdateWorkModeRequest {
  id: number
  modeName?: string
  activeClaudeSupplierId?: number
  activeCodexSupplierId?: number
  mcpTemplateIds?: number[]
}

// 工作模式切换请求
export interface WorkModeSwitchRequest {
  targetMode: string
  claudeSupplierId?: number
  codexSupplierId?: number
  mcpTemplateIds?: number[]
  createBackup: boolean
}

// 工作模式状态
export interface WorkModeStatus {
  currentMode: string
  isTransitioning: boolean
  lastSwitchTime?: string
  activeClaudeSupplier?: string
  activeCodexSupplier?: string
  activeMcpTemplates: string[]
}

// 工作模式切换结果
export interface WorkModeSwitchResult {
  success: boolean
  message: string
  backupId?: number
  appliedAt?: string
  stepsCompleted: string[]
  duration?: number
  appliedConfigurations?: string[]
  error?: string
}

// 工作模式操作结果
export interface WorkModeOperationResult {
  success: boolean
  operation: string
  message: string
  data?: any
  error?: string
  timestamp: string
  duration?: number
}
