-- Events and Shows Integration
-- Run after 004_achievements.sql

-- Store event interests for discovering common events
CREATE TABLE IF NOT EXISTS event_interests (
    id CHAR(36) PRIMARY KEY,
    user_id CHAR(36) NOT NULL,
    event_id VARCHAR(100) NOT NULL,
    event_name VARCHAR(255) NOT NULL,
    artist_name VARCHAR(255),
    venue_name VARCHAR(255),
    event_date TIMESTAMP,
    city VARCHAR(100),
    country VARCHAR(100),
    external_url VARCHAR(500),
    interested_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    
    UNIQUE KEY unique_user_event (user_id, event_id),
    INDEX idx_user_events (user_id, event_date),
    INDEX idx_event_date (event_date),
    INDEX idx_artist (artist_name),
    INDEX idx_location (city, country)
);

-- Cache external event data
CREATE TABLE IF NOT EXISTS events_cache (
    event_id VARCHAR(100) PRIMARY KEY,
    event_name VARCHAR(255) NOT NULL,
    artist_names TEXT,
    venue_name VARCHAR(255),
    event_date TIMESTAMP,
    city VARCHAR(100),
    country VARCHAR(100),
    latitude DECIMAL(10, 8),
    longitude DECIMAL(11, 8),
    external_url VARCHAR(500),
    image_url VARCHAR(500),
    ticket_url VARCHAR(500),
    genre_tags JSON,
    cached_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP,
    
    INDEX idx_event_location (city, country, event_date),
    INDEX idx_event_date (event_date),
    INDEX idx_cache_expiry (expires_at)
);
