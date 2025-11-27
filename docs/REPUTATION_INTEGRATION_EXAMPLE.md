# Reputation System Integration Examples

## Quick Start

### 1. Add Reputation Badge to Token Cards

```tsx
// src/components/coins/TokenCard.tsx
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api';
import { ReputationBadge } from '../reputation';

export function TokenCard({ token }) {
  const [reputation, setReputation] = useState(null);
  const [loading, setLoading] = useState(true);
  
  useEffect(() => {
    // Initialize token reputation if not exists
    invoke('initialize_token_reputation', {
      address: token.address,
      creatorAddress: token.creator
    }).catch(console.error);
    
    // Load reputation
    invoke('get_token_reputation', { address: token.address })
      .then(rep => {
        setReputation(rep);
        setLoading(false);
      })
      .catch(err => {
        console.error('Failed to load reputation:', err);
        setLoading(false);
      });
  }, [token.address]);
  
  return (
    <div className="token-card">
      <div className="token-header">
        <img src={token.logo} alt={token.name} />
        <div>
          <h3>{token.name}</h3>
          <p>{token.symbol}</p>
        </div>
        {!loading && reputation && (
          <ReputationBadge
            level={reputation.reputationLevel.toLowerCase()}
            score={reputation.trustScore}
            size="sm"
          />
        )}
      </div>
      {/* Rest of token card */}
    </div>
  );
}
```

### 2. Add Wallet Reputation View to Profile

```tsx
// src/pages/WalletProfile.tsx
import { WalletReputationView } from '../components/reputation';

export function WalletProfile({ address }) {
  return (
    <div className="wallet-profile">
      <h1>Wallet Profile</h1>
      
      {/* Wallet Reputation Section */}
      <section className="mb-6">
        <WalletReputationView address={address} />
      </section>
      
      {/* Other wallet information */}
    </div>
  );
}
```

### 3. Add Vouching to Token Detail Page

```tsx
// src/pages/TokenDetail.tsx
import { useState } from 'react';
import { VouchingWorkflow, ReportModal } from '../components/reputation';
import { useWallet } from '../contexts/WalletContext';

export default function TokenDetail({ tokenAddress, onBack }) {
  const { publicKey } = useWallet();
  const [showReportModal, setShowReportModal] = useState(false);
  
  return (
    <div className="token-detail">
      {/* Existing content */}
      
      {/* Add reputation section */}
      <section className="reputation-section">
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {/* Vouching */}
          <VouchingWorkflow
            targetAddress={tokenAddress}
            targetType="token"
            currentUserAddress={publicKey?.toString()}
            onVouchAdded={() => {
              // Refresh token data
            }}
          />
          
          {/* Report button */}
          <div className="flex items-center justify-center">
            <button
              onClick={() => setShowReportModal(true)}
              className="px-4 py-2 bg-red-600 hover:bg-red-700 rounded-lg"
            >
              Report This Token
            </button>
          </div>
        </div>
      </section>
      
      {/* Report Modal */}
      <ReportModal
        targetAddress={tokenAddress}
        targetType="token"
        reporterAddress={publicKey?.toString()}
        isOpen={showReportModal}
        onClose={() => setShowReportModal(false)}
        onSuccess={() => {
          setShowReportModal(false);
          // Show success message
        }}
      />
    </div>
  );
}
```

### 4. Add Reputation History Chart

```tsx
// src/pages/WalletProfile.tsx
import { ReputationHistoryChart } from '../components/reputation';

export function WalletProfile({ address }) {
  return (
    <div className="wallet-profile">
      {/* Other content */}
      
      {/* Reputation History */}
      <section className="mt-6">
        <ReputationHistoryChart 
          address={address} 
          limit={100}
        />
      </section>
    </div>
  );
}
```

### 5. Add Moderation Controls to Settings

```tsx
// src/pages/Settings.tsx
import { useState } from 'react';
import { ModerationControls } from '../components/reputation';

export function Settings() {
  const [activeSection, setActiveSection] = useState('general');
  
  return (
    <div className="settings">
      <nav className="settings-nav">
        <button onClick={() => setActiveSection('general')}>General</button>
        <button onClick={() => setActiveSection('security')}>Security</button>
        <button onClick={() => setActiveSection('reputation')}>Reputation</button>
      </nav>
      
      <div className="settings-content">
        {activeSection === 'general' && <GeneralSettings />}
        {activeSection === 'security' && <SecuritySettings />}
        {activeSection === 'reputation' && (
          <ModerationControls className="max-w-4xl" />
        )}
      </div>
    </div>
  );
}
```

## Automatic Reputation Updates

### Update Wallet Behavior After Transactions

