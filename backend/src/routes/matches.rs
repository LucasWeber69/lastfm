use crate::{
    db::DbPool,
    errors::AppError,
    middleware::AuthUser,
    models::CreateLike,
    services::MatchService,
};
use axum::{
    extract::{Path, State},
    Extension, Json,
};
use std::sync::Arc;

pub async fn create_like(
    Extension(auth_user): Extension<AuthUser>,
    State(match_service): State<Arc<MatchService>>,
    State(pool): State<DbPool>,
    Json(create_like): Json<CreateLike>,
) -> Result<Json<serde_json::Value>, AppError> {
    let match_result = match_service
        .create_like(&pool, &auth_user.user_id, &create_like.to_user_id)
        .await?;

    match match_result {
        Some(match_record) => Ok(Json(serde_json::json!({
            "liked": true,
            "matched": true,
            "match": match_record,
        }))),
        None => Ok(Json(serde_json::json!({
            "liked": true,
            "matched": false,
        }))),
    }
}

pub async fn get_matches(
    Extension(auth_user): Extension<AuthUser>,
    State(match_service): State<Arc<MatchService>>,
    State(pool): State<DbPool>,
) -> Result<Json<Vec<crate::models::Match>>, AppError> {
    let matches = match_service.get_user_matches(&pool, &auth_user.user_id).await?;
    Ok(Json(matches))
}

pub async fn delete_match(
    Extension(auth_user): Extension<AuthUser>,
    Path(match_id): Path<String>,
    State(match_service): State<Arc<MatchService>>,
    State(pool): State<DbPool>,
) -> Result<Json<serde_json::Value>, AppError> {
    match_service
        .delete_match(&pool, &match_id, &auth_user.user_id)
        .await?;

    Ok(Json(serde_json::json!({
        "message": "Match deleted successfully"
    })))
}
