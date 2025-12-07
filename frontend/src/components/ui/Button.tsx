import React from 'react';
import clsx from 'clsx';

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'ghost';
  children: React.ReactNode;
}

export const Button: React.FC<ButtonProps> = ({ 
  variant = 'primary', 
  children, 
  className,
  ...props 
}) => {
  const baseStyles = 'font-medium py-2 px-6 rounded-lg transition-all duration-300 disabled:opacity-50 disabled:cursor-not-allowed';
  
  const variantStyles = {
    primary: 'bg-gradient-to-br from-primary to-secondary text-white shadow-glow-gradient hover:shadow-glow-purple hover:scale-105 transform disabled:hover:scale-100',
    secondary: 'bg-surface hover:bg-surface-hover text-white border border-border hover:border-primary',
    ghost: 'bg-transparent hover:bg-surface text-white',
  };

  return (
    <button
      className={clsx(baseStyles, variantStyles[variant], className)}
      {...props}
    >
      {children}
    </button>
  );
};
