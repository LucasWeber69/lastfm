use crate::{
    db::DbPool,
    errors::AppError,
    middleware::AuthUser,
    services::LastFmService,
};
use axum::{
    extract::State,
    Extension, Json,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct ConnectLastFmRequest {
    pub username: String,
}

pub async fn connect_lastfm(
    Extension(auth_user): Extension<AuthUser>,
    State(pool): State<DbPool>,
    Json(req): Json<ConnectLastFmRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Update user with Last.fm username
    sqlx::query("UPDATE users SET lastfm_username = ?, lastfm_connected_at = NOW() WHERE id = ?")
        .bind(&req.username)
        .bind(&auth_user.user_id)
        .execute(&pool)
        .await?;

    Ok(Json(serde_json::json!({
        "message": "Last.fm account connected successfully",
        "username": req.username,
    })))
}

pub async fn sync_scrobbles(
    Extension(auth_user): Extension<AuthUser>,
    State(lastfm_service): State<Arc<LastFmService>>,
    State(pool): State<DbPool>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Get user's Last.fm username
    let user = sqlx::query_as::<_, crate::models::User>("SELECT * FROM users WHERE id = ?")
        .bind(&auth_user.user_id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let lastfm_username = user
        .lastfm_username
        .ok_or_else(|| AppError::Validation("Last.fm account not connected".to_string()))?;

    // Sync scrobbles
    let artists = lastfm_service
        .sync_user_scrobbles(&pool, &auth_user.user_id, &lastfm_username)
        .await?;

    Ok(Json(serde_json::json!({
        "message": "Scrobbles synced successfully",
        "artists_count": artists.len(),
    })))
}
