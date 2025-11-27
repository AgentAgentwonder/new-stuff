use super::types::*;
use anyhow::{anyhow, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use std::str::FromStr;

pub const ESCROW_PROGRAM_ID: &str = "EscrowProgram11111111111111111111111111111";

#[derive(Debug)]
pub struct EscrowStateMachine {
    pub escrow: Escrow,
}

impl EscrowStateMachine {
    pub fn new(escrow: Escrow) -> Self {
        Self { escrow }
    }

    pub fn can_transition(&self, target_state: &EscrowState) -> Result<bool> {
        let valid = match (&self.escrow.state, target_state) {
            (EscrowState::Created, EscrowState::Funded) => true,
            (EscrowState::Created, EscrowState::Cancelled) => true,

            (EscrowState::Funded, EscrowState::Confirmed) => true,
            (EscrowState::Funded, EscrowState::Disputed) => true,
            (EscrowState::Funded, EscrowState::Cancelled) => true,

            (EscrowState::Confirmed, EscrowState::Released) => true,
            (EscrowState::Confirmed, EscrowState::Disputed) => true,

            (EscrowState::Released, EscrowState::Completed) => true,

            (EscrowState::Disputed, EscrowState::Released) => true,
            (EscrowState::Disputed, EscrowState::Refunded) => true,
            (EscrowState::Disputed, EscrowState::Cancelled) => true,

            _ => false,
        };

        Ok(valid)
    }

    pub fn transition(&mut self, target_state: EscrowState) -> Result<()> {
        if !self.can_transition(&target_state)? {
            return Err(anyhow!(
                "Invalid state transition from {:?} to {:?}",
                self.escrow.state,
                target_state
            ));
        }

        self.escrow.state = target_state;
        Ok(())
    }

    pub fn is_timed_out(&self) -> bool {
        chrono::Utc::now() > self.escrow.timeout_at
    }

    pub fn requires_arbitration(&self) -> bool {
        self.escrow.state == EscrowState::Disputed
    }

    pub fn is_terminal_state(&self) -> bool {
        matches!(
            self.escrow.state,
            EscrowState::Completed | EscrowState::Cancelled | EscrowState::Refunded
        )
    }
}

pub struct EscrowSmartContract {
    rpc_client: Option<RpcClient>,
}

impl EscrowSmartContract {
    pub fn new(rpc_url: Option<String>) -> Self {
        let rpc_client = rpc_url.map(|url| RpcClient::new(url));
        Self { rpc_client }
    }

    pub async fn create_multisig_escrow(
        &self,
        escrow_id: &str,
        buyer: &str,
        seller: &str,
        amount: f64,
        token_mint: &str,
    ) -> Result<(String, String)> {
        if self.rpc_client.is_none() {
            return Ok((
                format!("mock_multisig_{}", escrow_id),
                format!("mock_escrow_pubkey_{}", escrow_id),
            ));
        }

        let buyer_pubkey = Pubkey::from_str(buyer)?;
        let seller_pubkey = Pubkey::from_str(seller)?;
        let token_mint_pubkey = Pubkey::from_str(token_mint)?;
        let program_id = Pubkey::from_str(ESCROW_PROGRAM_ID)?;

        let escrow_seed = format!("escrow:{}", escrow_id);
        let (escrow_pda, _bump) =
            Pubkey::find_program_address(&[escrow_seed.as_bytes()], &program_id);

        let multisig_address = format!("multisig_{}", escrow_id);
        let escrow_pubkey = escrow_pda.to_string();

        Ok((multisig_address, escrow_pubkey))
    }

    pub async fn fund_escrow(
        &self,
        escrow_pubkey: &str,
        from: &str,
        amount: f64,
        token_mint: &str,
    ) -> Result<String> {
        if self.rpc_client.is_none() {
            return Ok(format!("mock_tx_fund_{}", escrow_pubkey));
        }

        Ok(format!("tx_fund_{}", escrow_pubkey))
    }

    pub async fn release_funds(
        &self,
        escrow_pubkey: &str,
        to: &str,
        amount: f64,
    ) -> Result<String> {
        if self.rpc_client.is_none() {
            return Ok(format!("mock_tx_release_{}", escrow_pubkey));
        }

        Ok(format!("tx_release_{}", escrow_pubkey))
    }

    pub async fn refund_escrow(
        &self,
        escrow_pubkey: &str,
        to: &str,
        amount: f64,
    ) -> Result<String> {
        if self.rpc_client.is_none() {
            return Ok(format!("mock_tx_refund_{}", escrow_pubkey));
        }

        Ok(format!("tx_refund_{}", escrow_pubkey))
    }

    pub async fn dispute_escrow(&self, escrow_pubkey: &str, disputer: &str) -> Result<String> {
        if self.rpc_client.is_none() {
            return Ok(format!("mock_tx_dispute_{}", escrow_pubkey));
        }

        Ok(format!("tx_dispute_{}", escrow_pubkey))
    }

    pub async fn resolve_dispute(
        &self,
        escrow_pubkey: &str,
        arbitrator: &str,
        release_to: &str,
        amount: f64,
    ) -> Result<String> {
        if self.rpc_client.is_none() {
            return Ok(format!("mock_tx_resolve_{}", escrow_pubkey));
        }

        Ok(format!("tx_resolve_{}", escrow_pubkey))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_escrow() -> Escrow {
        Escrow {
            id: "test_escrow_1".to_string(),
            offer_id: "test_offer_1".to_string(),
            buyer: "buyer_address".to_string(),
            seller: "seller_address".to_string(),
            amount: 100.0,
            token_address: "token_address".to_string(),
            fiat_amount: 1000.0,
            fiat_currency: "USD".to_string(),
            state: EscrowState::Created,
            multisig_address: None,
            escrow_pubkey: None,
            created_at: Utc::now(),
            funded_at: None,
            released_at: None,
            timeout_at: Utc::now() + chrono::Duration::minutes(30),
            arbitrators: vec![],
            fee_rate: 0.01,
        }
    }

    #[test]
    fn test_state_machine_valid_transitions() {
        let mut machine = EscrowStateMachine::new(create_test_escrow());

        assert!(machine.can_transition(&EscrowState::Funded).unwrap());
        machine.transition(EscrowState::Funded).unwrap();
        assert_eq!(machine.escrow.state, EscrowState::Funded);

        assert!(machine.can_transition(&EscrowState::Confirmed).unwrap());
        machine.transition(EscrowState::Confirmed).unwrap();
        assert_eq!(machine.escrow.state, EscrowState::Confirmed);

        assert!(machine.can_transition(&EscrowState::Released).unwrap());
        machine.transition(EscrowState::Released).unwrap();
        assert_eq!(machine.escrow.state, EscrowState::Released);

        assert!(machine.can_transition(&EscrowState::Completed).unwrap());
        machine.transition(EscrowState::Completed).unwrap();
        assert_eq!(machine.escrow.state, EscrowState::Completed);
    }

    #[test]
    fn test_state_machine_invalid_transitions() {
        let mut machine = EscrowStateMachine::new(create_test_escrow());

        assert!(!machine.can_transition(&EscrowState::Released).unwrap());
        assert!(machine.transition(EscrowState::Released).is_err());

        machine.transition(EscrowState::Funded).unwrap();
        assert!(!machine.can_transition(&EscrowState::Completed).unwrap());
        assert!(machine.transition(EscrowState::Completed).is_err());
    }

    #[test]
    fn test_state_machine_dispute_flow() {
        let mut machine = EscrowStateMachine::new(create_test_escrow());

        machine.transition(EscrowState::Funded).unwrap();
        machine.transition(EscrowState::Disputed).unwrap();
        assert_eq!(machine.escrow.state, EscrowState::Disputed);
        assert!(machine.requires_arbitration());

        assert!(machine.can_transition(&EscrowState::Released).unwrap());
        assert!(machine.can_transition(&EscrowState::Refunded).unwrap());
    }

    #[test]
    fn test_state_machine_cancellation() {
        let mut machine = EscrowStateMachine::new(create_test_escrow());

        assert!(machine.can_transition(&EscrowState::Cancelled).unwrap());
        machine.transition(EscrowState::Cancelled).unwrap();
        assert_eq!(machine.escrow.state, EscrowState::Cancelled);
        assert!(machine.is_terminal_state());
    }

    #[test]
    fn test_state_machine_terminal_states() {
        let mut machine = EscrowStateMachine::new(create_test_escrow());

        machine.transition(EscrowState::Funded).unwrap();
        machine.transition(EscrowState::Confirmed).unwrap();
        machine.transition(EscrowState::Released).unwrap();
        machine.transition(EscrowState::Completed).unwrap();

        assert!(machine.is_terminal_state());
        assert!(!machine.can_transition(&EscrowState::Funded).unwrap());
    }

    #[test]
    fn test_timeout_detection() {
        let mut escrow = create_test_escrow();
        escrow.timeout_at = Utc::now() - chrono::Duration::minutes(10);

        let machine = EscrowStateMachine::new(escrow);
        assert!(machine.is_timed_out());
    }
}
