<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type {
  Supplier,
  McpTemplate,
  ConfigType,
  ConfigBackup,
  ConfigOperationResult,
  ConfigPreviewParams
} from '@/types'
import { supplierApi } from '@/services/supplierApi'
import { mcpTemplateApi } from '@/services/mcpTemplateApi'
import { configFileManager } from '@/services/configFileManager'

// 响应式数据
const activeTab = ref('template')
const loading = ref(false)
const suppliers = ref<Supplier[]>([])
const templates = ref<McpTemplate[]>([])
const configHistory = ref<ConfigBackup[]>([])
const previewContent = ref('')
const previewFormat = ref<'json' | 'toml'>('json')

// 表单数据
const templateForm = ref({
  configType: 'claude' as ConfigType,
  claudeSupplierId: null as number | null,
  codexSupplierId: null as number | null,
  mcpTemplateIds: [] as number[],
  variables: {} as Record<string, string>
})

// 应用配置表单
const applyForm = ref({
  configType: 'claude' as ConfigType,
  content: '',
  createBackup: true,
  description: ''
})

// 配置类型选项
const configTypeOptions = [
  { label: 'Claude配置', value: 'claude' },
  { label: 'Codex配置', value: 'codex' }
]

// 计算属性
const claudeSuppliers = computed(() =>
  suppliers.value.filter(s => s.type === 'claude')
)

const codexSuppliers = computed(() =>
  suppliers.value.filter(s => s.type === 'codex')
)

const availableTemplates = computed(() => {
  const configType = templateForm.value.configType
  return templates.value.filter(t => t.aiType === configType)
})

const selectedTemplates = computed(() => {
  return templates.value.filter(t =>
    templateForm.value.mcpTemplateIds.includes(t.id!)
  )
})

// 加载供应商列表
const loadSuppliers = async () => {
  try {
    suppliers.value = await supplierApi.listSuppliers()
  } catch (error) {
    ElMessage.error('加载供应商列表失败')
    console.error(error)
  }
}

// 加载模板列表
const loadTemplates = async () => {
  try {
    templates.value = await mcpTemplateApi.listMcpTemplates()
  } catch (error) {
    ElMessage.error('加载MCP模板列表失败')
    console.error(error)
  }
}

// 加载配置历史
const loadConfigHistory = async () => {
  loading.value = true
  try {
    const configType = activeTab.value === 'template' ? 'claude' :
                     activeTab.value === 'history' ? templateForm.value.configType :
                     applyForm.value.configType
    configHistory.value = await configFileManager.getConfigHistory(configType, 20)
  } catch (error) {
    ElMessage.error('加载配置历史失败')
    console.error(error)
  } finally {
    loading.value = false
  }
}

// 生成配置预览
const generatePreview = async () => {
  loading.value = true
  try {
    const params: ConfigPreviewParams = {
      configType: templateForm.value.configType,
      claudeSupplierId: templateForm.value.claudeSupplierId || undefined,
      codexSupplierId: templateForm.value.codexSupplierId || undefined,
      mcpTemplateIds: templateForm.value.mcpTemplateIds,
      variables: templateForm.value.variables
    }

    const result = await configFileManager.previewConfig(params)
    if (result.success) {
      previewContent.value = result.data.content
      previewFormat.value = result.data.format
      ElMessage.success('配置预览生成成功')
    } else {
      ElMessage.error(result.message)
    }
  } catch (error) {
    ElMessage.error('生成配置预览失败')
    console.error(error)
  } finally {
    loading.value = false
  }
}

// 应用配置
const applyConfig = async () => {
  if (!applyForm.value.content.trim()) {
    ElMessage.warning('请输入配置内容')
    return
  }

  try {
    await ElMessageBox.confirm(
      '确定要应用此配置吗？这将覆盖现有的配置文件。',
      '确认应用配置',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )

    loading.value = true
    const result = await configFileManager.applyConfig(
      applyForm.value.configType,
      applyForm.value.content,
      applyForm.value.createBackup
    )

    if (result.success) {
      ElMessage.success('配置应用成功')
      // 刷新配置历史
      await loadConfigHistory()
    } else {
      ElMessage.error(result.message)
    }
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('应用配置失败')
      console.error(error)
    }
  } finally {
    loading.value = false
  }
}

