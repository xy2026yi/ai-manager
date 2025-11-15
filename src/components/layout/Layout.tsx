// 主布局组件
//
// 提供应用的整体布局结构，包括侧边栏、顶部导航和主内容区

import React, { useState } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { useAtom } from 'jotai';
import {
  HomeIcon,
  UserGroupIcon,
  Cog6ToothIcon,
  ServerIcon,
  DocumentTextIcon,
  AdjustmentsHorizontalIcon,
  ComputerDesktopIcon,
  Bars3Icon,
  XMarkIcon,
  ChevronLeftIcon,
  ChevronRightIcon,
} from '@heroicons/react/24/outline';

// 导入状态
import { sidebarCollapsedAtom, themeAtom } from '../../stores';

// 菜单项配置
const menuItems = [
  {
    id: 'dashboard',
    label: '仪表板',
    icon: HomeIcon,
    path: '/dashboard',
  },
  {
    id: 'claude-providers',
    label: 'Claude 供应商',
    icon: UserGroupIcon,
    path: '/claude-providers',
  },
  {
    id: 'codex-providers',
    label: 'Codex 供应商',
    icon: ServerIcon,
    path: '/codex-providers',
  },
  {
    id: 'agent-guides',
    label: 'Agent 指导文件',
    icon: DocumentTextIcon,
    path: '/agent-guides',
  },
  {
    id: 'mcp-servers',
    label: 'MCP 服务器',
    icon: ComputerDesktopIcon,
    path: '/mcp-servers',
  },
  {
    id: 'common-configs',
    label: '通用配置',
    icon: AdjustmentsHorizontalIcon,
    path: '/common-configs',
  },
];

// 侧边栏组件
const Sidebar: React.FC = () => {
  const [sidebarCollapsed, setSidebarCollapsed] = useAtom(sidebarCollapsedAtom);
  const location = useLocation();

  const toggleSidebar = () => {
    setSidebarCollapsed(!sidebarCollapsed);
  };

  return (
    <div className={`fixed inset-y-0 left-0 z-50 bg-gray-800 text-white transition-all duration-300 ease-in-out ${
      sidebarCollapsed ? 'w-16' : 'w-64'
    }`}>
      {/* 侧边栏头部 */}
      <div className="flex items-center justify-between h-16 px-4 border-b border-gray-700">
        <h1
          className={`text-xl font-bold transition-all duration-300 ${
            sidebarCollapsed ? 'opacity-0' : 'opacity-100'
          }`}
        >
          AI Manager
        </h1>
        <button
          onClick={toggleSidebar}
          className="p-1 rounded-md hover:bg-gray-700 transition-colors"
        >
          {sidebarCollapsed ? (
            <ChevronRightIcon className="h-5 w-5" />
          ) : (
            <ChevronLeftIcon className="h-5 w-5" />
          )}
        </button>
      </div>

      {/* 导航菜单 */}
      <nav className="flex-1 px-2 py-4 space-y-1">
        {menuItems.map((item) => (
          <Link
            key={item.id}
            to={item.path}
            className={`group flex items-center px-3 py-2 text-sm font-medium rounded-md transition-colors ${
              location.pathname === item.path
                ? 'bg-gray-900 text-white'
                : 'text-gray-300 hover:bg-gray-700 hover:text-white'
            }`}
          >
            <item.icon className="h-6 w-6 flex-shrink-0" />
            <span className={`ml-3 ${sidebarCollapsed ? 'hidden' : ''}`}>
              {item.label}
            </span>
          </Link>
        ))}
      </nav>

      {/* 侧边栏底部 */}
      <div className="px-2 py-4 border-t border-gray-700">
        <Link
          to="/settings"
          className={`group flex items-center px-3 py-2 text-sm font-medium rounded-md transition-colors ${
            location.pathname === '/settings'
              ? 'bg-gray-900 text-white'
              : 'text-gray-300 hover:bg-gray-700 hover:text-white'
          }`}
        >
          <Cog6ToothIcon className="h-6 w-6 flex-shrink-0" />
          <span className={`ml-3 ${sidebarCollapsed ? 'hidden' : ''}`}>
            设置
          </span>
        </Link>
      </div>
    </div>
  );
};

// 顶部导航栏组件
const TopNav: React.FC = () => {
  const [theme, setTheme] = useAtom(themeAtom);

  const toggleTheme = () => {
    setTheme(theme === 'light' ? 'dark' : 'light');
  };

  return (
    <div className="bg-white border-b border-gray-200 px-4 py-4 sm:px-6 lg:px-8">
      <div className="flex items-center justify-between">
        <div className="flex items-center">
          <div className="h-8 w-8 bg-blue-600 rounded-lg flex items-center justify-center text-white font-bold">
            A
          </div>
          <h2 className="ml-3 text-xl font-semibold text-gray-900">
            AI Manager - 迁移版本
          </h2>
        </div>

        <div className="flex items-center space-x-4">
          {/* 主题切换 */}
          <button
            onClick={toggleTheme}
            className="p-2 rounded-lg hover:bg-gray-100 transition-colors"
            title={theme === 'light' ? '切换到深色模式' : '切换到浅色模式'}
          >
            {theme === 'light' ? (
              <svg className="h-5 w-5 text-gray-500" fill="none" viewBox="0 0 20 20">
                <path d="M17.293 13.293A8 8 0 016.586 0H8a8 8 0 01 5.657 2.343A8.001 8.001 0 1015.707 13.293z" />
              </svg>
            ) : (
              <svg className="h-5 w-5 text-gray-500" fill="none" viewBox="0 0 24 24">
                <path d="M21.752 15.002A9.718 9.718 0 0118.254 0 9.718 9.718 0 0118.254-2.018 9.718 9.718 0 0118.254 2.018z" />
                <path d="M9.754 18.25A7.727 7.727 0 018.001 0V5.25a7.727 7.727 0 018.001 0 13z" />
              </svg>
            )}
          </button>

          {/* 其他操作按钮 */}
          <div className="h-8 w-8 bg-gray-200 rounded-full animate-pulse"></div>
        </div>
      </div>
    </div>
  );
};

// 主内容区域组件
const MainContent: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [sidebarCollapsed] = useAtom(sidebarCollapsedAtom);

  return (
    <main
      className={`flex-1 overflow-y-auto bg-gray-50 transition-all duration-300 ease-in-out ${
        sidebarCollapsed ? 'lg:ml-16' : 'lg:ml-64'
      }`}
    >
      <div className="min-h-screen">
        {/* 顶部导航栏在移动端时显示 */}
        <div className="lg:hidden">
          <TopNav />
        </div>
        
        {/* 页面内容 */}
        <div className="p-4 sm:p-6 lg:p-8">
          {children}
        </div>
      </div>
    </main>
  );
};

// 响应式包装器组件
const ResponsiveWrapper: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [sidebarCollapsed] = useAtom(sidebarCollapsedAtom);

  return (
    <div className="flex h-screen bg-gray-100">
      {/* 侧边栏始终显示 */}
      <Sidebar />
      
      {/* 在桌面端显示顶部导航 */}
      <div className="hidden lg:block">
        <TopNav />
      </div>

      {/* 主内容 */}
      <MainContent>{children}</MainContent>
    </div>
  );
};

// 布局组件
const Layout: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  return <ResponsiveWrapper>{children}</ResponsiveWrapper>;
};

export default Layout;