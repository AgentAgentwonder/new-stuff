use super::models::*;
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const ALPHA_VANTAGE_BASE_URL: &str = "https://www.alphavantage.co/query";
const POLYGON_BASE_URL: &str = "https://api.polygon.io";
const IEX_BASE_URL: &str = "https://cloud.iexapis.com/stable";
const FINNHUB_BASE_URL: &str = "https://finnhub.io/api/v1";

pub struct StockApiClient {
    alpha_vantage_key: Option<String>,
    polygon_key: Option<String>,
    iex_key: Option<String>,
    finnhub_key: Option<String>,
    client: reqwest::Client,
}

impl StockApiClient {
    pub fn new(
        alpha_vantage_key: Option<String>,
        polygon_key: Option<String>,
        iex_key: Option<String>,
        finnhub_key: Option<String>,
    ) -> Self {
        Self {
            alpha_vantage_key,
            polygon_key,
            iex_key,
            finnhub_key,
            client: reqwest::Client::new(),
        }
    }

    pub async fn fetch_trending_stocks(&self) -> Result<Vec<TrendingStock>, String> {
        // Use Finnhub for most actives (trending)
        if let Some(api_key) = &self.finnhub_key {
            return self.fetch_finnhub_actives(api_key).await;
        }

        // Fallback to Alpha Vantage
        if let Some(api_key) = &self.alpha_vantage_key {
            return self.fetch_alpha_vantage_actives(api_key).await;
        }

        // Return mock data if no API key
        Ok(self.generate_mock_trending_stocks())
    }

    async fn fetch_finnhub_actives(&self, api_key: &str) -> Result<Vec<TrendingStock>, String> {
        let url = format!("{}/stock/market-movers?token={}", FINNHUB_BASE_URL, api_key);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Finnhub request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Finnhub API error: {}", response.status()));
        }

        #[derive(Deserialize)]
        struct FinnhubMoverItem {
            symbol: String,
            change: f64,
            #[serde(rename = "percentChange")]
            percent_change: f64,
            price: f64,
            volume: f64,
        }

        #[derive(Deserialize)]
        struct FinnhubResponse {
            data: Vec<FinnhubMoverItem>,
        }

        let data: FinnhubResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Finnhub response: {}", e))?;

        let stocks = data
            .data
            .into_iter()
            .map(|item| {
                let avg_volume = item.volume * 0.8; // Estimate
                let volume_change = ((item.volume - avg_volume) / avg_volume) * 100.0;
                let unusual_volume = volume_change > 50.0;

                TrendingStock {
                    symbol: item.symbol.clone(),
                    name: format!("{} Inc.", item.symbol), // Would need additional API call for full name
                    price: item.price,
                    change_24h: item.change,
                    percent_change_24h: item.percent_change,
                    volume: item.volume,
                    volume_change_24h: volume_change,
                    unusual_volume,
                    market_cap: None,
                    avg_volume,
                    reason: if unusual_volume {
                        Some("Unusual volume activity".to_string())
                    } else {
                        None
                    },
                }
            })
            .collect();

