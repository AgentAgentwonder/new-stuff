#[cfg(test)]
mod tests {
    use app_lib::trading::safety::cooldown::CooldownManager;
    use app_lib::trading::safety::insurance::InsuranceCoordinator;
    use app_lib::trading::safety::policy::{PolicyEngine, SafetyPolicy};
    use app_lib::trading::safety::simulator::TransactionSimulator;
    use app_lib::trading::safety::{SafetyCheckRequest, SafetyEngine};
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_policy_enforcement() {
        let mut policy = SafetyPolicy::default();
        policy.max_trade_amount_usd = Some(5000.0);
        policy.max_daily_trades = Some(3);

        let mut engine = PolicyEngine::new(policy);

        // Test 1: Trade within limits
        let result = engine.check_trade_policy("wallet1", 2000.0, 2.0, 0.5, Some(80.0));
        assert!(result.allowed);
        assert!(result.violations.is_empty());

        // Test 2: Trade exceeds amount limit
        let result = engine.check_trade_policy("wallet1", 7000.0, 2.0, 0.5, Some(80.0));
        assert!(!result.allowed);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].rule, "max_trade_amount");

        // Test 3: High price impact
        let result = engine.check_trade_policy("wallet1", 2000.0, 15.0, 0.5, Some(80.0));
        assert!(!result.allowed);
        assert!(result
            .violations
            .iter()
            .any(|v| v.rule == "max_price_impact"));

        // Test 4: Daily trade limit
        for _ in 0..3 {
            engine.increment_daily_trade_count("wallet1");
        }
        let result = engine.check_trade_policy("wallet1", 2000.0, 2.0, 0.5, Some(80.0));
        assert!(!result.allowed);
        assert!(result
            .violations
            .iter()
            .any(|v| v.rule == "max_daily_trades"));
    }

    #[test]
    fn test_cooldown_logic() {
        let mut manager = CooldownManager::new(2);

        // Record trade
        manager.record_trade("wallet1");

        // Should be on cooldown
        assert!(manager.is_on_cooldown("wallet1"));

        // Check status
        let status = manager.get_remaining_cooldown("wallet1");
        assert!(status.is_some());
        let status = status.unwrap();
        assert_eq!(status.wallet_address, "wallet1");
        assert!(status.remaining_seconds <= 2);

        // Wait for cooldown to expire
        sleep(Duration::from_secs(3));

        // Should not be on cooldown anymore
        assert!(!manager.is_on_cooldown("wallet1"));
        assert!(manager.get_remaining_cooldown("wallet1").is_none());
    }

    #[tokio::test]
    async fn test_transaction_simulation() {
        let simulator = TransactionSimulator::default();

        // Test 1: Normal trade
        let result = simulator
            .simulate_transaction(1000.0, "SOL", "USDC", 50)
            .await;
        assert!(result.is_ok());
        let sim = result.unwrap();
        assert!(sim.expected_output > 0.0);
        assert!(sim.minimum_output < sim.expected_output);
        assert!(sim.maximum_output > sim.expected_output);
        assert!(sim.success_probability > 0.0);

        // Test 2: High value trade with MEV risk
        let result = simulator
            .simulate_transaction(150000.0, "SOL", "USDC", 50)
            .await;
        assert!(result.is_ok());
        let sim = result.unwrap();
        assert!(sim.price_impact > 1.0);
        assert!(sim.mev_loss_estimate > 0.0);
    }

    #[tokio::test]
    async fn test_impact_preview() {
        let simulator = TransactionSimulator::default();

        let result = simulator
            .preview_impact(5000.0, "SOL".to_string(), "USDC".to_string(), 0.5)
            .await;

        assert!(result.is_ok());
        let preview = result.unwrap();
        assert_eq!(preview.input_amount, 5000.0);
        assert_eq!(preview.input_symbol, "SOL");
        assert_eq!(preview.output_symbol, "USDC");
        assert!(preview.output_amount > 0.0);
        assert!(preview.total_fees > 0.0);
        assert!(!preview.route.is_empty());
    }

    #[test]
    fn test_insurance_provider_listing() {
        let coordinator = InsuranceCoordinator::default();
        let providers = coordinator.list_providers();

        assert!(!providers.is_empty());
        assert!(providers.iter().all(|p| p.is_active));
    }

    #[test]
    fn test_insurance_quote() {
        let mut coordinator = InsuranceCoordinator::default();

        // Test 1: Normal trade
        let quote = coordinator.request_quote("sol_shield", 50000.0, 2.0, 0.3);
        assert!(quote.is_ok());
        let quote = quote.unwrap();
        assert_eq!(quote.provider_id, "sol_shield");
        assert!(quote.total_premium_usd > 0.0);
        assert!(quote.coverage_amount_usd > 0.0);

        // Test 2: High risk trade
        let quote = coordinator.request_quote("sol_shield", 100000.0, 5.0, 0.8);
        assert!(quote.is_ok());
        let quote = quote.unwrap();
        assert!(quote.mev_protection_included);
        assert!(quote.total_premium_usd > 0.0);
    }

    #[test]
    fn test_insurance_recommendation() {
        let mut coordinator = InsuranceCoordinator::default();

        let recommendation = coordinator.recommend_provider(75000.0, 3.5, 0.4);

        assert!(recommendation.is_some());
        let quote = recommendation.unwrap();
        assert!(quote.coverage_amount_usd > 0.0);
        assert!(quote.total_premium_usd > 0.0);
    }

    #[tokio::test]
    async fn test_safety_engine_full_check() {
        let policy = SafetyPolicy::default();
        let mut engine = SafetyEngine::new(policy, 30);

        let request = SafetyCheckRequest {
            wallet_address: "wallet1".to_string(),
            input_amount: 100.0,
            input_mint: "SOL".to_string(),
            output_mint: "USDC".to_string(),
            input_symbol: "SOL".to_string(),
            output_symbol: "USDC".to_string(),
            amount_usd: 5000.0,
            slippage_bps: 50,
            price_impact_percent: 1.5,
            security_score: Some(85.0),
        };

        // Test 1: First trade should be allowed
        let result = engine.check_trade_safety(request.clone()).await;
        assert!(result.is_ok());
        let check = result.unwrap();
        assert!(check.allowed);
        assert!(check.simulation.is_some());
        assert!(check.impact_preview.is_some());

        // Approve the trade
        engine.approve_trade("wallet1");

        // Test 2: Second immediate trade should be blocked by cooldown
        let result = engine.check_trade_safety(request.clone()).await;
        assert!(result.is_ok());
        let check = result.unwrap();
        assert!(!check.allowed);
        assert!(check.cooldown_status.is_some());
    }

    #[tokio::test]
    async fn test_safety_engine_high_risk_blocking() {
        let policy = SafetyPolicy::default();
        let mut engine = SafetyEngine::new(policy, 30);

        let request = SafetyCheckRequest {
            wallet_address: "wallet1".to_string(),
            input_amount: 100.0,
            input_mint: "SOL".to_string(),
            output_mint: "SCAM".to_string(),
            input_symbol: "SOL".to_string(),
            output_symbol: "SCAM".to_string(),
            amount_usd: 5000.0,
            slippage_bps: 50,
            price_impact_percent: 1.5,
            security_score: Some(25.0), // High risk token
        };

        let result = engine.check_trade_safety(request).await;
        assert!(result.is_ok());
        let check = result.unwrap();
        assert!(!check.allowed);
        assert!(!check.policy_result.violations.is_empty());
        assert!(check
            .policy_result
            .violations
            .iter()
            .any(|v| v.rule == "high_risk_token"));
    }

    #[tokio::test]
    async fn test_safety_engine_insurance_requirement() {
        let mut policy = SafetyPolicy::default();
        policy.require_insurance_above_usd = Some(40000.0);
        let mut engine = SafetyEngine::new(policy, 30);

        let request = SafetyCheckRequest {
            wallet_address: "wallet1".to_string(),
            input_amount: 1000.0,
            input_mint: "SOL".to_string(),
            output_mint: "USDC".to_string(),
            input_symbol: "SOL".to_string(),
            output_symbol: "USDC".to_string(),
            amount_usd: 50000.0,
            slippage_bps: 50,
            price_impact_percent: 1.5,
            security_score: Some(85.0),
        };

        let result = engine.check_trade_safety(request).await;
        assert!(result.is_ok());
        let check = result.unwrap();
        assert!(check.insurance_required);
        assert!(check.insurance_recommendation.is_some());
    }

    #[tokio::test]
    async fn test_safety_engine_disabled() {
        let mut policy = SafetyPolicy::default();
        policy.enabled = false;
        let mut engine = SafetyEngine::new(policy, 30);

        let request = SafetyCheckRequest {
            wallet_address: "wallet1".to_string(),
            input_amount: 100.0,
            input_mint: "SOL".to_string(),
            output_mint: "USDC".to_string(),
            input_symbol: "SOL".to_string(),
            output_symbol: "USDC".to_string(),
            amount_usd: 15000.0, // Exceeds default limit
            slippage_bps: 50,
            price_impact_percent: 1.5,
            security_score: Some(85.0),
        };

        let result = engine.check_trade_safety(request).await;
        assert!(result.is_ok());
        let check = result.unwrap();
        assert!(check.allowed); // Should be allowed since safety is disabled
    }
}
