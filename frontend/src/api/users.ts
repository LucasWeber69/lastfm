import { apiClient } from './client';
import type { User, UpdateUser } from '@/types/user';

export const usersApi = {
  getMe: async (): Promise<User> => {
    const response = await apiClient.get('/users/me');
    return response.data;
  },

  updateMe: async (data: UpdateUser): Promise<User> => {
    const response = await apiClient.put('/users/me', data);
    return response.data;
  },

  getUser: async (userId: string): Promise<User> => {
    const response = await apiClient.get(`/users/${userId}`);
    return response.data;
  },
};
