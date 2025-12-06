import React from 'react';
import { Button } from '@/components/ui/Button';
import { Heart } from 'lucide-react';

interface LikeButtonProps {
  onClick: () => void;
  disabled?: boolean;
}

export const LikeButton: React.FC<LikeButtonProps> = ({ onClick, disabled }) => {
  return (
    <Button
      variant="primary"
      onClick={onClick}
      disabled={disabled}
      className="rounded-full w-16 h-16 flex items-center justify-center"
    >
      <Heart size={28} fill="white" />
    </Button>
  );
};
