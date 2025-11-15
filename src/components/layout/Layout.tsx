// 主布局组件
//
// 提供应用的整体布局结构，包括侧边栏、顶部导航和主内容区
// 支持响应式设计、键盘导航和优化的用户体验

import React, { useState, useEffect } from 'react';
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
  SunIcon,
  MoonIcon,
  BellIcon,
  MagnifyingGlassIcon,
} from '@heroicons/react/24/outline';

// 导入状态
import { sidebarCollapsedAtom, themeAtom, notificationsAtom } from '../../stores';
import { useKeyboardShortcuts } from '../../hooks/useKeyboardShortcuts';

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
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);

  const toggleSidebar = () => {
    setSidebarCollapsed(!sidebarCollapsed);
  };

  const toggleMobileMenu = () => {
    setIsMobileMenuOpen(!isMobileMenuOpen);
  };

  // 键盘快捷键支持
  useKeyboardShortcuts({
    shortcuts: [
      {
        key: 'Ctrl+B',
        action: toggleSidebar,
        description: '切换侧边栏',
        priority: 2
      },
      {
        key: 'Ctrl+M',
        action: toggleMobileMenu,
        description: '切换移动端菜单',
        priority: 2
      }
    ],
    enabled: true,
    global: true
  });

  return (
    <>
      {/* 移动端遮罩层 */}
      {isMobileMenuOpen && (
        <div
          className="fixed inset-0 z-40 bg-gray-600 bg-opacity-50 backdrop-blur-sm lg:hidden animate-fade-in"
          onClick={toggleMobileMenu}
        />
      )}

      {/* 桌面端侧边栏 */}
      <div
        className={`
          fixed inset-y-0 left-0 z-50 bg-gray-800 text-white transition-all duration-300 ease-in-out
          transform lg:transform-none
          ${sidebarCollapsed ? 'w-16 -translate-x-full lg:translate-x-0' : 'w-64'}
          ${isMobileMenuOpen ? 'translate-x-0' : '-translate-x-full'}
        `}
      >
        {/* 侧边栏头部 */}
        <div className="flex items-center justify-between h-16 px-4 border-b border-gray-700">
          <h1
            className={`
              text-xl font-bold transition-all duration-300
              ${sidebarCollapsed ? 'opacity-0 w-0 overflow-hidden' : 'opacity-100'}
            `}
          >
            AI Manager
          </h1>
          <button
            onClick={toggleSidebar}
            className="hidden lg:block p-1 rounded-md hover:bg-gray-700 transition-colors focus:outline-none focus:ring-2 focus:ring-inset focus:ring-primary-500"
            aria-label="切换侧边栏"
          >
            {sidebarCollapsed ? (
              <ChevronRightIcon className="h-5 w-5" />
            ) : (
              <ChevronLeftIcon className="h-5 w-5" />
            )}
          </button>

          {/* 移动端关闭按钮 */}
          <button
            onClick={toggleMobileMenu}
            className="lg:hidden p-1 rounded-md hover:bg-gray-700 transition-colors focus:outline-none focus:ring-2 focus:ring-inset focus:ring-primary-500"
            aria-label="关闭菜单"
          >
            <XMarkIcon className="h-5 w-5" />
          </button>
        </div>

        {/* 导航菜单 */}
        <nav className="flex-1 px-2 py-4 space-y-1 overflow-y-auto">
          {menuItems.map((item) => {
            const isActive = location.pathname === item.path;
            return (
              <Link
                key={item.id}
                to={item.path}
                onClick={() => setIsMobileMenuOpen(false)}
                className={`
                  group flex items-center px-3 py-2 text-sm font-medium rounded-md transition-all duration-200
                  hover:scale-105 active:scale-105
                  ${isActive
                    ? 'bg-primary-600 text-white shadow-lg'
                    : 'text-gray-300 hover:bg-gray-700 hover:text-white'
                  }
                  ${sidebarCollapsed ? 'justify-center' : 'justify-start'}
                `}
                aria-label={item.label}
                title={sidebarCollapsed ? item.label : undefined}
              >
                <item.icon className={`h-6 w-6 flex-shrink-0 ${sidebarCollapsed ? '' : 'mr-3'}`} />
                <span className={`${sidebarCollapsed ? 'hidden' : 'block'} transition-all duration-200`}>
                  {item.label}
                </span>
                {/* 激活状态指示器 */}
                {isActive && (
                  <div className="absolute left-0 top-0 bottom-0 w-1 bg-white rounded-r-full animate-pulse" />
                )}
              </Link>
            );
          })}
        </nav>

        {/* 侧边栏底部 */}
        <div className="px-2 py-4 border-t border-gray-700 space-y-2">
          <Link
            to="/settings"
            onClick={() => setIsMobileMenuOpen(false)}
            className={`
              group flex items-center px-3 py-2 text-sm font-medium rounded-md transition-all duration-200
              hover:scale-105 active:scale-105
              ${location.pathname === '/settings'
                ? 'bg-primary-600 text-white shadow-lg'
                : 'text-gray-300 hover:bg-gray-700 hover:text-white'
              }
              ${sidebarCollapsed ? 'justify-center' : 'justify-start'}
            `}
            aria-label="设置"
            title={sidebarCollapsed ? '设置' : undefined}
          >
            <Cog6ToothIcon className={`h-6 w-6 flex-shrink-0 ${sidebarCollapsed ? '' : 'mr-3'}`} />
            <span className={`${sidebarCollapsed ? 'hidden' : 'block'} transition-all duration-200`}>
              设置
            </span>
            {/* 激活状态指示器 */}
            {location.pathname === '/settings' && (
              <div className="absolute left-0 top-0 bottom-0 w-1 bg-white rounded-r-full animate-pulse" />
            )}
          </Link>

          {/* 快捷键提示 */}
          {sidebarCollapsed && (
            <div className="text-center text-xs text-gray-400 px-2">
              按 <kbd className="px-1 py-0.5 text-xs bg-gray-700 rounded">B</kbd> 展开
            </div>
          )}
        </div>
      </div>
    </>
  );
};

