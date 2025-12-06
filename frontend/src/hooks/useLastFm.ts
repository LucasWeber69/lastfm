import { useQuery, useMutation } from '@tanstack/react-query';
import { lastfmApi } from '@/api/lastfm';

export const useLastFm = () => {
  const connectMutation = useMutation({
    mutationFn: (username: string) => lastfmApi.connect({ username }),
  });

  const syncMutation = useMutation({
    mutationFn: () => lastfmApi.sync(),
  });

  return {
    connect: connectMutation.mutate,
    sync: syncMutation.mutate,
    isConnecting: connectMutation.isPending,
    isSyncing: syncMutation.isPending,
  };
};
