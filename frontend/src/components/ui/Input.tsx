import React from 'react';
import clsx from 'clsx';

interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
}

export const Input: React.FC<InputProps> = ({ 
  label, 
  error, 
  className,
  ...props 
}) => {
  return (
    <div className="w-full">
      {label && (
        <label className="block text-sm font-medium mb-2 text-text-secondary">
          {label}
        </label>
      )}
      <input
        className={clsx(
          'w-full bg-surface border border-border text-white rounded-lg px-4 py-2.5',
          'focus:outline-none focus:border-primary focus:shadow-glow-purple',
          'placeholder:text-text-secondary transition-all duration-300',
          error && 'border-red-500 focus:border-red-500 focus:shadow-none',
          className
        )}
        {...props}
      />
      {error && (
        <p className="mt-1 text-sm text-red-500">{error}</p>
      )}
    </div>
  );
};
