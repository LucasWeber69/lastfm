use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Simple in-memory rate limiter
/// In production, use Redis for distributed rate limiting
#[derive(Clone)]
pub struct RateLimiter {
    // IP -> (request_count, window_start)
    requests: Arc<Mutex<HashMap<String, (u32, Instant)>>>,
    max_requests: u32,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window,
        }
    }

    pub fn check_rate_limit(&self, ip: &str) -> bool {
        let mut requests = self.requests.lock().unwrap();
        let now = Instant::now();

        if let Some((count, start)) = requests.get_mut(ip) {
            if now.duration_since(*start) > self.window {
                // Reset window
                *count = 1;
                *start = now;
                true
            } else if *count < self.max_requests {
                *count += 1;
                true
            } else {
                false
            }
        } else {
            requests.insert(ip.to_string(), (1, now));
            true
        }
    }

    pub fn cleanup_old_entries(&self) {
        let mut requests = self.requests.lock().unwrap();
        let now = Instant::now();
        requests.retain(|_, (_, start)| now.duration_since(*start) <= self.window);
    }
}

pub async fn rate_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get client IP from headers or connection
    let _ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .split(',')
        .next()
        .unwrap_or("unknown")
        .trim();

    // TODO: Implement actual rate limiting with Redis
    // For now, just pass through
    // In production, you would:
    // 1. Check Redis for rate limit
    // 2. Increment counter
    // 3. Return 429 if exceeded

    Ok(next.run(request).await)
}
