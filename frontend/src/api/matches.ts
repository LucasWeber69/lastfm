import { apiClient } from './client';
import type { Match } from '@/types/match';

export const matchesApi = {
  createLike: async (toUserId: string) => {
    const response = await apiClient.post('/likes', { to_user_id: toUserId });
    return response.data;
  },

  getMatches: async (): Promise<Match[]> => {
    const response = await apiClient.get('/matches');
    return response.data;
  },

  deleteMatch: async (matchId: string) => {
    const response = await apiClient.delete(`/matches/${matchId}`);
    return response.data;
  },

  getDiscoverProfiles: async () => {
    const response = await apiClient.get('/discover');
    return response.data;
  },
};
