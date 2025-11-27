// Security Enhancements Module
// Hardware wallets, transaction simulation, and audit logging

pub mod types;
pub mod audit_logger;
pub mod tx_simulator;
pub mod ledger;

// Export existing security modules
pub mod keystore;
pub mod audit;
pub mod activity_log;
pub mod reputation;

pub use types::*;
pub use audit_logger::AuditLogger;
pub use tx_simulator::TxSimulator;
pub use ledger::LedgerManager;
