use super::actions::{Action, ActionExecutionContext, ActionExecutionResult};
use super::conditions::{Condition, ConditionEvaluationResult, MarketData, WhaleActivity};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
#[serde(rename_all = "snake_case")]
pub enum LogicalOperator {
    And,
    Or,
}

impl LogicalOperator {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogicalOperator::And => "and",
            LogicalOperator::Or => "or",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "and" | "AND" => Some(LogicalOperator::And),
            "or" | "OR" => Some(LogicalOperator::Or),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleNode {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<Condition>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<RuleGroup>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleGroup {
    pub operator: LogicalOperator,
    pub nodes: Vec<RuleNode>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_minutes: Option<i32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub rule_tree: RuleNode,
    pub actions: Vec<Action>,
    pub enabled: bool,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_id: Option<String>,

    #[serde(default)]
    pub shared_with: Vec<SharedAccess>,

    #[serde(default)]
    pub tags: Vec<String>,

    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SharedAccess {
    pub user_id: String,
    pub permission: Permission,
    pub granted_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Permission {
    View,
    Edit,
    Execute,
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleEvaluationResult {
    pub rule_id: String,
    pub triggered: bool,
    pub condition_results: Vec<ConditionEvaluationResult>,
    pub message: String,
    pub confidence: f64,
    pub evaluated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_satisfied: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleExecutionResult {
    pub rule_id: String,
    pub triggered: bool,
    pub evaluation: RuleEvaluationResult,
    pub action_results: Vec<ActionExecutionResult>,
    pub dry_run: bool,
    pub executed_at: String,
}

impl AlertRule {
    pub fn evaluate(
        &self,
        market_data: &MarketData,
        whale_activity: &Option<WhaleActivity>,
    ) -> RuleEvaluationResult {
        let (triggered, condition_results, message, confidence, window_satisfied) =
            self.evaluate_node(&self.rule_tree, market_data, whale_activity);

        RuleEvaluationResult {
            rule_id: self.id.clone(),
            triggered,
            condition_results,
            message,
            confidence,
            evaluated_at: Utc::now().to_rfc3339(),
            window_satisfied,
        }
    }

    fn evaluate_node(
        &self,
        node: &RuleNode,
        market_data: &MarketData,
        whale_activity: &Option<WhaleActivity>,
    ) -> (
        bool,
        Vec<ConditionEvaluationResult>,
        String,
        f64,
        Option<bool>,
    ) {
        if let Some(condition) = &node.condition {
            let result = condition.evaluate(market_data, whale_activity);
            let triggered = result.met;
            let message = result.message.clone();
            let confidence = result.confidence;
            (triggered, vec![result], message, confidence, None)
        } else if let Some(group) = &node.group {
            self.evaluate_group(group, market_data, whale_activity)
        } else {
            (
                false,
                Vec::new(),
                node.label
                    .clone()
                    .unwrap_or_else(|| "Empty node without condition".to_string()),
                0.0,
                None,
            )
        }
    }

    fn evaluate_group(
        &self,
        group: &RuleGroup,
        market_data: &MarketData,
        whale_activity: &Option<WhaleActivity>,
    ) -> (
        bool,
        Vec<ConditionEvaluationResult>,
        String,
        f64,
        Option<bool>,
    ) {
        let mut all_results = Vec::new();
        let mut all_messages = Vec::new();
        let mut total_confidence = 0.0;
        let mut count = 0;

        for node in &group.nodes {
            let (met, results, message, confidence, _) =
                self.evaluate_node(node, market_data, whale_activity);
            all_results.extend(results);
            all_messages.push(format!("({}: {})", if met { "✓" } else { "✗" }, message));
            total_confidence += confidence;
            count += 1;
        }

        let node_results: Vec<bool> = all_results.iter().map(|r| r.met).collect();
        let mut triggered = match group.operator {
            LogicalOperator::And => node_results.iter().all(|&x| x),
            LogicalOperator::Or => node_results.iter().any(|&x| x),
        };

        let mut window_satisfied = None;
        if let Some(window_minutes) = group.window_minutes {
            window_satisfied = Some(self.is_within_window(window_minutes, market_data));
            if window_satisfied == Some(false) {
                triggered = false;
            }
        }

        let label = group
            .label
            .clone()
            .unwrap_or_else(|| group.operator.as_str().to_uppercase());

        let message = format!(
            "{} [{}]",
            label,
            if all_messages.is_empty() {
                "No child nodes evaluated".to_string()
            } else {
                all_messages.join(if group.operator == LogicalOperator::And {
                    " && "
                } else {
                    " || "
                })
            }
        );

        let avg_confidence = if count > 0 {
            total_confidence / count as f64
        } else {
            0.0
        };

        (
            triggered,
            all_results,
            message,
            avg_confidence,
            window_satisfied,
        )
    }

    fn is_within_window(&self, window_minutes: i32, market_data: &MarketData) -> bool {
        if window_minutes <= 0 {
            return true;
        }

        if let Some(ts) = &market_data.timestamp {
            if let Ok(parsed) = DateTime::parse_from_rfc3339(ts) {
                let parsed_utc = parsed.with_timezone(&Utc);
                let diff = Utc::now() - parsed_utc;
                return diff <= Duration::minutes(window_minutes as i64);
            }
        }

        false
    }

    pub fn has_access(&self, user_id: &str, required_permission: Permission) -> bool {
        if let Some(owner) = &self.owner_id {
            if owner == user_id {
                return true;
            }
        }

        self.shared_with
            .iter()
            .any(|access| access.user_id == user_id && access.permission >= required_permission)
    }
}

pub struct RuleEngine {
    rules: HashMap<String, AlertRule>,
}

impl RuleEngine {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, rule: AlertRule) {
        self.rules.insert(rule.id.clone(), rule);
    }

    pub fn remove_rule(&mut self, rule_id: &str) -> Option<AlertRule> {
        self.rules.remove(rule_id)
    }

    pub fn get_rule(&self, rule_id: &str) -> Option<&AlertRule> {
        self.rules.get(rule_id)
    }

    pub fn get_rule_mut(&mut self, rule_id: &str) -> Option<&mut AlertRule> {
        self.rules.get_mut(rule_id)
    }

    pub fn list_rules(&self) -> Vec<&AlertRule> {
        self.rules.values().collect()
    }

    pub fn list_rules_for_user(&self, user_id: &str, permission: Permission) -> Vec<&AlertRule> {
        self.rules
            .values()
            .filter(|rule| rule.has_access(user_id, permission))
            .collect()
    }

    pub fn evaluate_rule(
        &self,
        rule_id: &str,
        market_data: &MarketData,
        whale_activity: &Option<WhaleActivity>,
    ) -> Option<RuleEvaluationResult> {
        self.rules
            .get(rule_id)
            .filter(|rule| rule.enabled)
            .map(|rule| rule.evaluate(market_data, whale_activity))
    }

    pub fn evaluate_all_rules(
        &self,
        market_data: &MarketData,
        whale_activity: &Option<WhaleActivity>,
    ) -> Vec<RuleEvaluationResult> {
        self.rules
            .values()
            .filter(|rule| rule.enabled)
            .filter(|rule| {
                if let Some(symbol) = &rule.symbol {
                    symbol == &market_data.symbol
                } else {
                    true
                }
            })
            .map(|rule| rule.evaluate(market_data, whale_activity))
            .collect()
    }

    pub fn count_rules(&self) -> usize {
        self.rules.len()
    }

    pub fn count_enabled_rules(&self) -> usize {
        self.rules.values().filter(|rule| rule.enabled).count()
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alerts::logic::conditions::{ConditionParameters, ConditionType};

    #[test]
    fn test_simple_and_rule() {
        let rule = AlertRule {
            id: "test-1".to_string(),
            name: "Test AND Rule".to_string(),
            description: None,
            rule_tree: RuleNode {
                id: Some("root".to_string()),
                label: Some("Root".to_string()),
                condition: None,
                group: Some(RuleGroup {
                    operator: LogicalOperator::And,
                    nodes: vec![
                        RuleNode {
                            id: Some("cond1".to_string()),
                            label: Some("Price Above".to_string()),
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
                            metadata: None,
                        },
                        RuleNode {
                            id: Some("cond2".to_string()),
                            label: Some("Price Below".to_string()),
                            condition: Some(Condition {
                                id: Some("c2".to_string()),
                                condition_type: ConditionType::Below,
                                parameters: ConditionParameters {
                                    threshold: Some(200.0),
                                    ..Default::default()
                                },
                                description: None,
                            }),
                            group: None,
                            metadata: None,
                        },
                    ],
                    window_minutes: None,
                    label: Some("Price Window".to_string()),
                    description: None,
                }),
                metadata: None,
            },
            actions: vec![],
            enabled: true,
            symbol: Some("SOL".to_string()),
            owner_id: Some("user1".to_string()),
            team_id: None,
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

        let result = rule.evaluate(&market_data, &None);
        assert!(result.triggered);
        assert_eq!(result.window_satisfied, None);
    }

    #[test]
    fn test_simple_or_rule() {
        let rule = AlertRule {
            id: "test-2".to_string(),
            name: "Test OR Rule".to_string(),
            description: None,
            rule_tree: RuleNode {
                id: Some("root".to_string()),
                label: Some("Root".to_string()),
                condition: None,
                group: Some(RuleGroup {
                    operator: LogicalOperator::Or,
                    nodes: vec![
                        RuleNode {
                            id: Some("cond1".to_string()),
                            label: Some("Price Above".to_string()),
                            condition: Some(Condition {
                                id: Some("c1".to_string()),
                                condition_type: ConditionType::Above,
                                parameters: ConditionParameters {
                                    threshold: Some(200.0),
                                    ..Default::default()
                                },
                                description: None,
                            }),
                            group: None,
                            metadata: None,
                        },
                        RuleNode {
                            id: Some("cond2".to_string()),
                            label: Some("Price Below".to_string()),
                            condition: Some(Condition {
                                id: Some("c2".to_string()),
                                condition_type: ConditionType::Below,
                                parameters: ConditionParameters {
                                    threshold: Some(100.0),
                                    ..Default::default()
                                },
                                description: None,
                            }),
                            group: None,
                            metadata: None,
                        },
                    ],
                    window_minutes: None,
                    label: Some("Price Range".to_string()),
                    description: None,
                }),
                metadata: None,
            },
            actions: vec![],
            enabled: true,
            symbol: Some("SOL".to_string()),
            owner_id: Some("user1".to_string()),
            team_id: None,
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

        let result = rule.evaluate(&market_data, &None);
        assert!(!result.triggered);
    }

    #[test]
    fn test_group_window_enforcement() {
        let rule = AlertRule {
            id: "test-window".to_string(),
            name: "Test Window".to_string(),
            description: None,
            rule_tree: RuleNode {
                id: Some("root".to_string()),
                label: None,
                condition: None,
                group: Some(RuleGroup {
                    operator: LogicalOperator::And,
                    nodes: vec![RuleNode {
                        id: Some("cond1".to_string()),
                        label: Some("Price Above".to_string()),
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
                        metadata: None,
                    }],
                    window_minutes: Some(5),
                    label: Some("Recent Price".to_string()),
                    description: None,
                }),
                metadata: None,
            },
            actions: vec![],
            enabled: true,
            symbol: Some("SOL".to_string()),
            owner_id: Some("user1".to_string()),
            team_id: None,
            shared_with: vec![],
            tags: vec![],
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };

        let market_data = MarketData {
            symbol: "SOL".to_string(),
            current_price: 150.0,
            timestamp: Some((Utc::now() - Duration::minutes(10)).to_rfc3339()),
            ..Default::default()
        };

        let result = rule.evaluate(&market_data, &None);
        assert!(!result.triggered);
        assert_eq!(result.window_satisfied, Some(false));
    }

    #[test]
    fn test_permission_system() {
        let rule = AlertRule {
            id: "test-3".to_string(),
            name: "Test Permissions".to_string(),
            description: None,
            rule_tree: RuleNode {
                id: None,
                label: None,
                condition: None,
                group: None,
                metadata: None,
            },
            actions: vec![],
            enabled: true,
            symbol: None,
            owner_id: Some("owner".to_string()),
            team_id: None,
            shared_with: vec![
                SharedAccess {
                    user_id: "viewer".to_string(),
                    permission: Permission::View,
                    granted_at: Utc::now().to_rfc3339(),
                },
                SharedAccess {
                    user_id: "editor".to_string(),
                    permission: Permission::Edit,
                    granted_at: Utc::now().to_rfc3339(),
                },
            ],
            tags: vec![],
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };

        assert!(rule.has_access("owner", Permission::Admin));
        assert!(rule.has_access("viewer", Permission::View));
        assert!(!rule.has_access("viewer", Permission::Edit));
        assert!(rule.has_access("editor", Permission::Edit));
        assert!(rule.has_access("editor", Permission::View));
        assert!(!rule.has_access("stranger", Permission::View));
    }
}
