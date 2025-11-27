use crate::defi::auto_compound::AutoCompoundEngine;
use crate::defi::kamino::KaminoAdapter;
use crate::defi::marginfi::MarginfiAdapter;
use crate::defi::solend::SolendAdapter;
use crate::defi::staking::StakingAdapter;
use crate::defi::types::*;
use crate::defi::yield_farming::YieldFarmingAdapter;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct PositionSnapshot {
    pub wallet: String,
    pub summary: PortfolioSummary,
    pub risk_metrics: Vec<RiskMetrics>,
}

#[derive(Clone)]
pub struct PositionManager {
    solend: SolendAdapter,
    marginfi: MarginfiAdapter,
    kamino: KaminoAdapter,
    staking: StakingAdapter,
    farming: YieldFarmingAdapter,
}

impl Default for PositionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PositionManager {
    pub fn new() -> Self {
        Self {
            solend: SolendAdapter::new(),
            marginfi: MarginfiAdapter::new(),
            kamino: KaminoAdapter::new(),
            staking: StakingAdapter::new(),
            farming: YieldFarmingAdapter::new(),
        }
    }

    pub async fn build_portfolio_summary(&self, wallet: &str) -> Result<PortfolioSummary, String> {
        let solend_positions = self.solend.get_user_positions(wallet).await?;
        let marginfi_positions = self.marginfi.get_positions(wallet).await?;
        let kamino_positions = self.kamino.get_user_positions(wallet).await?;
        let staking_positions = self.staking.get_positions(wallet).await?;
        let farming_positions = self.farming.get_positions(wallet).await?;

        let mut positions = Vec::new();
        positions.extend(solend_positions);
        positions.extend(marginfi_positions);
        positions.extend(kamino_positions);
        positions.extend(staking_positions);
        positions.extend(farming_positions);

        let total_value_usd: f64 = positions.iter().map(|p| p.value_usd).sum();
        let lending_value = positions
            .iter()
            .filter(|p| p.position_type == PositionType::Lending)
            .map(|p| p.value_usd)
            .sum();
        let borrowing_value = positions
            .iter()
            .filter(|p| p.position_type == PositionType::Borrowing)
            .map(|p| p.value_usd)
            .sum();
        let lp_value = positions
            .iter()
            .filter(|p| p.position_type == PositionType::LiquidityPool)
            .map(|p| p.value_usd)
            .sum();
        let staking_value = positions
            .iter()
            .filter(|p| p.position_type == PositionType::Staking)
            .map(|p| p.value_usd)
            .sum();
        let farming_value = positions
            .iter()
            .filter(|p| p.position_type == PositionType::Farming)
            .map(|p| p.value_usd)
            .sum();

        let average_apy = if positions.is_empty() {
            0.0
        } else {
            positions.iter().map(|p| p.apy).sum::<f64>() / positions.len() as f64
        };

        let total_earnings_24h = total_value_usd * (average_apy / 100.0) / 365.0;

        Ok(PortfolioSummary {
            total_value_usd,
            lending_value,
            borrowing_value,
            lp_value,
            staking_value,
            farming_value,
            total_earnings_24h,
            average_apy,
            positions,
        })
    }

    pub async fn calculate_risk_metrics(&self, wallet: &str) -> Result<Vec<RiskMetrics>, String> {
        let summary = self.build_portfolio_summary(wallet).await?;
        let mut metrics = Vec::new();

        for position in summary.positions.iter() {
            let risk_level = match position.position_type {
                PositionType::Borrowing => {
                    if let Some(hf) = position.health_factor {
                        if hf < 1.1 {
                            RiskLevel::Critical
                        } else if hf < 1.5 {
                            RiskLevel::High
                        } else if hf < 2.0 {
                            RiskLevel::Medium
                        } else {
                            RiskLevel::Low
                        }
                    } else {
                        RiskLevel::Medium
                    }
                }
                PositionType::LiquidityPool | PositionType::Farming => {
                    if position.apy > 30.0 {
                        RiskLevel::High
                    } else {
                        RiskLevel::Medium
                    }
                }
                PositionType::Staking => RiskLevel::Low,
                PositionType::Lending => RiskLevel::Low,
            };

            let mut warnings = Vec::new();
            if risk_level == RiskLevel::High || risk_level == RiskLevel::Critical {
                warnings.push("Position health requires attention".to_string());
            }
            if position.apy > 35.0 {
                warnings.push("Yield may be unsustainable".to_string());
            }

            metrics.push(RiskMetrics {
                position_id: position.id.clone(),
                risk_level,
                liquidation_price: None,
                health_factor: position.health_factor,
                collateral_ratio: position.health_factor.map(|hf| hf * 0.5),
                warnings,
            });
        }

        Ok(metrics)
    }

    pub async fn snapshot(&self, wallet: &str) -> Result<PositionSnapshot, String> {
        let summary = self.build_portfolio_summary(wallet).await?;
        let risk_metrics = self.calculate_risk_metrics(wallet).await?;
        Ok(PositionSnapshot {
            wallet: wallet.to_string(),
            summary,
            risk_metrics,
        })
    }

    pub async fn recommend_auto_compound(
        &self,
        wallet: &str,
    ) -> Result<Vec<AutoCompoundSettings>, String> {
        let summary = self.build_portfolio_summary(wallet).await?;
        let auto_compound = AutoCompoundEngine::default();
        let recommendations = auto_compound.analyze_positions(&summary.positions).await;
        Ok(recommendations)
    }
}

#[tauri::command]
pub async fn get_defi_portfolio_summary(wallet: String) -> Result<PortfolioSummary, String> {
    PositionManager::new()
        .build_portfolio_summary(&wallet)
        .await
}

#[tauri::command]
pub async fn get_defi_risk_metrics(wallet: String) -> Result<Vec<RiskMetrics>, String> {
    PositionManager::new().calculate_risk_metrics(&wallet).await
}

#[tauri::command]
pub async fn get_defi_snapshot(wallet: String) -> Result<PositionSnapshot, String> {
    PositionManager::new().snapshot(&wallet).await
}

#[tauri::command]
pub async fn get_auto_compound_recommendations(
    wallet: String,
) -> Result<Vec<AutoCompoundSettings>, String> {
    PositionManager::new()
        .recommend_auto_compound(&wallet)
        .await
}
