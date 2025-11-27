use app_lib::market::holders::HolderAnalyzer;

async fn setup_test_pool() -> Result<sqlx::Pool<sqlx::Sqlite>, sqlx::Error> {
    let pool = sqlx::SqlitePool::connect(":memory:").await?;

    // Initialize tables required by HolderAnalyzer
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS holders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            token_address TEXT NOT NULL,
            holder_address TEXT NOT NULL,
            balance REAL NOT NULL,
            percentage REAL NOT NULL,
            is_known_wallet INTEGER NOT NULL DEFAULT 0,
            wallet_label TEXT,
            updated_at TEXT NOT NULL,
            UNIQUE(token_address, holder_address)
        )
        "#,
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}

#[tokio::test]
async fn test_gini_coefficient_calculation() {
    let pool = setup_test_pool().await.unwrap();
    let analyzer = HolderAnalyzer::with_pool(pool);

    // Equal distribution should have a low Gini coefficient
    let equal_balances = vec![100.0, 100.0, 100.0, 100.0, 100.0];
    let gini = analyzer.calculate_gini_coefficient(&equal_balances);
    assert!(gini < 0.1, "equal distribution should be near zero: {gini}");

    // Highly concentrated distribution should have a higher score
    let concentrated_balances = vec![10_000.0, 10.0, 10.0, 10.0, 10.0];
    let gini = analyzer.calculate_gini_coefficient(&concentrated_balances);
    assert!(
        gini > 0.5,
        "concentrated distribution should be high: {gini}"
    );

    // Empty balances should be zero
    let empty_balances: Vec<f64> = vec![];
    let gini = analyzer.calculate_gini_coefficient(&empty_balances);
    assert_eq!(gini, 0.0);
}

#[test]
fn test_holder_percentage_calculation() {
    let holders = vec![("addr1", 1000.0), ("addr2", 500.0), ("addr3", 500.0)];

    let total: f64 = holders.iter().map(|(_, bal)| bal).sum();
    assert_eq!(total, 2000.0);

    let percentages: Vec<f64> = holders
        .iter()
        .map(|(_, bal)| (bal / total) * 100.0)
        .collect();

    assert_eq!(percentages[0], 50.0);
    assert_eq!(percentages[1], 25.0);
    assert_eq!(percentages[2], 25.0);
}

#[test]
fn test_known_wallet_identification() {
    let known_wallets = vec![
        ("DeFi Protocol Treasury", "addr1"),
        ("Team Vesting", "addr2"),
    ];

    let address = "addr1";
    let is_known = known_wallets.iter().any(|(_, addr)| *addr == address);
    assert!(is_known);

    let unknown_address = "addr999";
    let is_unknown = known_wallets
        .iter()
        .any(|(_, addr)| *addr == unknown_address);
    assert!(!is_unknown);
}
