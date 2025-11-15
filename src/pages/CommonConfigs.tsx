import React, { useState, useEffect } from 'react';
import { Plus, Edit, Trash2, Save, Refresh, Cog, Settings } from '@heroicons/react/solid';
import { Card } from '../components/common/Card';
import { Button } from '../components/common/Button';
import { Modal } from '../components/common/Modal';
import { Input } from '../components/common/Input';
import { Table } from '../components/common/Table';
import {
  commonConfigsAtom,
  selectedCommonConfigAtom,
  loadingAtom,
  errorAtom,
  refreshTriggerAtom
} from '../stores';
import { useAtom } from 'jotai';
import { CommonConfig, CommonConfigService } from '../services/api';

/**
 * 通用配置管理页面
 * 提供系统通用配置的增删改查功能
 */
export default function CommonConfigs() {
  // 状态管理
  const [configs, setConfigs] = useAtom(commonConfigsAtom);
  const [selectedConfig, setSelectedConfig] = useAtom(selectedCommonConfigAtom);
  const [loading, setLoading] = useAtom(loadingAtom);
  const [error, setError] = useAtom(errorAtom);
  const [refreshTrigger] = useAtom(refreshTriggerAtom);

  // 本地状态
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [isEditModalOpen, setIsEditModalOpen] = useState(false);
  const [isDeleteModalOpen, setIsDeleteModalOpen] = useState(false);
  const [isBatchEditModalOpen, setIsBatchEditModalOpen] = useState(false);
  const [formData, setFormData] = useState<Partial<CommonConfig>>({});
  const [formErrors, setFormErrors] = useState<Record<string, string>>({});
  const [batchFormData, setBatchFormData] = useState<Record<string, string>>({});

  const configService = new CommonConfigService();

  // 加载配置列表
  useEffect(() => {
    loadConfigs();
  }, [refreshTrigger]);

  const loadConfigs = async () => {
    try {
      setLoading(true);
      setError(null);
      const result = await configService.list({ limit: 100 });
      setConfigs(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : '加载配置列表失败');
    } finally {
      setLoading(false);
    }
  };

  // 验证表单数据
  const validateForm = (data: Partial<CommonConfig>): Record<string, string> => {
    const errors: Record<string, string> = {};

    if (!data.config_key?.trim()) {
      errors.config_key = '配置键不能为空';
    } else if (!/^[a-zA-Z][a-zA-Z0-9_]*$/.test(data.config_key)) {
      errors.config_key = '配置键只能包含字母、数字和下划线，且必须以字母开头';
    }

    if (data.config_value === undefined || data.config_value === null) {
      errors.config_value = '配置值不能为空';
    }

    if (!data.category?.trim()) {
      errors.category = '分类不能为空';
    }

    return errors;
  };

  // 重置表单
  const resetForm = () => {
    setFormData({});
    setFormErrors({});
  };

  // 打开创建模态框
  const handleCreate = () => {
    resetForm();
    setFormData({ is_enabled: true });
    setIsCreateModalOpen(true);
  };

  // 创建配置
  const handleCreateSubmit = async () => {
    const errors = validateForm(formData);
    if (Object.keys(errors).length > 0) {
      setFormErrors(errors);
      return;
    }

    try {
      setLoading(true);
      await configService.create(formData as Omit<CommonConfig, 'id' | 'created_at' | 'updated_at'>);
      setIsCreateModalOpen(false);
      resetForm();
      await loadConfigs();
    } catch (err) {
      setError(err instanceof Error ? err.message : '创建配置失败');
    } finally {
      setLoading(false);
    }
  };

  // 打开编辑模态框
  const handleEdit = (config: CommonConfig) => {
    setSelectedConfig(config);
    setFormData({ ...config });
    setFormErrors({});
    setIsEditModalOpen(true);
  };

  // 更新配置
  const handleEditSubmit = async () => {
    if (!selectedConfig) return;

    const errors = validateForm(formData);
    if (Object.keys(errors).length > 0) {
      setFormErrors(errors);
      return;
    }

    try {
      setLoading(true);
      await configService.update(selectedConfig.id, formData as Partial<CommonConfig>);
      setIsEditModalOpen(false);
      setSelectedConfig(null);
      resetForm();
      await loadConfigs();
    } catch (err) {
      setError(err instanceof Error ? err.message : '更新配置失败');
    } finally {
      setLoading(false);
    }
  };

  // 打开删除确认框
  const handleDelete = (config: CommonConfig) => {
    setSelectedConfig(config);
    setIsDeleteModalOpen(true);
  };

  // 删除配置
  const handleDeleteSubmit = async () => {
    if (!selectedConfig) return;

    try {
      setLoading(true);
      await configService.delete(selectedConfig.id);
      setIsDeleteModalOpen(false);
      setSelectedConfig(null);
      await loadConfigs();
    } catch (err) {
      setError(err instanceof Error ? err.message : '删除配置失败');
    } finally {
      setLoading(false);
    }
  };

  // 切换启用状态
  const handleToggleEnabled = async (config: CommonConfig) => {
    try {
      setLoading(true);
      await configService.update(config.id, { is_enabled: !config.is_enabled });
      await loadConfigs();
    } catch (err) {
      setError(err instanceof Error ? err.message : '切换启用状态失败');
    } finally {
      setLoading(false);
    }
  };

  // 打开批量编辑模态框
  const handleBatchEdit = () => {
    const initialBatchData: Record<string, string> = {};
    configs?.data?.forEach(config => {
      initialBatchData[config.id.toString()] = config.config_value;
    });
    setBatchFormData(initialBatchData);
    setIsBatchEditModalOpen(true);
  };

  // 批量保存配置
  const handleBatchSave = async () => {
    try {
      setLoading(true);
      const promises = Object.entries(batchFormData).map(([id, value]) =>
        configService.update(parseInt(id), { config_value: value })
      );
      await Promise.all(promises);
      setIsBatchEditModalOpen(false);
      setBatchFormData({});
      await loadConfigs();
    } catch (err) {
      setError(err instanceof Error ? err.message : '批量保存配置失败');
    } finally {
      setLoading(false);
    }
  };

  // 格式化配置值显示
  const formatConfigValue = (value: string, maxLength: number = 50) => {
    if (value.length <= maxLength) return value;
    return value.substring(0, maxLength) + '...';
  };

  // 获取配置类型图标
  const getConfigIcon = (category: string) => {
    const iconMap: Record<string, string> = {
      'system': '⚙️',
      'api': '🔌',
      'database': '🗄️',
      'security': '🔐',
      'ui': '🎨',
      'performance': '⚡',
      'logging': '📝',
      'network': '🌐',
    };
    return iconMap[category] || '📋';
  };

  // 表格列定义
  const columns = [
    {
      key: 'config_key',
      title: '配置键',
      render: (config: CommonConfig) => (
        <div className="flex items-center">
          <span className="text-lg mr-2">{getConfigIcon(config.category)}</span>
          <div>
            <div className="text-sm font-medium text-gray-900 dark:text-white font-mono">
              {config.config_key}
            </div>
            <div className="text-xs text-gray-500 dark:text-gray-400">
              {config.category}
            </div>
          </div>
        </div>
      ),
    },
    {
      key: 'config_value',
      title: '配置值',
      render: (config: CommonConfig) => (
        <div className="max-w-md">
          <code className="text-sm bg-gray-100 dark:bg-gray-800 px-2 py-1 rounded text-gray-900 dark:text-white">
            {formatConfigValue(config.config_value)}
          </code>
          {config.config_value.length > 50 && (
            <Button
              variant="ghost"
              size="sm"
              onClick={() => handleEdit(config)}
              className="mt-1 p-0 text-xs"
            >
              查看完整值
            </Button>
          )}
        </div>
      ),
    },
    {
      key: 'description',
      title: '描述',
      render: (config: CommonConfig) => (
        <div className="text-sm text-gray-600 dark:text-gray-400 max-w-xs">
          {config.description || '-'}
        </div>
      ),
    },
    {
      key: 'status',
      title: '状态',
      render: (config: CommonConfig) => (
        <span
          className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
            config.is_enabled
              ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
              : 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200'
          }`}
        >
          {config.is_enabled ? '已启用' : '已禁用'}
        </span>
      ),
    },
    {
      key: 'updated_at',
      title: '更新时间',
      render: (config: CommonConfig) => (
        <div className="text-sm text-gray-500 dark:text-gray-400">
          {new Date(config.updated_at).toLocaleDateString('zh-CN')}
        </div>
      ),
    },
    {
      key: 'actions',
      title: '操作',
      render: (config: CommonConfig) => (
        <div className="flex items-center space-x-2">
          <Button
            variant={config.is_enabled ? "warning" : "success"}
            size="sm"
            onClick={() => handleToggleEnabled(config)}
            disabled={loading}
          >
            {config.is_enabled ? '禁用' : '启用'}
          </Button>
          <Button
            variant="primary"
            size="sm"
            onClick={() => handleEdit(config)}
            disabled={loading}
          >
            <Edit className="h-4 w-4" />
          </Button>
          <Button
            variant="danger"
            size="sm"
            onClick={() => handleDelete(config)}
            disabled={loading}
          >
            <Trash2 className="h-4 w-4" />
          </Button>
        </div>
      ),
    },
  ];

  // 按分类分组统计
  const getStatsByCategory = () => {
    if (!configs?.data) return {};
    return configs.data.reduce((acc, config) => {
      const category = config.category;
      if (!acc[category]) {
        acc[category] = { total: 0, enabled: 0 };
      }
      acc[category].total++;
      if (config.is_enabled) {
        acc[category].enabled++;
      }
      return acc;
    }, {} as Record<string, { total: number; enabled: number }>);
  };

  const categoryStats = getStatsByCategory();

  return (
    <div className="space-y-6">
      {/* 页面标题和操作 */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
            通用配置管理
          </h1>
          <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
            管理系统的各种配置参数，包括API设置、数据库连接、安全配置等
          </p>
        </div>
        <div className="flex items-center space-x-3">
          <Button
            variant="secondary"
            onClick={handleBatchEdit}
            disabled={loading || !configs?.data?.length}
            className="flex items-center space-x-2"
          >
            <Settings className="h-4 w-4" />
            <span>批量编辑</span>
          </Button>
          <Button
            variant="primary"
            onClick={handleCreate}
            disabled={loading}
            className="flex items-center space-x-2"
          >
            <Plus className="h-4 w-4" />
            <span>添加配置</span>
          </Button>
        </div>
      </div>

      {/* 统计信息 */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
        <Card>
          <Card.Header>
            <Card.Title>总配置数</Card.Title>
          </Card.Header>
          <Card.Content>
            <div className="text-2xl font-bold text-gray-900 dark:text-white">
              {configs?.total || 0}
            </div>
          </Card.Content>
        </Card>
        <Card>
          <Card.Header>
            <Card.Title>已启用</Card.Title>
          </Card.Header>
          <Card.Content>
            <div className="text-2xl font-bold text-green-600 dark:text-green-400">
              {configs?.data?.filter(c => c.is_enabled).length || 0}
            </div>
          </Card.Content>
        </Card>
        <Card>
          <Card.Header>
            <Card.Title>配置分类</Card.Title>
          </Card.Header>
          <Card.Content>
            <div className="text-2xl font-bold text-blue-600 dark:text-blue-400">
              {Object.keys(categoryStats).length}
            </div>
          </Card.Content>
        </Card>
        <Card>
          <Card.Header>
            <Card.Title>启用率</Card.Title>
          </Card.Header>
          <Card.Content>
            <div className="text-2xl font-bold text-purple-600 dark:text-purple-400">
              {configs?.data?.length
                ? Math.round((configs.data.filter(c => c.is_enabled).length / configs.data.length) * 100)
                : 0}%
            </div>
          </Card.Content>
        </Card>
      </div>

      {/* 分类统计详情 */}
      {Object.keys(categoryStats).length > 0 && (
        <Card>
          <Card.Header>
            <Card.Title>分类统计</Card.Title>
          </Card.Header>
          <Card.Content>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {Object.entries(categoryStats).map(([category, stats]) => (
                <div key={category} className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
                  <div className="flex items-center">
                    <span className="text-lg mr-2">{getConfigIcon(category)}</span>
                    <span className="text-sm font-medium text-gray-900 dark:text-white capitalize">
                      {category}
                    </span>
                  </div>
                  <div className="text-sm text-gray-600 dark:text-gray-400">
                    <span className="font-medium text-green-600 dark:text-green-400">{stats.enabled}</span>
                    <span className="mx-1">/</span>
                    <span>{stats.total}</span>
                  </div>
                </div>
              ))}
            </div>
          </Card.Content>
        </Card>
      )}

      {/* 配置列表 */}
      <Card>
        <Card.Header>
          <Card.Title>配置列表</Card.Title>
        </Card.Header>
        <Card.Content>
          <Table
            data={configs?.data || []}
            columns={columns}
            loading={loading}
            emptyState={{
              title: '暂无配置',
              description: '请点击"添加配置"按钮来创建第一个系统配置',
            }}
          />
        </Card.Content>
      </Card>

      {/* 创建配置模态框 */}
      <Modal
        isOpen={isCreateModalOpen}
        onClose={() => setIsCreateModalOpen(false)}
        title="添加系统配置"
        footer={
          <div className="flex justify-end space-x-3">
            <Button
              variant="secondary"
              onClick={() => setIsCreateModalOpen(false)}
              disabled={loading}
            >
              取消
            </Button>
            <Button
              variant="primary"
              onClick={handleCreateSubmit}
              disabled={loading}
            >
              创建
            </Button>
          </div>
        }
      >
        <div className="space-y-4">
          <Input
            label="配置键"
            value={formData.config_key || ''}
            onChange={(value) => setFormData(prev => ({ ...prev, config_key: value }))}
            placeholder="API_TIMEOUT"
            error={formErrors.config_key}
            required
            help="只能包含字母、数字和下划线，且必须以字母开头"
          />
          <Input
            label="分类"
            value={formData.category || ''}
            onChange={(value) => setFormData(prev => ({ ...prev, category: value }))}
            placeholder="system"
            error={formErrors.category}
            required
            help="如：system, api, database, security, ui等"
          />
          <div className="space-y-2">
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
              配置值 <span className="text-red-500">*</span>
            </label>
            <textarea
              value={formData.config_value || ''}
              onChange={(e) => setFormData(prev => ({ ...prev, config_value: e.target.value }))}
              placeholder="请输入配置值..."
              rows={3}
              className={`w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white font-mono text-sm ${
                formErrors.config_value ? 'border-red-500' : ''
              }`}
            />
            {formErrors.config_value && (
              <p className="text-sm text-red-500">{formErrors.config_value}</p>
            )}
          </div>
          <div className="space-y-2">
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
              描述
            </label>
            <textarea
              value={formData.description || ''}
              onChange={(e) => setFormData(prev => ({ ...prev, description: e.target.value }))}
              placeholder="请输入配置描述（可选）..."
              rows={2}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
            />
          </div>
        </div>
      </Modal>

      {/* 编辑配置模态框 */}
      <Modal
        isOpen={isEditModalOpen}
        onClose={() => setIsEditModalOpen(false)}
        title="编辑系统配置"
        footer={
          <div className="flex justify-end space-x-3">
            <Button
              variant="secondary"
              onClick={() => setIsEditModalOpen(false)}
              disabled={loading}
            >
              取消
            </Button>
            <Button
              variant="primary"
              onClick={handleEditSubmit}
              disabled={loading}
            >
              保存
            </Button>
          </div>
        }
      >
        <div className="space-y-4">
          <Input
            label="配置键"
            value={formData.config_key || ''}
            onChange={(value) => setFormData(prev => ({ ...prev, config_key: value }))}
            placeholder="API_TIMEOUT"
            error={formErrors.config_key}
            required
            disabled
          />
          <Input
            label="分类"
            value={formData.category || ''}
            onChange={(value) => setFormData(prev => ({ ...prev, category: value }))}
            placeholder="system"
            error={formErrors.category}
            required
          />
          <div className="space-y-2">
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
              配置值 <span className="text-red-500">*</span>
            </label>
            <textarea
              value={formData.config_value || ''}
              onChange={(e) => setFormData(prev => ({ ...prev, config_value: e.target.value }))}
              placeholder="请输入配置值..."
              rows={3}
              className={`w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white font-mono text-sm ${
                formErrors.config_value ? 'border-red-500' : ''
              }`}
            />
            {formErrors.config_value && (
              <p className="text-sm text-red-500">{formErrors.config_value}</p>
            )}
          </div>
          <div className="space-y-2">
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
              描述
            </label>
            <textarea
              value={formData.description || ''}
              onChange={(e) => setFormData(prev => ({ ...prev, description: e.target.value }))}
              placeholder="请输入配置描述（可选）..."
              rows={2}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
            />
          </div>
        </div>
      </Modal>

      {/* 批量编辑模态框 */}
      <Modal
        isOpen={isBatchEditModalOpen}
        onClose={() => setIsBatchEditModalOpen(false)}
        title="批量编辑配置"
        size="lg"
        footer={
          <div className="flex justify-end space-x-3">
            <Button
              variant="secondary"
              onClick={() => setIsBatchEditModalOpen(false)}
              disabled={loading}
            >
              取消
            </Button>
            <Button
              variant="primary"
              onClick={handleBatchSave}
              disabled={loading}
              className="flex items-center space-x-2"
            >
              <Save className="h-4 w-4" />
              <span>批量保存</span>
            </Button>
          </div>
        }
      >
        <div className="space-y-4 max-h-96 overflow-y-auto">
          {configs?.data?.map(config => (
            <div key={config.id} className="space-y-2">
              <div className="flex items-center justify-between">
                <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
                  {getConfigIcon(config.category)} {config.config_key}
                  <span className="ml-2 text-xs text-gray-500">({config.category})</span>
                </label>
              </div>
              <textarea
                value={batchFormData[config.id.toString()] || ''}
                onChange={(e) => setBatchFormData(prev => ({
                  ...prev,
                  [config.id.toString()]: e.target.value
                }))}
                rows={2}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white font-mono text-sm"
              />
              {config.description && (
                <p className="text-xs text-gray-500 dark:text-gray-400">{config.description}</p>
              )}
            </div>
          ))}
        </div>
      </Modal>

      {/* 删除确认模态框 */}
      <Modal
        isOpen={isDeleteModalOpen}
        onClose={() => setIsDeleteModalOpen(false)}
        title="删除配置"
        size="sm"
        footer={
          <div className="flex justify-end space-x-3">
            <Button
              variant="secondary"
              onClick={() => setIsDeleteModalOpen(false)}
              disabled={loading}
            >
              取消
            </Button>
            <Button
              variant="danger"
              onClick={handleDeleteSubmit}
              disabled={loading}
            >
              删除
            </Button>
          </div>
        }
      >
        <div className="text-center">
          <div className="mt-2">
            <p className="text-sm text-gray-500 dark:text-gray-400">
              确定要删除配置 <span className="font-medium text-gray-900 dark:text-white">{selectedConfig?.config_key}</span> 吗？
            </p>
            <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
              此操作不可撤销，配置将被永久删除。
            </p>
          </div>
        </div>
      </Modal>
    </div>
  );
}