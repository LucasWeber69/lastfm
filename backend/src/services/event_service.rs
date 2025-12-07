use crate::{db::DbPool, errors::AppError};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EventInterest {
    pub id: String,
    pub user_id: String,
    pub event_id: String,
    pub event_name: String,
    pub artist_name: Option<String>,
    pub venue_name: Option<String>,
    pub event_date: Option<NaiveDateTime>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub external_url: Option<String>,
    pub interested_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEventInterest {
    pub event_id: String,
    pub event_name: String,
    pub artist_name: Option<String>,
    pub venue_name: Option<String>,
    pub event_date: Option<NaiveDateTime>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub external_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventWithCommonUsers {
    pub event_id: String,
    pub event_name: String,
    pub artist_name: Option<String>,
    pub venue_name: Option<String>,
    pub event_date: Option<NaiveDateTime>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub external_url: Option<String>,
    pub common_user_count: i32,
    pub common_user_names: Vec<String>,
}

pub struct EventService;

impl EventService {
    /// Add event interest for a user
    pub async fn add_interest(
        pool: &DbPool,
        user_id: &str,
        event: CreateEventInterest,
    ) -> Result<EventInterest, AppError> {
        let id = Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO event_interests 
             (id, user_id, event_id, event_name, artist_name, venue_name, event_date, city, country, external_url)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(user_id)
        .bind(&event.event_id)
        .bind(&event.event_name)
        .bind(&event.artist_name)
        .bind(&event.venue_name)
        .bind(&event.event_date)
        .bind(&event.city)
        .bind(&event.country)
        .bind(&event.external_url)
        .execute(pool)
        .await?;

        let interest = sqlx::query_as::<_, EventInterest>(
            "SELECT * FROM event_interests WHERE id = ?",
        )
        .bind(&id)
        .fetch_one(pool)
        .await?;

        Ok(interest)
    }

    /// Remove event interest
    pub async fn remove_interest(
        pool: &DbPool,
        user_id: &str,
        event_id: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            "DELETE FROM event_interests WHERE user_id = ? AND event_id = ?",
        )
        .bind(user_id)
        .bind(event_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Get user's event interests
    pub async fn get_user_interests(
        pool: &DbPool,
        user_id: &str,
    ) -> Result<Vec<EventInterest>, AppError> {
        let interests = sqlx::query_as::<_, EventInterest>(
            "SELECT * FROM event_interests WHERE user_id = ? ORDER BY event_date ASC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(interests)
    }

    /// Get events nearby (based on city/country)
    /// In production, this would integrate with an external API like Songkick or Bandsintown
    pub async fn get_nearby_events(
        pool: &DbPool,
        user_id: &str,
        city: Option<String>,
        country: Option<String>,
    ) -> Result<Vec<EventInterest>, AppError> {
        // For now, return events from the same city/country that other users are interested in
        let mut query_parts = vec!["SELECT DISTINCT event_id, event_name, artist_name, venue_name, event_date, city, country, external_url FROM event_interests WHERE user_id != ?"];
        let mut conditions = vec![];

        if city.is_some() {
            conditions.push("city = ?");
        }
        if country.is_some() {
            conditions.push("country = ?");
        }

        if !conditions.is_empty() {
            query_parts.push(" AND ");
            query_parts.push(&conditions.join(" AND "));
        }

        query_parts.push(" ORDER BY event_date ASC LIMIT 50");

        let query_str = query_parts.join("");
        let mut query = sqlx::query_as::<_, EventInterest>(&query_str).bind(user_id);

        if let Some(c) = city {
            query = query.bind(c);
        }
        if let Some(c) = country {
            query = query.bind(c);
        }

        let events = query.fetch_all(pool).await?;

        Ok(events)
    }

    /// Get events in common with another user
    pub async fn get_common_events(
        pool: &DbPool,
        user_id: &str,
        other_user_id: &str,
    ) -> Result<Vec<EventInterest>, AppError> {
        let events = sqlx::query_as::<_, EventInterest>(
            "SELECT e1.* FROM event_interests e1
             INNER JOIN event_interests e2 ON e1.event_id = e2.event_id
             WHERE e1.user_id = ? AND e2.user_id = ?
             ORDER BY e1.event_date ASC",
        )
        .bind(user_id)
        .bind(other_user_id)
        .fetch_all(pool)
        .await?;

        Ok(events)
    }

    /// Get popular events with count of interested users
    pub async fn get_popular_events(
        pool: &DbPool,
        limit: i64,
    ) -> Result<Vec<EventWithCommonUsers>, AppError> {
        #[derive(sqlx::FromRow)]
        struct EventRow {
            event_id: String,
            event_name: String,
            artist_name: Option<String>,
            venue_name: Option<String>,
            event_date: Option<NaiveDateTime>,
            city: Option<String>,
            country: Option<String>,
            external_url: Option<String>,
            user_count: i32,
        }

        let rows = sqlx::query_as::<_, EventRow>(
            "SELECT 
                event_id, 
                event_name, 
                artist_name, 
                venue_name, 
                event_date, 
                city, 
                country, 
                external_url,
                COUNT(DISTINCT user_id) as user_count
             FROM event_interests
             GROUP BY event_id
             ORDER BY user_count DESC, event_date ASC
             LIMIT ?",
        )
        .bind(limit)
        .fetch_all(pool)
        .await?;

        let mut events = Vec::new();
        for row in rows {
            // Get user names interested in this event (first 5)
            let user_names: Vec<(String,)> = sqlx::query_as(
                "SELECT u.name FROM users u
                 INNER JOIN event_interests ei ON u.id = ei.user_id
                 WHERE ei.event_id = ?
                 LIMIT 5",
            )
            .bind(&row.event_id)
            .fetch_all(pool)
            .await?;

            events.push(EventWithCommonUsers {
                event_id: row.event_id,
                event_name: row.event_name,
                artist_name: row.artist_name,
                venue_name: row.venue_name,
                event_date: row.event_date,
                city: row.city,
                country: row.country,
                external_url: row.external_url,
                common_user_count: row.user_count,
                common_user_names: user_names.into_iter().map(|(name,)| name).collect(),
            });
        }

        Ok(events)
    }

    /// Get events based on user's top artists
    /// This would integrate with external API in production
    pub async fn get_recommended_events(
        pool: &DbPool,
        _user_id: &str,
        _top_artists: Vec<String>,
    ) -> Result<Vec<EventInterest>, AppError> {
        // Placeholder: In production, this would:
        // 1. Query external API (Songkick, Bandsintown) for events by these artists
        // 2. Cache results in events_cache table
        // 3. Return personalized event recommendations
        
        // For now, return empty vec as this requires external API integration
        Ok(vec![])
    }
}
