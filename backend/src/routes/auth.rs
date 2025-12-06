use crate::{
    db::DbPool,
    errors::AppError,
    models::CreateUser,
    services::auth_service::{AuthService, LoginRequest},
};
use axum::{extract::State, Json};
use std::sync::Arc;

pub async fn register(
    State(auth_service): State<Arc<AuthService>>,
    State(pool): State<DbPool>,
    Json(create_user): Json<CreateUser>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = auth_service.register(&pool, create_user).await?;

    Ok(Json(serde_json::json!({
        "id": user.id,
        "email": user.email,
        "name": user.name,
    })))
}

pub async fn login(
    State(auth_service): State<Arc<AuthService>>,
    State(pool): State<DbPool>,
    Json(login_req): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let auth_response = auth_service.login(&pool, login_req).await?;

    Ok(Json(serde_json::json!(auth_response)))
}

pub async fn logout() -> Result<Json<serde_json::Value>, AppError> {
    // In a real implementation, you might want to blacklist the token
    Ok(Json(serde_json::json!({
        "message": "Logged out successfully"
    })))
}
