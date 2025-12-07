use crate::{
    errors::AppError,
    models::CreateUser,
    services::auth_service::LoginRequest,
    AppState,
};
use axum::{extract::{ConnectInfo, State}, Json};
use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    #[serde(flatten)]
    pub user: CreateUser,
    pub captcha_id: String,
    pub captcha_answer: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequestWithCaptcha {
    #[serde(flatten)]
    pub login: LoginRequest,
    pub captcha_id: String,
    pub captcha_answer: String,
}

pub async fn register(
    State(app_state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Validate CAPTCHA first (anti-bot protection)
    let ip = addr.ip().to_string();
    let captcha_valid = app_state
        .captcha_service
        .validate(&req.captcha_id, &req.captcha_answer, &ip)
        .await?;
    
    if !captcha_valid {
        return Err(AppError::Auth("Invalid CAPTCHA".to_string()));
    }

    let user = app_state.auth_service.register(&app_state.pool, req.user).await?;
    
    // Generate JWT token for immediate login after registration
    let token = app_state.auth_service.generate_token(&user.id, &user.email)?;

    Ok(Json(serde_json::json!({
        "token": token,
        "user": {
            "id": user.id,
            "email": user.email,
            "name": user.name,
        }
    })))
}

pub async fn login(
    State(app_state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<LoginRequestWithCaptcha>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Validate CAPTCHA first (anti-brute-force protection)
    let ip = addr.ip().to_string();
    let captcha_valid = app_state
        .captcha_service
        .validate(&req.captcha_id, &req.captcha_answer, &ip)
        .await?;
    
    if !captcha_valid {
        return Err(AppError::Auth("Invalid CAPTCHA".to_string()));
    }

    let auth_response = app_state.auth_service.login(&app_state.pool, req.login).await?;

    Ok(Json(serde_json::json!(auth_response)))
}

pub async fn logout() -> Result<Json<serde_json::Value>, AppError> {
    // In a real implementation, you might want to blacklist the token
    Ok(Json(serde_json::json!({
        "message": "Logged out successfully"
    })))
}
