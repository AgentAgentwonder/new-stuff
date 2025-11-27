use super::types::*;
use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use std::path::PathBuf;
use std::str::FromStr;
use uuid::Uuid;

pub struct P2PDatabase {
    pool: Pool<Sqlite>,
}

impl P2PDatabase {
    pub async fn new(db_path: PathBuf) -> Result<Self> {
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&db_url).await?;

        let db = Self { pool };
        db.initialize().await?;

        Ok(db)
    }

    async fn initialize(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS p2p_offers (
                id TEXT PRIMARY KEY,
                creator TEXT NOT NULL,
                offer_type TEXT NOT NULL,
                token_address TEXT NOT NULL,
                token_symbol TEXT NOT NULL,
                amount REAL NOT NULL,
                price REAL NOT NULL,
                fiat_currency TEXT NOT NULL,
                payment_methods TEXT NOT NULL,
                min_amount REAL,
                max_amount REAL,
                terms TEXT,
                time_limit INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                expires_at TEXT,
                is_active INTEGER NOT NULL DEFAULT 1,
                completed_trades INTEGER NOT NULL DEFAULT 0,
                reputation_required REAL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS p2p_escrows (
                id TEXT PRIMARY KEY,
                offer_id TEXT NOT NULL,
                buyer TEXT NOT NULL,
                seller TEXT NOT NULL,
                amount REAL NOT NULL,
                token_address TEXT NOT NULL,
                fiat_amount REAL NOT NULL,
                fiat_currency TEXT NOT NULL,
                state TEXT NOT NULL,
                multisig_address TEXT,
                escrow_pubkey TEXT,
                created_at TEXT NOT NULL,
                funded_at TEXT,
                released_at TEXT,
                timeout_at TEXT NOT NULL,
                arbitrators TEXT NOT NULL,
                fee_rate REAL NOT NULL DEFAULT 0.01,
                FOREIGN KEY (offer_id) REFERENCES p2p_offers(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS p2p_disputes (
                id TEXT PRIMARY KEY,
                escrow_id TEXT NOT NULL,
                filed_by TEXT NOT NULL,
                reason TEXT NOT NULL,
                description TEXT NOT NULL,
                status TEXT NOT NULL,
                arbitrator TEXT,
                created_at TEXT NOT NULL,
                resolved_at TEXT,
                resolution TEXT,
                FOREIGN KEY (escrow_id) REFERENCES p2p_escrows(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS p2p_dispute_evidence (
                id TEXT PRIMARY KEY,
                dispute_id TEXT NOT NULL,
                submitted_by TEXT NOT NULL,
                evidence_type TEXT NOT NULL,
                content TEXT NOT NULL,
                attachments TEXT NOT NULL,
                submitted_at TEXT NOT NULL,
                FOREIGN KEY (dispute_id) REFERENCES p2p_disputes(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS p2p_dispute_votes (
                id TEXT PRIMARY KEY,
                dispute_id TEXT NOT NULL,
                voter TEXT NOT NULL,
                vote TEXT NOT NULL,
                comment TEXT,
                voted_at TEXT NOT NULL,
                FOREIGN KEY (dispute_id) REFERENCES p2p_disputes(id),
                UNIQUE(dispute_id, voter)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS p2p_chat_messages (
                id TEXT PRIMARY KEY,
                escrow_id TEXT NOT NULL,
                sender TEXT NOT NULL,
                recipient TEXT NOT NULL,
                message TEXT NOT NULL,
                encrypted INTEGER NOT NULL DEFAULT 0,
                timestamp TEXT NOT NULL,
                read INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (escrow_id) REFERENCES p2p_escrows(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS p2p_trader_profiles (
                address TEXT PRIMARY KEY,
                username TEXT,
                reputation_score REAL NOT NULL DEFAULT 50.0,
                total_trades INTEGER NOT NULL DEFAULT 0,
                successful_trades INTEGER NOT NULL DEFAULT 0,
                cancelled_trades INTEGER NOT NULL DEFAULT 0,
                disputed_trades INTEGER NOT NULL DEFAULT 0,
                avg_completion_time INTEGER NOT NULL DEFAULT 0,
                first_trade_at TEXT,
                last_trade_at TEXT,
                verified INTEGER NOT NULL DEFAULT 0,
                verification_level INTEGER NOT NULL DEFAULT 0
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_offers_creator ON p2p_offers(creator);
            CREATE INDEX IF NOT EXISTS idx_offers_active ON p2p_offers(is_active);
            CREATE INDEX IF NOT EXISTS idx_escrows_buyer ON p2p_escrows(buyer);
            CREATE INDEX IF NOT EXISTS idx_escrows_seller ON p2p_escrows(seller);
            CREATE INDEX IF NOT EXISTS idx_escrows_state ON p2p_escrows(state);
            CREATE INDEX IF NOT EXISTS idx_disputes_escrow ON p2p_disputes(escrow_id);
            CREATE INDEX IF NOT EXISTS idx_disputes_status ON p2p_disputes(status);
            CREATE INDEX IF NOT EXISTS idx_messages_escrow ON p2p_chat_messages(escrow_id);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_offer(&self, request: CreateOfferRequest) -> Result<P2POffer> {
        let offer = P2POffer {
            id: format!("offer_{}", Uuid::new_v4()),
            creator: request.creator,
            offer_type: request.offer_type,
            token_address: request.token_address,
            token_symbol: request.token_symbol,
            amount: request.amount,
            price: request.price,
            fiat_currency: request.fiat_currency,
            payment_methods: request.payment_methods.clone(),
            min_amount: request.min_amount,
            max_amount: request.max_amount,
            terms: request.terms,
            time_limit: request.time_limit,
            created_at: Utc::now(),
            expires_at: None,
            is_active: true,
            completed_trades: 0,
            reputation_required: request.reputation_required,
        };

        let payment_methods_json = serde_json::to_string(&request.payment_methods)?;

        sqlx::query(
            r#"
            INSERT INTO p2p_offers (
                id, creator, offer_type, token_address, token_symbol, amount, price, 
                fiat_currency, payment_methods, min_amount, max_amount, terms, time_limit,
                created_at, expires_at, is_active, completed_trades, reputation_required
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)
            "#,
        )
        .bind(&offer.id)
        .bind(&offer.creator)
        .bind(offer.offer_type.to_string())
        .bind(&offer.token_address)
        .bind(&offer.token_symbol)
        .bind(offer.amount)
        .bind(offer.price)
        .bind(&offer.fiat_currency)
        .bind(&payment_methods_json)
        .bind(offer.min_amount)
        .bind(offer.max_amount)
        .bind(&offer.terms)
        .bind(offer.time_limit)
        .bind(offer.created_at.to_rfc3339())
        .bind(offer.expires_at.map(|dt| dt.to_rfc3339()))
        .bind(if offer.is_active { 1 } else { 0 })
        .bind(offer.completed_trades)
        .bind(offer.reputation_required)
        .execute(&self.pool)
        .await?;

        Ok(offer)
    }

    pub async fn get_offer(&self, offer_id: &str) -> Result<Option<P2POffer>> {
        let row = sqlx::query(
            r#"
            SELECT * FROM p2p_offers WHERE id = ?1
            "#,
        )
        .bind(offer_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(self.row_to_offer(row)?))
        } else {
            Ok(None)
        }
    }

    pub async fn list_offers(
        &self,
        offer_type: Option<String>,
        token_address: Option<String>,
        active_only: bool,
    ) -> Result<Vec<P2POffer>> {
        let mut query = String::from("SELECT * FROM p2p_offers WHERE 1=1");
        let mut conditions = Vec::<String>::new();

        if active_only {
            conditions.push("is_active = 1".to_string());
        }

        if let Some(otype) = offer_type {
            conditions.push(format!("offer_type = '{}'", otype));
        }

        if let Some(token) = token_address {
            conditions.push(format!("token_address = '{}'", token));
        }

        if !conditions.is_empty() {
            query.push_str(&format!(" AND {}", conditions.join(" AND ")));
        }

        query.push_str(" ORDER BY created_at DESC");

        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;

        let mut offers = Vec::new();
        for row in rows {
            offers.push(self.row_to_offer(row)?);
        }

        Ok(offers)
    }

    fn row_to_offer(&self, row: sqlx::sqlite::SqliteRow) -> Result<P2POffer> {
        let payment_methods_json: String = row.try_get("payment_methods")?;
        let payment_methods: Vec<String> = serde_json::from_str(&payment_methods_json)?;

        Ok(P2POffer {
            id: row.try_get("id")?,
            creator: row.try_get("creator")?,
            offer_type: OfferType::from_str(&row.try_get::<String, _>("offer_type")?)?,
            token_address: row.try_get("token_address")?,
            token_symbol: row.try_get("token_symbol")?,
            amount: row.try_get("amount")?,
            price: row.try_get("price")?,
            fiat_currency: row.try_get("fiat_currency")?,
            payment_methods,
            min_amount: row.try_get("min_amount")?,
            max_amount: row.try_get("max_amount")?,
            terms: row.try_get("terms")?,
            time_limit: row.try_get("time_limit")?,
            created_at: DateTime::parse_from_rfc3339(&row.try_get::<String, _>("created_at")?)
                .map(|dt| dt.with_timezone(&Utc))?,
            expires_at: row
                .try_get::<Option<String>, _>("expires_at")?
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            is_active: row.try_get::<i64, _>("is_active")? != 0,
            completed_trades: row.try_get("completed_trades")?,
            reputation_required: row.try_get("reputation_required")?,
        })
    }

    pub async fn update_offer_status(&self, offer_id: &str, is_active: bool) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE p2p_offers SET is_active = ?1 WHERE id = ?2
            "#,
        )
        .bind(if is_active { 1 } else { 0 })
        .bind(offer_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_escrow(&self, request: CreateEscrowRequest) -> Result<Escrow> {
        let offer = self
            .get_offer(&request.offer_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Offer not found"))?;

        let escrow = Escrow {
            id: format!("escrow_{}", Uuid::new_v4()),
            offer_id: request.offer_id,
            buyer: request.buyer,
            seller: request.seller,
            amount: request.amount,
            token_address: offer.token_address,
            fiat_amount: request.fiat_amount,
            fiat_currency: offer.fiat_currency,
            state: EscrowState::Created,
            multisig_address: None,
            escrow_pubkey: None,
            created_at: Utc::now(),
            funded_at: None,
            released_at: None,
            timeout_at: Utc::now() + chrono::Duration::minutes(offer.time_limit as i64),
            arbitrators: vec![],
            fee_rate: 0.01,
        };

        let arbitrators_json = serde_json::to_string(&escrow.arbitrators)?;

        sqlx::query(
            r#"
            INSERT INTO p2p_escrows (
                id, offer_id, buyer, seller, amount, token_address, fiat_amount, fiat_currency,
                state, multisig_address, escrow_pubkey, created_at, funded_at, released_at,
                timeout_at, arbitrators, fee_rate
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
            "#,
        )
        .bind(&escrow.id)
        .bind(&escrow.offer_id)
        .bind(&escrow.buyer)
        .bind(&escrow.seller)
        .bind(escrow.amount)
        .bind(&escrow.token_address)
        .bind(escrow.fiat_amount)
        .bind(&escrow.fiat_currency)
        .bind(escrow.state.to_string())
        .bind(&escrow.multisig_address)
        .bind(&escrow.escrow_pubkey)
        .bind(escrow.created_at.to_rfc3339())
        .bind(escrow.funded_at.map(|dt| dt.to_rfc3339()))
        .bind(escrow.released_at.map(|dt| dt.to_rfc3339()))
        .bind(escrow.timeout_at.to_rfc3339())
        .bind(&arbitrators_json)
        .bind(escrow.fee_rate)
        .execute(&self.pool)
        .await?;

        Ok(escrow)
    }

    pub async fn get_escrow(&self, escrow_id: &str) -> Result<Option<Escrow>> {
        let row = sqlx::query(
            r#"
            SELECT * FROM p2p_escrows WHERE id = ?1
            "#,
        )
        .bind(escrow_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(self.row_to_escrow(row)?))
        } else {
            Ok(None)
        }
    }

    pub async fn list_escrows(&self, user_address: Option<String>) -> Result<Vec<Escrow>> {
        let query = if let Some(addr) = user_address {
            format!("SELECT * FROM p2p_escrows WHERE buyer = '{}' OR seller = '{}' ORDER BY created_at DESC", addr, addr)
        } else {
            "SELECT * FROM p2p_escrows ORDER BY created_at DESC".to_string()
        };

        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;

        let mut escrows = Vec::new();
        for row in rows {
            escrows.push(self.row_to_escrow(row)?);
        }

        Ok(escrows)
    }

    fn row_to_escrow(&self, row: sqlx::sqlite::SqliteRow) -> Result<Escrow> {
        let arbitrators_json: String = row.try_get("arbitrators")?;
        let arbitrators: Vec<String> = serde_json::from_str(&arbitrators_json)?;

        Ok(Escrow {
            id: row.try_get("id")?,
            offer_id: row.try_get("offer_id")?,
            buyer: row.try_get("buyer")?,
            seller: row.try_get("seller")?,
            amount: row.try_get("amount")?,
            token_address: row.try_get("token_address")?,
            fiat_amount: row.try_get("fiat_amount")?,
            fiat_currency: row.try_get("fiat_currency")?,
            state: EscrowState::from_str(&row.try_get::<String, _>("state")?)?,
            multisig_address: row.try_get("multisig_address")?,
            escrow_pubkey: row.try_get("escrow_pubkey")?,
            created_at: DateTime::parse_from_rfc3339(&row.try_get::<String, _>("created_at")?)
                .map(|dt| dt.with_timezone(&Utc))?,
            funded_at: row
                .try_get::<Option<String>, _>("funded_at")?
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            released_at: row
                .try_get::<Option<String>, _>("released_at")?
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            timeout_at: DateTime::parse_from_rfc3339(&row.try_get::<String, _>("timeout_at")?)
                .map(|dt| dt.with_timezone(&Utc))?,
            arbitrators,
            fee_rate: row.try_get("fee_rate")?,
        })
    }

    pub async fn update_escrow_state(
        &self,
        escrow_id: &str,
        state: EscrowState,
        multisig_address: Option<String>,
        escrow_pubkey: Option<String>,
    ) -> Result<()> {
        let mut query = String::from("UPDATE p2p_escrows SET state = ?1");
        let mut params = vec![state.to_string()];

        if state == EscrowState::Funded {
            query.push_str(", funded_at = ?2");
            params.push(Utc::now().to_rfc3339());
        } else if state == EscrowState::Released || state == EscrowState::Completed {
            query.push_str(", released_at = ?2");
            params.push(Utc::now().to_rfc3339());
        }

        if let Some(addr) = multisig_address {
            query.push_str(&format!(", multisig_address = '{}'", addr));
        }

        if let Some(pubkey) = escrow_pubkey {
            query.push_str(&format!(", escrow_pubkey = '{}'", pubkey));
        }

        query.push_str(&format!(" WHERE id = '{}'", escrow_id));

        sqlx::query(&query).execute(&self.pool).await?;

        Ok(())
    }

    pub async fn create_dispute(&self, request: FileDisputeRequest) -> Result<Dispute> {
        let dispute = Dispute {
            id: format!("dispute_{}", Uuid::new_v4()),
            escrow_id: request.escrow_id,
            filed_by: request.filed_by,
            reason: request.reason,
            description: request.description,
            evidence: vec![],
            status: DisputeStatus::Open,
            arbitrator: None,
            created_at: Utc::now(),
            resolved_at: None,
            resolution: None,
            votes: vec![],
        };

        sqlx::query(
            r#"
            INSERT INTO p2p_disputes (
                id, escrow_id, filed_by, reason, description, status, arbitrator,
                created_at, resolved_at, resolution
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
        )
        .bind(&dispute.id)
        .bind(&dispute.escrow_id)
        .bind(&dispute.filed_by)
        .bind(&dispute.reason)
        .bind(&dispute.description)
        .bind(dispute.status.to_string())
        .bind(&dispute.arbitrator)
        .bind(dispute.created_at.to_rfc3339())
        .bind(dispute.resolved_at.map(|dt| dt.to_rfc3339()))
        .bind(&dispute.resolution)
        .execute(&self.pool)
        .await?;

        Ok(dispute)
    }

    pub async fn get_dispute(&self, dispute_id: &str) -> Result<Option<Dispute>> {
        let row = sqlx::query(
            r#"
            SELECT * FROM p2p_disputes WHERE id = ?1
            "#,
        )
        .bind(dispute_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let dispute_id: String = row.try_get("id")?;
            let evidence = self.get_dispute_evidence(&dispute_id).await?;
            let votes = self.get_dispute_votes(&dispute_id).await?;

            Ok(Some(Dispute {
                id: dispute_id,
                escrow_id: row.try_get("escrow_id")?,
                filed_by: row.try_get("filed_by")?,
                reason: row.try_get("reason")?,
                description: row.try_get("description")?,
                evidence,
                status: DisputeStatus::from_str(&row.try_get::<String, _>("status")?)?,
                arbitrator: row.try_get("arbitrator")?,
                created_at: DateTime::parse_from_rfc3339(&row.try_get::<String, _>("created_at")?)
                    .map(|dt| dt.with_timezone(&Utc))?,
                resolved_at: row
                    .try_get::<Option<String>, _>("resolved_at")?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                resolution: row.try_get("resolution")?,
                votes,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn submit_evidence(&self, request: SubmitEvidenceRequest) -> Result<DisputeEvidence> {
        let evidence = DisputeEvidence {
            id: format!("evidence_{}", Uuid::new_v4()),
            dispute_id: request.dispute_id,
            submitted_by: request.submitted_by,
            evidence_type: request.evidence_type,
            content: request.content,
            attachments: request.attachments.clone(),
            submitted_at: Utc::now(),
        };

        let attachments_json = serde_json::to_string(&request.attachments)?;

        sqlx::query(
            r#"
            INSERT INTO p2p_dispute_evidence (
                id, dispute_id, submitted_by, evidence_type, content, attachments, submitted_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
        )
        .bind(&evidence.id)
        .bind(&evidence.dispute_id)
        .bind(&evidence.submitted_by)
        .bind(&evidence.evidence_type)
        .bind(&evidence.content)
        .bind(&attachments_json)
        .bind(evidence.submitted_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(evidence)
    }

    pub async fn get_dispute_evidence(&self, dispute_id: &str) -> Result<Vec<DisputeEvidence>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM p2p_dispute_evidence WHERE dispute_id = ?1 ORDER BY submitted_at ASC
            "#,
        )
        .bind(dispute_id)
        .fetch_all(&self.pool)
        .await?;

        let mut evidence_list = Vec::new();
        for row in rows {
            let attachments_json: String = row.try_get("attachments")?;
            let attachments: Vec<String> = serde_json::from_str(&attachments_json)?;

            evidence_list.push(DisputeEvidence {
                id: row.try_get("id")?,
                dispute_id: row.try_get("dispute_id")?,
                submitted_by: row.try_get("submitted_by")?,
                evidence_type: row.try_get("evidence_type")?,
                content: row.try_get("content")?,
                attachments,
                submitted_at: DateTime::parse_from_rfc3339(
                    &row.try_get::<String, _>("submitted_at")?,
                )
                .map(|dt| dt.with_timezone(&Utc))?,
            });
        }

        Ok(evidence_list)
    }

    pub async fn get_dispute_votes(&self, dispute_id: &str) -> Result<Vec<DisputeVote>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM p2p_dispute_votes WHERE dispute_id = ?1 ORDER BY voted_at ASC
            "#,
        )
        .bind(dispute_id)
        .fetch_all(&self.pool)
        .await?;

        let mut votes = Vec::new();
        for row in rows {
            votes.push(DisputeVote {
                id: row.try_get("id")?,
                dispute_id: row.try_get("dispute_id")?,
                voter: row.try_get("voter")?,
                vote: row.try_get("vote")?,
                comment: row.try_get("comment")?,
                voted_at: DateTime::parse_from_rfc3339(&row.try_get::<String, _>("voted_at")?)
                    .map(|dt| dt.with_timezone(&Utc))?,
            });
        }

        Ok(votes)
    }

    pub async fn update_dispute_status(
        &self,
        dispute_id: &str,
        status: DisputeStatus,
        resolution: Option<String>,
    ) -> Result<()> {
        let mut query = String::from("UPDATE p2p_disputes SET status = ?1");
        if status == DisputeStatus::Resolved {
            query.push_str(", resolved_at = ?2, resolution = ?3");
        }
        query.push_str(" WHERE id = ?4");

        if status == DisputeStatus::Resolved {
            sqlx::query(&query)
                .bind(status.to_string())
                .bind(Utc::now().to_rfc3339())
                .bind(resolution)
                .bind(dispute_id)
                .execute(&self.pool)
                .await?;
        } else {
            sqlx::query(&query)
                .bind(status.to_string())
                .bind(dispute_id)
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }

    pub async fn send_message(&self, request: SendMessageRequest) -> Result<ChatMessage> {
        let message = ChatMessage {
            id: format!("msg_{}", Uuid::new_v4()),
            escrow_id: request.escrow_id,
            sender: request.sender,
            recipient: request.recipient,
            message: request.message,
            encrypted: false,
            timestamp: Utc::now(),
            read: false,
        };

        sqlx::query(
            r#"
            INSERT INTO p2p_chat_messages (
                id, escrow_id, sender, recipient, message, encrypted, timestamp, read
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
        )
        .bind(&message.id)
        .bind(&message.escrow_id)
        .bind(&message.sender)
        .bind(&message.recipient)
        .bind(&message.message)
        .bind(if message.encrypted { 1 } else { 0 })
        .bind(message.timestamp.to_rfc3339())
        .bind(if message.read { 1 } else { 0 })
        .execute(&self.pool)
        .await?;

        Ok(message)
    }

    pub async fn get_messages(&self, escrow_id: &str) -> Result<Vec<ChatMessage>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM p2p_chat_messages WHERE escrow_id = ?1 ORDER BY timestamp ASC
            "#,
        )
        .bind(escrow_id)
        .fetch_all(&self.pool)
        .await?;

        let mut messages = Vec::new();
        for row in rows {
            messages.push(ChatMessage {
                id: row.try_get("id")?,
                escrow_id: row.try_get("escrow_id")?,
                sender: row.try_get("sender")?,
                recipient: row.try_get("recipient")?,
                message: row.try_get("message")?,
                encrypted: row.try_get::<i64, _>("encrypted")? != 0,
                timestamp: DateTime::parse_from_rfc3339(&row.try_get::<String, _>("timestamp")?)
                    .map(|dt| dt.with_timezone(&Utc))?,
                read: row.try_get::<i64, _>("read")? != 0,
            });
        }

        Ok(messages)
    }

    pub async fn get_or_create_trader_profile(&self, address: &str) -> Result<TraderProfile> {
        let row = sqlx::query(
            r#"
            SELECT * FROM p2p_trader_profiles WHERE address = ?1
            "#,
        )
        .bind(address)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(TraderProfile {
                address: row.try_get("address")?,
                username: row.try_get("username")?,
                reputation_score: row.try_get("reputation_score")?,
                total_trades: row.try_get("total_trades")?,
                successful_trades: row.try_get("successful_trades")?,
                cancelled_trades: row.try_get("cancelled_trades")?,
                disputed_trades: row.try_get("disputed_trades")?,
                avg_completion_time: row.try_get("avg_completion_time")?,
                first_trade_at: row
                    .try_get::<Option<String>, _>("first_trade_at")?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                last_trade_at: row
                    .try_get::<Option<String>, _>("last_trade_at")?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                verified: row.try_get::<i64, _>("verified")? != 0,
                verification_level: row.try_get("verification_level")?,
            })
        } else {
            let profile = TraderProfile {
                address: address.to_string(),
                username: None,
                reputation_score: 50.0,
                total_trades: 0,
                successful_trades: 0,
                cancelled_trades: 0,
                disputed_trades: 0,
                avg_completion_time: 0,
                first_trade_at: None,
                last_trade_at: None,
                verified: false,
                verification_level: 0,
            };

            sqlx::query(
                r#"
                INSERT INTO p2p_trader_profiles (address, reputation_score)
                VALUES (?1, ?2)
                "#,
            )
            .bind(address)
            .bind(profile.reputation_score)
            .execute(&self.pool)
            .await?;

            Ok(profile)
        }
    }

    pub async fn update_trader_stats(
        &self,
        address: &str,
        successful: bool,
        cancelled: bool,
        disputed: bool,
        completion_time: Option<i64>,
    ) -> Result<()> {
        let profile = self.get_or_create_trader_profile(address).await?;

        let new_total = profile.total_trades + 1;
        let new_successful = if successful {
            profile.successful_trades + 1
        } else {
            profile.successful_trades
        };
        let new_cancelled = if cancelled {
            profile.cancelled_trades + 1
        } else {
            profile.cancelled_trades
        };
        let new_disputed = if disputed {
            profile.disputed_trades + 1
        } else {
            profile.disputed_trades
        };

        let success_rate = new_successful as f64 / new_total as f64;
        let dispute_rate = new_disputed as f64 / new_total as f64;
        let cancel_rate = new_cancelled as f64 / new_total as f64;

        let reputation_score =
            50.0 + (success_rate * 40.0) - (dispute_rate * 30.0) - (cancel_rate * 20.0);
        let reputation_score = reputation_score.max(0.0).min(100.0);

        let now = Utc::now().to_rfc3339();
        let first_trade = profile
            .first_trade_at
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| now.clone());

        sqlx::query(
            r#"
            UPDATE p2p_trader_profiles
            SET reputation_score = ?1, total_trades = ?2, successful_trades = ?3,
                cancelled_trades = ?4, disputed_trades = ?5, first_trade_at = ?6,
                last_trade_at = ?7
            WHERE address = ?8
            "#,
        )
        .bind(reputation_score)
        .bind(new_total)
        .bind(new_successful)
        .bind(new_cancelled)
        .bind(new_disputed)
        .bind(&first_trade)
        .bind(&now)
        .bind(address)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_stats(&self) -> Result<P2PStats> {
        let total_offers: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM p2p_offers")
            .fetch_one(&self.pool)
            .await?;

        let active_offers: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM p2p_offers WHERE is_active = 1")
                .fetch_one(&self.pool)
                .await?;

        let total_escrows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM p2p_escrows")
            .fetch_one(&self.pool)
            .await?;

        let active_escrows: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM p2p_escrows WHERE state IN ('created', 'funded', 'confirmed')",
        )
        .fetch_one(&self.pool)
        .await?;

        let completed_escrows: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM p2p_escrows WHERE state = 'completed'")
                .fetch_one(&self.pool)
                .await?;

        let total_disputes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM p2p_disputes")
            .fetch_one(&self.pool)
            .await?;

        let open_disputes: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM p2p_disputes WHERE status IN ('open', 'under_review', 'evidence')"
        )
        .fetch_one(&self.pool)
        .await?;

        let total_volume: Option<f64> = sqlx::query_scalar(
            "SELECT SUM(fiat_amount) FROM p2p_escrows WHERE state = 'completed'",
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(P2PStats {
            total_offers,
            active_offers,
            total_escrows,
            active_escrows,
            completed_escrows,
            total_disputes,
            open_disputes,
            total_volume: total_volume.unwrap_or(0.0),
            avg_completion_time: 0,
        })
    }
}
