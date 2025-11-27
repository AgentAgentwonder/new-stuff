# Multi-Chain Support Implementation Guide

## Overview

Eclipse Market Pro now supports multi-chain operations across Solana, Ethereum, Base, Polygon, and Arbitrum with cross-chain bridging capabilities.

## Features

### 1. Chain Abstraction Layer

The chain abstraction layer provides a unified interface for interacting with multiple blockchains:

- **Supported Chains:**
  - Solana
  - Ethereum
  - Base
  - Polygon
  - Arbitrum

- **Core Functionality:**
  - Get balances across chains
  - Estimate gas/transaction fees
  - Build and submit transactions
  - Monitor chain health and status

### 2. Bridge Integrations

Cross-chain asset transfers are supported through multiple bridge providers:

- **Wormhole:** Multi-chain bridge with guardian network (slower but more secure)
- **AllBridge:** Fast liquidity pools across chains with flat fees
- **Synapse:** Low-fee optimistic bridge protocol (fastest option)

### 3. Cross-Chain Portfolio Aggregation

View consolidated portfolio value across all connected chains:

- Real-time balance aggregation
- Per-chain breakdown
- Per-wallet analysis
- Combined USD value tracking

## Usage

### Frontend Components

#### ChainSelector

Add chain selection to your UI:

```tsx
import { ChainSelector } from '../components/chains/ChainSelector';

<ChainSelector onChainChange={(chainId) => console.log('Active chain:', chainId)} />
```

#### BridgeInterface

Enable cross-chain transfers:

```tsx
import { BridgeInterface } from '../components/chains/BridgeInterface';

<BridgeInterface walletAddress={userWalletAddress} />
```

#### CrossChainPortfolioSummary

Display aggregated portfolio:

```tsx
import { CrossChainPortfolioSummary } from '../components/chains/CrossChainPortfolioSummary';

const walletMap = {
  solana: 'SolanaWalletAddress...',
  ethereum: '0xEthereumWalletAddress...',
  base: '0xBaseWalletAddress...',
};

<CrossChainPortfolioSummary walletMap={walletMap} />
```

### Backend Commands

#### Chain Management

```rust
// Get active chain
let active_chain = invoke('chain_get_active').await?;

// Set active chain
invoke('chain_set_active', json!({ "chain_id": "ethereum" })).await?;

// List all chains
let chains = invoke('chain_list_chains').await?;

// List enabled chains only
let enabled = invoke('chain_list_enabled').await?;

// Get balance for a wallet on a specific chain
let balance = invoke('chain_get_balance', json!({
    "wallet_address": "0x...",
    "chain_id": "ethereum"
})).await?;

// Get fee estimates
let fees = invoke('chain_get_fee_estimate', json!({
    "wallet_address": "0x...",
    "chain_id": "polygon"
})).await?;

// Get chain status
let status = invoke('chain_get_status', json!({
    "chain_id": "base"
})).await?;

// Get cross-chain portfolio
let portfolio = invoke('chain_get_cross_chain_portfolio', json!({
    "wallet_addresses": {
        "solana": "SolanaAddress...",
        "ethereum": "0xEthAddress..."
    }
})).await?;
```

#### Bridge Operations

```rust
// Get bridge quotes from all providers
let quotes = invoke('bridge_get_quote', json!({
    "request": {
        "from_chain": "solana",
        "to_chain": "ethereum",
        "token_address": "TokenMintAddress",
        "amount": 100.0,
        "recipient_address": "0xRecipient..."
    }
})).await?;

// Get quote from specific provider
let quotes = invoke('bridge_get_quote', json!({
    "request": { /* ... */ },
    "provider": "wormhole"
})).await?;

// Create bridge transaction
let tx = invoke('bridge_create_transaction', json!({
    "request": {
        "provider": "wormhole",
        "from_chain": "solana",
        "to_chain": "ethereum",
        "token_address": "TokenMint",
        "amount": 100.0,
        "recipient_address": "0xRecipient...",
        "sender_address": "SolanaAddress..."
    }
})).await?;

// List all bridge transactions
let transactions = invoke('bridge_list_transactions').await?;

// List transactions by status
let pending = invoke('bridge_list_transactions_by_status', json!({
    "status": "pending"
})).await?;

// Poll bridge transaction status
let status = invoke('bridge_poll_status', json!({
    "transaction_id": "tx_id",
    "provider": "wormhole"
})).await?;
```

