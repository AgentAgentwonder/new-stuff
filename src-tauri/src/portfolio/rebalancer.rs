use std::collections::HashMap;
use std::sync::Mutex;

use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use tauri::State;

use super::types::{
    AllocationTarget, PortfolioMetrics, Position, RebalanceAction, RebalanceHistory,
    RebalanceProfile,
};

#[derive(Debug)]
struct ProfileState {
    profile: RebalanceProfile,
    last_executed_at: Option<DateTime<Utc>>,
    last_notification_at: Option<DateTime<Utc>>,
}

impl ProfileState {
    fn new(profile: RebalanceProfile) -> Self {
        Self {
            profile,
            last_executed_at: None,
            last_notification_at: None,
        }
    }
}

#[derive(Debug)]
pub struct RebalancerState {
    profiles: HashMap<String, ProfileState>,
    history: Vec<RebalanceHistory>,
}

impl Default for RebalancerState {
    fn default() -> Self {
        let mut profiles = HashMap::new();
        let default_profile = RebalanceProfile {
            id: "core-growth".to_string(),
            name: "Core Growth".to_string(),
            targets: vec![
                AllocationTarget {
                    symbol: "SOL".to_string(),
                    target_percent: 40.0,
                },
                AllocationTarget {
                    symbol: "BTC".to_string(),
                    target_percent: 30.0,
                },
                AllocationTarget {
                    symbol: "ETH".to_string(),
                    target_percent: 20.0,
                },
                AllocationTarget {
                    symbol: "USDC".to_string(),
                    target_percent: 10.0,
                },
            ],
            deviation_trigger_percent: 5.0,
            time_interval_hours: Some(168),
            enabled: true,
        };
        profiles.insert(
            default_profile.id.clone(),
            ProfileState::new(default_profile),
        );

        Self {
            profiles,
            history: Vec::new(),
        }
    }
}

impl RebalancerState {
    fn list_profiles(&self) -> Vec<RebalanceProfile> {
        self.profiles
            .values()
            .map(|p| p.profile.clone())
            .collect::<Vec<_>>()
    }

    fn upsert_profile(&mut self, profile: RebalanceProfile) -> RebalanceProfile {
        let id = profile.id.clone();
        self.profiles
            .insert(id.clone(), ProfileState::new(profile.clone()));
        profile
    }

    fn remove_profile(&mut self, profile_id: &str) -> bool {
        self.profiles.remove(profile_id).is_some()
    }

    fn record_history(&mut self, entry: RebalanceHistory) {
        self.history.push(entry);
        if self.history.len() > 100 {
            let excess = self.history.len() - 100;
            self.history.drain(0..excess);
        }
    }

    fn find_profile_mut(&mut self, profile_id: &str) -> Option<&mut ProfileState> {
        self.profiles.get_mut(profile_id)
    }

    fn find_profile(&self, profile_id: &str) -> Option<&ProfileState> {
        self.profiles.get(profile_id)
    }
}

pub type SharedRebalancerState = Mutex<RebalancerState>;

#[derive(Debug)]
pub struct PortfolioDataState {
    metrics: PortfolioMetrics,
    positions: Vec<Position>,
}

impl Default for PortfolioDataState {
    fn default() -> Self {
        Self::new()
    }
}

impl PortfolioDataState {
    pub fn new() -> Self {
        let positions = Self::default_positions();
        let mut state = Self {
            metrics: Self::baseline_metrics(),
            positions,
        };
        state.recalculate();
        state
    }

    fn baseline_metrics() -> PortfolioMetrics {
        let now = Utc::now().to_rfc3339();
        PortfolioMetrics {
            total_value: 0.0,
            daily_pnl: 0.0,
            daily_pnl_percent: 1.2,
            weekly_pnl: 0.0,
            weekly_pnl_percent: 4.1,
            monthly_pnl: 0.0,
            monthly_pnl_percent: 12.4,
            all_time_pnl: 0.0,
            all_time_pnl_percent: 28.7,
            realized_pnl: 14850.0,
            unrealized_pnl: 0.0,
            last_updated: now,
        }
    }