```tsx
// src/hooks/useWallet.ts
import { useEffect } from 'react';
import { invoke } from '@tauri-apps/api';

export function useWallet() {
  const { publicKey, transactions } = useWalletContext();
  
  useEffect(() => {
    if (!publicKey) return;
    
    // Update wallet reputation after transaction activity
    const updateReputation = async () => {
      try {
        const txCount = transactions.length;
        const volume = transactions.reduce((sum, tx) => sum + tx.amount, 0);
        const ageInDays = calculateWalletAge(publicKey);
        
        await invoke('update_wallet_behavior', {
          address: publicKey.toString(),
          transactionCount: txCount,
          totalVolume: volume,
          ageDays: ageInDays
        });
      } catch (err) {
        console.error('Failed to update wallet reputation:', err);
      }
    };
    
    updateReputation();
  }, [publicKey, transactions.length]);
  
  return { publicKey, transactions };
}
```

### Update Token Metrics Periodically

```tsx
// src/services/tokenMetricsUpdater.ts
import { invoke } from '@tauri-apps/api';

export async function updateTokenMetrics(tokenAddress: string) {
  try {
    // Fetch current metrics from blockchain
    const holders = await fetchHolderCount(tokenAddress);
    const liquidity = await calculateLiquidityScore(tokenAddress);
    
    // Update reputation
    await invoke('update_token_metrics', {
      address: tokenAddress,
      holderCount: holders,
      liquidityScore: liquidity
    });
  } catch (err) {
    console.error('Failed to update token metrics:', err);
  }
}

// Call periodically
setInterval(() => {
  const watchedTokens = getWatchedTokens();
  watchedTokens.forEach(token => {
    updateTokenMetrics(token.address);
  });
}, 60 * 60 * 1000); // Every hour
```

## Display Reputation Warnings

### Show Warning Before Transaction

```tsx
// src/components/trading/SwapConfirmation.tsx
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api';
import { ReputationWarning } from '../reputation';

export function SwapConfirmation({ fromToken, toToken, onConfirm, onCancel }) {
  const [toTokenRep, setToTokenRep] = useState(null);
  const [loading, setLoading] = useState(true);
  
  useEffect(() => {
    invoke('get_token_reputation', { address: toToken.address })
      .then(rep => {
        setToTokenRep(rep);
        setLoading(false);
      })
      .catch(err => {
        console.error('Failed to load reputation:', err);
        setLoading(false);
      });
  }, [toToken.address]);
  
  return (
    <div className="swap-confirmation">
      <h2>Confirm Swap</h2>
      
      {/* Reputation Warning */}
      {!loading && toTokenRep && (
        <ReputationWarning
          level={toTokenRep.reputationLevel.toLowerCase()}
          isBlacklisted={toTokenRep.isBlacklisted}
          blacklistReason={toTokenRep.blacklistReason}
          className="mb-4"
        />
      )}
      
      {/* Swap details */}
      <div className="swap-details">
        <p>From: {fromToken.symbol}</p>
        <p>To: {toToken.symbol}</p>
      </div>
      
      {/* Actions */}
      <div className="actions">
        <button onClick={onCancel}>Cancel</button>
        <button 
          onClick={onConfirm}
          disabled={toTokenRep?.isBlacklisted}
        >
          {toTokenRep?.isBlacklisted ? 'Token Blacklisted' : 'Confirm'}
        </button>
      </div>
    </div>
  );
}
```

## Backend Integration

### Initialize Reputation on Token Discovery

```rust
// In your token discovery/scanning code
use crate::security::reputation::{ReputationEngine, SharedReputationEngine};

async fn on_new_token_discovered(
    token_address: &str,
    creator_address: &str,
    reputation_engine: SharedReputationEngine,
) -> Result<(), Box<dyn std::error::Error>> {
    let engine = reputation_engine.read().await;
    
    // Initialize reputation for the new token
    engine.initialize_token_reputation(token_address, creator_address).await?;
    
    // Get initial metrics
    let holder_count = get_holder_count(token_address).await?;
    let liquidity = calculate_liquidity_score(token_address).await?;
    
    // Update metrics
    engine.update_token_metrics(
        token_address,
        Some(holder_count),
        Some(liquidity)
    ).await?;
    
    Ok(())
}
```

### Check Reputation Before API Calls

