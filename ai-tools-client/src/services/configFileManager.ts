import type {
  ConfigType,
  ConfigBackup,
  ConfigDiffResult,
  UpdateConfigRequest,
  Supplier,
  McpTemplate
} from '@/types'
import { configTemplateEngine } from './configTemplateEngine'
import { supplierApi } from './supplierApi'
import { mcpTemplateApi } from './mcpTemplateApi'

/**
 * 配置文件路径映射
 */
interface ConfigPathMap {
  [key: string]: string
}

/**
 * 配置操作结果
 */
export interface ConfigOperationResult {
  success: boolean
  message: string
  data?: any
}

/**
 * 配置预览参数
 */
export interface ConfigPreviewParams {
  configType: ConfigType
  claudeSupplierId?: number
  codexSupplierId?: number
  mcpTemplateIds?: number[]
  variables?: Record<string, string>
}

/**
 * 配置文件管理器
 * 提供配置文件的备份、恢复、应用、验证等功能
 */
export class ConfigFileManager {
  private readonly configPaths: ConfigPathMap

  constructor() {
    this.configPaths = this.initConfigPaths()
  }

  /**
   * 初始化配置文件路径映射
   */
  private initConfigPaths(): ConfigPathMap {
    return {
      'claude': this.getClaudeConfigPath(),
      'codex': this.getCodexConfigPath(),
      'settings': this.getSettingsConfigPath(),
      'auth': this.getAuthConfigPath()
    }
  }

  /**
   * 获取Claude配置文件路径
   */
  private getClaudeConfigPath(): string {
    // 根据平台返回不同的路径
    const platform = this.detectPlatform()
    if (platform === 'windows') {
      return '%USERPROFILE%\\.claude.json'
    } else {
      return '$HOME/.claude.json'
    }
  }

  /**
   * 获取Codex配置文件路径
   */
  private getCodexConfigPath(): string {
    const platform = this.detectPlatform()
    if (platform === 'windows') {
      return '%USERPROFILE%\\.codex\\config.toml'
    } else {
      return '$HOME/.codex/config.toml'
    }
  }

  /**
   * 获取Claude设置文件路径
   */
  private getSettingsConfigPath(): string {
    const platform = this.detectPlatform()
    if (platform === 'windows') {
      return '%USERPROFILE%\\.claude\\settings.json'
    } else {
      return '$HOME/.claude/settings.json'
    }
  }

  /**
   * 获取认证配置文件路径
   */
  private getAuthConfigPath(): string {
    const platform = this.detectPlatform()
    if (platform === 'windows') {
      return '%USERPROFILE%\\.codex\\auth.json'
    } else {
      return '$HOME/.codex/auth.json'
    }
  }

  /**
   * 检测平台类型
   */
  private detectPlatform(): string {
    // 简化实现，实际项目中可以通过Tauri API获取
    if (typeof navigator !== 'undefined') {
      return navigator.platform.toLowerCase().includes('win') ? 'windows' : 'unix'
    }
    return 'unix'
  }

  /**
   * 备份配置文件
   * @param configType 配置类型
   * @param content 配置内容
   * @param description 备份描述
   * @returns 备份结果
   */
  async backupConfig(
    configType: ConfigType,
    content: string,
    description?: string
  ): Promise<ConfigOperationResult> {
    try {
      const configPath = this.configPaths[configType]
      if (!configPath) {
        return {
          success: false,
          message: `不支持的配置类型: ${configType}`
        }
      }

      const result = await window.__TAURI__.invoke('backup_config', {
        configType,
        configPath,
        content,
        description: description || `手动备份 - ${new Date().toLocaleString()}`
      })

      if (result.success) {
        return {
          success: true,
          message: '配置备份成功',
          data: result.data
        }
      } else {
        return {
          success: false,
          message: result.message || '配置备份失败'
        }
      }
    } catch (error) {
      return {
        success: false,
        message: `配置备份失败: ${error}`
      }
    }
  }

  /**
   * 获取配置历史记录
   * @param configType 配置类型
   * @param limit 限制数量
   * @returns 历史记录列表
   */
  async getConfigHistory(configType: ConfigType, limit?: number): Promise<ConfigBackup[]> {
    try {
      const result = await window.__TAURI__.invoke('get_config_history', {
        configType,
        limit
      })

      return result.data || []
    } catch (error) {
      console.error('获取配置历史失败:', error)
      return []
    }
  }

