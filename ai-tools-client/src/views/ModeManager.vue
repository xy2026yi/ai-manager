<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type {
  WorkModeConfig,
  WorkModeSwitchRequest,
  WorkModeStatus,
  WorkModeSwitchResult,
  WorkModeProgress,
  WorkModeStepStatus,
  WorkModeSwitchStep,
  Supplier,
  McpTemplate
} from '@/types'
import { modeApi } from '@/services/modeApi'
import { supplierApi } from '@/services/supplierApi'
import { mcpTemplateApi } from '@/services/mcpTemplateApi'

// 响应式数据
const currentStatus = ref<WorkModeStatus | null>(null)
const modeConfigs = ref<WorkModeConfig[]>([])
const suppliers = ref<Supplier[]>([])
const mcpTemplates = ref<McpTemplate[]>([])
const loading = ref(false)
const switching = ref(false)

// 切换相关状态
const switchDialogVisible = ref(false)
const selectedMode = ref<string>('claude_only')
const selectedClaudeSupplier = ref<number | null>(null)
const selectedCodexSupplier = ref<number | null>(null)
const selectedMcpTemplates = ref<number[]>([])
const createBackup = ref(true)

// 进度相关
const switchProgress = ref<WorkModeProgress | null>(null)
const currentStep = ref<WorkModeSwitchStep | null>(null)

// 历史记录
const historyVisible = ref(false)
const switchHistory = ref<any[]>([])

// 进度显示相关
const showDetails = ref(false)
const showTroubleshooting = ref(false)
const troubleshootingTab = ref('common')
const operationLogs = ref<Array<{
  id: string
  timestamp: string
  type: 'info' | 'success' | 'warning' | 'error'
  message: string
}>>([])

// 工作模式选项
const modeOptions = [
  { label: '单Claude模式', value: 'claude_only', description: '仅使用Claude供应商，适合Claude专用工作流' },
  { label: '单Codex模式', value: 'codex_only', description: '仅使用Codex供应商，适合Codex专用工作流' },
  { label: '混合模式', value: 'claude_codex', description: '同时使用Claude和Codex，适合复杂工作流' }
]

// 计算属性
const isFormValid = computed(() => {
  if (selectedMode.value === 'claude_only') {
    return selectedClaudeSupplier.value !== null
  } else if (selectedMode.value === 'codex_only') {
    return selectedCodexSupplier.value !== null
  } else {
    return selectedClaudeSupplier.value !== null && selectedCodexSupplier.value !== null
  }
})

const claudeSuppliers = computed(() => suppliers.value.filter(s => s.type === 'claude'))
const codexSuppliers = computed(() => suppliers.value.filter(s => s.type === 'codex'))

const progressPercentage = computed(() => {
  if (!switchProgress.value) return 0
  return Math.round((switchProgress.value.completedSteps / switchProgress.value.totalSteps) * 100)
})

// 加载数据
const loadData = async () => {
  loading.value = true
  try {
    const [status, configs, sups, templates] = await Promise.all([
      modeApi.getWorkModeStatus(),
      modeApi.getAllWorkModeConfigs(),
      supplierApi.listSuppliers(),
      mcpTemplateApi.listMcpTemplates()
    ])
    
    currentStatus.value = status
    modeConfigs.value = configs
    suppliers.value = sups
    mcpTemplates.value = templates
    
    // 设置默认选择
    if (selectedClaudeSupplier.value === null && claudeSuppliers.value.length > 0) {
      selectedClaudeSupplier.value = claudeSuppliers.value[0].id!
    }
    if (selectedCodexSupplier.value === null && codexSuppliers.value.length > 0) {
      selectedCodexSupplier.value = codexSuppliers.value[0].id!
    }
  } catch (error) {
    ElMessage.error('加载数据失败')
    console.error(error)
  } finally {
    loading.value = false
  }
}

// 显示切换对话框
const showSwitchDialog = () => {
  switchDialogVisible.value = true
  resetForm()
}

// 重置表单
const resetForm = () => {
  selectedMode.value = currentStatus.value?.currentMode || 'claude_only'
  selectedMcpTemplates.value = []
  createBackup.value = true
}

