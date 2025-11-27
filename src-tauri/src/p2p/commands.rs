use super::{
    compliance::ComplianceChecker,
    database::P2PDatabase,
    escrow::{EscrowSmartContract, EscrowStateMachine},
    matching::LocalMatcher,
    types::*,
};
use crate::security::reputation::SharedReputationEngine;
use anyhow::Result;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

pub type SharedP2PDatabase = Arc<RwLock<P2PDatabase>>;

#[tauri::command]
pub async fn create_p2p_offer(
    request: CreateOfferRequest,
    db: State<'_, SharedP2PDatabase>,
    reputation: State<'_, SharedReputationEngine>,
) -> Result<P2POffer, String> {
    let reputation_guard = reputation.read().await;
    let creator_rep = reputation_guard
        .get_wallet_reputation(&request.creator)
        .await
        .ok();

    let checker = ComplianceChecker::new();
    let compliance = checker
        .check_offer(
            &P2POffer {
                id: String::new(),
                creator: request.creator.clone(),
                offer_type: request.offer_type.clone(),
                token_address: request.token_address.clone(),
                token_symbol: request.token_symbol.clone(),
                amount: request.amount,
                price: request.price,
                fiat_currency: request.fiat_currency.clone(),
                payment_methods: request.payment_methods.clone(),
                min_amount: request.min_amount,
                max_amount: request.max_amount,
                terms: request.terms.clone(),
                time_limit: request.time_limit,
                created_at: chrono::Utc::now(),
                expires_at: None,
                is_active: true,
                completed_trades: 0,
                reputation_required: request.reputation_required,
            },
            creator_rep.as_ref(),
        )
        .await
        .map_err(|e| e.to_string())?;

    if !compliance.passed {
        return Err(format!("Compliance check failed: {:?}", compliance.errors));
    }

    let db_guard = db.write().await;
    db_guard
        .create_offer(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_p2p_offer(
    offer_id: String,
    db: State<'_, SharedP2PDatabase>,
) -> Result<Option<P2POffer>, String> {
    let db_guard = db.read().await;
    db_guard
        .get_offer(&offer_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_p2p_offers(
    offer_type: Option<String>,
    token_address: Option<String>,
    active_only: bool,
    db: State<'_, SharedP2PDatabase>,
) -> Result<Vec<P2POffer>, String> {
    let db_guard = db.read().await;
    db_guard
        .list_offers(offer_type, token_address, active_only)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_offer_status(
    offer_id: String,
    is_active: bool,
    db: State<'_, SharedP2PDatabase>,
) -> Result<(), String> {
    let db_guard = db.write().await;
    db_guard
        .update_offer_status(&offer_id, is_active)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn match_p2p_offers(
    user_address: String,
    offers: Vec<P2POffer>,
    db: State<'_, SharedP2PDatabase>,
    reputation: State<'_, SharedReputationEngine>,
) -> Result<Vec<super::matching::TraderMatch>, String> {
    let db_guard = db.read().await;
    let reputation_guard = reputation.read().await;

    let user_profile = db_guard
        .get_or_create_trader_profile(&user_address)
        .await
        .map_err(|e| e.to_string())?;

    let user_reputation = reputation_guard
        .get_wallet_reputation(&user_address)
        .await
        .ok();

    let matcher = LocalMatcher::new()
        .with_payment_priority("Bank Transfer", 90)
        .with_payment_priority("PayPal", 80)
        .with_payment_priority("Cash", 70);

    let matches = matcher.match_offers(&offers, &user_profile, user_reputation.as_ref());

    Ok(matches)
}

#[tauri::command]
pub async fn create_p2p_escrow(
    request: CreateEscrowRequest,
    db: State<'_, SharedP2PDatabase>,
    reputation: State<'_, SharedReputationEngine>,
) -> Result<Escrow, String> {
    let reputation_guard = reputation.read().await;

    let buyer_rep = reputation_guard
        .get_wallet_reputation(&request.buyer)
        .await
        .ok();

    let seller_rep = reputation_guard
        .get_wallet_reputation(&request.seller)
        .await
        .ok();

    let checker = ComplianceChecker::new();
    let db_guard = db.read().await;

    let offer = db_guard
        .get_offer(&request.offer_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Offer not found".to_string())?;

    let escrow = Escrow {
        id: String::new(),
        offer_id: request.offer_id.clone(),
        buyer: request.buyer.clone(),
        seller: request.seller.clone(),
        amount: request.amount,
        token_address: offer.token_address.clone(),
        fiat_amount: request.fiat_amount,
        fiat_currency: offer.fiat_currency.clone(),
        state: EscrowState::Created,
        multisig_address: None,
        escrow_pubkey: None,
        created_at: chrono::Utc::now(),
        funded_at: None,
        released_at: None,
        timeout_at: chrono::Utc::now() + chrono::Duration::minutes(offer.time_limit as i64),
        arbitrators: vec![],
        fee_rate: 0.01,
    };

    let compliance = checker
        .check_escrow(&escrow, buyer_rep.as_ref(), seller_rep.as_ref())
        .await
        .map_err(|e| e.to_string())?;

    if !compliance.passed {
        return Err(format!("Compliance check failed: {:?}", compliance.errors));
    }

    drop(db_guard);
    let db_guard = db.write().await;
    db_guard
        .create_escrow(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_p2p_escrow(
    escrow_id: String,
    db: State<'_, SharedP2PDatabase>,
) -> Result<Option<Escrow>, String> {
    let db_guard = db.read().await;
    db_guard
        .get_escrow(&escrow_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_p2p_escrows(
    user_address: Option<String>,
    db: State<'_, SharedP2PDatabase>,
) -> Result<Vec<Escrow>, String> {
    let db_guard = db.read().await;
    db_guard
        .list_escrows(user_address)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fund_p2p_escrow(
    escrow_id: String,
    db: State<'_, SharedP2PDatabase>,
) -> Result<String, String> {
    let db_guard = db.read().await;
    let escrow = db_guard
        .get_escrow(&escrow_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Escrow not found".to_string())?;

    let mut state_machine = EscrowStateMachine::new(escrow.clone());
    state_machine
        .transition(EscrowState::Funded)
        .map_err(|e| e.to_string())?;

    let contract = EscrowSmartContract::new(None);
    let (multisig_address, escrow_pubkey) = contract
        .create_multisig_escrow(
            &escrow_id,
            &escrow.buyer,
            &escrow.seller,
            escrow.amount,
            &escrow.token_address,
        )
        .await
        .map_err(|e| e.to_string())?;

    let tx_signature = contract
        .fund_escrow(
            &escrow_pubkey,
            &escrow.buyer,
            escrow.amount,
            &escrow.token_address,
        )
        .await
        .map_err(|e| e.to_string())?;

    drop(db_guard);
    let db_guard = db.write().await;
    db_guard
        .update_escrow_state(
            &escrow_id,
            EscrowState::Funded,
            Some(multisig_address),
            Some(escrow_pubkey),
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(tx_signature)
}

#[tauri::command]
pub async fn confirm_payment_p2p(
    escrow_id: String,
    db: State<'_, SharedP2PDatabase>,
) -> Result<(), String> {
    let db_guard = db.read().await;
    let escrow = db_guard
        .get_escrow(&escrow_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Escrow not found".to_string())?;

    let mut state_machine = EscrowStateMachine::new(escrow);
    state_machine
        .transition(EscrowState::Confirmed)
        .map_err(|e| e.to_string())?;

    drop(db_guard);
    let db_guard = db.write().await;
    db_guard
        .update_escrow_state(&escrow_id, EscrowState::Confirmed, None, None)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn release_p2p_escrow(
    escrow_id: String,
    db: State<'_, SharedP2PDatabase>,
) -> Result<String, String> {
    let db_guard = db.read().await;
    let escrow = db_guard
        .get_escrow(&escrow_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Escrow not found".to_string())?;

    let mut state_machine = EscrowStateMachine::new(escrow.clone());
    state_machine
        .transition(EscrowState::Released)
        .map_err(|e| e.to_string())?;

    let contract = EscrowSmartContract::new(None);
    let tx_signature = contract
        .release_funds(
            escrow.escrow_pubkey.as_ref().unwrap(),
            &escrow.seller,
            escrow.amount,
        )
        .await
        .map_err(|e| e.to_string())?;

    drop(db_guard);
    let db_guard = db.write().await;
    db_guard
        .update_escrow_state(&escrow_id, EscrowState::Released, None, None)
        .await
        .map_err(|e| e.to_string())?;

    db_guard
        .update_trader_stats(&escrow.buyer, true, false, false, None)
        .await
        .map_err(|e| e.to_string())?;

    db_guard
        .update_trader_stats(&escrow.seller, true, false, false, None)
        .await
        .map_err(|e| e.to_string())?;

    Ok(tx_signature)
}

#[tauri::command]
pub async fn cancel_p2p_escrow(
    escrow_id: String,
    db: State<'_, SharedP2PDatabase>,
) -> Result<(), String> {
    let db_guard = db.read().await;
    let escrow = db_guard
        .get_escrow(&escrow_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Escrow not found".to_string())?;

    let mut state_machine = EscrowStateMachine::new(escrow.clone());
    state_machine
        .transition(EscrowState::Cancelled)
        .map_err(|e| e.to_string())?;

    drop(db_guard);
    let db_guard = db.write().await;
    db_guard
        .update_escrow_state(&escrow_id, EscrowState::Cancelled, None, None)
        .await
        .map_err(|e| e.to_string())?;

    db_guard
        .update_trader_stats(&escrow.buyer, false, true, false, None)
        .await
        .map_err(|e| e.to_string())?;

    db_guard
        .update_trader_stats(&escrow.seller, false, true, false, None)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn file_p2p_dispute(
    request: FileDisputeRequest,
    db: State<'_, SharedP2PDatabase>,
) -> Result<Dispute, String> {
    let db_guard = db.read().await;
    let escrow = db_guard
        .get_escrow(&request.escrow_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Escrow not found".to_string())?;

    let mut state_machine = EscrowStateMachine::new(escrow);
    state_machine
        .transition(EscrowState::Disputed)
        .map_err(|e| e.to_string())?;

    drop(db_guard);
    let db_guard = db.write().await;

    db_guard
        .update_escrow_state(&request.escrow_id, EscrowState::Disputed, None, None)
        .await
        .map_err(|e| e.to_string())?;

    db_guard
        .create_dispute(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_p2p_dispute(
    dispute_id: String,
    db: State<'_, SharedP2PDatabase>,
) -> Result<Option<Dispute>, String> {
    let db_guard = db.read().await;
    db_guard
        .get_dispute(&dispute_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn submit_dispute_evidence(
    request: SubmitEvidenceRequest,
    db: State<'_, SharedP2PDatabase>,
) -> Result<DisputeEvidence, String> {
    let db_guard = db.write().await;
    db_guard
        .submit_evidence(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn resolve_p2p_dispute(
    dispute_id: String,
    resolution: String,
    release_to: String,
    db: State<'_, SharedP2PDatabase>,
) -> Result<String, String> {
    let db_guard = db.read().await;
    let dispute = db_guard
        .get_dispute(&dispute_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Dispute not found".to_string())?;

    let escrow = db_guard
        .get_escrow(&dispute.escrow_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Escrow not found".to_string())?;

    let contract = EscrowSmartContract::new(None);
    let tx_signature = contract
        .resolve_dispute(
            escrow.escrow_pubkey.as_ref().unwrap(),
            "arbitrator_address",
            &release_to,
            escrow.amount,
        )
        .await
        .map_err(|e| e.to_string())?;

    drop(db_guard);
    let db_guard = db.write().await;

    db_guard
        .update_dispute_status(&dispute_id, DisputeStatus::Resolved, Some(resolution))
        .await
        .map_err(|e| e.to_string())?;

    db_guard
        .update_escrow_state(&escrow.id, EscrowState::Completed, None, None)
        .await
        .map_err(|e| e.to_string())?;

    let disputed = true;
    db_guard
        .update_trader_stats(&escrow.buyer, false, false, disputed, None)
        .await
        .map_err(|e| e.to_string())?;

    db_guard
        .update_trader_stats(&escrow.seller, false, false, disputed, None)
        .await
        .map_err(|e| e.to_string())?;

    Ok(tx_signature)
}

#[tauri::command]
pub async fn send_p2p_message(
    request: SendMessageRequest,
    db: State<'_, SharedP2PDatabase>,
) -> Result<ChatMessage, String> {
    let db_guard = db.write().await;
    db_guard
        .send_message(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_p2p_messages(
    escrow_id: String,
    db: State<'_, SharedP2PDatabase>,
) -> Result<Vec<ChatMessage>, String> {
    let db_guard = db.read().await;
    db_guard
        .get_messages(&escrow_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_trader_profile(
    address: String,
    db: State<'_, SharedP2PDatabase>,
) -> Result<TraderProfile, String> {
    let db_guard = db.read().await;
    db_guard
        .get_or_create_trader_profile(&address)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_p2p_compliance(
    offer_id: Option<String>,
    escrow_id: Option<String>,
    db: State<'_, SharedP2PDatabase>,
    reputation: State<'_, SharedReputationEngine>,
) -> Result<ComplianceCheck, String> {
    let checker = ComplianceChecker::new();
    let db_guard = db.read().await;
    let reputation_guard = reputation.read().await;

    if let Some(oid) = offer_id {
        let offer = db_guard
            .get_offer(&oid)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Offer not found".to_string())?;

        let creator_rep = reputation_guard
            .get_wallet_reputation(&offer.creator)
            .await
            .ok();

        checker
            .check_offer(&offer, creator_rep.as_ref())
            .await
            .map_err(|e| e.to_string())
    } else if let Some(eid) = escrow_id {
        let escrow = db_guard
            .get_escrow(&eid)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Escrow not found".to_string())?;

        let buyer_rep = reputation_guard
            .get_wallet_reputation(&escrow.buyer)
            .await
            .ok();

        let seller_rep = reputation_guard
            .get_wallet_reputation(&escrow.seller)
            .await
            .ok();

        checker
            .check_escrow(&escrow, buyer_rep.as_ref(), seller_rep.as_ref())
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Either offer_id or escrow_id must be provided".to_string())
    }
}

#[tauri::command]
pub async fn get_p2p_stats(db: State<'_, SharedP2PDatabase>) -> Result<P2PStats, String> {
    let db_guard = db.read().await;
    db_guard.get_stats().await.map_err(|e| e.to_string())
}
