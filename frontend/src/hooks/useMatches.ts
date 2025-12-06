import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { matchesApi } from '@/api/matches';

export const useMatches = () => {
  const queryClient = useQueryClient();

  const { data: matches, isLoading } = useQuery({
    queryKey: ['matches'],
    queryFn: matchesApi.getMatches,
  });

  const { data: discoverProfiles, isLoading: isLoadingProfiles } = useQuery({
    queryKey: ['discover'],
    queryFn: matchesApi.getDiscoverProfiles,
  });

  const likeMutation = useMutation({
    mutationFn: (userId: string) => matchesApi.createLike(userId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['matches'] });
      queryClient.invalidateQueries({ queryKey: ['discover'] });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (matchId: string) => matchesApi.deleteMatch(matchId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['matches'] });
    },
  });

  return {
    matches,
    discoverProfiles,
    isLoading,
    isLoadingProfiles,
    likeUser: likeMutation.mutate,
    deleteMatch: deleteMutation.mutate,
    isLiking: likeMutation.isPending,
  };
};
