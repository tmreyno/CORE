// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Retry Logic - Exponential backoff with configurable strategies
//!
//! Provides automatic retry for transient failures with exponential backoff,
//! jitter, and maximum attempt limits.

use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (0 = no retries, only initial attempt)
    pub max_attempts: u32,
    
    /// Initial delay before first retry
    pub initial_delay: Duration,
    
    /// Maximum delay between retries
    pub max_delay: Duration,
    
    /// Backoff multiplier (e.g., 2.0 for exponential doubling)
    pub backoff_multiplier: f64,
    
    /// Add jitter to prevent thundering herd (0.0 to 1.0)
    pub jitter_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
        }
    }
}

impl RetryConfig {
    /// Create config for fast operations (network requests, etc.)
    pub fn fast() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(50),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
            jitter_factor: 0.2,
        }
    }
    
    /// Create config for slow operations (file I/O, heavy compute)
    pub fn slow() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
        }
    }
    
    /// Create config with no retries
    pub fn no_retry() -> Self {
        Self {
            max_attempts: 0,
            initial_delay: Duration::from_secs(0),
            max_delay: Duration::from_secs(0),
            backoff_multiplier: 1.0,
            jitter_factor: 0.0,
        }
    }
    
    /// Calculate delay for given attempt number
    fn calculate_delay(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::from_secs(0);
        }
        
        // Exponential backoff: initial_delay * (multiplier ^ (attempt - 1))
        let base_delay = self.initial_delay.as_millis() as f64
            * self.backoff_multiplier.powi((attempt - 1) as i32);
        
        let capped_delay = base_delay.min(self.max_delay.as_millis() as f64);
        
        // Add jitter: random value between (1 - jitter) and (1 + jitter)
        let jitter = if self.jitter_factor > 0.0 {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let jitter_range = 1.0 + self.jitter_factor;
            rng.gen_range((1.0 - self.jitter_factor)..jitter_range)
        } else {
            1.0
        };
        
        Duration::from_millis((capped_delay * jitter) as u64)
    }
}

/// Retry a fallible async operation with exponential backoff
///
/// # Example
/// ```ignore
/// use ffx_check_lib::common::retry::{RetryConfig, retry_async};
///
/// let config = RetryConfig::default();
/// let result = retry_async(config, "file_operation", || async {
///     read_file_with_potential_failure().await
/// }).await;
/// ```
pub async fn retry_async<F, Fut, T, E>(
    config: RetryConfig,
    operation_name: &str,
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempt = 0;
    
    loop {
        attempt += 1;
        
        match operation().await {
            Ok(result) => {
                if attempt > 1 {
                    debug!(
                        "Operation '{}' succeeded on attempt {}/{}",
                        operation_name,
                        attempt,
                        config.max_attempts + 1
                    );
                }
                return Ok(result);
            }
            Err(err) => {
                if attempt > config.max_attempts {
                    warn!(
                        "Operation '{}' failed after {} attempts: {}",
                        operation_name,
                        attempt,
                        err
                    );
                    return Err(err);
                }
                
                let delay = config.calculate_delay(attempt);
                warn!(
                    "Operation '{}' failed (attempt {}/{}): {}. Retrying in {:?}...",
                    operation_name,
                    attempt,
                    config.max_attempts + 1,
                    err,
                    delay
                );
                
                sleep(delay).await;
            }
        }
    }
}

