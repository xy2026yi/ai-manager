import type {
  Supplier,
  McpTemplate,
  ClaudeConfig,
  CodexConfig,
  ClaudeMcpServer,
  CodexMcpServer,
  CodexModelProvider,
  ConfigType
} from '@/types'
import { supplierApi } from './supplierApi'
import { mcpTemplateApi } from './mcpTemplateApi'

/**
 * 配置模板变量
 */
export interface ConfigVariable {
  name: string
  value: string
  description?: string
}

/**
 * 模板生成结果
 */
export interface TemplateGenerationResult {
  success: boolean
  content: string
  format: 'json' | 'toml'
  errors?: string[]
}

/**
 * 配置模板引擎
 * 负责基于供应商和MCP模板生成配置文件
 */
export class ConfigTemplateEngine {
  /**
   * 生成Claude配置
   * @param claudeSupplier Claude供应商
   * @param templates MCP模板列表
   * @param variables 额外的配置变量
   * @returns Claude配置内容
   */
  async generateClaudeConfig(
    claudeSupplier: Supplier,
    templates: McpTemplate[],
    variables: Record<string, string> = {}
  ): Promise<TemplateGenerationResult> {
    try {
      // 构建基础配置结构
      const config: Partial<ClaudeConfig> = {
        numStartups: 1,
        installMethod: 'global',
        autoUpdates: true,
        tipsHistory: {
          'new-user-warmup': 1
        },
        cachedStatsigGates: {
          'tengu_migrate_ignore_patterns': false,
          'tengu_disable_bypass_permissions_mode': false,
          'tengu_tool_pear': false,
          'tengu_halloween': true
        },
        cachedDynamicConfigs: {
          'tengu-top-of-feed-tip': {
            tip: '',
            color: ''
          }
        },
        firstStartTime: new Date().toISOString(),
        userID: this.generateUserID(),
        sonnet45MigrationComplete: true,
        changelogLastFetched: Date.now(),
        iterm2SetupInProgress: false,
        shiftEnterKeyBindingInstalled: true,
        hasCompletedOnboarding: true,
        lastOnboardingVersion: '2.0.30',
        hasOpusPlanDefault: false,
        lastReleaseNotesSeen: '2.0.30',
        projects: {},
        mcpServers: {},
        env: this.buildClaudeEnvironment(claudeSupplier, variables)
      }

      // 添加MCP服务器配置
      for (const template of templates) {
        if (template.aiType === 'claude') {
          try {
            const mcpConfig = this.parseClaudeMcpConfig(template.configContent)
            config.mcpServers![template.name] = mcpConfig
          } catch (error) {
            console.warn(`解析MCP模板 ${template.name} 失败:`, error)
          }
        }
      }

      // 替换配置中的变量
      const configJson = JSON.stringify(config, null, 2)
      const finalContent = this.replaceVariables(configJson, {
        ...this.buildDefaultVariables(claudeSupplier, templates),
        ...variables
      })

      return {
        success: true,
        content: finalContent,
        format: 'json'
      }
    } catch (error) {
      return {
        success: false,
        content: '',
        format: 'json',
        errors: [`生成Claude配置失败: ${error}`]
      }
    }
  }

  /**
   * 生成Codex配置
   * @param codexSupplier Codex供应商
   * @param templates MCP模板列表
   * @param variables 额外的配置变量
   * @returns Codex配置内容
   */
  async generateCodexConfig(
    codexSupplier: Supplier,
    templates: McpTemplate[],
    variables: Record<string, string> = {}
  ): Promise<TemplateGenerationResult> {
    try {
      // 构建基础配置结构
      const config: Partial<CodexConfig> = {
        model: 'gpt-5-codex',
        modelReasoningEffort: 'high',
        disableResponseStorage: true,
        preferredAuthMethod: 'apikey',
        windowsWslSetupAcknowledged: true,
        modelProvider: codexSupplier.name,
        modelProviders: {},
        mcpServers: {}
      }

      // 添加模型供应商配置
      const modelProvider: CodexModelProvider = {
        name: codexSupplier.name,
        baseUrl: codexSupplier.baseUrl,
        wireApi: 'responses',
        requiresOpenaiAuth: true
      }
      config.modelProviders![codexSupplier.name] = modelProvider

      // 添加MCP服务器配置
      for (const template of templates) {
        if (template.aiType === 'codex') {
          try {
            const mcpConfig = this.parseCodexMcpConfig(template.configContent)
            config.mcpServers![template.name] = mcpConfig
          } catch (error) {
            console.warn(`解析MCP模板 ${template.name} 失败:`, error)
          }
        }
      }

      // 生成TOML格式配置
      let tomlContent = this.convertCodexConfigToToml(config)

      // 替换配置中的变量
      tomlContent = this.replaceVariables(tomlContent, {
        ...this.buildDefaultVariables(codexSupplier, templates),
        ...variables
      })

      return {
        success: true,
        content: tomlContent,
        format: 'toml'
      }
    } catch (error) {
      return {
        success: false,
        content: '',
        format: 'toml',
        errors: [`生成Codex配置失败: ${error}`]
      }
    }
  }

