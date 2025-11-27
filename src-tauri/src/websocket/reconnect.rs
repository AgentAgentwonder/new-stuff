use crate::websocket::types::BackoffConfig;
use std::time::Duration;

pub struct ExponentialBackoff {
    config: BackoffConfig,
    attempts: u32,
}

impl ExponentialBackoff {
    pub fn new(config: BackoffConfig) -> Self {
        Self {
            config,
            attempts: 0,
        }
    }

    pub fn reset(&mut self) {
        self.attempts = 0;
    }

    pub fn next_delay(&mut self) -> Option<Duration> {
        if self.attempts >= self.config.max_attempts {
            return None;
        }

        let delay = if self.attempts == 0 {
            self.config.initial_delay
        } else {
            let exponential = self
                .config
                .initial_delay
                .checked_mul(2u32.pow(self.attempts - 1))
                .unwrap_or(self.config.max_delay);
            std::cmp::min(exponential, self.config.max_delay)
        };

        self.attempts += 1;

        // Add jitter: ±20%
        let jitter_range = delay.as_millis() as f64 * 0.2;
        let jitter = rand::random_range(-jitter_range..=jitter_range);
        let jittered_ms = (delay.as_millis() as f64 + jitter).max(0.0) as u64;

        Some(Duration::from_millis(jittered_ms))
    }

    pub fn attempts(&self) -> u32 {
        self.attempts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_backoff() {
        let config = BackoffConfig {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(8),
            max_attempts: 5,
        };
        let mut backoff = ExponentialBackoff::new(config);

        // First delay
        let delay1 = backoff.next_delay().unwrap();
        assert!(delay1.as_millis() >= 800 && delay1.as_millis() <= 1200); // 1s ± 20%

        // Second delay
        let delay2 = backoff.next_delay().unwrap();
        assert!(delay2.as_millis() >= 1600 && delay2.as_millis() <= 2400); // 2s ± 20%

        // Third delay
        let delay3 = backoff.next_delay().unwrap();
        assert!(delay3.as_millis() >= 3200 && delay3.as_millis() <= 4800); // 4s ± 20%

        // Fourth delay (capped at max)
        let delay4 = backoff.next_delay().unwrap();
        assert!(delay4.as_millis() >= 6400 && delay4.as_millis() <= 9600); // 8s ± 20%

        // Fifth delay (max attempts)
        let delay5 = backoff.next_delay().unwrap();
        assert!(delay5.as_millis() >= 6400 && delay5.as_millis() <= 9600); // 8s ± 20%

        // Should return None after max attempts
        assert!(backoff.next_delay().is_none());
    }

    #[test]
    fn test_backoff_reset() {
        let config = BackoffConfig::default();
        let mut backoff = ExponentialBackoff::new(config);

        backoff.next_delay();
        backoff.next_delay();
        assert_eq!(backoff.attempts(), 2);

        backoff.reset();
        assert_eq!(backoff.attempts(), 0);
    }
}
