import type {
  McpTemplate,
  CreateMcpTemplateRequest,
  UpdateMcpTemplateRequest,
  McpTemplateValidationResult,
  McpTemplateCategory,
  ApiResponse
} from '@/types'
import '@/types/tauri'
import { TauriHelper } from '@/utils/tauriHelper'

class McpTemplateApiService {
  // 获取MCP模板列表
  async listMcpTemplates(aiType?: string, platformType?: string): Promise<McpTemplate[]> {
    try {
      const result = await TauriHelper.invokeTauri<ApiResponse<McpTemplate[]>>('list_mcp_templates', {
        aiType,
        platformType
      })
      return result.data || []
    } catch (error) {
      console.error('获取MCP模板列表失败:', error)
      throw error
    }
  }

  // 创建MCP模板
  async createMcpTemplate(request: CreateMcpTemplateRequest): Promise<McpTemplate> {
    try {
      const result = await TauriHelper.invokeTauri<ApiResponse<McpTemplate>>('create_mcp_template', request)
      if (result.success) {
        return result.data!
      } else {
        throw new Error(result.message || '创建MCP模板失败')
      }
    } catch (error) {
      console.error('创建MCP模板失败:', error)
      throw error
    }
  }

  // 更新MCP模板
  async updateMcpTemplate(request: UpdateMcpTemplateRequest): Promise<McpTemplate | null> {
    try {
      const result = await TauriHelper.invokeTauri<ApiResponse<McpTemplate>>('update_mcp_template', request)
      if (result.success) {
        return result.data!
      } else {
        throw new Error(result.message || '更新MCP模板失败')
      }
    } catch (error) {
      console.error('更新MCP模板失败:', error)
      throw error
    }
  }

  // 删除MCP模板
  async deleteMcpTemplate(id: number): Promise<boolean> {
    try {
      const result = await TauriHelper.invokeTauri<ApiResponse<boolean>>('delete_mcp_template', { id })
      return result.success && !!result.data
    } catch (error) {
      console.error('删除MCP模板失败:', error)
      throw error
    }
  }

  // 根据ID获取MCP模板
  async getMcpTemplateById(id: number): Promise<McpTemplate | null> {
    try {
      const result = await TauriHelper.invokeTauri<ApiResponse<McpTemplate>>('get_mcp_template_by_id', { id })
      return result.data || null
    } catch (error) {
      console.error('获取MCP模板失败:', error)
      throw error
    }
  }

  // 验证MCP模板
  async validateMcpTemplate(request: CreateMcpTemplateRequest): Promise<McpTemplateValidationResult> {
    try {
      const result = await TauriHelper.invokeTauri<ApiResponse<McpTemplateValidationResult>>('validate_mcp_template', request)
      return result.data!
    } catch (error) {
      console.error('验证MCP模板失败:', error)
      throw error
    }
  }

  // 获取MCP模板分类
  async getMcpTemplateCategories(): Promise<McpTemplateCategory[]> {
    try {
      const result = await TauriHelper.invokeTauri<ApiResponse<McpTemplateCategory[]>>('get_mcp_template_categories')
      return result.data || []
    } catch (error) {
      console.error('获取MCP模板分类失败:', error)
      throw error
    }
  }

  // 增加模板使用次数
  async incrementTemplateUsage(id: number): Promise<boolean> {
    try {
      const result = await TauriHelper.invokeTauri<ApiResponse<boolean>>('increment_template_usage', { id })
      return result.success && !!result.data
    } catch (error) {
      console.error('增加模板使用次数失败:', error)
      throw error
    }
  }

  // 克隆MCP模板
  async cloneMcpTemplate(id: number, newName: string): Promise<McpTemplate | null> {
    try {
      const result = await TauriHelper.invokeTauri<ApiResponse<McpTemplate>>('clone_mcp_template', {
        id,
        newName
      })
      if (result.success) {
        return result.data!
      } else {
        throw new Error(result.message || '克隆MCP模板失败')
      }
    } catch (error) {
      console.error('克隆MCP模板失败:', error)
      throw error
    }
  }

  // 获取MCP模板统计
  async getMcpTemplateStats(): Promise<any> {
    try {
      const result = await TauriHelper.invokeTauri<ApiResponse<any>>('get_mcp_template_stats')
      return result.data
    } catch (error) {
      console.error('获取MCP模板统计失败:', error)
      throw error
    }
  }

  // 导入MCP模板
  async importMcpTemplates(templates: CreateMcpTemplateRequest[]): Promise<McpTemplate[]> {
    try {
      const result = await TauriHelper.invokeTauri<ApiResponse<McpTemplate[]>>('import_mcp_templates', {
        templates
      })
      if (result.success) {
        return result.data!
      } else {
        throw new Error(result.message || '导入MCP模板失败')
      }
    } catch (error) {
      console.error('导入MCP模板失败:', error)
      throw error
    }
  }

  // 导出MCP模板
  async exportMcpTemplates(): Promise<McpTemplate[]> {
    try {
      const result = await TauriHelper.invokeTauri<ApiResponse<McpTemplate[]>>('export_mcp_templates')
      return result.data || []
    } catch (error) {
      console.error('导出MCP模板失败:', error)
      throw error
    }
  }

  // 格式化配置内容（JSON/TOML）
  formatConfigContent(configContent: string, aiType: string): { content: string; language: string } {
    try {
      if (aiType === 'claude') {
        // JSON格式化
        const parsed = JSON.parse(configContent)
        return {
          content: JSON.stringify(parsed, null, 2),
          language: 'json'
        }
      } else {
        // TOML格式化（简单实现）
        return {
          content: configContent,
          language: 'toml'
        }
      }
    } catch (error) {
      // 格式化失败，返回原内容
      return {
        content: configContent,
        language: aiType === 'claude' ? 'json' : 'toml'
      }
    }
  }

  // 验证配置内容格式
  validateConfigFormat(configContent: string, aiType: string): { valid: boolean; error?: string } {
    try {
      if (aiType === 'claude') {
        JSON.parse(configContent)
      } else {
        // TOML格式验证（简单检查）
        if (!configContent.trim()) {
          return { valid: false, error: '配置内容不能为空' }
        }
      }
      return { valid: true }
    } catch (error) {
      return { valid: false, error: `配置格式错误: ${error}` }
    }
  }
}

export const mcpTemplateApi = new McpTemplateApiService()