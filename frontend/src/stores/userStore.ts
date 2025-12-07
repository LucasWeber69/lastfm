import { create } from 'zustand';
import type { UserProfile } from '@/types/user';

interface UserState {
  currentProfile: UserProfile | null;
  discoverProfiles: UserProfile[];
  setCurrentProfile: (profile: UserProfile | null) => void;
  setDiscoverProfiles: (profiles: UserProfile[]) => void;
  removeProfile: (profileId: string) => void;
}

export const useUserStore = create<UserState>((set) => ({
  currentProfile: null,
  discoverProfiles: [],

  setCurrentProfile: (profile) => {
    set({ currentProfile: profile });
  },

  setDiscoverProfiles: (profiles) => {
    set({ discoverProfiles: profiles });
  },

  removeProfile: (profileId) => {
    set((state) => ({
      discoverProfiles: state.discoverProfiles.filter((p) => p.id !== profileId),
    }));
  },
}));
