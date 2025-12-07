import React from 'react';
import { Card } from '@/components/ui/Card';
import type { UserProfile } from '@/types/user';
import { Music } from 'lucide-react';

interface SwipeCardProps {
  profile: UserProfile;
}

export const SwipeCard: React.FC<SwipeCardProps> = ({ profile }) => {
  const mainPhoto = profile.photos[0] || 'https://via.placeholder.com/400';

  return (
    <Card className="max-w-md mx-auto relative overflow-hidden">
      <div className="relative">
        <img 
          src={mainPhoto} 
          alt={profile.name} 
          className="w-full h-96 object-cover rounded-lg"
        />
        
        <div className="absolute top-4 right-4 bg-primary px-3 py-1 rounded-full text-white font-bold">
          {Math.round(profile.compatibility_score)}% match 🎵
        </div>
      </div>

      <div className="mt-4">
        <h2 className="text-2xl font-bold">
          {profile.name}
          {profile.age && <span className="text-gray-400">, {profile.age}</span>}
        </h2>
        
        {profile.bio && (
          <p className="text-gray-300 mt-2">{profile.bio}</p>
        )}

        {profile.common_artists.length > 0 && (
          <div className="mt-4">
            <p className="text-sm text-gray-400 flex items-center gap-2">
              <Music size={16} />
              Common artists:
            </p>
            <div className="flex flex-wrap gap-2 mt-2">
              {profile.common_artists.map((artist) => (
                <span 
                  key={artist}
                  className="bg-surface px-3 py-1 rounded-full text-sm border border-primary"
                >
                  {artist}
                </span>
              ))}
            </div>
          </div>
        )}

        <div className="mt-4">
          <p className="text-sm text-gray-400">Top artists:</p>
          <div className="flex flex-wrap gap-2 mt-2">
            {profile.top_artists.slice(0, 3).map((artist) => (
              <span 
                key={artist}
                className="bg-surface px-3 py-1 rounded-full text-sm"
              >
                {artist}
              </span>
            ))}
          </div>
        </div>
      </div>
    </Card>
  );
};
