import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      redirect: '/dashboard',
    },
    {
      path: '/dashboard',
      name: 'dashboard',
      component: () => import('../views/DashboardView.vue'),
      meta: {
        title: '仪表盘',
      },
    },
    {
      path: '/suppliers',
      name: 'suppliers',
      component: () => import('../views/SupplierManager.vue'),
      meta: {
        title: '供应商管理',
      },
    },
    {
      path: '/mcp-templates',
      name: 'mcp-templates',
      component: () => import('../views/McpTemplateManager.vue'),
      meta: {
        title: 'MCP模板管理',
      },
    },
    {
      path: '/config-manager',
      name: 'config-manager',
      component: () => import('../views/ConfigManager.vue'),
      meta: {
        title: '配置管理',
      },
    },
    {
      path: '/mode-manager',
      name: 'mode-manager',
      component: () => import('../views/ModeManager.vue'),
      meta: {
        title: '模式管理',
      },
    },
    {
      path: '/supplier-switcher',
      name: 'supplier-switcher',
      component: () => import('../views/SupplierSwitcher.vue'),
      meta: {
        title: '供应商切换',
      },
    },
  ],
})

export default router
