-- Users table
CREATE TABLE users (
    id CHAR(36) PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    name VARCHAR(100) NOT NULL,
    bio TEXT,
    birth_date DATE,
    gender VARCHAR(20),
    looking_for VARCHAR(50),
    lastfm_username VARCHAR(100),
    lastfm_connected_at TIMESTAMP NULL,
    latitude DECIMAL(10, 8),
    longitude DECIMAL(11, 8),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_lastfm_username (lastfm_username),
    INDEX idx_location (latitude, longitude)
);

-- Photos table
CREATE TABLE photos (
    id CHAR(36) PRIMARY KEY,
    user_id CHAR(36) NOT NULL,
    url VARCHAR(500) NOT NULL,
    position INT DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_user_photos (user_id, position)
);

-- Cached scrobbles/listening data from Last.fm
CREATE TABLE scrobbles_cache (
    id CHAR(36) PRIMARY KEY,
    user_id CHAR(36) NOT NULL,
    artist_name VARCHAR(255) NOT NULL,
    artist_mbid VARCHAR(36),
    track_name VARCHAR(255),
    play_count INT DEFAULT 0,
    listeners INT DEFAULT 0,
    period VARCHAR(20) DEFAULT '6month',
    last_synced_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_user_artist (user_id, artist_name),
    INDEX idx_user_period (user_id, period)
);

-- Likes (when someone swipes right)
CREATE TABLE likes (
    id CHAR(36) PRIMARY KEY,
    from_user_id CHAR(36) NOT NULL,
    to_user_id CHAR(36) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (from_user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (to_user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE KEY unique_like (from_user_id, to_user_id),
    INDEX idx_to_user (to_user_id)
);

-- Matches (mutual likes)
CREATE TABLE matches (
    id CHAR(36) PRIMARY KEY,
    user1_id CHAR(36) NOT NULL,
    user2_id CHAR(36) NOT NULL,
    compatibility_score DECIMAL(5, 2),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user1_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (user2_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE KEY unique_match (user1_id, user2_id),
    INDEX idx_user1 (user1_id),
    INDEX idx_user2 (user2_id)
);

-- Messages
CREATE TABLE messages (
    id CHAR(36) PRIMARY KEY,
    match_id CHAR(36) NOT NULL,
    sender_id CHAR(36) NOT NULL,
    content TEXT NOT NULL,
    read_at TIMESTAMP NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (match_id) REFERENCES matches(id) ON DELETE CASCADE,
    FOREIGN KEY (sender_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_match_messages (match_id, created_at)
);

-- Blocks/Hidden profiles
CREATE TABLE blocks (
    id CHAR(36) PRIMARY KEY,
    blocker_id CHAR(36) NOT NULL,
    blocked_id CHAR(36) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (blocker_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (blocked_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE KEY unique_block (blocker_id, blocked_id),
    INDEX idx_blocker (blocker_id)
);
