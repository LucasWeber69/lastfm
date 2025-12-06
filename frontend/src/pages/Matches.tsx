import React from 'react';
import { Card } from '@/components/ui/Card';
import { useMatches } from '@/hooks/useMatches';
import { Heart } from 'lucide-react';

export const Matches: React.FC = () => {
  const { matches, isLoading } = useMatches();

  if (isLoading) {
    return (
      <div className="container mx-auto p-4 pb-24 md:pb-4">
        <p>Loading matches...</p>
      </div>
    );
  }

  return (
    <div className="container mx-auto p-4 pb-24 md:pb-4">
      <h1 className="text-3xl font-bold mb-8">Matches</h1>

      {!matches || matches.length === 0 ? (
        <Card className="text-center">
          <Heart className="mx-auto mb-4 text-gray-400" size={48} />
          <h2 className="text-2xl font-bold mb-2">No matches yet</h2>
          <p className="text-gray-400">
            Start swiping to find your perfect music match!
          </p>
        </Card>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {matches.map((match) => (
            <Card key={match.id}>
              <div className="flex items-center gap-4">
                <div className="flex-1">
                  <h3 className="font-bold text-lg">Match</h3>
                  <p className="text-sm text-gray-400">
                    {match.compatibility_score
                      ? `${Math.round(match.compatibility_score)}% compatible`
                      : 'New match!'}
                  </p>
                </div>
              </div>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
};
