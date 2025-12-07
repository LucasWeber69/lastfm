use crate::{db::DbPool, errors::AppError};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    pub points: i32,
    pub category: Option<String>,
    pub requirement_type: String,
    pub requirement_value: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserAchievement {
    pub id: String,
    pub user_id: String,
    pub achievement_id: String,
    pub unlocked_at: NaiveDateTime,
    pub progress: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementWithProgress {
    #[serde(flatten)]
    pub achievement: Achievement,
    pub unlocked: bool,
    pub unlocked_at: Option<NaiveDateTime>,
    pub progress: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserStats {
    pub user_id: String,
    pub total_matches: i32,
    pub total_likes_sent: i32,
    pub total_likes_received: i32,
    pub messages_sent: i32,
    pub messages_received: i32,
    pub profile_views: i32,
    pub current_streak_days: i32,
    pub longest_streak_days: i32,
    pub last_message_date: Option<chrono::NaiveDate>,
    pub total_points: i32,
    pub level: i32,
}

pub struct AchievementService;

impl AchievementService {
    /// Get all achievements with user progress
    pub async fn get_user_achievements(
        pool: &DbPool,
        user_id: &str,
    ) -> Result<Vec<AchievementWithProgress>, AppError> {
        let achievements = sqlx::query_as::<_, Achievement>(
            "SELECT * FROM achievements ORDER BY category, points",
        )
        .fetch_all(pool)
        .await?;

        let user_achievements: Vec<UserAchievement> = sqlx::query_as(
            "SELECT * FROM user_achievements WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let stats = Self::get_user_stats(pool, user_id).await?;

        let mut results = Vec::new();
        for achievement in achievements {
            let user_achievement = user_achievements
                .iter()
                .find(|ua| ua.achievement_id == achievement.id);

            let (unlocked, unlocked_at, progress) = if let Some(ua) = user_achievement {
                (true, Some(ua.unlocked_at), ua.progress)
            } else {
                let progress = Self::calculate_progress(&achievement, &stats);
                (false, None, progress)
            };

            results.push(AchievementWithProgress {
                achievement,
                unlocked,
                unlocked_at,
                progress,
            });
        }

        Ok(results)
    }

    /// Get user statistics
    pub async fn get_user_stats(
        pool: &DbPool,
        user_id: &str,
    ) -> Result<UserStats, AppError> {
        let stats: Option<UserStats> = sqlx::query_as(
            "SELECT * FROM user_stats WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        match stats {
            Some(s) => Ok(s),
            None => {
                // Create default stats
                Self::initialize_user_stats(pool, user_id).await?;
                Self::get_user_stats(pool, user_id).await
            }
        }
    }

    /// Initialize user stats
    pub async fn initialize_user_stats(
        pool: &DbPool,
        user_id: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            "INSERT IGNORE INTO user_stats (user_id) VALUES (?)",
        )
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update user stats on match
    pub async fn on_match_created(
        pool: &DbPool,
        user_id: &str,
        compatibility_score: f64,
    ) -> Result<Vec<Achievement>, AppError> {
        // Update stats
        sqlx::query(
            "INSERT INTO user_stats (user_id, total_matches) VALUES (?, 1)
             ON DUPLICATE KEY UPDATE total_matches = total_matches + 1",
        )
        .bind(user_id)
        .execute(pool)
        .await?;

        // Check for achievements
        Self::check_and_unlock_achievements(pool, user_id, compatibility_score).await
    }

    /// Update user stats on like sent
    pub async fn on_like_sent(
        pool: &DbPool,
        user_id: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO user_stats (user_id, total_likes_sent) VALUES (?, 1)
             ON DUPLICATE KEY UPDATE total_likes_sent = total_likes_sent + 1",
        )
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update user stats on like received
    pub async fn on_like_received(
        pool: &DbPool,
        user_id: &str,
    ) -> Result<Vec<Achievement>, AppError> {
        sqlx::query(
            "INSERT INTO user_stats (user_id, total_likes_received) VALUES (?, 1)
             ON DUPLICATE KEY UPDATE total_likes_received = total_likes_received + 1",
        )
        .bind(user_id)
        .execute(pool)
        .await?;

        Self::check_and_unlock_achievements(pool, user_id, 0.0).await
    }

    /// Update user stats on message sent
    pub async fn on_message_sent(
        pool: &DbPool,
        user_id: &str,
    ) -> Result<Vec<Achievement>, AppError> {
        let today = chrono::Utc::now().date_naive();
        
        sqlx::query(
            "INSERT INTO user_stats (user_id, messages_sent, last_message_date) VALUES (?, 1, ?)
             ON DUPLICATE KEY UPDATE 
                messages_sent = messages_sent + 1,
                last_message_date = ?",
        )
        .bind(user_id)
        .bind(today)
        .bind(today)
        .execute(pool)
        .await?;

        // Update streak
        Self::update_streak(pool, user_id).await?;

        Self::check_and_unlock_achievements(pool, user_id, 0.0).await
    }

    /// Update messaging streak
    async fn update_streak(
        pool: &DbPool,
        user_id: &str,
    ) -> Result<(), AppError> {
        let stats = Self::get_user_stats(pool, user_id).await?;
        let today = chrono::Utc::now().date_naive();

        if let Some(last_date) = stats.last_message_date {
            let days_diff = (today - last_date).num_days();
            
            if days_diff == 1 {
                // Consecutive day - increment streak
                let new_streak = stats.current_streak_days + 1;
                let new_longest = new_streak.max(stats.longest_streak_days);
                
                sqlx::query(
                    "UPDATE user_stats 
                     SET current_streak_days = ?, longest_streak_days = ?
                     WHERE user_id = ?",
                )
                .bind(new_streak)
                .bind(new_longest)
                .bind(user_id)
                .execute(pool)
                .await?;
            } else if days_diff > 1 {
                // Streak broken - reset
                sqlx::query(
                    "UPDATE user_stats SET current_streak_days = 1 WHERE user_id = ?",
                )
                .bind(user_id)
                .execute(pool)
                .await?;
            }
        } else {
            // First message
            sqlx::query(
                "UPDATE user_stats SET current_streak_days = 1 WHERE user_id = ?",
            )
            .bind(user_id)
            .execute(pool)
            .await?;
        }

        Ok(())
    }

    /// Check and unlock achievements based on current stats
    async fn check_and_unlock_achievements(
        pool: &DbPool,
        user_id: &str,
        compatibility_score: f64,
    ) -> Result<Vec<Achievement>, AppError> {
        let stats = Self::get_user_stats(pool, user_id).await?;
        let achievements = sqlx::query_as::<_, Achievement>(
            "SELECT * FROM achievements",
        )
        .fetch_all(pool)
        .await?;

        let mut unlocked = Vec::new();

        for achievement in achievements {
            // Check if already unlocked
            let already_unlocked: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM user_achievements 
                 WHERE user_id = ? AND achievement_id = ?",
            )
            .bind(user_id)
            .bind(&achievement.id)
            .fetch_one(pool)
            .await?;

            if already_unlocked > 0 {
                continue;
            }

            // Check if requirements are met
            let should_unlock = match achievement.requirement_type.as_str() {
                "matches" => stats.total_matches >= achievement.requirement_value,
                "high_compatibility" => compatibility_score >= achievement.requirement_value as f64,
                "messages_sent" => stats.messages_sent >= achievement.requirement_value,
                "streak_days" => stats.current_streak_days >= achievement.requirement_value,
                "likes_received" => stats.total_likes_received >= achievement.requirement_value,
                _ => false,
            };

            if should_unlock {
                // Unlock achievement
                let ua_id = Uuid::new_v4().to_string();
                sqlx::query(
                    "INSERT INTO user_achievements (id, user_id, achievement_id) 
                     VALUES (?, ?, ?)",
                )
                .bind(ua_id)
                .bind(user_id)
                .bind(&achievement.id)
                .execute(pool)
                .await?;

                // Add points
                sqlx::query(
                    "UPDATE user_stats SET total_points = total_points + ? WHERE user_id = ?",
                )
                .bind(achievement.points)
                .bind(user_id)
                .execute(pool)
                .await?;

                unlocked.push(achievement);
            }
        }

        // Update level based on points
        Self::update_level(pool, user_id).await?;

        Ok(unlocked)
    }

    /// Calculate progress towards an achievement
    fn calculate_progress(achievement: &Achievement, stats: &UserStats) -> i32 {
        let current = match achievement.requirement_type.as_str() {
            "matches" => stats.total_matches,
            "messages_sent" => stats.messages_sent,
            "streak_days" => stats.current_streak_days,
            "likes_received" => stats.total_likes_received,
            _ => 0,
        };

        let progress = (current as f32 / achievement.requirement_value as f32 * 100.0) as i32;
        progress.min(100)
    }

    /// Update user level based on total points
    async fn update_level(
        pool: &DbPool,
        user_id: &str,
    ) -> Result<(), AppError> {
        let stats = Self::get_user_stats(pool, user_id).await?;
        
        // Simple leveling: level = points / 100 + 1
        let new_level = (stats.total_points / 100) + 1;

        sqlx::query("UPDATE user_stats SET level = ? WHERE user_id = ?")
            .bind(new_level)
            .bind(user_id)
            .execute(pool)
            .await?;

        Ok(())
    }
}
