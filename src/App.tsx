import { useEffect } from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { Provider } from 'jotai';
import { useAtom } from 'jotai';

// 导入状态
import { themeAtom, notificationsAtom } from './stores';
import { systemService } from './services/api';

// 导入组件
import Layout from './components/layout/Layout';
import Dashboard from './pages/Dashboard';
import ClaudeProviders from './pages/ClaudeProviders';
import CodexProviders from './pages/CodexProviders';
import AgentGuides from './pages/AgentGuides';
import McpServers from './pages/McpServers';
import CommonConfigs from './pages/CommonConfigs';
import Settings from './pages/Settings';

import './App.css';

// 主题提供者组件
const ThemeProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [theme] = useAtom(themeAtom);

  useEffect(() => {
    if (theme === 'dark') {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  }, [theme]);

  return <>{children}</>;
};

// 通知管理组件
const NotificationManager: React.FC = () => {
  const [notifications, setNotifications] = useAtom(notificationsAtom);

  // 自动移除过期通知
  useEffect(() => {
    const timer = setInterval(() => {
      setNotifications((prev) => 
        prev.filter((notification) => 
          !notification.duration || notification.persistent
        )
      );
    }, 1000);

    return () => clearInterval(timer);
  }, [setNotifications]);

  const removeNotification = (id: string) => {
    setNotifications((prev) => prev.filter((n) => n.id !== id));
  };

  return (
    <div className="fixed bottom-4 right-4 z-50 space-y-2">
      {notifications.map((notification) => (
        <div
          key={notification.id}
          className={`max-w-sm rounded-lg shadow-lg p-4 transform transition-all duration-300 ${
            notification.type === 'success'
              ? 'bg-green-50 border-green-200 text-green-800'
              : notification.type === 'error'
              ? 'bg-red-50 border-red-200 text-red-800'
              : notification.type === 'warning'
              ? 'bg-yellow-50 border-yellow-200 text-yellow-800'
              : 'bg-blue-50 border-blue-200 text-blue-800'
          }`}
        >
          <div className="flex">
            <div className="flex-shrink-0">
              {notification.type === 'success' && (
                <svg className="h-5 w-5 text-green-400" fill="currentColor" viewBox="0 0 20 20">
                  <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000-16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 12.293a1 1 0 01-1.414 0l-5.586-5.586a1 1 0 010-1.414l6.293 6.293a1 1 0 001.414-1.414z" clipRule="evenodd" />
                </svg>
              )}
              {notification.type === 'error' && (
                <svg className="h-5 w-5 text-red-400" fill="currentColor" viewBox="0 0 20 20">
                  <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000-16zM8.707 7.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 01-1.414 1.414l2 2a1 1 0 001.414 0l4-4a1 1 0 001.414-1.414l-4-4a1 1 0 00-1.414 0z" clipRule="evenodd" />
                </svg>
              )}
              {notification.type === 'warning' && (
                <svg className="h-5 w-5 text-yellow-400" fill="currentColor" viewBox="0 0 20 20">
                  <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 3.48l-5.58-9.92zM11.43 8.536a1 1 0 10-1.414 0L8.014 9.144a1 1 0 10-1.414 0l3.414-3.414z" clipRule="evenodd" />
                </svg>
              )}
              {notification.type === 'info' && (
                <svg className="h-5 w-5 text-blue-400" fill="currentColor" viewBox="0 0 20 20">
                  <path fillRule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 001-1v-3a1 1 0 00-1-1H9z" clipRule="evenodd" />
                </svg>
              )}
            </div>
            <div className="ml-3">
              <p className="text-sm font-medium">{notification.title}</p>
              <p className="text-sm">{notification.message}</p>
            </div>
            <button
              onClick={() => removeNotification(notification.id)}
              className="ml-auto -mx-1.5 -my-1.5 rounded-lg p-1 hover:bg-gray-100"
            >
              <svg className="h-4 w-4" fill="currentColor" viewBox="0 0 20 20">
                <path fillRule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L10 10.414l4.293 4.293a1 1 0 01-1.414 0L10 11.814l-4.293 4.293a1 1 0 01-1.414-1.414L10 13.414l-4.293-4.293a1 1 0 010-1.414z" clipRule="evenodd" />
              </svg>
            </button>
          </div>
        </div>
      ))}
    </div>
  );
};

// 应用初始化组件
const AppInitializer: React.FC = () => {
  useEffect(() => {
    // 初始化应用，检查API连接
    const initializeApp = async () => {
      try {
        await systemService.getInfo();
        console.log('✅ API服务连接成功');
      } catch (error) {
        console.error('❌ API服务连接失败:', error);
      }
    };

    initializeApp();
  }, []);

  return null;
};

// 主应用组件
function App() {
  return (
    <Provider>
      <ThemeProvider>
        <AppInitializer />
        <NotificationManager />
        <Router>
          <Layout>
            <Routes>
              <Route path="/" element={<Navigate to="/dashboard" replace />} />
              <Route path="/dashboard" element={<Dashboard />} />
              <Route path="/claude-providers" element={<ClaudeProviders />} />
              <Route path="/codex-providers" element={<CodexProviders />} />
              <Route path="/agent-guides" element={<AgentGuides />} />
              <Route path="/mcp-servers" element={<McpServers />} />
              <Route path="/common-configs" element={<CommonConfigs />} />
              <Route path="/settings" element={<Settings />} />
            </Routes>
          </Layout>
        </Router>
      </ThemeProvider>
    </Provider>
  );
}

export default App;