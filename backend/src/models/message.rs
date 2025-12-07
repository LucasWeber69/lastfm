use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: String,
    pub match_id: String,
    pub sender_id: String,
    pub content: String,
    pub read_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateMessage {
    pub content: String,
}

impl Message {
    pub fn new(match_id: String, sender_id: String, content: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            match_id,
            sender_id,
            content,
            read_at: None,
            created_at: chrono::Utc::now().naive_utc(),
        }
    }
}
