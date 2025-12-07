use crate::{
    errors::AppError,
    middleware::AuthUser,
    services::notification_service::CreatePushSubscription,
    AppState,
};
use axum::{
    extract::State,
    Extension, Json,
};
use serde::Deserialize;

/// Subscribe to push notifications
pub async fn subscribe(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Json(subscription): Json<CreatePushSubscription>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_agent = None; // TODO: Extract from request headers if needed

    app_state
        .notification_service
        .subscribe(&app_state.pool, &auth_user.user_id, subscription, user_agent)
        .await?;

    Ok(Json(serde_json::json!({
        "message": "Successfully subscribed to push notifications"
    })))
}

#[derive(Debug, Deserialize)]
pub struct UnsubscribeRequest {
    pub endpoint: String,
}

/// Unsubscribe from push notifications
pub async fn unsubscribe(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Json(req): Json<UnsubscribeRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    app_state
        .notification_service
        .unsubscribe(&app_state.pool, &auth_user.user_id, &req.endpoint)
        .await?;

    Ok(Json(serde_json::json!({
        "message": "Successfully unsubscribed from push notifications"
    })))
}

/// Get user's push subscriptions
pub async fn get_subscriptions(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let subscriptions = app_state
        .notification_service
        .get_user_subscriptions(&app_state.pool, &auth_user.user_id)
        .await?;

    Ok(Json(serde_json::json!({
        "subscriptions": subscriptions
    })))
}
