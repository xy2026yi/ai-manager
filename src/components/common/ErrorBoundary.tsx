import React, { Component, ErrorInfo, ReactNode } from 'react';

export interface ErrorBoundaryProps {
  /**
   * 子组件
   */
  children: ReactNode;
  
  /**
   * 错误时显示的回调函数
   */
  fallback?: React.ComponentType<ErrorFallbackProps>;
  
  /**
   * 错误处理回调
   */
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
  
  /**
   * 是否显示错误详情（开发环境）
   */
  showErrorDetails?: boolean;
}

export interface ErrorFallbackProps {
  /**
   * 错误对象
   */
  error: Error;
  
  /**
   * 错误信息
   */
  errorInfo?: ErrorInfo;
  
  /**
   * 重试函数
   */
  resetError: () => void;
  
  /**
   * 是否显示错误详情
   */
  showErrorDetails?: boolean;
}

/**
 * 默认错误回退组件
 */
const DefaultErrorFallback: React.FC<ErrorFallbackProps> = ({
  error,
  errorInfo,
  resetError,
  showErrorDetails = false
}) => {
  const getErrorMessage = (error: Error): string => {
    // 根据错误类型提供用户友好的错误信息
    if (error.name === 'TypeError' && error.message.includes('fetch')) {
      return '网络连接失败，请检查网络设置后重试';
    }
    
    if (error.name === 'ChunkLoadError' || error.message.includes('Loading chunk')) {
      return '应用加载失败，请刷新页面重试';
    }
    
    if (error.message.includes('401') || error.message.includes('403')) {
      return '权限不足，请重新登录后重试';
    }
    
    if (error.message.includes('404')) {
      return '请求的资源不存在';
    }
    
    if (error.message.includes('500') || error.message.includes('502') || error.message.includes('503')) {
      return '服务器暂时不可用，请稍后重试';
    }
    
    // 默认错误信息
    return error.message || '发生了未知错误';
  };

  const getErrorIcon = () => {
    if (error.name === 'NetworkError' || error.message.includes('fetch')) {
      return (
        <svg className="w-6 h-6 text-warning-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.82 16.5c-.77.833.192 2.5 1.732 2.5z" />
        </svg>
      );
    }
    
    return (
      <svg className="w-6 h-6 text-error-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.82 16.5c-.77.833.192 2.5 1.732 2.5z" />
      </svg>
    );
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50 px-4">
      <div className="max-w-md w-full bg-white rounded-lg shadow-lg p-6 animate-fade-in">
        <div className="flex items-center justify-center w-12 h-12 mx-auto bg-gray-100 rounded-full mb-4">
          {getErrorIcon()}
        </div>
        
        <div className="text-center">
          <h3 className="text-lg font-medium text-gray-900 mb-2">
            哎呀，出现了问题
          </h3>
          
          <p className="text-sm text-gray-600 mb-6">
            {getErrorMessage(error)}
          </p>
          
          <div className="space-y-3">
            <button
              onClick={resetError}
              className="w-full flex justify-center items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-primary-600 hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500 transition-colors"
            >
              <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
              重新加载
            </button>
            
            <button
              onClick={() => window.location.href = '/'}
              className="w-full flex justify-center items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500 transition-colors"
            >
              <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
              </svg>
              返回首页
            </button>
          </div>
          
          {showErrorDetails && (
            <details className="mt-6 text-left">
              <summary className="text-xs text-gray-500 cursor-pointer hover:text-gray-700">
                查看错误详情
              </summary>
              <div className="mt-2 p-3 bg-gray-50 rounded text-xs font-mono text-gray-700">
                <div className="mb-2">
                  <strong>错误类型:</strong> {error.name}
                </div>
                <div className="mb-2">
                  <strong>错误信息:</strong> {error.message}
                </div>
                {error.stack && (
                  <div>
                    <strong>堆栈跟踪:</strong>
                    <pre className="whitespace-pre-wrap break-all">{error.stack}</pre>
                  </div>
                )}
                {errorInfo && (
                  <div className="mt-2 pt-2 border-t border-gray-200">
                    <strong>组件堆栈:</strong>
                    <pre className="whitespace-pre-wrap break-all">{errorInfo.componentStack}</pre>
                  </div>
                )}
              </div>
            </details>
          )}
        </div>
      </div>
    </div>
  );
};

/**
 * 错误边界组件
 */
