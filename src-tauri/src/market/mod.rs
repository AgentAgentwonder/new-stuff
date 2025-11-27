mod trending_coins;
pub use trending_coins::*;
pub mod drift_adapter;
pub mod holders;
pub mod new_coins_scanner_clean;
pub mod polymarket_adapter;
pub mod predictions;
pub mod top_coins;

pub use drift_adapter::*;
pub use holders::*;
// Exclude HolderInfo from new_coins_scanner_clean to avoid conflict with holders::HolderInfo
pub use new_coins_scanner_clean::{
    CreatorInfo, LiquidityInfo, NewCoin, NewCoinsScanner, NewCoinsScannerError, SafetyAnalysis,
    SafetyChecks, SafetyReport, SharedNewCoinsScanner, start_new_coins_scanner,
    get_new_coins, get_coin_safety_report, scan_for_new_coins,
};
pub use polymarket_adapter::*;
pub use predictions::*;
pub use top_coins::*;

use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoinPrice {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub price_change_24h: f64,
    pub volume_24h: f64,
    pub market_cap: f64,
    pub liquidity: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PricePoint {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenSearchResult {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub logo_uri: Option<String>,
}

// Birdeye API integration
async fn fetch_birdeye_price(token: &str, api_key: &str) -> Result<CoinPrice, String> {
    let client = reqwest::Client::new();
    let url = format!("https://public-api.birdeye.so/defi/price?address={}", token);

    let response = client
        .get(&url)
        .header("X-API-KEY", api_key)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    #[derive(Deserialize)]
    struct BirdeyeResponse {
        data: BirdeyeData,
    }

    #[derive(Deserialize)]
    struct BirdeyeData {
        value: f64,
        #[serde(rename = "priceChange24h")]
        price_change_24h: Option<f64>,
    }

    let data: BirdeyeResponse = response
        .json()
        .await
        .map_err(|e| format!("Parse failed: {}", e))?;

    Ok(CoinPrice {
        address: token.to_string(),
        symbol: "UNKNOWN".to_string(),
        name: "Unknown Token".to_string(),
        price: data.data.value,
        price_change_24h: data.data.price_change_24h.unwrap_or(0.0),
        volume_24h: 0.0,
        market_cap: 0.0,
        liquidity: None,
    })
}

// Mock data generator for development
fn generate_mock_price(symbol: &str) -> CoinPrice {
    let base_price = match symbol {
        "SOL" => 100.0,
        "BONK" => 0.000023,
        "JUP" => 1.23,
        _ => 1.0,
    };

    CoinPrice {
        address: "mock".to_string(),
        symbol: symbol.to_string(),
        name: format!("{} Token", symbol),
        price: base_price * (1.0 + rand::random_range(-0.05..0.05)),
        price_change_24h: rand::random_range(-20.0..20.0),
        volume_24h: rand::random_range(100000.0..10000000.0),
        market_cap: rand::random_range(1000000.0..100000000.0),
        liquidity: Some(rand::random_range(50000.0..5000000.0)),
    }
}

fn generate_mock_history(hours: i64) -> Vec<PricePoint> {
    let mut history = Vec::new();
    let mut price = 100.0;

    let now = chrono::Utc::now().timestamp();

    for i in (0..hours).rev() {
        let change = rand::random_range(-2.0..2.0);
        price += change;
        let volatility = rand::random_range(0.5..2.0);

        history.push(PricePoint {
            timestamp: now - (i * 3600),
            open: price,
            high: price + volatility,
            low: price - volatility,
            close: price + rand::random_range(-1.0..1.0),
            volume: rand::random_range(10000.0..100000.0),
        });
    }

    history
}

#[tauri::command]
pub async fn get_coin_price(address: String, api_key: Option<String>) -> Result<CoinPrice, String> {
    // If API key provided, use real API
    if let Some(key) = api_key {
        if !key.is_empty() {
            match fetch_birdeye_price(&address, &key).await {
                Ok(price) => return Ok(price),
                Err(_) => {} // Fall through to mock data
            }
        }
    }

    // Otherwise use mock data
    Ok(generate_mock_price(&address))
}

#[tauri::command]
pub async fn get_price_history(
    address: String,
    timeframe: String,
    _api_key: Option<String>,
) -> Result<Vec<PricePoint>, String> {
    let hours = match timeframe.as_str() {
        "1H" => 1,
        "4H" => 4,
        "1D" => 24,
        "1W" => 168,
        "1M" => 720,
        _ => 24,
    };

    // For now, return mock data
    Ok(generate_mock_history(hours))
}

#[tauri::command]
pub async fn search_tokens(query: String) -> Result<Vec<TokenSearchResult>, String> {
    // Mock search results
    let tokens = vec![
        TokenSearchResult {
            address: "So11111111111111111111111111111111111111112".to_string(),
            symbol: "SOL".to_string(),
            name: "Solana".to_string(),
            logo_uri: None,
        },
        TokenSearchResult {
            address: "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_string(),
            symbol: "BONK".to_string(),
            name: "Bonk".to_string(),
            logo_uri: None,
        },
        TokenSearchResult {
            address: "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN".to_string(),
            symbol: "JUP".to_string(),
            name: "Jupiter".to_string(),
            logo_uri: None,
        },
    ];

    let filtered: Vec<TokenSearchResult> = tokens
        .into_iter()
        .filter(|t| {
            t.symbol.to_lowercase().contains(&query.to_lowercase())
                || t.name.to_lowercase().contains(&query.to_lowercase())
        })
        .collect();

    Ok(filtered)
}
