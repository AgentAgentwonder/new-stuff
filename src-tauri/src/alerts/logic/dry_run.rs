use super::actions::{Action, ActionExecutionContext, ActionExecutionResult, ActionType};
use super::conditions::{MarketData, WhaleActivity};
use super::rule_engine::{AlertRule, RuleExecutionResult};
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DryRunResult {
    pub rule_id: String,
    pub rule_name: String,
    pub would_trigger: bool,
    pub evaluation_message: String,
    pub actions_simulated: Vec<SimulatedAction>,
    pub warnings: Vec<String>,
    pub execution_time_ms: u64,
    pub dry_run_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulatedAction {
    pub action_type: String,
    pub would_execute: bool,
    pub reason: String,
    pub validation_errors: Vec<String>,
    pub estimated_impact: Option<String>,
}

pub struct DryRunSimulator;

impl DryRunSimulator {
    pub fn simulate_rule(
        rule: &AlertRule,
        market_data: &MarketData,
        whale_activity: &Option<WhaleActivity>,
    ) -> DryRunResult {
        let start = std::time::Instant::now();

        let evaluation = rule.evaluate(market_data, whale_activity);
        let mut actions_simulated = Vec::new();
        let mut warnings = Vec::new();

        if rule.actions.is_empty() {
            warnings.push("No actions configured for this rule".to_string());
        }

        for action in &rule.actions {
            let simulated = Self::simulate_action(action, rule, market_data, &evaluation.triggered);
            actions_simulated.push(simulated);
        }

        if evaluation.triggered
            && rule
                .actions
                .iter()
                .any(|a| a.action_type == ActionType::ExecuteTrade)
        {
            warnings.push("WARNING: This rule will execute trades when triggered!".to_string());
        }

        let execution_time_ms = start.elapsed().as_millis() as u64;

        DryRunResult {
            rule_id: rule.id.clone(),
            rule_name: rule.name.clone(),
            would_trigger: evaluation.triggered,
            evaluation_message: evaluation.message,
            actions_simulated,
            warnings,
            execution_time_ms,
            dry_run_at: Utc::now().to_rfc3339(),
        }
    }

    fn simulate_action(
        action: &Action,
        rule: &AlertRule,
        market_data: &MarketData,
        would_trigger: &bool,
    ) -> SimulatedAction {
        let mut validation_errors = Vec::new();

        if let Err(e) = action.validate() {
            validation_errors.push(e);
        }

        let would_execute = *would_trigger && action.enabled && validation_errors.is_empty();

        let reason = if !would_trigger {
            "Rule conditions not met".to_string()
        } else if !action.enabled {
            "Action is disabled".to_string()
        } else if !validation_errors.is_empty() {
            "Action has validation errors".to_string()
        } else {
            "Action would execute successfully".to_string()
        };

        let estimated_impact = Self::estimate_action_impact(action, market_data, rule);

        SimulatedAction {
            action_type: action.action_type.as_str().to_string(),
            would_execute,
            reason,
            validation_errors,
            estimated_impact,
        }
    }

    fn estimate_action_impact(
        action: &Action,
        market_data: &MarketData,
        rule: &AlertRule,
    ) -> Option<String> {
        match action.action_type {
            ActionType::Notify
            | ActionType::SendEmail
            | ActionType::SendWebhook
            | ActionType::SendTelegram
            | ActionType::SendSlack
            | ActionType::SendDiscord => Some("Would send notification".to_string()),
            ActionType::ExecuteTrade => {
                if let Some(trade_config) = &action.parameters.trade_config {
                    let amount_str = if let Some(amount) = trade_config.amount {
                        format!("{} tokens", amount)
                    } else if let Some(percent) = trade_config.amount_percent {
                        format!("{}% of balance", percent)
                    } else {
                        "Unknown amount".to_string()
                    };

                    Some(format!(
                        "Would {} {} at market price ~${:.6} (slippage: {}bps)",
                        match trade_config.side {
                            super::actions::TradeSide::Buy => "buy",
                            super::actions::TradeSide::Sell => "sell",
                        },
                        amount_str,
                        market_data.current_price,
                        trade_config.slippage_bps
                    ))
                } else {
                    Some("Trade config invalid".to_string())
                }
            }
            ActionType::PauseStrategy => {
                if let Some(strategy_id) = &action.parameters.strategy_id {
                    Some(format!("Would pause strategy: {}", strategy_id))
                } else {
                    Some("Strategy ID not specified".to_string())
                }
            }
            ActionType::UpdateAlert => Some(format!("Would update alert: {}", rule.name)),
            ActionType::LogEvent => Some("Would log event to system".to_string()),
        }
    }

    pub fn simulate_multiple_scenarios(
        rule: &AlertRule,
        scenarios: Vec<(MarketData, Option<WhaleActivity>)>,
    ) -> Vec<DryRunResult> {
        scenarios
            .into_iter()
            .map(|(market_data, whale_activity)| {
                Self::simulate_rule(rule, &market_data, &whale_activity)
            })
            .collect()
    }

    pub fn generate_test_scenarios(
        symbol: &str,
        current_price: f64,
    ) -> Vec<(MarketData, Option<WhaleActivity>)> {
        vec![
            (
                MarketData {
                    symbol: symbol.to_string(),
                    current_price,
                    price_24h_ago: Some(current_price * 0.9),
                    volume_24h: Some(1_000_000.0),
                    market_cap: Some(50_000_000.0),
                    liquidity: Some(2_000_000.0),
                    volatility: Some(5.0),
                    price_change_percentage: Some(10.0),
                    timestamp: Some(Utc::now().to_rfc3339()),
                },
                None,
            ),
            (
                MarketData {
                    symbol: symbol.to_string(),
                    current_price: current_price * 1.2,
                    price_24h_ago: Some(current_price),
                    volume_24h: Some(5_000_000.0),
                    market_cap: Some(50_000_000.0),
                    liquidity: Some(2_000_000.0),
                    volatility: Some(15.0),
                    price_change_percentage: Some(20.0),
                    timestamp: Some(Utc::now().to_rfc3339()),
                },
                None,
            ),
            (
                MarketData {
                    symbol: symbol.to_string(),
                    current_price: current_price * 0.8,
                    price_24h_ago: Some(current_price),
                    volume_24h: Some(800_000.0),
                    market_cap: Some(50_000_000.0),
                    liquidity: Some(2_000_000.0),
                    volatility: Some(12.0),
                    price_change_percentage: Some(-20.0),
                    timestamp: Some(Utc::now().to_rfc3339()),
                },
                None,
            ),
        ]
    }
}

pub fn execute_rule_with_dry_run(
    rule: &AlertRule,
    market_data: &MarketData,
    whale_activity: &Option<WhaleActivity>,
    dry_run: bool,
) -> RuleExecutionResult {
    let evaluation = rule.evaluate(market_data, whale_activity);
    let mut action_results = Vec::new();

    if evaluation.triggered {
        for action in &rule.actions {
            if !action.enabled {
                continue;
            }

            let result = if dry_run {
                ActionExecutionResult {
                    action_id: action.action_id(),
                    success: true,
                    message: format!(
                        "DRY RUN: Would execute {} action",
                        action.action_type.as_str()
                    ),
                    error: None,
                    data: Some(serde_json::json!({
                        "dryRun": true,
                        "actionType": action.action_type.as_str(),
                    })),
                    executed_at: Utc::now().to_rfc3339(),
                }
            } else {
                let context = ActionExecutionContext {
                    alert_id: rule.id.clone(),
                    alert_name: rule.name.clone(),
                    symbol: rule.symbol.clone().unwrap_or_default(),
                    current_price: market_data.current_price,
                    conditions_met: evaluation.message.clone(),
                    trigger_data: serde_json::to_value(&evaluation).unwrap_or_default(),
                    dry_run: false,
                };

                ActionExecutionResult {
                    action_id: action.action_id(),
                    success: true,
                    message: format!("Executed {} action", action.action_type.as_str()),
                    error: None,
                    data: Some(context.to_json()),
                    executed_at: Utc::now().to_rfc3339(),
                }
            };

            action_results.push(result);
        }
    }

    RuleExecutionResult {
        rule_id: rule.id.clone(),
        triggered: evaluation.triggered,
        evaluation,
        action_results,
        dry_run,
        executed_at: Utc::now().to_rfc3339(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alerts::logic::actions::{ActionParameters, NotificationPriority};
    use crate::alerts::logic::conditions::{Condition, ConditionParameters, ConditionType};
    use crate::alerts::logic::rule_engine::{LogicalOperator, RuleGroup, RuleNode};

    #[test]
    fn test_dry_run_simple_rule() {
        let rule = AlertRule {
            id: "test-dry-run".to_string(),
            name: "Test Dry Run".to_string(),
            description: Some("Test dry run simulation".to_string()),
            rule_tree: RuleNode {
                id: Some("root".to_string()),
                condition: Some(Condition {
                    id: Some("c1".to_string()),
                    condition_type: ConditionType::Above,
                    parameters: ConditionParameters {
                        threshold: Some(100.0),
                        ..Default::default()
                    },
                    description: None,
                }),
                group: None,
            },
            actions: vec![Action {
                id: Some("notify".to_string()),
                action_type: ActionType::Notify,
                parameters: ActionParameters {
                    message: Some("Price alert triggered".to_string()),
                    title: Some("Alert".to_string()),
                    priority: Some(NotificationPriority::High),
                    ..Default::default()
                },
                description: None,
                enabled: true,
            }],
            enabled: true,
            symbol: Some("SOL".to_string()),
            owner_id: Some("user1".to_string()),
            shared_with: vec![],
            tags: vec![],
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };

        let market_data = MarketData {
            symbol: "SOL".to_string(),
            current_price: 150.0,
            ..Default::default()
        };

        let result = DryRunSimulator::simulate_rule(&rule, &market_data, &None);

        assert!(result.would_trigger);
        assert_eq!(result.actions_simulated.len(), 1);
        assert!(result.actions_simulated[0].would_execute);
    }

    #[test]
    fn test_dry_run_with_trade_action() {
        use crate::alerts::logic::actions::{OrderType, TradeConfig, TradeSide};

        let rule = AlertRule {
            id: "test-trade-dry-run".to_string(),
            name: "Test Trade Dry Run".to_string(),
            description: None,
            rule_tree: RuleNode {
                id: Some("root".to_string()),
                condition: Some(Condition {
                    id: Some("c1".to_string()),
                    condition_type: ConditionType::Above,
                    parameters: ConditionParameters {
                        threshold: Some(100.0),
                        ..Default::default()
                    },
                    description: None,
                }),
                group: None,
            },
            actions: vec![Action {
                id: Some("trade".to_string()),
                action_type: ActionType::ExecuteTrade,
                parameters: ActionParameters {
                    trade_config: Some(TradeConfig {
                        token_mint: "So11111111111111111111111111111111111111112".to_string(),
                        side: TradeSide::Buy,
                        order_type: OrderType::Market,
                        amount: Some(10.0),
                        amount_percent: None,
                        price: None,
                        slippage_bps: 50,
                        stop_loss_percent: None,
                        take_profit_percent: None,
                        max_retries: 3,
                    }),
                    ..Default::default()
                },
                description: None,
                enabled: true,
            }],
            enabled: true,
            symbol: Some("SOL".to_string()),
            owner_id: Some("user1".to_string()),
            shared_with: vec![],
            tags: vec![],
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };

        let market_data = MarketData {
            symbol: "SOL".to_string(),
            current_price: 150.0,
            ..Default::default()
        };

        let result = DryRunSimulator::simulate_rule(&rule, &market_data, &None);

        assert!(result.would_trigger);
        assert!(result.warnings.iter().any(|w| w.contains("execute trades")));
        assert!(result.actions_simulated[0].estimated_impact.is_some());
    }
}
