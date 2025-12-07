use crate::{
    errors::AppError,
    middleware::AuthUser,
    models::User,
    AppState,
};
use axum::{
    extract::State,
    Extension, Json,
};
use serde::Serialize;

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
}

pub async fn get_discover_profiles(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
) -> Result<Json<Vec<DiscoverProfile>>, AppError> {
    // Get current user's top artists
    let current_user_artists = app_state.lastfm_service
        .get_user_top_artists(&app_state.pool, &auth_user.user_id, 50)
        .await?;

    if current_user_artists.is_empty() {
        return Ok(Json(vec![]));
    }

    // Get users that the current user hasn't liked or blocked
    let potential_matches = sqlx::query_as::<_, User>(
        "SELECT u.* FROM users u
         WHERE u.id != ?
         AND u.id NOT IN (SELECT to_user_id FROM likes WHERE from_user_id = ?)
         AND u.id NOT IN (SELECT blocked_id FROM blocks WHERE blocker_id = ?)
         AND u.lastfm_username IS NOT NULL
         LIMIT 50"
    )
    .bind(&auth_user.user_id)
    .bind(&auth_user.user_id)
    .bind(&auth_user.user_id)
    .fetch_all(&app_state.pool)
    .await?;

    let mut profiles = Vec::new();

    for user in potential_matches {
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
        });
    }

    // Sort by compatibility score (highest first)
    profiles.sort_by(|a, b| b.compatibility_score.partial_cmp(&a.compatibility_score).unwrap());

    Ok(Json(profiles))
}
