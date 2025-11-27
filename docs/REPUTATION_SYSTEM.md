# Reputation System Documentation

## Overview

The Reputation Network is a comprehensive trust and safety system that tracks wallet and token behavior, enables community vouching, maintains shared blacklists, and provides transparency through reputation scores and history tracking.

## Architecture

### Backend Components

#### Core Engine (`src-tauri/src/security/reputation.rs`)

The reputation engine provides:
- **Wallet Reputation Tracking**: Monitors wallet behavior including transaction counts, volume, age, and community vouches
- **Token Reputation**: Evaluates tokens based on creator reputation, holder distribution, liquidity, and community feedback
- **Vouching System**: Allows trusted community members to vouch for wallets and tokens
- **Blacklist Management**: Maintains shared blacklists with automated and community-driven entries
- **History Tracking**: Records all reputation events for audit trails and trend analysis
- **Reporting System**: Enables users to report suspicious activity with evidence

#### Database Schema

The system uses SQLite for persistent storage with the following tables:

1. **wallet_reputation**: Stores wallet reputation data
   - trust_score, vouches, transaction_count, total_volume, age_days, risk_flags
   
2. **token_reputation**: Stores token reputation data
   - trust_score, creator info, holder_count, liquidity_score, risk_flags
   
3. **vouches**: Tracks vouching relationships
   - voucher_address, target_address, target_type, comment, timestamp
   
4. **blacklist**: Maintains blacklisted addresses
   - address, entry_type, reason, reporter, source, timestamp
   
5. **reputation_history**: Audit trail of reputation changes
   - address, timestamp, trust_score, event_type, details
   
6. **reputation_reports**: User-submitted reports
   - reporter, target, report_type, description, evidence

### Frontend Components

#### UI Components (`src/components/reputation/`)

1. **ReputationBadge**: Visual indicator of reputation level
   - Displays trust score with color-coded levels (Excellent, Good, Neutral, Poor, Malicious)
   - Shows warnings for low-reputation or blacklisted addresses

2. **WalletReputationView**: Detailed wallet reputation display
   - Trust score and level
   - Transaction history metrics
   - Vouching statistics
   - Risk flags
   - Timeline information

3. **ReputationHistoryChart**: Historical trust score visualization
   - Line chart showing score changes over time
   - Event annotations
   - Interactive tooltips with event details

4. **VouchingWorkflow**: Community vouching interface
   - Add/remove vouches
   - View existing vouches
   - Comment system
   - Reputation requirements

5. **ReportModal**: Reporting interface
   - Report types (scam, rugpull, suspicious, other)
   - Detailed description
   - Evidence submission
   - Validation and warnings

6. **ModerationControls**: Admin/settings interface
   - System configuration
   - Statistics dashboard
   - Blacklist management
   - Privacy controls

## Trust Score Calculation

### Wallet Trust Score

Base score: 50.0

Factors:
- **Age** (max +20): Wallet age in days, scaled to 365 days
- **Transaction Count** (max +15): Number of transactions, scaled to 1000
- **Volume** (max +10): Total transaction volume, scaled to $1M
- **Vouches** (max +25): 5 points per vouch, capped at 5 vouches

Formula:
```
score = 50 
  + (age_days / 365) * 20
  + (tx_count / 1000) * 15
  + (volume / 1_000_000) * 10
  + min(vouches * 5, 25)
```

Range: 0-100

### Token Trust Score

Base score: 50.0

Factors:
- **Creator Trust** (max +25): 25% of creator's trust score
- **Holder Count** (max +20): Number of holders, scaled to 10,000
- **Liquidity** (max +15): Liquidity score (0-1 scale)
- **Vouches** (max +20): 5 points per vouch, capped at 4 vouches

Formula:
```
score = 50
  + (creator_trust / 100) * 25
  + (holder_count / 10000) * 20
  + liquidity_score * 15
  + min(vouches * 5, 20)
```

Range: 0-100

### Reputation Levels

- **Excellent**: 80-100
- **Good**: 60-79
- **Neutral**: 40-59
- **Poor**: 20-39
- **Malicious**: 0-19

## Features

### 1. Wallet Behavior Tracking

The system automatically tracks and scores wallet activity:
- First seen date
- Transaction count
- Total volume traded
- Account age
- Behavioral patterns

### 2. Community Vouching

Users with sufficient reputation (default: 50+) can vouch for addresses:
- Add personal vouches with optional comments
- Remove vouches at any time
- View all vouches received by an address
- Vouches contribute to trust score

### 3. Blacklist Management

Multiple sources contribute to the blacklist:
- **Automated**: Low trust scores (< threshold)
- **Community**: Multiple user reports (10+ reports)
- **Admin**: Manual additions by moderators

### 4. Reputation History

All reputation events are logged:
- Initial creation
- Score updates
- Vouches received
- Blacklist additions/removals
- Behavioral changes

### 5. Reporting System

Users can report suspicious activity:
- Report types: Scam, Rug Pull, Suspicious Activity, Other
- Required detailed description
- Optional evidence (links, transaction hashes)
- Automatic blacklisting after threshold

### 6. Privacy Controls

Users control their data sharing:
- **Enable/Disable**: Turn system on/off
- **Share Data**: Opt-in/out of community data sharing
- **Show Warnings**: Display reputation warnings in UI
- **Thresholds**: Customize auto-blacklist and vouch requirements

## API Commands

### Query Commands

