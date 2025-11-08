import type {
  ApiResponse,
  WorkModeConfig,
  CreateWorkModeRequest,
  UpdateWorkModeRequest,
  WorkModeSwitchRequest,
  WorkModeStatus,
  WorkModeSwitchResult,
  WorkModeProgress,
  WorkModeSwitchStep,
  Supplier,
  McpTemplate,
  ConfigType,
  ConfigBackup,
  ConfigOperationResult
} from '@/types'
import '@/types/tauri'
import { configTemplateEngine } from './configTemplateEngine'
import { configFileManager } from './configFileManager'
import { supplierApi } from './supplierApi'
import { mcpTemplateApi } from './mcpTemplateApi'

class ModeApiService {
  // 根据名称获取工作模式配置
  async getWorkModeConfig(modeName: string): Promise<WorkModeConfig | null> {
    try {
      const result = await window.__TAURI__.invoke('get_work_mode_by_name', {
        modeName
      })
      return result.data || null
    } catch (error) {
      console.error('获取工作模式配置失败:', error)
      throw error
    }
  }

  // 获取所有工作模式配置
  async getAllWorkModeConfigs(): Promise<WorkModeConfig[]> {
    try {
      const result = await window.__TAURI__.invoke('list_work_mode_configs')
      return result.data || []
    } catch (error) {
      console.error('获取所有工作模式配置失败:', error)
      throw error
    }
  }

  // 更新工作模式配置
  async updateWorkModeConfig(request: UpdateWorkModeRequest): Promise<WorkModeConfig | null> {
    try {
      const result = await window.__TAURI__.invoke('update_work_mode_by_id', request)
      if (result.success) {
        return result.data
      } else {
        throw new Error(result.message || '更新工作模式配置失败')
      }
    } catch (error) {
      console.error('更新工作模式配置失败:', error)
      throw error
    }
  }

  // 切换工作模式（增强版，集成配置管理）
  async switchWorkMode(request: WorkModeSwitchRequest): Promise<WorkModeSwitchResult> {
    const stepsCompleted: string[] = []
    const startTime = new Date()

    try {
      // 步骤1：验证请求参数
      stepsCompleted.push('验证请求参数')
      await this.validateSwitchRequest(request)

      // 步骤2：获取供应商和模板信息
      stepsCompleted.push('获取供应商和模板信息')
      const { claudeSupplier, codexSupplier, templates } = await this.getModeComponents(request)

      // 步骤3：备份当前配置
      stepsCompleted.push('备份当前配置')
      const backupResults = await this.backupCurrentConfigurations(request.targetMode)

      // 步骤4：生成新配置
      stepsCompleted.push('生成新配置')
      const newConfigs = await this.generateNewConfigurations(request, claudeSupplier, codexSupplier, templates)

      // 步骤5：应用新配置
      stepsCompleted.push('应用新配置')
      const applyResults = await this.applyNewConfigurations(newConfigs, request.createBackup)

      // 步骤6：验证配置应用结果
      stepsCompleted.push('验证配置应用结果')
      await this.validateAppliedConfigurations(newConfigs)

      // 步骤7：更新数据库中的工作模式
      stepsCompleted.push('更新数据库中的工作模式')
      const dbResult = await window.__TAURI__.invoke('switch_work_mode', request)

      if (!dbResult.success) {
        throw new Error(dbResult.message || '数据库更新失败')
      }

      const endTime = new Date()
      const duration = endTime.getTime() - startTime.getTime()

      return {
        success: true,
        message: `成功切换到${this.getModeLabel(request.targetMode)}模式`,
        backupId: backupResults.backupId,
        appliedAt: endTime.toISOString(),
        stepsCompleted,
        duration: duration,
        appliedConfigurations: Object.keys(newConfigs)
      }
    } catch (error) {
      console.error('切换工作模式失败:', error)

      // 尝试回滚配置
      try {
        await this.rollbackFailedSwitch(request.targetMode)
        stepsCompleted.push('执行配置回滚')
      } catch (rollbackError) {
        console.error('配置回滚失败:', rollbackError)
      }

      return {
        success: false,
        message: `模式切换失败: ${error}`,
        appliedAt: new Date().toISOString(),
        stepsCompleted,
        error: error as string
      }
    }
  }

