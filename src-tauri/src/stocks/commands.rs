use super::api::StockApiClient;
use super::models::*;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

pub type SharedStockCache = Arc<RwLock<StockCache>>;

pub struct StockCache {
    trending_stocks: Option<(Vec<TrendingStock>, std::time::SystemTime)>,
    top_movers: std::collections::HashMap<String, (Vec<TopMover>, std::time::SystemTime)>,
    ipos: Option<(Vec<NewIPO>, std::time::SystemTime)>,
    earnings: Option<(Vec<EarningsEvent>, std::time::SystemTime)>,
    news_cache: std::collections::HashMap<String, (Vec<StockNews>, std::time::SystemTime)>,
}

impl Default for StockCache {
    fn default() -> Self {
        Self {
            trending_stocks: None,
            top_movers: std::collections::HashMap::new(),
            ipos: None,
            earnings: None,
            news_cache: std::collections::HashMap::new(),
        }
    }
}

impl StockCache {
    const CACHE_TTL: std::time::Duration = std::time::Duration::from_secs(60);

    fn is_expired(timestamp: std::time::SystemTime) -> bool {
        std::time::SystemTime::now()
            .duration_since(timestamp)
            .unwrap_or(std::time::Duration::from_secs(0))
            >= Self::CACHE_TTL
    }
}

#[tauri::command]
pub async fn get_trending_stocks(
    cache: State<'_, SharedStockCache>,
) -> Result<Vec<TrendingStock>, String> {
    // Check cache first
    let mut cache_guard = cache.write().await;
    if let Some((cached_data, timestamp)) = &cache_guard.trending_stocks {
        if !StockCache::is_expired(*timestamp) {
            return Ok(cached_data.clone());
        }
    }

    // Fetch fresh data (using mock data for now)
    let client = StockApiClient::new(None, None, None, None);
    let stocks = client.fetch_trending_stocks().await?;

    // Update cache
    cache_guard.trending_stocks = Some((stocks.clone(), std::time::SystemTime::now()));

    Ok(stocks)
}

#[tauri::command]
pub async fn get_top_movers(
    cache: State<'_, SharedStockCache>,
    session: String,
) -> Result<Vec<TopMover>, String> {
    let trading_session = match session.as_str() {
        "premarket" => TradingSession::PreMarket,
        "afterhours" => TradingSession::AfterHours,
        _ => TradingSession::Regular,
    };

    // Check cache first
    let mut cache_guard = cache.write().await;
    let cache_key = format!("{:?}", trading_session);
    if let Some((cached_data, timestamp)) = cache_guard.top_movers.get(&cache_key) {
        if !StockCache::is_expired(*timestamp) {
            return Ok(cached_data.clone());
        }
    }

    // Fetch fresh data
    let client = StockApiClient::new(None, None, None, None);
    let movers = client.fetch_top_movers(trading_session).await?;

    // Update cache
    cache_guard
        .top_movers
        .insert(cache_key, (movers.clone(), std::time::SystemTime::now()));

    Ok(movers)
}

#[tauri::command]
pub async fn get_new_ipos(cache: State<'_, SharedStockCache>) -> Result<Vec<NewIPO>, String> {
    // Check cache first
    let mut cache_guard = cache.write().await;
    if let Some((cached_data, timestamp)) = &cache_guard.ipos {
        if !StockCache::is_expired(*timestamp) {
            return Ok(cached_data.clone());
        }
    }

    // Fetch fresh data
    let client = StockApiClient::new(None, None, None, None);
    let ipos = client.fetch_new_ipos().await?;

    // Update cache
    cache_guard.ipos = Some((ipos.clone(), std::time::SystemTime::now()));

    Ok(ipos)
}

#[tauri::command]
pub async fn get_earnings_calendar(
    cache: State<'_, SharedStockCache>,
    days_ahead: Option<u32>,
) -> Result<Vec<EarningsEvent>, String> {
    let days = days_ahead.unwrap_or(30);

    // Check cache first
    let mut cache_guard = cache.write().await;
    if let Some((cached_data, timestamp)) = &cache_guard.earnings {
        if !StockCache::is_expired(*timestamp) {
            return Ok(cached_data.clone());
        }
    }

    // Fetch fresh data
    let client = StockApiClient::new(None, None, None, None);
    let earnings = client.fetch_earnings_calendar(days).await?;

    // Update cache
    cache_guard.earnings = Some((earnings.clone(), std::time::SystemTime::now()));

    Ok(earnings)
}

#[tauri::command]
pub async fn get_stock_news(
    cache: State<'_, SharedStockCache>,
    symbol: String,
    limit: Option<usize>,
) -> Result<Vec<StockNews>, String> {
    let news_limit = limit.unwrap_or(20);

    // Check cache first
    let mut cache_guard = cache.write().await;
    if let Some((cached_data, timestamp)) = cache_guard.news_cache.get(&symbol) {
        if !StockCache::is_expired(*timestamp) {
            return Ok(cached_data.clone());
        }
    }

    // Fetch fresh data
    let client = StockApiClient::new(None, None, None, None);
    let news = client.fetch_stock_news(&symbol, news_limit).await?;

    // Update cache
    cache_guard
        .news_cache
        .insert(symbol, (news.clone(), std::time::SystemTime::now()));

    Ok(news)
}

#[tauri::command]
pub async fn get_institutional_holdings(
    symbol: String,
) -> Result<Vec<InstitutionalHolding>, String> {
    let client = StockApiClient::new(None, None, None, None);
    client.fetch_institutional_holdings(&symbol).await
}

#[tauri::command]
pub async fn get_insider_activity(symbol: String) -> Result<Vec<InsiderActivity>, String> {
    let client = StockApiClient::new(None, None, None, None);
    client.fetch_insider_activity(&symbol).await
}

#[tauri::command]
pub async fn create_stock_alert(
    symbol: String,
    alert_type: String,
    threshold: Option<f64>,
) -> Result<String, String> {
    // This would integrate with the alerts system
    // For now, just return success
    Ok(format!("Alert created for {} ({})", symbol, alert_type))
}

#[tauri::command]
pub async fn get_stock_alerts(symbol: Option<String>) -> Result<Vec<StockAlert>, String> {
    // This would fetch from alerts database
    // For now, return empty
    Ok(vec![])
}
