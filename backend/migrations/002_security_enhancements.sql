-- Security and Anti-Abuse Enhancements (Inspired by Duolicious)
-- Run after 001_initial_schema.sql

-- Add IP address tracking for security
ALTER TABLE users 
ADD COLUMN sign_up_ip_address VARCHAR(45),
ADD COLUMN last_login_ip VARCHAR(45),
ADD COLUMN last_login_at TIMESTAMP NULL;

-- Ban system (email + IP based)
CREATE TABLE IF NOT EXISTS banned_users (
    id CHAR(36) PRIMARY KEY,
    email VARCHAR(255),
    normalized_email VARCHAR(255),
    ip_address VARCHAR(45),
    reason TEXT NOT NULL,
    report_reasons TEXT,
    banned_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP NULL,
    permanent BOOLEAN DEFAULT FALSE,
    
    INDEX idx_banned_email (normalized_email),
    INDEX idx_banned_ip (ip_address),
    INDEX idx_banned_expires (expires_at)
);

-- Report system
CREATE TABLE IF NOT EXISTS reports (
    id CHAR(36) PRIMARY KEY,
    reporter_id CHAR(36) NOT NULL,
    reported_id CHAR(36) NOT NULL,
    reason VARCHAR(50) NOT NULL,
    details TEXT,
    resolved BOOLEAN DEFAULT FALSE,
    resolved_by CHAR(36),
    resolved_at TIMESTAMP NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (reporter_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (reported_id) REFERENCES users(id) ON DELETE CASCADE,
    
    INDEX idx_reports_reported (reported_id, created_at),
    INDEX idx_reports_resolved (resolved),
    
    UNIQUE KEY unique_report (reporter_id, reported_id)
);

-- Discover/search result cache (performance optimization)
CREATE TABLE IF NOT EXISTS discover_cache (
    user_id CHAR(36) NOT NULL,
    prospect_id CHAR(36) NOT NULL,
    compatibility_score DECIMAL(5, 2) NOT NULL,
    common_artists_count INT DEFAULT 0,
    cached_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    PRIMARY KEY (user_id, prospect_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (prospect_id) REFERENCES users(id) ON DELETE CASCADE,
    
    INDEX idx_cache_user_score (user_id, compatibility_score DESC),
    INDEX idx_cache_expiry (cached_at)
);

-- Add bot detection fields
ALTER TABLE users
ADD COLUMN bot_score DECIMAL(5, 2) DEFAULT 100.0,
ADD COLUMN verified BOOLEAN DEFAULT FALSE,
ADD COLUMN verification_level SMALLINT DEFAULT 0;

-- Indexes for performance (inspired by Duolicious multi-pass filtering)
CREATE INDEX idx_users_location ON users(latitude, longitude);
CREATE INDEX idx_users_gender_looking ON users(gender, looking_for);
CREATE INDEX idx_users_active ON users(created_at, updated_at);
CREATE INDEX idx_scrobbles_sync ON scrobbles_cache(user_id, last_synced_at);
CREATE INDEX idx_likes_created ON likes(created_at);
CREATE INDEX idx_matches_created ON matches(created_at);

-- Session tracking (optional - for database-backed sessions)
CREATE TABLE IF NOT EXISTS sessions (
    session_token_hash VARCHAR(128) PRIMARY KEY,
    user_id CHAR(36) NOT NULL,
    ip_address VARCHAR(45),
    user_agent TEXT,
    signed_in BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP NOT NULL,
    last_activity TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    
    INDEX idx_sessions_user (user_id),
    INDEX idx_sessions_expiry (expires_at)
);

-- Cleanup: Remove expired bans and old cache entries
-- This can be run as a cron job or scheduled task

DELIMITER //
CREATE PROCEDURE cleanup_expired_data()
BEGIN
    -- Remove expired bans
    DELETE FROM banned_users 
    WHERE expires_at IS NOT NULL 
    AND expires_at < NOW() 
    AND permanent = FALSE;
    
    -- Remove old cache entries (older than 1 day)
    DELETE FROM discover_cache 
    WHERE cached_at < DATE_SUB(NOW(), INTERVAL 1 DAY);
    
    -- Remove expired sessions
    DELETE FROM sessions 
    WHERE expires_at < NOW();
END //
DELIMITER ;