        Ok(stocks)
    }

    async fn fetch_alpha_vantage_actives(
        &self,
        api_key: &str,
    ) -> Result<Vec<TrendingStock>, String> {
        // Alpha Vantage doesn't have a direct "actives" endpoint
        // Would need to fetch multiple symbols or use premium endpoint
        // Return mock for now
        Ok(self.generate_mock_trending_stocks())
    }

    pub async fn fetch_top_movers(&self, session: TradingSession) -> Result<Vec<TopMover>, String> {
        if let Some(api_key) = &self.finnhub_key {
            return self.fetch_finnhub_movers(api_key, session).await;
        }

        Ok(self.generate_mock_top_movers(session))
    }

    async fn fetch_finnhub_movers(
        &self,
        api_key: &str,
        session: TradingSession,
    ) -> Result<Vec<TopMover>, String> {
        let url = format!("{}/stock/market-movers?token={}", FINNHUB_BASE_URL, api_key);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Finnhub request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Finnhub API error: {}", response.status()));
        }

        // Parse and transform to TopMover
        // For now return mock
        Ok(self.generate_mock_top_movers(session))
    }

    pub async fn fetch_new_ipos(&self) -> Result<Vec<NewIPO>, String> {
        if let Some(api_key) = &self.iex_key {
            return self.fetch_iex_ipos(api_key).await;
        }

        Ok(self.generate_mock_ipos())
    }

    async fn fetch_iex_ipos(&self, api_key: &str) -> Result<Vec<NewIPO>, String> {
        let url = format!(
            "{}/stock/market/upcoming-ipos?token={}",
            IEX_BASE_URL, api_key
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("IEX request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("IEX API error: {}", response.status()));
        }

        // Parse and transform
        Ok(self.generate_mock_ipos())
    }

    pub async fn fetch_earnings_calendar(
        &self,
        days_ahead: u32,
    ) -> Result<Vec<EarningsEvent>, String> {
        if let Some(api_key) = &self.alpha_vantage_key {
            return self.fetch_alpha_vantage_earnings(api_key).await;
        }

        Ok(self.generate_mock_earnings_calendar(days_ahead))
    }

    async fn fetch_alpha_vantage_earnings(
        &self,
        api_key: &str,
    ) -> Result<Vec<EarningsEvent>, String> {
        let url = format!(
            "{}?function=EARNINGS_CALENDAR&horizon=3month&apikey={}",
            ALPHA_VANTAGE_BASE_URL, api_key
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Alpha Vantage request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Alpha Vantage API error: {}", response.status()));
        }

        // Parse CSV response
        // For now return mock
        Ok(self.generate_mock_earnings_calendar(30))
    }

    pub async fn fetch_stock_news(
        &self,
        symbol: &str,
        limit: usize,
    ) -> Result<Vec<StockNews>, String> {
        if let Some(api_key) = &self.finnhub_key {
            return self.fetch_finnhub_news(api_key, symbol, limit).await;
        }

        Ok(self.generate_mock_stock_news(symbol, limit))
    }

    async fn fetch_finnhub_news(
        &self,
        api_key: &str,
        symbol: &str,
        limit: usize,
    ) -> Result<Vec<StockNews>, String> {
        let url = format!(
            "{}/company-news?symbol={}&from={}&to={}&token={}",
            FINNHUB_BASE_URL,
            symbol,
            chrono::Utc::now().format("%Y-%m-%d"),
            chrono::Utc::now().format("%Y-%m-%d"),
            api_key
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Finnhub request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Finnhub API error: {}", response.status()));
        }

        // Parse and transform
        Ok(self.generate_mock_stock_news(symbol, limit))
    }

    pub async fn fetch_institutional_holdings(
        &self,
        symbol: &str,
    ) -> Result<Vec<InstitutionalHolding>, String> {
        if let Some(api_key) = &self.finnhub_key {
            return self.fetch_finnhub_institutional(api_key, symbol).await;
        }

        Ok(self.generate_mock_institutional_holdings(symbol))
    }

    async fn fetch_finnhub_institutional(
        &self,
        api_key: &str,
        symbol: &str,
    ) -> Result<Vec<InstitutionalHolding>, String> {
        let url = format!(
            "{}/stock/institutional?symbol={}&token={}",
            FINNHUB_BASE_URL, symbol, api_key
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Finnhub request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Finnhub API error: {}", response.status()));
        }

        // Parse and transform
        Ok(self.generate_mock_institutional_holdings(symbol))
    }

    pub async fn fetch_insider_activity(
        &self,
        symbol: &str,
    ) -> Result<Vec<InsiderActivity>, String> {
        if let Some(api_key) = &self.finnhub_key {
            return self.fetch_finnhub_insider(api_key, symbol).await;
        }

        Ok(self.generate_mock_insider_activity(symbol))
    }

    async fn fetch_finnhub_insider(
        &self,
        api_key: &str,
        symbol: &str,
    ) -> Result<Vec<InsiderActivity>, String> {
        let url = format!(
            "{}/stock/insider-transactions?symbol={}&token={}",
            FINNHUB_BASE_URL, symbol, api_key
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Finnhub request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Finnhub API error: {}", response.status()));
        }

        // Parse and transform
        Ok(self.generate_mock_insider_activity(symbol))
    }

    // Mock data generators for fallback
    fn generate_mock_trending_stocks(&self) -> Vec<TrendingStock> {
        vec![
            TrendingStock {
                symbol: "AAPL".to_string(),
                name: "Apple Inc.".to_string(),
                price: 178.25,
                change_24h: 2.15,
                percent_change_24h: 1.22,
                volume: 52_450_000.0,
                volume_change_24h: 35.6,
                unusual_volume: false,
                market_cap: Some(2_800_000_000_000.0),
                avg_volume: 48_000_000.0,
                reason: None,
            },
            TrendingStock {
                symbol: "TSLA".to_string(),
                name: "Tesla Inc.".to_string(),
                price: 242.50,
                change_24h: -5.30,
                percent_change_24h: -2.14,
                volume: 125_600_000.0,
                volume_change_24h: 78.3,
                unusual_volume: true,
                market_cap: Some(770_000_000_000.0),
                avg_volume: 70_500_000.0,
                reason: Some("Earnings report".to_string()),
            },
            TrendingStock {
                symbol: "NVDA".to_string(),
                name: "NVIDIA Corporation".to_string(),
                price: 495.30,
                change_24h: 12.85,
                percent_change_24h: 2.66,
                volume: 45_300_000.0,
                volume_change_24h: 42.1,
                unusual_volume: false,
                market_cap: Some(1_220_000_000_000.0),
                avg_volume: 32_000_000.0,
                reason: None,
            },
        ]
    }

    fn generate_mock_top_movers(&self, session: TradingSession) -> Vec<TopMover> {
        vec![
            TopMover {
                symbol: "NVDA".to_string(),
                name: "NVIDIA Corporation".to_string(),
                price: 495.30,
                change: 12.85,
                percent_change: 2.66,
                volume: 45_300_000.0,
                market_cap: Some(1_220_000_000_000.0),
                direction: MoverDirection::Gainer,
                session: session.clone(),
                technical_indicators: TechnicalIndicators {
                    rsi: Some(68.5),
                    macd: Some("Bullish".to_string()),
                    volume_ratio: 1.42,
                    momentum: Some("Strong".to_string()),
                },
                reason: "AI chip demand surge".to_string(),
            },
            TopMover {
                symbol: "META".to_string(),
                name: "Meta Platforms Inc.".to_string(),
                price: 452.80,
                change: 8.20,
                percent_change: 1.84,
                volume: 18_600_000.0,
                market_cap: Some(1_180_000_000_000.0),
                direction: MoverDirection::Gainer,
                session: session.clone(),
                technical_indicators: TechnicalIndicators {
                    rsi: Some(65.2),
                    macd: Some("Bullish".to_string()),
                    volume_ratio: 1.23,
                    momentum: Some("Moderate".to_string()),
                },
                reason: "Strong user growth".to_string(),
            },
        ]
    }

    fn generate_mock_ipos(&self) -> Vec<NewIPO> {
        vec![
            NewIPO {
                symbol: "ARMH".to_string(),
                name: "ARM Holdings".to_string(),
                ipo_date: "2024-09-15".to_string(),
                offer_price: 51.0,
                current_price: Some(68.50),
                percent_change: Some(34.31),
                shares_offered: Some(95_500_000.0),
                market_cap: Some(70_000_000_000.0),
                exchange: "NASDAQ".to_string(),
                status: IPOStatus::Recent,
            },
            NewIPO {
                symbol: "RDDT".to_string(),
                name: "Reddit Inc.".to_string(),
                ipo_date: "2024-12-01".to_string(),
                offer_price: 34.0,
                current_price: None,
                percent_change: None,
                shares_offered: Some(22_000_000.0),
                market_cap: None,
                exchange: "NYSE".to_string(),
                status: IPOStatus::Upcoming,
            },
        ]
    }

    fn generate_mock_earnings_calendar(&self, days_ahead: u32) -> Vec<EarningsEvent> {
        vec![
            EarningsEvent {
                symbol: "AAPL".to_string(),
                name: "Apple Inc.".to_string(),
                date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                time: EarningsTime::AfterMarket,
                fiscal_quarter: "Q4 2024".to_string(),
                estimate_eps: Some(1.39),
                actual_eps: None,
                surprise_percent: None,
                historical_reaction: Some(HistoricalReaction {
                    avg_move_percent: 3.2,
                    last_reaction_percent: 4.1,
                    beat_miss_ratio: "8/2".to_string(),
                }),
                has_alert: true,
            },
            EarningsEvent {
                symbol: "MSFT".to_string(),
                name: "Microsoft Corporation".to_string(),
                date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                time: EarningsTime::AfterMarket,
                fiscal_quarter: "Q2 2024".to_string(),
                estimate_eps: Some(2.65),
                actual_eps: None,
                surprise_percent: None,
                historical_reaction: Some(HistoricalReaction {
                    avg_move_percent: 2.8,
                    last_reaction_percent: -1.5,
                    beat_miss_ratio: "7/3".to_string(),
                }),
                has_alert: false,
            },
        ]
    }

    fn generate_mock_stock_news(&self, symbol: &str, limit: usize) -> Vec<StockNews> {
        vec![
            StockNews {
                id: "1".to_string(),
                symbol: symbol.to_string(),
                title: format!("{} Reports Strong Quarterly Results", symbol),
                summary: "Company exceeds analyst expectations with strong revenue growth.".to_string(),
                ai_summary: Some("The company reported Q4 revenue of $123B, beating estimates by 8%. EPS came in at $2.10 vs expected $1.95.".to_string()),
                url: "https://example.com/news/1".to_string(),
                source: "Reuters".to_string(),
                published_at: chrono::Utc::now().to_rfc3339(),
                sentiment: Sentiment::Bullish,
                impact_level: ImpactLevel::High,
                topics: vec!["earnings".to_string(), "revenue".to_string()],
            },
            StockNews {
                id: "2".to_string(),
                symbol: symbol.to_string(),
                title: format!("{} Announces New Product Launch", symbol),
                summary: "Company unveils innovative new product line.".to_string(),
                ai_summary: Some("The company announced a new AI-powered product targeting enterprise customers, expected to launch in Q2.".to_string()),
                url: "https://example.com/news/2".to_string(),
                source: "Bloomberg".to_string(),
                published_at: chrono::Utc::now().to_rfc3339(),
                sentiment: Sentiment::Bullish,
                impact_level: ImpactLevel::Medium,
                topics: vec!["product".to_string(), "innovation".to_string()],
            },
        ]
        .into_iter()
        .take(limit)
        .collect()
    }

    fn generate_mock_institutional_holdings(&self, symbol: &str) -> Vec<InstitutionalHolding> {
        vec![
            InstitutionalHolding {
                symbol: symbol.to_string(),
                institution_name: "Vanguard Group Inc".to_string(),
                shares: 1_250_000_000.0,
                value: 223_125_000_000.0,
                percent_of_portfolio: 8.5,
                change_shares: 25_000_000.0,
                change_percent: 2.04,
                quarter: "Q4 2024".to_string(),
                is_whale: true,
            },
            InstitutionalHolding {
                symbol: symbol.to_string(),
                institution_name: "BlackRock Inc".to_string(),
                shares: 1_100_000_000.0,
                value: 196_350_000_000.0,
                percent_of_portfolio: 7.2,
                change_shares: -15_000_000.0,
                change_percent: -1.35,
                quarter: "Q4 2024".to_string(),
                is_whale: true,
            },
        ]
    }

    fn generate_mock_insider_activity(&self, symbol: &str) -> Vec<InsiderActivity> {
        vec![
            InsiderActivity {
                symbol: symbol.to_string(),
                insider_name: "John Smith".to_string(),
                insider_title: "CEO".to_string(),
                transaction_type: TransactionType::Buy,
                shares: 50_000.0,
                price: 175.50,
                value: 8_775_000.0,
                transaction_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                filing_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                is_significant: true,
            },
            InsiderActivity {
                symbol: symbol.to_string(),
                insider_name: "Jane Doe".to_string(),
                insider_title: "CFO".to_string(),
                transaction_type: TransactionType::Sell,
                shares: 10_000.0,
                price: 178.25,
                value: 1_782_500.0,
                transaction_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                filing_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                is_significant: false,
            },
        ]
    }
}
