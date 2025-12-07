use crate::{db::DbPool, errors::AppError, models::Message};
use axum::extract::ws::{Message as WsMessage, WebSocket};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessageType {
    #[serde(rename = "message")]
    Message {
        id: String,
        match_id: Option<String>,
        sender_id: String,
        receiver_id: String,
        content: String,
        created_at: String,
    },
    #[serde(rename = "typing")]
    Typing {
        match_id: String,
        user_id: String,
        is_typing: bool,
    },
    #[serde(rename = "read")]
    MessageRead {
        message_id: String,
        match_id: String,
    },
    #[serde(rename = "error")]
    Error { message: String },
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "pong")]
    Pong,
}

/// Client message from WebSocket
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "send_message")]
    SendMessage {
        match_id: String,
        receiver_id: String,
        content: String,
    },
    #[serde(rename = "typing")]
    Typing {
        match_id: String,
        is_typing: bool,
    },
    #[serde(rename = "mark_read")]
    MarkRead { message_id: String },
    #[serde(rename = "ping")]
    Ping,
}

type Tx = mpsc::UnboundedSender<WsMessageType>;
type ConnectionMap = Arc<RwLock<HashMap<String, Tx>>>;

/// WebSocket connection manager
#[derive(Clone)]
pub struct WebSocketService {
    connections: ConnectionMap,
}

impl WebSocketService {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new WebSocket connection
    pub async fn register_connection(&self, user_id: String, tx: Tx) {
        let mut connections = self.connections.write().await;
        connections.insert(user_id.clone(), tx);
        tracing::info!("WebSocket connection registered for user: {}", user_id);
    }

    /// Unregister a WebSocket connection
    pub async fn unregister_connection(&self, user_id: &str) {
        let mut connections = self.connections.write().await;
        connections.remove(user_id);
        tracing::info!("WebSocket connection unregistered for user: {}", user_id);
    }

    /// Send a message to a specific user
    pub async fn send_to_user(&self, user_id: &str, message: WsMessageType) -> Result<(), AppError> {
        let connections = self.connections.read().await;
        if let Some(tx) = connections.get(user_id) {
            tx.send(message).map_err(|e| {
                AppError::Internal(format!("Failed to send WebSocket message: {}", e))
            })?;
        }
        Ok(())
    }

    /// Check if a user is online
    pub async fn is_user_online(&self, user_id: &str) -> bool {
        let connections = self.connections.read().await;
        connections.contains_key(user_id)
    }

    /// Get count of active connections
    pub async fn connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }

    /// Handle WebSocket connection
    pub async fn handle_connection(
        &self,
        ws: WebSocket,
        user_id: String,
        pool: DbPool,
    ) {
        let (mut ws_tx, mut ws_rx) = ws.split();
        let (tx, mut rx) = mpsc::unbounded_channel();

        // Register the connection
        self.register_connection(user_id.clone(), tx).await;

        // Update user presence to online
        let _ = Self::update_presence(&pool, &user_id, "online").await;

        // Spawn task to send messages to the WebSocket
        let send_task = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if let Ok(json) = serde_json::to_string(&msg) {
                    if ws_tx.send(WsMessage::Text(json)).await.is_err() {
                        break;
                    }
                }
            }
        });

        // Handle incoming messages from the WebSocket
        let service_clone = self.clone();
        let user_id_clone2 = user_id.clone();
        let pool_clone = pool.clone();
        let receive_task = tokio::spawn(async move {
            while let Some(Ok(msg)) = ws_rx.next().await {
                if let WsMessage::Text(text) = msg {
                    if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                        service_clone
                            .handle_client_message(client_msg, &user_id_clone2, &pool_clone)
                            .await;
                    }
                } else if let WsMessage::Close(_) = msg {
                    break;
                }
            }
        });

        // Wait for either task to finish
        tokio::select! {
            _ = send_task => {},
            _ = receive_task => {},
        }

        // Clean up
        self.unregister_connection(&user_id).await;
        let _ = Self::update_presence(&pool, &user_id, "offline").await;
    }

    /// Handle client messages
    async fn handle_client_message(
        &self,
        msg: ClientMessage,
        user_id: &str,
        pool: &DbPool,
    ) {
        match msg {
            ClientMessage::SendMessage {
                match_id,
                receiver_id,
                content,
            } => {
                // Create and save message
                let message = Message::new(match_id.clone(), user_id.to_string(), content.clone());
                
                if let Err(e) = Self::save_message(pool, &message, Some(&receiver_id)).await {
                    tracing::error!("Failed to save message: {}", e);
                    
                    // Send error back to sender
                    let error_msg = WsMessageType::Error {
                        message: "Failed to send message".to_string(),
                    };
                    let _ = self.send_to_user(user_id, error_msg).await;
                    return;
                }

                // Send to receiver if online
                let ws_msg = WsMessageType::Message {
                    id: message.id.clone(),
                    match_id: Some(match_id),
                    sender_id: user_id.to_string(),
                    receiver_id: receiver_id.clone(),
                    content,
                    created_at: message.created_at.to_string(),
                };

                let _ = self.send_to_user(&receiver_id, ws_msg).await;
            }
            ClientMessage::Typing { match_id, is_typing } => {
                // Get the other user in the match
                if let Ok(Some(other_user_id)) = Self::get_other_user_in_match(pool, &match_id, user_id).await {
                    let ws_msg = WsMessageType::Typing {
                        match_id,
                        user_id: user_id.to_string(),
                        is_typing,
                    };
                    let _ = self.send_to_user(&other_user_id, ws_msg).await;
                }
            }
            ClientMessage::MarkRead { message_id } => {
                if let Err(e) = Self::mark_message_read(pool, &message_id).await {
                    tracing::error!("Failed to mark message as read: {}", e);
                }
            }
            ClientMessage::Ping => {
                // Respond with pong - connection keep-alive
            }
        }
    }

    /// Save a message to the database
    async fn save_message(
        pool: &DbPool,
        message: &Message,
        receiver_id: Option<&str>,
    ) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO messages (id, match_id, sender_id, receiver_id, content, created_at) 
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&message.id)
        .bind(&message.match_id)
        .bind(&message.sender_id)
        .bind(receiver_id)
        .bind(&message.content)
        .bind(&message.created_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Mark a message as read
    async fn mark_message_read(pool: &DbPool, message_id: &str) -> Result<(), AppError> {
        sqlx::query("UPDATE messages SET read_at = NOW() WHERE id = ?")
            .bind(message_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// Get the other user in a match
    async fn get_other_user_in_match(
        pool: &DbPool,
        match_id: &str,
        current_user_id: &str,
    ) -> Result<Option<String>, AppError> {
        let result: Option<(String, String)> = sqlx::query_as(
            "SELECT user1_id, user2_id FROM matches WHERE id = ?",
        )
        .bind(match_id)
        .fetch_optional(pool)
        .await?;

        Ok(result.map(|(user1_id, user2_id)| {
            if user1_id == current_user_id {
                user2_id
            } else {
                user1_id
            }
        }))
    }

    /// Update user presence status
    async fn update_presence(
        pool: &DbPool,
        user_id: &str,
        status: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO user_presence (user_id, status, last_seen) 
             VALUES (?, ?, NOW())
             ON DUPLICATE KEY UPDATE status = ?, last_seen = NOW()",
        )
        .bind(user_id)
        .bind(status)
        .bind(status)
        .execute(pool)
        .await?;

        Ok(())
    }
}

impl Default for WebSocketService {
    fn default() -> Self {
        Self::new()
    }
}
