// DeFi Integration Types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapQuote {
    pub input_mint: String,
    pub output_mint: String,
    pub input_amount: u64,
    pub output_amount: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum DefiError {
    #[error("defi error: {0}")]
    General(String),
}

pub type DefiResult<T> = Result<T, DefiError>;

// Protocol enum for DeFi platforms
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Protocol {
    Solend,
    MarginFi,
    Kamino,
    Raydium,
    Orca,
    Other(String),
}

// Position type enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PositionType {
    Lending,
    Borrowing,
    LiquidityPool,
    Staking,
    Farming,
}

// Risk level enum for DeFi positions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

// Reward structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reward {
    pub token: String,
    pub amount: f64,
    pub value_usd: f64,
}

// DeFi position structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeFiPosition {
    pub id: String,
    pub protocol: Protocol,
    pub position_type: PositionType,
    pub asset: String,
    pub amount: f64,
    pub value_usd: f64,
    pub apy: f64,
    pub rewards: Vec<Reward>,
    pub health_factor: Option<f64>,
    pub created_at: i64,
    pub last_updated: i64,
}

// Portfolio summary structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioSummary {
    pub total_value_usd: f64,
    pub lending_value: f64,
    pub borrowing_value: f64,
    pub lp_value: f64,
    pub staking_value: f64,
    pub farming_value: f64,
    pub total_earnings_24h: f64,
    pub average_apy: f64,
    pub positions: Vec<DeFiPosition>,
}

// Risk metrics structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskMetrics {
    pub position_id: String,
    pub risk_level: RiskLevel,
    pub liquidation_price: Option<f64>,
    pub health_factor: Option<f64>,
    pub collateral_ratio: Option<f64>,
    pub warnings: Vec<String>,
}

// Auto-compound settings structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoCompoundSettings {
    pub position_id: String,
    pub enabled: bool,
    pub threshold: f64,
    pub frequency: u64,
    pub slippage_tolerance: f64,
    pub gas_limit: u64,
}

// Governance proposal status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Executed,
    Cancelled,
}

// Governance proposal structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceProposal {
    pub id: String,
    pub protocol: Protocol,
    pub title: String,
    pub description: String,
    pub proposer: String,
    pub start_time: i64,
    pub end_time: i64,
    pub votes_for: f64,
    pub votes_against: f64,
    pub votes_abstain: f64,
    pub status: ProposalStatus,
    pub quorum: f64,
    pub execution_eta: Option<i64>,
}

// Yield position structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YieldPosition {
    pub id: String,
    pub protocol: Protocol,
    pub asset: String,
    pub deposited_amount: f64,
    pub current_value_usd: f64,
    pub apy: f64,
    pub rewards_earned: Vec<Reward>,
    pub start_date: i64,
    pub last_compounded: Option<i64>,
}

// Impermanent loss data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpermanentLossData {
    pub lp_position_id: String,
    pub initial_value_usd: f64,
    pub current_value_usd: f64,
    pub hold_value_usd: f64,
    pub il_percentage: f64,
    pub il_usd: f64,
    pub fees_earned_usd: f64,
    pub net_result_usd: f64,
    pub calculation_time: i64,
}

// LP analytics structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LpAnalytics {
    pub position_id: String,
    pub pool_address: String,
    pub token_a: String,
    pub token_b: String,
    pub liquidity_provided: f64,
    pub current_value_usd: f64,
    pub fees_earned_24h: f64,
    pub fees_earned_7d: f64,
    pub fees_earned_total: f64,
    pub il_current: ImpermanentLossData,
    pub apy_7d: f64,
    pub apy_30d: f64,
    pub price_range: Option<PriceRange>,
    pub in_range: bool,
}

// Price range structure for concentrated liquidity
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceRange {
    pub min_price: f64,
    pub max_price: f64,
    pub current_price: f64,
    pub optimal_min: Option<f64>,
    pub optimal_max: Option<f64>,
}

// Lending pool structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LendingPool {
    pub pool_address: String,
    pub protocol: Protocol,
    pub asset: String,
    pub total_supply: f64,
    pub total_borrowed: f64,
    pub supply_apy: f64,
    pub borrow_apy: f64,
    pub utilization_rate: f64,
    pub liquidation_threshold: f64,
    pub liquidation_bonus: f64,
}

// Yield farm structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YieldFarm {
    pub farm_address: String,
    pub protocol: Protocol,
    pub lp_token: String,
    pub reward_tokens: Vec<String>,
    pub tvl_usd: f64,
    pub base_apy: f64,
    pub reward_apy: f64,
    pub total_apy: f64,
    pub deposit_fee: f64,
    pub withdrawal_fee: f64,
    pub lock_period: Option<u64>,
    pub risk_score: u8,
}

// Staking pool structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StakingPool {
    pub pool_address: String,
    pub protocol: Protocol,
    pub stake_token: String,
    pub reward_token: String,
    pub total_staked: f64,
    pub apy: f64,
    pub lock_duration: Option<u64>,
    pub early_withdrawal_penalty: f64,
    pub minimum_stake: f64,
}
