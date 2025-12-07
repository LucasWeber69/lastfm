use crate::{config::Config, errors::AppError, services::AuthService};
use axum::{
    extract::{ConnectInfo, Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};
use std::net::SocketAddr;
use std::sync::Arc;

/// Request context containing authenticated user information and metadata
#[derive(Clone)]
pub struct RequestContext {
    pub user_id: String,
    pub email: String,
    pub ip_address: String,
}

/// Legacy alias for backward compatibility
pub type AuthUser = RequestContext;

pub async fn auth_middleware(
    State(config): State<Arc<Config>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Extract and validate Authorization header
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    // Verify "Bearer " prefix (case-sensitive as per RFC 6750)
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)?;

    // Verify JWT token and extract claims
    let auth_service = AuthService::new((*config).clone());
    let claims = auth_service.verify_token(token)?;

    // Extract IP address for logging and rate limiting
    let ip_address = addr.ip().to_string();

    // Create request context with user info and metadata
    let context = RequestContext {
        user_id: claims.sub.clone(),
        email: claims.email.clone(),
        ip_address,
    };

    // Insert context into request extensions
    request.extensions_mut().insert(context);

    Ok(next.run(request).await)
}