    fn default_positions() -> Vec<Position> {
        vec![
            Position {
                symbol: "SOL".to_string(),
                mint: "So11111111111111111111111111111111111111112".to_string(),
                amount: 320.0,
                current_price: 175.4,
                avg_entry_price: 142.0,
                total_value: 0.0,
                unrealized_pnl: 0.0,
                unrealized_pnl_percent: 0.0,
                allocation: 0.0,
            },
            Position {
                symbol: "BTC".to_string(),
                mint: "11111111111111111111111111111111".to_string(),
                amount: 2.6,
                current_price: 64000.0,
                avg_entry_price: 42800.0,
                total_value: 0.0,
                unrealized_pnl: 0.0,
                unrealized_pnl_percent: 0.0,
                allocation: 0.0,
            },
            Position {
                symbol: "ETH".to_string(),
                mint: "22222222222222222222222222222222".to_string(),
                amount: 35.0,
                current_price: 3400.0,
                avg_entry_price: 2600.0,
                total_value: 0.0,
                unrealized_pnl: 0.0,
                unrealized_pnl_percent: 0.0,
                allocation: 0.0,
            },
            Position {
                symbol: "USDC".to_string(),
                mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                amount: 85000.0,
                current_price: 1.0,
                avg_entry_price: 1.0,
                total_value: 0.0,
                unrealized_pnl: 0.0,
                unrealized_pnl_percent: 0.0,
                allocation: 0.0,
            },
        ]
    }

    pub fn metrics(&self) -> PortfolioMetrics {
        self.metrics.clone()
    }

    pub fn positions(&self) -> Vec<Position> {
        self.positions.clone()
    }

    pub fn recalculate(&mut self) {
        let total_value: f64 = self
            .positions
            .iter()
            .map(|p| p.amount * p.current_price)
            .sum();

        let mut unrealized_total = 0.0;

        for position in self.positions.iter_mut() {
            position.total_value = position.amount * position.current_price;
            let cost_basis = position.avg_entry_price * position.amount;
            position.unrealized_pnl = position.total_value - cost_basis;
            position.unrealized_pnl_percent = if cost_basis.abs() < f64::EPSILON {
                0.0
            } else {
                (position.unrealized_pnl / cost_basis) * 100.0
            };
            position.allocation = if total_value.abs() < f64::EPSILON {
                0.0
            } else {
                (position.total_value / total_value) * 100.0
            };
            unrealized_total += position.unrealized_pnl;
        }

        if total_value.abs() < f64::EPSILON {
            return;
        }

        let realized = self.metrics.realized_pnl;
        let daily_pct = self.metrics.daily_pnl_percent;
        let weekly_pct = self.metrics.weekly_pnl_percent;
        let monthly_pct = self.metrics.monthly_pnl_percent;
        let all_time_pct = self.metrics.all_time_pnl_percent;

        self.metrics.total_value = total_value;
        self.metrics.unrealized_pnl = unrealized_total;
        self.metrics.daily_pnl = total_value * (daily_pct / 100.0);
        self.metrics.weekly_pnl = total_value * (weekly_pct / 100.0);
        self.metrics.monthly_pnl = total_value * (monthly_pct / 100.0);
        self.metrics.all_time_pnl = realized + unrealized_total;

        let invested_capital = total_value - unrealized_total - realized;
        self.metrics.all_time_pnl_percent = if invested_capital.abs() < f64::EPSILON {
            all_time_pct
        } else {
            (self.metrics.all_time_pnl / invested_capital) * 100.0
        };

        self.metrics.last_updated = Utc::now().to_rfc3339();
    }

