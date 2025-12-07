use crate::errors::AppError;
use redis::{aio::ConnectionManager, AsyncCommands};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

/// Redis cache service for caching expensive operations
#[derive(Clone)]
pub struct CacheService {
    client: ConnectionManager,
}

impl CacheService {
    pub async fn new(redis_url: &str) -> Result<Self, AppError> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| AppError::Internal(format!("Failed to connect to Redis: {}", e)))?;
        
        let connection_manager = ConnectionManager::new(client)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to create Redis connection manager: {}", e)))?;

        Ok(Self {
            client: connection_manager,
        })
    }

    /// Get a cached value by key
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, AppError> {
        let mut conn = self.client.clone();
        let value: Option<String> = conn
            .get(key)
            .await
            .map_err(|e| AppError::Internal(format!("Redis get error: {}", e)))?;

        match value {
            Some(json) => {
                let data = serde_json::from_str(&json)
                    .map_err(|e| AppError::Internal(format!("Failed to deserialize cache: {}", e)))?;
                Ok(Some(data))
            }
            None => Ok(None),
        }
    }

    /// Set a cached value with TTL
    pub async fn set<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> Result<(), AppError> {
        let mut conn = self.client.clone();
        let json = serde_json::to_string(value)
            .map_err(|e| AppError::Internal(format!("Failed to serialize cache: {}", e)))?;

        conn.set_ex(key, json, ttl.as_secs())
            .await
            .map_err(|e| AppError::Internal(format!("Redis set error: {}", e)))?;

        Ok(())
    }

    /// Delete a cached value
    pub async fn delete(&self, key: &str) -> Result<(), AppError> {
        let mut conn = self.client.clone();
        conn.del(key)
            .await
            .map_err(|e| AppError::Internal(format!("Redis delete error: {}", e)))?;

        Ok(())
    }

    /// Delete multiple cached values by pattern
    pub async fn delete_pattern(&self, pattern: &str) -> Result<(), AppError> {
        let mut conn = self.client.clone();
        
        // Get all keys matching the pattern
        let keys: Vec<String> = conn
            .keys(pattern)
            .await
            .map_err(|e| AppError::Internal(format!("Redis keys error: {}", e)))?;

        if !keys.is_empty() {
            conn.del(keys)
                .await
                .map_err(|e| AppError::Internal(format!("Redis delete pattern error: {}", e)))?;
        }

        Ok(())
    }

    /// Check if a key exists
    pub async fn exists(&self, key: &str) -> Result<bool, AppError> {
        let mut conn = self.client.clone();
        let exists: bool = conn
            .exists(key)
            .await
            .map_err(|e| AppError::Internal(format!("Redis exists error: {}", e)))?;

        Ok(exists)
    }

    /// Increment a counter with TTL
    pub async fn increment(&self, key: &str, ttl: Duration) -> Result<i64, AppError> {
        let mut conn = self.client.clone();
        
        // Increment the counter
        let value: i64 = conn
            .incr(key, 1)
            .await
            .map_err(|e| AppError::Internal(format!("Redis incr error: {}", e)))?;

        // Set TTL if this is the first increment
        if value == 1 {
            conn.expire(key, ttl.as_secs() as i64)
                .await
                .map_err(|e| AppError::Internal(format!("Redis expire error: {}", e)))?;
        }

        Ok(value)
    }

    /// Get time to live for a key
    pub async fn ttl(&self, key: &str) -> Result<i64, AppError> {
        let mut conn = self.client.clone();
        let ttl: i64 = conn
            .ttl(key)
            .await
            .map_err(|e| AppError::Internal(format!("Redis ttl error: {}", e)))?;

        Ok(ttl)
    }
}

// Cache key builders for consistency
pub mod keys {
    /// Cache key for user's top artists
    pub fn user_top_artists(user_id: &str, limit: usize) -> String {
        format!("user:{}:top_artists:{}", user_id, limit)
    }

    /// Cache key for compatibility score between two users
    pub fn compatibility(user1_id: &str, user2_id: &str) -> String {
        let mut ids = vec![user1_id, user2_id];
        ids.sort();
        format!("compatibility:{}:{}", ids[0], ids[1])
    }

    /// Cache key for Last.fm API data
    pub fn lastfm_api(endpoint: &str, params: &str) -> String {
        format!("lastfm:{}:{}", endpoint, params)
    }

    /// Cache key for user's music DNA profile
    pub fn music_dna(user_id: &str) -> String {
        format!("user:{}:music_dna", user_id)
    }

    /// Cache key for discover profiles
    pub fn discover_profiles(user_id: &str) -> String {
        format!("user:{}:discover_profiles", user_id)
    }

    /// Cache key for rate limiting
    pub fn rate_limit(identifier: &str, endpoint: &str) -> String {
        format!("rate_limit:{}:{}", identifier, endpoint)
    }
}
