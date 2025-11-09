// Tauri环境检测和模拟数据工具
export class TauriHelper {
  static isTauriEnvironment(): boolean {
    return typeof window !== 'undefined' && !!window.__TAURI__
  }

  static async invokeTauri<T>(command: string, args?: any): Promise<T> {
    if (!this.isTauriEnvironment()) {
      // 开发模式下的模拟数据
      return this.getMockData<T>(command, args)
    }
    // 类型断言以绕过 TypeScript 检查
    return (window.__TAURI__.invoke as any)(command, args) as Promise<T>
  }

  // 开发模式数据持久化
  private static getStorageKey(command: string): string {
    return `dev_data_${command}`
  }

  private static saveToStorage<T>(key: string, data: T): void {
    try {
      localStorage.setItem(key, JSON.stringify(data))
    } catch (error) {
      console.warn('保存到本地存储失败:', error)
    }
  }

  private static getFromStorage<T>(key: string): T | null {
    try {
      const data = localStorage.getItem(key)
      return data ? JSON.parse(data) : null
    } catch (error) {
      console.warn('从本地存储读取失败:', error)
      return null
    }
  }

  private static async getMockData<T>(command: string, args?: any): Promise<T> {
    const storageKey = this.getStorageKey(command)

    // 对于创建操作，需要特殊处理以支持数据持久化
    if (command === 'create_supplier' || command === 'create_mcp_template') {
      const existingData = this.getFromStorage<T[]>(storageKey) || []
      const newItem = this.generateNewItem(command, args)
      const updatedData = [...existingData, newItem]
      this.saveToStorage(storageKey, updatedData)

      return {
        success: true,
        data: newItem
      } as T
    }

    // 对于列表操作，先从本地存储获取数据，如果没有则使用默认数据
    if (command === 'list_suppliers') {
      let suppliers = this.getFromStorage<T>(storageKey)
      if (!suppliers) {
        // 使用默认的模拟数据
        suppliers = [
          {
            id: 1,
            name: 'Claude API (开发模式)',
            type: 'claude',
            apiKey: 'sk-test-key',
            endpoint: 'https://api.anthropic.com',
            model: 'claude-3-sonnet-20241022',
            isActive: true,
            isHealthy: true,
            config: {},
            createdAt: new Date().toISOString(),
            updatedAt: new Date().toISOString()
          },
          {
            id: 2,
            name: 'OpenAI API (开发模式)',
            type: 'openai',
            apiKey: 'sk-test-key-openai',
            endpoint: 'https://api.openai.com',
            model: 'gpt-4',
            isActive: false,
            isHealthy: true,
            config: {},
            createdAt: new Date().toISOString(),
            updatedAt: new Date().toISOString()
          }
        ] as T
        this.saveToStorage(storageKey, suppliers)
      }

      return {
        success: true,
        data: suppliers
      } as T
    }

    if (command === 'list_mcp_templates') {
      let templates = this.getFromStorage<T>(storageKey)
      if (!templates) {
        // 使用默认的模拟数据
        templates = [
          {
            id: 1,
            name: 'Claude Code Assistant (开发模式)',
            description: '基于Claude的代码助手模板',
            aiType: 'claude',
            platformType: 'vscode',
            category: 'development',
            template: {
              command: 'code-review',
              parameters: {
                file_path: '{{file_path}}',
                focus_areas: ['security', 'performance', 'best-practices']
              }
            },
            tags: ['code', 'review', 'claude'],
            isActive: true,
            usageCount: 0,
            rating: 5,
            createdAt: new Date().toISOString(),
            updatedAt: new Date().toISOString()
          },
          {
            id: 2,
            name: 'ChatGPT Assistant (开发模式)',
            description: '基于GPT的通用助手模板',
            aiType: 'openai',
            platformType: 'web',
            category: 'general',
            template: {
              command: 'chat-assistant',
              parameters: {
                model: 'gpt-4',
                temperature: 0.7
              }
            },
            tags: ['chat', 'assistant', 'gpt'],
            isActive: false,
            usageCount: 0,
            rating: 4,
            createdAt: new Date().toISOString(),
            updatedAt: new Date().toISOString()
          }
        ] as T
        this.saveToStorage(storageKey, templates)
      }

      return {
        success: true,
        data: templates
      } as T
    }

    // 其他命令的默认响应
    const mockResponses: Record<string, any> = {
      'get_mcp_template_stats': {
        total: 2,
        active: 1,
        claude: 1,
        openai: 1,
        vscode: 1,
        web: 1
      },
      'get_supplier_stats': {
        total: 2,
        active: 1,
        claude: 1,
        openai: 1
      },
      'delete_supplier': {
        success: true,
        data: true
      },
      'delete_mcp_template': {
        success: true,
        data: true
      },
      'update_supplier': {
        success: true,
        data: args
      },
      'update_mcp_template': {
        success: true,
        data: args
      }
    }

    return mockResponses[command] || { success: false, message: `开发模式下不支持命令: ${command}` }
  }

  private static generateNewItem(command: string, args?: any): any {
    if (command === 'create_supplier') {
      return {
        id: Date.now(),
        name: args?.name || '新供应商 (开发模式)',
        type: args?.type || 'claude',
        apiKey: args?.apiKey || 'sk-new-key',
        endpoint: args?.endpoint || 'https://api.anthropic.com',
        model: args?.model || 'claude-3-sonnet-20241022',
        isActive: args?.isActive || false,
        isHealthy: true,
        config: args?.config || {},
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString()
      }
    }

    if (command === 'create_mcp_template') {
      return {
        id: Date.now(),
        name: args?.name || '新模板 (开发模式)',
        description: args?.description || '开发模式创建的模板',
        aiType: args?.aiType || 'claude',
        platformType: args?.platformType || 'vscode',
        category: args?.category || 'general',
        template: args?.template || {},
        tags: args?.tags || [],
        isActive: args?.isActive || false,
        usageCount: 0,
        rating: 0,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString()
      }
    }

    return {}
  }
}

