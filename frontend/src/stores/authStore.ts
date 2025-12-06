import { create } from 'zustand';
import { authApi } from '@/api/auth';
import type { User } from '@/types/user';

interface AuthState {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  login: (email: string, password: string) => Promise<void>;
  logout: () => void;
  setUser: (user: User) => void;
  initialize: () => void;
}

export const useAuthStore = create<AuthState>((set) => ({
  user: null,
  token: null,
  isAuthenticated: false,

  login: async (email: string, password: string) => {
    const response = await authApi.login({ email, password });
    localStorage.setItem('auth_token', response.token);
    set({
      token: response.token,
      user: response.user as any,
      isAuthenticated: true,
    });
  },

  logout: () => {
    localStorage.removeItem('auth_token');
    set({
      user: null,
      token: null,
      isAuthenticated: false,
    });
  },

  setUser: (user: User) => {
    set({ user });
  },

  initialize: () => {
    const token = localStorage.getItem('auth_token');
    if (token) {
      set({
        token,
        isAuthenticated: true,
      });
    }
  },
}));
