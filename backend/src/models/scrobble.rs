use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Scrobble {
    pub id: String,
    pub user_id: String,
    pub artist_name: String,
    pub artist_mbid: Option<String>,
    pub track_name: Option<String>,
    pub play_count: i32,
    pub listeners: i32,
    pub period: String,
    pub last_synced_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artist {
    pub name: String,
    pub mbid: Option<String>,
    pub play_count: i32,
    pub listeners: i32,
}

impl Scrobble {
    pub fn new(
        user_id: String,
        artist_name: String,
        artist_mbid: Option<String>,
        play_count: i32,
        listeners: i32,
        period: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            artist_name,
            artist_mbid,
            track_name: None,
            play_count,
            listeners,
            period,
            last_synced_at: chrono::Utc::now().naive_utc(),
        }
    }
}
