// 通用卡片组件
//
// 提供统一的卡片容器样式

import React, { HTMLAttributes, forwardRef } from 'react';
import { cva, type VariantProps } from 'class-variance-authority';

// 卡片变体类型
export interface CardVariants {
  variant: 'default' | 'outlined' | 'elevated';
  padding: 'none' | 'sm' | 'md' | 'lg';
}

// 卡片样式变体
const cardVariants = cva(
  // 基础样式
  'rounded-lg border',
  // 变体样式
  {
    variants: {
      variant: {
        default: 'border-gray-200 bg-white',
        outlined: 'border-gray-300 bg-white',
        elevated: 'border-gray-200 bg-white shadow-md',
      },
      padding: {
        none: '',
        sm: 'p-4',
        md: 'p-6',
        lg: 'p-8',
      },
    },
    defaultVariants: {
      variant: 'default',
      padding: 'md',
    },
  }
);

// 卡片属性接口
export interface CardProps
  extends HTMLAttributes<HTMLDivElement>,
    VariantProps<CardVariants> {}

// 卡片组件
export const Card = forwardRef<HTMLDivElement, CardProps>(
  ({ className, variant, padding, ...props }, ref) => (
    <div
      ref={ref}
      className={cardVariants({ variant, padding, className })}
      {...props}
    />
  )
);

Card.displayName = 'Card';

// 卡片头部组件
export const CardHeader = forwardRef<HTMLDivElement, HTMLAttributes<HTMLDivElement>>(
  ({ className, ...props }, ref) => (
    <div
      ref={ref}
      className="flex flex-col space-y-1.5 p-6"
      {...props}
    />
  )
);

CardHeader.displayName = 'CardHeader';

// 卡片标题组件
export const CardTitle = forwardRef<HTMLParagraphElement, HTMLAttributes<HTMLHeadingElement>>(
  ({ className, ...props }, ref) => (
    <h3
      ref={ref}
      className="text-2xl font-semibold leading-none tracking-tight"
      {...props}
    />
  )
);

CardTitle.displayName = 'CardTitle';

// 卡片描述组件
export const CardDescription = forwardRef<HTMLParagraphElement, HTMLAttributes<HTMLParagraphElement>>(
  ({ className, ...props }, ref) => (
    <p
      ref={ref}
      className="text-sm text-gray-600"
      {...props}
    />
  )
);

CardDescription.displayName = 'CardDescription';

// 卡片内容组件
export const CardContent = forwardRef<HTMLDivElement, HTMLAttributes<HTMLDivElement>>(
  ({ className, ...props }, ref) => (
    <div
      ref={ref}
      className="p-6 pt-0"
      {...props}
    />
  )
);

CardContent.displayName = 'CardContent';

// 卡片底部组件
export const CardFooter = forwardRef<HTMLDivElement, HTMLAttributes<HTMLDivElement>>(
  ({ className, ...props }, ref) => (
    <div
      ref={ref}
      className="flex items-center p-6 pt-0"
      {...props}
    />
  )
);

CardFooter.displayName = 'CardFooter';

// 导出类型
export type { CardProps };