// 执行模式切换
const executeSwitch = async () => {
  if (!isFormValid.value) {
    ElMessage.warning('请完善配置信息')
    return
  }

  try {
    switching.value = true
    switchDialogVisible.value = false
    
    // 初始化进度
    switchProgress.value = {
      totalSteps: 5,
      completedSteps: 0,
      overallProgress: 0,
      startTime: new Date().toISOString(),
      isCompleted: false,
      hasError: false
    }

    // 构建切换请求
    const request: WorkModeSwitchRequest = {
      targetMode: selectedMode.value,
      claudeSupplierId: selectedClaudeSupplier.value || undefined,
      codexSupplierId: selectedCodexSupplier.value || undefined,
      mcpTemplateIds: selectedMcpTemplates.value.length > 0 ? selectedMcpTemplates.value : undefined,
      createBackup: createBackup.value
    }

    // 执行切换
    const result = await modeApi.switchWorkMode(request)
    
    if (result.success) {
      ElMessage.success(`成功切换到${getModeLabel(selectedMode.value)}`)
      await loadData() // 重新加载数据
    } else {
      throw new Error(result.message)
    }
  } catch (error) {
    ElMessage.error('模式切换失败')
    console.error(error)
  } finally {
    switching.value = false
    switchProgress.value = null
    currentStep.value = null
  }
}

// 获取模式标签
const getModeLabel = (mode: string) => {
  const option = modeOptions.find(opt => opt.value === mode)
  return option?.label || mode
}

// 取消切换
const cancelSwitch = () => {
  if (switching.value) {
    ElMessageBox.confirm('切换正在进行中，确定要取消吗？', '确认取消', {
      type: 'warning'
    }).then(() => {
      switching.value = false
      switchProgress.value = null
      currentStep.value = null
      ElMessage.info('已取消模式切换')
    }).catch(() => {
      // 用户取消
    })
  }
}

// 显示历史记录
const showHistory = async () => {
  historyVisible.value = true
  // TODO: 加载历史记录
}

// 回滚模式
const rollbackMode = async (backupId: number) => {
  try {
    await ElMessageBox.confirm('确定要回滚到之前的配置吗？', '确认回滚', {
      type: 'warning'
    })
    
    const success = await modeApi.rollbackWorkMode(backupId)
    if (success) {
      ElMessage.success('回滚成功')
      await loadData()
    } else {
      ElMessage.error('回滚失败')
    }
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('回滚失败')
      console.error(error)
    }
  }
}

// ========== 进度显示和用户反馈相关方法 ==========

// 格式化时间显示
const formatTime = (seconds: number): string => {
  if (seconds < 60) {
    return `${seconds}秒`
  } else if (seconds < 3600) {
    return `${Math.floor(seconds / 60)}分${seconds % 60}秒`
  } else {
    const hours = Math.floor(seconds / 3600)
    const minutes = Math.floor((seconds % 3600) / 60)
    return `${hours}小时${minutes}分钟`
  }
}

// 获取步骤警告类型
const getStepAlertType = (status: string): 'success' | 'warning' | 'info' | 'error' => {
  switch (status) {
    case 'completed': return 'success'
    case 'failed': return 'error'
    case 'in_progress': return 'warning'
    default: return 'info'
  }
}

// 获取步骤状态类型
const getStepStatusType = (status: string): 'primary' | 'success' | 'warning' | 'danger' => {
  switch (status) {
    case 'completed': return 'success'
    case 'failed': return 'danger'
    case 'in_progress': return 'warning'
    default: return 'primary'
  }
}

// 获取步骤图标
const getStepIcon = (status: string): string => {
  switch (status) {
    case 'completed': return 'Check'
    case 'failed': return 'Close'
    case 'in_progress': return 'Loading'
    default: return 'Clock'
  }
}

// 获取步骤列表
const getStepsList = (): WorkModeSwitchStep[] => {
  if (!switchProgress.value) return []

  const steps: WorkModeSwitchStep[] = [
    { id: '1', name: '验证请求参数', status: 'completed', message: '验证配置参数完整性' },
    { id: '2', name: '获取供应商和模板信息', status: 'completed', message: '从数据库获取相关配置' },
    { id: '3', name: '备份当前配置', status: 'completed', message: '创建当前配置的备份' },
    { id: '4', name: '生成新配置', status: 'completed', message: '基于模板生成新配置文件' },
    { id: '5', name: '应用新配置', status: 'completed', message: '将新配置写入系统' },
    { id: '6', name: '验证配置应用结果', status: 'completed', message: '检查配置是否正确应用' },
    { id: '7', name: '更新数据库记录', status: 'completed', message: '保存模式切换记录' }
  ]

  // 根据当前进度更新状态
  const currentStepIndex = Math.min(switchProgress.value.completedSteps, steps.length - 1)

  steps.forEach((step, index) => {
    if (index < currentStepIndex) {
      step.status = 'completed'
    } else if (index === currentStepIndex) {
      step.status = 'in_progress'
      if (currentStep.value) {
        step.message = currentStep.value.message
        step.startedAt = currentStep.value.startedAt
        step.completedAt = currentStep.value.completedAt
        step.error = currentStep.value.error
      }
    } else {
      step.status = 'pending'
    }
  })

  if (switchProgress.value.hasError) {
    const failedStep = steps.find(step => step.status === 'in_progress')
    if (failedStep) {
      failedStep.status = 'failed'
    }
  }

  return steps
}

