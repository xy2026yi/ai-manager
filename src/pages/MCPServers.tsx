import React, { useState, useEffect } from 'react';
import { Plus, Edit, Trash2, Eye, EyeOff, Server, Cpu, Activity, Link } from '@heroicons/react/solid';
import { Card } from '../components/common/Card';
import { Button } from '../components/common/Button';
import { Modal } from '../components/common/Modal';
import { Input } from '../components/common/Input';
import { Table } from '../components/common/Table';
import {
  mcpServersAtom,
  selectedMCPServerAtom,
  loadingAtom,
  errorAtom,
  refreshTriggerAtom
} from '../stores';
import { useAtom } from 'jotai';
import { MCPServer, MCPServerService } from '../services/api';

/**
 * MCP服务器管理页面
 * 提供MCP服务器的增删改查功能
 */
export default function MCPServers() {
  // 状态管理
  const [servers, setServers] = useAtom(mcpServersAtom);
  const [selectedServer, setSelectedServer] = useAtom(selectedMCPServerAtom);
  const [loading, setLoading] = useAtom(loadingAtom);
  const [error, setError] = useAtom(errorAtom);
  const [refreshTrigger] = useAtom(refreshTriggerAtom);

  // 本地状态
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [isEditModalOpen, setIsEditModalOpen] = useState(false);
  const [isDeleteModalOpen, setIsDeleteModalOpen] = useState(false);
  const [showToken, setShowToken] = useState<{ [key: number]: boolean }>({});
  const [formData, setFormData] = useState<Partial<MCPServer>>({});
  const [formErrors, setFormErrors] = useState<Record<string, string>>({});

  const serverService = new MCPServerService();

  // 加载服务器列表
  useEffect(() => {
    loadServers();
  }, [refreshTrigger]);

  const loadServers = async () => {
    try {
      setLoading(true);
      setError(null);
      const result = await serverService.list({ limit: 100 });
      setServers(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : '加载MCP服务器列表失败');
    } finally {
      setLoading(false);
    }
  };

  // 验证表单数据
  const validateForm = (data: Partial<MCPServer>): Record<string, string> => {
    const errors: Record<string, string> = {};

    if (!data.name?.trim()) {
      errors.name = '服务器名称不能为空';
    }

    if (!data.host?.trim()) {
      errors.host = '主机地址不能为空';
    }

    if (data.port && (data.port < 1 || data.port > 65535)) {
      errors.port = '端口号必须在1-65535之间';
    }

    if (data.timeout && data.timeout < 1) {
      errors.timeout = '超时时间必须大于0秒';
    }

    if (data.max_connections && data.max_connections < 1) {
      errors.max_connections = '最大连接数必须大于0';
    }

    if (data.retry_count && data.retry_count < 0) {
      errors.retry_count = '重试次数不能为负数';
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
    setFormData({
      port: 8080,
      protocol: 'http',
      is_enabled: true,
      timeout: 30,
      max_connections: 10,
      retry_count: 3,
    });
    setIsCreateModalOpen(true);
  };

  // 创建服务器
  const handleCreateSubmit = async () => {
    const errors = validateForm(formData);
    if (Object.keys(errors).length > 0) {
      setFormErrors(errors);
      return;
    }

    try {
      setLoading(true);
      await serverService.create(formData as Omit<MCPServer, 'id' | 'created_at' | 'updated_at'>);
      setIsCreateModalOpen(false);
      resetForm();
      await loadServers();
    } catch (err) {
      setError(err instanceof Error ? err.message : '创建MCP服务器失败');
    } finally {
      setLoading(false);
    }
  };

  // 打开编辑模态框
  const handleEdit = (server: MCPServer) => {
    setSelectedServer(server);
    setFormData({ ...server });
    setFormErrors({});
    setIsEditModalOpen(true);
  };

  // 更新服务器
  const handleEditSubmit = async () => {
    if (!selectedServer) return;

    const errors = validateForm(formData);
    if (Object.keys(errors).length > 0) {
      setFormErrors(errors);
      return;
    }

    try {
      setLoading(true);
      await serverService.update(selectedServer.id, formData as Partial<MCPServer>);
      setIsEditModalOpen(false);
      setSelectedServer(null);
      resetForm();
      await loadServers();
    } catch (err) {
      setError(err instanceof Error ? err.message : '更新MCP服务器失败');
    } finally {
      setLoading(false);
    }
  };

  // 打开删除确认框
  const handleDelete = (server: MCPServer) => {
    setSelectedServer(server);
    setIsDeleteModalOpen(true);
  };

  // 删除服务器
  const handleDeleteSubmit = async () => {
    if (!selectedServer) return;

    try {
      setLoading(true);
      await serverService.delete(selectedServer.id);
      setIsDeleteModalOpen(false);
      setSelectedServer(null);
      await loadServers();
    } catch (err) {
      setError(err instanceof Error ? err.message : '删除MCP服务器失败');
    } finally {
      setLoading(false);
    }
  };

  // 切换启用状态
  const handleToggleEnabled = async (server: MCPServer) => {
    try {
      setLoading(true);
      await serverService.update(server.id, { is_enabled: !server.is_enabled });
      await loadServers();
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

  // 构建完整地址
  const buildAddress = (server: MCPServer) => {
    const protocol = server.protocol || 'http';
    const port = server.port || (protocol === 'https' ? 443 : 80);
    return `${protocol}://${server.host}:${port}`;
  };

  // 表格列定义
  const columns = [
    {
      key: 'name',
      title: '服务器名称',
      render: (server: MCPServer) => (
        <div className="flex items-center">
          <div className="flex-shrink-0 h-10 w-10 bg-purple-100 rounded-full flex items-center justify-center">
            <Server className="h-6 w-6 text-purple-600" />
          </div>
          <div className="ml-4">
            <div className="text-sm font-medium text-gray-900 dark:text-white">
              {server.name}
            </div>
            <div className="text-sm text-gray-500 dark:text-gray-400">
              {server.description || '无描述'}
            </div>
          </div>
        </div>
      ),
    },
    {
      key: 'address',
      title: '服务器地址',
      render: (server: MCPServer) => (
        <div className="flex items-center space-x-2">
          <Link className="h-4 w-4 text-gray-400" />
          <span className="text-sm font-mono text-gray-900 dark:text-white">
            {buildAddress(server)}
          </span>
        </div>
      ),
    },
    {
      key: 'token',
      title: '认证Token',
      render: (server: MCPServer) => (
        <div className="flex items-center space-x-2">
          <span className="text-sm font-mono text-gray-900 dark:text-white">
            {server.auth_token
              ? showToken[server.id]
                ? server.auth_token
                : '•'.repeat(Math.min(server.auth_token.length, 16))
              : '无认证'
            }
          </span>
          {server.auth_token && (
            <Button
              variant="ghost"
              size="sm"
              onClick={() => toggleShowToken(server.id)}
              className="p-1"
            >
              {showToken[server.id] ? (
                <EyeOff className="h-4 w-4" />
              ) : (
                <Eye className="h-4 w-4" />
              )}
            </Button>
          )}
        </div>
      ),
    },
    {
      key: 'connection',
      title: '连接配置',
      render: (server: MCPServer) => (
        <div className="text-sm text-gray-900 dark:text-white">
          <div>最大连接：{server.max_connections}</div>
          <div>超时：{server.timeout}s</div>
          <div>重试：{server.retry_count}次</div>
        </div>
      ),
    },
    {
      key: 'status',
      title: '状态',
      render: (server: MCPServer) => (
        <div className="flex flex-col space-y-1">
          <span
            className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
              server.is_enabled
                ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
                : 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200'
            }`}
          >
            {server.is_enabled ? '已启用' : '已禁用'}
          </span>
          <div className="flex items-center text-xs text-gray-500 dark:text-gray-400">
            <Activity className="h-3 w-3 mr-1" />
            {server.is_enabled ? '运行中' : '已停止'}
          </div>
        </div>
      ),
    },
    {
      key: 'actions',
      title: '操作',
      render: (server: MCPServer) => (
        <div className="flex items-center space-x-2">
          <Button
            variant={server.is_enabled ? "warning" : "success"}
            size="sm"
            onClick={() => handleToggleEnabled(server)}
            disabled={loading}
          >
            {server.is_enabled ? '禁用' : '启用'}
          </Button>
          <Button
            variant="primary"
            size="sm"
            onClick={() => handleEdit(server)}
            disabled={loading}
          >
            <Edit className="h-4 w-4" />
          </Button>
          <Button
            variant="danger"
            size="sm"
            onClick={() => handleDelete(server)}
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
            MCP服务器管理
          </h1>
          <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
            管理Model Context Protocol服务器配置，包括连接设置和认证信息
          </p>
        </div>
        <Button
          variant="primary"
          onClick={handleCreate}
          disabled={loading}
          className="flex items-center space-x-2"
        >
          <Plus className="h-4 w-4" />
          <span>添加服务器</span>
        </Button>
      </div>

      {/* 统计信息 */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <Card>
          <Card.Header>
            <Card.Title>总服务器数</Card.Title>
          </Card.Header>
          <Card.Content>
            <div className="text-2xl font-bold text-gray-900 dark:text-white">
              {servers?.total || 0}
            </div>
          </Card.Content>
        </Card>
        <Card>
          <Card.Header>
            <Card.Title>已启用</Card.Title>
          </Card.Header>
          <Card.Content>
            <div className="text-2xl font-bold text-green-600 dark:text-green-400">
              {servers?.data?.filter(s => s.is_enabled).length || 0}
            </div>
          </Card.Content>
        </Card>
        <Card>
          <Card.Header>
            <Card.Title>总连接容量</Card.Title>
          </Card.Header>
          <Card.Content>
            <div className="text-2xl font-bold text-purple-600 dark:text-purple-400">
              {servers?.data?.reduce((sum, s) => sum + (s.max_connections || 0), 0) || 0}
            </div>
          </Card.Content>
        </Card>
      </div>

      {/* 服务器列表 */}
      <Card>
        <Card.Header>
          <Card.Title>MCP服务器列表</Card.Title>
        </Card.Header>
        <Card.Content>
          <Table
            data={servers?.data || []}
            columns={columns}
            loading={loading}
            emptyState={{
              title: '暂无MCP服务器',
              description: '请点击"添加服务器"按钮来创建第一个MCP服务器配置',
            }}
          />
        </Card.Content>
      </Card>

      {/* 创建服务器模态框 */}
      <Modal
        isOpen={isCreateModalOpen}
        onClose={() => setIsCreateModalOpen(false)}
        title="添加MCP服务器"
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
            label="服务器名称"
            value={formData.name || ''}
            onChange={(value) => setFormData(prev => ({ ...prev, name: value }))}
            placeholder="请输入服务器名称"
            error={formErrors.name}
            required
          />
          <Input
            label="描述"
            value={formData.description || ''}
            onChange={(value) => setFormData(prev => ({ ...prev, description: value }))}
            placeholder="服务器功能描述（可选）"
          />
          <div className="grid grid-cols-2 gap-4">
            <Input
              label="主机地址"
              value={formData.host || ''}
              onChange={(value) => setFormData(prev => ({ ...prev, host: value }))}
              placeholder="localhost"
              error={formErrors.host}
              required
            />
            <Input
              label="端口号"
              value={formData.port?.toString() || ''}
              onChange={(value) => setFormData(prev => ({
                ...prev,
                port: value ? parseInt(value) : undefined
              }))}
              placeholder="8080"
              type="number"
              error={formErrors.port}
            />
          </div>
          <div className="grid grid-cols-2 gap-4">
            <Input
              label="协议"
              value={formData.protocol || ''}
              onChange={(value) => setFormData(prev => ({ ...prev, protocol: value }))}
              type="select"
              options={[
                { label: 'HTTP', value: 'http' },
                { label: 'HTTPS', value: 'https' },
              ]}
            />
            <Input
              label="认证Token"
              value={formData.auth_token || ''}
              onChange={(value) => setFormData(prev => ({ ...prev, auth_token: value }))}
              placeholder="服务器认证Token（可选）"
              type="password"
            />
          </div>
          <div className="grid grid-cols-3 gap-4">
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
            <Input
              label="最大连接数"
              value={formData.max_connections?.toString() || ''}
              onChange={(value) => setFormData(prev => ({
                ...prev,
                max_connections: value ? parseInt(value) : undefined
              }))}
              placeholder="10"
              type="number"
              error={formErrors.max_connections}
            />
            <Input
              label="重试次数"
              value={formData.retry_count?.toString() || ''}
              onChange={(value) => setFormData(prev => ({
                ...prev,
                retry_count: value ? parseInt(value) : undefined
              }))}
              placeholder="3"
              type="number"
              error={formErrors.retry_count}
            />
          </div>
        </div>
      </Modal>

      {/* 编辑服务器模态框 */}
      <Modal
        isOpen={isEditModalOpen}
        onClose={() => setIsEditModalOpen(false)}
        title="编辑MCP服务器"
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
            label="服务器名称"
            value={formData.name || ''}
            onChange={(value) => setFormData(prev => ({ ...prev, name: value }))}
            placeholder="请输入服务器名称"
            error={formErrors.name}
            required
          />
          <Input
            label="描述"
            value={formData.description || ''}
            onChange={(value) => setFormData(prev => ({ ...prev, description: value }))}
            placeholder="服务器功能描述（可选）"
          />
          <div className="grid grid-cols-2 gap-4">
            <Input
              label="主机地址"
              value={formData.host || ''}
              onChange={(value) => setFormData(prev => ({ ...prev, host: value }))}
              placeholder="localhost"
              error={formErrors.host}
              required
            />
            <Input
              label="端口号"
              value={formData.port?.toString() || ''}
              onChange={(value) => setFormData(prev => ({
                ...prev,
                port: value ? parseInt(value) : undefined
              }))}
              placeholder="8080"
              type="number"
              error={formErrors.port}
            />
          </div>
          <div className="grid grid-cols-2 gap-4">
            <Input
              label="协议"
              value={formData.protocol || ''}
              onChange={(value) => setFormData(prev => ({ ...prev, protocol: value }))}
              type="select"
              options={[
                { label: 'HTTP', value: 'http' },
                { label: 'HTTPS', value: 'https' },
              ]}
            />
            <Input
              label="认证Token"
              value={formData.auth_token || ''}
              onChange={(value) => setFormData(prev => ({ ...prev, auth_token: value }))}
              placeholder="服务器认证Token（可选）"
              type="password"
            />
          </div>
          <div className="grid grid-cols-3 gap-4">
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
            <Input
              label="最大连接数"
              value={formData.max_connections?.toString() || ''}
              onChange={(value) => setFormData(prev => ({
                ...prev,
                max_connections: value ? parseInt(value) : undefined
              }))}
              placeholder="10"
              type="number"
              error={formErrors.max_connections}
            />
            <Input
              label="重试次数"
              value={formData.retry_count?.toString() || ''}
              onChange={(value) => setFormData(prev => ({
                ...prev,
                retry_count: value ? parseInt(value) : undefined
              }))}
              placeholder="3"
              type="number"
              error={formErrors.retry_count}
            />
          </div>
        </div>
      </Modal>

      {/* 删除确认模态框 */}
      <Modal
        isOpen={isDeleteModalOpen}
        onClose={() => setIsDeleteModalOpen(false)}
        title="删除MCP服务器"
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
              确定要删除MCP服务器 <span className="font-medium text-gray-900 dark:text-white">{selectedServer?.name}</span> 吗？
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