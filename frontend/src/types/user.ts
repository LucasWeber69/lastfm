export interface User {
  id: string;
  email: string;
  name: string;
  bio?: string;
  birth_date?: string;
  gender?: string;
  looking_for?: string;
  lastfm_username?: string;
  lastfm_connected_at?: string;
  latitude?: number;
  longitude?: number;
  created_at: string;
  updated_at: string;
}

export interface CreateUser {
  email: string;
  password: string;
  name: string;
  birth_date?: string;
  gender?: string;
}

export interface UpdateUser {
  name?: string;
  bio?: string;
  birth_date?: string;
  gender?: string;
  looking_for?: string;
  latitude?: number;
  longitude?: number;
}

export interface UserProfile {
  id: string;
  name: string;
  age?: number;
  bio?: string;
  photos: string[];
  top_artists: string[];
  common_artists: string[];
  compatibility_score: number;
}
