-- Chat Enhancements for WebSocket Real-Time Messaging
-- Run after 002_security_enhancements.sql

-- Add receiver_id to messages table for direct messaging
-- This allows messages outside of matches context (future feature)
ALTER TABLE messages
ADD COLUMN receiver_id CHAR(36) NULL AFTER sender_id,
ADD FOREIGN KEY (receiver_id) REFERENCES users(id) ON DELETE CASCADE,
ADD INDEX idx_receiver_messages (receiver_id, created_at);

-- Make match_id nullable to support direct messages
ALTER TABLE messages
MODIFY COLUMN match_id CHAR(36) NULL;

-- Add message type and metadata
ALTER TABLE messages
ADD COLUMN message_type VARCHAR(20) DEFAULT 'text' AFTER content,
ADD COLUMN metadata JSON NULL AFTER message_type;

-- Add typing indicators tracking table
CREATE TABLE IF NOT EXISTS typing_indicators (
    user_id CHAR(36) NOT NULL,
    conversation_id CHAR(36) NOT NULL,
    is_typing BOOLEAN DEFAULT TRUE,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    PRIMARY KEY (user_id, conversation_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    
    INDEX idx_conversation_typing (conversation_id, updated_at)
);

-- Add online status tracking
CREATE TABLE IF NOT EXISTS user_presence (
    user_id CHAR(36) PRIMARY KEY,
    status VARCHAR(20) DEFAULT 'offline',
    last_seen TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_status (status, last_seen)
);
