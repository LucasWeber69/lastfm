import React, { useState } from 'react';
import { Card } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { useAuth } from '@/hooks/useAuth';
import { useLastFm } from '@/hooks/useLastFm';
import { useQuery } from '@tanstack/react-query';
import { usersApi } from '@/api/users';
import { Music, LogOut } from 'lucide-react';

export const Profile: React.FC = () => {
  const { logout } = useAuth();
  const { connect, sync, isConnecting, isSyncing } = useLastFm();
  const [lastfmUsername, setLastfmUsername] = useState('');

  const { data: user, isLoading } = useQuery({
    queryKey: ['me'],
    queryFn: usersApi.getMe,
  });

  const handleConnectLastFm = () => {
    if (lastfmUsername) {
      connect(lastfmUsername);
    }
  };

  const handleSync = () => {
    sync();
  };

  if (isLoading) {
    return (
      <div className="container mx-auto p-4 pb-24 md:pb-4">
        <p>Loading profile...</p>
      </div>
    );
  }

  return (
    <div className="container mx-auto p-4 pb-24 md:pb-4">
      <h1 className="text-3xl font-bold mb-8">Profile</h1>

      <div className="max-w-2xl mx-auto space-y-6">
        <Card>
          <h2 className="text-2xl font-bold mb-4">{user?.name}</h2>
          <p className="text-gray-400">{user?.email}</p>
          
          {user?.bio && (
            <p className="mt-4 text-gray-300">{user.bio}</p>
          )}
        </Card>

        <Card>
          <h3 className="text-xl font-bold mb-4 flex items-center gap-2">
            <Music size={24} className="text-primary" />
            Last.fm Connection
          </h3>

          {user?.lastfm_username ? (
            <div>
              <p className="text-gray-300 mb-4">
                Connected as: <span className="text-primary">{user.lastfm_username}</span>
              </p>
              <Button onClick={handleSync} disabled={isSyncing}>
                {isSyncing ? 'Syncing...' : 'Sync Scrobbles'}
              </Button>
            </div>
          ) : (
            <div className="space-y-4">
              <p className="text-gray-400">
                Connect your Last.fm account to find matches based on your music taste
              </p>
              <div className="flex gap-2">
                <Input
                  placeholder="Last.fm username"
                  value={lastfmUsername}
                  onChange={(e) => setLastfmUsername(e.target.value)}
                />
                <Button onClick={handleConnectLastFm} disabled={isConnecting}>
                  {isConnecting ? 'Connecting...' : 'Connect'}
                </Button>
              </div>
            </div>
          )}
        </Card>

        <Card>
          <Button
            onClick={logout}
            variant="ghost"
            className="w-full flex items-center justify-center gap-2"
          >
            <LogOut size={20} />
            Logout
          </Button>
        </Card>
      </div>
    </div>
  );
};
