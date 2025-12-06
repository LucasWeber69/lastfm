use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Like {
    pub id: String,
    pub from_user_id: String,
    pub to_user_id: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateLike {
    pub to_user_id: String,
}

impl Like {
    pub fn new(from_user_id: String, to_user_id: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            from_user_id,
            to_user_id,
            created_at: chrono::Utc::now().naive_utc(),
        }
    }
}
