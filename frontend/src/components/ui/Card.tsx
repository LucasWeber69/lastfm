import React from 'react';
import clsx from 'clsx';

interface CardProps {
  children: React.ReactNode;
  className?: string;
  variant?: 'default' | 'highlight';
}

export const Card: React.FC<CardProps> = ({ children, className, variant = 'default' }) => {
  const baseStyles = 'rounded-xl p-6 shadow-lg transition-all duration-300';
  
  const variantStyles = {
    default: 'bg-surface/80 backdrop-blur-xl border border-border/50 hover:border-primary/30 hover:shadow-glow-purple',
    highlight: 'bg-gradient-to-br from-surface to-surface-hover border border-primary/20 hover:shadow-glow-gradient',
  };

  return (
    <div className={clsx(baseStyles, variantStyles[variant], className)}>
      {children}
    </div>
  );
};