// 备份当前配置
const backupCurrentConfig = async () => {
  if (!applyForm.value.content.trim()) {
    ElMessage.warning('请先输入配置内容')
    return
  }

  try {
    const result = await configFileManager.backupConfig(
      applyForm.value.configType,
      applyForm.value.content,
      applyForm.value.description || '手动备份'
    )

    if (result.success) {
      ElMessage.success('配置备份成功')
      await loadConfigHistory()
    } else {
      ElMessage.error(result.message)
    }
  } catch (error) {
    ElMessage.error('备份配置失败')
    console.error(error)
  }
}

// 从备份恢复配置
const restoreFromBackup = async (backup: ConfigBackup) => {
  try {
    await ElMessageBox.confirm(
      `确定要从备份 "${backup.description || backup.operationTime}" 恢复配置吗？`,
      '确认恢复配置',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )

    loading.value = true
    const result = await configFileManager.restoreConfigFromBackup(backup.id!)

    if (result.success) {
      ElMessage.success('配置恢复成功')
      applyForm.value.content = backup.backupContent
      await loadConfigHistory()
    } else {
      ElMessage.error(result.message)
    }
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('恢复配置失败')
      console.error(error)
    }
  } finally {
    loading.value = false
  }
}

// 删除备份记录
const deleteBackup = async (backup: ConfigBackup) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除备份 "${backup.description || backup.operationTime}" 吗？`,
      '确认删除备份',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )

    const result = await configFileManager.deleteConfigHistory(backup.id!)

    if (result.success) {
      ElMessage.success('备份删除成功')
      await loadConfigHistory()
    } else {
      ElMessage.error(result.message)
    }
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('删除备份失败')
      console.error(error)
    }
  }
}

// 清理旧备份
const cleanupOldBackups = async () => {
  try {
    const { value: keepCount } = await ElMessageBox.prompt(
      '请输入要保留的备份数量：',
      '清理旧备份',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        inputValue: '10',
        inputValidator: (value) => {
          const num = parseInt(value)
          if (isNaN(num) || num < 1) {
            return '请输入大于0的数字'
          }
          return true
        }
      }
    )

    if (keepCount) {
      const count = parseInt(keepCount)
      const result = await configFileManager.cleanupOldConfigHistory(
        templateForm.value.configType,
        count
      )

      if (result.success) {
        ElMessage.success(result.message)
        await loadConfigHistory()
      } else {
        ElMessage.error(result.message)
      }
    }
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('清理备份失败')
      console.error(error)
    }
  }
}

// 验证配置格式
const validateConfig = () => {
  if (!applyForm.value.content.trim()) {
    ElMessage.warning('请输入配置内容')
    return
  }

  const result = configFileManager.validateConfigFormat(
    applyForm.value.content,
    applyForm.value.configType
  )

  if (result.valid) {
    ElMessage.success('配置格式验证通过')
  } else {
    ElMessage.error(result.error || '配置格式验证失败')
  }
}

// 复制配置内容
const copyToClipboard = async (content: string) => {
  try {
    await navigator.clipboard.writeText(content)
    ElMessage.success('已复制到剪贴板')
  } catch (error) {
    ElMessage.error('复制失败')
    console.error(error)
  }
}

// 格式化配置内容
const formatConfigContent = () => {
  if (!applyForm.value.content.trim()) {
    ElMessage.warning('请输入配置内容')
    return
  }

  try {
    if (applyForm.value.configType === 'claude') {
      // JSON格式化
      const parsed = JSON.parse(applyForm.value.content)
      applyForm.value.content = JSON.stringify(parsed, null, 2)
    } else {
      // TOML格式化（简单实现）
      ElMessage.info('TOML格式化功能开发中')
    }
    ElMessage.success('配置格式化完成')
  } catch (error) {
    ElMessage.error('配置格式化失败，请检查格式是否正确')
    console.error(error)
  }
}

// 监听配置类型变化
watch(() => templateForm.value.configType, () => {
  templateForm.value.mcpTemplateIds = []
  templateForm.value.variables = {}
  previewContent.value = ''
})

watch(() => applyForm.value.configType, () => {
  loadConfigHistory()
})

// 页面加载时获取数据
onMounted(() => {
  loadSuppliers()
  loadTemplates()
  loadConfigHistory()
})
</script>

<template>
  <div class="config-manager">
    <div class="header">
      <h2>配置管理</h2>
      <div class="actions">
        <el-button @click="loadConfigHistory">
          <el-icon><Refresh /></el-icon>
          刷新
        </el-button>
      </div>
    </div>

    <div class="content">
      <el-tabs v-model="activeTab" @tab-change="loadConfigHistory">
        <!-- 模板生成标签页 -->
        <el-tab-pane label="模板生成" name="template">
          <div class="template-generator">
            <el-row :gutter="16">
              <el-col :span="8">
                <el-card title="配置参数">
                  <template #header>
                    <span>配置参数</span>
                  </template>

                  <el-form :model="templateForm" label-width="100px">
                    <el-form-item label="配置类型">
                      <el-select v-model="templateForm.configType" style="width: 100%">
                        <el-option
                          v-for="type in configTypeOptions"
                          :key="type.value"
                          :label="type.label"
                          :value="type.value"
                        />
                      </el-select>
                    </el-form-item>

                    <el-form-item v-if="templateForm.configType === 'claude'" label="Claude供应商">
                      <el-select v-model="templateForm.claudeSupplierId" placeholder="请选择供应商" style="width: 100%">
                        <el-option
                          v-for="supplier in claudeSuppliers"
                          :key="supplier.id"
                          :label="supplier.name"
                          :value="supplier.id"
                        />
                      </el-select>
                    </el-form-item>

                    <el-form-item v-if="templateForm.configType === 'codex'" label="Codex供应商">
                      <el-select v-model="templateForm.codexSupplierId" placeholder="请选择供应商" style="width: 100%">
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
                        v-model="templateForm.mcpTemplateIds"
                        multiple
                        placeholder="请选择模板"
                        style="width: 100%"
                      >
                        <el-option
                          v-for="template in availableTemplates"
                          :key="template.id"
                          :label="template.name"
                          :value="template.id"
                        />
                      </el-select>
                    </el-form-item>

                    <el-form-item label="自定义变量">
                      <div class="variables-container">
                        <div
                          v-for="(value, key) in templateForm.variables"
                          :key="key"
                          class="variable-item"
                        >
                          <el-input
                            v-model="templateForm.variables[key]"
                            :placeholder="`变量 ${key} 的值`"
                            size="small"
                          />
                          <el-button
                            size="small"
                            type="danger"
                            @click="delete templateForm.variables[key]"
                          >
                            删除
                          </el-button>
                        </div>
                        <el-button
                          size="small"
                          type="primary"
                          @click="templateForm.variables['new_var'] = ''"
                        >
                          添加变量
                        </el-button>
                      </div>
                    </el-form-item>

                    <el-form-item>
                      <el-button
                        type="primary"
                        :loading="loading"
                        @click="generatePreview"
                        style="width: 100%"
                      >
                        生成配置预览
                      </el-button>
                    </el-form-item>
                  </el-form>
                </el-card>
              </el-col>

              <el-col :span="16">
                <el-card title="配置预览">
                  <template #header>
                    <div class="card-header">
                      <span>配置预览</span>
                      <div class="header-actions">
                        <el-button
                          v-if="previewContent"
                          size="small"
                          @click="copyToClipboard(previewContent)"
                        >
                          复制
                        </el-button>
                      </div>
                    </div>
                  </template>

                  <div v-if="previewContent" class="config-preview">
                    <el-input
                      v-model="previewContent"
                      type="textarea"
                      :rows="20"
                      readonly
                      :placeholder="`请先生成${templateForm.configType === 'claude' ? 'JSON' : 'TOML'}配置`"
                    />
                  </div>
                  <div v-else class="no-preview">
                    <el-empty description="请先生成配置预览" />
                  </div>
                </el-card>
              </el-col>
            </el-row>
          </div>
        </el-tab-pane>

        <!-- 配置历史标签页 -->
        <el-tab-pane label="配置历史" name="history">
          <div class="config-history">
            <div class="history-header">
              <div class="history-actions">
                <el-button @click="cleanupOldBackups">
                  <el-icon><Delete /></el-icon>
                  清理旧备份
                </el-button>
              </div>
            </div>

            <el-table
              :data="configHistory"
              v-loading="loading"
              stripe
              style="width: 100%"
            >
              <el-table-column prop="description" label="描述" min-width="200" />
              <el-table-column prop="operationType" label="操作类型" width="120">
                <template #default="{ row }">
                  <el-tag :type="row.operationType === 'backup' ? 'success' : 'warning'">
                    {{ row.operationType === 'backup' ? '备份' : '恢复' }}
                  </el-tag>
                </template>
              </el-table-column>
              <el-table-column prop="operationTime" label="操作时间" width="180">
                <template #default="{ row }">
                  {{ new Date(row.operationTime).toLocaleString() }}
                </template>
              </el-table-column>
              <el-table-column label="操作" width="200">
                <template #default="{ row }">
                  <el-button
                    size="small"
                    type="primary"
                    @click="restoreFromBackup(row)"
                  >
                    恢复
                  </el-button>
                  <el-button
                    size="small"
                    type="danger"
                    @click="deleteBackup(row)"
                  >
                    删除
                  </el-button>
                </template>
              </el-table-column>
            </el-table>

            <div v-if="configHistory.length === 0" class="no-history">
              <el-empty description="暂无配置历史记录" />
            </div>
          </div>
        </el-tab-pane>

        <!-- 配置应用标签页 -->
        <el-tab-pane label="配置应用" name="apply">
          <div class="config-apply">
            <el-row :gutter="16">
              <el-col :span="8">
                <el-card title="配置参数">
                  <template #header>
                    <span>配置参数</span>
                  </template>

                  <el-form :model="applyForm" label-width="100px">
                    <el-form-item label="配置类型">
                      <el-select v-model="applyForm.configType" style="width: 100%">
                        <el-option
                          v-for="type in configTypeOptions"
                          :key="type.value"
                          :label="type.label"
                          :value="type.value"
                        />
                      </el-select>
                    </el-form-item>

                    <el-form-item label="备份描述">
                      <el-input
                        v-model="applyForm.description"
                        placeholder="可选，为此配置添加描述"
                      />
                    </el-form-item>

                    <el-form-item>
                      <el-checkbox v-model="applyForm.createBackup">
                        应用前创建备份
                      </el-checkbox>
                    </el-form-item>

                    <el-form-item>
                      <el-button @click="validateConfig" style="width: 100%">
                        验证配置格式
                      </el-button>
                    </el-form-item>

                    <el-form-item>
                      <el-button @click="formatConfigContent" style="width: 100%">
                        格式化配置
                      </el-button>
                    </el-form-item>

                    <el-form-item>
                      <el-button
                        v-if="applyForm.content"
                        @click="backupCurrentConfig"
                        style="width: 100%"
                      >
                        备份当前配置
                      </el-button>
                    </el-form-item>

                    <el-form-item>
                      <el-button
                        type="primary"
                        :loading="loading"
                        @click="applyConfig"
                        style="width: 100%"
                      >
                        应用配置
                      </el-button>
                    </el-form-item>
                  </el-form>
                </el-card>
              </el-col>

              <el-col :span="16">
                <el-card title="配置内容">
                  <template #header>
                    <div class="card-header">
                      <span>配置内容</span>
                      <div class="header-actions">
                        <el-button
                          v-if="applyForm.content"
                          size="small"
                          @click="copyToClipboard(applyForm.content)"
                        >
                          复制
                        </el-button>
                      </div>
                    </div>
                  </template>

                  <el-input
                    v-model="applyForm.content"
                    type="textarea"
                    :rows="25"
                    :placeholder="`请输入${applyForm.configType === 'claude' ? 'JSON' : 'TOML'}格式的配置内容`"
                  />
                </el-card>
              </el-col>
            </el-row>
          </div>
        </el-tab-pane>
      </el-tabs>
    </div>
  </div>
</template>

<style scoped>
.config-manager {
  padding: 20px;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.header h2 {
  margin: 0;
  color: #333;
}

.content {
  flex: 1;
  background: white;
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.template-generator,
.config-history,
.config-apply {
  height: 100%;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.history-header {
  display: flex;
  justify-content: flex-end;
  margin-bottom: 16px;
}

.variables-container {
  width: 100%;
}

.variable-item {
  display: flex;
  gap: 8px;
  margin-bottom: 8px;
}

.variable-item .el-input {
  flex: 1;
}

.config-preview {
  height: 100%;
}

.no-preview,
.no-history {
  height: 300px;
  display: flex;
  align-items: center;
  justify-content: center;
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

:deep(.el-input__wrapper) {
  border-radius: 6px;
}

:deep(.el-select .el-input__wrapper) {
  border-radius: 6px;
}

:deep(.el-textarea__inner) {
  border-radius: 6px;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
}

/* 卡片样式优化 */
:deep(.el-card__header) {
  padding: 16px 20px;
  border-bottom: 1px solid #ebeef5;
}

:deep(.el-card__body) {
  padding: 20px;
}
</style>