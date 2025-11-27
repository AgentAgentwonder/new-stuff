use base64::{engine::general_purpose, Engine as _};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Client, StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, instrument, warn};

const JUPITER_BASE_URL: &str = "https://quote-api.jup.ag/v6";

#[derive(Debug, Error)]
pub enum JupiterError {
    #[error("network error: {0}")]
    Network(String),
    #[error("http error: status={status} body={body}")]
    Http { status: StatusCode, body: String },
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("invalid response: {0}")]
    InvalidResponse(String),
    #[error("missing quote when executing swap")]
    MissingQuote,
}

impl From<JupiterError> for String {
    fn from(value: JupiterError) -> Self {
        value.to_string()
    }
}

#[derive(Debug, Clone)]
pub struct JupiterClient {
    http: Client,
    base_url: String,
    api_key: Option<String>,
}

impl Default for JupiterClient {
    fn default() -> Self {
        Self::new(None)
    }
}

impl JupiterClient {
    pub fn new(api_key: Option<String>) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(20))
            .build()
            .expect("failed to construct http client");

        Self {
            http,
            base_url: JUPITER_BASE_URL.to_string(),
            api_key,
        }
    }

    #[cfg(test)]
    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        let http = Client::new();
        Self {
            http,
            base_url: base_url.into(),
            api_key: None,
        }
    }

    fn headers(&self) -> Result<HeaderMap, JupiterError> {
        let mut map = HeaderMap::new();
        if let Some(api_key) = &self.api_key {
            let value = HeaderValue::from_str(&format!("Bearer {api_key}"))
                .map_err(|e| JupiterError::Serialization(e.to_string()))?;
            map.insert(AUTHORIZATION, value);
        }
        Ok(map)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuoteCommandInput {
    pub input_mint: String,
    pub output_mint: String,
    pub amount: u64,
    #[serde(default)]
    pub slippage_bps: Option<u16>,
    #[serde(default)]
    pub swap_mode: Option<SwapMode>,
    #[serde(default)]
    pub platform_fee_bps: Option<u16>,
    #[serde(default)]
    pub only_direct_routes: Option<bool>,
    #[serde(default)]
    pub referral_account: Option<String>,
    #[serde(default)]
    pub as_legacy_transaction: Option<bool>,
    #[serde(default)]
    pub priority_fee_config: Option<PriorityFeeConfig>,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SwapMode {
    #[serde(rename = "exact_in")]
    ExactIn,
    #[serde(rename = "exact_out")]
    ExactOut,
}

impl Default for SwapMode {
    fn default() -> Self {
        SwapMode::ExactIn
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct PriorityFeeConfig {
    #[serde(default)]
    pub compute_unit_price_micro_lamports: Option<u64>,
    #[serde(default)]
    pub auto_multiplier: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    pub input_mint: String,
    pub output_mint: String,
    pub input_amount: String,
    pub output_amount: String,
    pub other_amount_threshold: String,
    pub swap_mode: SwapMode,
    #[serde(default)]
    pub slippage_bps: Option<u16>,
    pub price_impact_pct: f64,
    pub context_slot: u64,
    #[serde(default)]
    pub time_taken: f64,
    #[serde(default)]
    pub route_plan: Vec<RoutePlanStep>,
    #[serde(default)]
    pub prioritization_fee_lamports: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RoutePlanStep {
    pub swap_info: SwapInfo,
    pub percent: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SwapInfo {
    pub amm_key: String,
    #[serde(default)]
    pub label: Option<String>,
    pub input_mint: String,
    pub output_mint: String,
    pub in_amount: String,
    pub out_amount: String,
    #[serde(default)]
    pub fee_bps: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParsedRoutePlan {
    pub price_impact_pct: f64,
    pub total_fee_bps: u64,
    pub hops: Vec<ParsedRouteHop>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParsedRouteHop {
    pub dex: Option<String>,
    pub percent: f64,
    pub input_mint: String,
    pub output_mint: String,
    pub in_amount: f64,
    pub out_amount: f64,
    pub fee_bps: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QuoteResult {
    pub quote: QuoteResponse,
    pub route: ParsedRoutePlan,
    pub context_slot: u64,
    #[serde(default)]
    pub prioritization_fee_lamports: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SwapCommandInput {
    pub quote: QuoteResponse,
    pub user_public_key: String,
    #[serde(default)]
    pub fee_account: Option<String>,
    #[serde(default)]
    pub wrap_and_unwrap_sol: Option<bool>,
    #[serde(default)]
    pub as_legacy_transaction: Option<bool>,
    #[serde(default)]
    pub priority_fee_config: Option<PriorityFeeConfig>,
    #[serde(default)]
    pub simulate: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EncodedTransaction {
    pub base64: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SwapSimulationResult {
    pub logs: Vec<String>,
    #[serde(default)]
    pub compute_units_consumed: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SwapResult {
    pub transaction: EncodedTransaction,
    pub last_valid_block_height: u64,
    #[serde(default)]
    pub prioritization_fee_lamports: Option<String>,
    #[serde(default)]
    pub simulation: Option<SwapSimulationResult>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SwapResponse {
    #[serde(default)]
    pub swap_transaction: Option<String>,
    #[serde(default)]
    pub last_valid_block_height: Option<u64>,
    #[serde(default)]
    pub prioritization_fee_lamports: Option<String>,
    #[serde(default)]
    pub simulation_logs: Option<Vec<String>>,
    #[serde(default)]
    pub compute_units_consumed: Option<u64>,
    #[serde(default)]
    pub error: Option<Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct QuoteQueryParams<'a> {
    #[serde(rename = "inputMint")]
    input_mint: &'a str,
    #[serde(rename = "outputMint")]
    output_mint: &'a str,
    amount: &'a str,
    swap_mode: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    slippage_bps: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    platform_fee_bps: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    only_direct_routes: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    as_legacy_transaction: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    compute_unit_price_micro_lamports: Option<u64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SwapRequestBody<'a> {
    quote_response: &'a QuoteResponse,
    user_public_key: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    wrap_and_unwrap_sol: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fee_account: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    as_legacy_transaction: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    compute_unit_price_micro_lamports: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    auto_compute_unit_price_multiplier: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    referer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    simulate: Option<bool>,
}

#[tauri::command]
#[instrument(skip(input), fields(input_mint = %input.input_mint, output_mint = %input.output_mint, amount = input.amount))]
pub async fn jupiter_quote(input: QuoteCommandInput) -> Result<QuoteResult, String> {
    let client = JupiterClient::default();
    let response = client.quote(&input).await.map_err(String::from)?;
    let route = parse_route_plan(&response);
    Ok(QuoteResult {
        context_slot: response.context_slot,
        prioritization_fee_lamports: response.prioritization_fee_lamports.clone(),
        route,
        quote: response,
    })
}

#[tauri::command]
#[instrument(skip(input), fields(user = %input.user_public_key))]
pub async fn jupiter_swap(input: SwapCommandInput) -> Result<SwapResult, String> {
    if input.quote.route_plan.is_empty() {
        return Err(JupiterError::MissingQuote.into());
    }

    let client = JupiterClient::default();
    let response = client
        .execute_swap(&input, input.simulate.unwrap_or(false))
        .await?;
    let swap_transaction = response
        .swap_transaction
        .ok_or_else(|| JupiterError::InvalidResponse("missing transaction".into()))?;

    let transaction = decode_versioned_transaction(&swap_transaction)?;
    let simulation = response.simulation_logs.map(|logs| SwapSimulationResult {
        logs,
        compute_units_consumed: response.compute_units_consumed,
    });

    Ok(SwapResult {
        transaction,
        last_valid_block_height: response
            .last_valid_block_height
            .ok_or_else(|| JupiterError::InvalidResponse("missing lastValidBlockHeight".into()))?,
        prioritization_fee_lamports: response.prioritization_fee_lamports,
        simulation,
    })
}

fn parse_route_plan(quote: &QuoteResponse) -> ParsedRoutePlan {
    let hops: Vec<ParsedRouteHop> = quote
        .route_plan
        .iter()
        .map(|step| ParsedRouteHop {
            dex: step.swap_info.label.clone(),
            percent: step.percent,
            input_mint: step.swap_info.input_mint.clone(),
            output_mint: step.swap_info.output_mint.clone(),
            in_amount: step.swap_info.in_amount.parse::<f64>().unwrap_or_default(),
            out_amount: step.swap_info.out_amount.parse::<f64>().unwrap_or_default(),
            fee_bps: step.swap_info.fee_bps,
        })
        .collect();

    let total_fee_bps = hops.iter().filter_map(|hop| hop.fee_bps).sum();

    ParsedRoutePlan {
        hops,
        total_fee_bps,
        price_impact_pct: quote.price_impact_pct,
    }
}

fn decode_versioned_transaction(encoded: &str) -> Result<EncodedTransaction, String> {
    use solana_sdk::{signature::Signature, transaction::VersionedTransaction};

    let bytes = general_purpose::STANDARD
        .decode(encoded.as_bytes())
        .map_err(|e| JupiterError::Serialization(format!("invalid base64 transaction: {e}")))?;

    let tx: VersionedTransaction = bincode::deserialize(&bytes)
        .map_err(|e| JupiterError::Serialization(format!("invalid transaction bytes: {e}")))?;

    // Ensure signatures are valid length to avoid runtime issues
    for signature in tx.signatures.iter() {
        Signature::try_from(signature.as_ref())
            .map_err(|e| JupiterError::Serialization(format!("invalid signature: {e}")))?;
    }

    let version = tx.version();
    let version_str = match version {
        solana_sdk::transaction::TransactionVersion::Legacy(_) => "legacy",
        solana_sdk::transaction::TransactionVersion::Number(n) => {
            if n == 0 {
                "v0"
            } else {
                "unknown"
            }
        }
    };

    Ok(EncodedTransaction {
        base64: general_purpose::STANDARD.encode(bytes),
        version: version_str.to_string(),
    })
}

impl JupiterClient {
    async fn quote(&self, input: &QuoteCommandInput) -> Result<QuoteResponse, JupiterError> {
        let amount = input.amount.to_string();
        let swap_mode = input.swap_mode.unwrap_or_default();
        let params = QuoteQueryParams {
            input_mint: &input.input_mint,
            output_mint: &input.output_mint,
            amount: &amount,
            swap_mode: match swap_mode {
                SwapMode::ExactIn => "ExactIn",
                SwapMode::ExactOut => "ExactOut",
            },
            slippage_bps: input.slippage_bps,
            platform_fee_bps: input.platform_fee_bps,
            only_direct_routes: input.only_direct_routes,
            as_legacy_transaction: input.as_legacy_transaction,
            compute_unit_price_micro_lamports: input
                .priority_fee_config
                .as_ref()
                .and_then(|cfg| cfg.compute_unit_price_micro_lamports),
        };

        let query = serde_urlencoded::to_string(params)
            .map_err(|e| JupiterError::Serialization(e.to_string()))?;
        let url = format!("{}/quote?{}", self.base_url, query);

        let response = self
            .http
            .get(&url)
            .headers(self.headers()?)
            .send()
            .await
            .map_err(|e| JupiterError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "<unavailable>".into());
            return Err(JupiterError::Http { status, body });
        }

        response
            .json::<QuoteResponse>()
            .await
            .map_err(|e| JupiterError::Serialization(e.to_string()))
    }

    async fn execute_swap(
        &self,
        input: &SwapCommandInput,
        simulate: bool,
    ) -> Result<SwapResponse, String> {
        let body = SwapRequestBody {
            quote_response: &input.quote,
            user_public_key: &input.user_public_key,
            wrap_and_unwrap_sol: input.wrap_and_unwrap_sol,
            fee_account: input.fee_account.as_deref(),
            as_legacy_transaction: input.as_legacy_transaction,
            compute_unit_price_micro_lamports: input
                .priority_fee_config
                .as_ref()
                .and_then(|cfg| cfg.compute_unit_price_micro_lamports),
            auto_compute_unit_price_multiplier: input
                .priority_fee_config
                .as_ref()
                .and_then(|cfg| cfg.auto_multiplier),
            referer: input.fee_account.clone(),
            simulate: Some(simulate),
        };

        let response = self
            .http
            .post(format!("{}/swap", self.base_url))
            .headers(self.headers().map_err(String::from)?)
            .json(&body)
            .send()
            .await
            .map_err(|e| JupiterError::Network(e.to_string()).to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "<unavailable>".into());
            return Err(JupiterError::Http { status, body }.to_string());
        }

        response
            .json::<SwapResponse>()
            .await
            .map_err(|e| JupiterError::Serialization(e.to_string()).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;

    fn mock_quote_response() -> QuoteResponse {
        QuoteResponse {
            input_mint: "So11111111111111111111111111111111111111112".into(),
            output_mint: "Es9vMFrzaCERcFbB9HsCLewZE97TR7UrT9uJ1Xu1F1v".into(),
            input_amount: "1000000".into(),
            output_amount: "995000".into(),
            other_amount_threshold: "990000".into(),
            swap_mode: SwapMode::ExactIn,
            slippage_bps: Some(50),
            price_impact_pct: 0.23,
            context_slot: 123,
            time_taken: 0.25,
            route_plan: vec![RoutePlanStep {
                swap_info: SwapInfo {
                    amm_key: "amm".into(),
                    label: Some("Raydium".into()),
                    input_mint: "So11111111111111111111111111111111111111112".into(),
                    output_mint: "Es9vMFrzaCERcFbB9HsCLewZE97TR7UrT9uJ1Xu1F1v".into(),
                    in_amount: "1000000".into(),
                    out_amount: "995000".into(),
                    fee_bps: Some(10),
                },
                percent: 100.0,
            }],
            prioritization_fee_lamports: Some("2000".into()),
        }
    }

    #[tokio::test]
    async fn quote_successfully_parses_route() {
        let server = MockServer::start();

        let quote_response = mock_quote_response();

        let _mock = server.mock(|when, then| {
            when.method(GET)
                .path("/quote")
                .query_param("inputMint", quote_response.input_mint.clone())
                .query_param("outputMint", quote_response.output_mint.clone())
                .query_param("amount", quote_response.input_amount.clone())
                .query_param("swapMode", "ExactIn");
            then.status(200)
                .json_body(serde_json::to_value(&quote_response).unwrap());
        });

        let client = JupiterClient::with_base_url(server.base_url());

        let result = client
            .quote(&QuoteCommandInput {
                input_mint: quote_response.input_mint.clone(),
                output_mint: quote_response.output_mint.clone(),
                amount: quote_response.input_amount.parse().unwrap(),
                slippage_bps: quote_response.slippage_bps,
                swap_mode: Some(SwapMode::ExactIn),
                platform_fee_bps: None,
                only_direct_routes: None,
                referral_account: None,
                as_legacy_transaction: None,
                priority_fee_config: None,
            })
            .await
            .expect("quote should succeed");

        assert_eq!(result.price_impact_pct, 0.23);
        assert_eq!(result.route_plan.len(), 1);
        assert_eq!(
            result.route_plan[0].swap_info.label.as_deref(),
            Some("Raydium")
        );
    }

    #[tokio::test]
    async fn swap_simulation_handles_logs() {
        let server = MockServer::start();
        let quote = mock_quote_response();

        let _mock = server.mock(|when, then| {
            when.method(POST).path("/swap");
            then.status(200).json_body(serde_json::json!({
                "swapTransaction": general_purpose::STANDARD.encode(bincode::serialize(&dummy_versioned_tx()).unwrap()),
                "lastValidBlockHeight": 1,
                "simulationLogs": ["log1", "log2"],
                "computeUnitsConsumed": 50000
            }));
        });

        let client = JupiterClient::with_base_url(server.base_url());

        let result = client
            .execute_swap(
                &SwapCommandInput {
                    quote,
                    user_public_key: "user".into(),
                    fee_account: None,
                    wrap_and_unwrap_sol: Some(true),
                    as_legacy_transaction: Some(false),
                    priority_fee_config: None,
                    simulate: Some(true),
                },
                true,
            )
            .await
            .expect("swap should succeed");

        assert_eq!(result.simulation_logs.unwrap().len(), 2);
        assert_eq!(result.compute_units_consumed, Some(50000));
    }

    fn dummy_versioned_tx() -> solana_sdk::transaction::VersionedTransaction {
        use solana_sdk::{
            instruction::CompiledInstruction,
            message::{v0::Message, VersionedMessage},
            pubkey::Pubkey,
            signature::Signature,
            transaction::VersionedTransaction,
        };

        let message = Message::new(
            vec![CompiledInstruction {
                program_id_index: 0,
                accounts: vec![],
                data: vec![],
            }],
            Some(&Pubkey::new_unique()),
        );

        VersionedTransaction {
            signatures: vec![Signature::default()],
            message: VersionedMessage::V0(message),
        }
    }
}
