mod academy;
mod ai;
mod ai_legacy;
mod ai_chat;
mod alerts;
mod anomalies;
mod api;
mod api_analytics;
mod api_config;
mod auth;
mod auto_start;
mod backup;
mod bots;
mod bridges;
mod cache_commands;
mod chains;
mod chart_stream;
mod collab;
mod compiler;
mod config;
mod core;
mod data;
mod defi;
mod dev_tools;
mod diagnostics;
mod drawings;
mod errors;
mod features;
mod fixer;
mod governance;
mod indicators;
mod insiders;
mod journal;
mod launchpad;
mod logger;
mod market;
mod mobile;
mod monitor;
mod notifications;
mod portfolio;
mod position_manager;
mod auto_compound;
mod yield_farming;
mod recovery;
mod security;
mod sentiment;
mod social;
mod stocks;
mod stream_commands;
mod tax;
mod token_flow;
mod trading;
mod tray;
mod ui;
mod updater;
mod utils;
mod voice;
mod wallet;
mod webhooks;
mod websocket;
mod windowing;

pub use academy::*;
pub use governance::*;
pub use journal::*;
mod p2p;
pub use ai::*;
pub use ai_legacy::*;
pub use ai_chat::*;
pub use alerts::*;
pub use anomalies::*;
pub use api::*;
pub use api_analytics::*;
pub use api_config::*;
pub use auth::*;
pub use auto_start::*;
pub use backup::*;
pub use bots::*;
pub use bridges::*;
pub use chains::*;
pub use chart_stream::*;
pub use collab::*;
pub use compiler::*;
pub use config::*;
pub use core::*;
pub use data::*;
pub use defi::*;
pub use dev_tools::*;
pub use drawings::*;
pub use errors::*;
pub use features::*;
pub use fixer::*;
pub use indicators::*;
pub use insiders::*;
pub use launchpad::*;
pub use logger::*;
pub use market::*;
pub use mobile::*;
pub use monitor::*;
pub use notifications::*;
pub use p2p::*;
pub use portfolio::*;
pub use position_manager::*;
pub use auto_compound::*;
pub use yield_farming::*;
pub use recovery::*;
pub use sentiment::*;
pub use social::*;
pub use stocks::*;
pub use tax::*;
pub use token_flow::*;
pub use trading::*;
pub use tray::*;
pub use ui::theme_engine::*;
pub use updater::*;
pub use voice::*;
pub use wallet::hardware_wallet::*;
pub use wallet::ledger::*;
pub use wallet::multi_wallet::*;
pub use wallet::operations::*;
pub use wallet::phantom::*;
pub use webhooks::*;

pub use wallet::multisig::*;
pub use wallet::performance::*;
pub use windowing::*;

use ai_legacy::launch_predictor::{
    add_launch_training_data, extract_token_features, get_launch_bias_report,
    get_launch_prediction_history, load_latest_launch_model, predict_launch_success,
    retrain_launch_model, LaunchPredictor, SharedLaunchPredictor,
};
use alerts::{AlertManager, SharedAlertManager, SharedSmartAlertManager, SmartAlertManager};
use api::{ApiHealthMonitor, SharedApiHealthMonitor};
use auth::session_manager::SessionManager;
use auth::two_factor::TwoFactorManager;
use auto_start::{AutoStartManager, SharedAutoStartManager};
use bridges::{BridgeManager, SharedBridgeManager};
use chains::{ChainManager, SharedChainManager};
use chrono::{Timelike, Utc};
use collab::state::CollabState;
use config::settings_manager::{SettingsManager, SharedSettingsManager};
use core::cache_manager::{CacheType, SharedCacheManager};
use data::event_store::{EventStore, SharedEventStore};
use data::historical::{HistoricalReplayManager, SharedHistoricalReplayManager};
use drawings::{DrawingManager, SharedDrawingManager};
use governance::commands::*;
use indicators::{IndicatorManager, SharedIndicatorManager};
use journal::{JournalDatabase, SharedJournalDatabase};
use market::{HolderAnalyzer, SharedHolderAnalyzer};
use mobile::{
    MobileAuthManager, MobileSyncManager, MobileTradeEngine, PushNotificationManager,
    SharedMobileAuthManager, SharedMobileSyncManager, SharedPushNotificationManager, WidgetManager,
};
use notifications::router::{NotificationRouter, SharedNotificationRouter};
use p2p::init_p2p_system;
use portfolio::{
    AIPortfolioAdvisor, SharedAIPortfolioAdvisor, SharedWatchlistManager, WatchlistManager,
};
use security::activity_log::ActivityLogger;
use security::audit::AuditCache;
use security::keystore::Keystore;
use security::reputation::{ReputationEngine, SharedReputationEngine};
use social::service::{SocialDataService, SharedSocialDataService};
use std::error::Error;
use std::sync::Arc;
use stream_commands::*;
use tauri::Manager;
use tokio::sync::RwLock;
use tray::{attach_window_listeners, SharedTrayManager, TrayManager};
use voice::commands::{SharedVoiceState, VoiceState};
use wallet::hardware_wallet::HardwareWalletState;
use wallet::ledger::LedgerState;
use wallet::multi_wallet::MultiWalletManager;
use wallet::multisig::{MultisigDatabase, SharedMultisigDatabase};
use wallet::operations::WalletOperationsManager;
use wallet::performance::{PerformanceDatabase, SharedPerformanceDatabase};
use wallet::phantom::{hydrate_wallet_state, WalletState};
use webhooks::{SharedWebhookManager, WebhookManager};
use updater::{SharedUpdaterState, UpdaterState};

macro_rules! startup_log {
    ($($arg:tt)*) => {{
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3fZ");
        eprintln!("[startup][{}] {}", now, format_args!($($arg)*));
    }};
}

macro_rules! startup_error {
    ($($arg:tt)*) => {{
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3fZ");
        eprintln!("[startup][{}][ERROR] {}", now, format_args!($($arg)*));
    }};
}

macro_rules! manage_state {
    ($app:expr, $value:expr, $name:expr) => {{
        $app.manage($value);
        startup_log!("Managed state: {}", $name);
    }};
}

