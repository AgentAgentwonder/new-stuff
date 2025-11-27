pub mod launch_predictor;
pub use launch_predictor::*;

use crate::security::keystore::Keystore;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RiskFeatures {
    // Holder concentration features
    pub gini_coefficient: f64,
    pub top_10_percentage: f64,
    pub total_holders: u64,

    // Liquidity features
    pub liquidity_usd: f64,
    pub liquidity_to_mcap_ratio: f64,

    // Developer features
    pub has_mint_authority: bool,
    pub has_freeze_authority: bool,
    pub verified: bool,
    pub audited: bool,

    // Sentiment features
    pub community_trust_score: f64,
    pub sentiment_score: f64,

    // Age and activity features
    pub token_age_days: f64,
    pub volume_24h: f64,
    pub price_volatility: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RiskScore {
    pub token_address: String,
    pub score: f64,         // 0-100 scale (0 = safe, 100 = very risky)
    pub risk_level: String, // "Low", "Medium", "High", "Critical"
    pub contributing_factors: Vec<RiskFactor>,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RiskFactor {
    pub factor_name: String,
    pub impact: f64,      // Contribution to risk score
    pub severity: String, // "Low", "Medium", "High"
    pub description: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RiskHistory {
    pub token_address: String,
    pub history: Vec<RiskHistoryPoint>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RiskHistoryPoint {
    pub timestamp: String,
    pub score: f64,
    pub risk_level: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RiskModel {
    // Logistic regression weights for each feature
    weights: HashMap<String, f64>,
    intercept: f64,
    threshold: f64,
}

impl RiskModel {
    pub fn new() -> Self {
        // Pre-trained weights based on common rug pull patterns
        let mut weights = HashMap::new();

        // High concentration = high risk
        weights.insert("gini_coefficient".to_string(), 30.0);
        weights.insert("top_10_percentage".to_string(), 0.5);

        // Low holder count = higher risk
        weights.insert("holder_count_inverse".to_string(), 15.0);

        // Low liquidity = high risk
        weights.insert("liquidity_score".to_string(), -20.0);

        // Mint/freeze authority = high risk
        weights.insert("mint_authority".to_string(), 25.0);
        weights.insert("freeze_authority".to_string(), 20.0);

        // Verification reduces risk
        weights.insert("verified".to_string(), -15.0);
        weights.insert("audited".to_string(), -20.0);

        // Community trust reduces risk
        weights.insert("community_trust".to_string(), -10.0);

        // Negative sentiment = higher risk
        weights.insert("sentiment".to_string(), -5.0);

        // Very new tokens = higher risk
        weights.insert("age_score".to_string(), -8.0);

        // High volatility = higher risk
        weights.insert("volatility".to_string(), 12.0);

        Self {
            weights,
            intercept: 50.0, // Base risk score
            threshold: 0.5,
        }
    }

    pub fn from_weights(weights: HashMap<String, f64>, intercept: f64) -> Self {
        Self {
            weights,
            intercept,
            threshold: 0.5,
        }
    }

    pub fn score_token(&self, features: &RiskFeatures) -> (f64, Vec<RiskFactor>) {
        let mut feature_map = HashMap::new();
        let mut contributing_factors = Vec::new();

        // Transform features into model inputs
        feature_map.insert(
            "gini_coefficient",
            features.gini_coefficient.clamp(0.0, 1.0),
        );

        let normalized_top10 = (features.top_10_percentage / 100.0).clamp(0.0, 1.0);
        feature_map.insert("top_10_percentage", normalized_top10);

        let holder_diversity = (((features.total_holders as f64).max(1.0) + 10.0).ln()).max(1.0);
        let holder_count_inverse = (1.0 / holder_diversity).clamp(0.0, 5.0);
        feature_map.insert("holder_count_inverse", holder_count_inverse);

        // Liquidity score (normalized)
        let liquidity_value = features.liquidity_usd.max(1.0);
        let liquidity_score = (liquidity_value.ln() / 20.0).clamp(0.0, 1.0);
        feature_map.insert("liquidity_score", liquidity_score);

        let liquidity_ratio = features.liquidity_to_mcap_ratio.clamp(0.0, 1.0);
        feature_map.insert("liquidity_to_mcap", liquidity_ratio);

        // Authority flags
        feature_map.insert(
            "mint_authority",
            if features.has_mint_authority {
                1.0
            } else {
                0.0
            },
        );
        feature_map.insert(
            "freeze_authority",
            if features.has_freeze_authority {
                1.0
            } else {
                0.0
            },
        );

        // Verification
        feature_map.insert("verified", if features.verified { 1.0 } else { 0.0 });
        feature_map.insert("audited", if features.audited { 1.0 } else { 0.0 });

        // Community and sentiment
        feature_map.insert(
            "community_trust",
            features.community_trust_score.clamp(0.0, 1.0),
        );
        feature_map.insert("sentiment", features.sentiment_score.clamp(-1.0, 1.0));

        // Age score (tokens older than 30 days get lower risk)
        let age_score = (features.token_age_days / 30.0).clamp(0.0, 1.0);
        feature_map.insert("age_score", age_score);

        // Volatility (normalized)
        let volatility_score = (features.price_volatility / 100.0).clamp(0.0, 1.0);
        feature_map.insert("volatility", volatility_score);

        // Calculate weighted score
        let mut score = self.intercept;
        let mut factor_contributions = Vec::new();

        for (feature_name, feature_value) in &feature_map {
            if let Some(&weight) = self.weights.get(*feature_name) {
                let contribution = weight * feature_value;
                score += contribution;

                factor_contributions.push((
                    feature_name.to_string(),
                    contribution.abs(),
                    contribution,
                ));
            }
        }

        // Clamp score to 0-100
        score = score.max(0.0).min(100.0);

        // Sort factors by absolute contribution
        factor_contributions
            .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Create top contributing factors
        for (factor_name, abs_contrib, raw_contrib) in factor_contributions.iter().take(5) {
            if *abs_contrib > 1.0 {
                let (description, increases_risk) = match factor_name.as_str() {
                    "gini_coefficient" => ("High holder concentration detected", true),
                    "top_10_percentage" => ("Top 10 holders control significant supply", true),
                    "holder_count_inverse" => ("Low number of holders", true),
                    "liquidity_score" => ("Liquidity level", false),
                    "mint_authority" => ("Mint authority not revoked", true),
                    "freeze_authority" => ("Freeze authority not revoked", true),
                    "verified" => ("Token verification status", false),
                    "audited" => ("Security audit status", false),
                    "community_trust" => ("Community trust score", false),
                    "sentiment" => ("Market sentiment", false),
                    "age_score" => ("Token age", false),
                    "volatility" => ("Price volatility", true),
                    _ => ("Unknown factor", true),
                };

                let severity = if *abs_contrib > 15.0 {
                    "High"
                } else if *abs_contrib > 8.0 {
                    "Medium"
                } else {
                    "Low"
                };

                contributing_factors.push(RiskFactor {
                    factor_name: factor_name.clone(),
                    impact: *abs_contrib,
                    severity: severity.to_string(),
                    description: description.to_string(),
                });
            }
        }

        (score, contributing_factors)
    }

    // Serialize model to JSON for persistence
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }

    // Deserialize model from JSON
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| e.to_string())
    }
}

#[derive(Clone)]
pub struct RiskAnalyzer {
    pool: Pool<Sqlite>,
    model: Arc<RwLock<RiskModel>>,
}

pub type SharedRiskAnalyzer = Arc<RwLock<RiskAnalyzer>>;

impl RiskAnalyzer {
    pub async fn new(app: &AppHandle) -> Result<Self, sqlx::Error> {
        let mut db_path = app.path().app_data_dir().map_err(|_| {
            sqlx::Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "App data dir not found",
            ))
        })?;

        std::fs::create_dir_all(&db_path).map_err(sqlx::Error::Io)?;
        db_path.push("risk_scores.db");

        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&db_url).await?;

        Self::with_pool(pool).await
    }

    pub async fn with_pool(pool: Pool<Sqlite>) -> Result<Self, sqlx::Error> {
        let model = Arc::new(RwLock::new(RiskModel::new()));
        let analyzer = Self { pool, model };
        analyzer.initialize().await?;
        Ok(analyzer)
    }

    async fn initialize(&self) -> Result<(), sqlx::Error> {
        // Create risk scores table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS risk_scores (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                token_address TEXT NOT NULL,
                score REAL NOT NULL,
                risk_level TEXT NOT NULL,
                factors TEXT NOT NULL,
                timestamp TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create index
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_risk_scores_token_timestamp 
            ON risk_scores(token_address, timestamp DESC);
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create model storage table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS risk_models (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                version INTEGER NOT NULL,
                model_data TEXT NOT NULL,
                metrics TEXT,
                created_at TEXT NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 0
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn score_token(
        &self,
        token_address: &str,
        features: RiskFeatures,
    ) -> Result<RiskScore, sqlx::Error> {
        let model = self.model.read().await;
        let (score, factors) = model.score_token(&features);

        let risk_level = if score < 30.0 {
            "Low"
        } else if score < 60.0 {
            "Medium"
        } else if score < 80.0 {
            "High"
        } else {
            "Critical"
        };

        let risk_score = RiskScore {
            token_address: token_address.to_string(),
            score,
            risk_level: risk_level.to_string(),
            contributing_factors: factors.clone(),
            timestamp: Utc::now().to_rfc3339(),
        };

        // Store in database
        let factors_json = serde_json::to_string(&factors).unwrap_or_default();
        sqlx::query(
            r#"
            INSERT INTO risk_scores (token_address, score, risk_level, factors, timestamp)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&risk_score.token_address)
        .bind(risk_score.score)
        .bind(&risk_score.risk_level)
        .bind(&factors_json)
        .bind(&risk_score.timestamp)
        .execute(&self.pool)
        .await?;

        Ok(risk_score)
    }

    pub async fn get_risk_history(
        &self,
        token_address: &str,
        days: u32,
    ) -> Result<RiskHistory, sqlx::Error> {
        let cutoff = Utc::now() - chrono::Duration::days(days as i64);
        let cutoff_str = cutoff.to_rfc3339();

        let rows = sqlx::query(
            r#"
            SELECT score, risk_level, timestamp
            FROM risk_scores
            WHERE token_address = ? AND timestamp >= ?
            ORDER BY timestamp ASC
            "#,
        )
        .bind(token_address)
        .bind(&cutoff_str)
        .fetch_all(&self.pool)
        .await?;

        let history: Vec<RiskHistoryPoint> = rows
            .iter()
            .map(|row| RiskHistoryPoint {
                timestamp: row.get("timestamp"),
                score: row.get("score"),
                risk_level: row.get("risk_level"),
            })
            .collect();

        Ok(RiskHistory {
            token_address: token_address.to_string(),
            history,
        })
    }

    pub async fn get_latest_risk_score(
        &self,
        token_address: &str,
    ) -> Result<Option<RiskScore>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT score, risk_level, factors, timestamp
            FROM risk_scores
            WHERE token_address = ?
            ORDER BY timestamp DESC
            LIMIT 1
            "#,
        )
        .bind(token_address)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let factors_json: String = row.get("factors");
            let factors: Vec<RiskFactor> = serde_json::from_str(&factors_json).unwrap_or_default();

            Ok(Some(RiskScore {
                token_address: token_address.to_string(),
                score: row.get("score"),
                risk_level: row.get("risk_level"),
                contributing_factors: factors,
                timestamp: row.get("timestamp"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn save_model(&self, metrics: Option<String>) -> Result<(), sqlx::Error> {
        let model = self.model.read().await;
        let model_json = model.to_json().map_err(|e| {
            sqlx::Error::Decode(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e,
            )))
        })?;

        // Mark all existing models as inactive
        sqlx::query("UPDATE risk_models SET is_active = 0")
            .execute(&self.pool)
            .await?;

        // Insert new model as active
        sqlx::query(
            r#"
            INSERT INTO risk_models (version, model_data, metrics, created_at, is_active)
            VALUES (
                (SELECT COALESCE(MAX(version), 0) + 1 FROM risk_models),
                ?, ?, ?, 1
            )
            "#,
        )
        .bind(&model_json)
        .bind(metrics)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn load_latest_model(&self) -> Result<(), sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT model_data FROM risk_models
            WHERE is_active = 1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let model_json: String = row.get("model_data");
            let loaded_model = RiskModel::from_json(&model_json).map_err(|e| {
                sqlx::Error::Decode(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e,
                )))
            })?;

            let mut model = self.model.write().await;
            *model = loaded_model;
        }

        Ok(())
    }
}

