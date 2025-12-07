use crate::{db::DbPool, errors::AppError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PushSubscription {
    pub id: String,
    pub user_id: String,
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePushSubscription {
    pub endpoint: String,
    pub keys: PushSubscriptionKeys,
}

#[derive(Debug, Deserialize)]
pub struct PushSubscriptionKeys {
    pub p256dh: String,
    pub auth: String,
}

#[derive(Debug, Serialize)]
pub struct PushNotificationPayload {
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
    pub badge: Option<String>,
    pub data: Option<serde_json::Value>,
}

pub struct NotificationService {
    vapid_private_key: Option<String>,
    vapid_public_key: Option<String>,
    vapid_subject: Option<String>,
}

impl NotificationService {
    pub fn new(
        vapid_private_key: Option<String>,
        vapid_public_key: Option<String>,
        vapid_subject: Option<String>,
    ) -> Self {
        Self {
            vapid_private_key,
            vapid_public_key,
            vapid_subject,
        }
    }

    /// Subscribe a user to push notifications
    pub async fn subscribe(
        &self,
        pool: &DbPool,
        user_id: &str,
        subscription: CreatePushSubscription,
        user_agent: Option<String>,
    ) -> Result<PushSubscription, AppError> {
        let id = Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO push_subscriptions (id, user_id, endpoint, p256dh, auth, user_agent) 
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(user_id)
        .bind(&subscription.endpoint)
        .bind(&subscription.keys.p256dh)
        .bind(&subscription.keys.auth)
        .bind(user_agent)
        .execute(pool)
        .await?;

        Ok(PushSubscription {
            id,
            user_id: user_id.to_string(),
            endpoint: subscription.endpoint,
            p256dh: subscription.keys.p256dh,
            auth: subscription.keys.auth,
        })
    }

    /// Unsubscribe from push notifications
    pub async fn unsubscribe(
        &self,
        pool: &DbPool,
        user_id: &str,
        endpoint: &str,
    ) -> Result<(), AppError> {
        sqlx::query("DELETE FROM push_subscriptions WHERE user_id = ? AND endpoint = ?")
            .bind(user_id)
            .bind(endpoint)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// Get all subscriptions for a user
    pub async fn get_user_subscriptions(
        &self,
        pool: &DbPool,
        user_id: &str,
    ) -> Result<Vec<PushSubscription>, AppError> {
        let subscriptions = sqlx::query_as::<_, PushSubscription>(
            "SELECT id, user_id, endpoint, p256dh, auth FROM push_subscriptions WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(subscriptions)
    }

    /// Send a push notification to a user
    pub async fn send_notification(
        &self,
        pool: &DbPool,
        user_id: &str,
        payload: PushNotificationPayload,
        notification_type: &str,
    ) -> Result<(), AppError> {
        // Get user subscriptions
        let subscriptions = self.get_user_subscriptions(pool, user_id).await?;

        if subscriptions.is_empty() {
            return Ok(()); // No subscriptions, skip
        }

        // Check if VAPID keys are configured
        if self.vapid_private_key.is_none() || self.vapid_public_key.is_none() || self.vapid_subject.is_none() {
            tracing::warn!("VAPID keys not configured, skipping push notification");
            return Ok(());
        }

        // For now, just log that we would send a notification
        // Full web-push implementation requires complex VAPID signature generation
        // which varies by web-push library version
        tracing::info!("Would send push notification to user {} with {} subscriptions", user_id, subscriptions.len());
        
        // Save notification history
        self.save_notification_history(pool, user_id, notification_type, &payload)
            .await?;

        Ok(())
    }

    /// Send notification for a new match
    pub async fn send_match_notification(
        &self,
        pool: &DbPool,
        user_id: &str,
        match_name: &str,
    ) -> Result<(), AppError> {
        let payload = PushNotificationPayload {
            title: "New Match! 🎉".to_string(),
            body: format!("You matched with {}!", match_name),
            icon: Some("/icon-192.png".to_string()),
            badge: Some("/badge-72.png".to_string()),
            data: Some(serde_json::json!({
                "type": "match",
                "url": "/matches"
            })),
        };

        self.send_notification(pool, user_id, payload, "match").await
    }

    /// Send notification for a new message
    pub async fn send_message_notification(
        &self,
        pool: &DbPool,
        user_id: &str,
        sender_name: &str,
        message_preview: &str,
    ) -> Result<(), AppError> {
        let payload = PushNotificationPayload {
            title: format!("New message from {}", sender_name),
            body: message_preview.to_string(),
            icon: Some("/icon-192.png".to_string()),
            badge: Some("/badge-72.png".to_string()),
            data: Some(serde_json::json!({
                "type": "message",
                "url": "/chat"
            })),
        };

        self.send_notification(pool, user_id, payload, "message").await
    }

    /// Send notification for a new like
    pub async fn send_like_notification(
        &self,
        pool: &DbPool,
        user_id: &str,
    ) -> Result<(), AppError> {
        let payload = PushNotificationPayload {
            title: "Someone liked you! 💜".to_string(),
            body: "Check who's interested in your profile".to_string(),
            icon: Some("/icon-192.png".to_string()),
            badge: Some("/badge-72.png".to_string()),
            data: Some(serde_json::json!({
                "type": "like",
                "url": "/discover"
            })),
        };

        self.send_notification(pool, user_id, payload, "like").await
    }

    /// Save notification to history
    async fn save_notification_history(
        &self,
        pool: &DbPool,
        user_id: &str,
        notification_type: &str,
        payload: &PushNotificationPayload,
    ) -> Result<(), AppError> {
        let id = Uuid::new_v4().to_string();
        let data_json = serde_json::to_value(&payload.data)
            .ok();

        sqlx::query(
            "INSERT INTO notification_history (id, user_id, notification_type, title, body, data) 
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(user_id)
        .bind(notification_type)
        .bind(&payload.title)
        .bind(&payload.body)
        .bind(data_json)
        .execute(pool)
        .await?;

        Ok(())
    }
}
