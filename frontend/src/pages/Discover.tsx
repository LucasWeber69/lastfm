import React, { useState } from 'react';
import { SwipeCard } from '@/components/discover/SwipeCard';
import { LikeButton } from '@/components/discover/LikeButton';
import { DislikeButton } from '@/components/discover/DislikeButton';
import { useMatches } from '@/hooks/useMatches';
import { Card } from '@/components/ui/Card';
import { Heart } from 'lucide-react';

export const Discover: React.FC = () => {
  const { discoverProfiles, likeUser, isLoadingProfiles, isLiking } = useMatches();
  const [currentIndex, setCurrentIndex] = useState(0);

  if (isLoadingProfiles) {
    return (
      <div className="container mx-auto p-4 flex items-center justify-center min-h-screen">
        <p>Loading profiles...</p>
      </div>
    );
  }

  if (!discoverProfiles || discoverProfiles.length === 0) {
    return (
      <div className="container mx-auto p-4 flex items-center justify-center min-h-screen">
        <Card className="text-center">
          <Heart className="mx-auto mb-4 text-gray-400" size={48} />
          <h2 className="text-2xl font-bold mb-2">No more profiles</h2>
          <p className="text-gray-400">
            Check back later for new matches!
          </p>
        </Card>
      </div>
    );
  }

  const currentProfile = discoverProfiles[currentIndex];

  const handleLike = () => {
    if (currentProfile && !isLiking) {
      likeUser(currentProfile.id);
      handleNext();
    }
  };

  const handleDislike = () => {
    handleNext();
  };

  const handleNext = () => {
    if (currentIndex < discoverProfiles.length - 1) {
      setCurrentIndex(currentIndex + 1);
    }
  };

  return (
    <div className="container mx-auto p-4 pb-24 md:pb-4">
      <h1 className="text-3xl font-bold mb-8 text-center">Discover</h1>

      <div className="max-w-md mx-auto">
        {currentProfile && <SwipeCard profile={currentProfile} />}

        <div className="flex justify-center gap-8 mt-8">
          <DislikeButton onClick={handleDislike} disabled={isLiking} />
          <LikeButton onClick={handleLike} disabled={isLiking} />
        </div>

        <p className="text-center text-gray-400 mt-4">
          {currentIndex + 1} / {discoverProfiles.length}
        </p>
      </div>
    </div>
  );
};
