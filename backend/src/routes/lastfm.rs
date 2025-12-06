use crate::{
    errors::AppError,
    middleware::AuthUser,
    AppState,
};
use axum::{
    extract::State,
    Extension, Json,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ConnectLastFmRequest {
    pub username: String,
}

pub async fn connect_lastfm(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Json(req): Json<ConnectLastFmRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Update user with Last.fm username
    sqlx::query("UPDATE users SET lastfm_username = ?, lastfm_connected_at = NOW() WHERE id = ?")
        .bind(&req.username)
        .bind(&auth_user.user_id)
        .execute(&app_state.pool)
        .await?;

    Ok(Json(serde_json::json!({
        "message": "Last.fm account connected successfully",
        "username": req.username,
    })))
}

pub async fn sync_scrobbles(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Get user's Last.fm username
    let user = sqlx::query_as::<_, crate::models::User>("SELECT * FROM users WHERE id = ?")
        .bind(&auth_user.user_id)
        .fetch_optional(&app_state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let lastfm_username = user
        .lastfm_username
        .ok_or_else(|| AppError::Validation("Last.fm account not connected".to_string()))?;

    // Sync scrobbles
    let artists = app_state.lastfm_service
        .sync_user_scrobbles(&app_state.pool, &auth_user.user_id, &lastfm_username)
        .await?;

    Ok(Json(serde_json::json!({
        "message": "Scrobbles synced successfully",
        "artists_count": artists.len(),
    })))
}
