import type {
  Supplier,
  CreateSupplierRequest,
  UpdateSupplierRequest,
  ConnectionTestResult,
  SupplierHealth,
  SupplierSwitchRequest,
  SupplierSwitchResult,
  SupplierSwitchProgress,
  FailoverConfig,
  SupplierPerformanceMetrics,
  SupplierSwitchHistory,
  ApiResponse
} from '@/types'
import '@/types/tauri'
import { TauriHelper } from '@/utils/tauriHelper'

class SupplierApiService {

  // 获取供应商列表
  async listSuppliers(type?: string): Promise<Supplier[]> {
    try {
      const result = await TauriHelper.invokeTauri<ApiResponse<Supplier[]>>('list_suppliers', {
        supplierType: type
      })
      return result.data || []
    } catch (error) {
      console.error('获取供应商列表失败:', error)
      throw error
    }
  }

  // 创建供应商
  async createSupplier(request: CreateSupplierRequest): Promise<Supplier> {
    try {
      const result = await TauriHelper.invokeTauri<ApiResponse<Supplier>>('create_supplier', request)
      if (result.success) {
        return result.data!
      } else {
        throw new Error(result.message || '创建供应商失败')
      }
    } catch (error) {
      console.error('创建供应商失败:', error)
      throw error
    }
  }

  // 更新供应商
  async updateSupplier(request: UpdateSupplierRequest): Promise<Supplier> {
    try {
      const result = await TauriHelper.invokeTauri<ApiResponse<Supplier>>('update_supplier', request)
      if (result.success) {
        return result.data!
      } else {
        throw new Error(result.message || '更新供应商失败')
      }
    } catch (error) {
      console.error('更新供应商失败:', error)
      throw error
    }
  }

  // 删除供应商
  async deleteSupplier(id: number): Promise<boolean> {
    try {
      const result = await TauriHelper.invokeTauri<ApiResponse<boolean>>('delete_supplier', { id })
      return result.success && !!result.data
    } catch (error) {
      console.error('删除供应商失败:', error)
      throw error
    }
  }

  // 根据ID获取供应商
  async getSupplierById(id: number): Promise<Supplier | null> {
    try {
      const result = await TauriHelper.invokeTauri<ApiResponse<Supplier>>('get_supplier_by_id', { id })
      return result.data || null
    } catch (error) {
      console.error('获取供应商失败:', error)
      throw error
    }
  }

  // 设置活跃供应商
  async setActiveSupplier(id: number, type: string): Promise<boolean> {
    try {
      const result = await TauriHelper.invokeTauri<ApiResponse<boolean>>('set_active_supplier', {
        id,
        type
      })
      return result.success && !!result.data
    } catch (error) {
      console.error('设置活跃供应商失败:', error)
      throw error
    }
  }

  // 测试供应商连接
  async testConnection(supplier: Supplier): Promise<ConnectionTestResult> {
    try {
      const result = await TauriHelper.invokeTauri<ApiResponse<ConnectionTestResult>>('test_supplier_connection', {
        supplier
      })
      if (result.success) {
        return result.data!
      } else {
        throw new Error(result.message || '连接测试失败')
      }
    } catch (error) {
      console.error('连接测试失败:', error)
      throw error
    }
  }

  // 验证供应商配置
  async validateConfig(supplier: Supplier): Promise<boolean> {
    try {
      const result = await TauriHelper.invokeTauri<ApiResponse<boolean>>('validate_supplier_config', {
        supplier
      })
      return result.success && !!result.data
    } catch (error) {
      console.error('配置验证失败:', error)
      throw error
    }
  }

  // 获取供应商统计
  async getSupplierStats(): Promise<any> {
    try {
      const result = await TauriHelper.invokeTauri<ApiResponse<any>>('get_supplier_stats')
      return result.data
    } catch (error) {
      console.error('获取供应商统计失败:', error)
      throw error
    }
  }

  // 导入供应商
  async importSuppliers(suppliers: CreateSupplierRequest[]): Promise<Supplier[]> {
    try {
      const result = await TauriHelper.invokeTauri<ApiResponse<Supplier[]>>('import_suppliers', {
        suppliers
      })
      if (result.success) {
        return result.data!
      } else {
        throw new Error(result.message || '导入供应商失败')
      }
    } catch (error) {
      console.error('导入供应商失败:', error)
      throw error
    }
  }

