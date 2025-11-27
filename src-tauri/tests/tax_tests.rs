use app_lib::tax::types::{TaxJurisdiction, TaxSettings};
use app_lib::tax::TaxPlanningEngine;

#[tokio::test]
async fn test_tax_engine_initialization() {
    let jurisdiction = TaxJurisdiction::us_federal();
    let mut engine = TaxPlanningEngine::new(jurisdiction);

    assert_eq!(engine.settings.jurisdiction.code, "US");
    assert_eq!(engine.settings.enable_wash_sale_detection, true);

    engine.generate_mock_transactions();
    assert!(!engine.recent_transactions.is_empty());
}

#[tokio::test]
async fn test_jurisdiction_types() {
    let us = TaxJurisdiction::us_federal();
    assert_eq!(us.code, "US");
    assert_eq!(us.short_term_rate, 0.37);
    assert_eq!(us.long_term_rate, 0.20);
    assert_eq!(us.holding_period_days, 365);

    let uk = TaxJurisdiction::uk();
    assert_eq!(uk.code, "UK");
    assert_eq!(uk.holding_period_days, 0);

    let de = TaxJurisdiction::germany();
    assert_eq!(de.code, "DE");
    assert_eq!(de.long_term_rate, 0.0); // No capital gains tax after 1 year

    let au = TaxJurisdiction::australia();
    assert_eq!(au.code, "AU");
}

#[test]
fn test_default_tax_settings() {
    let settings = TaxSettings::default();
    assert_eq!(settings.jurisdiction.code, "US");
    assert_eq!(settings.default_cost_basis_method, "FIFO");
    assert_eq!(settings.enable_wash_sale_detection, true);
    assert_eq!(settings.enable_tax_loss_harvesting, true);
}

#[test]
fn test_current_prices() {
    let jurisdiction = TaxJurisdiction::us_federal();
    let engine = TaxPlanningEngine::new(jurisdiction);

    assert!(engine.current_prices.contains_key("SOL"));
    assert!(engine.current_prices.contains_key("BTC"));
    assert!(engine.current_prices.contains_key("ETH"));
}
