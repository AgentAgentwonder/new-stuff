use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::Position;
use crate::market::PricePoint;

// ==================== Data Types ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationMatrix {
    pub symbols: Vec<String>,
    pub matrix: Vec<Vec<f64>>,
    #[serde(rename = "calculatedAt")]
    pub calculated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiversificationMetrics {
    pub score: f64,
    #[serde(rename = "effectiveN")]
    pub effective_n: f64,
    #[serde(rename = "avgCorrelation")]
    pub avg_correlation: f64,
    #[serde(rename = "concentrationRisk")]
    pub concentration_risk: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConcentration {
    pub symbol: String,
    pub allocation: f64,
    #[serde(rename = "riskLevel")]
    pub risk_level: String, // "low", "medium", "high", "critical"
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharpeMetrics {
    #[serde(rename = "sharpeRatio")]
    pub sharpe_ratio: f64,
    #[serde(rename = "annualizedReturn")]
    pub annualized_return: f64,
    #[serde(rename = "annualizedVolatility")]
    pub annualized_volatility: f64,
    #[serde(rename = "riskFreeRate")]
    pub risk_free_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorExposure {
    pub name: String,
    pub beta: f64,
    pub exposure: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorAnalysis {
    pub factors: Vec<FactorExposure>,
    #[serde(rename = "marketBeta")]
    pub market_beta: f64,
    #[serde(rename = "systematicRisk")]
    pub systematic_risk: f64,
    #[serde(rename = "specificRisk")]
    pub specific_risk: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioAnalytics {
    pub correlation: CorrelationMatrix,
    pub diversification: DiversificationMetrics,
    pub concentration: Vec<RiskConcentration>,
    pub sharpe: SharpeMetrics,
    pub factors: FactorAnalysis,
    #[serde(rename = "calculatedAt")]
    pub calculated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorAllocation {
    pub sector: String,
    pub allocation: f64,
    pub value: f64,
    pub symbols: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcentrationAlert {
    pub id: String,
    pub symbol: String,
    pub allocation: f64,
    pub severity: String, // "warning", "critical"
    pub message: String,
    pub threshold: f64,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

// ==================== Cache ====================

struct CacheEntry<T> {
    data: T,
    timestamp: DateTime<Utc>,
    ttl_seconds: i64,
}

impl<T: Clone> CacheEntry<T> {
    fn is_valid(&self) -> bool {
        let age = Utc::now().signed_duration_since(self.timestamp);
        age.num_seconds() < self.ttl_seconds
    }

    fn get(&self) -> Option<T> {
        if self.is_valid() {
            Some(self.data.clone())
        } else {
            None
        }
    }
}

lazy_static! {
    static ref ANALYTICS_CACHE: RwLock<HashMap<String, CacheEntry<PortfolioAnalytics>>> =
        RwLock::new(HashMap::new());
    static ref CORRELATION_CACHE: RwLock<HashMap<String, CacheEntry<CorrelationMatrix>>> =
        RwLock::new(HashMap::new());
}

const DEFAULT_CACHE_TTL: i64 = 300; // 5 minutes

// ==================== Statistical Functions ====================

fn calculate_returns(prices: &[f64]) -> Vec<f64> {
    let mut returns = Vec::new();
    for i in 1..prices.len() {
        if prices[i - 1] != 0.0 {
            returns.push((prices[i] - prices[i - 1]) / prices[i - 1]);
        }
    }
    returns
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

fn variance(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let avg = mean(values);
    let squared_diffs: f64 = values.iter().map(|v| (v - avg).powi(2)).sum();
    squared_diffs / values.len() as f64
}

fn std_dev(values: &[f64]) -> f64 {
    variance(values).sqrt()
}

fn covariance(x: &[f64], y: &[f64]) -> f64 {
    if x.len() != y.len() || x.is_empty() {
        return 0.0;
    }

    let mean_x = mean(x);
    let mean_y = mean(y);

    let cov: f64 = x
        .iter()
        .zip(y.iter())
        .map(|(xi, yi)| (xi - mean_x) * (yi - mean_y))
        .sum();

    cov / x.len() as f64
}

fn correlation(x: &[f64], y: &[f64]) -> f64 {
    let cov = covariance(x, y);
    let std_x = std_dev(x);
    let std_y = std_dev(y);

    if std_x == 0.0 || std_y == 0.0 {
        return 0.0;
    }

    cov / (std_x * std_y)
}

// ==================== Core Analytics Functions ====================

pub fn calculate_correlation_matrix(
    time_series: &HashMap<String, Vec<PricePoint>>,
) -> CorrelationMatrix {
    let symbols: Vec<String> = time_series.keys().cloned().collect();
    let n = symbols.len();

    let mut matrix = vec![vec![0.0; n]; n];

    // Extract price series for each symbol
    let mut price_series: HashMap<String, Vec<f64>> = HashMap::new();
    for (symbol, points) in time_series.iter() {
        let prices: Vec<f64> = points.iter().map(|p| p.close).collect();
        let returns = calculate_returns(&prices);
        price_series.insert(symbol.clone(), returns);
    }

    // Calculate pairwise correlations
    for (i, sym_i) in symbols.iter().enumerate() {
        for (j, sym_j) in symbols.iter().enumerate() {
            if i == j {
                matrix[i][j] = 1.0;
            } else if i < j {
                let corr = if let (Some(series_i), Some(series_j)) =
                    (price_series.get(sym_i), price_series.get(sym_j))
                {
                    correlation(series_i, series_j)
                } else {
                    0.0
                };
                matrix[i][j] = corr;
                matrix[j][i] = corr; // Symmetric
            }
        }
    }

    CorrelationMatrix {
        symbols,
        matrix,
        calculated_at: Utc::now().to_rfc3339(),
    }
}

pub fn calculate_diversification_score(
    positions: &[Position],
    correlation_matrix: &CorrelationMatrix,
) -> DiversificationMetrics {
    let n = positions.len() as f64;

    if n == 0.0 {
        return DiversificationMetrics {
            score: 0.0,
            effective_n: 0.0,
            avg_correlation: 0.0,
            concentration_risk: 0.0,
        };
    }

    // Calculate Herfindahl index for concentration
    let herfindahl: f64 = positions
        .iter()
        .map(|p| (p.allocation / 100.0).powi(2))
        .sum();

    let effective_n = if herfindahl > 0.0 {
        1.0 / herfindahl
    } else {
        0.0
    };

    // Calculate average correlation
    let mut sum_corr = 0.0;
    let mut count = 0;
    for i in 0..correlation_matrix.matrix.len() {
        for j in (i + 1)..correlation_matrix.matrix[i].len() {
            sum_corr += correlation_matrix.matrix[i][j];
            count += 1;
        }
    }
    let avg_correlation = if count > 0 {
        sum_corr / count as f64
    } else {
        0.0
    };

    // Diversification score: weighted by effective N and inverse of correlation
    // Score ranges from 0-100
    let score = ((effective_n / n) * (1.0 - avg_correlation.abs()) * 100.0).min(100.0);

    DiversificationMetrics {
        score,
        effective_n,
        avg_correlation,
        concentration_risk: herfindahl,
    }
}

pub fn calculate_risk_concentration(positions: &[Position]) -> Vec<RiskConcentration> {
    positions
        .iter()
        .map(|p| {
            let (risk_level, recommendation) = if p.allocation >= 40.0 {
                (
                    "critical",
                    "Critically high concentration. Consider immediate rebalancing.",
                )
            } else if p.allocation >= 30.0 {
                (
                    "high",
                    "High concentration risk. Recommend diversifying into other assets.",
                )
            } else if p.allocation >= 20.0 {
                (
                    "medium",
                    "Moderate concentration. Monitor allocation closely.",
                )
            } else {
                ("low", "Healthy allocation level.")
            };

            RiskConcentration {
                symbol: p.symbol.clone(),
                allocation: p.allocation,
                risk_level: risk_level.to_string(),
                recommendation: recommendation.to_string(),
            }
        })
        .collect()
}

pub fn calculate_sharpe_ratio(
    time_series: &HashMap<String, Vec<PricePoint>>,
    positions: &[Position],
    risk_free_rate: f64, // Annual rate, e.g., 0.03 for 3%
) -> SharpeMetrics {
    // Calculate portfolio returns
    let mut portfolio_returns = Vec::new();

    // Get the minimum length across all series
    let min_len = time_series.values().map(|v| v.len()).min().unwrap_or(0);

    if min_len < 2 {
        return SharpeMetrics {
            sharpe_ratio: 0.0,
            annualized_return: 0.0,
            annualized_volatility: 0.0,
            risk_free_rate,
        };
    }

    // Calculate weighted returns for each time period
    for i in 1..min_len {
        let mut period_return = 0.0;
        for pos in positions {
            if let Some(series) = time_series.get(&pos.symbol) {
                if i < series.len() && series[i - 1].close != 0.0 {
                    let ret = (series[i].close - series[i - 1].close) / series[i - 1].close;
                    period_return += ret * (pos.allocation / 100.0);
                }
            }
        }
        portfolio_returns.push(period_return);
    }

    let avg_return = mean(&portfolio_returns);
    let volatility = std_dev(&portfolio_returns);

    // Annualize (assuming hourly data)
    let periods_per_year = 365.25 * 24.0;
    let annualized_return = avg_return * periods_per_year;
    let annualized_volatility = volatility * periods_per_year.sqrt();

    let sharpe_ratio = if annualized_volatility != 0.0 {
        (annualized_return - risk_free_rate) / annualized_volatility
    } else {
        0.0
    };

    SharpeMetrics {
        sharpe_ratio,
        annualized_return,
        annualized_volatility,
        risk_free_rate,
    }
}

pub fn calculate_factor_analysis(
    time_series: &HashMap<String, Vec<PricePoint>>,
    positions: &[Position],
    market_returns: &[f64],
) -> FactorAnalysis {
    // Calculate portfolio returns
    let min_len = time_series
        .values()
        .map(|v| v.len())
        .min()
        .unwrap_or(0)
        .min(market_returns.len());

    if min_len < 2 {
        return FactorAnalysis {
            factors: vec![],
            market_beta: 1.0,
            systematic_risk: 0.0,
            specific_risk: 0.0,
        };
    }

    let mut portfolio_returns = Vec::new();
    for i in 1..min_len {
        let mut period_return = 0.0;
        for pos in positions {
            if let Some(series) = time_series.get(&pos.symbol) {
                if i < series.len() && series[i - 1].close != 0.0 {
                    let ret = (series[i].close - series[i - 1].close) / series[i - 1].close;
                    period_return += ret * (pos.allocation / 100.0);
                }
            }
        }
        portfolio_returns.push(period_return);
    }

    // Calculate market beta
    let market_slice = &market_returns[0..min_len.min(market_returns.len())];
    let cov = covariance(&portfolio_returns, market_slice);
    let market_var = variance(market_slice);
    let market_beta = if market_var != 0.0 {
        cov / market_var
    } else {
        1.0
    };

    // Calculate systematic and specific risk
    let portfolio_var = variance(&portfolio_returns);
    let systematic_risk = market_beta.powi(2) * market_var;
    let specific_risk = (portfolio_var - systematic_risk).max(0.0);

    // Mock factor exposures (in real implementation, would use multi-factor models)
    let factors = vec![
        FactorExposure {
            name: "Market".to_string(),
            beta: market_beta,
            exposure: market_beta,
        },
        FactorExposure {
            name: "Size".to_string(),
            beta: 0.5,
            exposure: 0.5 * market_beta,
        },
        FactorExposure {
            name: "Momentum".to_string(),
            beta: mean(&portfolio_returns),
            exposure: 0.3,
        },
    ];

    FactorAnalysis {
        factors,
        market_beta,
        systematic_risk,
        specific_risk,
    }
}

pub fn check_concentration_alerts(positions: &[Position]) -> Vec<ConcentrationAlert> {
    let mut alerts = Vec::new();

    for pos in positions {
        if pos.allocation >= 40.0 {
            alerts.push(ConcentrationAlert {
                id: uuid::Uuid::new_v4().to_string(),
                symbol: pos.symbol.clone(),
                allocation: pos.allocation,
                severity: "critical".to_string(),
                message: format!(
                    "{} represents {:.1}% of your portfolio. Critical concentration risk detected. \
                    Recommend reducing allocation to below 30%.",
                    pos.symbol, pos.allocation
                ),
                threshold: 40.0,
                created_at: Utc::now().to_rfc3339(),
            });
        } else if pos.allocation >= 30.0 {
            alerts.push(ConcentrationAlert {
                id: uuid::Uuid::new_v4().to_string(),
                symbol: pos.symbol.clone(),
                allocation: pos.allocation,
                severity: "warning".to_string(),
                message: format!(
                    "{} represents {:.1}% of your portfolio. Consider diversifying to reduce concentration risk.",
                    pos.symbol, pos.allocation
                ),
                threshold: 30.0,
                created_at: Utc::now().to_rfc3339(),
            });
        }
    }

    alerts
}

// ==================== Caching Functions ====================

fn cache_key(positions: &[Position]) -> String {
    let symbols: Vec<&str> = positions.iter().map(|p| p.symbol.as_str()).collect();
    symbols.join("_")
}

pub fn get_cached_analytics(positions: &[Position]) -> Option<PortfolioAnalytics> {
    let key = cache_key(positions);
    let cache = ANALYTICS_CACHE.read();
    cache.get(&key).and_then(|entry| entry.get())
}

pub fn cache_analytics(positions: &[Position], analytics: PortfolioAnalytics) {
    let key = cache_key(positions);
    let entry = CacheEntry {
        data: analytics,
        timestamp: Utc::now(),
        ttl_seconds: DEFAULT_CACHE_TTL,
    };
    let mut cache = ANALYTICS_CACHE.write();
    cache.insert(key, entry);
}

pub fn clear_analytics_cache() {
    let mut cache = ANALYTICS_CACHE.write();
    cache.clear();
}

// ==================== Sector Classification ====================

pub fn classify_sector(symbol: &str) -> String {
    match symbol {
        "SOL" | "ETH" | "BTC" => "Layer 1",
        "JUP" | "ORCA" | "RAYD" => "DeFi",
        "BONK" | "SAMO" | "WIF" => "Meme",
        "USDC" | "USDT" | "DAI" => "Stablecoin",
        "PYTH" | "LINK" => "Oracle",
        "RNDR" | "FIL" | "AR" => "Storage/Compute",
        _ => "Other",
    }
    .to_string()
}

pub fn calculate_sector_allocation(positions: &[Position]) -> Vec<SectorAllocation> {
    let mut sector_map: HashMap<String, (f64, f64, Vec<String>)> = HashMap::new();

    for pos in positions {
        let sector = classify_sector(&pos.symbol);
        let entry = sector_map
            .entry(sector.clone())
            .or_insert((0.0, 0.0, Vec::new()));
        entry.0 += pos.allocation;
        entry.1 += pos.total_value;
        entry.2.push(pos.symbol.clone());
    }

    sector_map
        .into_iter()
        .map(|(sector, (allocation, value, symbols))| SectorAllocation {
            sector,
            allocation,
            value,
            symbols,
        })
        .collect()
}

// ==================== Tauri Commands ====================

#[tauri::command]
pub async fn calculate_portfolio_analytics(
    positions: Vec<Position>,
    time_series: HashMap<String, Vec<PricePoint>>,
    risk_free_rate: Option<f64>,
) -> Result<PortfolioAnalytics, String> {
    // Check cache first
    if let Some(cached) = get_cached_analytics(&positions) {
        return Ok(cached);
    }

    let risk_free = risk_free_rate.unwrap_or(0.03);

    // Calculate correlation matrix
    let correlation = calculate_correlation_matrix(&time_series);

    // Calculate diversification metrics
    let diversification = calculate_diversification_score(&positions, &correlation);

    // Calculate concentration risks
    let concentration = calculate_risk_concentration(&positions);

    // Calculate Sharpe ratio
    let sharpe = calculate_sharpe_ratio(&time_series, &positions, risk_free);

    // Generate mock market returns for factor analysis
    let market_returns: Vec<f64> = time_series
        .values()
        .next()
        .map(|series| {
            let prices: Vec<f64> = series.iter().map(|p| p.close).collect();
            calculate_returns(&prices)
        })
        .unwrap_or_default();

    let factors = calculate_factor_analysis(&time_series, &positions, &market_returns);

    let analytics = PortfolioAnalytics {
        correlation,
        diversification,
        concentration,
        sharpe,
        factors,
        calculated_at: Utc::now().to_rfc3339(),
    };

    // Cache the result
    cache_analytics(&positions, analytics.clone());

    Ok(analytics)
}

#[tauri::command]
pub async fn get_concentration_alerts(
    positions: Vec<Position>,
) -> Result<Vec<ConcentrationAlert>, String> {
    Ok(check_concentration_alerts(&positions))
}

#[tauri::command]
pub async fn get_sector_allocation(
    positions: Vec<Position>,
) -> Result<Vec<SectorAllocation>, String> {
    Ok(calculate_sector_allocation(&positions))
}

#[tauri::command]
pub async fn clear_portfolio_cache() -> Result<(), String> {
    clear_analytics_cache();
    Ok(())
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_positions() -> Vec<Position> {
        vec![
            Position {
                symbol: "SOL".to_string(),
                mint: "So11111111111111111111111111111111111111112".to_string(),
                amount: 100.0,
                current_price: 100.0,
                avg_entry_price: 90.0,
                total_value: 10000.0,
                unrealized_pnl: 1000.0,
                unrealized_pnl_percent: 11.11,
                allocation: 50.0,
            },
            Position {
                symbol: "JUP".to_string(),
                mint: "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN".to_string(),
                amount: 5000.0,
                current_price: 1.5,
                avg_entry_price: 1.0,
                total_value: 7500.0,
                unrealized_pnl: 2500.0,
                unrealized_pnl_percent: 50.0,
                allocation: 37.5,
            },
            Position {
                symbol: "BONK".to_string(),
                mint: "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_string(),
                amount: 1000000.0,
                current_price: 0.0025,
                avg_entry_price: 0.002,
                total_value: 2500.0,
                unrealized_pnl: 500.0,
                unrealized_pnl_percent: 25.0,
                allocation: 12.5,
            },
        ]
    }

    fn create_test_time_series() -> HashMap<String, Vec<PricePoint>> {
        let mut map = HashMap::new();

        let prices = vec![100.0, 102.0, 101.0, 103.0, 105.0];
        let sol_series: Vec<PricePoint> = prices
            .iter()
            .enumerate()
            .map(|(i, &price)| PricePoint {
                timestamp: 1700000000 + (i as i64 * 3600),
                open: price,
                high: price * 1.01,
                low: price * 0.99,
                close: price,
                volume: 1000000.0,
            })
            .collect();

        let jup_series: Vec<PricePoint> = prices
            .iter()
            .enumerate()
            .map(|(i, &price)| PricePoint {
                timestamp: 1700000000 + (i as i64 * 3600),
                open: price * 0.015,
                high: price * 0.015 * 1.01,
                low: price * 0.015 * 0.99,
                close: price * 0.015,
                volume: 500000.0,
            })
            .collect();

        map.insert("SOL".to_string(), sol_series);
        map.insert("JUP".to_string(), jup_series);
        map
    }

    #[test]
    fn test_calculate_returns() {
        let prices = vec![100.0, 105.0, 103.0, 107.0];
        let returns = calculate_returns(&prices);

        assert_eq!(returns.len(), 3);
        assert!((returns[0] - 0.05).abs() < 1e-6);
        assert!((returns[1] - (-0.019047619)).abs() < 1e-6);
    }

    #[test]
    fn test_mean_and_variance() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        assert_eq!(mean(&values), 3.0);
        assert_eq!(variance(&values), 2.0);
    }

    #[test]
    fn test_correlation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];

        let corr = correlation(&x, &y);
        assert!((corr - 1.0).abs() < 1e-6); // Perfect positive correlation
    }

    #[test]
    fn test_correlation_matrix() {
        let time_series = create_test_time_series();
        let matrix = calculate_correlation_matrix(&time_series);

        assert_eq!(matrix.symbols.len(), 2);
        assert_eq!(matrix.matrix.len(), 2);
        assert_eq!(matrix.matrix[0].len(), 2);

        // Diagonal should be 1.0
        assert_eq!(matrix.matrix[0][0], 1.0);
        assert_eq!(matrix.matrix[1][1], 1.0);

        // Matrix should be symmetric
        assert_eq!(matrix.matrix[0][1], matrix.matrix[1][0]);
    }

    #[test]
    fn test_diversification_score() {
        let positions = create_test_positions();
        let time_series = create_test_time_series();
        let correlation = calculate_correlation_matrix(&time_series);
        let metrics = calculate_diversification_score(&positions, &correlation);

        assert!(metrics.score >= 0.0 && metrics.score <= 100.0);
        assert!(metrics.effective_n > 0.0);
        assert!(metrics.concentration_risk >= 0.0);
    }

    #[test]
    fn test_risk_concentration() {
        let positions = create_test_positions();
        let concentration = calculate_risk_concentration(&positions);

        assert_eq!(concentration.len(), 3);

        // SOL at 50% should be critical
        let sol_risk = concentration.iter().find(|c| c.symbol == "SOL").unwrap();
        assert_eq!(sol_risk.risk_level, "critical");

        // JUP at 37.5% should be high
        let jup_risk = concentration.iter().find(|c| c.symbol == "JUP").unwrap();
        assert_eq!(jup_risk.risk_level, "high");

        // BONK at 12.5% should be low
        let bonk_risk = concentration.iter().find(|c| c.symbol == "BONK").unwrap();
        assert_eq!(bonk_risk.risk_level, "low");
    }

    #[test]
    fn test_sharpe_ratio() {
        let positions = create_test_positions();
        let time_series = create_test_time_series();
        let metrics = calculate_sharpe_ratio(&time_series, &positions, 0.03);

        assert!(metrics.annualized_return.is_finite());
        assert!(metrics.annualized_volatility >= 0.0);
    }

    #[test]
    fn test_concentration_alerts() {
        let positions = create_test_positions();
        let alerts = check_concentration_alerts(&positions);

        assert!(!alerts.is_empty());

        // Should have at least one critical alert (SOL at 50%)
        let critical_alerts: Vec<_> = alerts.iter().filter(|a| a.severity == "critical").collect();
        assert!(!critical_alerts.is_empty());
    }

    #[test]
    fn test_sector_classification() {
        assert_eq!(classify_sector("SOL"), "Layer 1");
        assert_eq!(classify_sector("JUP"), "DeFi");
        assert_eq!(classify_sector("BONK"), "Meme");
        assert_eq!(classify_sector("USDC"), "Stablecoin");
        assert_eq!(classify_sector("UNKNOWN"), "Other");
    }

    #[test]
    fn test_sector_allocation() {
        let positions = create_test_positions();
        let allocation = calculate_sector_allocation(&positions);

        assert!(!allocation.is_empty());

        let total_allocation: f64 = allocation.iter().map(|a| a.allocation).sum();
        assert!((total_allocation - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_cache_operations() {
        let positions = create_test_positions();
        let time_series = create_test_time_series();
        let correlation = calculate_correlation_matrix(&time_series);

        let analytics = PortfolioAnalytics {
            correlation: correlation.clone(),
            diversification: calculate_diversification_score(&positions, &correlation),
            concentration: calculate_risk_concentration(&positions),
            sharpe: calculate_sharpe_ratio(&time_series, &positions, 0.03),
            factors: FactorAnalysis {
                factors: vec![],
                market_beta: 1.0,
                systematic_risk: 0.0,
                specific_risk: 0.0,
            },
            calculated_at: Utc::now().to_rfc3339(),
        };

        // Cache the analytics
        cache_analytics(&positions, analytics.clone());

        // Retrieve from cache
        let cached = get_cached_analytics(&positions);
        assert!(cached.is_some());

        // Clear cache
        clear_analytics_cache();
        let after_clear = get_cached_analytics(&positions);
        assert!(after_clear.is_none());
    }
}
