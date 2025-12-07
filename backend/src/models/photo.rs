use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Photo {
    pub id: String,
    pub user_id: String,
    pub url: String,
    pub position: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreatePhoto {
    pub url: String,
    pub position: i32,
}

impl Photo {
    pub fn new(user_id: String, url: String, position: i32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            url,
            position,
            created_at: chrono::Utc::now().naive_utc(),
        }
    }
}
