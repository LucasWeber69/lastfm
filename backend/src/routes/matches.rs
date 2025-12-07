use crate::{
    errors::AppError,
    middleware::AuthUser,
    models::CreateLike,
    AppState,
};
use axum::{
    extract::{Path, State},
    Extension, Json,
};

pub async fn create_like(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Json(create_like): Json<CreateLike>,
) -> Result<Json<serde_json::Value>, AppError> {
    let match_result = app_state.match_service
        .create_like(&app_state.pool, &auth_user.user_id, &create_like.to_user_id)
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
    State(app_state): State<AppState>,
) -> Result<Json<Vec<crate::models::Match>>, AppError> {
    let matches = app_state.match_service.get_user_matches(&app_state.pool, &auth_user.user_id).await?;
    Ok(Json(matches))
}

pub async fn delete_match(
    Extension(auth_user): Extension<AuthUser>,
    Path(match_id): Path<String>,
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    app_state.match_service
        .delete_match(&app_state.pool, &match_id, &auth_user.user_id)
        .await?;

    Ok(Json(serde_json::json!({
        "message": "Match deleted successfully"
    })))
}
