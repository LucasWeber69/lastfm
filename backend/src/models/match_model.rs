use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Match {
    pub id: String,
    pub user1_id: String,
    pub user2_id: String,
    pub compatibility_score: Option<f64>,
    pub created_at: NaiveDateTime,
}

impl Match {
    pub fn new(user1_id: String, user2_id: String, compatibility_score: Option<f64>) -> Self {
        // Ensure user1_id is always lexicographically smaller for consistency
        let (user1, user2) = if user1_id < user2_id {
            (user1_id, user2_id)
        } else {
            (user2_id, user1_id)
        };

        Self {
            id: Uuid::new_v4().to_string(),
            user1_id: user1,
            user2_id: user2,
            compatibility_score,
            created_at: chrono::Utc::now().naive_utc(),
        }
    }
}
