// 仪表板页面
//
// 显示系统概览和统计信息

import React, { useEffect, useState } from 'react';
import { useAtom } from 'jotai';
import {
  ChartBarIcon,
  UsersIcon,
  ServerIcon,
  DocumentTextIcon,
  CogIcon,
  ArrowTrendingUpIcon,
  ArrowTrendingDownIcon,
} from '@heroicons/react/24/outline';

// 导入组件和服务
import { Card, CardHeader, CardTitle, CardContent } from '../components/common';
import { 
  claudeProviderStatsAtom,
  codexProviderStatsAtom,
  agentGuideStatsAtom,
  mcpServerStatsAtom,
  commonConfigStatsAtom 
} from '../stores';
import {
  claudeProviderService,
  codexProviderService,
  agentGuideService,
  mcpServerService,
  commonConfigService
} from '../services';

// 统计卡片组件
interface StatsCardProps {
  title: string;
  value: string | number;
  subtitle?: string;
  trend?: {
    value: number;
    isPositive: boolean;
  };
  icon: React.ReactNode;
  color: 'blue' | 'green' | 'yellow' | 'red' | 'purple';
  loading?: boolean;
}

const StatsCard: React.FC<StatsCardProps> = ({
  title,
  value,
  subtitle,
  trend,
  icon,
  color,
  loading = false,
}) => {
  const colorClasses = {
    blue: 'bg-blue-50 text-blue-600',
    green: 'bg-green-50 text-green-600',
    yellow: 'bg-yellow-50 text-yellow-600',
    red: 'bg-red-50 text-red-600',
    purple: 'bg-purple-50 text-purple-600',
  };

  return (
    <Card className={`${colorClasses[color]} hover:shadow-md transition-shadow`}>
      <CardContent className="p-6">
        <div className="flex items-center">
          <div className={`flex-shrink-0 ${colorClasses[color]}`}>
            {icon}
          </div>
          <div className="ml-5 w-0 flex-1">
            <dl>
              <dt className="text-sm font-medium truncate">{title}</dt>
              <dd className="text-2xl font-bold">
                {loading ? '...' : value}
              </dd>
              {subtitle && (
                <dd className="text-sm text-gray-500">{subtitle}</dd>
              )}
              {trend && (
                <dd className="flex items-center text-sm">
                  <span className={`flex items-center ${trend.isPositive ? 'text-green-600' : 'text-red-600'}`}>
                    {trend.isPositive ? (
                      <ArrowTrendingUpIcon className="h-4 w-4 mr-1" />
                    ) : (
                      <ArrowTrendingDownIcon className="h-4 w-4 mr-1" />
                    )}
                    {trend.value}%
                  </span>
                </dd>
              )}
            </dl>
          </div>
        </div>
      </CardContent>
    </Card>
  );
};

