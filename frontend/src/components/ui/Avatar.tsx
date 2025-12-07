import React from 'react';
import clsx from 'clsx';

interface AvatarProps {
  src?: string;
  name: string;
  size?: 'sm' | 'md' | 'lg';
  className?: string;
}

export const Avatar: React.FC<AvatarProps> = ({ 
  src, 
  name, 
  size = 'md',
  className 
}) => {
  const sizeStyles = {
    sm: 'w-8 h-8 text-sm',
    md: 'w-12 h-12 text-base',
    lg: 'w-16 h-16 text-xl',
  };

  const initials = name
    .split(' ')
    .map(n => n[0])
    .join('')
    .toUpperCase()
    .slice(0, 2);

  return (
    <div className={clsx(
      'rounded-full flex items-center justify-center font-semibold',
      sizeStyles[size],
      className
    )}>
      {src ? (
        <img src={src} alt={name} className="rounded-full w-full h-full object-cover" />
      ) : (
        <div className="bg-primary w-full h-full rounded-full flex items-center justify-center">
          {initials}
        </div>
      )}
    </div>
  );
};