## Architecture

### Backend Structure

```
src-tauri/src/
├── chains/
│   ├── mod.rs           # Chain manager and types
│   ├── types.rs         # Common types and traits
│   ├── solana.rs        # Solana adapter
│   ├── ethereum.rs      # Ethereum/EVM adapter
│   ├── base.rs          # Base chain adapter
│   ├── polygon.rs       # Polygon adapter
│   ├── arbitrum.rs      # Arbitrum adapter
│   └── commands.rs      # Tauri commands
└── bridges/
    ├── mod.rs           # Bridge manager
    ├── types.rs         # Bridge traits
    ├── wormhole.rs      # Wormhole integration
    ├── allbridge.rs     # AllBridge integration
    ├── synapse.rs       # Synapse integration
    └── commands.rs      # Bridge commands
```

### Frontend Structure

```
src/
├── components/chains/
│   ├── ChainSelector.tsx              # Chain selection UI
│   ├── BridgeInterface.tsx            # Bridge transfer UI
│   └── CrossChainPortfolioSummary.tsx # Portfolio aggregation
└── pages/
    └── MultiChain.tsx                 # Main multi-chain dashboard
```

## Chain Adapter Interface

All chain adapters implement the `ChainAdapter` trait:

```rust
#[async_trait]
pub trait ChainAdapter: Send + Sync + Debug {
    async fn get_balance(&self, wallet: &WalletInfo) -> Result<ChainBalance, String>;
    async fn get_fee_estimate(&self, wallet: &WalletInfo) -> Result<ChainFeeEstimate, String>;
    async fn build_transfer(&self, wallet: &WalletInfo, to: &str, amount: f64) -> Result<ChainTransaction, String>;
    async fn quote_swap(&self, request: ChainQuoteRequest) -> Result<ChainQuoteResponse, String>;
    async fn submit_transaction(&self, tx: ChainTransaction) -> Result<String, String>;
    async fn get_status(&self) -> Result<ChainStatus, String>;
}
```

## Bridge Adapter Interface

All bridge adapters implement the `BridgeAdapter` trait:

```rust
#[async_trait]
pub trait BridgeAdapter: Send + Sync {
    async fn quote(&self, request: &BridgeQuoteRequest) -> Result<BridgeQuote, String>;
    async fn prepare_transaction(&self, request: &BridgeTransactionRequest) -> Result<BridgeTransaction, String>;
    async fn poll_status(&self, transaction_id: &str) -> Result<BridgeTransactionStatus, String>;
}
```

## Testing

### Unit Tests

Run chain and bridge tests:

```bash
cargo test chains_test
cargo test bridge_tests
```

### Frontend Tests

```bash
npm test src/__tests__/chains.test.ts
```

## Best Practices

1. **Always verify recipient addresses** match the destination chain format
2. **Test with small amounts** first on new bridge routes
3. **Consider peak hours** - gas fees vary significantly
4. **Monitor transaction status** - bridge times vary by provider
5. **Keep RPC endpoints updated** for optimal performance

## Security Considerations

- Private keys never leave the user's device
- All transactions require explicit user approval
- Bridge providers are non-custodial
- Chain RPCs can be customized for privacy

## Configuration

### Custom RPC Endpoints

Update chain configurations in Settings > Multi-Chain:

```typescript
{
  chain_id: "ethereum",
  rpc_url: "https://your-custom-rpc.com",
  explorer_url: "https://etherscan.io",
  native_token: "ETH",
  enabled: true
}
```

## Troubleshooting

### Bridge Transaction Stuck

1. Check transaction status: `bridge_poll_status`
2. Verify source transaction confirmed on origin chain
3. Check bridge provider status page
4. Contact bridge support if stuck > 1 hour

### Balance Not Updating

1. Verify RPC endpoint is responsive
2. Check chain status: `chain_get_status`
3. Manually refresh balance
4. Switch to alternative RPC if issues persist

### Chain Switch Failed

1. Ensure chain is enabled
2. Verify chain configuration is valid
3. Check RPC connectivity
4. Review error logs

## Future Enhancements

- Additional chain support (Avalanche, BNB Chain, etc.)
- More bridge providers (Stargate, LayerZero)
- Advanced routing optimization
- Gas optimization suggestions
- Automated bridge transaction tracking with notifications
