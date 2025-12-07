-- Gamification System (Achievements, Badges, Stats)
-- Run after 003_chat_enhancements.sql

-- Achievements definitions
CREATE TABLE IF NOT EXISTS achievements (
    id CHAR(36) PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT NOT NULL,
    icon VARCHAR(100),
    points INT DEFAULT 0,
    category VARCHAR(50),
    requirement_type VARCHAR(50) NOT NULL,
    requirement_value INT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    INDEX idx_category (category),
    INDEX idx_requirement (requirement_type)
);

-- User achievements (unlocked badges)
CREATE TABLE IF NOT EXISTS user_achievements (
    id CHAR(36) PRIMARY KEY,
    user_id CHAR(36) NOT NULL,
    achievement_id CHAR(36) NOT NULL,
    unlocked_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    progress INT DEFAULT 100,
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (achievement_id) REFERENCES achievements(id) ON DELETE CASCADE,
    
    UNIQUE KEY unique_user_achievement (user_id, achievement_id),
    INDEX idx_user_achievements (user_id, unlocked_at DESC)
);

-- User statistics for gamification
CREATE TABLE IF NOT EXISTS user_stats (
    user_id CHAR(36) PRIMARY KEY,
    total_matches INT DEFAULT 0,
    total_likes_sent INT DEFAULT 0,
    total_likes_received INT DEFAULT 0,
    messages_sent INT DEFAULT 0,
    messages_received INT DEFAULT 0,
    profile_views INT DEFAULT 0,
    current_streak_days INT DEFAULT 0,
    longest_streak_days INT DEFAULT 0,
    last_message_date DATE NULL,
    total_points INT DEFAULT 0,
    level INT DEFAULT 1,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_level (level DESC),
    INDEX idx_points (total_points DESC)
);

-- Insert default achievements
INSERT INTO achievements (id, name, description, icon, points, category, requirement_type, requirement_value) VALUES
(UUID(), 'First Match', 'Get your first match', '🎉', 10, 'social', 'matches', 1),
(UUID(), 'Music Soulmate', 'Match with someone with 90%+ compatibility', '💜', 50, 'compatibility', 'high_compatibility', 90),
(UUID(), 'Conversation Starter', 'Send your first message', '💬', 5, 'chat', 'messages_sent', 1),
(UUID(), '7 Day Streak', 'Chat for 7 days in a row', '🔥', 30, 'engagement', 'streak_days', 7),
(UUID(), 'Indie Explorer', 'Have 50%+ niche artists in your top artists', '🎸', 25, 'music', 'niche_percentage', 50),
(UUID(), 'Genre Master', 'Have scrobbles in 10+ different genres', '🎵', 20, 'music', 'genre_count', 10),
(UUID(), 'Social Butterfly', 'Get 10 matches', '🦋', 40, 'social', 'matches', 10),
(UUID(), 'Active Chatter', 'Send 100 messages', '📱', 35, 'chat', 'messages_sent', 100),
(UUID(), 'Popular Profile', 'Receive 50 likes', '⭐', 45, 'social', 'likes_received', 50),
(UUID(), 'Dedicated User', 'Use the app for 30 days', '📅', 60, 'engagement', 'days_active', 30);