// 添加操作日志
const addOperationLog = (type: 'info' | 'success' | 'warning' | 'error', message: string) => {
  operationLogs.value.unshift({
    id: `log-${Date.now()}-${Math.random()}`,
    timestamp: new Date().toISOString(),
    type,
    message
  })

  // 限制日志数量
  if (operationLogs.value.length > 100) {
    operationLogs.value = operationLogs.value.slice(0, 100)
  }
}

// 获取平台信息
const getPlatformInfo = (): string => {
  const userAgent = navigator.userAgent
  if (userAgent.includes('Windows')) {
    return 'Windows'
  } else if (userAgent.includes('Mac')) {
    return 'macOS'
  } else if (userAgent.includes('Linux')) {
    return 'Linux'
  } else {
    return '未知系统'
  }
}

// 复制诊断信息
const copyDiagnosticInfo = async () => {
  const diagnosticInfo = {
    应用版本: 'v1.0.0',
    操作系统: getPlatformInfo(),
    当前模式: currentStatus.value?.currentMode || '未知',
    最后切换: currentStatus.value?.lastSwitchTime ? new Date(currentStatus.value.lastSwitchTime).toLocaleString() : '无记录',
    是否切换中: switching.value,
    切换进度: switchProgress.value ? `${switchProgress.value.completedSteps}/${switchProgress.value.totalSteps}` : '无',
    错误状态: switchProgress.value?.hasError || false,
    操作日志数量: operationLogs.value.length,
    浏览器信息: navigator.userAgent,
    时间戳: new Date().toISOString()
  }

  try {
    await navigator.clipboard.writeText(JSON.stringify(diagnosticInfo, null, 2))
    ElMessage.success('诊断信息已复制到剪贴板')
  } catch (error) {
    console.error('复制失败:', error)
    ElMessage.error('复制失败，请手动复制')
  }
}

// 增强的执行切换方法
const executeSwitch = async () => {
  if (!isFormValid.value) {
    ElMessage.warning('请完善配置信息')
    return
  }

  try {
    switching.value = true
    switchDialogVisible.value = false
    showDetails.value = true // 自动展开详情

    // 清空之前的日志
    operationLogs.value = []
    addOperationLog('info', '开始模式切换流程')

    // 初始化进度
    switchProgress.value = {
      totalSteps: 7,
      completedSteps: 0,
      overallProgress: 0,
      startTime: new Date().toISOString(),
      isCompleted: false,
      hasError: false
    }

    addOperationLog('info', `目标模式: ${getModeLabel(selectedMode.value)}`)
    addOperationLog('info', `Claude供应商: ${selectedClaudeSupplier.value}`)
    addOperationLog('info', `Codex供应商: ${selectedCodexSupplier.value}`)
    addOperationLog('info', `MCP模板数量: ${selectedMcpTemplates.value.length}`)

    // 构建切换请求
    const request: WorkModeSwitchRequest = {
      targetMode: selectedMode.value,
      claudeSupplierId: selectedClaudeSupplier.value || undefined,
      codexSupplierId: selectedCodexSupplier.value || undefined,
      mcpTemplateIds: selectedMcpTemplates.value.length > 0 ? selectedMcpTemplates.value : undefined,
      createBackup: createBackup.value
    }

    addOperationLog('info', '正在执行模式切换...')

    // 执行切换
    const result = await modeApi.switchWorkMode(request)

    if (result.success) {
      // 更新最终进度
      if (switchProgress.value) {
        switchProgress.value.completedSteps = switchProgress.value.totalSteps
        switchProgress.value.overallProgress = 100
        switchProgress.value.isCompleted = true
      }

      addOperationLog('success', `成功切换到${getModeLabel(selectedMode.value)}`)
      addOperationLog('info', `耗时: ${result.duration ? `${result.duration}ms` : '未知'}`)
      addOperationLog('info', `应用配置: ${result.appliedConfigurations?.join(', ') || '无'}`)

      ElMessage.success(`成功切换到${getModeLabel(selectedMode.value)}`)
      await loadData() // 重新加载数据
    } else {
      throw new Error(result.message)
    }
  } catch (error) {
    addOperationLog('error', `模式切换失败: ${error}`)

    if (switchProgress.value) {
      switchProgress.value.hasError = true
    }

    ElMessage.error('模式切换失败')
    console.error(error)
  } finally {
    switching.value = false
    // 延迟关闭进度显示
    setTimeout(() => {
      if (switchProgress.value?.isCompleted) {
        addOperationLog('info', '切换流程完成，进度显示将在3秒后自动关闭')
        setTimeout(() => {
          switchProgress.value = null
          currentStep.value = null
        }, 3000)
      }
    }, 1000)
  }
}