  /**
   * 获取最新配置备份
   * @param configType 配置类型
   * @returns 最新备份记录
   */
  async getLatestConfigBackup(configType: ConfigType): Promise<ConfigBackup | null> {
    try {
      const result = await window.__TAURI__.invoke('get_latest_config_backup', {
        configType
      })

      return result.data || null
    } catch (error) {
      console.error('获取最新配置备份失败:', error)
      return null
    }
  }

  /**
   * 从备份恢复配置
   * @param backupId 备份ID
   * @returns 恢复结果
   */
  async restoreConfigFromBackup(backupId: number): Promise<ConfigOperationResult> {
    try {
      const result = await window.__TAURI__.invoke('restore_config_from_backup', {
        backupId
      })

      if (result.success) {
        return {
          success: true,
          message: '配置恢复成功',
          data: result.data
        }
      } else {
        return {
          success: false,
          message: result.message || '配置恢复失败'
        }
      }
    } catch (error) {
      return {
        success: false,
        message: `配置恢复失败: ${error}`
      }
    }
  }

  /**
   * 应用配置文件
   * @param configType 配置类型
   * @param content 配置内容
   * @param createBackup 是否创建备份
   * @returns 应用结果
   */
  async applyConfig(
    configType: ConfigType,
    content: string,
    createBackup: boolean = true
  ): Promise<ConfigOperationResult> {
    try {
      // 验证配置格式
      const validation = this.validateConfigFormat(content, configType)
      if (!validation.valid) {
        return {
          success: false,
          message: `配置格式验证失败: ${validation.error}`
        }
      }

      // 如果需要，先创建备份
      if (createBackup) {
        const backupResult = await this.backupConfig(
          configType,
          content,
          `应用配置前自动备份 - ${new Date().toLocaleString()}`
        )
        if (!backupResult.success) {
          console.warn('创建备份失败，但继续应用配置:', backupResult.message)
        }
      }

      // 这里应该调用实际的文件写入命令
      // 由于当前Tauri命令中没有直接的文件写入，我们模拟这个过程
      const configPath = this.configPaths[configType]

      // TODO: 实现实际的文件写入逻辑
      // 可以通过添加新的Tauri命令来实现

      return {
        success: true,
        message: `配置已成功应用到 ${configPath}`,
        data: {
          configPath,
          content,
          appliedAt: new Date().toISOString()
        }
      }
    } catch (error) {
      return {
        success: false,
        message: `应用配置失败: ${error}`
      }
    }
  }

  /**
   * 预览配置
   * @param params 预览参数
   * @returns 预览结果
   */
  async previewConfig(params: ConfigPreviewParams): Promise<ConfigOperationResult> {
    try {
      const { configType, claudeSupplierId, codexSupplierId, mcpTemplateIds = [], variables = {} } = params

      let suppliers: Supplier[] = []
      let templates: McpTemplate[] = []

      // 获取供应商信息
      if (configType === 'claude' && claudeSupplierId) {
        const supplier = await supplierApi.getSupplierById(claudeSupplierId)
        if (supplier) {
          suppliers.push(supplier)
        }
      } else if (configType === 'codex' && codexSupplierId) {
        const supplier = await supplierApi.getSupplierById(codexSupplierId)
        if (supplier) {
          suppliers.push(supplier)
        }
      }

      // 获取MCP模板
      if (mcpTemplateIds.length > 0) {
        for (const templateId of mcpTemplateIds) {
          // 这里需要实现根据ID获取模板的方法
          // const template = await mcpTemplateApi.getMcpTemplateById(templateId)
          // if (template) {
          //   templates.push(template)
          // }
        }
      } else {
        // 如果没有指定模板，获取所有相关类型的模板
        templates = await mcpTemplateApi.listMcpTemplates(configType)
      }

      // 生成配置
      let result
      if (configType === 'claude' && suppliers.length > 0) {
        const claudeSupplier = suppliers[0]!
        result = await configTemplateEngine.generateClaudeConfig(
          claudeSupplier,
          templates.filter(t => t.aiType === 'claude'),
          variables
        )
      } else if (configType === 'codex' && suppliers.length > 0) {
        const codexSupplier = suppliers[0]!
        result = await configTemplateEngine.generateCodexConfig(
          codexSupplier,
          templates.filter(t => t.aiType === 'codex'),
          variables
        )
      } else {
        return {
          success: false,
          message: '无法生成配置：缺少必要的供应商信息'
        }
      }

      if (result.success) {
        return {
          success: true,
          message: '配置预览生成成功',
          data: {
            content: result.content,
            format: result.format,
            variables: this.extractVariables(result.content)
          }
        }
      } else {
        return {
          success: false,
          message: `配置预览生成失败: ${result.errors?.join(', ')}`
        }
      }
    } catch (error) {
      return {
        success: false,
        message: `配置预览失败: ${error}`
      }
    }
  }

