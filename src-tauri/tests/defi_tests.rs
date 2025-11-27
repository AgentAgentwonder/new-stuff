use app_lib::defi::*;
use tokio;

#[tokio::test]
async fn solend_reserves_are_available() {
    let reserves = get_solend_reserves().await.expect("should fetch reserves");
    assert!(!reserves.is_empty(), "no solend reserves returned");
}

#[tokio::test]
async fn marginfi_positions_format() {
    let positions = get_marginfi_positions("test-wallet".to_string())
        .await
        .expect("should fetch positions");
    assert!(
        !positions.is_empty(),
        "marginfi positions should not be empty"
    );
    for position in positions {
        assert!(
            position.value_usd >= 0.0,
            "position value must be non-negative"
        );
    }
}

#[tokio::test]
async fn kamino_vaults_present() {
    let vaults = get_kamino_vaults()
        .await
        .expect("should fetch kamino vaults");
    assert!(!vaults.is_empty());
}

#[tokio::test]
async fn position_manager_snapshot() {
    let snapshot = get_defi_snapshot("test-wallet".to_string())
        .await
        .expect("should fetch snapshot");
    assert_eq!(snapshot.wallet, "test-wallet");
    assert!(snapshot.summary.total_value_usd >= 0.0);
}
