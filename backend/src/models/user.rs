use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub name: String,
    pub bio: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub gender: Option<String>,
    pub looking_for: Option<String>,
    pub lastfm_username: Option<String>,
    pub lastfm_connected_at: Option<NaiveDateTime>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub password: String,
    pub name: String,
    pub birth_date: Option<NaiveDate>,
    pub gender: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUser {
    pub name: Option<String>,
    pub bio: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub gender: Option<String>,
    pub looking_for: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct UserProfile {
    pub id: String,
    pub name: String,
    pub bio: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub gender: Option<String>,
    pub looking_for: Option<String>,
    pub lastfm_username: Option<String>,
    pub photos: Vec<String>,
    pub top_artists: Vec<String>,
    pub compatibility_score: Option<f64>,
}

impl User {
    pub fn new(email: String, password_hash: String, name: String, birth_date: Option<NaiveDate>, gender: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            email,
            password_hash,
            name,
            bio: None,
            birth_date,
            gender,
            looking_for: None,
            lastfm_username: None,
            lastfm_connected_at: None,
            latitude: None,
            longitude: None,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
        }
    }

    pub fn age(&self) -> Option<u32> {
        self.birth_date.map(|bd| {
            let today = chrono::Utc::now().date_naive();
            let years = today.years_since(bd).unwrap_or(0);
            years
        })
    }
}
