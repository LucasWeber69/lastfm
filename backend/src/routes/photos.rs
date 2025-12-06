use crate::{
    db::DbPool,
    errors::AppError,
    middleware::AuthUser,
    models::CreatePhoto,
    services::PhotoService,
};
use axum::{
    extract::{Path, State},
    Extension, Json,
};
use std::sync::Arc;

pub async fn create_photo(
    Extension(auth_user): Extension<AuthUser>,
    State(photo_service): State<Arc<PhotoService>>,
    State(pool): State<DbPool>,
    Json(create_photo): Json<CreatePhoto>,
) -> Result<Json<crate::models::Photo>, AppError> {
    let photo = photo_service
        .add_photo(&pool, &auth_user.user_id, create_photo)
        .await?;

    Ok(Json(photo))
}

pub async fn get_user_photos(
    Path(user_id): Path<String>,
    State(photo_service): State<Arc<PhotoService>>,
    State(pool): State<DbPool>,
) -> Result<Json<Vec<crate::models::Photo>>, AppError> {
    let photos = photo_service.get_user_photos(&pool, &user_id).await?;
    Ok(Json(photos))
}

pub async fn delete_photo(
    Extension(auth_user): Extension<AuthUser>,
    Path(photo_id): Path<String>,
    State(photo_service): State<Arc<PhotoService>>,
    State(pool): State<DbPool>,
) -> Result<Json<serde_json::Value>, AppError> {
    photo_service
        .delete_photo(&pool, &photo_id, &auth_user.user_id)
        .await?;

    Ok(Json(serde_json::json!({
        "message": "Photo deleted successfully"
    })))
}
