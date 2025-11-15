import React, { useState, useEffect } from 'react';
import { Plus, Edit, Trash2, Eye, EyeOff, Key, Cog } from '@heroicons/react/solid';
import { Card } from '../components/common/Card';
import { Button } from '../components/common/Button';
import { Modal } from '../components/common/Modal';
import { Input } from '../components/common/Input';
import { Table } from '../components/common/Table';
import {
  codexProvidersAtom,
  selectedCodexProviderAtom,
  loadingAtom,
  errorAtom,
  refreshTriggerAtom
} from '../stores';
import { useAtom } from 'jotai';
import { CodexProvider, CodexProviderService } from '../services/api';

/**
 * Codex供应商管理页面
 * 提供Codex供应商的增删改查功能
 */
export default function CodexProviders() {
  // 状态管理
  const [providers, setProviders] = useAtom(codexProvidersAtom);
  const [selectedProvider, setSelectedProvider] = useAtom(selectedCodexProviderAtom);
  const [loading, setLoading] = useAtom(loadingAtom);
  const [error, setError] = useAtom(errorAtom);
  const [refreshTrigger] = useAtom(refreshTriggerAtom);

  // 本地状态
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [isEditModalOpen, setIsEditModalOpen] = useState(false);
  const [isDeleteModalOpen, setIsDeleteModalOpen] = useState(false);
  const [showToken, setShowToken] = useState<{ [key: number]: boolean }>({});
  const [formData, setFormData] = useState<Partial<CodexProvider>>({});
  const [formErrors, setFormErrors] = useState<Record<string, string>>({});

  const providerService = new CodexProviderService();

  // 加载供应商列表
  useEffect(() => {
    loadProviders();
  }, [refreshTrigger]);

  const loadProviders = async () => {
    try {
      setLoading(true);
      setError(null);
      const result = await providerService.list({ limit: 100 });
      setProviders(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : '加载供应商列表失败');
    } finally {
      setLoading(false);
    }
  };

  // 验证表单数据
  const validateForm = (data: Partial<CodexProvider>): Record<string, string> => {
    const errors: Record<string, string> = {};

    if (!data.name?.trim()) {
      errors.name = '供应商名称不能为空';
    }

    if (!data.url?.trim()) {
      errors.url = 'API地址不能为空';
    } else {
      try {
        new URL(data.url);
      } catch {
        errors.url = '请输入有效的URL地址';
      }
    }

    if (!data.token?.trim()) {
      errors.token = 'API密钥不能为空';
    }

    if (data.timeout && data.timeout < 1) {
      errors.timeout = '超时时间必须大于0秒';
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
    setIsCreateModalOpen(true);
  };

  // 创建供应商
  const handleCreateSubmit = async () => {
    const errors = validateForm(formData);
    if (Object.keys(errors).length > 0) {
      setFormErrors(errors);
      return;
    }

    try {
      setLoading(true);
      await providerService.create(formData as Omit<CodexProvider, 'id' | 'created_at' | 'updated_at'>);
      setIsCreateModalOpen(false);
      resetForm();
      await loadProviders();
    } catch (err) {
      setError(err instanceof Error ? err.message : '创建供应商失败');
    } finally {
      setLoading(false);
    }
  };

  // 打开编辑模态框
  const handleEdit = (provider: CodexProvider) => {
    setSelectedProvider(provider);
    setFormData({ ...provider });
    setFormErrors({});
    setIsEditModalOpen(true);
  };

  // 更新供应商
  const handleEditSubmit = async () => {
    if (!selectedProvider) return;

    const errors = validateForm(formData);
    if (Object.keys(errors).length > 0) {
      setFormErrors(errors);
      return;
    }

    try {
      setLoading(true);
      await providerService.update(selectedProvider.id, formData as Partial<CodexProvider>);
      setIsEditModalOpen(false);
      setSelectedProvider(null);
      resetForm();
      await loadProviders();
    } catch (err) {
      setError(err instanceof Error ? err.message : '更新供应商失败');
    } finally {
      setLoading(false);
    }
  };

  // 打开删除确认框
  const handleDelete = (provider: CodexProvider) => {
    setSelectedProvider(provider);
    setIsDeleteModalOpen(true);
  };

  // 删除供应商
  const handleDeleteSubmit = async () => {
    if (!selectedProvider) return;

    try {
      setLoading(true);
      await providerService.delete(selectedProvider.id);
      setIsDeleteModalOpen(false);
      setSelectedProvider(null);
      await loadProviders();
    } catch (err) {
      setError(err instanceof Error ? err.message : '删除供应商失败');
    } finally {
      setLoading(false);
    }
  };

  // 切换启用状态
  const handleToggleEnabled = async (provider: CodexProvider) => {
    try {
      setLoading(true);
      await providerService.update(provider.id, { is_enabled: !provider.is_enabled });
      await loadProviders();
    } catch (err) {
      setError(err instanceof Error ? err.message : '切换启用状态失败');
    } finally {
      setLoading(false);
    }
  };

  // 切换Token显示
  const toggleShowToken = (id: number) => {
    setShowToken(prev => ({ ...prev, [id]: !prev[id] }));
  };

  // 表格列定义
  const columns = [
    {
      key: 'name',
      title: '供应商名称',
      render: (provider: CodexProvider) => (
        <div className="flex items-center">
          <div className="flex-shrink-0 h-10 w-10 bg-blue-100 rounded-full flex items-center justify-center">
            <Cog className="h-6 w-6 text-blue-600" />
          </div>
          <div className="ml-4">
            <div className="text-sm font-medium text-gray-900 dark:text-white">
              {provider.name}
            </div>
            <div className="text-sm text-gray-500 dark:text-gray-400">
              {provider.url}
            </div>
          </div>
        </div>
      ),
    },
    {
      key: 'version',
      title: 'API版本',
      render: (provider: CodexProvider) => (
        <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200">
          {provider.version || 'v1'}
        </span>
      ),
    },
    {
      key: 'token',
      title: 'API密钥',
      render: (provider: CodexProvider) => (
        <div className="flex items-center space-x-2">
          <Key className="h-4 w-4 text-gray-400" />
          <span className="text-sm font-mono text-gray-900 dark:text-white">
            {showToken[provider.id]
              ? provider.token
              : '•'.repeat(Math.min(provider.token.length, 20))
            }
          </span>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => toggleShowToken(provider.id)}
            className="p-1"
          >
            {showToken[provider.id] ? (
              <EyeOff className="h-4 w-4" />
            ) : (
              <Eye className="h-4 w-4" />
            )}
          </Button>
        </div>
      ),
    },
    {
      key: 'status',
      title: '状态',
      render: (provider: CodexProvider) => (
        <span
          className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
            provider.is_enabled
              ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
              : 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200'
          }`}
        >
          {provider.is_enabled ? '已启用' : '已禁用'}
        </span>
      ),
    },
    {
      key: 'timeout',
      title: '超时设置',
      render: (provider: CodexProvider) => (
        <span className="text-sm text-gray-900 dark:text-white">
          {provider.timeout}s
        </span>
      ),
    },
    {
      key: 'actions',
      title: '操作',
      render: (provider: CodexProvider) => (
        <div className="flex items-center space-x-2">
          <Button
            variant={provider.is_enabled ? "warning" : "success"}
            size="sm"
            onClick={() => handleToggleEnabled(provider)}
            disabled={loading}
          >
            {provider.is_enabled ? '禁用' : '启用'}
          </Button>
          <Button
            variant="primary"
            size="sm"
            onClick={() => handleEdit(provider)}
            disabled={loading}
          >
            <Edit className="h-4 w-4" />
          </Button>
          <Button
            variant="danger"
            size="sm"
            onClick={() => handleDelete(provider)}
            disabled={loading}
          >
            <Trash2 className="h-4 w-4" />
          </Button>
        </div>
      ),
    },
  ];

  return (
    <div className="space-y-6">
      {/* 页面标题和操作 */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
            Codex供应商管理
          </h1>
          <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
            管理Codex API供应商配置，包括密钥、版本设置等
          </p>
        </div>
        <Button
          variant="primary"
          onClick={handleCreate}
          disabled={loading}
          className="flex items-center space-x-2"
        >
          <Plus className="h-4 w-4" />
          <span>添加供应商</span>
        </Button>
      </div>

      {/* 统计信息 */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <Card>
          <Card.Header>
            <Card.Title>总供应商数</Card.Title>
          </Card.Header>
          <Card.Content>
            <div className="text-2xl font-bold text-gray-900 dark:text-white">
              {providers?.total || 0}
            </div>
          </Card.Content>
        </Card>
        <Card>
          <Card.Header>
            <Card.Title>已启用</Card.Title>
          </Card.Header>
          <Card.Content>
            <div className="text-2xl font-bold text-green-600 dark:text-green-400">
              {providers?.data?.filter(p => p.is_enabled).length || 0}
            </div>
          </Card.Content>
        </Card>
        <Card>
          <Card.Header>
            <Card.Title>已禁用</Card.Title>
          </Card.Header>
          <Card.Content>
            <div className="text-2xl font-bold text-gray-600 dark:text-gray-400">
              {providers?.data?.filter(p => !p.is_enabled).length || 0}
            </div>
          </Card.Content>
        </Card>
      </div>

      {/* 供应商列表 */}
      <Card>
        <Card.Header>
          <Card.Title>供应商列表</Card.Title>
        </Card.Header>
        <Card.Content>
          <Table
            data={providers?.data || []}
            columns={columns}
            loading={loading}
            emptyState={{
              title: '暂无供应商',
              description: '请点击"添加供应商"按钮来创建第一个Codex供应商配置',
            }}
          />
        </Card.Content>
      </Card>

      {/* 创建供应商模态框 */}
      <Modal
        isOpen={isCreateModalOpen}
        onClose={() => setIsCreateModalOpen(false)}
        title="添加Codex供应商"
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
            label="供应商名称"
            value={formData.name || ''}
            onChange={(value) => setFormData(prev => ({ ...prev, name: value }))}
            placeholder="请输入供应商名称"
            error={formErrors.name}
            required
          />
          <Input
            label="API地址"
            value={formData.url || ''}
            onChange={(value) => setFormData(prev => ({ ...prev, url: value }))}
            placeholder="https://api.openai.com/v1"
            error={formErrors.url}
            required
          />
          <Input
            label="API版本"
            value={formData.version || ''}
            onChange={(value) => setFormData(prev => ({ ...prev, version: value }))}
            placeholder="v1"
          />
          <Input
            label="API密钥"
            value={formData.token || ''}
            onChange={(value) => setFormData(prev => ({ ...prev, token: value }))}
            placeholder="请输入API密钥"
            type="password"
            error={formErrors.token}
            required
          />
          <Input
            label="超时时间(秒)"
            value={formData.timeout?.toString() || ''}
            onChange={(value) => setFormData(prev => ({
              ...prev,
              timeout: value ? parseInt(value) : undefined
            }))}
            placeholder="30"
            type="number"
            error={formErrors.timeout}
          />
        </div>
      </Modal>

      {/* 编辑供应商模态框 */}
      <Modal
        isOpen={isEditModalOpen}
        onClose={() => setIsEditModalOpen(false)}
        title="编辑Codex供应商"
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
            label="供应商名称"
            value={formData.name || ''}
            onChange={(value) => setFormData(prev => ({ ...prev, name: value }))}
            placeholder="请输入供应商名称"
            error={formErrors.name}
            required
          />
          <Input
            label="API地址"
            value={formData.url || ''}
            onChange={(value) => setFormData(prev => ({ ...prev, url: value }))}
            placeholder="https://api.openai.com/v1"
            error={formErrors.url}
            required
          />
          <Input
            label="API版本"
            value={formData.version || ''}
            onChange={(value) => setFormData(prev => ({ ...prev, version: value }))}
            placeholder="v1"
          />
          <Input
            label="API密钥"
            value={formData.token || ''}
            onChange={(value) => setFormData(prev => ({ ...prev, token: value }))}
            placeholder="请输入API密钥"
            type="password"
            error={formErrors.token}
            required
          />
          <Input
            label="超时时间(秒)"
            value={formData.timeout?.toString() || ''}
            onChange={(value) => setFormData(prev => ({
              ...prev,
              timeout: value ? parseInt(value) : undefined
            }))}
            placeholder="30"
            type="number"
            error={formErrors.timeout}
          />
        </div>
      </Modal>

      {/* 删除确认模态框 */}
      <Modal
        isOpen={isDeleteModalOpen}
        onClose={() => setIsDeleteModalOpen(false)}
        title="删除供应商"
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
              确定要删除供应商 <span className="font-medium text-gray-900 dark:text-white">{selectedProvider?.name}</span> 吗？
            </p>
            <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
              此操作不可撤销，所有相关配置将被永久删除。
            </p>
          </div>
        </div>
      </Modal>
    </div>
  );
}