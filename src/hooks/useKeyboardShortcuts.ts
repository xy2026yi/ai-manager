import { useEffect, useCallback } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';

export interface Shortcut {
  /**
   * 快捷键组合 (e.g., 'Ctrl+S', 'Alt+F4')
   */
  key: string;
  
  /**
   * 回调函数
   */
  action: () => void;
  
  /**
   * 快捷键描述
   */
  description: string;
  
  /**
   * 是否禁用
   */
  disabled?: boolean;
  
  /**
   * 是否需要按下修饰键
   */
  requiresModKey?: boolean;
  
  /**
   * 优先级（数字越大优先级越高）
   */
  priority?: number;
}

export interface UseKeyboardShortcutsOptions {
  /**
   * 快捷键列表
   */
  shortcuts: Shortcut[];
  
  /**
   * 是否启用
   */
  enabled?: boolean;
  
  /**
   * 全局快捷键（不区分输入焦点）
   */
  global?: boolean;
  
  /**
   * 快捷键触发的回调
   */
  onShortcutTriggered?: (shortcut: Shortcut, event: KeyboardEvent) => void;
}

/**
 * 键盘快捷键Hook
 */
export const useKeyboardShortcuts = (options: UseKeyboardShortcutsOptions) => {
  const navigate = useNavigate();
  const location = useLocation();

  // 默认应用快捷键
  const defaultShortcuts: Shortcut[] = [
    {
      key: 'Ctrl+S',
      action: () => {
        console.log('保存快捷键触发');
      },
      description: '保存',
      priority: 1
    },
    {
      key: 'Ctrl+Z',
      action: () => {
        window.history.back();
      },
      description: '撤销/返回',
      priority: 1
    },
    {
      key: 'Ctrl+Y',
      action: () => {
        window.history.forward();
      },
      description: '重做/前进',
      priority: 1
    },
    {
      key: 'Ctrl+R',
      action: () => {
        window.location.reload();
      },
      description: '刷新页面',
      priority: 1
    },
    {
      key: 'Ctrl+F',
      action: () => {
        // 如果有搜索框，聚焦搜索框
        const searchInput = document.querySelector('input[type="search"]') as HTMLInputElement;
        if (searchInput) {
          searchInput.focus();
        }
      },
      description: '搜索',
      priority: 1
    },
    {
      key: 'Escape',
      action: () => {
        // 关闭模态框或返回
        const modal = document.querySelector('[role="dialog"]') as HTMLDialogElement;
        if (modal?.open) {
          modal.close();
        } else {
          window.history.back();
        }
      },
      description: '关闭/返回',
      priority: 2
    },
    {
      key: 'F1',
      action: () => {
        // 显示帮助
        console.log('显示帮助信息');
      },
      description: '帮助',
      priority: 3
    },
    {
      key: 'F11',
      action: () => {
        if (!document.fullscreenElement) {
          document.documentElement.requestFullscreen();
        } else {
          document.exitFullscreen();
        }
      },
      description: '全屏切换',
      priority: 2
    },
    // 页面导航快捷键
    {
      key: 'Alt+H',
      action: () => navigate('/dashboard'),
      description: '返回首页',
      priority: 1
    },
    {
      key: 'Alt+1',
      action: () => navigate('/dashboard'),
      description: '仪表盘',
      priority: 1
    },
    {
      key: 'Alt+2',
      action: () => navigate('/claude-providers'),
      description: 'Claude供应商',
      priority: 1
    },
    {
      key: 'Alt+3',
      action: () => navigate('/codex-providers'),
      description: 'Codex供应商',
      priority: 1
    },
    {
      key: 'Alt+4',
      action: () => navigate('/agent-guides'),
      description: 'Agent指导',
      priority: 1
    },
    {
      key: 'Alt+5',
      action: () => navigate('/mcp-servers'),
      description: 'MCP服务器',
      priority: 1
    },
    {
      key: 'Alt+6',
      action: () => navigate('/common-configs'),
      description: '通用配置',
      priority: 1
    },
    {
      key: 'Alt+7',
      action: () => navigate('/settings'),
      description: '设置',
      priority: 1
    },
    // 功能快捷键
    {
      key: 'Ctrl+N',
      action: () => {
        console.log('新建快捷键触发');
      },
      description: '新建',
      priority: 1
    },
    {
      key: 'Ctrl+E',
      action: () => {
        console.log('编辑快捷键触发');
      },
      description: '编辑',
      priority: 1
    },
    {
      key: 'Ctrl+D',
      action: () => {
        console.log('删除快捷键触发');
      },
      description: '删除',
      priority: 1
    },
    {
      key: 'Ctrl+A',
      action: () => {
        // 全选当前聚焦的输入框内容
        const activeElement = document.activeElement as HTMLInputElement;
        if (activeElement && activeElement.select) {
          activeElement.select();
        }
      },
      description: '全选',
      priority: 1
    },
    {
      key: 'Ctrl+C',
      action: () => {
        // 复制选中的内容
        document.execCommand('copy');
      },
      description: '复制',
      priority: 1
    },
    {
      key: 'Ctrl+V',
      action: () => {
        // 粘贴内容
        document.execCommand('paste');
      },
      description: '粘贴',
      priority: 1
    },
    {
      key: 'Ctrl+X',
      action: () => {
        // 剪切内容
        document.execCommand('cut');
      },
      description: '剪切',
      priority: 1
    }
  ];

  // 合并默认快捷键和用户自定义快捷键
  const allShortcuts = [...defaultShortcuts, ...options.shortcuts];

  // 解析快捷键字符串
  const parseShortcut = (shortcut: string): { ctrl: boolean; alt: boolean; shift: boolean; key: string } => {
    const parts = shortcut.toLowerCase().split('+');
    return {
      ctrl: parts.includes('ctrl') || parts.includes('control'),
      alt: parts.includes('alt'),
      shift: parts.includes('shift'),
      key: parts.find(part => !['ctrl', 'alt', 'shift', 'control'].includes(part)) || ''
    };
  };

  // 检查快捷键是否匹配
  const matchesShortcut = (event: KeyboardEvent, shortcut: Shortcut): boolean => {
    const parsed = parseShortcut(shortcut.key);
    
    return (
      event.ctrlKey === parsed.ctrl &&
      event.altKey === parsed.alt &&
      event.shiftKey === parsed.shift &&
      event.key.toLowerCase() === parsed.key.toLowerCase()
    );
  };

  // 键盘事件处理函数
  const handleKeyDown = useCallback((event: KeyboardEvent) => {
    // 检查是否应该处理键盘事件
    if (!options.enabled || options.global === false && event.target instanceof HTMLInputElement) {
      return;
    }

    // 在输入框中，除非是全局快捷键，否则不处理
    if (event.target instanceof HTMLInputElement || 
        event.target instanceof HTMLTextAreaElement ||
        event.target instanceof HTMLSelectElement) {
      // 只处理全局快捷键和Ctrl组合键
      const isGlobalShortcut = allShortcuts.some(shortcut => 
        !shortcut.disabled && (
          event.ctrlKey || 
          event.altKey ||
          shortcut.key === 'Escape' ||
          shortcut.key.startsWith('F')
        )
      );
      
      if (!isGlobalShortcut) {
        return;
      }
    }

    // 按优先级排序的快捷键
    const sortedShortcuts = [...allShortcuts].sort((a, b) => (b.priority || 0) - (a.priority || 0));

    // 查找匹配的快捷键
    for (const shortcut of sortedShortcuts) {
      if (shortcut.disabled) {
        continue;
      }

      if (matchesShortcut(event, shortcut)) {
        event.preventDefault();
        event.stopPropagation();
        
        try {
          shortcut.action();
          
          if (options.onShortcutTriggered) {
            options.onShortcutTriggered(shortcut, event);
          }
        } catch (error) {
          console.error('快捷键执行失败:', error);
        }
        
        break;
      }
    }
  }, [allShortcuts, options.enabled, options.global, options.onShortcutTriggered]);

  useEffect(() => {
    // 添加键盘事件监听器
    document.addEventListener('keydown', handleKeyDown);
    
    // 清理函数
    return () => {
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [handleKeyDown]);

  // 返回快捷键列表和帮助信息
  const getActiveShortcuts = useCallback(() => {
    return allShortcuts.filter(shortcut => !shortcut.disabled);
  }, [allShortcuts]);

  const getShortcutHelp = useCallback(() => {
    return getActiveShortcuts().map(shortcut => ({
      key: shortcut.key,
      description: shortcut.description
    }));
  }, [getActiveShortcuts]);

  return {
    shortcuts: getActiveShortcuts(),
    shortcutHelp: getShortcutHelp(),
    addShortcut: (newShortcut: Shortcut) => {
      options.shortcuts.push(newShortcut);
    },
    removeShortcut: (key: string) => {
      const index = options.shortcuts.findIndex(s => s.key === key);
      if (index > -1) {
        options.shortcuts.splice(index, 1);
      }
    },
    toggleShortcut: (key: string, disabled?: boolean) => {
      const shortcut = options.shortcuts.find(s => s.key === key);
      if (shortcut) {
        shortcut.disabled = disabled !== undefined ? disabled : !shortcut.disabled;
      }
    }
  };
};

/**
 * 快捷键提示组件
 */
export const KeyboardShortcutsHelp: React.FC<{
  /**
   * 是否显示
   */
  show?: boolean;
  
  /**
   * 关闭回调
   */
  onClose?: () => void;
}> = ({ show = false, onClose }) => {
  const { shortcutHelp } = useKeyboardShortcuts({
    shortcuts: [],
    enabled: true
  });

  if (!show) {
    return null;
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50 backdrop-blur-sm">
      <div className="bg-white rounded-lg shadow-xl max-w-md w-full mx-4 animate-fade-in">
        <div className="px-6 py-4 border-b border-gray-200">
          <div className="flex items-center justify-between">
            <h3 className="text-lg font-medium text-gray-900">键盘快捷键</h3>
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-gray-500 transition-colors"
            >
              <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
        </div>
        
        <div className="px-6 py-4 max-h-96 overflow-y-auto">
          <div className="space-y-3">
            {shortcutHelp.map((shortcut, index) => (
              <div key={index} className="flex items-center justify-between py-2 border-b border-gray-100 last:border-b-0">
                <div className="flex items-center space-x-3">
                  <kbd className="px-2 py-1 text-xs font-mono bg-gray-100 border border-gray-300 rounded">
                    {shortcut.key}
                  </kbd>
                  <span className="text-sm text-gray-600">{shortcut.description}</span>
                </div>
              </div>
            ))}
          </div>
        </div>
        
        <div className="px-6 py-4 bg-gray-50 border-t border-gray-200">
          <p className="text-xs text-gray-500">
            提示：按 <kbd className="px-1 py-0.5 text-xs bg-white border border-gray-300 rounded">?</kbd> 
            键或 <kbd className="px-1 py-0.5 text-xs bg-white border border-gray-300 rounded">Alt+H</kbd> 
            打开此帮助窗口
          </p>
        </div>
      </div>
    </div>
  );
};

export default useKeyboardShortcuts;