// Legacy command for backward compatibility
#[tauri::command]
pub async fn assess_risk(features: Vec<f32>) -> Result<f32, String> {
    if features.is_empty() {
        return Err("Features cannot be empty".to_string());
    }

    let _model = RiskModel::new();
    let weights = vec![0.3, 0.2, 0.15, 0.15, 0.1, 0.1];

    let score: f32 = features
        .iter()
        .zip(weights.iter())
        .take(features.len().min(weights.len()))
        .map(|(f, w)| f * w)
        .sum();

    Ok(score.max(0.0).min(1.0))
}

// New ML-based commands
#[tauri::command]
pub async fn get_token_risk_score(
    token_address: String,
    risk_analyzer: State<'_, SharedRiskAnalyzer>,
    holder_analyzer: State<'_, crate::market::SharedHolderAnalyzer>,
) -> Result<RiskScore, String> {
    // Gather features from various sources
    let holder_data = {
        let analyzer = holder_analyzer.read().await;
        analyzer
            .get_holder_distribution(&token_address)
            .await
            .map_err(|e| format!("Failed to get holder data: {}", e))?
    };

    let metadata = {
        let analyzer = holder_analyzer.read().await;
        analyzer
            .get_token_metadata(&token_address)
            .await
            .map_err(|e| format!("Failed to get metadata: {}", e))?
    };

    let verification = {
        let analyzer = holder_analyzer.read().await;
        analyzer
            .get_verification_status(&token_address)
            .await
            .map_err(|e| format!("Failed to get verification: {}", e))?
    };

    // Calculate token age
    let token_age_days = {
        let creation_date = chrono::DateTime::parse_from_rfc3339(&metadata.creation_date)
            .map_err(|e| format!("Failed to parse creation date: {}", e))?
            .with_timezone(&Utc);
        let now = Utc::now();
        (now - creation_date).num_days() as f64
    };

    // Build features
    let features = RiskFeatures {
        gini_coefficient: holder_data.gini_coefficient,
        top_10_percentage: holder_data.top_10_percentage,
        total_holders: holder_data.total_holders,
        liquidity_usd: 100000.0,      // Mock - would fetch from market data
        liquidity_to_mcap_ratio: 0.1, // Mock
        has_mint_authority: metadata.mint_authority.is_some(),
        has_freeze_authority: metadata.freeze_authority.is_some(),
        verified: verification.verified,
        audited: verification.audit_status == "Audited",
        community_trust_score: verification.community_votes.trust_score,
        sentiment_score: 0.0, // Mock - would fetch from sentiment analysis
        token_age_days,
        volume_24h: 50000.0,    // Mock
        price_volatility: 15.0, // Mock
    };

    let analyzer = risk_analyzer.read().await;
    let risk_score = analyzer
        .score_token(&token_address, features)
        .await
        .map_err(|e| format!("Failed to score token: {}", e))?;

    Ok(risk_score)
}

