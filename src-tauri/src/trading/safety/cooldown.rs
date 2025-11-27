use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CooldownStatus {
    pub wallet_address: String,
    pub cooldown_seconds: u64,
    pub remaining_seconds: u64,
    pub last_trade_timestamp: u64,
}

#[derive(Debug, Default)]
pub struct CooldownManager {
    cooldowns: HashMap<String, SystemTime>,
    cooldown_duration: Duration,
}

impl CooldownManager {
    pub fn new(cooldown_seconds: u64) -> Self {
        Self {
            cooldowns: HashMap::new(),
            cooldown_duration: Duration::from_secs(cooldown_seconds),
        }
    }

    pub fn set_cooldown_duration(&mut self, seconds: u64) {
        self.cooldown_duration = Duration::from_secs(seconds);
    }

    pub fn record_trade(&mut self, wallet_address: &str) {
        self.cooldown_duration = self.cooldown_duration;
        self.cooldowns
            .insert(wallet_address.to_string(), SystemTime::now());
    }

    pub fn is_on_cooldown(&self, wallet_address: &str) -> bool {
        if let Some(&last_trade) = self.cooldowns.get(wallet_address) {
            if let Ok(elapsed) = last_trade.elapsed() {
                return elapsed < self.cooldown_duration;
            }
        }
        false
    }

    pub fn get_remaining_cooldown(&self, wallet_address: &str) -> Option<CooldownStatus> {
        self.cooldowns.get(wallet_address).and_then(|last_trade| {
            let now = SystemTime::now();
            if let Ok(elapsed) = now.duration_since(*last_trade) {
                if elapsed < self.cooldown_duration {
                    let remaining = self.cooldown_duration - elapsed;
                    let last_trade_timestamp = last_trade
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_else(|_| Duration::from_secs(0));
                    return Some(CooldownStatus {
                        wallet_address: wallet_address.to_string(),
                        cooldown_seconds: self.cooldown_duration.as_secs(),
                        remaining_seconds: remaining.as_secs(),
                        last_trade_timestamp: last_trade_timestamp.as_secs(),
                    });
                }
            }
            None
        })
    }

    pub fn clear(&mut self) {
        self.cooldowns.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_cooldown_logic() {
        let mut manager = CooldownManager::new(1);
        manager.record_trade("wallet1");
        assert!(manager.is_on_cooldown("wallet1"));

        sleep(Duration::from_millis(1100));
        assert!(!manager.is_on_cooldown("wallet1"));
    }

    #[test]
    fn test_remaining_cooldown() {
        let mut manager = CooldownManager::new(2);
        manager.record_trade("wallet1");
        let status = manager.get_remaining_cooldown("wallet1");
        assert!(status.is_some());
        let status = status.unwrap();
        assert_eq!(status.cooldown_seconds, 2);
        assert!(status.remaining_seconds <= 2);
    }
}