    pub fn apply_rebalance(&mut self, actions: &[RebalanceAction]) {
        let mut position_map: HashMap<String, usize> = self
            .positions
            .iter()
            .enumerate()
            .map(|(idx, p)| (p.symbol.clone(), idx))
            .collect();

        for action in actions {
            if let Some(idx) = position_map.get(&action.symbol).cloned() {
                let position = self.positions.get_mut(idx).unwrap();
                let value_change = if action.action == "buy" {
                    action.estimated_value
                } else {
                    -action.estimated_value
                };

                let new_total = (position.total_value + value_change).max(0.0);
                if position.current_price.abs() > f64::EPSILON {
                    position.amount = new_total / position.current_price;
                }
                position.total_value = new_total;
            }
        }

        self.recalculate();
    }
}

pub type SharedPortfolioData = Mutex<PortfolioDataState>;

#[derive(Debug, Deserialize)]
pub struct RebalanceProfileInput {
    pub id: Option<String>,
    pub name: String,
    pub targets: Vec<AllocationTarget>,
    #[serde(rename = "deviationTriggerPercent")]
    pub deviation_trigger_percent: f64,
    #[serde(rename = "timeIntervalHours")]
    pub time_interval_hours: Option<u32>,
    pub enabled: bool,
}

fn map_actions(
    profile: &RebalanceProfile,
    positions: &[Position],
    metrics: &PortfolioMetrics,
) -> Vec<RebalanceAction> {
    let mut target_lookup: HashMap<String, &AllocationTarget> = profile
        .targets
        .iter()
        .map(|t| (t.symbol.clone(), t))
        .collect();

    let mut actions = Vec::new();
    for position in positions.iter() {
        let target_percent = target_lookup
            .get(&position.symbol)
            .map(|t| t.target_percent)
            .unwrap_or(0.0);

        let deviation = position.allocation - target_percent;
        if deviation.abs() < 0.25 {
            continue;
        }

        let target_value = metrics.total_value * (target_percent / 100.0);
        let current_value = position.total_value;
        let difference = target_value - current_value;

        let action = if difference > 0.0 { "buy" } else { "sell" };
        let estimated_value = difference.abs();
        let amount = if position.current_price.abs() < f64::EPSILON {
            0.0
        } else {
            estimated_value / position.current_price
        };

        actions.push(RebalanceAction {
            symbol: position.symbol.clone(),
            mint: position.mint.clone(),
            current_percent: position.allocation,
            target_percent,
            deviation,
            action: action.to_string(),
            amount,
            estimated_value,
        });
    }

    actions
}

fn generate_history_id() -> String {
    format!("rebalance-{}", Utc::now().timestamp_millis())
}

pub fn create_history(
    profile_id: &str,
    trigger_type: &str,
    actions: Vec<RebalanceAction>,
    executed: bool,
) -> RebalanceHistory {
    RebalanceHistory {
        id: generate_history_id(),
        profile_id: profile_id.to_string(),
        actions,
        trigger_type: trigger_type.to_string(),
        executed,
        executed_at: if executed {
            Some(Utc::now().to_rfc3339())
        } else {
            None
        },
        created_at: Utc::now().to_rfc3339(),
    }
}

fn should_trigger_deviation(profile: &RebalanceProfile, actions: &[RebalanceAction]) -> bool {
    actions
        .iter()
        .any(|action| action.deviation.abs() >= profile.deviation_trigger_percent)
}

fn should_trigger_time(profile_state: &ProfileState) -> bool {
    if let (Some(hours), Some(last_exec)) = (
        profile_state.profile.time_interval_hours,
        profile_state.last_executed_at,
    ) {
        let due = last_exec + Duration::hours(hours as i64);
        return Utc::now() >= due;
    }
    false
}