#[tauri::command]
pub async fn get_risk_history(
    token_address: String,
    days: u32,
    risk_analyzer: State<'_, SharedRiskAnalyzer>,
) -> Result<RiskHistory, String> {
    let analyzer = risk_analyzer.read().await;
    analyzer
        .get_risk_history(&token_address, days)
        .await
        .map_err(|e| format!("Failed to get risk history: {}", e))
}

#[tauri::command]
pub async fn get_latest_risk_score(
    token_address: String,
    risk_analyzer: State<'_, SharedRiskAnalyzer>,
) -> Result<Option<RiskScore>, String> {
    let analyzer = risk_analyzer.read().await;
    analyzer
        .get_latest_risk_score(&token_address)
        .await
        .map_err(|e| format!("Failed to get latest risk score: {}", e))
}

// ==================== LLM Integration ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LLMProvider {
    Claude,
    GPT4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LLMConfig {
    pub provider: LLMProvider,
    pub model: String,
    pub api_key: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradingContext {
    pub portfolio: Option<serde_json::Value>,
    pub active_alerts: Vec<String>,
    pub market_data: HashMap<String, serde_json::Value>,
    pub recent_trades: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    pub id: String,
    pub user_id: String,
    pub messages: Vec<Message>,
    pub context: TradingContext,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatRequest {
    pub conversation_id: Option<String>,
    pub message: String,
    pub include_context: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatResponse {
    pub conversation_id: String,
    pub message: String,
    pub function_calls: Vec<FunctionCall>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageStats {
    pub requests_count: u32,
    pub tokens_used: u64,
    pub last_request_at: String,
    pub window_start: String,
}

// Claude API request/response structures
#[derive(Debug, Serialize, Deserialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ClaudeTool>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClaudeTool {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClaudeResponse {
    id: String,
    content: Vec<ClaudeContent>,
    #[serde(default)]
    stop_reason: String,
    usage: ClaudeUsage,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ClaudeContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct ClaudeUsage {
    input_tokens: u32,
    output_tokens: u32,
}

// GPT-4 API request/response structures
#[derive(Debug, Serialize, Deserialize)]
struct GPTRequest {
    model: String,
    messages: Vec<GPTMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<GPTTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GPTMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GPTTool {
    #[serde(rename = "type")]
    tool_type: String,
    function: GPTFunction,
}

#[derive(Debug, Serialize, Deserialize)]
struct GPTFunction {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct GPTResponse {
    id: String,
    choices: Vec<GPTChoice>,
    usage: GPTUsage,
}

#[derive(Debug, Serialize, Deserialize)]
struct GPTChoice {
    message: GPTResponseMessage,
    finish_reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GPTResponseMessage {
    role: String,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<GPTToolCall>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GPTToolCall {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: GPTFunctionCall,
}

#[derive(Debug, Serialize, Deserialize)]
struct GPTFunctionCall {
    name: String,
    arguments: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GPTUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

pub struct LLMClient {
    config: LLMConfig,
    http_client: reqwest::Client,
    anthropic_url: String,
    openai_url: String,
}

impl LLMClient {
    pub fn new(config: LLMConfig) -> Self {
        Self {
            config,
            http_client: reqwest::Client::new(),
            anthropic_url: "https://api.anthropic.com/v1/messages".to_string(),
            openai_url: "https://api.openai.com/v1/chat/completions".to_string(),
        }
    }

    #[cfg(test)]
    pub fn new_with_endpoints(
        config: LLMConfig,
        anthropic_url: String,
        openai_url: String,
    ) -> Self {
        Self {
            config,
            http_client: reqwest::Client::new(),
            anthropic_url,
            openai_url,
        }
    }

    pub async fn chat(
        &self,
        messages: Vec<Message>,
        system_prompt: Option<String>,
        functions: Vec<FunctionDefinition>,
    ) -> Result<ChatResponse, String> {
        match self.config.provider {
            LLMProvider::Claude => self.chat_claude(messages, system_prompt, functions).await,
            LLMProvider::GPT4 => self.chat_gpt4(messages, system_prompt, functions).await,
        }
    }

    async fn chat_claude(
        &self,
        messages: Vec<Message>,
        system_prompt: Option<String>,
        functions: Vec<FunctionDefinition>,
    ) -> Result<ChatResponse, String> {
        let tools: Vec<ClaudeTool> = functions
            .iter()
            .map(|f| ClaudeTool {
                name: f.name.clone(),
                description: f.description.clone(),
                input_schema: f.parameters.clone(),
            })
            .collect();

        let claude_messages: Vec<ClaudeMessage> = messages
            .iter()
            .map(|m| ClaudeMessage {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect();

        let request = ClaudeRequest {
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens,
            messages: claude_messages,
            system: system_prompt,
            temperature: Some(self.config.temperature),
            tools: if tools.is_empty() { None } else { Some(tools) },
        };

        let response = self
            .http_client
            .post(&self.anthropic_url)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Claude API request failed: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Claude API error: {}", error_text));
        }

        let claude_response: ClaudeResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Claude response: {}", e))?;

        let mut message_text = String::new();
        let mut function_calls = Vec::new();

        for content in claude_response.content {
            match content {
                ClaudeContent::Text { text } => {
                    message_text.push_str(&text);
                }
                ClaudeContent::ToolUse { name, input, .. } => {
                    function_calls.push(FunctionCall {
                        name,
                        arguments: input,
                    });
                }
            }
        }

        Ok(ChatResponse {
            conversation_id: claude_response.id,
            message: message_text,
            function_calls,
            timestamp: Utc::now().to_rfc3339(),
        })
    }

    async fn chat_gpt4(
        &self,
        messages: Vec<Message>,
        system_prompt: Option<String>,
        functions: Vec<FunctionDefinition>,
    ) -> Result<ChatResponse, String> {
        let mut gpt_messages: Vec<GPTMessage> = Vec::new();

        if let Some(system) = system_prompt {
            gpt_messages.push(GPTMessage {
                role: "system".to_string(),
                content: system,
            });
        }

        for msg in messages {
            gpt_messages.push(GPTMessage {
                role: msg.role,
                content: msg.content,
            });
        }

        let tools: Option<Vec<GPTTool>> = if functions.is_empty() {
            None
        } else {
            Some(
                functions
                    .iter()
                    .map(|f| GPTTool {
                        tool_type: "function".to_string(),
                        function: GPTFunction {
                            name: f.name.clone(),
                            description: f.description.clone(),
                            parameters: f.parameters.clone(),
                        },
                    })
                    .collect(),
            )
        };

        let request = GPTRequest {
            model: self.config.model.clone(),
            messages: gpt_messages,
            temperature: Some(self.config.temperature),
            max_tokens: Some(self.config.max_tokens),
            tools,
            tool_choice: None,
        };

        let response = self
            .http_client
            .post(&self.openai_url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("GPT-4 API request failed: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("GPT-4 API error: {}", error_text));
        }

        let gpt_response: GPTResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse GPT-4 response: {}", e))?;

        let choice = gpt_response
            .choices
            .first()
            .ok_or_else(|| "No response from GPT-4".to_string())?;

        let message = choice.message.content.clone().unwrap_or_default();
        let mut function_calls = Vec::new();

        if let Some(tool_calls) = &choice.message.tool_calls {
            for call in tool_calls {
                let arguments: serde_json::Value =
                    serde_json::from_str(&call.function.arguments).unwrap_or(serde_json::json!({}));
                function_calls.push(FunctionCall {
                    name: call.function.name.clone(),
                    arguments,
                });
            }
        }

        Ok(ChatResponse {
            conversation_id: gpt_response.id,
            message,
            function_calls,
            timestamp: Utc::now().to_rfc3339(),
        })
    }
}

// ==================== Conversation Manager ====================

pub struct ConversationManager {
    pool: Pool<Sqlite>,
}

impl ConversationManager {
    pub async fn new(app: &AppHandle) -> Result<Self, sqlx::Error> {
        let mut db_path = app.path().app_data_dir().map_err(|_| {
            sqlx::Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "App data dir not found",
            ))
        })?;

        std::fs::create_dir_all(&db_path).map_err(sqlx::Error::Io)?;
        db_path.push("conversations.db");

        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&db_url).await?;

        let manager = Self { pool };
        manager.initialize().await?;
        Ok(manager)
    }

    async fn initialize(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS conversations (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                context TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                conversation_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                FOREIGN KEY (conversation_id) REFERENCES conversations(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_messages_conversation 
            ON messages(conversation_id, timestamp);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_conversation(
        &self,
        user_id: &str,
        context: TradingContext,
    ) -> Result<String, sqlx::Error> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let context_json = serde_json::to_string(&context).unwrap_or_default();

        sqlx::query(
            r#"
            INSERT INTO conversations (id, user_id, context, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(user_id)
        .bind(&context_json)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(id)
    }

    pub async fn add_message(
        &self,
        conversation_id: &str,
        message: Message,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO messages (conversation_id, role, content, timestamp)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(conversation_id)
        .bind(&message.role)
        .bind(&message.content)
        .bind(&message.timestamp)
        .execute(&self.pool)
        .await?;

        // Update conversation timestamp
        sqlx::query(
            r#"
            UPDATE conversations SET updated_at = ? WHERE id = ?
            "#,
        )
        .bind(&message.timestamp)
        .bind(conversation_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_messages(
        &self,
        conversation_id: &str,
        limit: u32,
    ) -> Result<Vec<Message>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT role, content, timestamp
            FROM messages
            WHERE conversation_id = ?
            ORDER BY timestamp ASC
            LIMIT ?
            "#,
        )
        .bind(conversation_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .iter()
            .map(|row| Message {
                role: row.get("role"),
                content: row.get("content"),
                timestamp: row.get("timestamp"),
            })
            .collect())
    }

    pub async fn get_conversation(
        &self,
        conversation_id: &str,
    ) -> Result<Option<Conversation>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT user_id, context, created_at, updated_at
            FROM conversations
            WHERE id = ?
            "#,
        )
        .bind(conversation_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let context_json: String = row.get("context");
            let context: TradingContext =
                serde_json::from_str(&context_json).unwrap_or_else(|_| TradingContext {
                    portfolio: None,
                    active_alerts: Vec::new(),
                    market_data: HashMap::new(),
                    recent_trades: Vec::new(),
                });

            let messages = self.get_messages(conversation_id, 100).await?;

            Ok(Some(Conversation {
                id: conversation_id.to_string(),
                user_id: row.get("user_id"),
                messages,
                context,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_conversations(
        &self,
        user_id: &str,
        limit: u32,
    ) -> Result<Vec<Conversation>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id FROM conversations
            WHERE user_id = ?
            ORDER BY updated_at DESC
            LIMIT ?
            "#,
        )
        .bind(user_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        let mut conversations = Vec::new();
        for row in rows {
            let id: String = row.get("id");
            if let Some(conv) = self.get_conversation(&id).await? {
                conversations.push(conv);
            }
        }

        Ok(conversations)
    }

    pub async fn delete_conversation(&self, conversation_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM messages WHERE conversation_id = ?")
            .bind(conversation_id)
            .execute(&self.pool)
            .await?;

        sqlx::query("DELETE FROM conversations WHERE id = ?")
            .bind(conversation_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

// ==================== Usage Throttle ====================

pub struct UsageThrottle {
    pool: Pool<Sqlite>,
    max_requests_per_hour: u32,
    max_tokens_per_day: u64,
}

impl UsageThrottle {
    pub async fn new(
        app: &AppHandle,
        max_requests_per_hour: u32,
        max_tokens_per_day: u64,
    ) -> Result<Self, sqlx::Error> {
        let mut db_path = app.path().app_data_dir().map_err(|_| {
            sqlx::Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "App data dir not found",
            ))
        })?;

        std::fs::create_dir_all(&db_path).map_err(sqlx::Error::Io)?;
        db_path.push("usage.db");

        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&db_url).await?;

        let throttle = Self {
            pool,
            max_requests_per_hour,
            max_tokens_per_day,
        };
        throttle.initialize().await?;
        Ok(throttle)
    }

    async fn initialize(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS usage_stats (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL,
                requests_count INTEGER NOT NULL,
                tokens_used INTEGER NOT NULL,
                timestamp TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_usage_user_time 
            ON usage_stats(user_id, timestamp DESC);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn check_and_record(&self, user_id: &str, tokens: u64) -> Result<bool, sqlx::Error> {
        // Check hourly request limit
        let hour_ago = (Utc::now() - chrono::Duration::hours(1)).to_rfc3339();
        let hourly_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM usage_stats
            WHERE user_id = ? AND timestamp >= ?
            "#,
        )
        .bind(user_id)
        .bind(&hour_ago)
        .fetch_one(&self.pool)
        .await?;

        if hourly_count >= self.max_requests_per_hour as i64 {
            return Ok(false);
        }

        // Check daily token limit
        let day_ago = (Utc::now() - chrono::Duration::days(1)).to_rfc3339();
        let daily_tokens: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT SUM(tokens_used) FROM usage_stats
            WHERE user_id = ? AND timestamp >= ?
            "#,
        )
        .bind(user_id)
        .bind(&day_ago)
        .fetch_one(&self.pool)
        .await?;

        let total_tokens = daily_tokens.unwrap_or(0) as u64 + tokens;
        if total_tokens > self.max_tokens_per_day {
            return Ok(false);
        }

        // Record usage
        sqlx::query(
            r#"
            INSERT INTO usage_stats (user_id, requests_count, tokens_used, timestamp)
            VALUES (?, 1, ?, ?)
            "#,
        )
        .bind(user_id)
        .bind(tokens as i64)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(true)
    }

    pub async fn get_usage_stats(&self, user_id: &str) -> Result<UsageStats, sqlx::Error> {
        let hour_ago = (Utc::now() - chrono::Duration::hours(1)).to_rfc3339();
        let day_ago = (Utc::now() - chrono::Duration::days(1)).to_rfc3339();

        let hourly_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM usage_stats
            WHERE user_id = ? AND timestamp >= ?
            "#,
        )
        .bind(user_id)
        .bind(&hour_ago)
        .fetch_one(&self.pool)
        .await?;

        let daily_tokens: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT SUM(tokens_used) FROM usage_stats
            WHERE user_id = ? AND timestamp >= ?
            "#,
        )
        .bind(user_id)
        .bind(&day_ago)
        .fetch_one(&self.pool)
        .await?;

        let last_request: Option<String> = sqlx::query_scalar(
            r#"
            SELECT timestamp FROM usage_stats
            WHERE user_id = ?
            ORDER BY timestamp DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(UsageStats {
            requests_count: hourly_count as u32,
            tokens_used: daily_tokens.unwrap_or(0) as u64,
            last_request_at: last_request.unwrap_or_else(|| Utc::now().to_rfc3339()),
            window_start: hour_ago,
        })
    }
}

// ==================== AI Assistant ====================

pub struct AIAssistant {
    llm_client: Option<Arc<LLMClient>>,
    conversation_manager: Arc<ConversationManager>,
    usage_throttle: Arc<UsageThrottle>,
    functions: Vec<FunctionDefinition>,
}

pub type SharedAIAssistant = Arc<RwLock<AIAssistant>>;

impl AIAssistant {
    pub async fn new(app: &AppHandle, keystore: &Keystore) -> Result<Self, String> {
        // Try to retrieve API key from keystore (it may not exist yet)
        let llm_client = Self::load_llm_client(keystore);

        let conversation_manager = Arc::new(
            ConversationManager::new(app)
                .await
                .map_err(|e| format!("Failed to initialize conversation manager: {}", e))?,
        );

        let usage_throttle = Arc::new(
            UsageThrottle::new(app, 60, 100_000)
                .await
                .map_err(|e| format!("Failed to initialize usage throttle: {}", e))?,
        );

        let functions = Self::register_functions();

        Ok(Self {
            llm_client,
            conversation_manager,
            usage_throttle,
            functions,
        })
    }

    pub fn is_configured(&self) -> bool {
        self.llm_client.is_some()
    }

    fn load_llm_client(keystore: &Keystore) -> Option<Arc<LLMClient>> {
        keystore
            .retrieve_secret("llm_api_key")
            .ok()
            .and_then(|key| String::from_utf8(key.to_vec()).ok())
            .and_then(|api_key| {
                // Retrieve provider preference (default to Claude)
                let provider_str = keystore
                    .retrieve_secret("llm_provider")
                    .ok()
                    .and_then(|p| String::from_utf8(p.to_vec()).ok())
                    .unwrap_or_else(|| "claude".to_string());

                let provider = match provider_str.to_lowercase().as_str() {
                    "gpt4" | "gpt-4" | "openai" => LLMProvider::GPT4,
                    _ => LLMProvider::Claude,
                };

                let model = match provider {
                    LLMProvider::Claude => "claude-3-5-sonnet-20241022".to_string(),
                    LLMProvider::GPT4 => "gpt-4-turbo-preview".to_string(),
                };

                let config = LLMConfig {
                    provider,
                    model,
                    api_key,
                    max_tokens: 4096,
                    temperature: 0.7,
                };

                Some(Arc::new(LLMClient::new(config)))
            })
    }

    pub fn reload_llm_client(&mut self, keystore: &Keystore) {
        self.llm_client = Self::load_llm_client(keystore);
    }

    fn register_functions() -> Vec<FunctionDefinition> {
        vec![
            FunctionDefinition {
                name: "execute_trade".to_string(),
                description: "Execute a trade (buy/sell) for a given token".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "enum": ["buy", "sell"],
                            "description": "Trade action"
                        },
                        "token_address": {
                            "type": "string",
                            "description": "Token address to trade"
                        },
                        "amount": {
                            "type": "number",
                            "description": "Amount to trade"
                        },
                        "slippage": {
                            "type": "number",
                            "description": "Maximum slippage tolerance (percentage)"
                        }
                    },
                    "required": ["action", "token_address", "amount"]
                }),
            },
            FunctionDefinition {
                name: "create_alert".to_string(),
                description: "Create a price alert for a token".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "token_address": {
                            "type": "string",
                            "description": "Token address to monitor"
                        },
                        "condition": {
                            "type": "string",
                            "enum": ["above", "below"],
                            "description": "Alert condition"
                        },
                        "price": {
                            "type": "number",
                            "description": "Target price for alert"
                        },
                        "message": {
                            "type": "string",
                            "description": "Custom alert message"
                        }
                    },
                    "required": ["token_address", "condition", "price"]
                }),
            },
            FunctionDefinition {
                name: "get_portfolio_analytics".to_string(),
                description: "Get detailed portfolio analytics and performance metrics".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "timeframe": {
                            "type": "string",
                            "enum": ["24h", "7d", "30d", "all"],
                            "description": "Timeframe for analytics"
                        },
                        "include_breakdown": {
                            "type": "boolean",
                            "description": "Include per-token breakdown"
                        }
                    }
                }),
            },
            FunctionDefinition {
                name: "get_token_info".to_string(),
                description:
                    "Get detailed information about a token including price, volume, and risk score"
                        .to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "token_address": {
                            "type": "string",
                            "description": "Token address to query"
                        },
                        "include_risk": {
                            "type": "boolean",
                            "description": "Include risk score analysis"
                        }
                    },
                    "required": ["token_address"]
                }),
            },
            FunctionDefinition {
                name: "search_tokens".to_string(),
                description: "Search for tokens by name or symbol".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of results"
                        }
                    },
                    "required": ["query"]
                }),
            },
        ]
    }

    pub async fn chat(&self, user_id: &str, request: ChatRequest) -> Result<ChatResponse, String> {
        // Check if LLM client is configured
        let llm_client = self
            .llm_client
            .as_ref()
            .ok_or_else(|| "AI assistant not configured. Please set API key first.".to_string())?;

        // Check throttle limits (estimate 1000 tokens for request)
        let allowed = self
            .usage_throttle
            .check_and_record(user_id, 1000)
            .await
            .map_err(|e| format!("Throttle check failed: {}", e))?;

        if !allowed {
            return Err("Rate limit exceeded. Please try again later.".to_string());
        }

        // Get or create conversation
        let conversation_id = if let Some(id) = request.conversation_id.clone() {
            id
        } else {
            let context = if request.include_context {
                self.build_trading_context(user_id).await?
            } else {
                TradingContext {
                    portfolio: None,
                    active_alerts: Vec::new(),
                    market_data: HashMap::new(),
                    recent_trades: Vec::new(),
                }
            };

            self.conversation_manager
                .create_conversation(user_id, context)
                .await
                .map_err(|e| format!("Failed to create conversation: {}", e))?
        };

        // Add user message to conversation
        let user_message = Message {
            role: "user".to_string(),
            content: request.message.clone(),
            timestamp: Utc::now().to_rfc3339(),
        };

        self.conversation_manager
            .add_message(&conversation_id, user_message.clone())
            .await
            .map_err(|e| format!("Failed to save message: {}", e))?;

        // Get conversation history
        let messages = self
            .conversation_manager
            .get_messages(&conversation_id, 20)
            .await
            .map_err(|e| format!("Failed to get messages: {}", e))?;

        // Build system prompt with context
        let system_prompt = self
            .build_system_prompt(user_id, request.include_context)
            .await?;

        // Call LLM
        let response = llm_client
            .chat(messages, Some(system_prompt), self.functions.clone())
            .await?;

        // Save assistant response
        let assistant_message = Message {
            role: "assistant".to_string(),
            content: response.message.clone(),
            timestamp: response.timestamp.clone(),
        };

        self.conversation_manager
            .add_message(&conversation_id, assistant_message)
            .await
            .map_err(|e| format!("Failed to save response: {}", e))?;

        Ok(ChatResponse {
            conversation_id,
            ..response
        })
    }

    async fn build_trading_context(&self, _user_id: &str) -> Result<TradingContext, String> {
        // In a real implementation, this would fetch actual portfolio, alerts, etc.
        // For now, return mock context
        Ok(TradingContext {
            portfolio: Some(serde_json::json!({
                "total_value": 10000.0,
                "tokens": []
            })),
            active_alerts: vec![],
            market_data: HashMap::new(),
            recent_trades: vec![],
        })
    }

    async fn build_system_prompt(
        &self,
        user_id: &str,
        include_context: bool,
    ) -> Result<String, String> {
        let mut prompt = String::from(
            "You are an AI trading assistant for a cryptocurrency trading platform. \
            You help users analyze markets, manage their portfolio, execute trades, and create alerts. \
            Always prioritize risk management and provide clear explanations. \
            When executing trades or creating alerts, use the available function calls. \
            Be concise but informative.",
        );

        if include_context {
            let context = self.build_trading_context(user_id).await?;

            if let Some(portfolio) = context.portfolio {
                prompt.push_str(&format!("\n\nCurrent Portfolio: {}", portfolio));
            }

            if !context.active_alerts.is_empty() {
                prompt.push_str(&format!(
                    "\n\nActive Alerts: {}",
                    context.active_alerts.join(", ")
                ));
            }
        }

        Ok(prompt)
    }

    pub async fn get_conversations(&self, user_id: &str) -> Result<Vec<Conversation>, String> {
        self.conversation_manager
            .list_conversations(user_id, 50)
            .await
            .map_err(|e| format!("Failed to list conversations: {}", e))
    }

    pub async fn delete_conversation(&self, conversation_id: &str) -> Result<(), String> {
        self.conversation_manager
            .delete_conversation(conversation_id)
            .await
            .map_err(|e| format!("Failed to delete conversation: {}", e))
    }

    pub async fn get_usage_stats(&self, user_id: &str) -> Result<UsageStats, String> {
        self.usage_throttle
            .get_usage_stats(user_id)
            .await
            .map_err(|e| format!("Failed to get usage stats: {}", e))
    }
}

