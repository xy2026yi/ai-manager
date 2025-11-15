import React from 'react';

export interface LoadingIndicatorProps {
  /**
   * 加载状态的类型
   */
  variant?: 'spinner' | 'dots' | 'pulse' | 'skeleton';
  
  /**
   * 加载指示器的大小
   */
  size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl';
  
  /**
   * 加载文本
   */
  text?: string;
  
  /**
   * 是否显示背景遮罩
   */
  overlay?: boolean;
  
  /**
   * 自定义颜色主题
   */
  color?: 'primary' | 'secondary' | 'success' | 'warning' | 'error';
  
  /**
   * 额外的CSS类名
   */
  className?: string;
  
  /**
   * 是否全屏显示
   */
  fullscreen?: boolean;
}

/**
 * 统一的加载指示器组件
 */
export const LoadingIndicator: React.FC<LoadingIndicatorProps> = ({
  variant = 'spinner',
  size = 'md',
  text,
  overlay = false,
  color = 'primary',
  className = '',
  fullscreen = false
}) => {
  // 尺寸映射
  const sizeClasses = {
    xs: {
      spinner: 'w-4 h-4',
      dots: 'space-x-1',
      pulse: 'w-4 h-4',
      skeleton: 'h-4'
    },
    sm: {
      spinner: 'w-6 h-6',
      dots: 'space-x-2',
      pulse: 'w-6 h-6',
      skeleton: 'h-6'
    },
    md: {
      spinner: 'w-8 h-8',
      dots: 'space-x-2',
      pulse: 'w-8 h-8',
      skeleton: 'h-8'
    },
    lg: {
      spinner: 'w-12 h-12',
      dots: 'space-x-3',
      pulse: 'w-12 h-12',
      skeleton: 'h-12'
    },
    xl: {
      spinner: 'w-16 h-16',
      dots: 'space-x-4',
      pulse: 'w-16 h-16',
      skeleton: 'h-16'
    }
  };

  // 颜色映射
  const colorClasses = {
    primary: {
      spinner: 'text-primary-600',
      dots: 'bg-primary-600',
      pulse: 'bg-primary-600',
      skeleton: 'bg-primary-200'
    },
    secondary: {
      spinner: 'text-secondary-600',
      dots: 'bg-secondary-600',
      pulse: 'bg-secondary-600',
      skeleton: 'bg-secondary-200'
    },
    success: {
      spinner: 'text-success-600',
      dots: 'bg-success-600',
      pulse: 'bg-success-600',
      skeleton: 'bg-success-200'
    },
    warning: {
      spinner: 'text-warning-600',
      dots: 'bg-warning-600',
      pulse: 'bg-warning-600',
      skeleton: 'bg-warning-200'
    },
    error: {
      spinner: 'text-error-600',
      dots: 'bg-error-600',
      pulse: 'bg-error-600',
      skeleton: 'bg-error-200'
    }
  };

  const currentSizeClasses = sizeClasses[size];
  const currentColorClasses = colorClasses[color];

  // 渲染不同类型的加载指示器
  const renderIndicator = () => {
    switch (variant) {
      case 'spinner':
        return (
          <div className={`${currentSizeClasses.spinner} ${currentColorClasses.spinner} animate-spin`}>
            <svg className="w-full h-full" fill="none" viewBox="0 0 24 24">
              <circle
                className="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                strokeWidth="4"
              />
              <path
                className="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
              />
            </svg>
          </div>
        );

      case 'dots':
        return (
          <div className={`flex items-center ${currentSizeClasses.dots}`}>
            {[0, 1, 2].map((index) => (
              <div
                key={index}
                className={`${currentColorClasses.dots} rounded-full animate-pulse`}
                style={{
                  width: size === 'xs' ? '8px' : size === 'sm' ? '10px' : size === 'md' ? '12px' : size === 'lg' ? '16px' : '20px',
                  height: size === 'xs' ? '8px' : size === 'sm' ? '10px' : size === 'md' ? '12px' : size === 'lg' ? '16px' : '20px',
                  animationDelay: `${index * 0.2}s`
                }}
              />
            ))}
          </div>
        );

      case 'pulse':
        return (
          <div className={`${currentSizeClasses.pulse} ${currentColorClasses.pulse} rounded-full animate-pulse`} />
        );

      case 'skeleton':
        return (
          <div className={`${currentColorClasses.skeleton} rounded animate-pulse`} />
        );

      default:
        return null;
    }
  };

  const content = (
    <div 
      className={`
        flex flex-col items-center justify-center space-y-3
        ${fullscreen ? 'min-h-screen' : ''}
        ${className}
      `}
    >
      {renderIndicator()}
      {text && (
        <p className={`text-sm ${
          color === 'primary' ? 'text-primary-700' :
          color === 'secondary' ? 'text-secondary-700' :
          color === 'success' ? 'text-success-700' :
          color === 'warning' ? 'text-warning-700' :
          color === 'error' ? 'text-error-700' :
          'text-gray-700'
        } ${size === 'xs' ? 'text-xs' : size === 'sm' ? 'text-sm' : size === 'lg' ? 'text-lg' : size === 'xl' ? 'text-xl' : 'text-base'}`}>
          {text}
        </p>
      )}
    </div>
  );

  // 如果需要遮罩层
  if (overlay || fullscreen) {
    return (
      <div 
        className={`
          fixed inset-0 z-50 flex items-center justify-center
          ${fullscreen ? 'bg-gray-900 bg-opacity-50 backdrop-blur-sm' : 'bg-black bg-opacity-30'}
          animate-fade-in
        `}
      >
        {content}
      </div>
    );
  }

  return content;
};

