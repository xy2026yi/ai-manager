<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useAppStore } from '@/stores/app'

const appStore = useAppStore()

// 仪表盘数据
const stats = ref({
  suppliers: {
    claude: 0,
    codex: 0,
    total: 0,
  },
  mcpTemplates: {
    builtin: 0,
    custom: 0,
    total: 0,
  },
  currentMode: '未知',
  lastSync: '从未同步',
})

onMounted(async () => {
  // 加载仪表盘数据
  await loadDashboardData()
})

const loadDashboardData = async () => {
  try {
    // TODO: 调用API获取实际数据
    stats.value = {
      suppliers: {
        claude: 2,
        codex: 1,
        total: 3,
      },
      mcpTemplates: {
        builtin: 8,
        custom: 0,
        total: 8,
      },
      currentMode: '单Claude',
      lastSync: new Date().toLocaleString('zh-CN'),
    }
  } catch (error) {
    console.error('加载仪表盘数据失败:', error)
  }
}
</script>

<template>
  <div class="dashboard">
    <div class="dashboard-header">
      <h2>仪表盘</h2>
      <p class="subtitle">AI工具管理平台概览</p>
    </div>

    <!-- 统计卡片 -->
    <el-row :gutter="20" class="stats-row">
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon claude">
              <el-icon size="24"><Connection /></el-icon>
            </div>
            <div class="stat-info">
              <h3>{{ stats.suppliers.claude }}</h3>
              <p>Claude供应商</p>
            </div>
          </div>
        </el-card>
      </el-col>

      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon codex">
              <el-icon size="24"><Platform /></el-icon>
            </div>
            <div class="stat-info">
              <h3>{{ stats.suppliers.codex }}</h3>
              <p>Codex供应商</p>
            </div>
          </div>
        </el-card>
      </el-col>

      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon templates">
              <el-icon size="24"><Document /></el-icon>
            </div>
            <div class="stat-info">
              <h3>{{ stats.mcpTemplates.total }}</h3>
              <p>MCP模板</p>
            </div>
          </div>
        </el-card>
      </el-col>

      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon mode">
              <el-icon size="24"><Setting /></el-icon>
            </div>
            <div class="stat-info">
              <h3>{{ stats.currentMode }}</h3>
              <p>当前模式</p>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 快捷操作 -->
    <el-row :gutter="20" class="actions-row">
      <el-col :span="12">
        <el-card header="快捷操作">
          <div class="quick-actions">
            <el-button type="primary" size="large" @click="$router.push('/suppliers')">
              <el-icon><Plus /></el-icon>
              添加供应商
            </el-button>
            <el-button size="large" @click="$router.push('/mcp-templates')">
              <el-icon><DocumentAdd /></el-icon>
              管理MCP模板
            </el-button>
            <el-button size="large" @click="$router.push('/config-manager')">
              <el-icon><Setting /></el-icon>
              配置管理
            </el-button>
            <el-button size="large" @click="$router.push('/mode-manager')">
              <el-icon><Refresh /></el-icon>
              切换模式
            </el-button>
          </div>
        </el-card>
      </el-col>

      <el-col :span="12">
        <el-card header="系统信息">
          <div class="system-info">
            <div class="info-item">
              <span class="label">平台:</span>
              <span class="value">{{ appStore.currentPlatform || '未知' }}</span>
            </div>
            <div class="info-item">
              <span class="label">应用类型:</span>
              <span class="value">{{ appStore.isTauriApp ? '桌面应用' : 'Web应用' }}</span>
            </div>
            <div class="info-item">
              <span class="label">最后同步:</span>
              <span class="value">{{ stats.lastSync }}</span>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<style scoped>
.dashboard {
  max-width: 1200px;
  margin: 0 auto;
}

.dashboard-header {
  margin-bottom: 24px;
}

.dashboard-header h2 {
  margin: 0 0 8px 0;
  font-size: 24px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.subtitle {
  margin: 0;
  color: var(--el-text-color-regular);
  font-size: 14px;
}

.stats-row {
  margin-bottom: 24px;
}

.stat-card {
  height: 120px;
}

.stat-content {
  display: flex;
  align-items: center;
  height: 100%;
}

.stat-icon {
  width: 60px;
  height: 60px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-right: 16px;
}

.stat-icon.claude {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.stat-icon.codex {
  background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
  color: white;
}

.stat-icon.templates {
  background: linear-gradient(135deg, #4facfe 0%, #00f2fe 100%);
  color: white;
}

.stat-icon.mode {
  background: linear-gradient(135deg, #43e97b 0%, #38f9d7 100%);
  color: white;
}

.stat-info h3 {
  margin: 0 0 4px 0;
  font-size: 28px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.stat-info p {
  margin: 0;
  font-size: 14px;
  color: var(--el-text-color-regular);
}

.actions-row {
  margin-bottom: 24px;
}

.quick-actions {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.quick-actions .el-button {
  justify-content: flex-start;
}

.system-info {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.info-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.info-item .label {
  color: var(--el-text-color-regular);
  font-size: 14px;
}

.info-item .value {
  color: var(--el-text-color-primary);
  font-weight: 500;
}
</style>
