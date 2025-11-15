// 通用表格组件
//
// 提供统一的表格样式和功能，包括排序、分页、搜索等

import React, { useState, useMemo } from 'react';
import {
  ChevronLeftIcon,
  ChevronRightIcon,
  ChevronUpDownIcon,
  MagnifyingGlassIcon,
} from '@heroicons/react/24/outline';
import type { TableColumn, ActionItem } from '../types';

// 表格属性接口
export interface TableProps<T> {
  data: T[];
  columns: TableColumn<T>[];
  loading?: boolean;
  pagination?: {
    current: number;
    pageSize: number;
    total: number;
    onChange: (page: number, pageSize: number) => void;
  };
  search?: {
    placeholder?: string;
    value?: string;
    onChange: (value: string) => void;
  };
  actions?: (record: T) => ActionItem[];
  emptyMessage?: string;
  className?: string;
}

// 排序方向类型
type SortDirection = 'asc' | 'desc';

// 排序状态接口
interface SortState {
  key: string;
  direction: SortDirection;
}

// 表格组件
export function Table<T extends Record<string, any>>({
  data,
  columns,
  loading = false,
  pagination,
  search,
  actions,
  emptyMessage = '暂无数据',
  className = '',
}: TableProps<T>) {
  const [sortState, setSortState] = useState<SortState | null>(null);

  // 处理排序
  const handleSort = (key: string) => {
    let newDirection: SortDirection = 'asc';
    if (sortState?.key === key) {
      newDirection = sortState.direction === 'asc' ? 'desc' : 'asc';
    }
    setSortState({ key, direction: newDirection });
  };

  // 排序和过滤数据
  const sortedData = useMemo(() => {
    if (!sortState) return data;

    return [...data].sort((a, b) => {
      const aValue = a[sortState.key];
      const bValue = b[sortState.key];

      if (aValue === null || aValue === undefined) return 1;
      if (bValue === null || bValue === undefined) return -1;

      let comparison = 0;
      if (typeof aValue === 'string' && typeof bValue === 'string') {
        comparison = aValue.localeCompare(bValue);
      } else {
        comparison = aValue > bValue ? 1 : aValue < bValue ? -1 : 0;
      }

      return sortState.direction === 'desc' ? -comparison : comparison;
    });
  }, [data, sortState]);

  // 渲染表头
  const renderHeader = () => (
    <thead className="bg-gray-50">
      <tr>
        {columns.map((column) => (
          <th
            key={String(column.key)}
            className={`px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider ${
              column.sortable ? 'cursor-pointer hover:bg-gray-100' : ''
            } ${column.width || ''}`}
            onClick={() => column.sortable && handleSort(String(column.key))}
          >
            <div className="flex items-center space-x-1">
              {column.title}
              {column.sortable && (
                <span>
                  {sortState?.key === String(column.key) ? (
                    sortState.direction === 'asc' ? (
                      <ChevronUpDownIcon className="h-4 w-4" />
                    ) : (
                    <ChevronUpDownIcon className="h-4 w-4 rotate-180" />
                    )
                  ) : (
                    <ChevronUpDownIcon className="h-4 w-4 opacity-30" />
                  )}
                </span>
              )}
            </div>
          </th>
        ))}
        {actions && <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
          操作
        </th>}
      </tr>
    </thead>
  );

  // 渲染表格内容
  const renderBody = () => {
    if (loading) {
      return (
        <tbody>
          <tr>
            <td colSpan={columns.length + (actions ? 1 : 0)} className="px-6 py-12 text-center">
              <div className="flex justify-center">
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
              </div>
              <p className="mt-2 text-sm text-gray-500">加载中...</p>
            </td>
          </tr>
        </tbody>
      );
    }

    if (sortedData.length === 0) {
      return (
        <tbody>
          <tr>
            <td colSpan={columns.length + (actions ? 1 : 0)} className="px-6 py-12 text-center">
              <p className="text-gray-500">{emptyMessage}</p>
            </td>
          </tr>
        </tbody>
      );
    }

    return (
      <tbody className="bg-white divide-y divide-gray-200">
        {sortedData.map((record, index) => (
          <tr key={index} className="hover:bg-gray-50">
            {columns.map((column) => (
              <td
                key={String(column.key)}
                className={`px-6 py-4 whitespace-nowrap text-sm text-gray-900 ${column.width || ''}`}
              >
                {column.render ? (
                  column.render(record[column.key], record)
                ) : (
                  String(record[column.key] || '')
                )}
              </td>
            ))}
            {actions && (
              <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                <div className="flex justify-end space-x-2">
                  {actions(record).map((action, actionIndex) => (
                    <button
                      key={actionIndex}
                      onClick={action.onClick}
                      className={`text-blue-600 hover:text-blue-900 ${
                        action.variant === 'danger'
                          ? 'text-red-600 hover:text-red-900'
                          : action.variant === 'secondary'
                          ? 'text-gray-600 hover:text-gray-900'
                          : 'text-blue-600 hover:text-blue-900'
                      }`}
                    >
                      {action.icon && <span className="mr-1">{action.icon}</span>}
                      {action.label}
                    </button>
                  ))}
                </div>
              </td>
            )}
          </tr>
        ))}
      </tbody>
    );
  };

  // 渲染分页
  const renderPagination = () => {
    if (!pagination) return null;

    const { current, pageSize, total } = pagination;
    const totalPages = Math.ceil(total / pageSize);
    const startItem = (current - 1) * pageSize + 1;
    const endItem = Math.min(current * pageSize, total);

    return (
      <div className="bg-white px-4 py-3 flex items-center justify-between border-t border-gray-200 sm:px-6">
        <div className="flex flex-1 justify-between sm:hidden">
          <button
            onClick={() => pagination.onChange(Math.max(1, current - 1), pageSize)}
            disabled={current === 1}
            className="relative inline-flex items-center rounded-md bg-white px-2 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50 disabled:opacity-50"
          >
            上一页
          </button>
          <button
            onClick={() => pagination.onChange(Math.min(totalPages, current + 1), pageSize)}
            disabled={current === totalPages}
            className="relative ml-3 inline-flex items-center rounded-md bg-white px-2 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50 disabled:opacity-50"
          >
            下一页
          </button>
        </div>
        <div className="hidden sm:flex sm:flex-1 sm:items-center sm:justify-between">
          <div>
            <p className="text-sm text-gray-700">
              显示第 <span className="font-medium">{startItem}</span> 到{' '}
              <span className="font-medium">{endItem}</span> 项，共{' '}
              <span className="font-medium">{total}</span> 项
            </p>
          </div>
          <div>
            <nav
              className="isolate inline-flex -space-x-px rounded-md shadow-sm"
              aria-label="Pagination"
            >
              <button
                onClick={() => pagination.onChange(Math.max(1, current - 1), pageSize)}
                disabled={current === 1}
                className="relative inline-flex items-center rounded-l-md px-2 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50 focus:z-20 focus:outline-offset-0 disabled:opacity-50"
              >
                <span className="sr-only">上一页</span>
                <ChevronLeftIcon className="h-5 w-5" aria-hidden="true" />
              </button>

              {/* 页码按钮 */}
              {Array.from({ length: Math.min(5, totalPages) }, (_, i) => {
                let pageNum;
                if (totalPages <= 5) {
                  pageNum = i + 1;
                } else if (current <= 3) {
                  pageNum = i + 1;
                } else if (current >= totalPages - 2) {
                  pageNum = totalPages - 4 + i;
                } else {
                  pageNum = current - 2 + i;
                }

                return (
                  <button
                    key={pageNum}
                    onClick={() => pagination.onChange(pageNum, pageSize)}
                    className={`relative inline-flex items-center px-4 py-2 text-sm font-medium ${
                      current === pageNum
                        ? 'z-10 bg-blue-50 text-blue-600 focus:outline-offset-0'
                        : 'text-gray-700 hover:bg-gray-50 focus:z-20 focus:outline-offset-0'
                    } ${i === 0 ? 'rounded-l-md' : ''} ${
                      i === 4 ? 'rounded-r-md' : ''
                    }`}
                  >
                    {pageNum}
                  </button>
                );
              })}

              <button
                onClick={() => pagination.onChange(Math.min(totalPages, current + 1), pageSize)}
                disabled={current === totalPages}
                className="relative inline-flex items-center rounded-r-md px-2 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50 focus:z-20 focus:outline-offset-0 disabled:opacity-50"
              >
                <span className="sr-only">下一页</span>
                <ChevronRightIcon className="h-5 w-5" aria-hidden="true" />
              </button>
            </nav>
          </div>
        </div>
      </div>
    );
  };

  return (
    <div className={`overflow-hidden shadow ring-1 ring-black ring-opacity-5 md:rounded-lg ${className}`}>
      {/* 搜索栏 */}
      {search && (
        <div className="bg-white px-4 py-3 border-b border-gray-200 sm:px-6">
          <div className="flex items-center">
            <div className="relative flex-1">
              <div className="pointer-events-none absolute inset-y-0 left-0 pl-3 flex items-center">
                <MagnifyingGlassIcon className="h-5 w-5 text-gray-400" />
              </div>
              <input
                type="text"
                className="block w-full rounded-lg border-0 py-1.5 pl-10 text-gray-900 placeholder-gray-500 focus:ring-0 sm:text-sm sm:leading-6"
                placeholder={search.placeholder || '搜索...'}
                value={search.value || ''}
                onChange={(e) => search.onChange(e.target.value)}
              />
            </div>
          </div>
        </div>
      )}

      {/* 表格 */}
      <div className="overflow-x-auto">
        <table className="min-w-full divide-y divide-gray-200">
          {renderHeader()}
          {renderBody()}
        </table>
      </div>

      {/* 分页 */}
      {renderPagination()}
    </div>
  );
}

// 导出类型
export type { TableProps };