pub fn check_rebalance_triggers_internal(
    rebalancer: &mut RebalancerState,
    portfolio: &PortfolioDataState,
) -> Vec<RebalanceHistory> {
    let mut notifications = Vec::new();
    let now = Utc::now();

    // Collect histories during the loop
    for profile_state in rebalancer.profiles.values_mut() {
        if !profile_state.profile.enabled {
            continue;
        }

        let actions = map_actions(
            &profile_state.profile,
            &portfolio.positions,
            &portfolio.metrics,
        );
        if actions.is_empty() {
            continue;
        }

        let deviation_triggered = should_trigger_deviation(&profile_state.profile, &actions);
        let time_triggered = should_trigger_time(profile_state);

        if !deviation_triggered && !time_triggered {
            continue;
        }

        if let Some(last_notified) = profile_state.last_notification_at {
            if now - last_notified < Duration::minutes(10) {
                continue;
            }
        }

        let trigger_type = if deviation_triggered {
            "deviation"
        } else {
            "time"
        };
        let history = create_history(&profile_state.profile.id, trigger_type, actions, false);

        profile_state.last_notification_at = Some(now);
        notifications.push(history);
    }

    // Record all histories after the loop to avoid borrowing conflict
    for history in &notifications {
        rebalancer.record_history(history.clone());
    }

    notifications
}

#[tauri::command]
pub fn get_portfolio_metrics(
    data: State<'_, SharedPortfolioData>,
) -> Result<PortfolioMetrics, String> {
    data.lock()
        .map_err(|_| "Portfolio data locked".to_string())
        .map(|guard| guard.metrics())
}

#[tauri::command]
pub fn get_positions(data: State<'_, SharedPortfolioData>) -> Result<Vec<Position>, String> {
    data.lock()
        .map_err(|_| "Portfolio data locked".to_string())
        .map(|guard| guard.positions())
}

#[tauri::command]
pub fn list_rebalance_profiles(
    state: State<'_, SharedRebalancerState>,
) -> Result<Vec<RebalanceProfile>, String> {
    state
        .lock()
        .map_err(|_| "Rebalancer unavailable".to_string())
        .map(|guard| guard.list_profiles())
}

#[tauri::command]
pub fn save_rebalance_profile(
    input: RebalanceProfileInput,
    state: State<'_, SharedRebalancerState>,
) -> Result<RebalanceProfile, String> {
    let mut guard = state
        .lock()
        .map_err(|_| "Rebalancer unavailable".to_string())?;

    let profile = RebalanceProfile {
        id: input
            .id
            .unwrap_or_else(|| format!("profile-{}", Utc::now().timestamp_millis())),
        name: input.name,
        targets: input.targets,
        deviation_trigger_percent: input.deviation_trigger_percent,
        time_interval_hours: input.time_interval_hours,
        enabled: input.enabled,
    };

    Ok(guard.upsert_profile(profile))
}

#[tauri::command]
pub fn delete_rebalance_profile(
    profile_id: String,
    state: State<'_, SharedRebalancerState>,
) -> Result<bool, String> {
    state
        .lock()
        .map_err(|_| "Rebalancer unavailable".to_string())
        .map(|mut guard| guard.remove_profile(&profile_id))
}

#[tauri::command]
pub fn preview_rebalance(
    profile_id: String,
    state: State<'_, SharedRebalancerState>,
    data: State<'_, SharedPortfolioData>,
) -> Result<Vec<RebalanceAction>, String> {
    let rebalancer = state
        .lock()
        .map_err(|_| "Rebalancer unavailable".to_string())?;
    let portfolio = data
        .lock()
        .map_err(|_| "Portfolio data locked".to_string())?;

    let profile_state = rebalancer
        .find_profile(&profile_id)
        .ok_or_else(|| "Profile not found".to_string())?;

    Ok(map_actions(
        &profile_state.profile,
        &portfolio.positions,
        &portfolio.metrics,
    ))
}

