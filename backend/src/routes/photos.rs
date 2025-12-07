use crate::{
    errors::AppError,
    middleware::AuthUser,
    models::CreatePhoto,
    AppState,
};
use axum::{
    extract::{Path, State},
    Extension, Json,
};

pub async fn create_photo(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Json(create_photo): Json<CreatePhoto>,
) -> Result<Json<crate::models::Photo>, AppError> {
    let photo = app_state.photo_service
        .add_photo(&app_state.pool, &auth_user.user_id, create_photo)
        .await?;

    Ok(Json(photo))
}

pub async fn get_user_photos(
    Path(user_id): Path<String>,
    State(app_state): State<AppState>,
) -> Result<Json<Vec<crate::models::Photo>>, AppError> {
    let photos = app_state.photo_service.get_user_photos(&app_state.pool, &user_id).await?;
    Ok(Json(photos))
}

pub async fn delete_photo(
    Extension(auth_user): Extension<AuthUser>,
    Path(photo_id): Path<String>,
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    app_state.photo_service
        .delete_photo(&app_state.pool, &photo_id, &auth_user.user_id)
        .await?;

    Ok(Json(serde_json::json!({
        "message": "Photo deleted successfully"
    })))
}