  // 获取当前工作模式状态
  async getWorkModeStatus(): Promise<WorkModeStatus> {
    try {
      const result = await window.__TAURI__.invoke('get_work_mode_status')
      if (result.success) {
        return result.data
      } else {
        throw new Error(result.message || '获取工作模式状态失败')
      }
    } catch (error) {
      console.error('获取工作模式状态失败:', error)
      throw error
    }
  }

  // 回滚工作模式
  async rollbackWorkMode(backupId: number): Promise<boolean> {
    try {
      const result = await window.__TAURI__.invoke('rollback_work_mode', {
        backupId
      })
      return result.success && result.data
    } catch (error) {
      console.error('回滚工作模式失败:', error)
      throw error
    }
  }

  // 创建新的工作模式配置
  async createWorkModeConfig(request: CreateWorkModeRequest): Promise<WorkModeConfig> {
    try {
      // 暂时使用update命令，后续可以添加专门的create命令
      const result = await window.__TAURI__.invoke('update_work_mode_by_id', {
        id: 0, // 临时ID，后端会处理
        ...request
      })
      if (result.success) {
        return result.data
      } else {
        throw new Error(result.message || '创建工作模式配置失败')
      }
    } catch (error) {
      console.error('创建工作模式配置失败:', error)
      throw error
    }
  }

  // 删除工作模式配置
  async deleteWorkModeConfig(id: number): Promise<boolean> {
    try {
      // 暂时返回true，后续需要添加删除命令
      console.warn('删除工作模式配置功能待实现')
      return true
    } catch (error) {
      console.error('删除工作模式配置失败:', error)
      throw error
    }
  }

  // 检查工作模式配置是否有效
  async validateWorkModeConfig(config: WorkModeConfig): Promise<boolean> {
    try {
      // 基本验证
      if (!config.modeName || config.modeName.trim() === '') {
        return false
      }

      // 检查是否至少有一个供应商配置
      if (!config.activeClaudeSupplierId && !config.activeCodexSupplierId) {
        return false
      }

      return true
    } catch (error) {
      console.error('验证工作模式配置失败:', error)
      return false
    }
  }

  // 获取工作模式配置的统计信息
  async getWorkModeStats(): Promise<{
    total: number
    claudeOnly: number
    codexOnly: number
    mixed: number
  }> {
    try {
      const configs = await this.getAllWorkModeConfigs()
      
      const stats = {
        total: configs.length,
        claudeOnly: 0,
        codexOnly: 0,
        mixed: 0
      }

      configs.forEach(config => {
        if (config.activeClaudeSupplierId && config.activeCodexSupplierId) {
          stats.mixed++
        } else if (config.activeClaudeSupplierId) {
          stats.claudeOnly++
        } else if (config.activeCodexSupplierId) {
          stats.codexOnly++
        }
      })

      return stats
    } catch (error) {
      console.error('获取工作模式统计失败:', error)
      return {
        total: 0,
        claudeOnly: 0,
        codexOnly: 0,
        mixed: 0
      }
    }
  }

  // 获取推荐的工作模式配置
  async getRecommendedWorkModeConfig(): Promise<WorkModeSwitchRequest> {
    try {
      const status = await this.getWorkModeStatus()

      // 基于当前状态推荐配置
      const request: WorkModeSwitchRequest = {
        targetMode: status.currentMode,
        claudeSupplierId: undefined, // 需要从活跃供应商中获取
        codexSupplierId: undefined, // 需要从活跃供应商中获取
        mcpTemplateIds: [], // 需要从当前模式中获取
        createBackup: true
      }

      return request
    } catch (error) {
      console.error('获取推荐工作模式配置失败:', error)
      throw error
    }
  }

  // ========== 配置集成相关方法 ==========