// 页面加载时获取数据
onMounted(() => {
  loadData()
})
</script>

<template>
  <div class="mode-manager">
    <!-- 页面头部 -->
    <div class="header">
      <div class="header-left">
        <h2>模式管理</h2>
        <p class="subtitle">管理和切换AI工具工作模式</p>
      </div>
      <div class="header-right">
        <el-button type="primary" @click="showSwitchDialog" :disabled="switching">
          <el-icon><Refresh /></el-icon>
          切换模式
        </el-button>
        <el-button @click="showHistory">
          <el-icon><Clock /></el-icon>
          历史记录
        </el-button>
      </div>
    </div>

    <!-- 当前状态卡片 -->
    <div class="status-section">
      <el-row :gutter="20">
        <el-col :span="8">
          <el-card class="status-card">
            <div class="status-header">
              <h3>当前模式</h3>
              <el-tag :type="currentStatus?.isTransitioning ? 'warning' : 'success'">
                {{ currentStatus?.isTransitioning ? '切换中' : '正常' }}
              </el-tag>
            </div>
            <div class="status-content">
              <div class="mode-info">
                <span class="mode-name">{{ getModeLabel(currentStatus?.currentMode || '未知') }}</span>
                <span class="mode-time" v-if="currentStatus?.lastSwitchTime">
                  最后切换: {{ new Date(currentStatus.lastSwitchTime).toLocaleString() }}
                </span>
              </div>
            </div>
          </el-card>
        </el-col>
        
        <el-col :span="8">
          <el-card class="status-card">
            <div class="status-header">
              <h3>活跃供应商</h3>
            </div>
            <div class="status-content">
              <div class="supplier-info">
                <div class="supplier-item" v-if="currentStatus?.activeClaudeSupplier">
                  <span class="supplier-type">Claude:</span>
                  <span class="supplier-name">{{ currentStatus.activeClaudeSupplier }}</span>
                </div>
                <div class="supplier-item" v-if="currentStatus?.activeCodexSupplier">
                  <span class="supplier-type">Codex:</span>
                  <span class="supplier-name">{{ currentStatus.activeCodexSupplier }}</span>
                </div>
              </div>
            </div>
          </el-card>
        </el-col>
        
        <el-col :span="8">
          <el-card class="status-card">
            <div class="status-header">
              <h3>活跃模板</h3>
            </div>
            <div class="status-content">
              <div class="template-info">
                <span class="template-count">{{ currentStatus?.activeMcpTemplates.length || 0 }}</span>
                <span class="template-label">个MCP模板</span>
              </div>
            </div>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <!-- 进度显示 -->
    <div v-if="switching || switchProgress" class="progress-section">
      <el-card>
        <div class="progress-header">
          <h3>模式切换进度</h3>
          <div class="progress-actions">
            <el-button v-if="switching" size="small" @click="cancelSwitch" type="warning">
              <el-icon><Close /></el-icon>
              取消切换
            </el-button>
            <el-button size="small" @click="showDetails = !showDetails" text>
              <el-icon><View /></el-icon>
              {{ showDetails ? '收起详情' : '查看详情' }}
            </el-button>
          </div>
        </div>

        <!-- 总体进度条 -->
        <div class="overall-progress">
          <el-progress
            :percentage="progressPercentage"
            :status="switchProgress?.hasError ? 'exception' : (switchProgress?.isCompleted ? 'success' : undefined)"
            :stroke-width="8"
          >
            <template #default="{ percentage }">
              <span class="progress-text">{{ percentage }}%</span>
            </template>
          </el-progress>
          <div class="progress-time" v-if="switchProgress?.startTime">
            <span>开始时间: {{ new Date(switchProgress.startTime).toLocaleString() }}</span>
            <span v-if="switchProgress.estimatedTimeRemaining" class="time-remaining">
              预计剩余: {{ formatTime(switchProgress.estimatedTimeRemaining) }}
            </span>
          </div>
        </div>

        <!-- 当前步骤信息 -->
        <div class="current-step" v-if="currentStep">
          <el-alert
            :type="getStepAlertType(currentStep.status)"
            :title="currentStep.name"
            :description="currentStep.message"
            :closable="false"
            show-icon
          >
            <template #default>
              <div class="step-content">
                <p class="step-message" v-if="currentStep.message">{{ currentStep.message }}</p>
                <div class="step-timing" v-if="currentStep.startedAt">
                  <span>开始: {{ new Date(currentStep.startedAt).toLocaleTimeString() }}</span>
                  <span v-if="currentStep.completedAt">
                    完成: {{ new Date(currentStep.completedAt).toLocaleTimeString() }}
                  </span>
                </div>
              </div>
            </template>
          </el-alert>
        </div>

        <!-- 详细信息 -->
        <el-collapse-transition>
          <div v-show="showDetails" class="progress-details">
            <!-- 步骤时间线 -->
            <div class="steps-timeline">
              <h4>执行步骤</h4>
              <el-timeline>
                <el-timeline-item
                  v-for="step in getStepsList()"
                  :key="step.id"
                  :type="getStepStatusType(step.status)"
                  :icon="getStepIcon(step.status)"
                  :timestamp="step.startedAt ? new Date(step.startedAt).toLocaleTimeString() : ''"
                  placement="top"
                >
                  <div class="timeline-content">
                    <h5>{{ step.name }}</h5>
                    <p v-if="step.message" class="step-desc">{{ step.message }}</p>
                    <el-alert
                      v-if="step.error"
                      type="error"
                      :title="step.error"
                      :closable="false"
                      show-icon
                      size="small"
                    />
                  </div>
                </el-timeline-item>
              </el-timeline>
            </div>

            <!-- 操作日志 -->
            <div class="operation-log" v-if="operationLogs.length > 0">
              <h4>操作日志</h4>
              <div class="log-container">
                <div
                  v-for="log in operationLogs"
                  :key="log.id"
                  :class="['log-item', `log-${log.type}`]"
                >
                  <span class="log-time">{{ new Date(log.timestamp).toLocaleTimeString() }}</span>
                  <span class="log-message">{{ log.message }}</span>
                </div>
              </div>
            </div>

            <!-- 帮助信息 -->
            <div class="help-info">
              <h4>
                <el-icon><QuestionFilled /></el-icon>
                帮助信息
              </h4>
              <div class="help-content">
                <el-alert
                  type="info"
                  title="切换过程说明"
                  :closable="false"
                  show-icon
                >
                  <template #default>
                    <ul class="help-list">
                      <li>验证请求参数和配置完整性</li>
                      <li>获取供应商和MCP模板信息</li>
                      <li>备份当前配置文件</li>
                      <li>生成新的配置文件</li>
                      <li>应用新配置到系统</li>
                      <li>验证配置应用结果</li>
                      <li>更新数据库记录</li>
                    </ul>
                  </template>
                </el-alert>

                <el-alert
                  v-if="switchProgress?.hasError"
                  type="warning"
                  title="遇到问题？"
                  :closable="false"
                  show-icon
                >
                  <template #default>
                    <p>如果切换失败，系统会自动尝试回滚到之前的配置。</p>
                    <p>您也可以在历史记录中手动回滚到其他配置。</p>
                    <el-button size="small" @click="showTroubleshooting = true" type="primary" text>
                      查看故障排除指南
                    </el-button>
                  </template>
                </el-alert>
              </div>
            </div>
          </div>
        </el-collapse-transition>
      </el-card>
    </div>

    <!-- 配置列表 -->
    <div class="configs-section">
      <el-card>
        <template #header>
          <div class="configs-header">
            <h3>已配置模式</h3>
            <el-button size="small" @click="loadData">
              <el-icon><Refresh /></el-icon>
              刷新
            </el-button>
          </div>
        </template>
        
        <el-table :data="modeConfigs" v-loading="loading" stripe>
          <el-table-column prop="modeName" label="模式名称" min-width="120" />
          <el-table-column prop="modeName" label="模式类型" width="120">
            <template #default="{ row }">
              <el-tag :type="row.modeName === 'claude_only' ? 'primary' : row.modeName === 'codex_only' ? 'success' : 'warning'">
                {{ getModeLabel(row.modeName) }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column label="供应商配置" min-width="200">
            <template #default="{ row }">
              <div class="supplier-config">
                <span v-if="row.activeClaudeSupplierId" class="supplier-tag">
                  Claude: {{ row.activeClaudeSupplierId }}
                </span>
                <span v-if="row.activeCodexSupplierId" class="supplier-tag">
                  Codex: {{ row.activeCodexSupplierId }}
                </span>
              </div>
            </template>
          </el-table-column>
          <el-table-column label="模板数量" width="100">
            <template #default="{ row }">
              {{ row.mcpTemplateIds?.length || 0 }}
            </template>
          </el-table-column>
          <el-table-column prop="updatedAt" label="更新时间" width="180">
            <template #default="{ row }">
              {{ row.updatedAt ? new Date(row.updatedAt).toLocaleString() : '-' }}
            </template>
          </el-table-column>
          <el-table-column label="操作" width="150" fixed="right">
            <template #default="{ row }">
              <el-button size="small" link type="primary" @click="selectedMode = row.modeName; showSwitchDialog()">
                切换到此模式
              </el-button>
            </template>
          </el-table-column>
        </el-table>
      </el-card>
    </div>

    <!-- 模式切换对话框 -->
    <el-dialog v-model="switchDialogVisible" title="切换工作模式" width="600px" :close-on-click-modal="false">
      <el-form label-width="120px">
        <el-form-item label="目标模式">
          <el-radio-group v-model="selectedMode">
            <el-radio 
              v-for="option in modeOptions" 
              :key="option.value" 
              :value="option.value"
              class="mode-option"
            >
              <div class="option-content">
                <div class="option-title">{{ option.label }}</div>
                <div class="option-desc">{{ option.description }}</div>
              </div>
            </el-radio>
          </el-radio-group>
        </el-form-item>

        <el-form-item v-if="selectedMode === 'claude_only' || selectedMode === 'claude_codex'" label="Claude供应商">
          <el-select v-model="selectedClaudeSupplier" placeholder="请选择Claude供应商" style="width: 100%">
            <el-option
              v-for="supplier in claudeSuppliers"
              :key="supplier.id"
              :label="supplier.name"
              :value="supplier.id"
            />
          </el-select>
        </el-form-item>

        <el-form-item v-if="selectedMode === 'codex_only' || selectedMode === 'claude_codex'" label="Codex供应商">
          <el-select v-model="selectedCodexSupplier" placeholder="请选择Codex供应商" style="width: 100%">
            <el-option
              v-for="supplier in codexSuppliers"
              :key="supplier.id"
              :label="supplier.name"
              :value="supplier.id"
            />
          </el-select>
        </el-form-item>

        <el-form-item label="MCP模板">
          <el-select
            v-model="selectedMcpTemplates"
            multiple
            placeholder="请选择MCP模板"
            style="width: 100%"
          >
            <el-option
              v-for="template in mcpTemplates"
              :key="template.id"
              :label="template.name"
              :value="template.id"
            />
          </el-select>
        </el-form-item>

        <el-form-item>
          <el-checkbox v-model="createBackup">
            切换前创建备份
          </el-checkbox>
        </el-form-item>
      </el-form>

      <template #footer>
        <span class="dialog-footer">
          <el-button @click="switchDialogVisible = false">取消</el-button>
          <el-button 
            type="primary" 
            @click="executeSwitch" 
            :disabled="!isFormValid || switching"
            :loading="switching"
          >
            {{ switching ? '切换中...' : '确认切换' }}
          </el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 历史记录对话框 -->
    <el-dialog v-model="historyVisible" title="切换历史" width="800px">
      <el-table :data="switchHistory" stripe>
        <el-table-column prop="timestamp" label="时间" width="180" />
        <el-table-column prop="fromMode" label="从模式" width="120" />
        <el-table-column prop="toMode" label="到模式" width="120" />
        <el-table-column prop="success" label="状态" width="80">
          <template #default="{ row }">
            <el-tag :type="row.success ? 'success' : 'danger'">
              {{ row.success ? '成功' : '失败' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="duration" label="耗时" width="100">
          <template #default="{ row }">
            {{ row.duration ? `${row.duration}ms` : '-' }}
          </template>
        </el-table-column>
        <el-table-column label="操作" width="100" fixed="right">
          <template #default="{ row }">
            <el-button 
              v-if="row.backupId" 
              size="small" 
              link 
              type="warning"
              @click="rollbackMode(row.backupId)"
            >
              回滚
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-dialog>

    <!-- 故障排除指南对话框 -->
    <el-dialog
      v-model="showTroubleshooting"
      title="故障排除指南"
      width="700px"
      :close-on-click-modal="false"
    >
      <div class="troubleshooting-content">
        <el-tabs v-model="troubleshootingTab" type="border-card">
          <el-tab-pane label="常见问题" name="common">
            <div class="troubleshooting-section">
              <h4>切换失败常见原因</h4>
              <el-collapse accordion>
                <el-collapse-item title="配置文件权限问题" name="1">
                  <div>
                    <p><strong>问题表现：</strong>无法写入配置文件</p>
                    <p><strong>解决方案：</strong></p>
                    <ul>
                      <li>检查应用是否有配置目录的写入权限</li>
                      <li>确保配置文件没有被其他程序占用</li>
                      <li>尝试以管理员权限运行应用</li>
                    </ul>
                  </div>
                </el-collapse-item>

                <el-collapse-item title="供应商连接失败" name="2">
                  <div>
                    <p><strong>问题表现：</strong>无法连接到指定的AI供应商</p>
                    <p><strong>解决方案：</strong></p>
                    <ul>
                      <li>检查供应商API密钥是否正确</li>
                      <li>验证网络连接是否正常</li>
                      <li>确认供应商服务是否可用</li>
                      <li>检查防火墙设置</li>
                    </ul>
                  </div>
                </el-collapse-item>

                <el-collapse-item title="MCP模板错误" name="3">
                  <div>
                    <p><strong>问题表现：</strong>MCP服务器模板配置有误</p>
                    <p><strong>解决方案：</strong></p>
                    <ul>
                      <li>检查模板语法是否正确</li>
                      <li>验证MCP服务器路径和参数</li>
                      <li>确保所需的依赖包已安装</li>
                      <li>查看MCP服务器日志获取详细错误信息</li>
                    </ul>
                  </div>
                </el-collapse-item>

                <el-collapse-item title="配置格式错误" name="4">
                  <div>
                    <p><strong>问题表现：</strong>生成的配置文件格式不正确</p>
                    <p><strong>解决方案：</strong></p>
                    <ul>
                      <li>检查配置模板语法</li>
                      <li>验证变量替换是否正确</li>
                      <li>使用配置验证工具检查格式</li>
                      <li>参考官方配置文档</li>
                    </ul>
                  </div>
                </el-collapse-item>
              </el-collapse>
            </div>
          </el-tab-pane>

          <el-tab-pane label="联系支持" name="support">
            <div class="troubleshooting-section">
              <h4>获取技术支持</h4>

              <div class="diagnostic-info">
                <h5>系统诊断信息</h5>
                <el-descriptions :column="2" size="small" border>
                  <el-descriptions-item label="应用版本">
                    v1.0.0
                  </el-descriptions-item>
                  <el-descriptions-item label="操作系统">
                    {{ getPlatformInfo() }}
                  </el-descriptions-item>
                  <el-descriptions-item label="当前模式">
                    {{ currentStatus?.currentMode || '未知' }}
                  </el-descriptions-item>
                  <el-descriptions-item label="最后切换">
                    {{ currentStatus?.lastSwitchTime ? new Date(currentStatus.lastSwitchTime).toLocaleString() : '无记录' }}
                  </el-descriptions-item>
                </el-descriptions>
              </div>
            </div>
          </el-tab-pane>
        </el-tabs>
      </div>

      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showTroubleshooting = false">关闭</el-button>
          <el-button type="primary" @click="copyDiagnosticInfo">
            复制诊断信息
          </el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
.mode-manager {
  padding: 20px;
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}

.header-left h2 {
  margin: 0 0 8px 0;
  color: #333;
  font-size: 24px;
  font-weight: 600;
}

.subtitle {
  margin: 0;
  color: #666;
  font-size: 14px;
}

.header-right {
  display: flex;
  gap: 12px;
}

.status-section {
  margin-bottom: 20px;
}

.status-card {
  height: 120px;
}

.status-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.status-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 500;
}

.status-content {
  height: 60px;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.mode-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.mode-name {
  font-size: 18px;
  font-weight: 600;
  color: #333;
}

.mode-time {
  font-size: 12px;
  color: #999;
}

.supplier-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.supplier-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.supplier-type {
  font-size: 12px;
  color: #666;
  background: #f0f0f0;
  padding: 2px 6px;
  border-radius: 4px;
}

.supplier-name {
  font-size: 14px;
  color: #333;
}

.template-info {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  gap: 4px;
}

.template-count {
  font-size: 24px;
  font-weight: 600;
  color: #409eff;
}

.template-label {
  font-size: 14px;
  color: #666;
}

.progress-section {
  margin-bottom: 20px;
}

.progress-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.step-info {
  margin-top: 16px;
  text-align: center;
}

.step-name {
  font-size: 16px;
  font-weight: 500;
  color: #333;
  margin: 0 0 8px 0;
}

.step-message {
  font-size: 14px;
  color: #666;
  margin: 0;
}

.configs-section {
  flex: 1;
}

.configs-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.supplier-config {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.supplier-tag {
  font-size: 12px;
  background: #f0f0f0;
  padding: 2px 6px;
  border-radius: 4px;
}

.mode-option {
  width: 100%;
  margin-bottom: 12px;
}

.option-content {
  margin-left: 24px;
}

.option-title {
  font-weight: 500;
  color: #333;
  margin-bottom: 4px;
}

.option-desc {
  font-size: 12px;
  color: #666;
  line-height: 1.4;
}

.dialog-footer {
  text-align: right;
}

/* 表格样式优化 */
:deep(.el-table) {
  font-size: 14px;
}

:deep(.el-table th) {
  background-color: #f5f7fa;
  color: #606266;
  font-weight: 600;
}

:deep(.el-table td) {
  padding: 12px 0;
}

/* 表单样式优化 */
:deep(.el-form-item__label) {
  font-weight: 500;
}

:deep(.el-radio) {
  margin-right: 0;
  margin-bottom: 12px;
}

:deep(.el-radio__label) {
  width: 100%;
}

/* ========== 进度显示和用户反馈样式 ========== */

/* 进度区域样式 */
.progress-section {
  margin-bottom: 20px;
}

.progress-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.progress-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

.overall-progress {
  margin-bottom: 20px;
}

.progress-text {
  font-weight: 600;
  color: #409eff;
}

.progress-time {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 8px;
  font-size: 12px;
  color: #666;
}

.time-remaining {
  color: #e6a23c;
  font-weight: 500;
}

.current-step {
  margin-bottom: 20px;
}

.step-content {
  margin-top: 8px;
}

.step-message {
  margin: 4px 0 0 0;
  color: #666;
  font-size: 14px;
}

.step-timing {
  display: flex;
  gap: 16px;
  margin-top: 8px;
  font-size: 12px;
  color: #999;
}

.progress-details {
  margin-top: 20px;
  border-top: 1px solid #e4e7ed;
  padding-top: 20px;
}

/* 步骤时间线样式 */
.steps-timeline,
.operation-log,
.help-info {
  margin-bottom: 30px;
}

.steps-timeline h4,
.operation-log h4,
.help-info h4 {
  margin: 0 0 16px 0;
  font-size: 16px;
  font-weight: 600;
  color: #333;
  display: flex;
  align-items: center;
  gap: 8px;
}

.timeline-content h5 {
  margin: 0 0 8px 0;
  font-size: 14px;
  font-weight: 500;
  color: #333;
}

.step-desc {
  margin: 4px 0 0 0;
  color: #666;
  font-size: 13px;
  line-height: 1.4;
}

/* 操作日志样式 */
.log-container {
  max-height: 300px;
  overflow-y: auto;
  background: #f8f9fa;
  border-radius: 6px;
  padding: 12px;
}

.log-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 8px 0;
  border-bottom: 1px solid #e9ecef;
  font-size: 13px;
}

.log-item:last-child {
  border-bottom: none;
}

.log-time {
  color: #999;
  font-family: monospace;
  white-space: nowrap;
  font-size: 12px;
  min-width: 80px;
}

.log-message {
  flex: 1;
  line-height: 1.4;
}

.log-info .log-message {
  color: #666;
}

.log-success .log-message {
  color: #67c23a;
}

.log-warning .log-message {
  color: #e6a23c;
}

.log-error .log-message {
  color: #f56c6c;
  font-weight: 500;
}

/* 帮助信息样式 */
.help-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.help-list {
  margin: 0;
  padding-left: 20px;
}

.help-list li {
  margin-bottom: 8px;
  line-height: 1.5;
  color: #666;
}

.help-list li:last-child {
  margin-bottom: 0;
}

/* 故障排除对话框样式 */
.troubleshooting-content {
  max-height: 600px;
  overflow-y: auto;
}

.troubleshooting-section {
  padding: 16px 0;
}

.troubleshooting-section h4 {
  margin: 0 0 16px 0;
  font-size: 16px;
  font-weight: 600;
  color: #333;
}

.troubleshooting-section h5 {
  margin: 16px 0 12px 0;
  font-size: 14px;
  font-weight: 600;
  color: #333;
}

.diagnostic-info {
  margin-top: 20px;
  padding: 16px;
  background: #f8f9fa;
  border-radius: 6px;
}

/* 支持选项样式 */
.support-options {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 16px;
  margin-bottom: 24px;
}

.support-card {
  transition: all 0.3s ease;
}

.support-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.support-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
}

.manual-actions {
  margin-top: 24px;
}

.action-buttons {
  display: flex;
  gap: 12px;
  margin-top: 16px;
  flex-wrap: wrap;
}

/* 动画效果 */
:deep(.el-progress-bar__outer) {
  border-radius: 4px;
  overflow: hidden;
}

:deep(.el-progress-bar__inner) {
  transition: width 0.3s ease;
}

:deep(.el-timeline-item__tail) {
  border-left-width: 2px;
}

:deep(.el-collapse-transition) {
  transition: all 0.3s ease;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .progress-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .progress-actions {
    width: 100%;
    justify-content: flex-end;
  }

  .progress-time {
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
  }

  .support-options {
    grid-template-columns: 1fr;
  }

  .action-buttons {
    flex-direction: column;
  }

  .step-timing {
    flex-direction: column;
    gap: 4px;
  }
}

/* 深色模式支持 */
@media (prefers-color-scheme: dark) {
  .log-container {
    background: #2c2c2c;
  }

  .log-item {
    border-bottom-color: #404040;
  }

  .diagnostic-info {
    background: #2c2c2c;
  }
}

/* 打印样式 */
@media print {
  .progress-actions,
  .action-buttons {
    display: none;
  }

  .troubleshooting-content {
    max-height: none;
    overflow: visible;
  }
}
</style>
