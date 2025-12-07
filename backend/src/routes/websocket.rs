use crate::{
    errors::AppError,
    middleware::AuthUser,
    services::WebSocketService,
    AppState,
};
use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
    Extension,
};

/// WebSocket endpoint for real-time chat
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
) -> Result<Response, AppError> {
    Ok(ws.on_upgrade(move |socket| handle_socket(socket, auth_user.user_id, app_state)))
}

async fn handle_socket(socket: WebSocket, user_id: String, app_state: AppState) {
    app_state
        .websocket_service
        .handle_connection(socket, user_id, app_state.pool)
        .await;
}