/// Retry a fallible synchronous operation with exponential backoff
///
/// # Example
/// ```ignore
/// use ffx_check_lib::common::retry::{RetryConfig, retry_sync};
///
/// let config = RetryConfig::fast();
/// let result = retry_sync(config, "network_request", || {
///     make_http_request()
/// });
/// ```
pub fn retry_sync<F, T, E>(
    config: RetryConfig,
    operation_name: &str,
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
    E: std::fmt::Display,
{
    let mut attempt = 0;
    
    loop {
        attempt += 1;
        
        match operation() {
            Ok(result) => {
                if attempt > 1 {
                    debug!(
                        "Operation '{}' succeeded on attempt {}/{}",
                        operation_name,
                        attempt,
                        config.max_attempts + 1
                    );
                }
                return Ok(result);
            }
            Err(err) => {
                if attempt > config.max_attempts {
                    warn!(
                        "Operation '{}' failed after {} attempts: {}",
                        operation_name,
                        attempt,
                        err
                    );
                    return Err(err);
                }
                
                let delay = config.calculate_delay(attempt);
                warn!(
                    "Operation '{}' failed (attempt {}/{}): {}. Retrying in {:?}...",
                    operation_name,
                    attempt,
                    config.max_attempts + 1,
                    err,
                    delay
                );
                
                std::thread::sleep(delay);
            }
        }
    }
}

/// Conditionally retry based on error type
///
/// Only retries if the predicate returns true for the error
pub async fn retry_if_async<F, Fut, T, E, P>(
    config: RetryConfig,
    operation_name: &str,
    mut operation: F,
    mut should_retry: P,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
    P: FnMut(&E) -> bool,
{
    let mut attempt = 0;
    
    loop {
        attempt += 1;
        
        match operation().await {
            Ok(result) => {
                if attempt > 1 {
                    debug!(
                        "Operation '{}' succeeded on attempt {}/{}",
                        operation_name,
                        attempt,
                        config.max_attempts + 1
                    );
                }
                return Ok(result);
            }
            Err(err) => {
                // Check if we should retry this error
                if !should_retry(&err) {
                    debug!(
                        "Operation '{}' failed with non-retriable error: {}",
                        operation_name, err
                    );
                    return Err(err);
                }
                
                if attempt > config.max_attempts {
                    warn!(
                        "Operation '{}' failed after {} attempts: {}",
                        operation_name,
                        attempt,
                        err
                    );
                    return Err(err);
                }
                
                let delay = config.calculate_delay(attempt);
                warn!(
                    "Operation '{}' failed (attempt {}/{}): {}. Retrying in {:?}...",
                    operation_name,
                    attempt,
                    config.max_attempts + 1,
                    err,
                    delay
                );
                
                sleep(delay).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    
    #[tokio::test]
    async fn test_retry_success_on_third_attempt() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();
        
        let config = RetryConfig {
            max_attempts: 5,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            jitter_factor: 0.0,
        };
        
        let result = retry_async(config, "test_op", || {
            let counter = counter_clone.clone();
            async move {
                let count = counter.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    Err("Transient failure")
                } else {
                    Ok("Success")
                }
            }
        })
        .await;
        
        assert_eq!(result, Ok("Success"));
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }
    
    #[tokio::test]
    async fn test_retry_exhausted() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();
        
        let config = RetryConfig {
            max_attempts: 2,
            initial_delay: Duration::from_millis(5),
            max_delay: Duration::from_millis(50),
            backoff_multiplier: 2.0,
            jitter_factor: 0.0,
        };
        
        let result = retry_async(config, "test_op", || {
            let counter = counter_clone.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>("Permanent failure")
            }
        })
        .await;
        
        assert_eq!(result, Err("Permanent failure"));
        assert_eq!(counter.load(Ordering::SeqCst), 3); // Initial + 2 retries
    }
    
    #[tokio::test]
    async fn test_retry_if_predicate() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();
        
        let config = RetryConfig::fast();
        
        // Only retry "transient" errors
        let result = retry_if_async(
            config,
            "test_op",
            || {
                let counter = counter_clone.clone();
                async move {
                    let count = counter.fetch_add(1, Ordering::SeqCst);
                    if count == 0 {
                        Err("transient_error")
                    } else if count == 1 {
                        Err("permanent_error")
                    } else {
                        Ok("Success")
                    }
                }
            },
            |err| err.starts_with("transient"),
        )
        .await;
        
        // Should fail on "permanent_error" without further retries
        assert_eq!(result, Err("permanent_error"));
        assert_eq!(counter.load(Ordering::SeqCst), 2); // Initial + 1 retry
    }
    
    #[test]
    fn test_delay_calculation() {
        let config = RetryConfig {
            max_attempts: 5,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            jitter_factor: 0.0,
        };
        
        assert_eq!(config.calculate_delay(0), Duration::from_millis(0));
        assert_eq!(config.calculate_delay(1), Duration::from_millis(100));
        assert_eq!(config.calculate_delay(2), Duration::from_millis(200));
        assert_eq!(config.calculate_delay(3), Duration::from_millis(400));
        assert_eq!(config.calculate_delay(4), Duration::from_millis(800));
    }
}