// 顶部导航栏组件
const TopNav: React.FC = () => {
  const [theme, setTheme] = useAtom(themeAtom);
  const [sidebarCollapsed] = useAtom(sidebarCollapsedAtom);
  const [notifications] = useAtom(notificationsAtom);
  const [showSearch, setShowSearch] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const searchInputRef = useRef<HTMLInputElement>(null);

  const toggleTheme = () => {
    setTheme(theme === 'light' ? 'dark' : 'light');
  };

  // 快捷键支持
  useKeyboardShortcuts({
    shortcuts: [
      {
        key: 'Ctrl+K',
        action: () => {
          setShowSearch(!showSearch);
          if (!showSearch) {
            setTimeout(() => searchInputRef.current?.focus(), 100);
          }
        },
        description: '搜索',
        priority: 1
      },
      {
        key: 'Ctrl+T',
        action: toggleTheme,
        description: '切换主题',
        priority: 2
      }
    ],
    enabled: true,
    global: true
  });

  // 模拟通知数据
  const hasNotifications = notifications.length > 0;

  return (
    <div className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-4 py-3 sm:px-6 lg:px-8 transition-all duration-200">
      <div className="flex items-center justify-between">
        <div className="flex items-center min-w-0 flex-1">
          {/* 移动端菜单按钮 */}
          <button
            onClick={() => window.dispatchEvent(new CustomEvent('toggleMobileMenu'))}
            className="lg:hidden p-2 rounded-md text-gray-400 hover:text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors mr-3"
            aria-label="打开侧边栏"
          >
            <Bars3Icon className="h-6 w-6" />
          </button>

          <div className="flex items-center">
            <div className="h-8 w-8 bg-gradient-to-br from-primary-500 to-primary-600 rounded-lg flex items-center justify-center text-white font-bold shadow-sm">
              A
            </div>
            <div className="ml-3">
              <h2 className="text-lg font-semibold text-gray-900 dark:text-white truncate">
                AI Manager
              </h2>
              <p className="text-xs text-gray-500 dark:text-gray-400 hidden sm:block">
                迁移版本 • 高性能桌面应用
              </p>
            </div>
          </div>
        </div>

        <div className="flex items-center space-x-2 sm:space-x-4">
          {/* 搜索框 */}
          <div className="relative">
            {showSearch && (
              <div className="absolute right-0 top-0 z-50 animate-fade-in">
                <div className="flex items-center bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-lg shadow-lg">
                  <input
                    ref={searchInputRef}
                    type="text"
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    placeholder="搜索..."
                    className="w-48 sm:w-64 px-3 py-2 text-sm border-0 focus:ring-0 bg-transparent"
                    onBlur={() => setTimeout(() => setShowSearch(false), 200)}
                  />
                  <button
                    onClick={() => setShowSearch(false)}
                    className="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 mr-1"
                  >
                    <XMarkIcon className="h-4 w-4" />
                  </button>
                </div>
              </div>
            )}

            <button
              onClick={() => setShowSearch(!showSearch)}
              className="p-2 rounded-lg text-gray-400 hover:text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
              title="搜索 (Ctrl+K)"
            >
              <MagnifyingGlassIcon className="h-5 w-5" />
            </button>
          </div>

          {/* 通知按钮 */}
          <div className="relative">
            <button
              className="p-2 rounded-lg text-gray-400 hover:text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors relative"
              title="通知"
            >
              <BellIcon className="h-5 w-5" />
              {hasNotifications && (
                <span className="absolute top-1 right-1 h-2 w-2 bg-error-500 rounded-full animate-pulse" />
              )}
            </button>
          </div>

          {/* 主题切换 */}
          <button
            onClick={toggleTheme}
            className="p-2 rounded-lg text-gray-400 hover:text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-700 transition-all duration-200 group"
            title={theme === 'light' ? '切换到深色模式 (Ctrl+T)' : '切换到浅色模式 (Ctrl+T)'}
          >
            <div className="relative">
              {theme === 'light' ? (
                <MoonIcon className="h-5 w-5 group-hover:rotate-12 transition-transform" />
              ) : (
                <SunIcon className="h-5 w-5 group-hover:rotate-12 transition-transform" />
              )}
            </div>
          </button>

          {/* 用户头像 */}
          <div className="h-8 w-8 bg-gradient-to-br from-secondary-400 to-secondary-500 rounded-full flex items-center justify-center text-white text-sm font-medium shadow-sm cursor-pointer hover:shadow-md transition-shadow">
            <span>U</span>
          </div>
        </div>
      </div>
    </div>
  );
};

