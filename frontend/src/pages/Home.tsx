import React from 'react';
import { useNavigate } from 'react-router-dom';
import { Card } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Music } from 'lucide-react';

export const Home: React.FC = () => {
  const navigate = useNavigate();

  return (
    <div className="min-h-screen flex items-center justify-center p-4">
      <Card className="max-w-2xl text-center">
        <div className="flex items-center justify-center mb-6">
          <Music className="text-primary" size={64} />
        </div>
        
        <h1 className="text-4xl font-bold mb-4">Last.fm Dating</h1>
        <p className="text-xl text-gray-400 mb-8">
          Find your perfect match through music
        </p>
        
        <p className="text-gray-300 mb-8">
          Connect with people who share your music taste. Our algorithm uses your Last.fm
          scrobbles to find compatible matches based on your favorite artists and listening habits.
        </p>

        <div className="flex gap-4 justify-center">
          <Button onClick={() => navigate('/register')} variant="primary">
            Get Started
          </Button>
          <Button onClick={() => navigate('/login')} variant="secondary">
            Login
          </Button>
        </div>
      </Card>
    </div>
  );
};