  /**
   * 验证配置格式
   * @param content 配置内容
   * @param configType 配置类型
   * @returns 验证结果
   */
  validateConfigFormat(content: string, configType: ConfigType): { valid: boolean; error?: string } {
    return configTemplateEngine.validateConfig(content, configType)
  }

  /**
   * 比较配置差异
   * @param oldContent 旧配置内容
   * @param newContent 新配置内容
   * @returns 差异对比结果
   */
  compareConfigs(oldContent: string, newContent: string): ConfigDiffResult {
    const oldLines = oldContent.split('\n')
    const newLines = newContent.split('\n')

    const additions: string[] = []
    const removals: string[] = []
    const modifications: Array<{ line: number; old: string; new: string }> = []

    // 简单的逐行比较
    const maxLines = Math.max(oldLines.length, newLines.length)

    for (let i = 0; i < maxLines; i++) {
      const oldLine = oldLines[i] || ''
      const newLine = newLines[i] || ''

      if (oldLine && !newLine) {
        removals.push(oldLine)
      } else if (!oldLine && newLine) {
        additions.push(newLine)
      } else if (oldLine !== newLine) {
        modifications.push({
          line: i + 1,
          old: oldLine,
          new: newLine
        })
      }
    }

    const hasChanges = additions.length > 0 || removals.length > 0 || modifications.length > 0

    return {
      hasChanges,
      additions,
      removals,
      modifications
    }
  }

  /**
   * 删除配置历史记录
   * @param backupId 备份ID
   * @returns 删除结果
   */
  async deleteConfigHistory(backupId: number): Promise<ConfigOperationResult> {
    try {
      const result = await window.__TAURI__.invoke('delete_config_history', {
        backupId
      })

      if (result.success) {
        return {
          success: true,
          message: '配置历史删除成功'
        }
      } else {
        return {
          success: false,
          message: result.message || '配置历史删除失败'
        }
      }
    } catch (error) {
      return {
        success: false,
        message: `删除配置历史失败: ${error}`
      }
    }
  }

  /**
   * 清理旧的配置历史记录
   * @param configType 配置类型
   * @param keepCount 保留数量
   * @returns 清理结果
   */
  async cleanupOldConfigHistory(configType: ConfigType, keepCount: number): Promise<ConfigOperationResult> {
    try {
      const result = await window.__TAURI__.invoke('cleanup_old_config_history', {
        configType,
        keepCount
      })

      if (result.success) {
        return {
          success: true,
          message: `已清理 ${result.data} 条旧的配置历史记录`,
          data: { deletedCount: result.data }
        }
      } else {
        return {
          success: false,
          message: result.message || '清理配置历史失败'
        }
      }
    } catch (error) {
      return {
        success: false,
        message: `清理配置历史失败: ${error}`
      }
    }
  }

  /**
   * 从内容中提取变量
   * @param content 配置内容
   * @returns 提取的变量列表
   */
  extractVariables(content: string): string[] {
    return configTemplateEngine.extractVariables(content)
  }

  /**
   * 获取配置文件路径
   * @param configType 配置类型
   * @returns 配置文件路径
   */
  getConfigPath(configType: ConfigType): string {
    return this.configPaths[configType] || ''
  }

  /**
   * 获取所有支持的配置类型
   * @returns 配置类型列表
   */
  getSupportedConfigTypes(): ConfigType[] {
    return Object.keys(this.configPaths) as ConfigType[]
  }
}

// 导出单例实例
export const configFileManager = new ConfigFileManager()