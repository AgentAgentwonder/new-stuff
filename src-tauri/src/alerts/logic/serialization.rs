use super::rule_engine::AlertRule;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleExport {
    pub version: String,
    pub exported_at: String,
    pub rule: AlertRule,
    pub metadata: ExportMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportMetadata {
    pub exported_by: Option<String>,
    pub application: String,
    pub format_version: String,
}

#[derive(Debug, thiserror::Error)]
pub enum SerializationError {
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid rule format: {0}")]
    InvalidFormat(String),

    #[error("Version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: String, actual: String },
}

pub fn serialize_rule_to_json(rule: &AlertRule) -> Result<String, SerializationError> {
    let export = RuleExport {
        version: "1.0.0".to_string(),
        exported_at: chrono::Utc::now().to_rfc3339(),
        rule: rule.clone(),
        metadata: ExportMetadata {
            exported_by: rule.owner_id.clone(),
            application: "Smart Alerts Builder".to_string(),
            format_version: "1.0".to_string(),
        },
    };

    serde_json::to_string_pretty(&export).map_err(SerializationError::from)
}

pub fn deserialize_rule_from_json(json: &str) -> Result<AlertRule, SerializationError> {
    let export: RuleExport = serde_json::from_str(json)?;

    if export.metadata.format_version != "1.0" {
        return Err(SerializationError::VersionMismatch {
            expected: "1.0".to_string(),
            actual: export.metadata.format_version,
        });
    }

    Ok(export.rule)
}

pub fn export_rule_to_file<P: AsRef<Path>>(
    rule: &AlertRule,
    path: P,
) -> Result<(), SerializationError> {
    let json = serialize_rule_to_json(rule)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn import_rule_from_file<P: AsRef<Path>>(path: P) -> Result<AlertRule, SerializationError> {
    let json = fs::read_to_string(path)?;
    deserialize_rule_from_json(&json)
}

pub fn serialize_rules_batch(rules: &[AlertRule]) -> Result<String, SerializationError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct BatchExport {
        version: String,
        exported_at: String,
        count: usize,
        rules: Vec<AlertRule>,
    }

    let batch = BatchExport {
        version: "1.0.0".to_string(),
        exported_at: chrono::Utc::now().to_rfc3339(),
        count: rules.len(),
        rules: rules.to_vec(),
    };

    serde_json::to_string_pretty(&batch).map_err(SerializationError::from)
}

pub fn deserialize_rules_batch(json: &str) -> Result<Vec<AlertRule>, SerializationError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct BatchExport {
        #[allow(dead_code)]
        version: String,
        #[allow(dead_code)]
        exported_at: String,
        #[allow(dead_code)]
        count: usize,
        rules: Vec<AlertRule>,
    }

    let batch: BatchExport = serde_json::from_str(json)?;
    Ok(batch.rules)
}

pub fn validate_rule_json(json: &str) -> Result<bool, SerializationError> {
    let _: RuleExport = serde_json::from_str(json)?;
    Ok(true)
}

pub fn rule_to_compact_json(rule: &AlertRule) -> Result<String, SerializationError> {
    serde_json::to_string(rule).map_err(SerializationError::from)
}

pub fn rule_from_compact_json(json: &str) -> Result<AlertRule, SerializationError> {
    serde_json::from_str(json).map_err(SerializationError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alerts::logic::conditions::{Condition, ConditionParameters, ConditionType};
    use crate::alerts::logic::rule_engine::{LogicalOperator, RuleGroup, RuleNode};
    use chrono::Utc;

    #[test]
    fn test_serialize_deserialize_rule() {
        let rule = AlertRule {
            id: "test-1".to_string(),
            name: "Test Rule".to_string(),
            description: Some("Test serialization".to_string()),
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
            actions: vec![],
            enabled: true,
            symbol: Some("SOL".to_string()),
            owner_id: Some("user1".to_string()),
            shared_with: vec![],
            tags: vec!["test".to_string()],
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };

        let json = serialize_rule_to_json(&rule).expect("Serialization failed");
        let deserialized = deserialize_rule_from_json(&json).expect("Deserialization failed");

        assert_eq!(rule.id, deserialized.id);
        assert_eq!(rule.name, deserialized.name);
        assert_eq!(rule.enabled, deserialized.enabled);
    }

    #[test]
    fn test_serialize_batch_rules() {
        let rules = vec![
            AlertRule {
                id: "rule-1".to_string(),
                name: "Rule 1".to_string(),
                description: None,
                rule_tree: RuleNode {
                    id: None,
                    condition: None,
                    group: None,
                },
                actions: vec![],
                enabled: true,
                symbol: None,
                owner_id: None,
                shared_with: vec![],
                tags: vec![],
                created_at: Utc::now().to_rfc3339(),
                updated_at: Utc::now().to_rfc3339(),
            },
            AlertRule {
                id: "rule-2".to_string(),
                name: "Rule 2".to_string(),
                description: None,
                rule_tree: RuleNode {
                    id: None,
                    condition: None,
                    group: None,
                },
                actions: vec![],
                enabled: true,
                symbol: None,
                owner_id: None,
                shared_with: vec![],
                tags: vec![],
                created_at: Utc::now().to_rfc3339(),
                updated_at: Utc::now().to_rfc3339(),
            },
        ];

        let json = serialize_rules_batch(&rules).expect("Batch serialization failed");
        let deserialized = deserialize_rules_batch(&json).expect("Batch deserialization failed");

        assert_eq!(rules.len(), deserialized.len());
        assert_eq!(rules[0].id, deserialized[0].id);
        assert_eq!(rules[1].id, deserialized[1].id);
    }

    #[test]
    fn test_compact_json() {
        let rule = AlertRule {
            id: "compact-test".to_string(),
            name: "Compact Test".to_string(),
            description: None,
            rule_tree: RuleNode {
                id: None,
                condition: None,
                group: None,
            },
            actions: vec![],
            enabled: true,
            symbol: None,
            owner_id: None,
            shared_with: vec![],
            tags: vec![],
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };

        let compact = rule_to_compact_json(&rule).expect("Compact serialization failed");
        let deserialized =
            rule_from_compact_json(&compact).expect("Compact deserialization failed");

        assert_eq!(rule.id, deserialized.id);
        assert!(compact.len() < serialize_rule_to_json(&rule).unwrap().len());
    }
}
