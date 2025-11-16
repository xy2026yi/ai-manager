import React, { useState, useEffect } from 'react';
import { Plus, Edit, Trash2, Eye, DocumentText, Tag, Clock } from '@heroicons/react/solid';
import { Card } from '../components/common/Card';
import { Button } from '../components/common/Button';
import { Modal } from '../components/common/Modal';
import { Input } from '../components/common/Input';
import { Table } from '../components/common/Table';
import {
  agentGuidesAtom,
  selectedAgentGuideAtom,
  loadingAtom,
  errorAtom,
  refreshTriggerAtom
} from '../stores';
import { useAtom } from 'jotai';
import { AgentGuide, AgentGuideService } from '../services/api';

/**
 * Agent指导文件管理页面
 * 提供Agent指导文件的增删改查功能
 */
export default function AgentGuides() {
  // 状态管理
  const [guides, setGuides] = useAtom(agentGuidesAtom);
  const [selectedGuide, setSelectedGuide] = useAtom(selectedAgentGuideAtom);
  const [loading, setLoading] = useAtom(loadingAtom);
  const [error, setError] = useAtom(errorAtom);
  const [refreshTrigger] = useAtom(refreshTriggerAtom);

  // 本地状态
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [isEditModalOpen, setIsEditModalOpen] = useState(false);
  const [isDeleteModalOpen, setIsDeleteModalOpen] = useState(false);
  const [isPreviewModalOpen, setIsPreviewModalOpen] = useState(false);
  const [formData, setFormData] = useState<Partial<AgentGuide>>({});
  const [formErrors, setFormErrors] = useState<Record<string, string>>({});

  const guideService = new AgentGuideService();

  // 加载指导文件列表
  useEffect(() => {
    loadGuides();
  }, [refreshTrigger]);

  const loadGuides = async () => {
    try {
      setLoading(true);
      setError(null);
      const result = await guideService.list({ limit: 100 });
      setGuides(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : '加载指导文件列表失败');
    } finally {
      setLoading(false);
    }
  };

  // 验证表单数据
  const validateForm = (data: Partial<AgentGuide>): Record<string, string> => {
    const errors: Record<string, string> = {};

    if (!data.name?.trim()) {
      errors.name = '指导文件名称不能为空';
    }

    if (!data.content?.trim()) {
      errors.content = '指导内容不能为空';
    }

    if (!data.category?.trim()) {
      errors.category = '分类不能为空';
    }

    if (data.tags && typeof data.tags === 'string') {
      try {
        JSON.parse(data.tags);
      } catch {
        errors.tags = '标签格式不正确，请输入有效的JSON数组';
      }
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
      tags: JSON.stringify([]),
      is_enabled: true,
    });
    setIsCreateModalOpen(true);
  };

  // 创建指导文件
  const handleCreateSubmit = async () => {
    const errors = validateForm(formData);
    if (Object.keys(errors).length > 0) {
      setFormErrors(errors);
      return;
    }

    try {
      setLoading(true);
      const submitData = {
        ...formData,
        tags: typeof formData.tags === 'string'
          ? JSON.parse(formData.tags)
          : formData.tags,
      };
      await guideService.create(submitData as Omit<AgentGuide, 'id' | 'created_at' | 'updated_at'>);
      setIsCreateModalOpen(false);
      resetForm();
      await loadGuides();
    } catch (err) {
      setError(err instanceof Error ? err.message : '创建指导文件失败');
    } finally {
      setLoading(false);
    }
  };

  // 打开编辑模态框
  const handleEdit = (guide: AgentGuide) => {
    setSelectedGuide(guide);
    setFormData({
      ...guide,
      tags: Array.isArray(guide.tags) ? JSON.stringify(guide.tags) : guide.tags,
    });
    setFormErrors({});
    setIsEditModalOpen(true);
  };

  // 更新指导文件
  const handleEditSubmit = async () => {
    if (!selectedGuide) return;

    const errors = validateForm(formData);
    if (Object.keys(errors).length > 0) {
      setFormErrors(errors);
      return;
    }

    try {
      setLoading(true);
      const submitData = {
        ...formData,
        tags: typeof formData.tags === 'string'
          ? JSON.parse(formData.tags)
          : formData.tags,
      };
      await guideService.update(selectedGuide.id, submitData as Partial<AgentGuide>);
      setIsEditModalOpen(false);
      setSelectedGuide(null);
      resetForm();
      await loadGuides();
    } catch (err) {
      setError(err instanceof Error ? err.message : '更新指导文件失败');
    } finally {
      setLoading(false);
    }
  };

  // 打开删除确认框
  const handleDelete = (guide: AgentGuide) => {
    setSelectedGuide(guide);
    setIsDeleteModalOpen(true);
  };

  // 删除指导文件
  const handleDeleteSubmit = async () => {
    if (!selectedGuide) return;

    try {
      setLoading(true);
      await guideService.deleteById(selectedGuide.id);
      setIsDeleteModalOpen(false);
      setSelectedGuide(null);
      await loadGuides();
    } catch (err) {
      setError(err instanceof Error ? err.message : '删除指导文件失败');
    } finally {
      setLoading(false);
    }
  };

  // 切换启用状态
  const handleToggleEnabled = async (guide: AgentGuide) => {
    try {
      setLoading(true);
      await guideService.update(guide.id, { is_enabled: !guide.is_enabled });
      await loadGuides();
    } catch (err) {
      setError(err instanceof Error ? err.message : '切换启用状态失败');
    } finally {
      setLoading(false);
    }
  };

  // 预览指导文件
  const handlePreview = (guide: AgentGuide) => {
    setSelectedGuide(guide);
    setIsPreviewModalOpen(true);
  };

  // 格式化标签
  const formatTags = (tags: string | string[]) => {
    try {
      const parsedTags = Array.isArray(tags) ? tags : JSON.parse(tags || '[]');
      return parsedTags.slice(0, 3).map((tag: string) => (
        <span
          key={tag}
          className="inline-flex items-center px-2 py-1 rounded text-xs font-medium bg-indigo-100 text-indigo-800 dark:bg-indigo-900 dark:text-indigo-200 mr-1 mb-1"
        >
          {tag}
        </span>
      ));
    } catch {
      return null;
    }
  };

  // 截取内容预览
  const getContentPreview = (content: string, maxLength: number = 100) => {
    if (content.length <= maxLength) return content;
    return content.substring(0, maxLength) + '...';
  };

  // 表格列定义
  const columns = [
    {
      key: 'name',
      title: '指导文件名称',
      render: (guide: AgentGuide) => (
        <div className="flex items-center">
          <div className="flex-shrink-0 h-10 w-10 bg-indigo-100 rounded-full flex items-center justify-center">
            <DocumentText className="h-6 w-6 text-indigo-600" />
          </div>
          <div className="ml-4">
            <div className="text-sm font-medium text-gray-900 dark:text-white">
              {guide.name}
            </div>
            <div className="text-sm text-gray-500 dark:text-gray-400">
              {guide.category}
            </div>
          </div>
        </div>
      ),
    },
    {
      key: 'content',
      title: '内容预览',
      render: (guide: AgentGuide) => (
        <div className="max-w-md">
          <p className="text-sm text-gray-900 dark:text-white line-clamp-2">
            {getContentPreview(guide.content)}
          </p>
          {guide.content.length > 100 && (
            <Button
              variant="ghost"
              size="sm"
              onClick={() => handlePreview(guide)}
              className="mt-1 p-0 text-xs"
            >
              查看完整内容
            </Button>
          )}
        </div>
      ),
    },
    {
      key: 'tags',
      title: '标签',
      render: (guide: AgentGuide) => (
        <div className="flex flex-wrap">
          {formatTags(guide.tags)}
          {(() => {
            try {
              const parsedTags = Array.isArray(guide.tags) ? guide.tags : JSON.parse(guide.tags || '[]');
              if (parsedTags.length > 3) {
                return (
                  <span className="text-xs text-gray-500 dark:text-gray-400 ml-1">
                    +{parsedTags.length - 3}
                  </span>
                );
              }
            } catch {
              return null;
            }
          })()}
        </div>
      ),
    },
    {
      key: 'status',
      title: '状态',
      render: (guide: AgentGuide) => (
        <span
          className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
            guide.is_enabled
              ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
              : 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200'
          }`}
        >
          {guide.is_enabled ? '已启用' : '已禁用'}
        </span>
      ),
    },
    {
      key: 'updated_at',
      title: '更新时间',
      render: (guide: AgentGuide) => (
        <div className="flex items-center text-sm text-gray-500 dark:text-gray-400">
          <Clock className="h-4 w-4 mr-1" />
          {new Date(guide.updated_at).toLocaleDateString('zh-CN')}
        </div>
      ),
    },
    {
      key: 'actions',
      title: '操作',
      render: (guide: AgentGuide) => (
        <div className="flex items-center space-x-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => handlePreview(guide)}
            disabled={loading}
          >
            <Eye className="h-4 w-4" />
          </Button>
          <Button
            variant={guide.is_enabled ? "warning" : "success"}
            size="sm"
            onClick={() => handleToggleEnabled(guide)}
            disabled={loading}
          >
            {guide.is_enabled ? '禁用' : '启用'}
          </Button>
          <Button
            variant="primary"
            size="sm"
            onClick={() => handleEdit(guide)}
            disabled={loading}
          >
            <Edit className="h-4 w-4" />
          </Button>
          <Button
            variant="danger"
            size="sm"
            onClick={() => handleDelete(guide)}
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
            Agent指导文件管理
          </h1>
          <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
            管理AI Agent的指导文件，包括分类、标签和详细内容
          </p>
        </div>
        <Button
          variant="primary"
          onClick={handleCreate}
          disabled={loading}
          className="flex items-center space-x-2"
        >
          <Plus className="h-4 w-4" />
          <span>添加指导文件</span>
        </Button>
      </div>

      {/* 统计信息 */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <Card>
          <Card.Header>
            <Card.Title>总指导文件数</Card.Title>
          </Card.Header>
          <Card.Content>
            <div className="text-2xl font-bold text-gray-900 dark:text-white">
              {guides?.total || 0}
            </div>
          </Card.Content>
        </Card>
        <Card>
          <Card.Header>
            <Card.Title>已启用</Card.Title>
          </Card.Header>
          <Card.Content>
            <div className="text-2xl font-bold text-green-600 dark:text-green-400">
              {guides?.data?.filter(g => g.is_enabled).length || 0}
            </div>
          </Card.Content>
        </Card>
        <Card>
          <Card.Header>
            <Card.Title>分类数量</Card.Title>
          </Card.Header>
          <Card.Content>
            <div className="text-2xl font-bold text-indigo-600 dark:text-indigo-400">
              {new Set(guides?.data?.map(g => g.category) || []).size}
            </div>
          </Card.Content>
        </Card>
      </div>

      {/* 指导文件列表 */}
      <Card>
        <Card.Header>
          <Card.Title>指导文件列表</Card.Title>
        </Card.Header>
        <Card.Content>
          <Table
            data={guides?.data || []}
            columns={columns}
            loading={loading}
            emptyState={{
              title: '暂无指导文件',
              description: '请点击"添加指导文件"按钮来创建第一个Agent指导文件',
            }}
          />
        </Card.Content>
      </Card>

      {/* 创建指导文件模态框 */}
      <Modal
        isOpen={isCreateModalOpen}
        onClose={() => setIsCreateModalOpen(false)}
        title="添加Agent指导文件"
        size="lg"
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
            label="指导文件名称"
            value={formData.name || ''}
            onChange={(value) => setFormData(prev => ({ ...prev, name: value }))}
            placeholder="请输入指导文件名称"
            error={formErrors.name}
            required
          />
          <Input
            label="分类"
            value={formData.category || ''}
            onChange={(value) => setFormData(prev => ({ ...prev, category: value }))}
            placeholder="如：编程助手、写作助手等"
            error={formErrors.category}
            required
          />
          <Input
            label="标签"
            value={formData.tags || ''}
            onChange={(value) => setFormData(prev => ({ ...prev, tags: value }))}
            placeholder='["编程", "Python", "调试"]'
            error={formErrors.tags}
            help="JSON数组格式，如：[&quot;标签1&quot;, &quot;标签2&quot;]"
          />
          <div className="space-y-2">
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
              指导内容 <span className="text-red-500">*</span>
            </label>
            <textarea
              value={formData.content || ''}
              onChange={(e) => setFormData(prev => ({ ...prev, content: e.target.value }))}
              placeholder="请输入详细的指导内容和规则..."
              rows={8}
              className={`w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white ${
                formErrors.content ? 'border-red-500' : ''
              }`}
            />
            {formErrors.content && (
              <p className="text-sm text-red-500">{formErrors.content}</p>
            )}
          </div>
        </div>
      </Modal>

      {/* 编辑指导文件模态框 */}
      <Modal
        isOpen={isEditModalOpen}
        onClose={() => setIsEditModalOpen(false)}
        title="编辑Agent指导文件"
        size="lg"
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
            label="指导文件名称"
            value={formData.name || ''}
            onChange={(value) => setFormData(prev => ({ ...prev, name: value }))}
            placeholder="请输入指导文件名称"
            error={formErrors.name}
            required
          />
          <Input
            label="分类"
            value={formData.category || ''}
            onChange={(value) => setFormData(prev => ({ ...prev, category: value }))}
            placeholder="如：编程助手、写作助手等"
            error={formErrors.category}
            required
          />
          <Input
            label="标签"
            value={formData.tags || ''}
            onChange={(value) => setFormData(prev => ({ ...prev, tags: value }))}
            placeholder='["编程", "Python", "调试"]'
            error={formErrors.tags}
            help="JSON数组格式，如：[&quot;标签1&quot;, &quot;标签2&quot;]"
          />
          <div className="space-y-2">
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
              指导内容 <span className="text-red-500">*</span>
            </label>
            <textarea
              value={formData.content || ''}
              onChange={(e) => setFormData(prev => ({ ...prev, content: e.target.value }))}
              placeholder="请输入详细的指导内容和规则..."
              rows={8}
              className={`w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white ${
                formErrors.content ? 'border-red-500' : ''
              }`}
            />
            {formErrors.content && (
              <p className="text-sm text-red-500">{formErrors.content}</p>
            )}
          </div>
        </div>
      </Modal>

      {/* 预览模态框 */}
      <Modal
        isOpen={isPreviewModalOpen}
        onClose={() => setIsPreviewModalOpen(false)}
        title={selectedGuide?.name}
        size="lg"
        footer={
          <div className="flex justify-end">
            <Button
              variant="secondary"
              onClick={() => setIsPreviewModalOpen(false)}
            >
              关闭
            </Button>
          </div>
        }
      >
        {selectedGuide && (
          <div className="space-y-4">
            <div className="flex items-center space-x-4">
              <div>
                <h3 className="text-lg font-medium text-gray-900 dark:text-white">
                  {selectedGuide.name}
                </h3>
                <p className="text-sm text-gray-500 dark:text-gray-400">
                  分类：{selectedGuide.category}
                </p>
              </div>
            </div>
            <div className="flex flex-wrap gap-2">
              {formatTags(selectedGuide.tags)}
            </div>
            <div className="border-t border-gray-200 dark:border-gray-700 pt-4">
              <h4 className="text-sm font-medium text-gray-900 dark:text-white mb-2">
                指导内容
              </h4>
              <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
                <pre className="whitespace-pre-wrap text-sm text-gray-900 dark:text-white">
                  {selectedGuide.content}
                </pre>
              </div>
            </div>
          </div>
        )}
      </Modal>

      {/* 删除确认模态框 */}
      <Modal
        isOpen={isDeleteModalOpen}
        onClose={() => setIsDeleteModalOpen(false)}
        title="删除指导文件"
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
              确定要删除指导文件 <span className="font-medium text-gray-900 dark:text-white">{selectedGuide?.name}</span> 吗？
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