  /**
   * 验证切换请求参数
   */
  private async validateSwitchRequest(request: WorkModeSwitchRequest): Promise<void> {
    if (!request.targetMode) {
      throw new Error('目标模式不能为空')
    }

    const validModes = ['claude_only', 'codex_only', 'claude_codex']
    if (!validModes.includes(request.targetMode)) {
      throw new Error(`不支持的目标模式: ${request.targetMode}`)
    }

    // 验证供应商配置
    if (request.targetMode === 'claude_only' && !request.claudeSupplierId) {
      throw new Error('Claude模式需要指定Claude供应商')
    }

    if (request.targetMode === 'codex_only' && !request.codexSupplierId) {
      throw new Error('Codex模式需要指定Codex供应商')
    }

    if (request.targetMode === 'claude_codex' && (!request.claudeSupplierId || !request.codexSupplierId)) {
      throw new Error('混合模式需要同时指定Claude和Codex供应商')
    }
  }

  /**
   * 获取模式组件（供应商和模板）
   */
  private async getModeComponents(request: WorkModeSwitchRequest): Promise<{
    claudeSupplier?: Supplier
    codexSupplier?: Supplier
    templates: McpTemplate[]
  }> {
    const templates: McpTemplate[] = []
    let claudeSupplier: Supplier | undefined
    let codexSupplier: Supplier | undefined

    // 获取供应商信息
    if (request.claudeSupplierId) {
      const supplier = await supplierApi.getSupplierById(request.claudeSupplierId)
      if (!supplier) {
        throw new Error(`找不到Claude供应商 (ID: ${request.claudeSupplierId})`)
      }
      claudeSupplier = supplier
    }

    if (request.codexSupplierId) {
      const supplier = await supplierApi.getSupplierById(request.codexSupplierId)
      if (!supplier) {
        throw new Error(`找不到Codex供应商 (ID: ${request.codexSupplierId})`)
      }
      codexSupplier = supplier
    }

    // 获取MCP模板
    if (request.mcpTemplateIds && request.mcpTemplateIds.length > 0) {
      for (const templateId of request.mcpTemplateIds) {
        // 这里需要实现根据ID获取模板的方法
        // const template = await mcpTemplateApi.getMcpTemplateById(templateId)
        // if (template) {
        //   templates.push(template)
        // }
      }
    } else {
      // 如果没有指定模板，获取所有相关类型的模板
      const allTemplates = await mcpTemplateApi.listMcpTemplates()

      if (request.targetMode === 'claude_only' || request.targetMode === 'claude_codex') {
        templates.push(...allTemplates.filter(t => t.aiType === 'claude'))
      }

      if (request.targetMode === 'codex_only' || request.targetMode === 'claude_codex') {
        templates.push(...allTemplates.filter(t => t.aiType === 'codex'))
      }
    }

    return { claudeSupplier, codexSupplier, templates }
  }

  /**
   * 备份当前配置
   */
  private async backupCurrentConfigurations(targetMode: string): Promise<{ backupId?: number; results: ConfigOperationResult[] }> {
    const results: ConfigOperationResult[] = []
    let backupId: number | undefined

    try {
      // 确定需要备份的配置类型
      const configTypesToBackup: ConfigType[] = []

      if (targetMode === 'claude_only' || targetMode === 'claude_codex') {
        configTypesToBackup.push('claude')
      }

      if (targetMode === 'codex_only' || targetMode === 'claude_codex') {
        configTypesToBackup.push('codex')
      }

      // 备份每种配置类型
      for (const configType of configTypesToBackup) {
        try {
          // 读取当前配置内容
          const currentConfigPath = configFileManager.getConfigPath(configType)
          // 这里需要实现读取现有配置的逻辑
          // const currentContent = await this.readCurrentConfig(configType)

          // 暂时使用空内容作为示例
          const currentContent = `// 当前${configType}配置备份 - ${new Date().toISOString()}`

          const result = await configFileManager.backupConfig(
            configType,
            currentContent,
            `切换到${targetMode}模式前自动备份`
          )

          results.push(result)

          if (result.success && result.data?.backupId) {
            backupId = result.data.backupId
          }
        } catch (error) {
          results.push({
            success: false,
            message: `备份${configType}配置失败: ${error}`
          })
        }
      }

      return { backupId, results }
    } catch (error) {
      console.error('备份配置失败:', error)
      return { results }
    }
  }