// 主内容区域组件
const MainContent: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [sidebarCollapsed] = useAtom(sidebarCollapsedAtom);
  const [theme] = useAtom(themeAtom);
  const [scrollY, setScrollY] = useState(0);
  const [showScrollTop, setShowScrollTop] = useState(false);

  // 监听滚动事件
  useEffect(() => {
    const handleScroll = () => {
      setScrollY(window.scrollY);
      setShowScrollTop(window.scrollY > 200);
    };

    window.addEventListener('scroll', handleScroll);
    return () => window.removeEventListener('scroll', handleScroll);
  }, []);

  // 滚动到顶部
  const scrollToTop = () => {
    window.scrollTo({
      top: 0,
      behavior: 'smooth'
    });
  };

  // 快捷键支持
  useKeyboardShortcuts({
    shortcuts: [
      {
        key: 'Home',
        action: scrollToTop,
        description: '滚动到顶部',
        priority: 3
      },
      {
        key: 'End',
        action: () => window.scrollTo({ top: document.body.scrollHeight, behavior: 'smooth' }),
        description: '滚动到底部',
        priority: 3
      }
    ],
    enabled: true,
    global: true
  });

  return (
    <main
      className={`flex-1 overflow-y-auto bg-gray-50 dark:bg-gray-900 transition-all duration-300 ease-in-out relative ${
        sidebarCollapsed ? 'lg:ml-16' : 'lg:ml-64'
      }`}
    >
      <div className="min-h-screen">
        {/* 进度指示器 */}
        {scrollY > 100 && (
          <div className="fixed top-0 left-0 right-0 z-50 lg:left-16 lg:left-64 transition-all duration-300">
            <div
              className="h-1 bg-primary-500 dark:bg-primary-400 transition-all duration-150"
              style={{
                width: `${Math.min((scrollY / (document.body.scrollHeight - window.innerHeight)) * 100, 100)}%`
              }}
            />
          </div>
        )}

        {/* 返回顶部按钮 */}
        {showScrollTop && (
          <button
            onClick={scrollToTop}
            className="fixed bottom-6 right-6 z-40 p-3 bg-primary-500 hover:bg-primary-600 text-white rounded-full shadow-lg hover:shadow-xl transition-all duration-300 transform hover:scale-110 animate-bounce-in"
            title="返回顶部 (Home)"
          >
            <svg className="h-5 w-5" fill="none" viewBox="0 0 20 20">
              <path fillRule="evenodd" d="M14.707 12.707a1 1 0 01-1.414 0L10 9.414l-3.293 3.293a1 1 0 01-1.414-1.414l4-4a1 1 0 011.414 0l4 4a1 1 0 010 1.414z" clipRule="evenodd" />
            </svg>
          </button>
        )}

        {/* 顶部导航栏在移动端时显示 */}
        <div className="lg:hidden sticky top-0 z-30">
          <TopNav />
        </div>

        {/* 桌面端顶部导航 */}
        <div className="hidden lg:block sticky top-0 z-30">
          <TopNav />
        </div>

        {/* 页面内容 */}
        <div className="p-4 sm:p-6 lg:p-8 xl:p-10 transition-all duration-200">
          <div className="max-w-7xl mx-auto">
            {children}
          </div>
        </div>

        {/* 页脚 */}
        <footer className="mt-16 border-t border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 transition-colors">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
            <div className="text-center">
              <p className="text-sm text-gray-500 dark:text-gray-400">
                AI Manager 迁移版本 • 高性能 Rust/Tauri 桌面应用
              </p>
              <div className="mt-2 flex items-center justify-center space-x-4 text-xs text-gray-400 dark:text-gray-500">
                <span>版本 0.1.0</span>
                <span>•</span>
                <span>构建时间 {new Date().toLocaleDateString('zh-CN')}</span>
                <span>•</span>
                <span>响应时间 &lt; 500ms</span>
              </div>
            </div>
          </div>
        </footer>
      </div>
    </main>
  );
};

