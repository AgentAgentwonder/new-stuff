use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TokenTransaction {
    pub source: String,
    pub target: String,
    pub amount: f64,
    pub timestamp: i64,
    pub token_address: String,
    pub transaction_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TokenFlowNode {
    pub id: String,
    pub address: String,
    pub label: Option<String>,
    pub balance: f64,
    pub kind: NodeKind,
    pub cluster_id: Option<String>,
    pub risk: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Source,
    Destination,
    Intermediate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TokenFlowEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub amount: f64,
    pub timestamp: i64,
    pub token_address: String,
    pub transaction_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TimeRange {
    pub start: i64,
    pub end: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TokenFlowGraph {
    pub nodes: Vec<TokenFlowNode>,
    pub edges: Vec<TokenFlowEdge>,
    pub token_address: String,
    pub time_range: TimeRange,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ClusterPerformance {
    pub total_pnl: f64,
    pub win_rate: f64,
    pub average_hold_time: f64,
    pub top_tokens: Vec<TopToken>,
    pub distribution_pattern: DistributionPattern,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TopToken {
    pub address: String,
    pub symbol: String,
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DistributionPattern {
    Accumulation,
    Distribution,
    Neutral,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WalletCluster {
    pub id: String,
    pub wallets: Vec<String>,
    pub total_volume: f64,
    pub transaction_count: usize,
    pub first_seen: i64,
    pub last_seen: i64,
    pub performance: ClusterPerformance,
    pub risk: RiskLevel,
    pub suspicious: bool,
    pub suspicion_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CircularFlow {
    pub id: String,
    pub wallets: Vec<String>,
    pub amount: f64,
    pub token_address: String,
    pub cycles: usize,
    pub confidence: f64,
    pub detected_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WashTradingPattern {
    pub id: String,
    pub wallets: Vec<String>,
    pub token_address: String,
    pub volume: f64,
    pub transaction_count: usize,
    pub confidence: f64,
    pub detected_at: i64,
    pub pattern: WashTradingPatternKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WashTradingPatternKind {
    PingPong,
    Circular,
    Layered,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TokenFlowAlert {
    pub id: String,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub title: String,
    pub description: String,
    pub cluster_id: Option<String>,
    pub wallets: Vec<String>,
    pub token_address: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub timestamp: i64,
    pub acknowledged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AlertType {
    CircularFlow,
    WashTrading,
    NewClusterMember,
    DistributionChange,
    SuspiciousPattern,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ClusterSubscription {
    pub id: String,
    pub cluster_id: String,
    pub alerts: ClusterSubscriptionAlerts,
    pub notification_channels: Vec<NotificationChannel>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ClusterSubscriptionAlerts {
    pub new_members: bool,
    pub suspicious_flows: bool,
    pub performance_changes: bool,
    pub distribution_changes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationChannel {
    Ui,
    Email,
    Webhook,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SankeyNode {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SankeyLink {
    pub source: usize,
    pub target: usize,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SankeyData {
    pub nodes: Vec<SankeyNode>,
    pub links: Vec<SankeyLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FlowSnapshot {
    pub id: String,
    pub timestamp: i64,
    pub graph: TokenFlowGraph,
    pub clusters: Vec<WalletCluster>,
    pub alerts: Vec<TokenFlowAlert>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TimelineFrame {
    pub timestamp: i64,
    pub flows: Vec<TokenFlowEdge>,
    pub active_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FlowExportMetadata {
    pub exported_at: i64,
    pub time_range: TimeRange,
    pub filters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FlowExportData {
    pub format: FlowExportFormat,
    pub data: FlowExportContent,
    pub metadata: FlowExportMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FlowExportFormat {
    Json,
    Csv,
    Png,
    Svg,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FlowExportContent {
    pub graph: Option<TokenFlowGraph>,
    pub clusters: Option<Vec<WalletCluster>>,
    pub alerts: Option<Vec<TokenFlowAlert>>,
    pub snapshot: Option<String>,
}