  /**
   * 生成新配置
   */
  private async generateNewConfigurations(
    request: WorkModeSwitchRequest,
    claudeSupplier?: Supplier,
    codexSupplier?: Supplier,
    templates: McpTemplate[] = []
  ): Promise<Record<ConfigType, string>> {
    const configs: Record<string, string> = {}

    try {
      // 生成Claude配置
      if (claudeSupplier && (request.targetMode === 'claude_only' || request.targetMode === 'claude_codex')) {
        const claudeTemplates = templates.filter(t => t.aiType === 'claude')
        const claudeConfig = await configTemplateEngine.generateClaudeConfig(
          claudeSupplier,
          claudeTemplates,
          {
            MODE: request.targetMode,
            SWITCH_TIME: new Date().toISOString(),
            TEMPLATE_COUNT: claudeTemplates.length.toString()
          }
        )

        if (claudeConfig.success) {
          configs.claude = claudeConfig.content
        } else {
          throw new Error(`生成Claude配置失败: ${claudeConfig.errors?.join(', ')}`)
        }
      }

      // 生成Codex配置
      if (codexSupplier && (request.targetMode === 'codex_only' || request.targetMode === 'claude_codex')) {
        const codexTemplates = templates.filter(t => t.aiType === 'codex')
        const codexConfig = await configTemplateEngine.generateCodexConfig(
          codexSupplier,
          codexTemplates,
          {
            MODE: request.targetMode,
            SWITCH_TIME: new Date().toISOString(),
            TEMPLATE_COUNT: codexTemplates.length.toString()
          }
        )

        if (codexConfig.success) {
          configs.codex = codexConfig.content
        } else {
          throw new Error(`生成Codex配置失败: ${codexConfig.errors?.join(', ')}`)
        }
      }

      return configs as Record<ConfigType, string>
    } catch (error) {
      console.error('生成新配置失败:', error)
      throw error
    }
  }

  /**
   * 应用新配置
   */
  private async applyNewConfigurations(
    configs: Record<ConfigType, string>,
    createBackup: boolean
  ): Promise<Record<ConfigType, ConfigOperationResult>> {
    const results: Record<string, ConfigOperationResult> = {}

    try {
      for (const [configType, content] of Object.entries(configs)) {
        try {
          const result = await configFileManager.applyConfig(
            configType as ConfigType,
            content,
            createBackup
          )
          results[configType] = result
        } catch (error) {
          results[configType] = {
            success: false,
            message: `应用${configType}配置失败: ${error}`
          }
        }
      }

      return results as Record<ConfigType, ConfigOperationResult>
    } catch (error) {
      console.error('应用新配置失败:', error)
      throw error
    }
  }

  /**
   * 验证应用的配置
   */
  private async validateAppliedConfigurations(configs: Record<ConfigType, string>): Promise<void> {
    try {
      for (const [configType, expectedContent] of Object.entries(configs)) {
        // 验证配置格式
        const validation = configFileManager.validateConfigFormat(expectedContent, configType as ConfigType)
        if (!validation.valid) {
          throw new Error(`${configType}配置格式验证失败: ${validation.error}`)
        }

        // 这里可以添加更多的验证逻辑，比如：
        // 1. 检查配置文件是否真的写入磁盘
        // 2. 验证关键配置项是否存在
        // 3. 检查配置语法是否正确
      }
    } catch (error) {
      console.error('验证应用配置失败:', error)
      throw error
    }
  }

  /**
   * 回滚失败的切换
   */
  private async rollbackFailedSwitch(targetMode: string): Promise<void> {
    try {
      // 获取最新的备份并恢复
      const configTypes: ConfigType[] = []

      if (targetMode === 'claude_only' || targetMode === 'claude_codex') {
        configTypes.push('claude')
      }

      if (targetMode === 'codex_only' || targetMode === 'claude_codex') {
        configTypes.push('codex')
      }

      for (const configType of configTypes) {
        const latestBackup = await configFileManager.getLatestConfigBackup(configType)
        if (latestBackup) {
          const restoreResult = await configFileManager.restoreConfigFromBackup(latestBackup.id!)
          if (!restoreResult.success) {
            console.warn(`恢复${configType}配置失败:`, restoreResult.message)
          }
        }
      }
    } catch (error) {
      console.error('回滚配置失败:', error)
      throw error
    }
  }