  // 导出供应商
  async exportSuppliers(): Promise<Supplier[]> {
    try {
      const result = await TauriHelper.invokeTauri<ApiResponse<Supplier[]>>('export_suppliers')
      return result.data || []
    } catch (error) {
      console.error('导出供应商失败:', error)
      throw error
    }
  }

  // ========== 健康检查相关方法 ==========

  // 检查单个供应商健康状态
  async checkSupplierHealth(supplierId: number): Promise<SupplierHealth> {
    try {
      const result = await TauriHelper.invokeTauri<ApiResponse<SupplierHealth>>('check_supplier_health', {
        supplierId
      })
      return result.data!
    } catch (error) {
      console.error('健康检查失败:', error)
      throw error
    }
  }

  // 检查所有供应商健康状态
  async checkAllSuppliersHealth(): Promise<SupplierHealth[]> {
    try {
      const result = await TauriHelper.invokeTauri<ApiResponse<SupplierHealth[]>>('check_all_suppliers_health')
      return result.data || []
    } catch (error) {
      console.error('批量健康检查失败:', error)
      throw error
    }
  }

  // 执行供应商切换
  async switchSupplier(request: SupplierSwitchRequest): Promise<SupplierSwitchResult> {
    try {
      const result = await TauriHelper.invokeTauri<ApiResponse<SupplierSwitchResult>>('switch_supplier', request)
      return result.data!
    } catch (error) {
      console.error('供应商切换失败:', error)
      throw error
    }
  }

  // 执行自动故障转移
  async autoFailover(supplierType: string): Promise<SupplierSwitchResult> {
    try {
      const result = await TauriHelper.invokeTauri<ApiResponse<SupplierSwitchResult>>('auto_failover', {
        supplierType
      })
      return result.data!
    } catch (error) {
      console.error('自动故障转移失败:', error)
      throw error
    }
  }

  // 获取故障转移配置
  async getFailoverConfig(supplierType: string): Promise<FailoverConfig> {
    try {
      const result = await TauriHelper.invokeTauri<ApiResponse<FailoverConfig>>('get_failover_config', {
        supplierType
      })
      return result.data!
    } catch (error) {
      console.error('获取故障转移配置失败:', error)
      throw error
    }
  }

  // 更新故障转移配置
  async updateFailoverConfig(supplierType: string, config: FailoverConfig): Promise<boolean> {
    try {
      const result = await TauriHelper.invokeTauri<ApiResponse<boolean>>('update_failover_config', {
        supplierType,
        config
      })
      return result.success && !!result.data
    } catch (error) {
      console.error('更新故障转移配置失败:', error)
      throw error
    }
  }

  // 获取供应商切换进度
  async getSupplierSwitchProgress(switchId: string): Promise<SupplierSwitchProgress | null> {
    try {
      const result = await TauriHelper.invokeTauri<ApiResponse<SupplierSwitchProgress>>('get_supplier_switch_progress', {
        switchId
      })
      return result.data || null
    } catch (error) {
      console.error('获取供应商切换进度失败:', error)
      throw error
    }
  }

  // 获取供应商性能指标
  async getSupplierPerformanceMetrics(supplierId: number): Promise<SupplierPerformanceMetrics> {
    try {
      // 基于健康检查数据计算性能指标
      const health = await this.checkSupplierHealth(supplierId)

      // 简化的性能指标计算
      const metrics: SupplierPerformanceMetrics = {
        supplierId,
        avgResponseTime: health.responseTime,
        successRate: health.totalRequests > 0 ? ((health.totalRequests - health.failedRequests) / health.totalRequests) * 100 : 0,
        errorCount: health.failedRequests,
        uptime: health.uptimePercentage,
        totalRequests: health.totalRequests,
        lastHour: {
          requests: 1, // 简化实现
          errors: health.failedRequests,
          avgResponseTime: health.responseTime
        }
      }

      return metrics
    } catch (error) {
      console.error('获取供应商性能指标失败:', error)
      throw error
    }
  }