// ==================== Tauri Commands ====================

#[tauri::command]
pub async fn ai_chat(
    user_id: String,
    request: ChatRequest,
    ai_assistant: State<'_, SharedAIAssistant>,
) -> Result<ChatResponse, String> {
    let assistant = ai_assistant.read().await;
    assistant.chat(&user_id, request).await
}

#[tauri::command]
pub async fn ai_get_conversations(
    user_id: String,
    ai_assistant: State<'_, SharedAIAssistant>,
) -> Result<Vec<Conversation>, String> {
    let assistant = ai_assistant.read().await;
    assistant.get_conversations(&user_id).await
}

#[tauri::command]
pub async fn ai_delete_conversation(
    conversation_id: String,
    ai_assistant: State<'_, SharedAIAssistant>,
) -> Result<(), String> {
    let assistant = ai_assistant.read().await;
    assistant.delete_conversation(&conversation_id).await
}

#[tauri::command]
pub async fn ai_get_usage_stats(
    user_id: String,
    ai_assistant: State<'_, SharedAIAssistant>,
) -> Result<UsageStats, String> {
    let assistant = ai_assistant.read().await;
    assistant.get_usage_stats(&user_id).await
}

#[tauri::command]
pub async fn ai_set_api_key(
    provider: String,
    api_key: String,
    keystore: State<'_, Keystore>,
    ai_assistant: State<'_, SharedAIAssistant>,
) -> Result<(), String> {
    keystore
        .store_secret("llm_api_key", api_key.as_bytes())
        .map_err(|e| format!("Failed to store API key: {}", e))?;

    keystore
        .store_secret("llm_provider", provider.as_bytes())
        .map_err(|e| format!("Failed to store provider: {}", e))?;

    // Reload LLM client with new API key
    let mut assistant = ai_assistant.write().await;
    assistant.reload_llm_client(&keystore);

    Ok(())
}