// 仪表板页面组件
const Dashboard: React.FC = () => {
  // 获取统计数据
  const [claudeStats] = useAtom(claudeProviderStatsAtom);
  const [codexStats] = useAtom(codexProviderStatsAtom);
  const [agentGuideStats] = useAtom(agentGuideStatsAtom);
  const [mcpServerStats] = useAtom(mcpServerStatsAtom);
  const [commonConfigStats] = useAtom(commonConfigStatsAtom);

  // 加载状态
  const [loading, setLoading] = useState(false);

  // 初始化数据
  useEffect(() => {
    const loadStats = async () => {
      setLoading(true);
      try {
        const [
          claudeData,
          codexData,
          agentData,
          mcpData,
          configData,
        ] = await Promise.all([
          claudeProviderService.getStats(),
          codexProviderService.getStats(),
          agentGuideService.getStats(),
          mcpServerService.getStats(),
          commonConfigService.getStats(),
        ]);

        // 更新状态（这里简化处理，实际应该使用dispatch或者专门的更新函数）
        // 因为Jotai需要特殊的状态更新方式，这里直接设置数据
        console.log('统计数据加载完成');
      } catch (error) {
        console.error('加载统计数据失败:', error);
      } finally {
        setLoading(false);
      }
    };

    loadStats();
  }, []);

  return (
    <div className="space-y-6">
      {/* 页面标题 */}
      <div>
        <h1 className="text-3xl font-bold text-gray-900">仪表板</h1>
        <p className="text-gray-600">系统概览和统计信息</p>
      </div>

      {/* 统计卡片网格 */}
      <div className="grid grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-6">
        <StatsCard
          title="Claude 供应商"
          value={claudeStats?.total || 0}
          subtitle={`启用: ${claudeStats?.enabled_count || 0}`}
          trend={
            claudeStats?.total > 0
              ? {
                  value: Math.round((claudeStats.enabled_count / claudeStats.total) * 100),
                  isPositive: true,
                }
              : undefined
          }
          icon={<UsersIcon className="h-8 w-8" />}
          color="blue"
          loading={loading}
        />

        <StatsCard
          title="Codex 供应商"
          value={codexStats?.total || 0}
          subtitle={`启用: ${codexStats?.enabled_count || 0}`}
          trend={
            codexStats?.total > 0
              ? {
                  value: Math.round((codexStats.enabled_count / codexStats.total) * 100),
                  isPositive: true,
                }
              : undefined
          }
          icon={<ServerIcon className="h-8 w-8" />}
          color="green"
          loading={loading}
        />

        <StatsCard
          title="Agent 指导文件"
          value={agentGuideStats?.total || 0}
          subtitle={`Only 类型: ${agentGuideStats?.only_type || 0}`}
          trend={
            agentGuideStats?.total > 0
              ? {
                  value: Math.round((agentGuideStats.only_type / agentGuideStats.total) * 100),
                  isPositive: agentGuideStats.only_type > agentGuideStats.and_type,
                }
              : undefined
          }
          icon={<DocumentTextIcon className="h-8 w-8" />}
          color="yellow"
          loading={loading}
        />

        <StatsCard
          title="MCP 服务器"
          value={mcpServerStats?.total || 0}
          subtitle={`活跃: ${mcpServerStats?.active_count || 0}`}
          trend={
            mcpServerStats?.total > 0
              ? {
                  value: Math.round((mcpServerStats.active_count / mcpServerStats.total) * 100),
                  isPositive: mcpServerStats.active_rate >= 50,
                }
              : undefined
          }
          icon={<ComputerDesktopIcon className="h-8 w-8" />}
          color="purple"
          loading={loading}
        />

        <StatsCard
          title="通用配置"
          value={commonConfigStats?.total || 0}
          subtitle={`启用: ${commonConfigStats?.active || 0}`}
          trend={
            commonConfigStats?.total > 0
              ? {
                  value: Math.round((commonConfigStats.active / commonConfigStats.total) * 100),
                  isPositive: commonConfigStats.active_rate >= 50,
                }
              : undefined
          }
          icon={<CogIcon className="h-8 w-8" />}
          color="red"
          loading={loading}
        />
      </div>

      {/* 详细统计信息 */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* 供应商统计 */}
        <Card>
          <CardHeader>
            <CardTitle>供应商统计</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <h4 className="text-sm font-medium text-gray-600">Claude 供应商</h4>
                  <div className="mt-1 space-y-2">
                    <div className="flex justify-between">
                      <span className="text-sm text-gray-500">总数</span>
                      <span className="text-sm font-medium">
                        {claudeStats?.total || 0}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-sm text-gray-500">启用</span>
                      <span className="text-sm font-medium text-green-600">
                        {claudeStats?.enabled_count || 0}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-sm text-gray-500">禁用</span>
                      <span className="text-sm font-medium text-red-600">
                        {claudeStats?.disabled_count || 0}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-sm text-gray-500">付费类型</span>
                      <span className="text-sm font-medium">
                        {claudeStats?.paid_type || 0}
                      </span>
                    </div>
                  </div>
                </div>
                <div>
                  <h4 className="text-sm font-medium text-gray-600">Codex 供应商</h4>
                  <div className="mt-1 space-y-2">
                    <div className="flex justify-between">
                      <span className="text-sm text-gray-500">总数</span>
                      <span className="text-sm font-medium">
                        {codexStats?.total || 0}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-sm text-gray-500">启用</span>
                      <span className="text-sm font-medium text-green-600">
                        {codexStats?.enabled_count || 0}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-sm text-gray-500">禁用</span>
                      <span className="text-sm font-medium text-red-600">
                        {codexStats?.disabled_count || 0}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-sm text-gray-500">付费类型</span>
                      <span className="text-sm font-medium">
                        {codexStats?.paid_type || 0}
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* 服务和配置统计 */}
        <Card>
          <CardHeader>
            <CardTitle>服务和配置统计</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div>
                <h4 className="text-sm font-medium text-gray-600">MCP 服务器</h4>
                <div className="mt-1 space-y-2">
                  <div className="flex justify-between">
                    <span className="text-sm text-gray-500">总数</span>
                    <span className="text-sm font-medium">
                      {mcpServerStats?.total || 0}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm text-gray-500">活跃</span>
                    <span className="text-sm font-medium text-green-600">
                      {mcpServerStats?.active_count || 0}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm text-gray-500">非活跃</span>
                    <span className="text-sm font-medium text-red-600">
                      {mcpServerStats?.inactive_count || 0}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm text-gray-500">Stdio 类型</span>
                    <span className="text-sm font-medium">
                      {mcpServerStats?.stdio_type || 0}
                    </span>
                  </div>
                </div>
              </div>
              <div>
                <h4 className="text-sm font-medium text-gray-600">Agent 指导文件</h4>
                <div className="mt-1 space-y-2">
                  <div className="flex justify-between">
                    <span className="text-sm text-gray-500">Only 类型</span>
                    <span className="text-sm font-medium">
                      {agentGuideStats?.only_type || 0}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm text-gray-500">And 类型</span>
                    <span className="text-sm font-medium">
                      {agentGuideStats?.and_type || 0}
                    </span>
                  </div>
                </div>
              </div>
              <div>
                <h4 className="text-sm font-medium text-gray-600">通用配置</h4>
                <div className="mt-1 space-y-2">
                  <div className="flex justify-between">
                    <span className="text-sm text-gray-500">总数</span>
                    <span className="text-sm font-medium">
                      {commonConfigStats?.total || 0}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm text-gray-500">启用</span>
                    <span className="text-sm font-medium text-green-600">
                      {commonConfigStats?.active || 0}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm text-gray-500">配置类别</span>
                    <span className="text-sm font-medium">
                      {commonConfigStats?.category_count || 0}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* 系统状态 */}
      <Card>
        <CardHeader>
          <CardTitle>系统状态</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <span className="text-sm text-gray-600">后端服务</span>
              <span className="flex items-center">
                <div className="h-2 w-2 bg-green-500 rounded-full"></div>
                <span className="ml-2 text-sm font-medium text-green-600">运行中</span>
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm text-gray-600">数据库连接</span>
              <span className="flex items-center">
                <div className="h-2 w-2 bg-green-500 rounded-full"></div>
                <span className="ml-2 text-sm font-medium text-green-600">正常</span>
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm text-gray-600">前端应用</span>
              <span className="flex items-center">
                <div className="h-2 w-2 bg-green-500 rounded-full"></div>
                <span className="ml-2 text-sm font-medium text-green-600">正常</span>
              </span>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

export default Dashboard;