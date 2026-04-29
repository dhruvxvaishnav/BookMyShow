use std::collections::VecDeque;
use std::time::{Duration, Instant};
use std::sync::Arc;

use tokio::sync::RwLock;

/// Maximum tracked timestamps per rate-limit entry.
const WINDOW_SIZE: usize = 120;

/// A rate-limit bucket for a single key.
#[derive(Debug, Clone)]
struct RateLimitEntry {
    /// Rolling window of recent request timestamps.
    timestamps: VecDeque<Instant>,
}

impl RateLimitEntry {
    fn new() -> Self {
        Self {
            timestamps: VecDeque::with_capacity(WINDOW_SIZE),
        }
    }

    /// Add a timestamp and prune old entries outside the 1-minute window.
    fn record(&mut self, now: Instant) {
        self.timestamps.push_back(now);
        self.prune(now);
    }

    /// Returns the count of requests in the current 1-minute window.
    fn count(&self) -> usize {
        let cutoff = Instant::now() - Duration::from_secs(60);
        self.timestamps.iter().filter(|&&t| t > cutoff).count()
    }

    fn prune(&mut self, now: Instant) {
        let cutoff = now - Duration::from_secs(60);
        while self.timestamps.front().map_or(false, |&t| t <= cutoff) {
            self.timestamps.pop_front();
        }
    }
}

/// Thread-safe in-memory rate limiter.
#[derive(Clone)]
pub struct RateLimiter {
    store: Arc<RwLock<std::collections::HashMap<String, RateLimitEntry>>>,
    /// How often to prune the store (every 60s).
    _cleanup_interval: Duration,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(std::collections::HashMap::new())),
            _cleanup_interval: Duration::from_secs(60),
        }
    }

    /// Check and record a request. Returns Ok(remaining) on pass, Err on limit exceeded.
    pub async fn check(&self, key: &str, limit: usize) -> Result<usize, ()> {
        let now = Instant::now();

        let count = {
            let mut store = self.store.write().await;
            let entry = store.entry(key.to_string()).or_insert_with(RateLimitEntry::new);
            entry.record(now);
            entry.count()
        };

        if count > limit {
            Err(())
        } else {
            Ok(limit - count)
        }
    }

    /// Get current count without recording.
    pub async fn current_count(&self, key: &str) -> usize {
        let store = self.store.read().await;
        store.get(key).map(|e| e.count()).unwrap_or(0)
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_allows_under_limit() {
        let limiter = RateLimiter::new();
        for _ in 0..5 {
            let result = limiter.check("test-key", 10).await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_over_limit() {
        let limiter = RateLimiter::new();
        // First 2 requests pass (limit = 2)
        limiter.check("burst-key", 2).await.unwrap();
        limiter.check("burst-key", 2).await.unwrap();
        // Third request is blocked
        let result = limiter.check("burst-key", 2).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rate_limiter_different_keys_independent() {
        let limiter = RateLimiter::new();
        limiter.check("key-a", 1).await.unwrap();
        let result = limiter.check("key-b", 1).await;
        assert!(result.is_ok());
    }
}