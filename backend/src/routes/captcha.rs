use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use crate::{errors::AppError, services::CaptchaService, state::AppState};

#[derive(Debug, Deserialize)]
pub struct ValidateCaptchaRequest {
    pub id: String,
    pub answer: String,
}

#[derive(Debug, Serialize)]
pub struct ValidateCaptchaResponse {
    pub valid: bool,
}

/// Generate a new CAPTCHA
/// Public endpoint - no authentication required
pub async fn generate_captcha(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Result<Json<crate::services::captcha_service::CaptchaResponse>, AppError> {
    let ip = addr.ip().to_string();
    let captcha = state.captcha_service.generate(&ip).await?;
    Ok(Json(captcha))
}

/// Validate CAPTCHA (can be called before login/register to check)
/// Public endpoint - no authentication required
pub async fn validate_captcha(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<ValidateCaptchaRequest>,
) -> Result<Json<ValidateCaptchaResponse>, AppError> {
    let ip = addr.ip().to_string();
    let valid = state
        .captcha_service
        .validate(&payload.id, &payload.answer, &ip)
        .await?;
    Ok(Json(ValidateCaptchaResponse { valid }))
}