```typescript
// Get wallet reputation
invoke<WalletReputation>('get_wallet_reputation', { address: string })

// Get token reputation
invoke<TokenReputation>('get_token_reputation', { address: string })

// Get reputation history
invoke<ReputationHistory[]>('get_reputation_history', { 
  address: string, 
  limit?: number 
})

// Get vouches for an address
invoke<VouchRecord[]>('get_vouches', { targetAddress: string })

// Get blacklist
invoke<BlacklistEntry[]>('get_blacklist', { entryType?: string })

// Get statistics
invoke<ReputationStats>('get_reputation_stats')

// Get settings
invoke<ReputationSettings>('get_reputation_settings')
```

### Action Commands

```typescript
// Update wallet behavior
invoke('update_wallet_behavior', {
  address: string,
  transactionCount?: number,
  totalVolume?: number,
  ageDays?: number
})

// Initialize token reputation
invoke('initialize_token_reputation', {
  address: string,
  creatorAddress: string
})

// Update token metrics
invoke('update_token_metrics', {
  address: string,
  holderCount?: number,
  liquidityScore?: number
})

// Add vouch
invoke('add_vouch', {
  voucherAddress: string,
  targetAddress: string,
  targetType: 'wallet' | 'token',
  comment?: string
})

// Remove vouch
invoke('remove_vouch', {
  voucherAddress: string,
  targetAddress: string,
  targetType: string
})

// Submit report
invoke('submit_reputation_report', {
  report: ReputationReport
})

// Add to blacklist
invoke('add_to_blacklist', {
  address: string,
  entryType: string,
  reason: string,
  reporter?: string,
  source: string
})

// Remove from blacklist
invoke('remove_from_blacklist', {
  address: string,
  entryType: string
})

// Update settings
invoke('update_reputation_settings', {
  settings: ReputationSettings
})
```

## Integration Examples

### Display Reputation Badge on Token Cards

```tsx
import { ReputationBadge } from '@/components/reputation';

function TokenCard({ token }) {
  const [reputation, setReputation] = useState(null);
  
  useEffect(() => {
    invoke('get_token_reputation', { address: token.address })
      .then(setReputation);
  }, [token.address]);
  
  return (
    <div className="token-card">
      {reputation && (
        <ReputationBadge
          level={reputation.reputationLevel}
          score={reputation.trustScore}
        />
      )}
    </div>
  );
}
```

### Show Wallet Reputation in Profile

```tsx
import { WalletReputationView } from '@/components/reputation';

function WalletProfile({ address }) {
  return (
    <div>
      <WalletReputationView address={address} />
    </div>
  );
}
```

### Enable Vouching on Token Details

```tsx
import { VouchingWorkflow } from '@/components/reputation';

function TokenDetails({ tokenAddress, userAddress }) {
  return (
    <div>
      <VouchingWorkflow
        targetAddress={tokenAddress}
        targetType="token"
        currentUserAddress={userAddress}
      />
    </div>
  );
}
```

## Privacy Considerations

### Data Collection

The reputation system collects:
- Wallet addresses (public on blockchain)
- Transaction counts and volumes (public on blockchain)
- Vouch relationships (opt-in)
- User reports (anonymous option available)

### Data Sharing

Users control their participation through settings:
- **Enabled**: Participate in reputation system
- **Share Data**: Contribute to community reputation network
- All data sharing is opt-in and can be disabled

### Data Storage

- All data stored locally in encrypted SQLite database
- No external servers or third-party data sharing by default
- Community sharing (if enabled) uses anonymized aggregates

### User Rights

Users can:
- View their own reputation data
- Remove their vouches at any time
- Opt-out of the system entirely
- Request data deletion (via settings reset)

## Security Considerations

### Sybil Resistance

- Minimum reputation required to vouch (default: 50)
- Transaction history and age verification
- Volume-based scoring
- Community cross-validation

### False Reporting

- Reporter's reputation affects report weight
- Multiple reports required for automated action
- Admin review for disputed cases
- False reporting affects reporter's score

### Manipulation Prevention

- Time-weighted scoring
- Multiple factor verification
- Automated anomaly detection
- Community oversight

## Testing

### Backend Tests

Located in `src-tauri/src/security/reputation.rs`:

```bash
cargo test --package app -- reputation::tests
```

Test coverage:
- Trust score calculations
- Reputation level mapping
- Vouch validation
- Blacklist operations
- History tracking

### Frontend Tests

Located in `src/__tests__/`:

```bash
npm test -- reputation
```

Test coverage:
- Component rendering
- API integration
- User interactions
- State management
- Error handling

## Configuration

### Default Settings

```typescript
{
  enabled: true,
  autoBlacklistThreshold: 10.0,  // Auto-blacklist below this score
  minVouchWeight: 50.0,           // Min score to vouch
  showWarnings: true,             // Show reputation warnings
  shareData: false                // Opt-in community sharing
}
```

### Customization

Settings can be adjusted through:
1. UI: Settings page → Reputation System
2. API: `update_reputation_settings` command
3. Config file: `app_data/reputation_settings.json`

## Roadmap

### Future Enhancements

1. **Machine Learning**: AI-powered risk detection
2. **Cross-chain**: Support for multiple blockchains
3. **Social Proof**: Integration with social media verification
4. **Decentralized**: Move to on-chain governance
5. **Advanced Analytics**: Predictive risk scoring
6. **Community Governance**: DAO-based moderation

## Support

For issues or questions:
- GitHub Issues: [repository]/issues
- Documentation: This file
- API Reference: See inline code documentation

## License

See main project LICENSE file.
