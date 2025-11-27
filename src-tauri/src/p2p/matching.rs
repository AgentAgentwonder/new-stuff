use super::types::*;
use crate::security::reputation::WalletReputation;
use anyhow::Result;
use chrono::{DateTime, Utc};
use rand::{seq::SliceRandom, Rng};
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize)]
pub struct TraderMatch {
    pub offer: P2POffer,
    pub compatibility_score: f64,
    pub reputation_delta: f64,
    pub geographic_score: f64,
    pub payment_match_score: f64,
    pub counterparty_profile: TraderProfile,
}

pub struct LocalMatcher {
    max_distance_km: f64,
    payment_priority: HashMap<String, i32>,
}

impl LocalMatcher {
    pub fn new() -> Self {
        Self {
            max_distance_km: 100.0,
            payment_priority: HashMap::new(),
        }
    }

    pub fn with_max_distance(mut self, distance_km: f64) -> Self {
        self.max_distance_km = distance_km;
        self
    }

    pub fn with_payment_priority(mut self, method: &str, priority: i32) -> Self {
        self.payment_priority.insert(method.to_string(), priority);
        self
    }

    pub fn match_offers(
        &self,
        offers: &[P2POffer],
        user_profile: &TraderProfile,
        user_reputation: Option<&WalletReputation>,
    ) -> Vec<TraderMatch> {
        let mut matches = Vec::new();

        for offer in offers {
            if offer.creator == user_profile.address {
                continue;
            }

            if let Some(required) = offer.reputation_required {
                if let Some(rep) = user_reputation {
                    if rep.trust_score < required {
                        continue;
                    }
                }
            }

            let compatibility_score = rand::random_range(0.6..0.95);
            let reputation_delta = if let Some(rep) = user_reputation {
                rep.trust_score - offer.reputation_required.unwrap_or(0.0)
            } else {
                0.0
            };
            let geographic_score = rand::random_range(0.7..0.98);

            let payment_match_score = offer
                .payment_methods
                .iter()
                .map(|method| self.payment_priority.get(method).copied().unwrap_or(50) as f64)
                .fold(f64::NEG_INFINITY, f64::max)
                .max(50.0)
                / 100.0;

            matches.push(TraderMatch {
                offer: offer.clone(),
                compatibility_score,
                reputation_delta,
                geographic_score,
                payment_match_score,
                counterparty_profile: TraderProfile {
                    address: offer.creator.clone(),
                    username: Some(format!("Trader-{}", &offer.creator[..6])),
                    reputation_score: 60.0 + rand::random_range(-10.0..10.0),
                    total_trades: rand::random_range(5..200),
                    successful_trades: rand::random_range(5..200),
                    cancelled_trades: rand::random_range(0..20),
                    disputed_trades: rand::random_range(0..10),
                    avg_completion_time: rand::random_range(5..120) as i64,
                    first_trade_at: Some(DateTime::<Utc>::default()),
                    last_trade_at: Some(Utc::now()),
                    verified: rand::random::<f64>() < 0.6,
                    verification_level: rand::random_range(0..3),
                },
            });
        }

        matches.sort_by(|a, b| {
            b.compatibility_score
                .partial_cmp(&a.compatibility_score)
                .unwrap()
        });
        matches
    }
}

impl Default for LocalMatcher {
    fn default() -> Self {
        Self::new()
    }
}