#[tauri::command]
pub async fn ai_is_configured(ai_assistant: State<'_, SharedAIAssistant>) -> Result<bool, String> {
    let assistant = ai_assistant.read().await;
    Ok(assistant.is_configured())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_model_scoring() {
        let model = RiskModel::new();

        // High risk token features
        let high_risk = RiskFeatures {
            gini_coefficient: 0.95,
            top_10_percentage: 85.0,
            total_holders: 50,
            liquidity_usd: 5000.0,
            liquidity_to_mcap_ratio: 0.01,
            has_mint_authority: true,
            has_freeze_authority: true,
            verified: false,
            audited: false,
            community_trust_score: 0.2,
            sentiment_score: -0.5,
            token_age_days: 2.0,
            volume_24h: 1000.0,
            price_volatility: 50.0,
        };

        let (score, factors) = model.score_token(&high_risk);
        assert!(score > 60.0, "High risk token should score > 60");
        assert!(!factors.is_empty(), "Should have contributing factors");

        // Low risk token features
        let low_risk = RiskFeatures {
            gini_coefficient: 0.3,
            top_10_percentage: 25.0,
            total_holders: 10000,
            liquidity_usd: 1000000.0,
            liquidity_to_mcap_ratio: 0.2,
            has_mint_authority: false,
            has_freeze_authority: false,
            verified: true,
            audited: true,
            community_trust_score: 0.9,
            sentiment_score: 0.7,
            token_age_days: 180.0,
            volume_24h: 500000.0,
            price_volatility: 5.0,
        };

        let (score, _) = model.score_token(&low_risk);
        assert!(score < 40.0, "Low risk token should score < 40");
    }

    #[test]
    fn test_model_serialization() {
        let model = RiskModel::new();
        let json = model.to_json().expect("Should serialize");
        let loaded = RiskModel::from_json(&json).expect("Should deserialize");

        assert_eq!(model.intercept, loaded.intercept);
        assert_eq!(model.threshold, loaded.threshold);
    }

    #[test]
    fn test_feature_extraction() {
        let features = RiskFeatures {
            gini_coefficient: 0.5,
            top_10_percentage: 50.0,
            total_holders: 1000,
            liquidity_usd: 100000.0,
            liquidity_to_mcap_ratio: 0.1,
            has_mint_authority: false,
            has_freeze_authority: false,
            verified: true,
            audited: false,
            community_trust_score: 0.7,
            sentiment_score: 0.3,
            token_age_days: 30.0,
            volume_24h: 50000.0,
            price_volatility: 10.0,
        };

        let model = RiskModel::new();
        let (score, factors) = model.score_token(&features);

        assert!(
            score >= 0.0 && score <= 100.0,
            "Score should be in valid range"
        );
        assert!(factors.len() <= 5, "Should have at most 5 top factors");
    }

    #[test]
    fn test_function_registration() {
        let functions = AIAssistant::register_functions();

        assert!(!functions.is_empty(), "Should register functions");
        assert!(
            functions.iter().any(|f| f.name == "execute_trade"),
            "Should register execute_trade"
        );
        assert!(
            functions.iter().any(|f| f.name == "create_alert"),
            "Should register create_alert"
        );
        assert!(
            functions
                .iter()
                .any(|f| f.name == "get_portfolio_analytics"),
            "Should register get_portfolio_analytics"
        );
    }

    #[test]
    fn test_trading_context_serialization() {
        let context = TradingContext {
            portfolio: Some(serde_json::json!({"balance": 1000})),
            active_alerts: vec!["alert1".to_string(), "alert2".to_string()],
            market_data: HashMap::new(),
            recent_trades: vec![],
        };

        let json = serde_json::to_string(&context).expect("Should serialize");
        let deserialized: TradingContext = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(
            context.active_alerts.len(),
            deserialized.active_alerts.len()
        );
    }

    #[test]
    fn test_message_structure() {
        let message = Message {
            role: "user".to_string(),
            content: "What's the price of SOL?".to_string(),
            timestamp: Utc::now().to_rfc3339(),
        };

        let json = serde_json::to_string(&message).expect("Should serialize");
        let deserialized: Message = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(message.role, deserialized.role);
        assert_eq!(message.content, deserialized.content);
    }

    #[test]
    fn test_function_call_structure() {
        let function_call = FunctionCall {
            name: "execute_trade".to_string(),
            arguments: serde_json::json!({
                "action": "buy",
                "token_address": "So11111111111111111111111111111111111111112",
                "amount": 100.0
            }),
        };

        let json = serde_json::to_string(&function_call).expect("Should serialize");
        let deserialized: FunctionCall = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(function_call.name, deserialized.name);
    }

    #[tokio::test]
    async fn test_usage_throttle_limits() {
        use std::fs;
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_usage.db");
        fs::create_dir_all(temp_dir.path()).unwrap();

        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&db_url).await.unwrap();

        let throttle = UsageThrottle {
            pool,
            max_requests_per_hour: 2,
            max_tokens_per_day: 5000,
        };

        throttle.initialize().await.unwrap();

        // First request should succeed
        let result1 = throttle.check_and_record("user1", 1000).await.unwrap();
        assert!(result1, "First request should be allowed");

        // Second request should succeed
        let result2 = throttle.check_and_record("user1", 1000).await.unwrap();
        assert!(result2, "Second request should be allowed");

        // Third request should fail (exceeds hourly limit)
        let result3 = throttle.check_and_record("user1", 1000).await.unwrap();
        assert!(!result3, "Third request should be throttled");
    }

    #[test]
    fn test_llm_provider_parsing() {
        let provider = LLMProvider::Claude;
        let json = serde_json::to_string(&provider).unwrap();
        assert!(json.contains("claude"));

        let provider = LLMProvider::GPT4;
        let json = serde_json::to_string(&provider).unwrap();
        assert!(json.contains("gpt4"));
    }

    #[test]
    fn test_chat_request_validation() {
        let request = ChatRequest {
            conversation_id: Some("test-conv-id".to_string()),
            message: "Hello AI".to_string(),
            include_context: true,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: ChatRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.message, deserialized.message);
        assert_eq!(request.include_context, deserialized.include_context);
    }

    #[test]
    fn test_usage_stats_structure() {
        let stats = UsageStats {
            requests_count: 10,
            tokens_used: 5000,
            last_request_at: Utc::now().to_rfc3339(),
            window_start: (Utc::now() - chrono::Duration::hours(1)).to_rfc3339(),
        };

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: UsageStats = serde_json::from_str(&json).unwrap();

        assert_eq!(stats.requests_count, deserialized.requests_count);
        assert_eq!(stats.tokens_used, deserialized.tokens_used);
    }
}
