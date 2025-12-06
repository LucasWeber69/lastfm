import { apiClient } from './client';
import type { ConnectLastFmRequest, TopArtistsResponse } from '@/types/lastfm';

export const lastfmApi = {
  connect: async (data: ConnectLastFmRequest) => {
    const response = await apiClient.post('/lastfm/connect', data);
    return response.data;
  },

  sync: async (): Promise<TopArtistsResponse> => {
    const response = await apiClient.post('/lastfm/sync');
    return response.data;
  },
};