  // 获取供应商切换历史记录
  async getSupplierSwitchHistory(limit?: number): Promise<SupplierSwitchHistory[]> {
    try {
      // TODO: 从后端获取切换历史记录
      // 这里返回模拟数据
      const history: SupplierSwitchHistory[] = [
        {
          id: '1',
          operation: 'switch',
          fromSupplierId: 1,
          toSupplierId: 2,
          reason: '手动切换',
          timestamp: new Date().toISOString(),
          success: true,
          duration: 1500,
          backupId: 1
        }
      ]

      return limit ? history.slice(0, limit) : history
    } catch (error) {
      console.error('获取供应商切换历史失败:', error)
      throw error
    }
  }

  // 验证供应商切换请求
  validateSwitchRequest(request: SupplierSwitchRequest): { valid: boolean; errors: string[] } {
    const errors: string[] = []

    // 基本验证
    if (!request.fromSupplierId) {
      errors.push('源供应商ID不能为空')
    }

    if (!request.toSupplierId) {
      errors.push('目标供应商ID不能为空')
    }

    if (request.fromSupplierId === request.toSupplierId) {
      errors.push('源供应商和目标供应商不能相同')
    }

    if (!request.switchReason) {
      errors.push('切换原因不能为空')
    }

    return {
      valid: errors.length === 0,
      errors
    }
  }

  // 检查供应商是否可以进行切换
  async canSwitchSupplier(fromSupplierId: number, toSupplierId: number): Promise<{
    canSwitch: boolean
    reasons: string[]
    suggestions: string[]
  }> {
    try {
      const errors: string[] = []
      const suggestions: string[] = []

      // 检查目标供应商健康状态
      const toHealth = await this.checkSupplierHealth(toSupplierId)
      if (!toHealth.isHealthy) {
        errors.push('目标供应商不健康')
        suggestions.push('请选择健康的备用供应商')
      }

      // 检查源供应商状态
      const fromHealth = await this.checkSupplierHealth(fromSupplierId)
      if (fromHealth.isHealthy && fromHealth.consecutiveFailures < 3) {
        errors.push('源供应商状态良好，无需切换')
        suggestions.push('仅在必要时进行供应商切换')
      }

      return {
        canSwitch: errors.length === 0,
        reasons: errors,
        suggestions
      }
    } catch (error) {
      console.error('检查供应商切换条件失败:', error)
      return {
        canSwitch: false,
        reasons: ['无法检查供应商状态'],
        suggestions: ['请检查网络连接或稍后重试']
      }
    }
  }

  // 获取推荐的故障转移配置
  async getRecommendedFailoverConfig(supplierType: string): Promise<FailoverConfig> {
    try {
      const currentConfig = await this.getFailoverConfig(supplierType)

      // 基于供应商类型提供推荐配置
      const recommendedConfig: FailoverConfig = {
        ...currentConfig,
        maxConsecutiveFailures: supplierType === 'claude' ? 3 : 5,
        maxResponseTimeMs: supplierType === 'claude' ? 5000 : 10000,
        minSuccessRate: supplierType === 'claude' ? 95.0 : 90.0,
        rollbackDelaySeconds: 300
      }

      return recommendedConfig
    } catch (error) {
      console.error('获取推荐故障转移配置失败:', error)
      throw error
    }
  }

  // 模拟供应商切换（用于测试）
  async simulateSupplierSwitch(request: SupplierSwitchRequest): Promise<SupplierSwitchResult> {
    try {
      const validationResult = this.validateSwitchRequest(request)
      if (!validationResult.valid) {
        throw new Error(`切换请求验证失败: ${validationResult.errors.join(', ')}`)
      }

      // 模拟切换过程
      const canSwitch = await this.canSwitchSupplier(request.fromSupplierId, request.toSupplierId)
      if (!canSwitch.canSwitch) {
        throw new Error(`切换条件不满足: ${canSwitch.reasons.join(', ')}`)
      }

      // 模拟切换时间
      await new Promise(resolve => setTimeout(resolve, 1000))

      const result: SupplierSwitchResult = {
        success: true,
        message: `模拟切换成功：从供应商 ${request.fromSupplierId} 切换到供应商 ${request.toSupplierId}`,
        fromSupplierId: request.fromSupplierId,
        toSupplierId: request.toSupplierId,
        switchTime: new Date().toISOString(),
        rollbackAvailable: true,
        backupId: Math.floor(Math.random() * 1000),
        error: undefined
      }

      return result
    } catch (error) {
      console.error('模拟供应商切换失败:', error)
      throw error
    }
  }
}

export const supplierApi = new SupplierApiService()