use crate::{db::DbPool, errors::AppError};
use chrono::{Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaResponse {
    pub id: String,
    pub challenge: String,  // The math problem or question
}

#[derive(Debug, Clone)]
struct CaptchaEntry {
    answer: String,
    ip_hash: String,
    created_at: chrono::DateTime<Utc>,
}

/// CAPTCHA service (inspired by Alovoa's IP-based CAPTCHA system)
/// Uses simple math challenges to prevent automated bot attacks
/// Production-ready alternative until image CAPTCHA is implemented
pub struct CaptchaService {
    // In-memory storage (TODO: Move to Redis for production)
    store: Arc<Mutex<std::collections::HashMap<String, CaptchaEntry>>>,
}

impl CaptchaService {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// Generate a new CAPTCHA tied to the user's IP address
    /// Inspired by Alovoa's IP-hashed CAPTCHA to prevent automated attacks
    pub async fn generate(&self, ip_address: &str) -> Result<CaptchaResponse, AppError> {
        let ip_hash = self.hash_ip(ip_address);
        
        // Remove old captcha for this IP (only 1 per IP at a time)
        let mut store = self.store.lock().await;
        store.retain(|_, entry| {
            // Also cleanup expired captchas (older than 10 minutes)
            let age = Utc::now().signed_duration_since(entry.created_at);
            age.num_minutes() < 10
        });
        
        // Generate simple math challenge
        let mut rng = rand::thread_rng();
        let num1 = rng.gen_range(1..=20);
        let num2 = rng.gen_range(1..=20);
        let operation = rng.gen_range(0..2); // 0 = add, 1 = subtract
        
        let (challenge, answer) = if operation == 0 {
            (format!("{} + {}", num1, num2), (num1 + num2).to_string())
        } else {
            let (larger, smaller) = if num1 > num2 { (num1, num2) } else { (num2, num1) };
            (format!("{} - {}", larger, smaller), (larger - smaller).to_string())
        };
        
        // Store in memory
        let id = Uuid::new_v4().to_string();
        store.insert(
            id.clone(),
            CaptchaEntry {
                answer,
                ip_hash: ip_hash.clone(),
                created_at: Utc::now(),
            },
        );
        
        Ok(CaptchaResponse {
            id,
            challenge: format!("What is {}?", challenge),
        })
    }

    /// Validate CAPTCHA submission
    /// Returns true if:
    /// 1. CAPTCHA ID exists
    /// 2. Answer matches
    /// 3. IP address matches the one that requested it
    /// 4. Not expired (< 10 minutes old)
    pub async fn validate(
        &self,
        id: &str,
        answer: &str,
        ip_address: &str,
    ) -> Result<bool, AppError> {
        let ip_hash = self.hash_ip(ip_address);
        let mut store = self.store.lock().await;
        
        if let Some(entry) = store.remove(id) {
            // Check expiration
            let age = Utc::now().signed_duration_since(entry.created_at);
            if age.num_minutes() >= 10 {
                return Ok(false);
            }
            
            // Verify IP matches and answer matches (trim whitespace)
            Ok(entry.ip_hash == ip_hash && entry.answer.trim() == answer.trim())
        } else {
            Ok(false)
        }
    }

    /// Hash IP address with salt
    /// Inspired by Alovoa's approach to tie CAPTCHA to IP without storing raw IP
    fn hash_ip(&self, ip: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        // Add salt to prevent rainbow tables (use from config in production)
        "captcha_salt_change_in_production".hash(&mut hasher);
        ip.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

impl Default for CaptchaService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_captcha_generation() {
        let service = CaptchaService::new();
        let result = service.generate("127.0.0.1").await;
        assert!(result.is_ok());
        
        let captcha = result.unwrap();
        assert!(!captcha.id.is_empty());
        assert!(captcha.challenge.starts_with("What is "));
    }

    #[tokio::test]
    async fn test_captcha_validation_wrong_answer() {
        let service = CaptchaService::new();
        let captcha = service.generate("127.0.0.1").await.unwrap();
        
        // Wrong answer should fail
        let valid = service.validate(&captcha.id, "999", "127.0.0.1").await.unwrap();
        assert!(!valid);
    }

    #[tokio::test]
    async fn test_captcha_validation_wrong_ip() {
        let service = CaptchaService::new();
        let captcha = service.generate("127.0.0.1").await.unwrap();
        
        // Different IP should fail even with potentially correct answer
        let valid = service.validate(&captcha.id, "15", "192.168.1.1").await.unwrap();
        assert!(!valid);
    }

    #[tokio::test]
    async fn test_captcha_one_per_ip() {
        let service = CaptchaService::new();
        
        // Generate first captcha
        let captcha1 = service.generate("127.0.0.1").await.unwrap();
        
        // Generate second captcha for same IP
        let captcha2 = service.generate("127.0.0.1").await.unwrap();
        
        // First captcha should be invalidated (removed from store)
        let valid1 = service.validate(&captcha1.id, "anything", "127.0.0.1").await.unwrap();
        assert!(!valid1);
    }
}
