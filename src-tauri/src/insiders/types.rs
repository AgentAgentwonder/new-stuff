use crate::utils::Rfc3339DateTime;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Row, Sqlite};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoredWallet {
    pub id: String,
    pub wallet_address: String,
    pub label: Option<String>,
    pub min_transaction_size: Option<f64>,
    pub is_whale: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for MonitoredWallet {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        Ok(MonitoredWallet {
            id: row.try_get("id")?,
            wallet_address: row.try_get("wallet_address")?,
            label: row.try_get("label")?,
            min_transaction_size: row.try_get("min_transaction_size")?,
            is_whale: row.try_get("is_whale")?,
            is_active: row.try_get("is_active")?,
            created_at: Rfc3339DateTime::try_from(row.try_get::<String, _>("created_at")?)?.into(),
            updated_at: Rfc3339DateTime::try_from(row.try_get::<String, _>("updated_at")?)?.into(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddMonitoredWalletRequest {
    pub wallet_address: String,
    pub label: Option<String>,
    pub min_transaction_size: Option<f64>,
    pub is_whale: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMonitoredWalletRequest {
    pub id: String,
    pub label: Option<String>,
    pub min_transaction_size: Option<f64>,
    pub is_whale: Option<bool>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletActivity {
    pub id: String,
    pub wallet_address: String,
    pub wallet_label: Option<String>,
    pub tx_signature: String,
    #[serde(rename = "type")]
    pub action_type: String,
    pub input_mint: Option<String>,
    pub output_mint: Option<String>,
    pub input_symbol: Option<String>,
    pub output_symbol: Option<String>,
    pub amount: Option<f64>,
    pub amount_usd: Option<f64>,
    pub price: Option<f64>,
    pub is_whale: bool,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletActivityBatch {
    pub activities: Vec<WalletActivity>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActivityAction {
    Buy,
    Sell,
    Transfer,
    Swap,
    Unknown,
}

impl ActivityAction {
    pub fn as_str(&self) -> &str {
        match self {
            ActivityAction::Buy => "buy",
            ActivityAction::Sell => "sell",
            ActivityAction::Transfer => "transfer",
            ActivityAction::Swap => "swap",
            ActivityAction::Unknown => "unknown",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "buy" => ActivityAction::Buy,
            "sell" => ActivityAction::Sell,
            "transfer" => ActivityAction::Transfer,
            "swap" => ActivityAction::Swap,
            _ => ActivityAction::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyTradeRequest {
    pub wallet_activity_id: String,
    pub wallet_address: String,
    pub multiplier: f64,
    pub delay_seconds: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityFilter {
    pub wallets: Option<Vec<String>>,
    pub tokens: Option<Vec<String>>,
    pub actions: Option<Vec<String>>,
    pub min_amount_usd: Option<f64>,
    pub max_amount_usd: Option<f64>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletActivityRecord {
    pub id: String,
    pub wallet_address: String,
    pub tx_signature: String,
    pub action_type: String,
    pub input_mint: Option<String>,
    pub output_mint: Option<String>,
    pub input_symbol: Option<String>,
    pub output_symbol: Option<String>,
    pub amount: Option<f64>,
    pub amount_usd: Option<f64>,
    pub price: Option<f64>,
    pub timestamp: DateTime<Utc>,
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for WalletActivityRecord {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        Ok(WalletActivityRecord {
            id: row.try_get("id")?,
            wallet_address: row.try_get("wallet_address")?,
            tx_signature: row.try_get("tx_signature")?,
            action_type: row.try_get("action_type")?,
            input_mint: row.try_get("input_mint")?,
            output_mint: row.try_get("output_mint")?,
            input_symbol: row.try_get("input_symbol")?,
            output_symbol: row.try_get("output_symbol")?,
            amount: row.try_get("amount")?,
            amount_usd: row.try_get("amount_usd")?,
            price: row.try_get("price")?,
            timestamp: Rfc3339DateTime::try_from(row.try_get::<String, _>("timestamp")?)?.into(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletStatistics {
    pub wallet_address: String,
    pub total_transactions: i64,
    pub buy_count: i64,
    pub sell_count: i64,
    pub transfer_count: i64,
    pub total_volume_usd: f64,
    pub avg_transaction_size: f64,
    pub last_activity: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SmartMoneyWallet {
    pub id: String,
    pub wallet_address: String,
    pub label: Option<String>,
    pub total_trades: i64,
    pub winning_trades: i64,
    pub losing_trades: i64,
    pub win_rate: f64,
    pub total_pnl: f64,
    pub avg_hold_time_hours: f64,
    pub smart_money_score: f64,
    pub is_smart_money: bool,
    pub first_seen: String,
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartMoneyClassification {
    pub wallet_address: String,
    pub is_smart_money: bool,
    pub score: f64,
    pub reason: String,
    pub metrics: SmartMoneyMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartMoneyMetrics {
    pub total_trades: i64,
    pub win_rate: f64,
    pub avg_profit_per_trade: f64,
    pub sharpe_ratio: Option<f64>,
    pub max_drawdown: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhaleAlert {
    pub id: String,
    pub wallet_address: String,
    pub wallet_label: Option<String>,
    pub activity_id: String,
    pub tx_signature: String,
    pub action_type: String,
    pub token_symbol: Option<String>,
    pub amount_usd: f64,
    pub threshold: f64,
    pub alert_sent: bool,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartMoneyConsensus {
    pub token_mint: String,
    pub token_symbol: Option<String>,
    pub action: String,
    pub smart_wallets_count: i64,
    pub total_volume_usd: f64,
    pub avg_price: f64,
    pub consensus_strength: f64,
    pub first_seen: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    pub id: String,
    pub alert_type: AlertType,
    pub enabled: bool,
    pub threshold: Option<f64>,
    pub push_enabled: bool,
    pub email_enabled: bool,
    pub telegram_enabled: bool,
    pub telegram_config_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AlertType {
    WhaleTransaction,
    SmartMoneyBuy,
    SmartMoneySell,
    SmartMoneyConsensus,
}

impl AlertType {
    pub fn as_str(&self) -> &str {
        match self {
            AlertType::WhaleTransaction => "whale_transaction",
            AlertType::SmartMoneyBuy => "smart_money_buy",
            AlertType::SmartMoneySell => "smart_money_sell",
            AlertType::SmartMoneyConsensus => "smart_money_consensus",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentComparison {
    pub token_mint: String,
    pub token_symbol: Option<String>,
    pub smart_money_sentiment: f64,
    pub retail_sentiment: f64,
    pub divergence: f64,
    pub smart_money_volume: f64,
    pub retail_volume: f64,
    pub timestamp: DateTime<Utc>,
}

pub struct WalletMonitorDatabase {
    pool: Pool<Sqlite>,
}

impl WalletMonitorDatabase {
    pub async fn new(pool: Pool<Sqlite>) -> Result<Self, sqlx::Error> {
        let db = Self { pool };
        db.initialize().await?;
        Ok(db)
    }

    async fn initialize(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS monitored_wallets (
                id TEXT PRIMARY KEY,
                wallet_address TEXT NOT NULL UNIQUE,
                label TEXT,
                min_transaction_size REAL,
                is_whale INTEGER NOT NULL DEFAULT 0,
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS wallet_activities (
                id TEXT PRIMARY KEY,
                wallet_address TEXT NOT NULL,
                tx_signature TEXT NOT NULL UNIQUE,
                action_type TEXT NOT NULL,
                input_mint TEXT,
                output_mint TEXT,
                input_symbol TEXT,
                output_symbol TEXT,
                amount REAL,
                amount_usd REAL,
                price REAL,
                timestamp TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_monitored_wallets_active ON monitored_wallets(is_active);
            CREATE INDEX IF NOT EXISTS idx_monitored_wallets_address ON monitored_wallets(wallet_address);
            CREATE INDEX IF NOT EXISTS idx_wallet_activities_wallet ON wallet_activities(wallet_address);
            CREATE INDEX IF NOT EXISTS idx_wallet_activities_time ON wallet_activities(timestamp);
            CREATE INDEX IF NOT EXISTS idx_wallet_activities_action ON wallet_activities(action_type);
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS smart_money_wallets (
                id TEXT PRIMARY KEY,
                wallet_address TEXT NOT NULL UNIQUE,
                label TEXT,
                total_trades INTEGER NOT NULL DEFAULT 0,
                winning_trades INTEGER NOT NULL DEFAULT 0,
                losing_trades INTEGER NOT NULL DEFAULT 0,
                win_rate REAL NOT NULL DEFAULT 0.0,
                total_pnl REAL NOT NULL DEFAULT 0.0,
                avg_hold_time_hours REAL NOT NULL DEFAULT 0.0,
                smart_money_score REAL NOT NULL DEFAULT 0.0,
                is_smart_money INTEGER NOT NULL DEFAULT 0,
                first_seen TEXT NOT NULL,
                last_updated TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS whale_alerts (
                id TEXT PRIMARY KEY,
                wallet_address TEXT NOT NULL,
                wallet_label TEXT,
                activity_id TEXT NOT NULL,
                tx_signature TEXT NOT NULL,
                action_type TEXT NOT NULL,
                token_symbol TEXT,
                amount_usd REAL NOT NULL,
                threshold REAL NOT NULL,
                alert_sent INTEGER NOT NULL DEFAULT 0,
                timestamp TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS alert_configs (
                id TEXT PRIMARY KEY,
                alert_type TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                threshold REAL,
                push_enabled INTEGER NOT NULL DEFAULT 1,
                email_enabled INTEGER NOT NULL DEFAULT 0,
                telegram_enabled INTEGER NOT NULL DEFAULT 0,
                telegram_config_id TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_smart_money_wallets_score ON smart_money_wallets(smart_money_score);
            CREATE INDEX IF NOT EXISTS idx_smart_money_wallets_is_smart ON smart_money_wallets(is_smart_money);
            CREATE INDEX IF NOT EXISTS idx_whale_alerts_timestamp ON whale_alerts(timestamp);
            CREATE INDEX IF NOT EXISTS idx_whale_alerts_wallet ON whale_alerts(wallet_address);
            CREATE INDEX IF NOT EXISTS idx_alert_configs_type ON alert_configs(alert_type);
            "#,
        )
        .execute(&self.pool)
        .await?;

        self.insert_default_alert_configs().await?;

        Ok(())
    }

    async fn insert_default_alert_configs(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO alert_configs (id, alert_type, enabled, threshold, push_enabled, email_enabled, telegram_enabled)
            VALUES 
                ('whale_tx', 'whale_transaction', 1, 50000.0, 1, 0, 0),
                ('smart_buy', 'smart_money_buy', 1, 10000.0, 1, 0, 0),
                ('smart_sell', 'smart_money_sell', 1, 10000.0, 1, 0, 0),
                ('consensus', 'smart_money_consensus', 1, NULL, 1, 0, 0)
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn add_monitored_wallet(&self, wallet: &MonitoredWallet) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO monitored_wallets (
                id, wallet_address, label, min_transaction_size, is_whale, is_active, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
        )
        .bind(&wallet.id)
        .bind(&wallet.wallet_address)
        .bind(&wallet.label)
        .bind(wallet.min_transaction_size)
        .bind(if wallet.is_whale { 1 } else { 0 })
        .bind(if wallet.is_active { 1 } else { 0 })
        .bind(wallet.created_at.to_rfc3339())
        .bind(wallet.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_monitored_wallet(
        &self,
        id: &str,
    ) -> Result<Option<MonitoredWallet>, sqlx::Error> {
        sqlx::query_as::<_, MonitoredWallet>("SELECT * FROM monitored_wallets WHERE id = ?1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn list_monitored_wallets(&self) -> Result<Vec<MonitoredWallet>, sqlx::Error> {
        sqlx::query_as::<_, MonitoredWallet>(
            "SELECT * FROM monitored_wallets ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_active_monitored_wallets(&self) -> Result<Vec<MonitoredWallet>, sqlx::Error> {
        sqlx::query_as::<_, MonitoredWallet>("SELECT * FROM monitored_wallets WHERE is_active = 1")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn update_monitored_wallet(
        &self,
        id: &str,
        label: Option<String>,
        min_transaction_size: Option<f64>,
        is_whale: Option<bool>,
        is_active: Option<bool>,
    ) -> Result<(), sqlx::Error> {
        let wallet = self
            .get_monitored_wallet(id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            UPDATE monitored_wallets
            SET label = ?1, min_transaction_size = ?2, is_whale = ?3, is_active = ?4, updated_at = ?5
            WHERE id = ?6
            "#,
        )
        .bind(label.unwrap_or(wallet.label.unwrap_or_default()))
        .bind(min_transaction_size.unwrap_or(wallet.min_transaction_size.unwrap_or(0.0)))
        .bind(if is_whale.unwrap_or(wallet.is_whale) { 1 } else { 0 })
        .bind(if is_active.unwrap_or(wallet.is_active) { 1 } else { 0 })
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn remove_monitored_wallet(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM monitored_wallets WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn add_activity(&self, activity: &WalletActivityRecord) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO wallet_activities (
                id, wallet_address, tx_signature, action_type, input_mint, output_mint,
                input_symbol, output_symbol, amount, amount_usd, price, timestamp
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            "#,
        )
        .bind(&activity.id)
        .bind(&activity.wallet_address)
        .bind(&activity.tx_signature)
        .bind(&activity.action_type)
        .bind(&activity.input_mint)
        .bind(&activity.output_mint)
        .bind(&activity.input_symbol)
        .bind(&activity.output_symbol)
        .bind(activity.amount)
        .bind(activity.amount_usd)
        .bind(activity.price)
        .bind(activity.timestamp.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_activities(
        &self,
        filter: &ActivityFilter,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<WalletActivityRecord>, sqlx::Error> {
        let mut query = String::from("SELECT * FROM wallet_activities WHERE 1=1");

        if let Some(wallets) = &filter.wallets {
            if !wallets.is_empty() {
                let placeholders: Vec<String> = wallets.iter().map(|_| "?".to_string()).collect();
                query.push_str(&format!(
                    " AND wallet_address IN ({})",
                    placeholders.join(",")
                ));
            }
        }

        if let Some(actions) = &filter.actions {
            if !actions.is_empty() {
                let placeholders: Vec<String> = actions.iter().map(|_| "?".to_string()).collect();
                query.push_str(&format!(" AND action_type IN ({})", placeholders.join(",")));
            }
        }

        if let Some(min) = filter.min_amount_usd {
            query.push_str(&format!(" AND amount_usd >= {}", min));
        }

        if let Some(max) = filter.max_amount_usd {
            query.push_str(&format!(" AND amount_usd <= {}", max));
        }

        query.push_str(" ORDER BY timestamp DESC LIMIT ? OFFSET ?");

        let mut q = sqlx::query_as::<_, WalletActivityRecord>(&query);

        if let Some(wallets) = &filter.wallets {
            for wallet in wallets {
                q = q.bind(wallet);
            }
        }

        if let Some(actions) = &filter.actions {
            for action in actions {
                q = q.bind(action);
            }
        }

        q = q.bind(limit).bind(offset);

        q.fetch_all(&self.pool).await
    }

    pub async fn get_wallet_statistics(
        &self,
        wallet_address: &str,
    ) -> Result<WalletStatistics, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total,
                SUM(CASE WHEN action_type = 'buy' THEN 1 ELSE 0 END) as buys,
                SUM(CASE WHEN action_type = 'sell' THEN 1 ELSE 0 END) as sells,
                SUM(CASE WHEN action_type = 'transfer' THEN 1 ELSE 0 END) as transfers,
                COALESCE(SUM(amount_usd), 0) as volume,
                COALESCE(AVG(amount_usd), 0) as avg_size,
                MAX(timestamp) as last_activity
            FROM wallet_activities
            WHERE wallet_address = ?1
            "#,
        )
        .bind(wallet_address)
        .fetch_one(&self.pool)
        .await?;

        Ok(WalletStatistics {
            wallet_address: wallet_address.to_string(),
            total_transactions: row.try_get("total").unwrap_or(0),
            buy_count: row.try_get("buys").unwrap_or(0),
            sell_count: row.try_get("sells").unwrap_or(0),
            transfer_count: row.try_get("transfers").unwrap_or(0),
            total_volume_usd: row.try_get("volume").unwrap_or(0.0),
            avg_transaction_size: row.try_get("avg_size").unwrap_or(0.0),
            last_activity: row
                .try_get::<String, _>("last_activity")
                .ok()
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
        })
    }

    pub fn pool(&self) -> Pool<Sqlite> {
        self.pool.clone()
    }
}
