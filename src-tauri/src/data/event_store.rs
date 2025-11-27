use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite, SqlitePool};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    OrderPlaced {
        order_id: String,
        symbol: String,
        side: String,
        quantity: f64,
        price: Option<f64>,
        timestamp: DateTime<Utc>,
    },
    OrderFilled {
        order_id: String,
        fill_price: f64,
        filled_quantity: f64,
        timestamp: DateTime<Utc>,
    },
    OrderCancelled {
        order_id: String,
        reason: String,
        timestamp: DateTime<Utc>,
    },
    PositionOpened {
        position_id: String,
        symbol: String,
        quantity: f64,
        entry_price: f64,
        timestamp: DateTime<Utc>,
    },
    PositionClosed {
        position_id: String,
        exit_price: f64,
        pnl: f64,
        timestamp: DateTime<Utc>,
    },
    BalanceChanged {
        wallet: String,
        token: String,
        old_balance: f64,
        new_balance: f64,
        reason: String,
        timestamp: DateTime<Utc>,
    },
    SettingChanged {
        key: String,
        old_value: String,
        new_value: String,
        timestamp: DateTime<Utc>,
    },
    WalletConnected {
        wallet_address: String,
        wallet_type: String,
        timestamp: DateTime<Utc>,
    },
    WalletDisconnected {
        wallet_address: String,
        timestamp: DateTime<Utc>,
    },
    TradeExecuted {
        trade_id: String,
        from_token: String,
        to_token: String,
        from_amount: f64,
        to_amount: f64,
        price: f64,
        timestamp: DateTime<Utc>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EventRecord {
    pub id: String,
    pub event_type: String,
    pub event_data: String,
    pub aggregate_id: String,
    pub sequence: i64,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SnapshotRecord {
    pub id: String,
    pub aggregate_id: String,
    pub state_data: String,
    pub sequence: i64,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFilter {
    pub aggregate_id: Option<String>,
    pub event_type: Option<String>,
    pub from_time: Option<DateTime<Utc>>,
    pub to_time: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub struct EventStore {
    pool: Pool<Sqlite>,
    sequence_counters: Arc<RwLock<HashMap<String, i64>>>,
    point_in_time_cache: Arc<RwLock<HashMap<String, (DateTime<Utc>, String)>>>,
}

impl EventStore {
    pub async fn new(db_path: PathBuf) -> Result<Self, sqlx::Error> {
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        
        let pool = match SqlitePool::connect(&db_url).await {
            Ok(pool) => pool,
            Err(e) => {
                eprintln!("Warning: EventStore failed to connect to {:?}: {}", db_path, e);
                eprintln!("Falling back to in-memory database for EventStore");
                eprintln!("EventStore using in-memory database for this session");
                SqlitePool::connect("sqlite::memory:").await?
            }
        };

        let store = Self {
            pool,
            sequence_counters: Arc::new(RwLock::new(HashMap::new())),
            point_in_time_cache: Arc::new(RwLock::new(HashMap::new())),
        };

        store.initialize().await?;
        store.load_sequence_counters().await?;

        Ok(store)
    }

    async fn initialize(&self) -> Result<(), sqlx::Error> {
        // Create events table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                event_type TEXT NOT NULL,
                event_data TEXT NOT NULL,
                aggregate_id TEXT NOT NULL,
                sequence INTEGER NOT NULL,
                timestamp TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes on events table
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_events_aggregate ON events(aggregate_id, sequence);
            CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp);
            CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type);
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create snapshots table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS snapshots (
                id TEXT PRIMARY KEY,
                aggregate_id TEXT NOT NULL,
                state_data TEXT NOT NULL,
                sequence INTEGER NOT NULL,
                timestamp TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes on snapshots table
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_snapshots_aggregate ON snapshots(aggregate_id, sequence DESC);
            CREATE INDEX IF NOT EXISTS idx_snapshots_timestamp ON snapshots(timestamp);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn load_sequence_counters(&self) -> Result<(), sqlx::Error> {
        let records = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT aggregate_id, MAX(sequence) as max_sequence
            FROM events
            GROUP BY aggregate_id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut counters = self.sequence_counters.write().await;
        for (aggregate_id, max_sequence) in records {
            counters.insert(aggregate_id, max_sequence);
        }

        Ok(())
    }

    async fn get_next_sequence(&self, aggregate_id: &str) -> i64 {
        let mut counters = self.sequence_counters.write().await;
        let counter = counters.entry(aggregate_id.to_string()).or_insert(0);
        *counter += 1;
        *counter
    }

    pub async fn publish_event(
        &self,
        event: Event,
        aggregate_id: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let event_id = Uuid::new_v4().to_string();
        let event_type = self.get_event_type(&event);
        let event_data = serde_json::to_string(&event)?;
        let sequence = self.get_next_sequence(aggregate_id).await;
        let timestamp = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO events (id, event_type, event_data, aggregate_id, sequence, timestamp)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(&event_id)
        .bind(&event_type)
        .bind(&event_data)
        .bind(aggregate_id)
        .bind(sequence)
        .bind(&timestamp)
        .execute(&self.pool)
        .await?;

        // Check if we should create a snapshot (every 1000 events)
        if sequence % 1000 == 0 {
            if let Err(e) = self.create_snapshot_internal(aggregate_id).await {
                eprintln!("Failed to create automatic snapshot: {}", e);
            }
        }

        Ok(event_id)
    }

    fn get_event_type(&self, event: &Event) -> String {
        match event {
            Event::OrderPlaced { .. } => "order_placed",
            Event::OrderFilled { .. } => "order_filled",
            Event::OrderCancelled { .. } => "order_cancelled",
            Event::PositionOpened { .. } => "position_opened",
            Event::PositionClosed { .. } => "position_closed",
            Event::BalanceChanged { .. } => "balance_changed",
            Event::SettingChanged { .. } => "setting_changed",
            Event::WalletConnected { .. } => "wallet_connected",
            Event::WalletDisconnected { .. } => "wallet_disconnected",
            Event::TradeExecuted { .. } => "trade_executed",
        }
        .to_string()
    }

    pub async fn get_events(&self, filter: EventFilter) -> Result<Vec<EventRecord>, sqlx::Error> {
        let mut query = String::from("SELECT * FROM events WHERE 1=1");
        let mut conditions = Vec::new();

        if filter.aggregate_id.is_some() {
            conditions.push("aggregate_id = ?".to_string());
        }
        if filter.event_type.is_some() {
            conditions.push("event_type = ?".to_string());
        }
        if filter.from_time.is_some() {
            conditions.push("timestamp >= ?".to_string());
        }
        if filter.to_time.is_some() {
            conditions.push("timestamp <= ?".to_string());
        }

        for condition in conditions {
            query.push_str(&format!(" AND {}", condition));
        }

        query.push_str(" ORDER BY sequence ASC");

        if let Some(limit) = filter.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }
        if let Some(offset) = filter.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        let mut sql_query = sqlx::query_as::<_, EventRecord>(&query);

        if let Some(ref aggregate_id) = filter.aggregate_id {
            sql_query = sql_query.bind(aggregate_id);
        }
        if let Some(ref event_type) = filter.event_type {
            sql_query = sql_query.bind(event_type);
        }
        if let Some(from_time) = filter.from_time {
            sql_query = sql_query.bind(from_time.to_rfc3339());
        }
        if let Some(to_time) = filter.to_time {
            sql_query = sql_query.bind(to_time.to_rfc3339());
        }

        sql_query.fetch_all(&self.pool).await
    }

    pub async fn replay_events(
        &self,
        aggregate_id: &str,
    ) -> Result<Vec<Event>, Box<dyn std::error::Error>> {
        let filter = EventFilter {
            aggregate_id: Some(aggregate_id.to_string()),
            event_type: None,
            from_time: None,
            to_time: None,
            limit: None,
            offset: None,
        };

        let records = self.get_events(filter).await?;
        let mut events = Vec::new();

        for record in records {
            let event: Event = serde_json::from_str(&record.event_data)?;
            events.push(event);
        }

        Ok(events)
    }

    pub async fn get_state_at_time(
        &self,
        aggregate_id: &str,
        timestamp: DateTime<Utc>,
    ) -> Result<Vec<Event>, Box<dyn std::error::Error>> {
        // Check cache first (5 minute cache)
        let cache_key = format!("{}_{}", aggregate_id, timestamp.to_rfc3339());
        {
            let cache = self.point_in_time_cache.read().await;
            if let Some((cached_time, cached_data)) = cache.get(&cache_key) {
                let elapsed = Utc::now().signed_duration_since(*cached_time);
                if elapsed.num_seconds() < 300 {
                    // 5 minutes
                    let events: Vec<Event> = serde_json::from_str(cached_data)?;
                    return Ok(events);
                }
            }
        }

        // Find latest snapshot before timestamp
        let snapshot_opt = sqlx::query_as::<_, SnapshotRecord>(
            r#"
            SELECT * FROM snapshots
            WHERE aggregate_id = ?1 AND timestamp <= ?2
            ORDER BY sequence DESC
            LIMIT 1
            "#,
        )
        .bind(aggregate_id)
        .bind(timestamp.to_rfc3339())
        .fetch_optional(&self.pool)
        .await?;

        let mut events = Vec::new();

        // Load events after snapshot
        let from_sequence = snapshot_opt.as_ref().map(|s| s.sequence).unwrap_or(0);

        let records = sqlx::query_as::<_, EventRecord>(
            r#"
            SELECT * FROM events
            WHERE aggregate_id = ?1 AND sequence > ?2 AND timestamp <= ?3
            ORDER BY sequence ASC
            "#,
        )
        .bind(aggregate_id)
        .bind(from_sequence)
        .bind(timestamp.to_rfc3339())
        .fetch_all(&self.pool)
        .await?;

        for record in records {
            let event: Event = serde_json::from_str(&record.event_data)?;
            events.push(event);
        }

        // Cache the result
        let cached_data = serde_json::to_string(&events)?;
        let mut cache = self.point_in_time_cache.write().await;
        cache.insert(cache_key, (Utc::now(), cached_data));

        // Limit cache size to 100 entries
        if cache.len() > 100 {
            if let Some(first_key) = cache.keys().next().cloned() {
                cache.remove(&first_key);
            }
        }

        Ok(events)
    }

    pub async fn create_snapshot(
        &self,
        aggregate_id: &str,
        state_data: &str,
    ) -> Result<String, sqlx::Error> {
        let snapshot_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now().to_rfc3339();

        // Get current sequence for this aggregate
        let sequence = {
            let counters = self.sequence_counters.read().await;
            *counters.get(aggregate_id).unwrap_or(&0)
        };

        sqlx::query(
            r#"
            INSERT INTO snapshots (id, aggregate_id, state_data, sequence, timestamp)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(&snapshot_id)
        .bind(aggregate_id)
        .bind(state_data)
        .bind(sequence)
        .bind(&timestamp)
        .execute(&self.pool)
        .await?;

        Ok(snapshot_id)
    }

    async fn create_snapshot_internal(&self, aggregate_id: &str) -> Result<String, sqlx::Error> {
        // Create a simple snapshot with current sequence number
        let state_data = serde_json::json!({
            "aggregate_id": aggregate_id,
            "snapshot_type": "automatic",
        })
        .to_string();

        self.create_snapshot(aggregate_id, &state_data).await
    }

    pub async fn export_events(
        &self,
        filter: EventFilter,
        format: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let events = self.get_events(filter).await?;

        match format {
            "json" => {
                let json_events: Vec<serde_json::Value> = events
                    .into_iter()
                    .map(|record| {
                        serde_json::json!({
                            "id": record.id,
                            "event_type": record.event_type,
                            "aggregate_id": record.aggregate_id,
                            "sequence": record.sequence,
                            "timestamp": record.timestamp,
                            "data": serde_json::from_str::<serde_json::Value>(&record.event_data).unwrap_or_default(),
                        })
                    })
                    .collect();
                Ok(serde_json::to_string_pretty(&json_events)?)
            }
            "csv" => {
                let mut csv =
                    String::from("ID,Event Type,Aggregate ID,Sequence,Timestamp,Description\n");
                for record in events {
                    let event: Event = serde_json::from_str(&record.event_data)?;
                    let description = self.get_event_description(&event);
                    csv.push_str(&format!(
                        "{},{},{},{},{},\"{}\"\n",
                        record.id,
                        record.event_type,
                        record.aggregate_id,
                        record.sequence,
                        record.timestamp,
                        description
                    ));
                }
                Ok(csv)
            }
            _ => Err("Unsupported format".into()),
        }
    }

    fn get_event_description(&self, event: &Event) -> String {
        match event {
            Event::OrderPlaced {
                order_id,
                symbol,
                side,
                quantity,
                price,
                ..
            } => {
                format!(
                    "Order {} placed: {} {} {} at {}",
                    order_id,
                    side,
                    quantity,
                    symbol,
                    price
                        .map(|p| p.to_string())
                        .unwrap_or_else(|| "market price".to_string())
                )
            }
            Event::OrderFilled {
                order_id,
                fill_price,
                filled_quantity,
                ..
            } => {
                format!(
                    "Order {} filled: {} units at {}",
                    order_id, filled_quantity, fill_price
                )
            }
            Event::OrderCancelled {
                order_id, reason, ..
            } => {
                format!("Order {} cancelled: {}", order_id, reason)
            }
            Event::PositionOpened {
                position_id,
                symbol,
                quantity,
                entry_price,
                ..
            } => {
                format!(
                    "Position {} opened: {} {} at {}",
                    position_id, quantity, symbol, entry_price
                )
            }
            Event::PositionClosed {
                position_id,
                exit_price,
                pnl,
                ..
            } => {
                format!(
                    "Position {} closed at {} (P&L: {})",
                    position_id, exit_price, pnl
                )
            }
            Event::BalanceChanged {
                wallet,
                token,
                old_balance,
                new_balance,
                reason,
                ..
            } => {
                format!(
                    "Balance changed for {} {}: {} -> {} ({})",
                    wallet, token, old_balance, new_balance, reason
                )
            }
            Event::SettingChanged {
                key,
                old_value,
                new_value,
                ..
            } => {
                format!("Setting '{}' changed: {} -> {}", key, old_value, new_value)
            }
            Event::WalletConnected {
                wallet_address,
                wallet_type,
                ..
            } => {
                format!("Wallet {} ({}) connected", wallet_address, wallet_type)
            }
            Event::WalletDisconnected { wallet_address, .. } => {
                format!("Wallet {} disconnected", wallet_address)
            }
            Event::TradeExecuted {
                trade_id,
                from_token,
                to_token,
                from_amount,
                to_amount,
                price,
                ..
            } => {
                format!(
                    "Trade {} executed: {} {} -> {} {} at {}",
                    trade_id, from_amount, from_token, to_amount, to_token, price
                )
            }
        }
    }

    pub async fn get_event_count(&self, aggregate_id: Option<&str>) -> Result<i64, sqlx::Error> {
        if let Some(aggregate_id) = aggregate_id {
            let (count,) =
                sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM events WHERE aggregate_id = ?1")
                    .bind(aggregate_id)
                    .fetch_one(&self.pool)
                    .await?;
            Ok(count)
        } else {
            let (count,) = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM events")
                .fetch_one(&self.pool)
                .await?;
            Ok(count)
        }
    }
}

pub type SharedEventStore = Arc<RwLock<EventStore>>;

// Tauri commands
#[tauri::command]
pub async fn get_events_command(
    event_store: tauri::State<'_, SharedEventStore>,
    aggregate_id: Option<String>,
    event_type: Option<String>,
    from_time: Option<String>,
    to_time: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<EventRecord>, String> {
    let from_time = if let Some(time_str) = from_time {
        Some(
            DateTime::parse_from_rfc3339(&time_str)
                .map_err(|e| e.to_string())?
                .with_timezone(&Utc),
        )
    } else {
        None
    };

    let to_time = if let Some(time_str) = to_time {
        Some(
            DateTime::parse_from_rfc3339(&time_str)
                .map_err(|e| e.to_string())?
                .with_timezone(&Utc),
        )
    } else {
        None
    };

    let filter = EventFilter {
        aggregate_id,
        event_type,
        from_time,
        to_time,
        limit,
        offset,
    };

    let store = event_store.read().await;
    store.get_events(filter).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn replay_events_command(
    event_store: tauri::State<'_, SharedEventStore>,
    aggregate_id: String,
) -> Result<Vec<Event>, String> {
    let store = event_store.read().await;
    store
        .replay_events(&aggregate_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_state_at_time_command(
    event_store: tauri::State<'_, SharedEventStore>,
    aggregate_id: String,
    timestamp: String,
) -> Result<Vec<Event>, String> {
    let timestamp = DateTime::parse_from_rfc3339(&timestamp)
        .map_err(|e| e.to_string())?
        .with_timezone(&Utc);

    let store = event_store.read().await;
    store
        .get_state_at_time(&aggregate_id, timestamp)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_audit_trail_command(
    event_store: tauri::State<'_, SharedEventStore>,
    aggregate_id: Option<String>,
    event_type: Option<String>,
    from_time: Option<String>,
    to_time: Option<String>,
    format: String,
) -> Result<String, String> {
    let from_time = if let Some(time_str) = from_time {
        Some(
            DateTime::parse_from_rfc3339(&time_str)
                .map_err(|e| e.to_string())?
                .with_timezone(&Utc),
        )
    } else {
        None
    };

    let to_time = if let Some(time_str) = to_time {
        Some(
            DateTime::parse_from_rfc3339(&time_str)
                .map_err(|e| e.to_string())?
                .with_timezone(&Utc),
        )
    } else {
        None
    };

    let filter = EventFilter {
        aggregate_id,
        event_type,
        from_time,
        to_time,
        limit: None,
        offset: None,
    };

    let store = event_store.read().await;
    store
        .export_events(filter, &format)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_snapshot_command(
    event_store: tauri::State<'_, SharedEventStore>,
    aggregate_id: String,
    state_data: String,
) -> Result<String, String> {
    let store = event_store.read().await;
    store
        .create_snapshot(&aggregate_id, &state_data)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_event_stats(
    event_store: tauri::State<'_, SharedEventStore>,
) -> Result<serde_json::Value, String> {
    let store = event_store.read().await;
    let total_count = store
        .get_event_count(None)
        .await
        .map_err(|e| e.to_string())?;

    // Get event type counts
    let event_type_counts = sqlx::query_as::<_, (String, i64)>(
        r#"
        SELECT event_type, COUNT(*) as count
        FROM events
        GROUP BY event_type
        ORDER BY count DESC
        "#,
    )
    .fetch_all(&store.pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut type_counts: HashMap<String, i64> = HashMap::new();
    for (event_type, count) in event_type_counts {
        type_counts.insert(event_type, count);
    }

    // Get recent events count (last 24 hours)
    let yesterday = Utc::now() - chrono::Duration::hours(24);
    let recent_count = sqlx::query_as::<_, (i64,)>(
        r#"
        SELECT COUNT(*) FROM events
        WHERE timestamp >= ?1
        "#,
    )
    .bind(yesterday.to_rfc3339())
    .fetch_one(&store.pool)
    .await
    .map_err(|e| e.to_string())?
    .0;

    Ok(serde_json::json!({
        "total_events": total_count,
        "events_last_24h": recent_count,
        "event_type_counts": type_counts,
    }))
}
