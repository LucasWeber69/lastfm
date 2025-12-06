export interface Match {
  id: string;
  user1_id: string;
  user2_id: string;
  compatibility_score?: number;
  created_at: string;
}

export interface Like {
  id: string;
  from_user_id: string;
  to_user_id: string;
  created_at: string;
}

export interface Message {
  id: string;
  match_id: string;
  sender_id: string;
  content: string;
  read_at?: string;
  created_at: string;
}
