use crate::{
    errors::AppError,
    models::CreateUser,
    services::auth_service::LoginRequest,
    AppState,
};
use axum::{extract::State, Json};

pub async fn register(
    State(app_state): State<AppState>,
    Json(create_user): Json<CreateUser>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = app_state.auth_service.register(&app_state.pool, create_user).await?;

    Ok(Json(serde_json::json!({
        "id": user.id,
        "email": user.email,
        "name": user.name,
    })))
}

pub async fn login(
    State(app_state): State<AppState>,
    Json(login_req): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let auth_response = app_state.auth_service.login(&app_state.pool, login_req).await?;

    Ok(Json(serde_json::json!(auth_response)))
}

pub async fn logout() -> Result<Json<serde_json::Value>, AppError> {
    // In a real implementation, you might want to blacklist the token
    Ok(Json(serde_json::json!({
        "message": "Logged out successfully"
    })))
}