#[tauri::command]
pub fn execute_rebalance(
    profile_id: String,
    dry_run: bool,
    state: State<'_, SharedRebalancerState>,
    data: State<'_, SharedPortfolioData>,
) -> Result<RebalanceHistory, String> {
    let mut rebalancer = state
        .lock()
        .map_err(|_| "Rebalancer unavailable".to_string())?;
    let mut portfolio = data
        .lock()
        .map_err(|_| "Portfolio data locked".to_string())?;

    let profile_state = rebalancer
        .find_profile_mut(&profile_id)
        .ok_or_else(|| "Profile not found".to_string())?;

    let actions = map_actions(
        &profile_state.profile,
        &portfolio.positions,
        &portfolio.metrics,
    );

    if actions.is_empty() {
        return Err("Portfolio already aligned with targets".to_string());
    }

    let mut history = create_history(
        &profile_state.profile.id,
        "manual",
        actions.clone(),
        !dry_run,
    );

    if !dry_run {
        portfolio.apply_rebalance(&actions);
        history.executed_at = Some(Utc::now().to_rfc3339());
        history.executed = true;
        profile_state.last_executed_at = Some(Utc::now());
    }

    rebalancer.record_history(history.clone());
    Ok(history)
}

#[tauri::command]
pub fn get_rebalance_history(
    state: State<'_, SharedRebalancerState>,
) -> Result<Vec<RebalanceHistory>, String> {
    state
        .lock()
        .map_err(|_| "Rebalancer unavailable".to_string())
        .map(|guard| guard.history.clone())
}

#[tauri::command]
pub fn check_rebalance_triggers(
    state: State<'_, SharedRebalancerState>,
    data: State<'_, SharedPortfolioData>,
) -> Result<Vec<RebalanceHistory>, String> {
    let mut rebalancer = state
        .lock()
        .map_err(|_| "Rebalancer unavailable".to_string())?;
    let portfolio = data
        .lock()
        .map_err(|_| "Portfolio data locked".to_string())?;

    Ok(check_rebalance_triggers_internal(
        &mut rebalancer,
        &portfolio,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_actions_detects_overweight_position() {
        let profile = RebalanceProfile {
            id: "test".to_string(),
            name: "Test".to_string(),
            targets: vec![AllocationTarget {
                symbol: "SOL".to_string(),
                target_percent: 10.0,
            }],
            deviation_trigger_percent: 2.0,
            time_interval_hours: None,
            enabled: true,
        };

        let data = PortfolioDataState::new();
        let positions = data.positions();
        let metrics = data.metrics();
        let actions = map_actions(&profile, &positions, &metrics);

        assert!(!actions.is_empty());
        let sol_action = actions.iter().find(|a| a.symbol == "SOL").unwrap();
        assert_eq!(sol_action.action, "sell");
        assert!(sol_action.deviation > 0.0);
    }

    #[test]
    fn apply_rebalance_moves_allocation_toward_target() {
        let mut data = PortfolioDataState::new();

        let profile = RebalanceProfile {
            id: "test".to_string(),
            name: "Test".to_string(),
            targets: vec![
                AllocationTarget {
                    symbol: "SOL".to_string(),
                    target_percent: 20.0,
                },
                AllocationTarget {
                    symbol: "BTC".to_string(),
                    target_percent: 40.0,
                },
            ],
            deviation_trigger_percent: 2.0,
            time_interval_hours: None,
            enabled: true,
        };

        let positions_before = data.positions();
        let metrics = data.metrics();
        let actions = map_actions(&profile, &positions_before, &metrics);
        assert!(!actions.is_empty());

        data.apply_rebalance(&actions);
        let positions_after = data.positions();

        let sol_after = positions_after.iter().find(|p| p.symbol == "SOL").unwrap();
        let btc_after = positions_after.iter().find(|p| p.symbol == "BTC").unwrap();

        assert!((sol_after.allocation - 20.0).abs() < 5.0);
        assert!((btc_after.allocation - 40.0).abs() < 5.0);
    }

    #[test]
    fn deviation_trigger_detection() {
        let mut rebalancer = RebalancerState::default();
        let data = PortfolioDataState::new();
        let notifications = check_rebalance_triggers_internal(&mut rebalancer, &data);
        assert!(notifications.iter().any(|h| h.trigger_type == "deviation"));
    }
}
