use super::types::{RetryPolicy, WebhookError};
use std::time::Duration;
use tokio::time::sleep;

pub struct RetryExecutor {
    policy: RetryPolicy,
}

impl RetryExecutor {
    pub fn new(policy: RetryPolicy) -> Self {
        Self { policy }
    }

    pub async fn execute<F, Fut, T>(&self, mut operation: F) -> Result<T, WebhookError>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, WebhookError>>,
    {
        let mut last_error = None;

        for attempt in 1..=self.policy.max_attempts {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(err) => {
                    last_error = Some(err);

                    if attempt < self.policy.max_attempts {
                        let delay = self.calculate_delay(attempt);
                        sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| WebhookError::Internal("Retry failed without error".to_string())))
    }

    fn calculate_delay(&self, attempt: u32) -> Duration {
        let base_delay = Duration::from_secs(self.policy.base_delay_secs);
        let multiplier = 2u64.pow(attempt - 1);
        let delay_secs = (base_delay.as_secs() * multiplier).min(self.policy.max_delay_secs);

        let delay = if self.policy.jitter {
            let jitter_range = delay_secs / 4;
            let jitter: i64 = rand::random_range(-(jitter_range as i64)..=(jitter_range as i64));
            Duration::from_secs((delay_secs as i64 + jitter).max(1) as u64)
        } else {
            Duration::from_secs(delay_secs)
        };

        delay
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_successful_first_attempt() {
        let policy = RetryPolicy {
            max_attempts: 3,
            base_delay_secs: 1,
            max_delay_secs: 10,
            jitter: false,
        };
        let executor = RetryExecutor::new(policy);

        let result = executor
            .execute(|| async { Ok::<_, WebhookError>(42) })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_and_succeed() {
        let policy = RetryPolicy {
            max_attempts: 3,
            base_delay_secs: 1,
            max_delay_secs: 10,
            jitter: false,
        };
        let executor = RetryExecutor::new(policy);
        let counter = Arc::new(AtomicU32::new(0));

        let result = executor
            .execute(|| {
                let counter = counter.clone();
                async move {
                    let count = counter.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err(WebhookError::Internal("Not yet".to_string()))
                    } else {
                        Ok(42)
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_exhausted() {
        let policy = RetryPolicy {
            max_attempts: 3,
            base_delay_secs: 1,
            max_delay_secs: 10,
            jitter: false,
        };
        let executor = RetryExecutor::new(policy);
        let counter = Arc::new(AtomicU32::new(0));

        let result = executor
            .execute(|| {
                let counter = counter.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err::<(), _>(WebhookError::Internal("Always fails".to_string()))
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_calculate_delay_exponential() {
        let policy = RetryPolicy {
            max_attempts: 5,
            base_delay_secs: 2,
            max_delay_secs: 60,
            jitter: false,
        };
        let executor = RetryExecutor::new(policy);

        let delay1 = executor.calculate_delay(1);
        let delay2 = executor.calculate_delay(2);
        let delay3 = executor.calculate_delay(3);

        assert_eq!(delay1.as_secs(), 2);
        assert_eq!(delay2.as_secs(), 4);
        assert_eq!(delay3.as_secs(), 8);
    }

    #[test]
    fn test_calculate_delay_max_cap() {
        let policy = RetryPolicy {
            max_attempts: 10,
            base_delay_secs: 2,
            max_delay_secs: 10,
            jitter: false,
        };
        let executor = RetryExecutor::new(policy);

        let delay = executor.calculate_delay(10);
        assert!(delay.as_secs() <= 10);
    }
}
