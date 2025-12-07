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

        // Use enhanced vector-based algorithm (inspired by Duolicious)
        let score = self.compute_vector_score(&user1_artists, &user2_artists);
        Ok(score)
    }

    /// Vector-based compatibility calculation (inspired by Duolicious)
    /// Uses cosine similarity between music preference vectors
    fn compute_vector_score(&self, user1_artists: &[Artist], user2_artists: &[Artist]) -> f64 {
        const VECTOR_SIZE: usize = 50;
        
        // Build all artists universe for vector space
        let mut all_artists = HashSet::new();
        for artist in user1_artists.iter().chain(user2_artists.iter()) {
            all_artists.insert(artist.name.clone());
        }
        
        // Create artist to index mapping
        let artist_to_idx: HashMap<String, usize> = all_artists
            .iter()
            .enumerate()
            .map(|(i, name)| (name.clone(), i))
            .collect();
        
        let vector_size = all_artists.len();
        let mut user1_vector = vec![0.0; vector_size];
        let mut user2_vector = vec![0.0; vector_size];
        
        // Build weighted vectors for user 1
        for (i, artist) in user1_artists.iter().enumerate().take(VECTOR_SIZE) {
            if let Some(&idx) = artist_to_idx.get(&artist.name) {
                // Position weight: earlier in list = more important
                let position_weight = 1.0 - (i as f64 / VECTOR_SIZE as f64);
                
                // Play count weight: normalize by total if available
                let play_weight = if artist.play_count > 0 {
                    (artist.play_count as f64).ln().max(1.0)
                } else {
                    1.0
                };
                
                // Popularity weight: less popular artists have more weight
                let popularity_weight = if artist.listeners > 0 {
                    1.0 / (artist.listeners as f64).log10().max(1.0)
                } else {
                    1.0
                };
                
                user1_vector[idx] = position_weight * play_weight * popularity_weight;
            }
        }
        
        // Build weighted vectors for user 2
        for (i, artist) in user2_artists.iter().enumerate().take(VECTOR_SIZE) {
            if let Some(&idx) = artist_to_idx.get(&artist.name) {
                let position_weight = 1.0 - (i as f64 / VECTOR_SIZE as f64);
                let play_weight = if artist.play_count > 0 {
                    (artist.play_count as f64).ln().max(1.0)
                } else {
                    1.0
                };
                let popularity_weight = if artist.listeners > 0 {
                    1.0 / (artist.listeners as f64).log10().max(1.0)
                } else {
                    1.0
                };
                
                user2_vector[idx] = position_weight * play_weight * popularity_weight;
            }
        }
        
        // Normalize vectors (L2 normalization - like Duolicious does)
        let norm1 = user1_vector.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm2 = user2_vector.iter().map(|x| x * x).sum::<f64>().sqrt();
        
        if norm1 == 0.0 || norm2 == 0.0 {
            return 0.0;
        }
        
        for v in &mut user1_vector {
            *v /= norm1;
        }
        for v in &mut user2_vector {
            *v /= norm2;
        }
        
        // Calculate cosine similarity (dot product of normalized vectors)
        let dot_product: f64 = user1_vector
            .iter()
            .zip(user2_vector.iter())
            .map(|(a, b)| a * b)
            .sum();
        
        // Convert cosine similarity [-1, 1] to percentage [0, 99]
        // Cosine similarity of 1.0 (identical vectors) = 99%
        // Cosine similarity of 0.0 (orthogonal) = 49.5%
        // Cosine similarity of -1.0 (opposite) = 0%
        let similarity = (dot_product + 1.0) / 2.0; // Maps [-1,1] to [0,1]
        (similarity * 99.0).max(0.0).min(99.0)
    }

    /// Original compatibility calculation (kept for comparison)
    #[allow(dead_code)]
    fn compute_score_original(&self, user1_artists: &[Artist], user2_artists: &[Artist]) -> f64 {
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

        // Calculate weighted score
        let mut weighted_score = 0.0;
        
        for artist_name in &common_artists {
            if let (Some(artist1), Some(artist2)) = (user1_map.get(*artist_name), user2_map.get(*artist_name)) {
                let avg_listeners = ((artist1.listeners + artist2.listeners) as f64) / 2.0;
                let popularity_weight = if avg_listeners > 0.0 {
                    1.0 / avg_listeners.log10().max(1.0)
                } else {
                    1.0
                };

                let pos1 = user1_artists.iter().position(|a| &a.name == *artist_name).unwrap_or(50) as f64;
                let pos2 = user2_artists.iter().position(|a| &a.name == *artist_name).unwrap_or(50) as f64;
                let position_diff = (pos1 - pos2).abs();
                let position_weight = 1.0 / (1.0 + position_diff / 10.0);

                weighted_score += popularity_weight * position_weight;
            }
        }

        let common_count_score = (common_artists.len() as f64 / 10.0) * 30.0;
        let weighted_normalized = (weighted_score / common_artists.len() as f64) * 70.0;
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

