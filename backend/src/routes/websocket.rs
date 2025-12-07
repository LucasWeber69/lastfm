use crate::{
    middleware::AuthUser,
    AppState,
};
use axum::{
    extract::{
        ws::WebSocket,
        WebSocketUpgrade,
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
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, auth_user.user_id, app_state))
}

async fn handle_socket(socket: WebSocket, user_id: String, app_state: AppState) {
    let ws_service = app_state.websocket_service.clone();
    let pool = app_state.pool.clone();
    ws_service.handle_connection(socket, user_id, pool).await;
}
