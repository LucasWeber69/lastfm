use crate::{
    errors::AppError,
    middleware::AuthUser,
    services::AchievementService,
    AppState,
};
use axum::{
    extract::{Path, State},
    Extension, Json,
};

/// Get all achievements with user's progress
pub async fn get_achievements(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let achievements = AchievementService::get_user_achievements(
        &app_state.pool,
        &auth_user.user_id,
    )
    .await?;

    Ok(Json(serde_json::json!({ "achievements": achievements })))
}

/// Get user's statistics
pub async fn get_user_stats(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let stats = AchievementService::get_user_stats(&app_state.pool, &auth_user.user_id).await?;

    Ok(Json(serde_json::json!({ "stats": stats })))
}

/// Get another user's achievements (public view)
pub async fn get_user_achievements_public(
    State(app_state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let achievements = AchievementService::get_user_achievements(&app_state.pool, &user_id).await?;

    // Filter to only show unlocked achievements for privacy
    let unlocked: Vec<_> = achievements
        .into_iter()
        .filter(|a| a.unlocked)
        .collect();

    Ok(Json(serde_json::json!({ "achievements": unlocked })))
}

/// Get another user's stats (public view)
pub async fn get_user_stats_public(
    State(app_state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let stats = AchievementService::get_user_stats(&app_state.pool, &user_id).await?;

    // Return limited stats for privacy
    Ok(Json(serde_json::json!({
        "stats": {
            "level": stats.level,
            "total_points": stats.total_points,
            "current_streak_days": stats.current_streak_days,
        }
    })))
}