export class ErrorBoundary extends Component<ErrorBoundaryProps, { hasError: boolean; error: Error | null; errorInfo: ErrorInfo | null }> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: null, errorInfo: null };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundary['state'] {
    return { hasError: true, error, errorInfo: null };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    // 记录错误到控制台
    console.error('ErrorBoundary caught an error:', error, errorInfo);
    
    // 记录错误到错误报告服务（如果需要）
    if (this.props.onError) {
      this.props.onError(error, errorInfo);
    }
    
    // 更新状态
    this.setState({
      error,
      errorInfo
    });
  }

  resetError = () => {
    this.setState({ hasError: false, error: null, errorInfo: null });
  };

  render() {
    if (this.state.hasError && this.state.error) {
      const FallbackComponent = this.props.fallback || DefaultErrorFallback;
      
      return (
        <FallbackComponent
          error={this.state.error}
          errorInfo={this.state.errorInfo}
          resetError={this.resetError}
          showErrorDetails={this.props.showErrorDetails || process.env.NODE_ENV === 'development'}
        />
      );
    }

    return this.props.children;
  }
}

/**
 * 错误提示组件，用于显示操作错误信息
 */
export interface ErrorAlertProps {
  /**
   * 错误消息
   */
  message: string;
  
  /**
   * 错误类型
   */
  type?: 'error' | 'warning' | 'info';
  
  /**
   * 是否可关闭
   */
  dismissible?: boolean;
  
  /**
   * 操作按钮
   */
  actions?: React.ReactNode;
  
  /**
   * 关闭回调
   */
  onDismiss?: () => void;
  
  /**
   * 是否显示重试按钮
   */
  showRetry?: boolean;
  
  /**
   * 重试回调
   */
  onRetry?: () => void;
}

/**
 * 错误提示组件
 */
export const ErrorAlert: React.FC<ErrorAlertProps> = ({
  message,
  type = 'error',
  dismissible = true,
  actions,
  onDismiss,
  showRetry = false,
  onRetry
}) => {
  const getIcon = () => {
    switch (type) {
      case 'warning':
        return (
          <svg className="w-5 h-5 text-yellow-400" fill="currentColor" viewBox="0 0 20 20">
            <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 3.48l-5.58-9.92zM11.43 8.536a1 1 0 10-1.414 0L8.014 9.144a1 1 0 10-1.414 0l3.414-3.414z" clipRule="evenodd" />
          </svg>
        );
      case 'info':
        return (
          <svg className="w-5 h-5 text-blue-400" fill="currentColor" viewBox="0 0 20 20">
            <path fillRule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 001-1v-3a1 1 0 00-1-1H9z" clipRule="evenodd" />
          </svg>
        );
      default:
        return (
          <svg className="w-5 h-5 text-red-400" fill="currentColor" viewBox="0 0 20 20">
            <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 01-1.414 0l2 2a1 1 0 001.414 0l4-4a1 1 0 001.414-1.414l-4-4a1 1 0 00-1.414 0z" clipRule="evenodd" />
          </svg>
        );
    }
  };

  const getBackgroundClasses = () => {
    switch (type) {
      case 'warning':
        return 'bg-yellow-50 border-yellow-200 text-yellow-800';
      case 'info':
        return 'bg-blue-50 border-blue-200 text-blue-800';
      default:
        return 'bg-red-50 border-red-200 text-red-800';
    }
  };

  return (
    <div className={`rounded-md border p-4 animate-fade-in ${getBackgroundClasses()}`}>
      <div className="flex">
        <div className="flex-shrink-0">
          {getIcon()}
        </div>
        <div className="ml-3 flex-1">
          <p className="text-sm font-medium">{message}</p>
          {(actions || showRetry) && (
            <div className="mt-3 flex items-center space-x-3">
              {showRetry && onRetry && (
                <button
                  onClick={onRetry}
                  className="text-sm underline hover:no-underline"
                >
                  重试
                </button>
              )}
              {actions}
            </div>
          )}
        </div>
        {dismissible && onDismiss && (
          <button
            onClick={onDismiss}
            className="ml-auto -mx-1.5 -my-1.5 rounded-lg p-1 hover:bg-gray-100"
          >
            <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
              <path fillRule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L10 10.414l4.293 4.293a1 1 0 01-1.414 0L10 11.814l-4.293 4.293a1 1 0 01-1.414-1.414L10 13.414l-4.293-4.293a1 1 0 010-1.414z" clipRule="evenodd" />
            </svg>
          </button>
        )}
      </div>
    </div>
  );
};

export default ErrorBoundary;