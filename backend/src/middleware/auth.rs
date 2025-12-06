use crate::{config::Config, errors::AppError, services::AuthService};
use axum::{
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AuthUser {
    pub user_id: String,
}

pub async fn auth_middleware(
    State(config): State<Arc<Config>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)?;

    let auth_service = AuthService::new((*config).clone());
    let claims = auth_service.verify_token(token)?;

    request.extensions_mut().insert(AuthUser {
        user_id: claims.sub,
    });

    Ok(next.run(request).await)
}
