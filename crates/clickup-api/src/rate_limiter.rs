use std::time::{Duration, Instant};

/// Tracks ClickUp API rate-limit state and pauses requests when exhausted.
pub struct RateLimiter {
    remaining: u64,
    reset_at: Instant,
}

impl RateLimiter {
    /// Creates a new rate limiter with a generous initial allowance.
    pub fn new() -> Self {
        Self {
            remaining: 100,
            reset_at: Instant::now(),
        }
    }

    /// Returns the current remaining request count.
    pub fn remaining(&self) -> u64 {
        self.remaining
    }

    /// Waits until the rate-limit window resets if no requests remain.
    pub async fn check_and_wait(&mut self) {
        if self.remaining == 0 {
            let now = Instant::now();
            if now < self.reset_at {
                let wait = self.reset_at - now;
                tracing::warn!(wait_secs = wait.as_secs(), "rate limit exhausted, waiting");
                tokio::time::sleep(wait).await;
                // After sleeping, assume the window has reset.
                self.remaining = 100;
            }
        }
    }

    /// Updates the limiter from API response headers.
    ///
    /// `remaining` is the value of `X-RateLimit-Remaining`.
    /// `reset_timestamp` is the Unix timestamp from `X-RateLimit-Reset`.
    pub fn update_from_headers(&mut self, remaining: u64, reset_timestamp: u64) {
        self.remaining = remaining;

        let now_ts = chrono::Utc::now().timestamp() as u64;
        let delta = reset_timestamp.saturating_sub(now_ts);
        self.reset_at = Instant::now() + Duration::from_secs(delta);

        tracing::trace!(remaining, reset_in_secs = delta, "rate limit updated");
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
    async fn test_new_limiter_does_not_wait() {
        let mut limiter = RateLimiter::new();
        assert_eq!(limiter.remaining(), 100);
        // Should return immediately — no wait needed.
        limiter.check_and_wait().await;
    }

    #[tokio::test]
    async fn test_update_from_headers() {
        let mut limiter = RateLimiter::new();
        let future_ts = chrono::Utc::now().timestamp() as u64 + 60;
        limiter.update_from_headers(42, future_ts);
        assert_eq!(limiter.remaining(), 42);
    }

    #[tokio::test]
    async fn test_zero_remaining_with_past_reset_does_not_wait() {
        let mut limiter = RateLimiter::new();
        // Simulate exhausted budget with reset already passed.
        let past_ts = chrono::Utc::now().timestamp() as u64 - 10;
        limiter.update_from_headers(0, past_ts);
        // reset_at is in the past, so check_and_wait returns immediately.
        limiter.check_and_wait().await;
    }

    #[tokio::test]
    async fn test_zero_remaining_with_future_reset_waits() {
        let mut limiter = RateLimiter::new();
        // Set remaining to 0 with a reset a short time in the future.
        limiter.remaining = 0;
        limiter.reset_at = Instant::now() + Duration::from_millis(100);

        let start = Instant::now();
        limiter.check_and_wait().await;
        let elapsed = start.elapsed();

        assert!(
            elapsed >= Duration::from_millis(100),
            "should have waited ~100ms, waited {:?}",
            elapsed
        );
        // After waiting, remaining should be reset.
        assert_eq!(limiter.remaining(), 100);
    }
}
