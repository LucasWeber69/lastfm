use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use std::time::Duration;
use crate::services::CacheService;

/// Rate limiter using Redis
#[derive(Clone)]
pub struct RateLimiter {
    cache: Arc<CacheService>,
    max_requests: u32,
    window: Duration,
}

impl RateLimiter {
    pub fn new(cache: Arc<CacheService>, max_requests: u32, window: Duration) -> Self {
        Self {
            cache,
            max_requests,
            window,
        }
    }

    pub async fn check_rate_limit(&self, identifier: &str, endpoint: &str) -> Result<(bool, u32, i64), String> {
        let key = format!("rate_limit:{}:{}", identifier, endpoint);
        
        match self.cache.increment(&key, self.window).await {
            Ok(count) => {
                let remaining = if count as u32 <= self.max_requests {
                    self.max_requests - count as u32
                } else {
                    0
                };
                
                let ttl = self.cache.ttl(&key).await.unwrap_or(self.window.as_secs() as i64);
                let allowed = count as u32 <= self.max_requests;
                
                Ok((allowed, remaining, ttl))
            }
            Err(e) => {
                tracing::error!("Rate limit check failed: {}", e);
                // On Redis failure, allow the request to prevent blocking legitimate traffic
                Ok((true, self.max_requests, self.window.as_secs() as i64))
            }
        }
    }
}

pub async fn rate_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get client IP from headers or connection
    let _ip = get_client_ip(request.headers());

    // For now, just pass through
    // In a full implementation, you would:
    // 1. Extract the rate limiter from the state
    // 2. Determine the endpoint and rate limit rules
    // 3. Check the rate limit
    // 4. Add rate limit headers to response
    // 5. Return 429 if exceeded

    Ok(next.run(request).await)
}

fn get_client_ip(headers: &HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("unknown").trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Create a rate limit middleware with custom limits
pub fn create_rate_limiter(
    cache: Arc<CacheService>,
    max_requests: u32,
    window_secs: u64,
) -> RateLimiter {
    RateLimiter::new(cache, max_requests, Duration::from_secs(window_secs))
}