  /**
   * 替换模板变量
   * @param content 原始内容
   * @param variables 变量映射
   * @returns 替换后的内容
   */
  replaceVariables(content: string, variables: Record<string, string>): string {
    let result = content

    for (const [key, value] of Object.entries(variables)) {
      // 支持 {{variable}} 和 ${variable} 两种格式
      const patterns = [
        `{{\\s*${key}\\s*}}`,
        `\\$\\{${key}\\}`
      ]

      for (const pattern of patterns) {
        const regex = new RegExp(pattern, 'g')
        result = result.replace(regex, value)
      }
    }

    return result
  }

  /**
   * 从模板中提取变量
   * @param content 模板内容
   * @returns 提取的变量列表
   */
  extractVariables(content: string): string[] {
    const patterns = [
      /{{\s*([^}]+)\s*}}/g,
      /\${([^}]+)}/g
    ]

    const variables = new Set<string>()

    for (const pattern of patterns) {
      let match
      while ((match = pattern.exec(content)) !== null) {
        if (match[1]) {
          variables.add(match[1].trim())
        }
      }
    }

    return Array.from(variables)
  }

  /**
   * 验证配置格式
   * @param content 配置内容
   * @param configType 配置类型
   * @returns 验证结果
   */
  validateConfig(content: string, configType: ConfigType): { valid: boolean; error?: string } {
    try {
      if (configType === 'claude') {
        JSON.parse(content)
      } else if (configType === 'codex') {
        // 简单的TOML格式验证
        this.validateTomlFormat(content)
      }
      return { valid: true }
    } catch (error) {
      return { valid: false, error: `配置格式错误: ${error}` }
    }
  }

  /**
   * 构建Claude环境变量
   */
  private buildClaudeEnvironment(supplier: Supplier, variables: Record<string, string>): Record<string, string> {
    const env: Record<string, string> = {
      ANTHROPIC_BASE_URL: supplier.baseUrl,
      ANTHROPIC_AUTH_TOKEN: supplier.authToken,
      ...variables
    }

    // 添加可选的环境变量
    if (variables.API_TIMEOUT_MS) {
      env.API_TIMEOUT_MS = variables.API_TIMEOUT_MS
    }
    if (variables.CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC) {
      env.CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = variables.CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC
    }

    return env
  }

  /**
   * 解析Claude MCP配置
   */
  private parseClaudeMcpConfig(configContent: string): ClaudeMcpServer {
    try {
      return JSON.parse(configContent) as ClaudeMcpServer
    } catch (error) {
      // 如果解析失败，返回默认配置
      return {
        type: 'stdio',
        command: 'npx',
        args: ['-y', 'mcp-server'],
        env: {}
      }
    }
  }

  /**
   * 解析Codex MCP配置
   */
  private parseCodexMcpConfig(configContent: string): CodexMcpServer {
    try {
      // 简单的TOML解析（实际项目中建议使用专门的TOML解析库）
      const config: Partial<CodexMcpServer> = {
        type: 'stdio',
        command: 'npx',
        args: ['-y', 'mcp-server'],
        env: {}
      }

      // 从配置内容中提取关键信息
      const lines = configContent.split('\n')
      for (const line of lines) {
        const trimmed = line.trim()
        if (trimmed.startsWith('type = ')) {
          const typeValue = trimmed.split('=')[1]?.trim().replace(/['"]/g, '')
          if (typeValue) {
            config.type = typeValue as 'stdio' | 'websocket'
          }
        } else if (trimmed.startsWith('command = ')) {
          const commandValue = trimmed.split('=')[1]?.trim().replace(/['"]/g, '')
          if (commandValue) {
            config.command = commandValue
          }
        } else if (trimmed.startsWith('args = ')) {
          // 简化处理，实际需要更复杂的解析
          config.args = ['-y', 'mcp-server']
        }
      }

      return config as CodexMcpServer
    } catch (error) {
      return {
        type: 'stdio',
        command: 'npx',
        args: ['-y', 'mcp-server'],
        env: {}
      }
    }
  }

  /**
   * 将Codex配置转换为TOML格式
   */
  private convertCodexConfigToToml(config: Partial<CodexConfig>): string {
    const lines: string[] = []

    // 基本配置
    if (config.model) {
      lines.push(`model = "${config.model}"`)
    }
    if (config.modelReasoningEffort) {
      lines.push(`model_reasoning_effort = "${config.modelReasoningEffort}"`)
    }
    if (config.disableResponseStorage !== undefined) {
      lines.push(`disable_response_storage = ${config.disableResponseStorage}`)
    }
    if (config.preferredAuthMethod) {
      lines.push(`preferred_auth_method = "${config.preferredAuthMethod}"`)
    }
    lines.push('windows_wsl_setup_acknowledged = true')
    if (config.modelProvider) {
      lines.push(`model_provider = "${config.modelProvider}"`)
    }

    lines.push('')

    // 模型供应商配置
    if (config.modelProviders) {
      lines.push('[model_providers]')
      for (const [name, provider] of Object.entries(config.modelProviders)) {
        lines.push(`[model_providers.${name}]`)
        lines.push(`name = "${provider.name}"`)
        lines.push(`base_url = "${provider.baseUrl}"`)
        if (provider.wireApi) {
          lines.push(`wire_api = "${provider.wireApi}"`)
        }
        if (provider.requiresOpenaiAuth !== undefined) {
          lines.push(`requires_openai_auth = ${provider.requiresOpenaiAuth}`)
        }
        lines.push('')
      }
    }

    // MCP服务器配置
    if (config.mcpServers) {
      lines.push('[mcp_servers]')
      for (const [name, server] of Object.entries(config.mcpServers)) {
        lines.push(`[mcp_servers.${name}]`)
        lines.push(`type = "${server.type}"`)
        if (server.command) {
          lines.push(`command = "${server.command}"`)
        }
        if (server.args && server.args.length > 0) {
          lines.push(`args = [${server.args.map(arg => `"${arg}"`).join(', ')}]`)
        }
        if (server.startupTimeoutMs) {
          lines.push(`startup_timeout_ms = ${server.startupTimeoutMs}`)
        }
        if (server.env && Object.keys(server.env).length > 0) {
          lines.push('env = {}')
          for (const [key, value] of Object.entries(server.env)) {
            lines.push(`env.${key} = "${value}"`)
          }
        }
        lines.push('')
      }
    }

    return lines.join('\n')
  }

  /**
   * 构建默认变量
   */
  private buildDefaultVariables(supplier: Supplier, templates: McpTemplate[]): Record<string, string> {
    return {
      'SUPPLIER_NAME': supplier.name,
      'SUPPLIER_BASE_URL': supplier.baseUrl,
      'SUPPLIER_TYPE': supplier.type,
      'TEMPLATE_COUNT': templates.length.toString(),
      'GENERATION_TIME': new Date().toISOString(),
      'PLATFORM': this.detectPlatform()
    }
  }

  /**
   * 生成用户ID
   */
  private generateUserID(): string {
    return Array.from({ length: 32 }, () =>
      Math.floor(Math.random() * 16).toString(16)
    ).join('')
  }

  /**
   * 检测平台类型
   */
  private detectPlatform(): string {
    // 简化实现，实际项目中可以通过Tauri API获取
    return navigator.platform.toLowerCase().includes('win') ? 'windows' : 'unix'
  }

  /**
   * 简单的TOML格式验证
   */
  private validateTomlFormat(content: string): void {
    const lines = content.split('\n')
    let inSection = false

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i]
      if (!line) continue

      const trimmed = line.trim()

      // 跳过空行和注释
      if (!trimmed || trimmed.startsWith('#')) {
        continue
      }

      // 检查节标题
      if (line.startsWith('[') && line.endsWith(']')) {
        inSection = true
        continue
      }

      // 检查键值对
      if (line.includes('=')) {
        const parts = line.split('=').map(s => s.trim())
        const key = parts[0]
        const value = parts[1]

        if (!key) {
          throw new Error(`第 ${i + 1} 行: 键不能为空`)
        }

        if (!value) {
          throw new Error(`第 ${i + 1} 行: 值不能为空`)
        }

        // 简单的值格式检查
        if (!(value.startsWith('"') && value.endsWith('"')) &&
            !(value.startsWith("'") && value.endsWith("'")) &&
            !['true', 'false'].includes(value.toLowerCase()) &&
            !/^\d+$/.test(value) &&
            !value.includes('[')) {
          throw new Error(`第 ${i + 1} 行: 值格式不正确`)
        }
      }
    }
  }
}

// 导出单例实例
export const configTemplateEngine = new ConfigTemplateEngine()