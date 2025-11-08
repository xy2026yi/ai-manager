import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type {
  WorkModeConfig,
  WorkModeStatus,
  WorkModeSwitchRequest,
  WorkModeSwitchResult,
  WorkModeProgress,
  WorkModeSwitchStep,
  WorkModeStepStatus,
  WorkModeHistoryRecord,
  WorkModeSettings,
  WorkModeNotification
} from '@/types'
import { modeApi } from '@/services/modeApi'
import { supplierApi } from '@/services/supplierApi'
import { mcpTemplateApi } from '@/services/mcpTemplateApi'

export const useModeStore = defineStore('mode', () => {
  // 状态
  const currentStatus = ref<WorkModeStatus | null>(null)
  const modeConfigs = ref<WorkModeConfig[]>([])
  const suppliers = ref<any[]>([])
  const mcpTemplates = ref<any[]>([])
  const isSwitching = ref(false)
  const switchProgress = ref<WorkModeProgress | null>(null)
  const currentStep = ref<WorkModeSwitchStep | null>(null)
  const switchHistory = ref<WorkModeHistoryRecord[]>([])
  const notifications = ref<WorkModeNotification[]>([])
  const settings = ref<WorkModeSettings>({
    autoBackupBeforeSwitch: true,
    requireConfirmation: true,
    showProgressNotifications: true,
    enableAutoRecovery: true,
    maxBackupHistory: 10,
    defaultTimeout: 30000,
    enableRollbackConfirmation: true,
    logLevel: 'info'
  })

  // 计算属性
  const isReady = computed(() => 
    currentStatus.value !== null && !isSwitching.value
  )

  const currentMode = computed(() => 
    currentStatus.value?.currentMode || 'claude_only'
  )

  const isTransitioning = computed(() => 
    currentStatus.value?.isTransitioning || false
  )

  const availableClaudeSuppliers = computed(() => 
    suppliers.value.filter(s => s.type === 'claude')
  )

  const availableCodexSuppliers = computed(() => 
    suppliers.value.filter(s => s.type === 'codex')
  )

  const activeClaudeSupplier = computed(() => 
    suppliers.value.find(s => s.type === 'claude' && s.isActive)
  )

  const activeCodexSupplier = computed(() => 
    suppliers.value.find(s => s.type === 'codex' && s.isActive)
  )

  const progressPercentage = computed(() => {
    if (!switchProgress.value) return 0
    return Math.round((switchProgress.value.completedSteps / switchProgress.value.totalSteps) * 100)
  })

  const unreadNotifications = computed(() => 
    notifications.value.filter(n => !n.read)
  )

  const recentHistory = computed(() => 
    switchHistory.value.slice(0, 10)
  )

  // 加载相关操作
  const loadCurrentStatus = async () => {
    try {
      const status = await modeApi.getWorkModeStatus()
      currentStatus.value = status
      return status
    } catch (error) {
      console.error('加载工作模式状态失败:', error)
      throw error
    }
  }

  const loadModeConfigs = async () => {
    try {
      const configs = await modeApi.getAllWorkModeConfigs()
      modeConfigs.value = configs
      return configs
    } catch (error) {
      console.error('加载工作模式配置失败:', error)
      throw error
    }
  }

  const loadSuppliers = async () => {
    try {
      const sups = await supplierApi.listSuppliers()
      suppliers.value = sups
      return sups
    } catch (error) {
      console.error('加载供应商列表失败:', error)
      throw error
    }
  }

  const loadMcpTemplates = async () => {
    try {
      const templates = await mcpTemplateApi.listMcpTemplates()
      mcpTemplates.value = templates
      return templates
    } catch (error) {
      console.error('加载MCP模板列表失败:', error)
      throw error
    }
  }

  const loadAllData = async () => {
    try {
      await Promise.all([
        loadCurrentStatus(),
        loadModeConfigs(),
        loadSuppliers(),
        loadMcpTemplates()
      ])
    } catch (error) {
      console.error('加载数据失败:', error)
      throw error
    }
  }

  // 模式切换相关操作
  const switchMode = async (request: WorkModeSwitchRequest) => {
    try {
      isSwitching.value = true
      
      // 初始化进度
      switchProgress.value = {
        totalSteps: 5,
        completedSteps: 0,
        overallProgress: 0,
        startTime: new Date().toISOString(),
        isCompleted: false,
        hasError: false
      }

      // 执行切换
      const result = await modeApi.switchWorkMode(request)
      
      if (result.success) {
        // 切换成功，重新加载状态
        await loadCurrentStatus()
        await loadModeConfigs()
        
        // 添加历史记录
        const historyRecord: WorkModeHistoryRecord = {
          id: Date.now().toString(),
          operation: 'switch',
          fromMode: currentMode.value,
          toMode: request.targetMode,
          timestamp: new Date().toISOString(),
          success: true,
          duration: undefined,
          backupId: result.backupId
        }
        switchHistory.value.unshift(historyRecord)
        
        // 添加通知
        addNotification({
          id: `switch-${Date.now()}`,
          type: 'success',
          title: '模式切换成功',
          message: `成功切换到${getModeLabel(request.targetMode)}`,
          timestamp: new Date().toISOString(),
          read: false
        })
        
        return result
      } else {
        throw new Error(result.message)
      }
    } catch (error) {
      console.error('模式切换失败:', error)
      
      // 添加失败通知
      addNotification({
        id: `error-${Date.now()}`,
        type: 'error',
        title: '模式切换失败',
        message: error instanceof Error ? error.message : '未知错误',
        timestamp: new Date().toISOString(),
        read: false
      })
      
      throw error
    } finally {
      isSwitching.value = false
      switchProgress.value = null
      currentStep.value = null
    }
  }

  const rollbackMode = async (backupId: number) => {
    try {
      const success = await modeApi.rollbackWorkMode(backupId)
      
      if (success) {
        await loadCurrentStatus()
        await loadModeConfigs()
        
        addNotification({
          id: `rollback-${Date.now()}`,
          type: 'success',
          title: '回滚成功',
          message: '已成功回滚到之前的配置',
          timestamp: new Date().toISOString(),
          read: false
        })
      }
      
      return success
    } catch (error) {
      console.error('回滚失败:', error)
      
      addNotification({
        id: `rollback-error-${Date.now()}`,
        type: 'error',
        title: '回滚失败',
        message: error instanceof Error ? error.message : '回滚操作失败',
        timestamp: new Date().toISOString(),
        read: false
      })
      
      throw error
    }
  }

  // 进度管理
  const updateProgress = (progress: WorkModeProgress) => {
    switchProgress.value = progress
  }

  const updateCurrentStep = (step: WorkModeSwitchStep) => {
    currentStep.value = step
    if (switchProgress.value) {
      switchProgress.value.completedSteps = switchProgress.value.completedSteps + 1
      switchProgress.value.overallProgress = Math.round((switchProgress.value.completedSteps / switchProgress.value.totalSteps) * 100)
      switchProgress.value.isCompleted = switchProgress.value.completedSteps >= switchProgress.value.totalSteps
      switchProgress.value.hasError = step.status === 'failed'
    }
  }

  const resetProgress = () => {
    switchProgress.value = null
    currentStep.value = null
  }

  // 通知管理
  const addNotification = (notification: WorkModeNotification) => {
    notifications.value.unshift(notification)
    // 保持最多100条通知
    if (notifications.value.length > 100) {
      notifications.value = notifications.value.slice(0, 100)
    }
  }

  const markNotificationAsRead = (id: string) => {
    const notification = notifications.value.find(n => n.id === id)
    if (notification) {
      notification.read = true
    }
  }

  const markAllNotificationsAsRead = () => {
    notifications.value.forEach(n => {
      n.read = true
    })
  }

  const clearNotifications = () => {
    notifications.value = []
  }

  // 设置管理
  const updateSettings = (newSettings: Partial<WorkModeSettings>) => {
    settings.value = { ...settings.value, ...newSettings }
    // TODO: 持久化设置到本地存储
  }

  const getSettings = (): WorkModeSettings => {
    return settings.value
  }

  // 获取推荐配置
  const getRecommendedConfig = async () => {
    try {
      return await modeApi.getRecommendedWorkModeConfig()
    } catch (error) {
      console.error('获取推荐配置失败:', error)
      throw error
    }
  }

  // 统计信息
  const getStats = async () => {
    try {
      return await modeApi.getWorkModeStats()
    } catch (error) {
      console.error('获取统计信息失败:', error)
      throw error
    }
  }

  // 工具函数
  const getModeLabel = (mode: string) => {
    const modeMap: Record<string, string> = {
      'claude_only': '单Claude模式',
      'codex_only': '单Codex模式',
      'claude_codex': '混合模式'
    }
    return modeMap[mode] || mode
  }

  const validateSwitchRequest = (request: WorkModeSwitchRequest): { valid: boolean; errors: string[] } => {
    const errors: string[] = []
    
    // 基本验证
    if (!request.targetMode) {
      errors.push('目标模式不能为空')
    }
    
    // 供应商验证
    if (request.targetMode === 'claude_only' && !request.claudeSupplierId) {
      errors.push('单Claude模式需要选择Claude供应商')
    }
    
    if (request.targetMode === 'codex_only' && !request.codexSupplierId) {
      errors.push('单Codex模式需要选择Codex供应商')
    }
    
    if (request.targetMode === 'claude_codex' && (!request.claudeSupplierId || !request.codexSupplierId)) {
      errors.push('混合模式需要同时选择Claude和Codex供应商')
    }
    
    return {
      valid: errors.length === 0,
      errors
    }
  }

  // 初始化
  const initialize = async () => {
    try {
      await loadAllData()
      
      // 从本地存储恢复设置
      // TODO: 从localStorage恢复设置
      
      console.log('工作模式Store初始化完成')
    } catch (error) {
      console.error('工作模式Store初始化失败:', error)
      throw error
    }
  }

  return {
    // 状态
    currentStatus,
    modeConfigs,
    suppliers,
    mcpTemplates,
    isSwitching,
    switchProgress,
    currentStep,
    switchHistory,
    notifications,
    settings,

    // 计算属性
    isReady,
    currentMode,
    isTransitioning,
    availableClaudeSuppliers,
    availableCodexSuppliers,
    activeClaudeSupplier,
    activeCodexSupplier,
    progressPercentage,
    unreadNotifications,
    recentHistory,

    // 加载操作
    loadCurrentStatus,
    loadModeConfigs,
    loadSuppliers,
    loadMcpTemplates,
    loadAllData,

    // 模式切换操作
    switchMode,
    rollbackMode,

    // 进度管理
    updateProgress,
    updateCurrentStep,
    resetProgress,

    // 通知管理
    addNotification,
    markNotificationAsRead,
    markAllNotificationsAsRead,
    clearNotifications,

    // 设置管理
    updateSettings,
    getSettings,

    // 其他操作
    getRecommendedConfig,
    getStats,
    getModeLabel,
    validateSwitchRequest,

    // 初始化
    initialize
  }
})