// 响应式包装器组件
const ResponsiveWrapper: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [sidebarCollapsed] = useAtom(sidebarCollapsedAtom);
  const [theme] = useAtom(themeAtom);

  // 监听窗口大小变化，优化移动端体验
  useEffect(() => {
    const handleResize = () => {
      // 在小屏幕上自动收起侧边栏
      if (window.innerWidth < 1024 && !sidebarCollapsed) {
        // 这里可以通过事件或状态管理来收起侧边栏
        window.dispatchEvent(new CustomEvent('autoCollapseSidebar'));
      }
    };

    window.addEventListener('resize', handleResize);
    handleResize(); // 初始检查

    return () => window.removeEventListener('resize', handleResize);
  }, [sidebarCollapsed]);

  return (
    <div className={`flex h-screen ${theme === 'dark' ? 'dark' : ''}`}>
      <div className="flex-1 flex bg-gray-100 dark:bg-gray-950 transition-colors duration-200">
        {/* 侧边栏始终显示 */}
        <Sidebar />

        {/* 主内容区域 */}
        <MainContent>{children}</MainContent>
      </div>

      {/* 全局快捷键帮助对话框 */}
      <KeyboardShortcutsHelp />
    </div>
  );
};

// 键盘快捷键帮助对话框
const KeyboardShortcutsHelp: React.FC = () => {
  const [isOpen, setIsOpen] = useState(false);

  // 快捷键显示帮助
  useKeyboardShortcuts({
    shortcuts: [
      {
        key: '?',
        action: () => setIsOpen(!isOpen),
        description: '显示快捷键帮助',
        priority: 4
      }
    ],
    enabled: true,
    global: true
  });

  const shortcuts = [
    { key: 'Ctrl+B', description: '切换侧边栏' },
    { key: 'Ctrl+M', description: '切换移动端菜单' },
    { key: 'Ctrl+K', description: '搜索' },
    { key: 'Ctrl+T', description: '切换主题' },
    { key: 'Ctrl+/', description: '显示快捷键帮助' },
    { key: 'Alt+1-7', description: '快速导航到不同页面' },
    { key: 'Home', description: '滚动到顶部' },
    { key: 'End', description: '滚动到底部' },
    { key: 'Esc', description: '关闭弹窗/取消操作' },
  ];

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50 backdrop-blur-sm animate-fade-in">
      <div className="bg-white dark:bg-gray-800 rounded-xl shadow-2xl max-w-md w-full mx-4 animate-bounce-in">
        <div className="flex items-center justify-between p-6 border-b border-gray-200 dark:border-gray-700">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
            键盘快捷键
          </h3>
          <button
            onClick={() => setIsOpen(false)}
            className="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
          >
            <XMarkIcon className="h-5 w-5" />
          </button>
        </div>

        <div className="p-6 max-h-96 overflow-y-auto">
          <div className="space-y-3">
            {shortcuts.map((shortcut, index) => (
              <div key={index} className="flex items-center justify-between">
                <kbd className="px-2 py-1 text-xs font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded">
                  {shortcut.key}
                </kbd>
                <span className="text-sm text-gray-600 dark:text-gray-400">
                  {shortcut.description}
                </span>
              </div>
            ))}
          </div>
        </div>

        <div className="p-4 border-t border-gray-200 dark:border-gray-700">
          <p className="text-xs text-gray-500 dark:text-gray-400 text-center">
            提示：按 <kbd className="px-1 py-0.5 text-xs bg-gray-100 dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded">ESC</kbd> 关闭此窗口
          </p>
        </div>
      </div>
    </div>
  );
};

// 布局组件
const Layout: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  return <ResponsiveWrapper>{children}</ResponsiveWrapper>;
};

export default Layout;