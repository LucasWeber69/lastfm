use crate::{
    errors::AppError,
    middleware::AuthUser,
    services::event_service::{CreateEventInterest, EventService},
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct NearbyEventsQuery {
    pub city: Option<String>,
    pub country: Option<String>,
}

/// Get events nearby based on user location
pub async fn get_nearby_events(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Query(query): Query<NearbyEventsQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let events = EventService::get_nearby_events(
        &app_state.pool,
        &auth_user.user_id,
        query.city,
        query.country,
    )
    .await?;

    Ok(Json(serde_json::json!({ "events": events })))
}

/// Get events in common with another user
pub async fn get_common_events(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Path(other_user_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let events = EventService::get_common_events(
        &app_state.pool,
        &auth_user.user_id,
        &other_user_id,
    )
    .await?;

    Ok(Json(serde_json::json!({ "events": events })))
}

/// Get user's event interests
pub async fn get_user_interests(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let interests = EventService::get_user_interests(&app_state.pool, &auth_user.user_id).await?;

    Ok(Json(serde_json::json!({ "interests": interests })))
}

/// Add interest in an event
pub async fn add_interest(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Json(event): Json<CreateEventInterest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let interest = EventService::add_interest(&app_state.pool, &auth_user.user_id, event).await?;

    Ok(Json(serde_json::json!({ "interest": interest })))
}

/// Remove interest in an event
pub async fn remove_interest(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    EventService::remove_interest(&app_state.pool, &auth_user.user_id, &event_id).await?;

    Ok(Json(serde_json::json!({
        "message": "Interest removed successfully"
    })))
}

/// Get popular events
pub async fn get_popular_events(
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let events = EventService::get_popular_events(&app_state.pool, 50).await?;

    Ok(Json(serde_json::json!({ "events": events })))
}
