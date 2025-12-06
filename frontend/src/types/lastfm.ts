export interface Artist {
  name: string;
  mbid?: string;
  play_count: number;
  listeners: number;
}

export interface TopArtistsResponse {
  artists_count: number;
  message: string;
}

export interface ConnectLastFmRequest {
  username: string;
}
