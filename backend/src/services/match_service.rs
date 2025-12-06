use crate::{
    db::DbPool,
    errors::AppError,
    models::{Like, Match},
    services::compatibility_service::CompatibilityService,
};
use uuid::Uuid;

pub struct MatchService {
    compatibility_service: CompatibilityService,
}

impl MatchService {
    pub fn new(compatibility_service: CompatibilityService) -> Self {
        Self {
            compatibility_service,
        }
    }

    pub async fn create_like(
        &self,
        pool: &DbPool,
        from_user_id: &str,
        to_user_id: &str,
    ) -> Result<Option<Match>, AppError> {
        // Check if like already exists
        let existing = sqlx::query_as::<_, Like>(
            "SELECT * FROM likes WHERE from_user_id = ? AND to_user_id = ?"
        )
        .bind(from_user_id)
        .bind(to_user_id)
        .fetch_optional(pool)
        .await?;

        if existing.is_some() {
            return Ok(None); // Already liked
        }

        // Create the like
        let like = Like::new(from_user_id.to_string(), to_user_id.to_string());

        sqlx::query("INSERT INTO likes (id, from_user_id, to_user_id) VALUES (?, ?, ?)")
            .bind(&like.id)
            .bind(&like.from_user_id)
            .bind(&like.to_user_id)
            .execute(pool)
            .await?;

        // Check if there's a mutual like (match)
        let mutual_like = sqlx::query_as::<_, Like>(
            "SELECT * FROM likes WHERE from_user_id = ? AND to_user_id = ?"
        )
        .bind(to_user_id)
        .bind(from_user_id)
        .fetch_optional(pool)
        .await?;

        if let Some(_) = mutual_like {
            // Create match
            let compatibility_score = self
                .compatibility_service
                .calculate_compatibility(pool, from_user_id, to_user_id)
                .await?;

            let match_record = Match::new(
                from_user_id.to_string(),
                to_user_id.to_string(),
                Some(compatibility_score),
            );

            sqlx::query(
                "INSERT INTO matches (id, user1_id, user2_id, compatibility_score) VALUES (?, ?, ?, ?)"
            )
            .bind(&match_record.id)
            .bind(&match_record.user1_id)
            .bind(&match_record.user2_id)
            .bind(match_record.compatibility_score)
            .execute(pool)
            .await?;

            return Ok(Some(match_record));
        }

        Ok(None)
    }

    pub async fn get_user_matches(&self, pool: &DbPool, user_id: &str) -> Result<Vec<Match>, AppError> {
        let matches = sqlx::query_as::<_, Match>(
            "SELECT * FROM matches WHERE user1_id = ? OR user2_id = ? ORDER BY created_at DESC"
        )
        .bind(user_id)
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(matches)
    }

    pub async fn delete_match(&self, pool: &DbPool, match_id: &str, user_id: &str) -> Result<(), AppError> {
        // Verify that the user is part of this match
        let match_record = sqlx::query_as::<_, Match>(
            "SELECT * FROM matches WHERE id = ? AND (user1_id = ? OR user2_id = ?)"
        )
        .bind(match_id)
        .bind(user_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Match not found".to_string()))?;

        // Delete the match
        sqlx::query("DELETE FROM matches WHERE id = ?")
            .bind(match_id)
            .execute(pool)
            .await?;

        Ok(())
    }
}
