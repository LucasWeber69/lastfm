use crate::{
    db::DbPool,
    errors::AppError,
    models::Artist,
    services::lastfm_service::LastFmService,
};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub struct CompatibilityService {
    lastfm_service: Arc<LastFmService>,
}

impl CompatibilityService {
    pub fn new(lastfm_service: Arc<LastFmService>) -> Self {
        Self { lastfm_service }
    }

    pub async fn calculate_compatibility(
        &self,
        pool: &DbPool,
        user1_id: &str,
        user2_id: &str,
    ) -> Result<f64, AppError> {
        // Get top 50 artists for both users
        let user1_artists = self.lastfm_service.get_user_top_artists(pool, user1_id, 50).await?;
        let user2_artists = self.lastfm_service.get_user_top_artists(pool, user2_id, 50).await?;

        if user1_artists.is_empty() || user2_artists.is_empty() {
            return Ok(0.0);
        }

        let score = self.compute_score(&user1_artists, &user2_artists);
        Ok(score)
    }

    fn compute_score(&self, user1_artists: &[Artist], user2_artists: &[Artist]) -> f64 {
        // Create HashSets for quick lookup
        let user1_set: HashSet<_> = user1_artists.iter().map(|a| a.name.clone()).collect();
        let user2_set: HashSet<_> = user2_artists.iter().map(|a| a.name.clone()).collect();

        // Find common artists
        let common_artists: Vec<_> = user1_set.intersection(&user2_set).collect();

        if common_artists.is_empty() {
            return 0.0;
        }

        // Create lookup maps for artist data
        let user1_map: HashMap<_, _> = user1_artists
            .iter()
            .map(|a| (a.name.clone(), a))
            .collect();

        let user2_map: HashMap<_, _> = user2_artists
            .iter()
            .map(|a| (a.name.clone(), a))
            .collect();

        // Calculate weighted score based on:
        // 1. Common artists count (base score)
        // 2. Popularity weight (less popular = higher weight)
        // 3. Position similarity (closer ranks = higher weight)
        
        let mut weighted_score = 0.0;
        
        for artist_name in &common_artists {
            if let (Some(artist1), Some(artist2)) = (user1_map.get(*artist_name), user2_map.get(*artist_name)) {
                // Popularity weight: inverse log of listeners count
                // Artists with fewer listeners get higher weight (more niche = better match)
                let avg_listeners = ((artist1.listeners + artist2.listeners) as f64) / 2.0;
                let popularity_weight = if avg_listeners > 0.0 {
                    1.0 / avg_listeners.log10().max(1.0)
                } else {
                    1.0
                };

                // Position weight: how similar are the rankings
                // Find positions in the arrays
                let pos1 = user1_artists.iter().position(|a| &a.name == *artist_name).unwrap_or(50) as f64;
                let pos2 = user2_artists.iter().position(|a| &a.name == *artist_name).unwrap_or(50) as f64;
                let position_diff = (pos1 - pos2).abs();
                let position_weight = 1.0 / (1.0 + position_diff / 10.0); // Normalize position difference

                // Combine weights
                weighted_score += popularity_weight * position_weight;
            }
        }

        // Base score from number of common artists
        let common_count_score = (common_artists.len() as f64 / 10.0) * 30.0; // Up to 30 points for common count
        
        // Weighted score normalized
        let weighted_normalized = (weighted_score / common_artists.len() as f64) * 70.0; // Up to 70 points for quality

        // Combine and cap at 100
        let total_score = (common_count_score + weighted_normalized).min(100.0);

        total_score
    }

    pub fn get_common_artists(
        &self,
        user1_artists: &[Artist],
        user2_artists: &[Artist],
        limit: usize,
    ) -> Vec<String> {
        let user1_set: HashSet<_> = user1_artists.iter().map(|a| a.name.clone()).collect();
        let user2_set: HashSet<_> = user2_artists.iter().map(|a| a.name.clone()).collect();

        let common: Vec<_> = user1_set
            .intersection(&user2_set)
            .take(limit)
            .cloned()
            .collect();

        common
    }
}
