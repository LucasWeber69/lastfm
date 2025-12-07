-- Push Notifications Subscriptions
-- Run after 005_events.sql

-- Store Web Push API subscriptions
CREATE TABLE IF NOT EXISTS push_subscriptions (
    id CHAR(36) PRIMARY KEY,
    user_id CHAR(36) NOT NULL,
    endpoint TEXT NOT NULL,
    p256dh VARCHAR(255) NOT NULL,
    auth VARCHAR(255) NOT NULL,
    user_agent TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_used_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    
    INDEX idx_user_subscriptions (user_id),
    INDEX idx_last_used (last_used_at)
);

-- Notification history for tracking
CREATE TABLE IF NOT EXISTS notification_history (
    id CHAR(36) PRIMARY KEY,
    user_id CHAR(36) NOT NULL,
    notification_type VARCHAR(50) NOT NULL,
    title VARCHAR(255),
    body TEXT,
    data JSON,
    sent_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    read_at TIMESTAMP NULL,
    clicked_at TIMESTAMP NULL,
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    
    INDEX idx_user_notifications (user_id, sent_at DESC),
    INDEX idx_notification_type (notification_type),
    INDEX idx_unread (user_id, read_at)
);