async fn warm_cache_on_startup(
    _app_handle: tauri::AppHandle,
    cache_manager: SharedCacheManager,
) -> Result<(), String> {
    use serde_json::json;

    // Define top tokens to warm
    let top_tokens = vec![
        "So11111111111111111111111111111111111111112",  // SOL
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
        "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", // USDT
        "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263", // BONK
        "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN",  // JUP
        "7vfCXTUXx5WJV5JADk17DUJ4ksgau7utNKj4b963voxs", // ETH
        "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",  // mSOL
        "7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARj", // stSOL
        "orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE",  // ORCA
        "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R", // RAY
    ];

    let manager = cache_manager.read().await;

    // Preload frequently accessed entries from disk cache first
    let warmed_from_disk = manager.populate_from_disk(64).await;
    tracing::info!(
        preloaded_entries = warmed_from_disk,
        "cache warmup from disk"
    );

    // Warm cache with top tokens
    let keys: Vec<String> = top_tokens
        .iter()
        .map(|addr| format!("token_price_{}", addr))
        .collect();

    let _ = manager
        .warm_cache(keys, |key| async move {
            // Mock data - in real implementation would fetch from API
            let data = json!({
                "price": 100.0,
                "change24h": 5.0,
                "volume": 1000000.0,
            });
            Ok((data, CacheType::TokenPrice))
        })
        .await;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    startup_log!("run() invoked");
    let builder = tauri::Builder::default();
    startup_log!("Base Tauri builder created");

    let builder =
        builder.plugin(tauri_plugin_global_shortcut::Builder::new().build());
    startup_log!("Global shortcut plugin registered");

    let builder = builder.plugin(tauri_plugin_notification::init());
    startup_log!("Notification plugin registered");

    let builder = builder.manage(WalletState::new());
    startup_log!("Wallet state registered");

    let builder = builder.manage(HardwareWalletState::new());
    startup_log!("Hardware wallet state registered");

    let builder = builder.manage(LedgerState::new());
    startup_log!("Ledger state registered");

    let builder = builder.setup(|app| {
            startup_log!("setup() closure entered");
            if let Err(e) = hydrate_wallet_state(&app.handle()) {
                startup_error!("Failed to hydrate wallet state: {}", e);
            }

            startup_log!("Initializing keystore");
            let keystore = Keystore::initialize(&app.handle()).map_err(|e| {
                startup_error!("Failed to initialize keystore: {}", e);
                Box::new(e) as Box<dyn Error>
            })?;
            startup_log!("Keystore initialized");

            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| Box::new(e) as Box<dyn Error>)?;
            std::fs::create_dir_all(&app_data_dir)
                .map_err(|e| Box::new(e) as Box<dyn Error>)?;
            eprintln!("App data directory created/verified: {:?}", app_data_dir);

            let tax_engine = tax::initialize_tax_engine(&keystore);
            startup_log!("Tax engine initialized");

            let audit_cache = AuditCache::new();
            manage_state!(app, audit_cache, "AuditCache");

            let session_manager = SessionManager::new();
            startup_log!("Session manager created");
            if let Err(e) = session_manager.hydrate(&keystore) {
                startup_error!("Failed to hydrate session manager: {}", e);
            } else {
                startup_log!("Session manager hydrated");
            }

            let two_factor_manager = TwoFactorManager::new();
            startup_log!("Two-factor manager created");
            if let Err(e) = two_factor_manager.hydrate(&keystore) {
                startup_error!("Failed to hydrate 2FA manager: {}", e);
            } else {
                startup_log!("2FA manager hydrated");
            }

            let ws_manager = core::websocket_manager::WebSocketManager::new(app.handle().clone());
            startup_log!("WebSocket manager created");

            startup_log!("Initializing multi-wallet manager");
            let multi_wallet_manager = MultiWalletManager::initialize(&keystore).map_err(|e| {
                startup_error!("Failed to initialize multi-wallet manager: {}", e);
                Box::new(e) as Box<dyn Error>
            })?;
            startup_log!("Multi-wallet manager initialized");

            startup_log!("Initializing wallet operations manager");
            let wallet_operations_manager = WalletOperationsManager::initialize(&keystore)
                .map_err(|e| {
                    startup_error!("Failed to initialize wallet operations manager: {}", e);
                    Box::new(e) as Box<dyn Error>
                })?;
            startup_log!("Wallet operations manager initialized");

            startup_log!("Initializing activity logger");
            let activity_logger =
                tauri::async_runtime::block_on(async { ActivityLogger::new(&app.handle()).await })
                    .map_err(|e| {
                        startup_error!("Failed to initialize activity logger: {}", e);
                        Box::new(e) as Box<dyn Error>
                    })?;
            startup_log!("Activity logger initialized");

            let cleanup_logger = activity_logger.clone();

            // Initialize reputation engine
            startup_log!("Initializing reputation engine");
            let reputation_engine = tauri::async_runtime::block_on(async {
                ReputationEngine::new(&app.handle()).await
            })
            .map_err(|e| {
                startup_error!("Failed to initialize reputation engine: {}", e);
                Box::new(e) as Box<dyn Error>
            })?;
            startup_log!("Reputation engine initialized");

            let shared_reputation_engine: SharedReputationEngine =
                Arc::new(RwLock::new(reputation_engine));
            manage_state!(app, shared_reputation_engine.clone(), "SharedReputationEngine");

            // Initialize P2P system
            startup_log!("Initializing P2P system");
            let p2p_db =
                tauri::async_runtime::block_on(async { init_p2p_system(&app.handle()).await })
                    .map_err(|e| {
                        startup_error!("Failed to initialize P2P system: {}", e);
                        e
                    })?;
            startup_log!("P2P system initialized");
            manage_state!(app, p2p_db.clone(), "P2PDatabase");

            // Initialize academy engine
            startup_log!("Initializing academy engine");
            let academy_engine = tauri::async_runtime::block_on(async {
                academy::AcademyEngine::new(&app.handle()).await
            })
            .map_err(|e| {
                startup_error!("Failed to initialize academy engine: {}", e);
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                )) as Box<dyn Error>
            })?;
            startup_log!("Academy engine initialized");

            let shared_academy_engine: academy::SharedAcademyEngine =
                Arc::new(RwLock::new(academy_engine));
            manage_state!(app, shared_academy_engine.clone(), "SharedAcademyEngine");

            // Initialize API config manager
            let api_config_manager = api_config::ApiConfigManager::new();
            startup_log!("API config manager created");
            if let Err(e) = api_config_manager.initialize(&keystore) {
                startup_error!("Failed to initialize API config manager: {}", e);
            } else {
                startup_log!("API config manager initialized");
            }

            // Initialize API health monitor
            startup_log!("Initializing API health monitor");
            let api_health_monitor = tauri::async_runtime::block_on(async {
                ApiHealthMonitor::new(&app.handle()).await
            })
            .map_err(|e| {
                startup_error!("Failed to initialize API health monitor: {}", e);
                Box::new(e) as Box<dyn Error>
            })?;
            startup_log!("API health monitor initialized");

            let api_health_state: SharedApiHealthMonitor =
                Arc::new(RwLock::new(api_health_monitor));

            manage_state!(app, multi_wallet_manager, "MultiWalletManager");
            manage_state!(app, wallet_operations_manager, "WalletOperationsManager");
            manage_state!(app, session_manager, "SessionManager");
            manage_state!(app, two_factor_manager, "TwoFactorManager");
            manage_state!(app, ws_manager, "WebSocketManager");
            manage_state!(app, activity_logger, "ActivityLogger");
            manage_state!(app, api_config_manager, "ApiConfigManager");
            manage_state!(app, api_health_state.clone(), "ApiHealthMonitor");

            startup_log!("Creating chain manager");
            let chain_manager: SharedChainManager = Arc::new(RwLock::new(ChainManager::new()));
            manage_state!(app, chain_manager.clone(), "ChainManager");

            startup_log!("Creating bridge manager");
            let bridge_manager: SharedBridgeManager = Arc::new(RwLock::new(BridgeManager::new()));
            manage_state!(app, bridge_manager.clone(), "BridgeManager");

            startup_log!("Initializing API usage tracker");
            let usage_tracker =
                api_analytics::initialize_usage_tracker(&app.handle()).map_err(|e| {
                    startup_error!("Failed to initialize API usage tracker: {}", e);
                    Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.clone())) as Box<dyn Error>
                })?;
            startup_log!("API usage tracker initialized");
            manage_state!(app, usage_tracker, "ApiUsageTracker");

            // Initialize universal settings manager
            startup_log!("Initializing settings manager");
            let settings_manager = SettingsManager::new(&app.handle()).map_err(|e| {
                startup_error!("Failed to initialize settings manager: {}", e);
                Box::new(e) as Box<dyn Error>
            })?;
            startup_log!("Settings manager initialized");
            let settings_state: SharedSettingsManager = Arc::new(RwLock::new(settings_manager));
            manage_state!(app, settings_state.clone(), "SettingsManager");

            // Initialize launchpad state
            let rpc_url = "https://api.mainnet-beta.solana.com".to_string();
            startup_log!("Creating launchpad state");
            let launchpad_state = launchpad::commands::create_launchpad_state(rpc_url);
            manage_state!(app, launchpad_state, "LaunchpadState");

            // Initialize collaborative rooms state
            startup_log!("Initializing collaborative rooms state");
            let collab_websocket = collab::websocket::CollabWebSocketManager::new(app.handle().clone());
            let collab_state = CollabState::new(collab_websocket);
            manage_state!(app, collab_state, "CollabState");

            startup_log!("Spawning activity log cleanup task");
            tauri::async_runtime::spawn(async move {
                use tokio::time::{sleep, Duration};

                if let Err(err) = cleanup_logger.cleanup_old_logs(None).await {
                    startup_error!("Failed to run initial activity log cleanup: {}", err);
                }

                loop {
                    sleep(Duration::from_secs(24 * 60 * 60)).await;
                    if let Err(err) = cleanup_logger.cleanup_old_logs(None).await {
                        startup_error!("Failed to run scheduled activity log cleanup: {}", err);
                    }
                }
            });

            startup_log!("Registering trading states");
            trading::register_trading_state(&app.handle());
            trading::register_paper_trading_state(&app.handle());
            trading::register_auto_trading_state(&app);
            trading::register_optimizer_state(&app);
            startup_log!("Trading states registered");

            // Initialize safety engine
            let default_policy = trading::safety::policy::SafetyPolicy::default();
            let safety_engine = trading::SafetyEngine::new(default_policy, 30);
            startup_log!("Safety engine created");
            let safety_state: trading::SharedSafetyEngine = Arc::new(RwLock::new(safety_engine));
            manage_state!(app, safety_state.clone(), "SafetyEngine");

            // Initialize contract risk service
            startup_log!("Initializing contract risk service");
            let contract_risk_service = tauri::async_runtime::block_on(async {
                trading::contract_risk::ContractVerificationService::new(&app.handle()).await
            })
            .map_err(|e| {
                startup_error!("Failed to initialize contract risk service: {}", e);
                Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
                    as Box<dyn Error>
            })?;
            startup_log!("Contract risk service initialized");
            let contract_risk_state: trading::contract_risk::SharedContractRiskService =
                Arc::new(RwLock::new(contract_risk_service));
            manage_state!(app, contract_risk_state.clone(), "ContractRiskService");

            // Initialize wallet monitor
            let monitor_handle = app.handle().clone();
            startup_log!("Spawning wallet monitor task");
            tauri::async_runtime::spawn(async move {
                if let Err(err) = insiders::init_wallet_monitor(&monitor_handle).await {
                    startup_error!("Failed to initialize wallet monitor: {}", err);
                }
            });

            // Initialize multisig database
            let mut multisig_db_path = app
                .path()
                .app_data_dir()
                .map_err(|e| Box::new(e) as Box<dyn Error>)?;

            std::fs::create_dir_all(&multisig_db_path)
                .map_err(|e| Box::new(e) as Box<dyn Error>)?;

            multisig_db_path.push("multisig.db");

            startup_log!("Initializing multisig database");
            let multisig_db = tauri::async_runtime::block_on(MultisigDatabase::new(
                multisig_db_path,
            ))
            .map_err(|e| {
                startup_error!("Failed to initialize multisig database: {}", e);
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                )) as Box<dyn Error>
            })?;
            startup_log!("Multisig database initialized");

            let multisig_state: SharedMultisigDatabase = Arc::new(RwLock::new(multisig_db));
            manage_state!(app, multisig_state.clone(), "MultisigDatabase");

            // Initialize performance database
            let mut performance_db_path = app
                .path()
                .app_data_dir()
                .map_err(|e| Box::new(e) as Box<dyn Error>)?;

            std::fs::create_dir_all(&performance_db_path)
                .map_err(|e| Box::new(e) as Box<dyn Error>)?;

            performance_db_path.push("performance.db");

            startup_log!("Initializing performance database");
            let performance_db =
                tauri::async_runtime::block_on(PerformanceDatabase::new(performance_db_path))
                    .map_err(|e| {
                        startup_error!("Failed to initialize performance database: {}", e);
                        Box::new(e) as Box<dyn Error>
                    })?;
            startup_log!("Performance database initialized");

            let performance_state: SharedPerformanceDatabase =
                Arc::new(RwLock::new(performance_db));
            manage_state!(app, performance_state.clone(), "PerformanceDatabase");

            // Initialize journal database
            let mut journal_db_path = app
                .path()
                .app_data_dir()
                .map_err(|e| Box::new(e) as Box<dyn Error>)?;

            journal_db_path.push("journal.db");

            startup_log!("Initializing journal database");
            let journal_db = tauri::async_runtime::block_on(JournalDatabase::new(journal_db_path))
                .map_err(|e| {
                    startup_error!("Failed to initialize journal database: {}", e);
                    Box::new(e) as Box<dyn Error>
                })?;
            startup_log!("Journal database initialized");

            let journal_state: SharedJournalDatabase = Arc::new(RwLock::new(journal_db));
            manage_state!(app, journal_state.clone(), "JournalDatabase");

            // Initialize backup service and scheduler
            startup_log!("Initializing backup service");
            let backup_service = backup::service::BackupService::new(&app.handle());
            let backup_service_state: backup::service::SharedBackupService =
                Arc::new(RwLock::new(backup_service));
            manage_state!(app, backup_service_state.clone(), "BackupService");

            startup_log!("Initializing backup scheduler");
            let backup_scheduler =
                backup::scheduler::BackupScheduler::new(&app.handle()).map_err(|e| {
                    startup_error!("Failed to initialize backup scheduler: {}", e);
                    Box::new(e) as Box<dyn Error>
                })?;
            startup_log!("Backup scheduler initialized");
            let backup_scheduler_state: backup::scheduler::SharedBackupScheduler =
                Arc::new(RwLock::new(backup_scheduler));
            manage_state!(app, backup_scheduler_state.clone(), "BackupScheduler");

            let automation_handle = app.handle().clone();
            startup_log!("Spawning automation tasks");
            tauri::async_runtime::spawn(async move {
                if let Err(err) = bots::init_dca(&automation_handle).await {
                    startup_error!("Failed to initialize DCA bots: {}", err);
                }
                if let Err(err) = trading::init_copy_trading(&automation_handle).await {
                    startup_error!("Failed to initialize copy trading: {}", err);
                }
            });

            let portfolio_data = portfolio::PortfolioDataState::new();
            let rebalancer_state = portfolio::RebalancerState::default();
            let tax_lots_state = portfolio::TaxLotsState::default();

            startup_log!("Registering portfolio state containers");
            manage_state!(
                app,
                std::sync::Mutex::new(portfolio_data),
                "PortfolioDataState"
            );
            manage_state!(
                app,
                std::sync::Mutex::new(rebalancer_state),
                "RebalancerState"
            );
            manage_state!(app, std::sync::Mutex::new(tax_lots_state), "TaxLotsState");
            manage_state!(app, tax_engine.clone(), "TaxEngine");

            // Initialize new coins scanner
            startup_log!("Initializing new coins scanner");
            let new_coins_scanner = tauri::async_runtime::block_on(async {
                market::NewCoinsScanner::new(&app.handle()).await
            })
            .map_err(|e| {
                startup_error!("Failed to initialize new coins scanner: {}", e);
                Box::new(e) as Box<dyn Error>
            })?;
            startup_log!("New coins scanner initialized");

            let scanner_state: market::SharedNewCoinsScanner =
                Arc::new(RwLock::new(new_coins_scanner));
            manage_state!(app, scanner_state.clone(), "NewCoinsScanner");

            // Start background scanning task
            let scanner_for_loop = scanner_state.clone();
            market::start_new_coins_scanner(scanner_for_loop);

            startup_log!("Creating top coins cache");
            let top_coins_cache: market::SharedTopCoinsCache =
                Arc::new(RwLock::new(market::TopCoinsCache::new()));
            manage_state!(app, top_coins_cache.clone(), "TopCoinsCache");

            // Initialize watchlist manager
            startup_log!("Initializing watchlist manager");
            let watchlist_manager = tauri::async_runtime::block_on(async {
                WatchlistManager::new(&app.handle()).await
            })
            .map_err(|e| {
                startup_error!("Failed to initialize watchlist manager: {}", e);
                Box::new(e) as Box<dyn Error>
            })?;
            startup_log!("Watchlist manager initialized");

            let watchlist_state: SharedWatchlistManager = Arc::new(RwLock::new(watchlist_manager));
            manage_state!(app, watchlist_state.clone(), "WatchlistManager");

            let token_flow_state = token_flow::commands::create_token_flow_state();
            manage_state!(app, token_flow_state.clone(), "TokenFlowState");

            // Initialize alert manager
            startup_log!("Initializing alert manager");
            let alert_manager =
                tauri::async_runtime::block_on(async { AlertManager::new(&app.handle()).await })
                    .map_err(|e| {
                        startup_error!("Failed to initialize alert manager: {}", e);
                        Box::new(e) as Box<dyn Error>
                    })?;
            startup_log!("Alert manager initialized");

            let alert_state: SharedAlertManager = Arc::new(RwLock::new(alert_manager));
            manage_state!(app, alert_state.clone(), "AlertManager");

            startup_log!("Initializing smart alert manager");
            let smart_alert_manager = tauri::async_runtime::block_on(async {
                SmartAlertManager::new(&app.handle()).await
            })
            .map_err(|e| {
                startup_error!("Failed to initialize smart alert manager: {}", e);
                Box::new(e) as Box<dyn Error>
            })?;
            startup_log!("Smart alert manager initialized");

            let smart_alert_state: SharedSmartAlertManager =
                Arc::new(RwLock::new(smart_alert_manager));
            manage_state!(app, smart_alert_state.clone(), "SmartAlertManager");

            // Start alert cooldown reset task
            let alert_reset_state = alert_state.clone();
            startup_log!("Spawning alert cooldown reset task");
            tauri::async_runtime::spawn(async move {
                use tokio::time::{sleep, Duration};
                loop {
                    sleep(Duration::from_secs(60)).await; // Check every minute
                    let mgr = alert_reset_state.read().await;
                    if let Err(err) = mgr.reset_cooldowns().await {
                        startup_error!("Failed to reset alert cooldowns: {}", err);
                    }
                }
            });

            // Initialize notification router
            startup_log!("Initializing notification router");
            let notification_router = tauri::async_runtime::block_on(async {
                NotificationRouter::new(&app.handle()).await
            })
            .map_err(|e| {
                startup_error!("Failed to initialize notification router: {}", e);
                Box::new(e) as Box<dyn Error>
            })?;
            startup_log!("Notification router initialized");

            let notification_state: SharedNotificationRouter =
                Arc::new(RwLock::new(notification_router));
            manage_state!(app, notification_state.clone(), "NotificationRouter");

            // Initialize indicator manager
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| Box::new(e) as Box<dyn Error>)?;

            startup_log!("Initializing indicator manager");
            let indicator_manager = IndicatorManager::new(app_data_dir.clone());
            let indicator_state: SharedIndicatorManager = Arc::new(RwLock::new(indicator_manager));
            manage_state!(app, indicator_state.clone(), "IndicatorManager");

            // Initialize drawing manager
            startup_log!("Initializing drawing manager");
            let drawing_manager = DrawingManager::new(app_data_dir.clone());
            let drawing_state: SharedDrawingManager = Arc::new(RwLock::new(drawing_manager));
            manage_state!(app, drawing_state.clone(), "DrawingManager");

            // Initialize webhook manager
            startup_log!("Initializing webhook manager");
            let webhook_manager =
                tauri::async_runtime::block_on(async { WebhookManager::new(&app.handle()).await })
                    .map_err(|e| {
                        startup_error!("Failed to initialize webhook manager: {}", e);
                        Box::new(e) as Box<dyn Error>
                    })?;
            startup_log!("Webhook manager initialized");

            let webhook_state: SharedWebhookManager = Arc::new(RwLock::new(webhook_manager));
            manage_state!(app, webhook_state.clone(), "WebhookManager");

            // Initialize cache manager
            startup_log!("Initializing cache manager");
            let cache_manager = core::cache_manager::CacheManager::new(100, 1000);
            let shared_cache_manager = Arc::new(RwLock::new(cache_manager));
            manage_state!(app, shared_cache_manager.clone(), "CacheManager");

            // Start background cache warming
            let app_handle = app.handle().clone();
            let cache_manager_handle = shared_cache_manager.clone();
            startup_log!("Spawning cache warmup task");
            tauri::async_runtime::spawn(async move {
                if let Err(err) = warm_cache_on_startup(app_handle.clone(), cache_manager_handle).await {
                    startup_error!("Failed to warm cache on startup: {}", err);
                }
            });

            // Initialize sentiment manager
            startup_log!("Initializing sentiment manager");
            let sentiment_manager = sentiment::SentimentManager::new();
            let sentiment_state: sentiment::SharedSentimentManager =
                Arc::new(RwLock::new(sentiment_manager));
            manage_state!(app, sentiment_state.clone(), "SentimentManager");

            // Initialize social data service
            startup_log!("Initializing social data service");
            let social_service = tauri::async_runtime::block_on(async {
                SocialDataService::new(&app.handle()).await
            })
            .map_err(|e| {
                startup_error!("Failed to initialize social data service: {}", e);
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                )) as Box<dyn Error>
            })?;
            startup_log!("Social data service initialized");
            let social_state: SharedSocialDataService = Arc::new(RwLock::new(social_service));
            manage_state!(app, social_state.clone(), "SocialDataService");

            // Initialize social analysis service
            let mut social_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| Box::new(e) as Box<dyn Error>)?;
            social_data_dir.push("social");
            std::fs::create_dir_all(&social_data_dir)
                .map_err(|e| Box::new(e) as Box<dyn Error>)?;

            startup_log!("Initializing social cache for analysis");
            let social_cache = tauri::async_runtime::block_on(async {
                social::SocialCache::new(social_data_dir).await
            })
            .map_err(|e| {
                startup_error!("Failed to initialize social cache for analysis: {}", e);
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                )) as Box<dyn Error>
            })?;
            startup_log!("Social cache initialized");

            let mut analysis_service = social::SocialAnalysisService::new(social_cache);
            startup_log!("Initializing social analysis service");
            tauri::async_runtime::block_on(async { analysis_service.initialize().await }).map_err(
                |e| {
                    startup_error!("Failed to initialize social analysis service: {}", e);
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        e.to_string(),
                    )) as Box<dyn Error>
                },
            )?;
            startup_log!("Social analysis service initialized");

            let analysis_state: social::SharedSocialAnalysisService =
                Arc::new(RwLock::new(analysis_service));
            manage_state!(app, analysis_state.clone(), "SocialAnalysisService");

            // Initialize anomaly detector
            startup_log!("Initializing anomaly detector");
            let anomaly_detector = anomalies::AnomalyDetector::new();
            let anomaly_state: anomalies::SharedAnomalyDetector =
                Arc::new(RwLock::new(anomaly_detector));
            manage_state!(app, anomaly_state.clone(), "AnomalyDetector");

            // Initialize event store
            let mut event_store_path = app
                .path()
                .app_data_dir()
                .map_err(|e| Box::new(e) as Box<dyn Error>)?;

            event_store_path.push("events.db");

            startup_log!("Initializing event store");
            let event_store = tauri::async_runtime::block_on(EventStore::new(event_store_path))
                .map_err(|e| {
                    startup_error!("Failed to initialize event store: {}", e);
                    Box::new(e) as Box<dyn Error>
                })?;
            startup_log!("Event store initialized");

            let shared_event_store: SharedEventStore = Arc::new(RwLock::new(event_store));
            manage_state!(app, shared_event_store.clone(), "EventStore");

            // Initialize compression manager
            let mut compression_db_path = app
                .path()
                .app_data_dir()
                .map_err(|e| Box::new(e) as Box<dyn Error>)?;

            compression_db_path.push("events.db");

            startup_log!("Initializing compression manager");
            let compression_manager =
                tauri::async_runtime::block_on(CompressionManager::new(compression_db_path))
                    .map_err(|e| {
                        startup_error!("Failed to initialize compression manager: {}", e);
                        Box::new(e) as Box<dyn Error>
                    })?;
            startup_log!("Compression manager initialized");

            let shared_compression_manager: SharedCompressionManager =
                Arc::new(RwLock::new(compression_manager));
            manage_state!(app, shared_compression_manager.clone(), "CompressionManager");

            // Initialize holder analyzer
            startup_log!("Initializing holder analyzer");
            let holder_analyzer =
                tauri::async_runtime::block_on(async { HolderAnalyzer::new(&app.handle()).await })
                    .map_err(|e| {
                        startup_error!("Failed to initialize holder analyzer: {}", e);
                        Box::new(e) as Box<dyn Error>
                    })?;
            startup_log!("Holder analyzer initialized");

            let shared_holder_analyzer: SharedHolderAnalyzer =
                Arc::new(RwLock::new(holder_analyzer));
            manage_state!(app, shared_holder_analyzer.clone(), "HolderAnalyzer");

            // Initialize stock cache state
            startup_log!("Initializing stock cache state");
            let stock_cache: stocks::SharedStockCache =
                Arc::new(RwLock::new(stocks::StockCache::default()));
            manage_state!(app, stock_cache.clone(), "StockCache");
            // Initialize risk analyzer
            startup_log!("Initializing risk analyzer");
            let risk_analyzer = tauri::async_runtime::block_on(async {
                ai_legacy::RiskAnalyzer::new(&app.handle()).await
            })
            .map_err(|e| {
                startup_error!("Failed to initialize risk analyzer: {}", e);
                Box::new(e) as Box<dyn Error>
            })?;
            startup_log!("Risk analyzer initialized");

            let shared_risk_analyzer: ai_legacy::SharedRiskAnalyzer = Arc::new(RwLock::new(risk_analyzer));
            manage_state!(app, shared_risk_analyzer.clone(), "RiskAnalyzer");

            // Initialize AI portfolio advisor
            startup_log!("Initializing AI portfolio advisor");
            let ai_advisor = tauri::async_runtime::block_on(async {
                AIPortfolioAdvisor::new(&app.handle()).await
            })
            .map_err(|e| {
                startup_error!("Failed to initialize AI portfolio advisor: {}", e);
                Box::new(e) as Box<dyn Error>
            })?;
            startup_log!("AI portfolio advisor initialized");

            let shared_ai_advisor: SharedAIPortfolioAdvisor = Arc::new(RwLock::new(ai_advisor));
            manage_state!(app, shared_ai_advisor.clone(), "AIPortfolioAdvisor");

            // Initialize AI Assistant
            startup_log!("Initializing AI assistant");
            let ai_assistant = tauri::async_runtime::block_on(async {
                ai_legacy::AIAssistant::new(&app.handle(), &keystore).await
            })
            .map_err(|e| {
                startup_error!("Failed to initialize AI assistant: {}", e);
                Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)) as Box<dyn Error>
            })?;
            startup_log!("AI assistant initialized");

            let shared_ai_assistant: ai_legacy::SharedAIAssistant = Arc::new(RwLock::new(ai_assistant));
            manage_state!(app, shared_ai_assistant.clone(), "AIAssistant");
            manage_state!(app, keystore, "Keystore");

            // Initialize launch predictor
            startup_log!("Initializing launch predictor");
            let launch_predictor =
                tauri::async_runtime::block_on(async { LaunchPredictor::new(&app.handle()).await })
                    .map_err(|e| {
                        startup_error!("Failed to initialize launch predictor: {}", e);
                        Box::new(e) as Box<dyn Error>
                    })?;
            startup_log!("Launch predictor initialized");

            let shared_launch_predictor: SharedLaunchPredictor =
                Arc::new(RwLock::new(launch_predictor));
            manage_state!(app, shared_launch_predictor.clone(), "LaunchPredictor");

            // Initialize updater state
            startup_log!("Initializing updater state");
            let updater_state = UpdaterState::new(&app.handle()).map_err(|e| {
                startup_error!("Failed to initialize updater state: {}", e);
                Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)) as Box<dyn Error>
            })?;
            startup_log!("Updater state initialized");

            let shared_updater_state: SharedUpdaterState = Arc::new(updater_state);
            manage_state!(app, shared_updater_state.clone(), "UpdaterState");

            // Initialize system tray manager
            startup_log!("Initializing system tray manager");
            let tray_manager = TrayManager::new();
            tray_manager.initialize(&app.handle());
            let shared_tray_manager: SharedTrayManager = Arc::new(tray_manager);
            manage_state!(app, shared_tray_manager.clone(), "TrayManager");

            // Initialize auto-start manager
            startup_log!("Preparing auto-start manager");
            let app_name = "Eclipse Market Pro";
            let app_path = std::env::current_exe()
                .ok()
                .and_then(|p| p.to_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "eclipse-market-pro".to_string());

            let auto_start_manager = AutoStartManager::new(app_name, &app_path).map_err(|e| {
                startup_error!("Failed to initialize auto-start manager: {}", e);
                Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)) as Box<dyn Error>
            })?;
            auto_start_manager.initialize(&app.handle());
            let shared_auto_start_manager: SharedAutoStartManager = Arc::new(auto_start_manager);
            manage_state!(app, shared_auto_start_manager.clone(), "AutoStartManager");

            // Initialize historical replay manager
            startup_log!("Initializing historical replay manager");
            let historical_replay_manager = tauri::async_runtime::block_on(async {
                HistoricalReplayManager::new(&app.handle(), None).await
            })
            .map_err(|e| {
                startup_error!("Failed to initialize historical replay manager: {}", e);
                Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)) as Box<dyn Error>
            })?;
            startup_log!("Historical replay manager initialized");

            let shared_historical_manager: SharedHistoricalReplayManager =
                Arc::new(RwLock::new(historical_replay_manager));
            manage_state!(app, shared_historical_manager.clone(), "HistoricalReplayManager");

            // Initialize voice state
            startup_log!("Initializing voice state");
            let voice_state = VoiceState::new();
            let shared_voice_state: SharedVoiceState = Arc::new(RwLock::new(voice_state));
            manage_state!(app, shared_voice_state.clone(), "VoiceState");

            // Initialize theme engine
            startup_log!("Initializing theme engine");
            let theme_engine =
                ui::theme_engine::ThemeEngine::initialize(&app.handle()).map_err(|e| {
                    startup_error!("Failed to initialize theme engine: {}", e);
                    Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)) as Box<dyn Error>
                })?;
            let shared_theme_engine: ui::theme_engine::SharedThemeEngine =
                Arc::new(std::sync::Mutex::new(theme_engine));
            manage_state!(app, shared_theme_engine.clone(), "ThemeEngine");

            // Attach tray window listeners
            if let Some(window) = app.get_webview_window("main") {
                attach_window_listeners(&window, shared_tray_manager.clone());
            }

            // Handle auto-start behavior
            let auto_settings = shared_auto_start_manager.get_settings();
            let launched_from_auto_start = std::env::args().any(|arg| arg == "--auto-start");
            if launched_from_auto_start {
                if auto_settings.start_minimized {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.hide();
                    }
                    shared_tray_manager.notify_minimized(&app.handle());
                } else if auto_settings.delay_seconds > 0 {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.hide();
                    }
                    let app_handle = app.handle().clone();
                    let delay = auto_settings.delay_seconds;
                    tauri::async_runtime::spawn(async move {
                        use tokio::time::{sleep, Duration};
                        sleep(Duration::from_secs(delay as u64)).await;
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    });
                }
            }

            // Start background compression job (runs daily at 3 AM)
            let compression_job = shared_compression_manager.clone();
            startup_log!("Spawning compression maintenance task");
            tauri::async_runtime::spawn(async move {
                use tokio::time::{sleep, Duration};

                loop {
                    let now = chrono::Utc::now();

                    // Calculate time until 3 AM
                    let mut next_run = match now.date_naive().and_hms_opt(3, 0, 0) {
                        Some(time) => time.and_utc(),
                        None => {
                            startup_error!("Failed to create 3 AM schedule time - using fallback");
                            now + chrono::Duration::hours(1) // Fallback: run in 1 hour
                        }
                    };

                    if now.hour() >= 3 {
                        next_run = next_run + chrono::Duration::days(1);
                    }

                    let duration_until_next = next_run.signed_duration_since(now);
                    let sleep_secs = duration_until_next.num_seconds().max(0) as u64;

                    sleep(Duration::from_secs(sleep_secs)).await;

                    // Run compression
                    let manager = compression_job.read().await;
                    let config = manager.get_config().await;

                    if config.enabled && config.auto_compress {
                        if let Err(err) = manager.compress_old_events().await {
                            startup_error!("Failed to compress old events: {}", err);
                        }
                        if let Err(err) = manager.compress_old_trades().await {
                            startup_error!("Failed to compress old trades: {}", err);
                        }
                        manager.cleanup_cache().await;
                    }
                }
            });

            // Initialize prediction market service
            startup_log!("Initializing prediction market service");
            let prediction_service = market::PredictionMarketService::new();
            let shared_prediction_service: market::SharedPredictionMarketService =
                Arc::new(RwLock::new(prediction_service));
            manage_state!(app, shared_prediction_service.clone(), "PredictionMarketService");

            // Initialize diagnostics engine
            startup_log!("Initializing diagnostics engine");
            let diagnostics_engine = diagnostics::tauri_commands::initialize_diagnostics_engine(
                &app.handle(),
            )
            .map_err(|e| {
                startup_error!("Failed to initialize diagnostics engine: {}", e);
                Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)) as Box<dyn Error>
            })?;
            startup_log!("Diagnostics engine initialized");
            manage_state!(app, diagnostics_engine.clone(), "DiagnosticsEngine");

            let diagnostics_state = diagnostics_engine.clone();
            startup_log!("Spawning diagnostics maintenance task");
            tauri::async_runtime::spawn(async move {
                use tokio::time::{sleep, Duration};
                loop {
                    {
                        let mut engine = diagnostics_state.write().await;
                        let _ = engine.run_full_diagnostics().await;
                    }
                    sleep(Duration::from_secs(60 * 60)).await;
                }
            });
            // Initialize dev tools
            startup_log!("Initializing comprehensive logger");
            let logger = logger::ComprehensiveLogger::new(&app.handle()).map_err(|e| {
                startup_error!("Failed to initialize logger: {}", e);
                Box::new(e) as Box<dyn Error>
            })?;
            startup_log!("Logger initialized");
            let shared_logger: logger::SharedLogger = Arc::new(logger);
            manage_state!(app, shared_logger.clone(), "Logger");

            startup_log!("Initializing crash reporter");
            let crash_reporter = errors::CrashReporter::new(&app.handle(), shared_logger.clone())
                .map_err(|e| {
                    startup_error!("Failed to initialize crash reporter: {}", e);
                    Box::new(e) as Box<dyn Error>
                })?;
            startup_log!("Crash reporter initialized");
            let shared_crash_reporter: errors::SharedCrashReporter = Arc::new(crash_reporter);
            manage_state!(app, shared_crash_reporter.clone(), "CrashReporter");

            let runtime_handler = errors::RuntimeHandler::new(shared_logger.clone());
            let shared_runtime_handler: errors::SharedRuntimeHandler = Arc::new(runtime_handler);
            manage_state!(app, shared_runtime_handler.clone(), "RuntimeHandler");

            let performance_monitor = monitor::PerformanceMonitor::new();
            let shared_performance_monitor: monitor::SharedPerformanceMonitor =
                Arc::new(performance_monitor);
            manage_state!(app, shared_performance_monitor.clone(), "PerformanceMonitor");
            let perf_monitor = shared_performance_monitor.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = perf_monitor.start().await {
                    eprintln!("Failed to start performance monitor: {e}");
                }
            });

            let auto_compiler = compiler::AutoCompiler::new();
            let shared_auto_compiler = Arc::new(auto_compiler);
            manage_state!(app, shared_auto_compiler.clone(), "AutoCompiler");

            let auto_fixer = fixer::AutoFixer::new(3);
            let shared_auto_fixer = Arc::new(auto_fixer);
            manage_state!(app, shared_auto_fixer.clone(), "AutoFixer");

            shared_logger.info("Dev tools initialized successfully", None);

            // Initialize mobile managers
            let mut mobile_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| Box::new(e) as Box<dyn Error>)?;
            mobile_data_dir.push("mobile");
            std::fs::create_dir_all(&mobile_data_dir)
                .map_err(|e| Box::new(e) as Box<dyn Error>)?;

            let mut mobile_auth_manager = MobileAuthManager::new(mobile_data_dir.clone());
            startup_log!("Loading mobile auth manager state");
            tauri::async_runtime::block_on(mobile_auth_manager.load()).map_err(|e| {
                startup_error!("Failed to load mobile auth manager: {}", e);
                Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)) as Box<dyn Error>
            })?;
            startup_log!("Mobile auth manager state loaded");
            let mobile_auth_state: SharedMobileAuthManager =
                Arc::new(RwLock::new(mobile_auth_manager));
            manage_state!(app, mobile_auth_state.clone(), "MobileAuthManager");

            startup_log!("Initializing mobile push notification manager");
            let push_notification_manager = PushNotificationManager::new(1000);
            let push_notification_state: SharedPushNotificationManager =
                Arc::new(RwLock::new(push_notification_manager));
            manage_state!(app, push_notification_state.clone(), "PushNotificationManager");

            startup_log!("Initializing mobile sync manager");
            let mobile_sync_manager = MobileSyncManager::new();
            let mobile_sync_state: SharedMobileSyncManager =
                Arc::new(RwLock::new(mobile_sync_manager));
            manage_state!(app, mobile_sync_state.clone(), "MobileSyncManager");

            startup_log!("Initializing mobile trade engine");
            let mobile_trade_engine = MobileTradeEngine::new();
            let mobile_trade_state: Arc<RwLock<MobileTradeEngine>> =
                Arc::new(RwLock::new(mobile_trade_engine));
            manage_state!(app, mobile_trade_state.clone(), "MobileTradeEngine");

            startup_log!("Initializing widget manager");
            let widget_manager = WidgetManager::new();
            let widget_state: Arc<RwLock<WidgetManager>> = Arc::new(RwLock::new(widget_manager));
            manage_state!(app, widget_state.clone(), "WidgetManager");

            // Initialize governance manager
            startup_log!("Initializing governance manager");
            let governance_manager = governance::GovernanceManager::new();
            let governance_state: governance::SharedGovernanceManager =
                Arc::new(RwLock::new(governance_manager));
            manage_state!(app, governance_state.clone(), "GovernanceManager");

            // Initialize feature flags database
            let mut features_db_path = app
                .path()
                .app_data_dir()
                .map_err(|e| Box::new(e) as Box<dyn Error>)?;

            features_db_path.push("features.db");

            startup_log!("Initializing feature flags database");
            let features_pool = match tauri::async_runtime::block_on(async {
                let pool = sqlx::SqlitePool::connect(&format!("sqlite:{}", features_db_path.display()))
                    .await?;
                sqlx::migrate!("./migrations").run(&pool).await?;
                Ok::<_, Box<dyn Error>>(pool)
            }) {
                Ok(pool) => pool,
                Err(e) => {
                    startup_error!("Failed to initialize features database at {:?}: {}", features_db_path, e);
                    startup_error!("Using in-memory features database for this session");
                    tauri::async_runtime::block_on(async {
                        sqlx::SqlitePool::connect("sqlite::memory:").await
                    }).map_err(|e| {
                        startup_error!("Failed to create fallback in-memory features pool: {}", e);
                        Box::new(e) as Box<dyn Error>
                    })?
                }
            };
            startup_log!("Features database initialized");

            let feature_flags = features::FeatureFlags::new(features_pool);
            manage_state!(app, feature_flags, "FeatureFlags");

            startup_log!("setup() closure completed successfully");

            Ok(())
        });
    startup_log!("Setup closure attached");

    let builder = builder.invoke_handler(tauri::generate_handler![
             // Wallet
             phantom_connect,
             phantom_disconnect,
             phantom_session,
             phantom_sign_message,
             phantom_sign_transaction,
             phantom_balance,
            list_hardware_wallets,
            connect_hardware_wallet,
            disconnect_hardware_wallet,
            get_hardware_wallet_address,
            sign_with_hardware_wallet,
            get_firmware_version,
            ledger_register_device,
            ledger_list_devices,
            ledger_get_device,
            ledger_connect_device,
            ledger_disconnect_device,
            ledger_update_device_address,
            ledger_validate_transaction,
            ledger_get_active_device,
            ledger_remove_device,
            ledger_clear_devices,
            // Multi-Wallet
            multi_wallet_add,
            multi_wallet_update,
            multi_wallet_remove,
            multi_wallet_set_active,
            multi_wallet_get_active,
            multi_wallet_list,
            multi_wallet_update_balance,
            multi_wallet_update_performance,
            multi_wallet_create_group,
            multi_wallet_update_group,
            multi_wallet_delete_group,
            multi_wallet_list_groups,
            multi_wallet_get_aggregated,
            // Wallet Operations
            wallet_get_token_balances,
            wallet_estimate_fee,
            wallet_send_transaction,
            wallet_generate_qr,
            wallet_generate_solana_pay_qr,
            address_book_add_contact,
            address_book_update_contact,
            address_book_delete_contact,
            address_book_list_contacts,
            address_book_search_contacts,
            address_book_export,
            address_book_import,
            swap_history_add_entry,
            swap_history_get_recent,
            wallet_get_bridge_providers,
            // Wallet Performance
            record_trade,
            calculate_wallet_performance,
            get_wallet_performance_data,
            get_performance_score_history,
            get_token_performance_breakdown,
            get_timing_analysis_data,
            get_best_worst_trades_data,
            get_benchmark_comparison_data,
            get_performance_alerts,
            // Multisig
            create_multisig_wallet,
            list_multisig_wallets,
            get_multisig_wallet,
            create_proposal,
            list_proposals,
            sign_proposal,
            execute_proposal,
            cancel_proposal,
            // Auth
            biometric_get_status,
            biometric_enroll,
            biometric_verify,
            biometric_disable,
            biometric_verify_fallback,
            connect_phantom,
            // Session Management
            // TODO: Re-enable when session commands are implemented
            // session_create,
            // session_renew,
            // session_end,
            // session_status,
            // session_verify,
            // session_update_activity,
            // session_configure_timeout,
            // 2FA
            // TODO: Re-enable when 2FA commands are implemented
            // two_factor_enroll,
            // two_factor_verify,
            // two_factor_disable,
            // two_factor_status,
            // two_factor_regenerate_backup_codes,
            // API Config
            save_api_key,
            remove_api_key,
            set_use_default_key,
            test_api_connection,
            get_api_status,
            rotate_api_key,
            check_rotation_reminders,
            export_api_keys,
            import_api_keys,
            // API Analytics
            record_api_usage,
            get_api_analytics,
            get_fair_use_status,
            // AI & Sentiment
            assess_risk,
            analyze_text_sentiment,
            get_token_sentiment,
            get_all_token_sentiments,
            ingest_social_data,
            get_sentiment_alerts,
            update_sentiment_alert_config,
            get_sentiment_alert_config,
            dismiss_sentiment_alert,
            fetch_social_mentions,
            get_token_risk_score,
            get_risk_history,
            get_latest_risk_score,
            // Social Data
            // TODO: Re-enable when social commands are implemented
            // social_fetch_reddit,
            // social_search_reddit_mentions,
            // social_fetch_twitter,
            // social_fetch_twitter_user,
            // social_get_cached_mentions,
            // social_get_mention_aggregates,
            // social_get_trend_snapshots,
            // social_create_trend_snapshot,
            // social_set_twitter_bearer_token,
            // social_cleanup_old_posts,
            // social_run_sentiment_analysis,
            // social_run_full_analysis_all,
            // social_get_sentiment_snapshot,
            // social_get_sentiment_snapshots,
            // social_get_trending_tokens,
            // social_get_token_trends,
            // social_get_influencer_scores,
            // social_get_fomo_fud,
            // Launch Predictor
            extract_token_features,
            predict_launch_success,
            get_launch_prediction_history,
            add_launch_training_data,
            retrain_launch_model,
            load_latest_launch_model,
            get_launch_bias_report,
            // AI Assistant
            ai_chat,
            ai_get_conversations,
            ai_delete_conversation,
            ai_get_usage_stats,
            ai_set_api_key,
            ai_is_configured,
            // Market Data
            get_coin_price,
            get_price_history,
            search_tokens,
            get_trending_coins,
            get_coin_sentiment,
            refresh_trending,
            // New Coins Scanner
            get_new_coins,
            get_coin_safety_report,
            scan_for_new_coins,
            // Top Coins
            get_top_coins,
            refresh_top_coins,
            // Portfolio & Analytics
            get_portfolio_metrics,
            get_positions,
            list_rebalance_profiles,
            save_rebalance_profile,
            delete_rebalance_profile,
            preview_rebalance,
            execute_rebalance,
            get_rebalance_history,
            check_rebalance_triggers,
            get_tax_lots,
            get_open_tax_lots,
            set_tax_lot_strategy,
            get_tax_lot_strategy,
            dispose_tax_lot,
            generate_tax_report,
            export_tax_report,
            get_tax_loss_harvesting_suggestions,
            get_tax_center_summary,
            update_tax_settings,
            export_tax_center_report,
            calculate_portfolio_analytics,
            get_concentration_alerts,
            get_sector_allocation,
            clear_portfolio_cache,
            watchlist_create,
            watchlist_list,
            watchlist_get,
            watchlist_update,
            watchlist_delete,
            watchlist_add_item,
            watchlist_remove_item,
            watchlist_reorder_items,
            watchlist_export,
            watchlist_import,
            // AI Portfolio Advisor
            save_risk_profile,
            get_risk_profile,
            generate_portfolio_recommendation,
            get_portfolio_recommendations,
            apply_portfolio_recommendation,
            track_recommendation_performance,
            generate_weekly_portfolio_update,
            get_weekly_portfolio_updates,
            get_performance_history,
            // Alerts & Notifications
            alert_create,
            alert_list,
            alert_get,
            alert_update,
            alert_delete,
            alert_test,
            alert_check_triggers,
            alert_reset_cooldowns,
            smart_alert_create_rule,
            smart_alert_update_rule,
            smart_alert_delete_rule,
            smart_alert_list_rules,
            smart_alert_get_rule,
            smart_alert_dry_run,
            smart_alert_execute,
            // Chat Integrations
            chat_integration_get_settings,
            chat_integration_save_settings,
            chat_integration_add_telegram,
            chat_integration_update_telegram,
            chat_integration_delete_telegram,
            chat_integration_add_slack,
            chat_integration_update_slack,
            chat_integration_delete_slack,
            chat_integration_add_discord,
            chat_integration_update_discord,
            chat_integration_delete_discord,
            chat_integration_test_telegram,
            chat_integration_test_slack,
            chat_integration_test_discord,
            chat_integration_get_delivery_logs,
            chat_integration_clear_delivery_logs,
            chat_integration_get_rate_limits,
            // Webhooks
            list_webhooks,
            get_webhook,
            create_webhook,
            update_webhook,
            delete_webhook,
            trigger_webhook,
            test_webhook,
            list_webhook_delivery_logs,
            // API Health
            get_api_health_dashboard,
            get_service_health_metrics,
            cleanup_health_records,
            // WebSocket Streams
            subscribe_price_stream,
            unsubscribe_price_stream,
            subscribe_wallet_stream,
            unsubscribe_wallet_stream,
            get_stream_status,
            reconnect_stream,
            // Chart Streams
            subscribe_chart_prices,
            unsubscribe_chart_prices,
            get_chart_subscriptions,
            // Jupiter v6 & execution safeguards
            jupiter_quote,
            jupiter_swap,
            get_network_congestion,
            get_priority_fee_estimates,
            submit_with_mev_protection,
            validate_trade_thresholds,
            // Trading & Orders
            trading_init,
            create_order,
            cancel_order,
            get_active_orders,
            get_order_history,
            get_order,
            acknowledge_order,
            update_order_prices,
            // Auto Trading Engine
            auto_trading_create_strategy,
            auto_trading_update_strategy,
            auto_trading_delete_strategy,
            auto_trading_start_strategy,
            auto_trading_stop_strategy,
            auto_trading_pause_strategy,
            auto_trading_activate_kill_switch,
            auto_trading_deactivate_kill_switch,
            auto_trading_get_strategies,
            auto_trading_get_strategy,
            auto_trading_get_executions,
            auto_trading_apply_parameters,
            // Backtesting & Optimization
            backtest_run,
            optimizer_start,
            optimizer_cancel,
            optimizer_get_runs,
            optimizer_get_run,
            // Paper Trading Simulation
            paper_trading_init,
            get_paper_account,
            reset_paper_account,
            execute_paper_trade,
            get_paper_positions,
            get_paper_trade_history,
            get_paper_performance,
            update_paper_position_prices,
            // DCA Bots
            dca_init,
            dca_create,
            dca_list,
            dca_get,
            dca_pause,
            dca_resume,
            dca_delete,
            dca_history,
            dca_performance,
            // Copy Trading
            copy_trading_init,
            copy_trading_create,
            copy_trading_list,
            copy_trading_get,
            copy_trading_pause,
            copy_trading_resume,
            copy_trading_delete,
            copy_trading_history,
            copy_trading_performance,
            copy_trading_process_activity,
            copy_trading_followed_wallets,
            // Wallet Monitor
            wallet_monitor_init,
            wallet_monitor_add_wallet,
            wallet_monitor_update_wallet,
            wallet_monitor_remove_wallet,
            wallet_monitor_list_wallets,
            wallet_monitor_get_activities,
            wallet_monitor_get_statistics,
            // Smart Money & Whale Alerts
            classify_smart_money_wallet,
            get_smart_money_wallets,
            get_smart_money_consensus,
            get_sentiment_comparison,
            get_alert_configs,
            update_alert_config,
            get_recent_whale_alerts,
            scan_wallets_for_smart_money,
            // Activity Logging
            security::activity_log::get_activity_logs,
            security::activity_log::export_activity_logs,
            security::activity_log::get_activity_stats,
            security::activity_log::check_suspicious_activity,
            security::activity_log::cleanup_activity_logs,
            security::activity_log::get_activity_retention,
            security::activity_log::set_activity_retention,
            // Smart Contract Security
            security::audit::scan_contract,
            security::audit::get_cached_audit,
            security::audit::clear_audit_cache,
            security::audit::check_risk_threshold,
            // Reputation System
            security::reputation::get_wallet_reputation,
            security::reputation::get_token_reputation,
            security::reputation::update_wallet_behavior,
            security::reputation::initialize_token_reputation,
            security::reputation::update_token_metrics,
            security::reputation::add_vouch,
            security::reputation::remove_vouch,
            security::reputation::get_vouches,
            security::reputation::add_to_blacklist,
            security::reputation::remove_from_blacklist,
            security::reputation::get_blacklist,
            security::reputation::submit_reputation_report,
            security::reputation::get_reputation_history,
            security::reputation::get_reputation_stats,
            security::reputation::get_reputation_settings,
            security::reputation::update_reputation_settings,
            // Academy System
            academy::create_course,
            academy::get_course,
            academy::list_courses,
            academy::create_lesson,
            academy::get_course_lessons,
            academy::create_quiz,
            academy::get_quiz,
            academy::create_challenge,
            academy::list_challenges,
            academy::create_webinar,
            academy::list_webinars,
            academy::create_mentor,
            academy::list_mentors,
            academy::get_content_stats,
            academy::start_course,
            academy::get_user_progress,
            academy::complete_course,
            academy::start_lesson,
            academy::get_lesson_progress,
            academy::update_lesson_progress,
            academy::complete_lesson,
            academy::submit_quiz,
            academy::get_quiz_attempts,
            academy::submit_challenge,
            academy::get_challenge_submissions,
            academy::record_webinar_attendance,
            academy::create_mentor_session,
            academy::get_user_mentor_sessions,
            academy::get_user_stats,
            academy::get_leaderboard,
            academy::create_badge,
            academy::get_badge,
            academy::list_badges,
            academy::award_badge,
            academy::get_user_badges,
            academy::issue_certificate,
            academy::get_user_certificates,
            academy::verify_certificate,
            academy::get_user_rewards,
            academy::claim_reward,
            academy::claim_all_rewards,
            academy::get_reward_stats,
            // Performance & Diagnostics
            get_performance_metrics,
            run_performance_test,
            reset_performance_stats,
            // Cache Management
            cache_commands::get_cache_statistics,
            cache_commands::clear_cache,
            cache_commands::warm_cache,
            cache_commands::get_ttl_config,
            cache_commands::update_ttl_config,
            cache_commands::reset_ttl_config,
            cache_commands::test_cache_performance,
            // Market Surveillance & Anomaly Detection
            add_price_data,
            add_transaction_data,
            get_anomalies,
            get_active_anomalies,
            dismiss_anomaly,
            update_anomaly_detection_config,
            get_anomaly_detection_config,
            get_anomaly_statistics,
            generate_mock_anomaly_data,
            // Event Sourcing & Audit Trail
            data::event_store::get_events_command,
            data::event_store::replay_events_command,
            data::event_store::get_state_at_time_command,
            data::event_store::export_audit_trail_command,
            data::event_store::create_snapshot_command,
            data::event_store::get_event_stats,
            // Data Compression
            data::compression_commands::get_compression_stats,
            data::compression_commands::compress_old_data,
            data::compression_commands::update_compression_config,
            data::compression_commands::get_compression_config,
            data::compression_commands::decompress_data,
            data::compression_commands::get_database_size,
            // Email Notifications
            email_save_config,
            email_get_config,
            email_delete_config,
            email_test_connection,
            email_send,
            email_get_stats,
            email_get_history,
            // Twitter Integration
            twitter_save_config,
            twitter_get_config,
            twitter_delete_config,
            twitter_test_connection,
            twitter_add_keyword,
            twitter_list_keywords,
            twitter_remove_keyword,
            twitter_add_influencer,
            twitter_list_influencers,
            twitter_remove_influencer,
            twitter_fetch_sentiment,
            twitter_get_sentiment_history,
            twitter_get_stats,
            twitter_get_tweet_history,
            // Token Flow Intelligence
            token_flow::commands::analyze_token_flows,
            token_flow::commands::export_flow_analysis,
            token_flow::commands::list_cluster_subscriptions,
            token_flow::commands::upsert_cluster_subscription,
            token_flow::commands::remove_cluster_subscription,
            // Holder Analysis & Metadata
            market::holders::get_holder_distribution,
            market::holders::get_holder_trends,
            market::holders::get_large_transfers,
            market::holders::get_token_metadata,
            market::holders::get_verification_status,
            market::holders::export_holder_data,
            market::holders::export_metadata_snapshot,
            // Prediction Markets
            market::get_prediction_markets,
            market::search_prediction_markets,
            market::create_custom_prediction,
            market::get_custom_predictions,
            market::update_custom_prediction,
            market::get_portfolio_comparison,
            market::get_consensus_data,
            market::record_prediction_performance,
            // Indicator & drawing commands
            indicator_save_state,
            indicator_list_presets,
            indicator_save_preset,
            indicator_delete_preset,
            indicator_update_preset,
            indicator_list_alerts,
            indicator_create_alert,
            indicator_delete_alert,
            indicator_update_alert,
            drawing_list,
            drawing_save,
            drawing_sync,
            drawing_list_templates,
            drawing_save_templates,
            // Chain management
            chain_get_active,
            chain_set_active,
            chain_list_chains,
            chain_list_enabled,
            chain_update_config,
            chain_get_balance,
            chain_get_fee_estimate,
            chain_get_status,
            chain_get_cross_chain_portfolio,
            // Bridge integrations
            bridge_get_quote,
            bridge_create_transaction,
            bridge_get_transaction,
            bridge_list_transactions,
            bridge_list_transactions_by_status,
            bridge_update_transaction_status,
            bridge_update_transaction_hash,
            bridge_poll_status,
            // Launchpad commands
            create_launch_config,
            update_launch_config,
            get_launch_config,
            list_launches,
            simulate_token_creation,
            launchpad_create_token,
            check_launch_safety,
            check_vesting_compliance,
            check_liquidity_lock_compliance,
            create_liquidity_lock,
            unlock_liquidity,
            get_liquidity_lock,
            list_liquidity_locks,
            create_vesting_schedule,
            release_vested_tokens,
            get_vesting_schedule,
            list_vesting_schedules,
            create_airdrop,
            activate_airdrop,
            claim_airdrop_tokens,
            get_airdrop,
            get_airdrop_metrics,
            get_distribution_metrics,
            // Stock commands
            stocks::get_trending_stocks,
            stocks::get_top_movers,
            stocks::get_new_ipos,
            stocks::get_earnings_calendar,
            stocks::get_stock_news,
            stocks::get_institutional_holdings,
            stocks::get_insider_activity,
            stocks::create_stock_alert,
            stocks::get_stock_alerts,
            // DeFi commands
            get_solend_reserves,
            get_solend_pools,
            get_solend_positions,
            get_marginfi_banks,
            get_marginfi_positions,
            get_kamino_vaults,
            get_kamino_positions,
            get_kamino_farms,
            get_staking_pools,
            get_staking_positions,
            get_staking_schedule,
            get_yield_farms,
            get_farming_opportunities,
            get_farming_positions,
            get_defi_portfolio_summary,
            get_defi_risk_metrics,
            get_defi_snapshot,
            get_auto_compound_recommendations,
            configure_auto_compound,
            get_auto_compound_config,
            get_compound_history,
            estimate_compound_apy_boost,
            get_governance_proposals,
            vote_on_proposal,
            get_governance_participation,
            // Updater commands
            get_update_settings,
            save_update_settings,
            dismiss_update,
            get_rollback_info,
            rollback_update,
            // Windowing & Multi-monitor commands
            get_monitors,
            create_floating_window,
            close_floating_window,
            set_window_position,
            set_window_size,
            set_window_always_on_top,
            get_window_position,
            get_window_size,
            snap_window_to_edge,
            maximize_window,
            minimize_window,
            // Backup & Settings Management
            backup::service::create_backup,
            backup::service::restore_backup,
            backup::service::list_backups,
            backup::service::delete_backup,
            backup::service::verify_backup_integrity,
            backup::service::export_settings,
            backup::service::import_settings,
            backup::service::reset_settings,
            backup::service::get_settings_template,
            backup::service::get_backup_schedule,
            backup::service::update_backup_schedule,
            backup::service::get_backup_status,
            backup::service::trigger_manual_backup,
            // Universal Settings
            config::commands::get_all_settings,
            config::commands::update_setting,
            config::commands::bulk_update_settings,
            config::commands::reset_config_settings,
            config::commands::export_config_settings,
            config::commands::import_config_settings,
            config::commands::get_setting_schema,
            config::commands::create_settings_profile,
            config::commands::load_settings_profile,
            config::commands::delete_settings_profile,
            config::commands::list_settings_profiles,
            config::commands::get_settings_change_history,
            config::commands::get_config_settings_template,
            // System Tray
            get_tray_settings,
            update_tray_settings,
            update_tray_stats,
            update_tray_badge,
            minimize_to_tray,
            restore_from_tray,
            // Auto-start
            get_auto_start_settings,
            update_auto_start_settings,
            check_auto_start_enabled,
            enable_auto_start,
            disable_auto_start,
            // Historical Replay
            historical_fetch_dataset,
            historical_fetch_orderbooks,
            historical_run_simulation,
            historical_compute_counterfactual,
            historical_get_cache_stats,
            historical_clear_old_data,
            historical_set_api_key,
            // Voice Interaction
            voice_request_permissions,
            voice_revoke_permissions,
            voice_start_microphone,
            voice_stop_microphone,
            voice_get_audio_status,
            voice_start_wake_word,
            voice_stop_wake_word,
            voice_get_wake_word_config,
            voice_update_wake_word_config,
            voice_process_audio_for_wake_word,
            voice_start_recognition,
            voice_stop_recognition,
            voice_get_stt_config,
            voice_update_stt_config,
            voice_get_supported_languages,
            voice_set_stt_language,
            voice_simulate_transcription,
            voice_speak,
            voice_stop_speaking,
            voice_pause_speaking,
            voice_resume_speaking,
            voice_get_tts_status,
            voice_get_tts_config,
            voice_update_tts_config,
            voice_get_available_voices,
            voice_set_voice,
            voice_set_rate,
            voice_set_pitch,
            voice_set_volume,
            // AI Chat
            ai_chat_message,
            ai_chat_message_stream,
            ai_submit_feedback,
            ai_execute_quick_action,
            ai_optimize_portfolio,
            ai_apply_optimization,
            ai_get_pattern_warnings,
            ai_dismiss_pattern_warning,
            // Voice Trading
            execute_voice_trade,
            get_portfolio_data,
            get_current_price,
            create_price_alert,
            list_alerts,
            get_market_summary,
            synthesize_speech,
            validate_voice_mfa,
            check_voice_permission,
            get_voice_capabilities,
            // Safety Mode Engine
            check_trade_safety,
            pre_trade_contract_check,
            approve_trade,
            get_safety_policy,
            update_safety_policy,
            get_cooldown_status,
            reset_daily_limits,
            get_insurance_quote,
            select_insurance,
            list_insurance_providers,
            get_emergency_halt,
            set_emergency_halt,
            // Contract Risk Monitoring
            assess_contract_risk,
            get_contract_risk_events,
            monitor_contract,
            unmonitor_contract,
            list_monitored_contracts,
            refresh_monitored_contracts,
            // Theme Engine
            theme_get_presets,
            theme_get_settings,
            theme_update_settings,
            theme_save_custom,
            theme_delete_custom,
            theme_export,
            theme_import,
            theme_get_os_preference,
            // Mobile companion commands
            mobile_register_device,
            mobile_create_biometric_challenge,
            mobile_verify_biometric,
            mobile_authenticate_session,
            mobile_revoke_session,
            mobile_update_push_token,
            mobile_get_devices,
            mobile_remove_device,
            mobile_queue_notification,
            mobile_get_pending_notifications,
            mobile_dequeue_notification,
            mobile_sync_data,
            mobile_get_last_sync,
            mobile_get_cached_sync_data,
            mobile_execute_quick_trade,
            mobile_safety_checks,
            mobile_get_widget_data,
            mobile_get_all_widgets,
            // Collaborative Rooms
            collab::commands::collab_create_room,
            collab::commands::collab_list_rooms,
            collab::commands::collab_get_room,
            collab::commands::collab_delete_room,
            collab::commands::collab_join_room,
            collab::commands::collab_leave_room,
            collab::commands::collab_get_participants,
            collab::commands::collab_update_permissions,
            collab::commands::collab_send_message,
            collab::commands::collab_get_messages,
            collab::commands::collab_share_watchlist,
            collab::commands::collab_get_watchlists,
            collab::commands::collab_share_order,
            collab::commands::collab_get_orders,
            collab::commands::collab_update_order,
            collab::commands::collab_share_strategy,
            collab::commands::collab_send_webrtc_signal,
            collab::commands::collab_get_webrtc_signals,
            collab::commands::collab_moderate_user,
            collab::commands::collab_get_room_state,
            collab::commands::collab_set_competition,
            collab::commands::collab_get_competition,
            collab::commands::collab_update_leaderboard,
            // Diagnostics & Troubleshooter
            diagnostics::tauri_commands::run_diagnostics,
            diagnostics::tauri_commands::get_health_report,
            diagnostics::tauri_commands::auto_repair_issue,
            diagnostics::tauri_commands::auto_repair,
            diagnostics::tauri_commands::verify_integrity,
            diagnostics::tauri_commands::manual_repair,
            diagnostics::tauri_commands::download_missing,
            diagnostics::tauri_commands::restore_defaults,
            diagnostics::tauri_commands::get_repair_history,
            diagnostics::tauri_commands::get_diagnostics_settings,
            diagnostics::tauri_commands::save_diagnostics_settings,
            diagnostics::tauri_commands::backup_before_repair,
            diagnostics::tauri_commands::rollback_repair,
            diagnostics::tauri_commands::export_diagnostics_report,
            // Governance
            sync_governance_memberships,
            get_governance_memberships,
            sync_governance_proposals,
            get_governance_proposals,
            get_all_active_governance_proposals,
            get_wallet_voting_power,
            submit_signed_vote,
            delegate_governance_votes,
            revoke_governance_delegation,
            get_governance_delegations,
            analyze_governance_proposal,
            create_governance_reminder,
            get_governance_summary,
            get_governance_deadlines,
            prepare_vote_signature,
            verify_vote_signature,
            prepare_vote_transaction,
            // Journal
            create_journal_entry,
            get_journal_entry,
            update_journal_entry,
            delete_journal_entry,
            get_journal_entries,
            get_journal_entries_count,
            generate_weekly_report,
            get_weekly_report,
            get_weekly_reports,
            get_behavioral_analytics,
            get_journal_stats,
            // Dev Tools
            compile_now,
            get_build_status,
            get_compile_errors,
            auto_fix_errors,
            get_fix_stats,
            get_fix_attempts,
            clear_fix_history,
            get_logs,
            clear_logs,
            export_logs,
            log_message,
            get_logger_config,
            set_logger_config,
            get_error_stats,
            report_crash,
            get_crash_report,
            list_crash_reports,
            force_gc,
            restart_service,
            get_dev_settings,
            update_dev_settings,
            check_tauri_health,
            // P2P Marketplace & Escrow
            create_p2p_offer,
            get_p2p_offer,
            list_p2p_offers,
            update_offer_status,
            match_p2p_offers,
            create_p2p_escrow,
            get_p2p_escrow,
            list_p2p_escrows,
            fund_p2p_escrow,
            confirm_payment_p2p,
            release_p2p_escrow,
            cancel_p2p_escrow,
            file_p2p_dispute,
            get_p2p_dispute,
            submit_dispute_evidence,
            resolve_p2p_dispute,
            send_p2p_message,
            get_p2p_messages,
            get_trader_profile,
            check_p2p_compliance,
            get_p2p_stats,
            // Feature Flags
            get_feature_flags,
            enable_feature_flag,
            disable_feature_flag,
            is_feature_enabled,
        ]);

    startup_log!("Invoke handler attached");
    startup_log!("Launching Tauri application loop");
    if let Err(e) = builder.run(tauri::generate_context!()) {
        startup_error!("Failed to run Tauri application: {}", e);
        std::process::exit(1);
    }
}