/**
 * 内联加载指示器，用于按钮或表单内
 */
export const InlineLoading: React.FC<Omit<LoadingIndicatorProps, 'overlay' | 'fullscreen'>> = (props) => {
  return <LoadingIndicator {...props} size="sm" variant="dots" />;
};

/**
 * 全屏加载指示器，用于页面级别的加载状态
 */
export const FullscreenLoading: React.FC<Omit<LoadingIndicatorProps, 'overlay' | 'fullscreen'>> = (props) => {
  return (
    <LoadingIndicator
      {...props}
      fullscreen
      overlay
      text={props.text || '正在加载中...'}
      size="lg"
    />
  );
};

/**
 * 页面骨架屏加载指示器
 */
export const SkeletonLoader: React.FC<{
  /**
   * 骨架屏的行数
   */
  lines?: number;
  
  /**
   * 是否显示头像
   */
  avatar?: boolean;
  
  /**
   * 是否显示标题
   */
  title?: boolean;
  
  /**
   * 容器的类名
   */
  className?: string;
}> = ({ lines = 3, avatar = false, title = true, className = '' }) => {
  return (
    <div className={`animate-pulse ${className}`}>
      <div className="space-y-3">
        {avatar && (
          <div className="flex items-center space-x-4">
            <div className="w-12 h-12 bg-gray-300 rounded-full"></div>
            <div className="flex-1 space-y-2">
              <div className="h-4 bg-gray-300 rounded w-3/4"></div>
              <div className="h-3 bg-gray-300 rounded w-1/2"></div>
            </div>
          </div>
        )}
        
        {title && (
          <div className="h-8 bg-gray-300 rounded w-1/3"></div>
        )}
        
        {Array.from({ length: lines }).map((_, index) => (
          <div key={index} className="space-y-2">
            <div 
              className="h-4 bg-gray-300 rounded" 
              style={{ width: `${Math.random() * 40 + 60}%` }}
            ></div>
            {index % 2 === 0 && (
              <div 
                className="h-4 bg-gray-300 rounded" 
                style={{ width: `${Math.random() * 30 + 20}%` }}
              ></div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};

/**
 * 进度条加载指示器
 */
export const ProgressLoader: React.FC<{
  /**
   * 当前进度 (0-100)
   */
  progress: number;
  
  /**
   * 是否显示百分比
   */
  showPercentage?: boolean;
  
  /**
   * 进度文本
   */
  label?: string;
  
  /**
   * 高度
   */
  height?: 'sm' | 'md' | 'lg';
  
  /**
   * 颜色主题
   */
  color?: 'primary' | 'success' | 'warning' | 'error';
}> = ({ 
  progress, 
  showPercentage = false, 
  label, 
  height = 'md',
  color = 'primary' 
}) => {
  const heightClasses = {
    sm: 'h-1',
    md: 'h-2',
    lg: 'h-3'
  };

  const colorClasses = {
    primary: 'bg-primary-600',
    success: 'bg-success-600',
    warning: 'bg-warning-600',
    error: 'bg-error-600'
  };

  return (
    <div className="w-full">
      {label && (
        <div className="flex justify-between items-center mb-2">
          <span className="text-sm font-medium text-gray-700">{label}</span>
          {showPercentage && (
            <span className="text-sm text-gray-500">{progress}%</span>
          )}
        </div>
      )}
      
      <div className={`w-full bg-gray-200 rounded-full ${heightClasses[height]}`}>
        <div
          className={`${colorClasses[color]} ${heightClasses[height]} rounded-full transition-all duration-300 ease-out`}
          style={{ width: `${progress}%` }}
        />
      </div>
    </div>
  );
};

export default LoadingIndicator;