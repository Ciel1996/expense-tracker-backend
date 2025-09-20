import { ReactNode } from 'react';

interface CardProps {
  children: ReactNode;
  className?: string;
  padding?: 'none' | 'sm' | 'md' | 'lg' | 'xl';
  shadow?: 'none' | 'sm' | 'md' | 'lg' | 'xl';
  hover?: boolean;
}

export function Card({
                       children,
                       className = '',
                       padding = 'md',
                       shadow = 'md',
                       hover = false
                     }: CardProps) {
  const paddingClasses = {
    none: '',
    sm: 'p-3',
    md: 'p-4',
    lg: 'p-6',
    xl: 'p-8'
  };

  const shadowClasses = {
    none: '',
    sm: 'shadow-sm',
    md: 'shadow-md',
    lg: 'shadow-lg',
    xl: 'shadow-xl'
  };

  const baseClasses = [
    'bg-white dark:bg-gray-800',
    'border border-gray-200 dark:border-gray-700',
    'rounded-lg',
    'w-fit',
    paddingClasses[padding],
    shadowClasses[shadow],
    hover ? 'hover:shadow-lg transition-shadow duration-200' : '',
    className
  ].filter(Boolean).join(' ');

  return (
    <div className={baseClasses}>
      {children}
    </div>
  );
}