  /**
   * 获取模式标签
   */
  private getModeLabel(mode: string): string {
    const modeLabels: Record<string, string> = {
      'claude_only': 'Claude专用',
      'codex_only': 'Codex专用',
      'claude_codex': '混合模式'
    }
    return modeLabels[mode] || mode
  }

  /**
   * 预览模式配置
   */
  async previewModeConfig(request: WorkModeSwitchRequest): Promise<{
    success: boolean
    configs?: Record<ConfigType, { content: string; format: string; variables: string[] }>
    message?: string
  }> {
    try {
      // 验证请求
      await this.validateSwitchRequest(request)

      // 获取组件
      const { claudeSupplier, codexSupplier, templates } = await this.getModeComponents(request)

      // 生成配置
      const configs: Record<string, { content: string; format: string; variables: string[] }> = {}

      if (claudeSupplier && (request.targetMode === 'claude_only' || request.targetMode === 'claude_codex')) {
        const claudeTemplates = templates.filter(t => t.aiType === 'claude')
        const result = await configTemplateEngine.generateClaudeConfig(
          claudeSupplier,
          claudeTemplates,
          { MODE: request.targetMode }
        )

        if (result.success) {
          configs.claude = {
            content: result.content,
            format: result.format,
            variables: configTemplateEngine.extractVariables(result.content)
          }
        }
      }

      if (codexSupplier && (request.targetMode === 'codex_only' || request.targetMode === 'claude_codex')) {
        const codexTemplates = templates.filter(t => t.aiType === 'codex')
        const result = await configTemplateEngine.generateCodexConfig(
          codexSupplier,
          codexTemplates,
          { MODE: request.targetMode }
        )

        if (result.success) {
          configs.codex = {
            content: result.content,
            format: result.format,
            variables: configTemplateEngine.extractVariables(result.content)
          }
        }
      }

      return {
        success: true,
        configs: configs as Record<ConfigType, { content: string; format: string; variables: string[] }>
      }
    } catch (error) {
      return {
        success: false,
        message: `预览配置失败: ${error}`
      }
    }
  }

  /**
   * 检测配置冲突
   */
  async detectConfigConflicts(request: WorkModeSwitchRequest): Promise<{
    hasConflicts: boolean
    conflicts: Array<{ type: string; description: string; severity: 'low' | 'medium' | 'high' }>
  }> {
    const conflicts: Array<{ type: string; description: string; severity: 'low' | 'medium' | 'high' }> = []

    try {
      // 检查供应商冲突
      if (request.claudeSupplierId === request.codexSupplierId) {
        conflicts.push({
          type: 'supplier_conflict',
          description: 'Claude和Codex使用了相同的供应商',
          severity: 'medium'
        })
      }

      // 检查模板冲突
      if (request.mcpTemplateIds) {
        const duplicateIds = request.mcpTemplateIds.filter((id, index) =>
          request.mcpTemplateIds!.indexOf(id) !== index
        )
        if (duplicateIds.length > 0) {
          conflicts.push({
            type: 'template_duplicate',
            description: '存在重复的MCP模板',
            severity: 'low'
          })
        }
      }

      // 检查路径冲突
      const configTypes: ConfigType[] = []
      if (request.targetMode === 'claude_only' || request.targetMode === 'claude_codex') {
        configTypes.push('claude')
      }
      if (request.targetMode === 'codex_only' || request.targetMode === 'claude_codex') {
        configTypes.push('codex')
      }

      // 这里可以添加更多的冲突检测逻辑

      return {
        hasConflicts: conflicts.length > 0,
        conflicts
      }
    } catch (error) {
      console.error('检测配置冲突失败:', error)
      return {
        hasConflicts: false,
        conflicts: []
      }
    }
  }
}

export const modeApi = new ModeApiService()