```rust
// Add reputation checks to your trading API
#[tauri::command]
pub async fn execute_swap(
    token_in: String,
    token_out: String,
    amount: f64,
    reputation_engine: tauri::State<'_, SharedReputationEngine>,
) -> Result<String, String> {
    let engine = reputation_engine.read().await;
    
    // Check token out reputation
    let token_rep = engine.get_token_reputation(&token_out)
        .await
        .map_err(|e| e.to_string())?;
    
    // Block if blacklisted
    if token_rep.is_blacklisted {
        return Err(format!(
            "Cannot swap: {} is blacklisted. Reason: {}",
            token_out,
            token_rep.blacklist_reason.unwrap_or_else(|| "Unknown".to_string())
        ));
    }
    
    // Warn if low reputation
    if token_rep.trust_score < 40.0 {
        // Log warning or return warning in response
        eprintln!("Warning: Swapping to low reputation token (score: {})", token_rep.trust_score);
    }
    
    // Proceed with swap
    execute_swap_internal(token_in, token_out, amount).await
}
```

## Advanced Features

### Custom Reputation Filters

```tsx
// src/components/coins/CoinsList.tsx
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api';

export function CoinsList() {
  const [coins, setCoins] = useState([]);
  const [minReputation, setMinReputation] = useState(50);
  const [hideBlacklisted, setHideBlacklisted] = useState(true);
  
  useEffect(() => {
    loadCoinsWithReputation();
  }, [minReputation, hideBlacklisted]);
  
  const loadCoinsWithReputation = async () => {
    const allCoins = await fetchCoins();
    
    // Load reputation for each coin
    const coinsWithRep = await Promise.all(
      allCoins.map(async (coin) => {
        try {
          const rep = await invoke('get_token_reputation', { 
            address: coin.address 
          });
          return { ...coin, reputation: rep };
        } catch {
          return { ...coin, reputation: null };
        }
      })
    );
    
    // Filter by reputation
    const filtered = coinsWithRep.filter(coin => {
      if (!coin.reputation) return true;
      if (hideBlacklisted && coin.reputation.isBlacklisted) return false;
      return coin.reputation.trustScore >= minReputation;
    });
    
    setCoins(filtered);
  };
  
  return (
    <div>
      {/* Reputation filters */}
      <div className="filters">
        <label>
          Min Reputation: {minReputation}
          <input
            type="range"
            min="0"
            max="100"
            value={minReputation}
            onChange={(e) => setMinReputation(Number(e.target.value))}
          />
        </label>
        
        <label>
          <input
            type="checkbox"
            checked={hideBlacklisted}
            onChange={(e) => setHideBlacklisted(e.target.checked)}
          />
          Hide Blacklisted
        </label>
      </div>
      
      {/* Coins list */}
      <div className="coins-grid">
        {coins.map(coin => (
          <TokenCard key={coin.address} token={coin} />
        ))}
      </div>
    </div>
  );
}
```

### Reputation-Based Notifications

```tsx
// src/services/reputationNotifications.ts
import { invoke } from '@tauri-apps/api';

export async function checkWatchlistReputation() {
  const watchlist = await getWatchlist();
  const alerts = [];
  
  for (const token of watchlist) {
    const rep = await invoke('get_token_reputation', { 
      address: token.address 
    });
    
    // Alert on reputation drop
    if (rep.trustScore < 40 && !rep.alerted) {
      alerts.push({
        type: 'warning',
        token: token.name,
        message: `${token.name} reputation dropped to ${rep.trustScore.toFixed(0)}`
      });
    }
    
    // Alert on blacklist
    if (rep.isBlacklisted) {
      alerts.push({
        type: 'danger',
        token: token.name,
        message: `${token.name} has been blacklisted: ${rep.blacklistReason}`
      });
    }
  }
  
  return alerts;
}
```

## Testing Integration

### Mock Reputation Data

```tsx
// src/__tests__/helpers/mockReputation.ts
export const mockWalletReputation = {
  address: 'TestWallet123...',
  trustScore: 75.5,
  reputationLevel: 'good',
  vouchesReceived: 3,
  vouchesGiven: 5,
  isBlacklisted: false,
  blacklistReason: null,
  firstSeen: new Date().toISOString(),
  lastUpdated: new Date().toISOString(),
  transactionCount: 250,
  totalVolume: 50000,
  ageDays: 120,
  riskFlags: [],
};

export const mockTokenReputation = {
  address: 'TestToken456...',
  trustScore: 85.0,
  reputationLevel: 'excellent',
  creatorAddress: 'TestWallet123...',
  creatorTrustScore: 75.5,
  vouchesReceived: 10,
  isBlacklisted: false,
  blacklistReason: null,
  firstSeen: new Date().toISOString(),
  lastUpdated: new Date().toISOString(),
  holderCount: 5000,
  liquidityScore: 0.8,
  riskFlags: [],
};
```

This integration guide should help you implement the reputation system throughout your application!
