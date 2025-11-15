import React, { useState, useEffect } from 'react';
import { Cog, Palette, Globe, Shield, Database, Bell, User } from '@heroicons/react/solid';
import { Card } from '../components/common/Card';
import { Button } from '../components/common/Button';
import { Input } from '../components/common/Input';
import { useAtom } from 'jotai';
import { themeAtom, notificationsAtom } from '../stores';
import { systemService } from '../services/api';

/**
 * 系统设置页面
 * 提供应用配置、主题设置、通知设置等功能
 */
export default function Settings() {
  const [theme, setTheme] = useAtom(themeAtom);
  const [notifications, setNotifications] = useAtom(notificationsAtom);

  // 本地状态
  const [loading, setLoading] = useState(false);
  const [systemInfo, setSystemInfo] = useState<any>(null);
  const [activeTab, setActiveTab] = useState('general');

  // 加载系统信息
  useEffect(() => {
    loadSystemInfo();
  }, []);

  const loadSystemInfo = async () => {
    try {
      setLoading(true);
      const info = await systemService.getInfo();
      setSystemInfo(info);
    } catch (error) {
      console.error('加载系统信息失败:', error);
    } finally {
      setLoading(false);
    }
  };

  // 主题设置
  const handleThemeChange = (newTheme: 'light' | 'dark' | 'system') => {
    setTheme(newTheme);

    // 添加通知
    const notification = {
      id: Date.now().toString(),
      type: 'success' as const,
      title: '主题已更改',
      message: `已切换到${newTheme === 'light' ? '浅色' : newTheme === 'dark' ? '深色' : '跟随系统'}主题`,
      duration: 3000,
    };
    setNotifications(prev => [...prev, notification]);
  };

  // 清除所有通知
  const clearAllNotifications = () => {
    setNotifications([]);
  };

  // 测试通知
  const testNotification = () => {
    const notification = {
      id: Date.now().toString(),
      type: 'info' as const,
      title: '测试通知',
      message: '这是一个测试通知，用于验证通知系统是否正常工作',
      duration: 5000,
    };
    setNotifications(prev => [...prev, notification]);
  };

  // 设置选项卡
  const tabs = [
    { id: 'general', name: '常规设置', icon: Cog },
    { id: 'appearance', name: '外观设置', icon: Palette },
    { id: 'notifications', name: '通知设置', icon: Bell },
    { id: 'system', name: '系统信息', icon: Database },
  ];

  return (
    <div className="space-y-6">
      {/* 页面标题 */}
      <div>
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
          系统设置
        </h1>
        <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
          配置应用程序的各项设置和偏好
        </p>
      </div>

      {/* 设置选项卡 */}
      <div className="border-b border-gray-200 dark:border-gray-700">
        <nav className="-mb-px flex space-x-8">
          {tabs.map((tab) => {
            const Icon = tab.icon;
            return (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`flex items-center py-2 px-1 border-b-2 font-medium text-sm ${
                  activeTab === tab.id
                    ? 'border-blue-500 text-blue-600 dark:text-blue-400'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 dark:text-gray-400 dark:hover:text-gray-300'
                }`}
              >
                <Icon className="h-5 w-5 mr-2" />
                {tab.name}
              </button>
            );
          })}
        </nav>
      </div>

      {/* 设置内容 */}
      <div className="space-y-6">
        {activeTab === 'general' && (
          <div className="space-y-6">
            <Card>
              <Card.Header>
                <Card.Title>常规设置</Card.Title>
                <Card.Description>
                  配置应用程序的基本行为和功能
                </Card.Description>
              </Card.Header>
              <Card.Content className="space-y-4">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                      语言设置
                    </label>
                    <select className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white">
                      <option value="zh-CN">简体中文</option>
                      <option value="en-US">English</option>
                    </select>
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                      时区设置
                    </label>
                    <select className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white">
                      <option value="Asia/Shanghai">Asia/Shanghai (UTC+8)</option>
                      <option value="UTC">UTC (UTC+0)</option>
                    </select>
                  </div>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                    自动保存间隔
                  </label>
                  <select className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white">
                    <option value="30">30秒</option>
                    <option value="60">1分钟</option>
                    <option value="300">5分钟</option>
                    <option value="0">禁用自动保存</option>
                  </select>
                </div>
              </Card.Content>
            </Card>

            <Card>
              <Card.Header>
                <Card.Title>数据管理</Card.Title>
                <Card.Description>
                  管理应用程序数据和缓存
                </Card.Description>
              </Card.Header>
              <Card.Content className="space-y-4">
                <div className="flex items-center justify-between">
                  <div>
                    <h3 className="text-sm font-medium text-gray-900 dark:text-white">清除缓存</h3>
                    <p className="text-sm text-gray-500 dark:text-gray-400">
                      清除应用程序缓存以释放存储空间
                    </p>
                  </div>
                  <Button variant="secondary" size="sm">
                    清除缓存
                  </Button>
                </div>

                <div className="flex items-center justify-between">
                  <div>
                    <h3 className="text-sm font-medium text-gray-900 dark:text-white">重置设置</h3>
                    <p className="text-sm text-gray-500 dark:text-gray-400">
                      将所有设置恢复为默认值
                    </p>
                  </div>
                  <Button variant="warning" size="sm">
                    重置设置
                  </Button>
                </div>
              </Card.Content>
            </Card>
          </div>
        )}

        {activeTab === 'appearance' && (
          <div className="space-y-6">
            <Card>
              <Card.Header>
                <Card.Title>主题设置</Card.Title>
                <Card.Description>
                  选择应用程序的外观主题
                </Card.Description>
              </Card.Header>
              <Card.Content>
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
                      选择主题
                    </label>
                    <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                      <button
                        onClick={() => handleThemeChange('light')}
                        className={`p-4 border-2 rounded-lg text-center ${
                          theme === 'light'
                            ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                            : 'border-gray-200 dark:border-gray-600 hover:border-gray-300 dark:hover:border-gray-500'
                        }`}
                      >
                        <div className="w-8 h-8 mx-auto mb-2 bg-white border border-gray-300 rounded"></div>
                        <div className="text-sm font-medium">浅色主题</div>
                      </button>

                      <button
                        onClick={() => handleThemeChange('dark')}
                        className={`p-4 border-2 rounded-lg text-center ${
                          theme === 'dark'
                            ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                            : 'border-gray-200 dark:border-gray-600 hover:border-gray-300 dark:hover:border-gray-500'
                        }`}
                      >
                        <div className="w-8 h-8 mx-auto mb-2 bg-gray-800 border border-gray-600 rounded"></div>
                        <div className="text-sm font-medium">深色主题</div>
                      </button>

                      <button
                        onClick={() => handleThemeChange('system')}
                        className={`p-4 border-2 rounded-lg text-center ${
                          theme === 'system'
                            ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                            : 'border-gray-200 dark:border-gray-600 hover:border-gray-300 dark:hover:border-gray-500'
                        }`}
                      >
                        <div className="w-8 h-8 mx-auto mb-2 bg-gradient-to-br from-white to-gray-800 border border-gray-400 rounded"></div>
                        <div className="text-sm font-medium">跟随系统</div>
                      </button>
                    </div>
                  </div>
                </div>
              </Card.Content>
            </Card>

            <Card>
              <Card.Header>
                <Card.Title>字体设置</Card.Title>
                <Card.Description>
                  配置应用程序的字体显示
                </Card.Description>
              </Card.Header>
              <Card.Content className="space-y-4">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                      字体系列
                    </label>
                    <select className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white">
                      <option value="system">系统默认</option>
                      <option value="sans">无衬线字体</option>
                      <option value="serif">衬线字体</option>
                      <option value="mono">等宽字体</option>
                    </select>
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                      字体大小
                    </label>
                    <select className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white">
                      <option value="small">小</option>
                      <option value="medium">中</option>
                      <option value="large">大</option>
                      <option value="extra-large">特大</option>
                    </select>
                  </div>
                </div>
              </Card.Content>
            </Card>
          </div>
        )}

        {activeTab === 'notifications' && (
          <div className="space-y-6">
            <Card>
              <Card.Header>
                <Card.Title>通知设置</Card.Title>
                <Card.Description>
                  配置应用程序的通知行为
                </Card.Description>
              </Card.Header>
              <Card.Content className="space-y-4">
                <div className="space-y-4">
                  <div className="flex items-center justify-between">
                    <div>
                      <h3 className="text-sm font-medium text-gray-900 dark:text-white">启用通知</h3>
                      <p className="text-sm text-gray-500 dark:text-gray-400">
                        允许应用程序显示通知消息
                      </p>
                    </div>
                    <input
                      type="checkbox"
                      defaultChecked={true}
                      className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                    />
                  </div>

                  <div className="flex items-center justify-between">
                    <div>
                      <h3 className="text-sm font-medium text-gray-900 dark:text-white">声音提醒</h3>
                      <p className="text-sm text-gray-500 dark:text-gray-400">
                        在显示通知时播放声音
                      </p>
                    </div>
                    <input
                      type="checkbox"
                      defaultChecked={false}
                      className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                    />
                  </div>

                  <div className="flex items-center justify-between">
                    <div>
                      <h3 className="text-sm font-medium text-gray-900 dark:text-white">桌面通知</h3>
                      <p className="text-sm text-gray-500 dark:text-gray-400">
                        显示系统桌面通知
                      </p>
                    </div>
                    <input
                      type="checkbox"
                      defaultChecked={true}
                      className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                    />
                  </div>
                </div>

                <div className="pt-4 border-t border-gray-200 dark:border-gray-700">
                  <div className="flex items-center space-x-3">
                    <Button
                      variant="primary"
                      onClick={testNotification}
                    >
                      测试通知
                    </Button>
                    <Button
                      variant="secondary"
                      onClick={clearAllNotifications}
                    >
                      清除所有通知
                    </Button>
                  </div>
                  <p className="text-xs text-gray-500 dark:text-gray-400 mt-2">
                    当前通知数量: {notifications.length}
                  </p>
                </div>
              </Card.Content>
            </Card>
          </div>
        )}

        {activeTab === 'system' && (
          <div className="space-y-6">
            <Card>
              <Card.Header>
                <Card.Title>系统信息</Card.Title>
                <Card.Description>
                  查看应用程序和系统的详细信息
                </Card.Description>
              </Card.Header>
              <Card.Content>
                {loading ? (
                  <div className="text-center py-4">
                    <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto"></div>
                    <p className="mt-2 text-sm text-gray-500 dark:text-gray-400">加载中...</p>
                  </div>
                ) : systemInfo ? (
                  <div className="space-y-4">
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                      <div>
                        <h3 className="text-sm font-medium text-gray-900 dark:text-white">应用版本</h3>
                        <p className="text-sm text-gray-500 dark:text-gray-400">{systemInfo.version || '1.0.0'}</p>
                      </div>
                      <div>
                        <h3 className="text-sm font-medium text-gray-900 dark:text-white">构建时间</h3>
                        <p className="text-sm text-gray-500 dark:text-gray-400">{systemInfo.buildTime || '未知'}</p>
                      </div>
                      <div>
                        <h3 className="text-sm font-medium text-gray-900 dark:text-white">运行环境</h3>
                        <p className="text-sm text-gray-500 dark:text-gray-400">{systemInfo.environment || 'development'}</p>
                      </div>
                      <div>
                        <h3 className="text-sm font-medium text-gray-900 dark:text-white">数据库状态</h3>
                        <p className="text-sm text-green-600 dark:text-green-400">连接正常</p>
                      </div>
                    </div>

                    <div className="pt-4 border-t border-gray-200 dark:border-gray-700">
                      <div className="flex items-center space-x-3">
                        <Button variant="secondary" onClick={loadSystemInfo}>
                          刷新信息
                        </Button>
                      </div>
                    </div>
                  </div>
                ) : (
                  <div className="text-center py-4">
                    <p className="text-sm text-gray-500 dark:text-gray-400">无法加载系统信息</p>
                    <Button variant="primary" onClick={loadSystemInfo} className="mt-2">
                      重试
                    </Button>
                  </div>
                )}
              </Card.Content>
            </Card>

            <Card>
              <Card.Header>
                <Card.Title>关于</Card.Title>
                <Card.Description>
                  关于AI Manager应用程序
                </Card.Description>
              </Card.Header>
              <Card.Content>
                <div className="space-y-4">
                  <div>
                    <h3 className="text-sm font-medium text-gray-900 dark:text-white">AI Manager</h3>
                    <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
                      一个强大的AI管理工具，用于管理和配置各种AI服务提供商
                    </p>
                  </div>

                  <div className="pt-4 border-t border-gray-200 dark:border-gray-700">
                    <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
                      <div>
                        <h4 className="font-medium text-gray-900 dark:text-white">技术栈</h4>
                        <p className="text-gray-500 dark:text-gray-400">Rust + Tauri + React</p>
                      </div>
                      <div>
                        <h4 className="font-medium text-gray-900 dark:text-white">许可证</h4>
                        <p className="text-gray-500 dark:text-gray-400">MIT</p>
                      </div>
                      <div>
                        <h4 className="font-medium text-gray-900 dark:text-white">作者</h4>
                        <p className="text-gray-500 dark:text-gray-400">AI Manager Team</p>
                      </div>
                    </div>
                  </div>
                </div>
              </Card.Content>
            </Card>
          </div>
        )}
      </div>
    </div>
  );
}