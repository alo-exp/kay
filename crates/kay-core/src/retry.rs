//! Retry Logic for Kay
//!
//! Provides automatic retry with exponential backoff for transient failures.

use std::time::Duration;

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Add random jitter to delays
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

/// Retry result
pub enum RetryResult<T, E> {
    /// Operation succeeded
    Success(T),
    /// Operation failed after all retries
    Exhausted(E),
}

/// Execute a retryable operation
pub async fn retry<F, Fut, T, E>(config: RetryConfig, mut operation: F) -> RetryResult<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let mut attempts = 0;
    let mut delay = config.initial_delay;

    loop {
        match operation().await {
            Ok(result) => return RetryResult::Success(result),
            Err(e) => {
                attempts += 1;
                if attempts >= config.max_attempts {
                    return RetryResult::Exhausted(e);
                }

                // Apply exponential backoff
                delay = Duration::min(
                    delay * config.backoff_multiplier as u32,
                    config.max_delay,
                );

                // Add jitter if enabled
                if config.jitter {
                    let jitter_range = delay.as_millis() as f64 * 0.1;
                    let jitter_ms = (rand_simple() * jitter_range) as u64;
                    delay += Duration::from_millis(jitter_ms);
                }

                tokio::time::sleep(delay).await;
            }
        }
    }
}

/// Simple random number generator (deterministic for testing)
fn rand_simple() -> f64 {
    use std::time::Instant;
    let now = Instant::now();
    let nanos = now.elapsed().as_nanos();
    (nanos % 1000) as f64 / 1000.0
}

/// Retry with a custom condition for what constitutes a retryable error
pub async fn retry_with_condition<F, Fut, T, E, C>(
    config: RetryConfig,
    is_retryable: C,
    mut operation: F,
) -> RetryResult<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    C: Fn(&E) -> bool,
{
    let mut attempts = 0;
    let mut delay = config.initial_delay;

    loop {
        match operation().await {
            Ok(result) => return RetryResult::Success(result),
            Err(e) => {
                attempts += 1;
                if attempts >= config.max_attempts || !is_retryable(&e) {
                    return RetryResult::Exhausted(e);
                }

                delay = Duration::min(
                    delay * config.backoff_multiplier as u32,
                    config.max_delay,
                );

                tokio::time::sleep(delay).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retry_success_first_try() {
        let config = RetryConfig::default();
        let mut call_count = 0;
        
        let result = retry(config, || {
            call_count += 1;
            async { Ok::<(), ()>(()) }
        }).await;
        
        match result {
            RetryResult::Success(()) => assert_eq!(call_count, 1),
            RetryResult::Exhausted(_) => panic!("Should have succeeded"),
        }
    }

    #[tokio::test]
    async fn test_retry_exhausted() {
        let mut config = RetryConfig::default();
        config.max_attempts = 2;
        config.initial_delay = Duration::from_millis(10);
        
        let result = retry(config, || async { Err::<(), ()>(()) }).await;
        
        match result {
            RetryResult::Success(_) => panic!("Should have failed"),
            RetryResult::Exhausted(()) => {},
        }
    }
}