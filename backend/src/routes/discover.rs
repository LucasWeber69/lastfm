use crate::{
    errors::AppError,
    middleware::AuthUser,
    models::User,
    AppState,
};
use axum::{
    extract::{Query, State},
    Extension, Json,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct DiscoverProfile {
    pub id: String,
    pub name: String,
    pub age: Option<u32>,
    pub bio: Option<String>,
    pub photos: Vec<String>,
    pub top_artists: Vec<String>,
    pub common_artists: Vec<String>,
    pub compatibility_score: f64,
    pub distance_km: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct DiscoverFilters {
    pub min_age: Option<u32>,
    pub max_age: Option<u32>,
    pub gender: Option<String>,
    pub max_distance: Option<f64>,
    pub genres: Option<String>, // Comma-separated list
}

pub async fn get_discover_profiles(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Query(filters): Query<DiscoverFilters>,
) -> Result<Json<Vec<DiscoverProfile>>, AppError> {
    // Get current user's top artists
    let current_user_artists = app_state.lastfm_service
        .get_user_top_artists(&app_state.pool, &auth_user.user_id, 50)
        .await?;

    if current_user_artists.is_empty() {
        return Ok(Json(vec![]));
    }

    // Get current user's location for distance filtering
    let current_user: User = sqlx::query_as("SELECT * FROM users WHERE id = ?")
        .bind(&auth_user.user_id)
        .fetch_one(&app_state.pool)
        .await?;

    // Build query with filters
    let mut query = String::from(
        "SELECT u.* FROM users u
         WHERE u.id != ?
         AND u.id NOT IN (SELECT to_user_id FROM likes WHERE from_user_id = ?)
         AND u.id NOT IN (SELECT blocked_id FROM blocks WHERE blocker_id = ?)
         AND u.lastfm_username IS NOT NULL"
    );

    // Apply filters
    if filters.gender.is_some() {
        query.push_str(" AND u.gender = ?");
    }

    query.push_str(" LIMIT 50");

    let mut sql_query = sqlx::query_as::<_, User>(&query)
        .bind(&auth_user.user_id)
        .bind(&auth_user.user_id)
        .bind(&auth_user.user_id);

    if let Some(gender) = &filters.gender {
        sql_query = sql_query.bind(gender);
    }

    let potential_matches = sql_query.fetch_all(&app_state.pool).await?;

    let mut profiles = Vec::new();

    for user in potential_matches {
        // Apply age filter
        if let Some(age) = user.age() {
            if let Some(min_age) = filters.min_age {
                if age < min_age {
                    continue;
                }
            }
            if let Some(max_age) = filters.max_age {
                if age > max_age {
                    continue;
                }
            }
        }

        // Calculate distance if both users have location
        let distance_km = if let (Some(lat1), Some(lon1), Some(lat2), Some(lon2)) = (
            current_user.latitude,
            current_user.longitude,
            user.latitude,
            user.longitude,
        ) {
            Some(calculate_distance(lat1, lon1, lat2, lon2))
        } else {
            None
        };

        // Apply distance filter
        if let Some(max_distance) = filters.max_distance {
            if let Some(distance) = distance_km {
                if distance > max_distance {
                    continue;
                }
            } else {
                // Skip users without location if distance filter is applied
                continue;
            }
        }

        // Get compatibility score
        let compatibility_score = app_state.compatibility_service
            .calculate_compatibility(&app_state.pool, &auth_user.user_id, &user.id)
            .await
            .unwrap_or(0.0);

        // Skip users with very low compatibility
        if compatibility_score < 10.0 {
            continue;
        }

        // Get user's top artists
        let user_artists = app_state.lastfm_service
            .get_user_top_artists(&app_state.pool, &user.id, 10)
            .await?;

        // Apply genre filter if specified
        if let Some(genres_str) = &filters.genres {
            let requested_genres: Vec<String> = genres_str
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .collect();
            
            // TODO: Implement genre filtering based on Last.fm artist tags
            // For now, we'll skip this filter as it requires artist tag data
            // In production, you would query artist tags and match against requested genres
        }

        // Get common artists
        let common_artists = app_state.compatibility_service.get_common_artists(
            &current_user_artists,
            &user_artists,
            3,
        );

        // Get user's photos
        let photos = sqlx::query_as::<_, crate::models::Photo>(
            "SELECT * FROM photos WHERE user_id = ? ORDER BY position ASC"
        )
        .bind(&user.id)
        .fetch_all(&app_state.pool)
        .await?;

        let age = user.age();
        
        profiles.push(DiscoverProfile {
            id: user.id,
            name: user.name,
            age,
            bio: user.bio,
            photos: photos.into_iter().map(|p| p.url).collect(),
            top_artists: user_artists.into_iter().take(5).map(|a| a.name).collect(),
            common_artists,
            compatibility_score,
            distance_km,
        });
    }

    // Sort by compatibility score (highest first)
    profiles.sort_by(|a, b| b.compatibility_score.partial_cmp(&a.compatibility_score).unwrap());

    Ok(Json(profiles))
}

/// Calculate distance between two points using Haversine formula
fn calculate_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6371.0; // Earth's radius in kilometers

    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let delta_lat = (lat2 - lat1).to_radians();
    let delta_lon = (lon2 - lon1).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    r * c
}
