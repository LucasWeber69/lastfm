import React from 'react';
import { Button } from '@/components/ui/Button';
import { X } from 'lucide-react';

interface DislikeButtonProps {
  onClick: () => void;
  disabled?: boolean;
}

export const DislikeButton: React.FC<DislikeButtonProps> = ({ onClick, disabled }) => {
  return (
    <Button
      variant="ghost"
      onClick={onClick}
      disabled={disabled}
      className="rounded-full w-16 h-16 flex items-center justify-center border border-gray-600"
    >
      <X size={28} />
    </Button>
  );
};
