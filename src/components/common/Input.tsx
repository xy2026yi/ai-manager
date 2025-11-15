// 通用输入框组件
//
// 提供统一的输入框样式和行为，支持文本、密码、数字等类型

import React, { InputHTMLAttributes, forwardRef } from 'react';
import { cva, type VariantProps } from 'class-variance-authority';
import { ExclamationCircleIcon } from '@heroicons/react/24/outline';

// 输入框变体类型
export interface InputVariants {
  variant: 'default' | 'filled' | 'flushed' | 'unstyled';
  size: 'sm' | 'md' | 'lg';
}

// 输入框样式变体
const inputVariants = cva(
  // 基础样式
  'flex w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm placeholder:text-gray-500 shadow-sm transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 disabled:cursor-not-allowed disabled:opacity-50',
  // 变体样式
  {
    variants: {
      variant: {
        default: 'border-gray-300 bg-white',
        filled: 'border-gray-300 bg-gray-50',
        flushed: 'border-0 border-b-2 border-gray-300 rounded-none px-0 bg-transparent shadow-none focus:ring-0 focus:border-blue-500',
        unstyled: 'border-0 bg-transparent px-0 shadow-none focus:ring-0 focus:ring-transparent',
      },
      size: {
        sm: 'h-8 text-xs',
        md: 'h-9 text-sm',
        lg: 'h-10 text-base',
      },
    },
    defaultVariants: {
      variant: 'default',
      size: 'md',
    },
  }
);

// 输入框属性接口
export interface InputProps
  extends InputHTMLAttributes<HTMLInputElement>,
    VariantProps<InputVariants> {
  label?: string;
  error?: string;
  helperText?: string;
  required?: boolean;
  leftIcon?: React.ReactNode;
  rightIcon?: React.ReactNode;
}

// 输入框组件
export const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ className, variant, size, type, label, error, helperText, required, leftIcon, rightIcon, id, ...props }, ref) => {
    const inputId = id || `input-${Math.random().toString(36).substr(2, 9)}`;

    return (
      <div className="w-full">
        {label && (
          <label
            htmlFor={inputId}
            className="block text-sm font-medium text-gray-700 mb-1"
          >
            {label}
            {required && <span className="text-red-500 ml-1">*</span>}
          </label>
        )}

        <div className="relative">
          {/* 左侧图标 */}
          {leftIcon && (
            <div className="absolute left-0 pl-3 flex items-center pointer-events-none">
              {leftIcon}
            </div>
          )}

          {/* 输入框 */}
          <input
            type={type}
            id={inputId}
            className={inputVariants({ variant, size, className })}
            ref={ref}
            {...(leftIcon && { className: `${inputVariants({ variant, size, className })} pl-10` })}
            {...(rightIcon && { className: `${inputVariants({ variant, size, className })} pr-10` })}
            {...(error && { className: `${inputVariants({ variant, size, className })} border-red-500 focus:ring-red-500 focus:border-red-500` })}
            {...props}
          />

          {/* 右侧图标 */}
          {rightIcon && (
            <div className="absolute right-0 pr-3 flex items-center pointer-events-none">
              {rightIcon}
            </div>
          )}

          {/* 错误图标 */}
          {error && !rightIcon && (
            <div className="absolute right-0 pr-3 flex items-center pointer-events-none">
              <ExclamationCircleIcon className="h-5 w-5 text-red-500" />
            </div>
          )}
        </div>

        {/* 帮助文本 */}
        {(helperText || error) && (
          <p className={`mt-1 text-xs ${error ? 'text-red-500' : 'text-gray-500'}`}>
            {error || helperText}
          </p>
        )}
      </div>
    );
  }
);

Input.displayName = 'Input';

// Textarea组件
export const Textarea = forwardRef<HTMLTextAreaElement, Omit<InputProps, 'type'>>(
  ({ className, variant, size, ...props }, ref) => (
    <div className="w-full">
      {props.label && (
        <label
          htmlFor={props.id}
          className="block text-sm font-medium text-gray-700 mb-1"
        >
          {props.label}
          {props.required && <span className="text-red-500 ml-1">*</span>}
        </label>
      )}

      <div className="relative">
        <textarea
          className={`flex min-h-[80px] w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm placeholder:text-gray-500 shadow-sm transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 disabled:cursor-not-allowed disabled:opacity-50 ${
            variant === 'filled' ? 'bg-gray-50' : 'bg-white'
          } ${variant === 'flushed' ? 'border-0 border-b-2 border-gray-300 rounded-none px-0 bg-transparent shadow-none focus:ring-0 focus:border-blue-500' : ''} ${
            variant === 'unstyled' ? 'border-0 bg-transparent px-0 shadow-none focus:ring-0' : ''
          } ${size === 'sm' ? 'text-xs' : size === 'lg' ? 'text-base' : 'text-sm'} ${className} ${
            props.error ? 'border-red-500 focus:ring-red-500 focus:border-red-500' : ''
          }`}
          ref={ref}
          {...props}
        />

        {props.error && (
          <div className="absolute right-0 top-2 pr-3 flex items-start pointer-events-none">
            <ExclamationCircleIcon className="h-5 w-5 text-red-500" />
          </div>
        )}
      </div>

      {(props.helperText || props.error) && (
        <p className={`mt-1 text-xs ${props.error ? 'text-red-500' : 'text-gray-500'}`}>
          {props.error || props.helperText}
        </p>
      )}
    </div>
  )
);

Textarea.displayName = 'Textarea';

// Select组件
export interface SelectProps
  extends React.SelectHTMLAttributes<HTMLSelectElement>,
    VariantProps<InputVariants> {
  label?: string;
  error?: string;
  helperText?: string;
  required?: boolean;
  options: { label: string; value: string }[];
  placeholder?: string;
}

export const Select = forwardRef<HTMLSelectElement, SelectProps>(
  ({ className, variant, size, label, error, helperText, required, options, placeholder, id, ...props }, ref) => {
  const selectId = id || `select-${Math.random().toString(36).substr(2, 9)}`;

  return (
    <div className="w-full">
      {label && (
        <label
          htmlFor={selectId}
          className="block text-sm font-medium text-gray-700 mb-1"
        >
          {label}
          {required && <span className="text-red-500 ml-1">*</span>}
        </label>
      )}

      <select
        id={selectId}
        className={inputVariants({ variant, size, className })}
        ref={ref}
        {...(error && { className: `${inputVariants({ variant, size, className })} border-red-500 focus:ring-red-500 focus:border-red-500` })}
        {...props}
      >
        {placeholder && (
          <option value="" disabled>
            {placeholder}
          </option>
        )}
        {options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>

      {(helperText || error) && (
        <p className={`mt-1 text-xs ${error ? 'text-red-500' : 'text-gray-500'}`}>
          {error || helperText}
        </p>
      )}
    </div>
  );
});

Select.displayName = 'Select';

// 导出类型
export type { InputProps, SelectProps };