# Token Launchpad Guide

## Overview

The Eclipse Market Pro Launchpad is a comprehensive platform for creating, launching, and managing SPL tokens on Solana. It provides secure, auditable tooling for token creation, liquidity locking, vesting schedules, airdrop distribution, and launch monitoring.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Features](#features)
3. [Security Considerations](#security-considerations)
4. [Getting Started](#getting-started)
5. [Token Creation](#token-creation)
6. [Liquidity Locking](#liquidity-locking)
7. [Vesting Schedules](#vesting-schedules)
8. [Airdrop Management](#airdrop-management)
9. [Distribution Monitoring](#distribution-monitoring)
10. [API Reference](#api-reference)
11. [Risks and Warnings](#risks-and-warnings)
12. [Best Practices](#best-practices)
13. [Troubleshooting](#troubleshooting)

## Prerequisites

### Technical Requirements

- **Solana Wallet**: Connected wallet with sufficient SOL for transaction fees
- **Solana RPC**: Access to a Solana RPC endpoint (mainnet-beta, devnet, or custom)
- **Minimum SOL Balance**: ~0.5 SOL for token creation and initial operations
- **System Requirements**: Eclipse Market Pro desktop application installed

### Knowledge Requirements

- Understanding of SPL tokens and Solana blockchain
- Familiarity with token economics and tokenomics
- Knowledge of cryptocurrency regulatory landscape
- Understanding of smart contract security basics

### Regulatory Compliance

**⚠️ IMPORTANT**: Token creation and distribution may be subject to securities laws in your jurisdiction. Consult with legal professionals before launching any token.

## Features

### Token Creation

- **SPL Token Minting**: Create standard SPL tokens with customizable parameters
- **Metadata Management**: Add name, symbol, description, logo, and social links
- **Authority Controls**: Configure mint and freeze authorities
- **Transaction Simulation**: Test token creation before execution
- **Safety Checks**: Automated compliance and security validation

### Liquidity Locking

- **Time-Locked Liquidity**: Lock liquidity pool tokens for specified durations
- **Beneficiary Management**: Designate beneficiary addresses for locked funds
- **Revocable Locks**: Option to create revocable or irrevocable locks
- **Multi-Pool Support**: Lock liquidity across multiple pools
- **Lock Monitoring**: Track lock status and unlock dates

### Vesting Schedules

- **Linear Vesting**: Gradual token release over time
- **Cliff Periods**: Initial lock period before vesting begins
- **Staged Vesting**: Custom milestone-based release schedules
- **Beneficiary Tracking**: Manage multiple vesting schedules per token
- **Release Management**: Controlled token release to beneficiaries

### Airdrop Management

- **Bulk Distribution**: Distribute tokens to multiple addresses
- **CSV Import**: Import recipient lists from CSV files
- **Merkle Tree Claims**: Gas-efficient claimable airdrops
- **Time-Bound Airdrops**: Set start and end dates for claim periods
- **Claim Tracking**: Monitor airdrop claim rates and status

### Distribution Monitoring

- **Real-Time Metrics**: Track distribution progress
- **Transfer Analytics**: Monitor successful and failed transfers
- **Vesting Status**: View active and completed vesting schedules
- **Liquidity Status**: Monitor locked liquidity amounts
- **Historical Data**: Access past distribution events

## Security Considerations

### Key Management

The Launchpad uses the Eclipse Market Pro **Keystore** system for secure key management:

- **Encrypted Storage**: All private keys are encrypted using AES-256-GCM
- **Keyring Integration**: Master keys stored in system keyring (Windows Credential Manager, macOS Keychain)
- **Ephemeral Keys**: Temporary keys for launch operations with automatic cleanup
- **Key Derivation**: Argon2 password hashing for key derivation

### Transaction Simulation

Before executing any transaction, the Launchpad simulates it to:

- **Estimate Costs**: Calculate compute units and transaction fees
- **Validate Logic**: Ensure transaction will succeed
- **Identify Warnings**: Detect potential issues or risks
- **Test Authorities**: Verify authority permissions

### Compliance Checks

The Launchpad integrates with the **Smart Audit Module** to perform safety checks:

- **Token Supply**: Validate total supply against safe limits
- **Authority Configuration**: Check for potentially dangerous authorities
- **Metadata Completeness**: Ensure adequate token information
- **Social Presence**: Verify community channels
- **Risk Scoring**: Calculate overall security score (0-100)

### Risk Levels

- **Low (80-100)**: Minimal risks detected, best practices followed
- **Medium (60-79)**: Some concerns present, review recommended
- **High (40-59)**: Significant risks, caution advised
- **Critical (0-39)**: Serious issues detected, launch not recommended

## Getting Started

### 1. Access the Launchpad

Navigate to **Launchpad** from the main menu or sidebar in Eclipse Market Pro.

### 2. Connect Wallet

Ensure your Solana wallet is connected with sufficient SOL balance.

### 3. Choose Launch Type

Select the appropriate tab based on your current stage:

- **Token Setup**: Create new token
- **Liquidity Lock**: Lock liquidity for existing token
- **Vesting**: Set up vesting schedules
- **Airdrop**: Configure token distribution
- **Monitor**: Track launch metrics

## Token Creation

### Step-by-Step Guide

#### 1. Configure Token Parameters

```
Name: YourToken
Symbol: YTKN
Decimals: 9 (standard for Solana)
Total Supply: 1,000,000,000
```

#### 2. Set Metadata

- **Description**: Clear explanation of token purpose
- **Image URL**: Link to token logo (recommended: 512x512 PNG)
- **Website**: Project website
- **Twitter**: Official Twitter handle
- **Telegram**: Community Telegram group
- **Discord**: Discord server invite

#### 3. Configure Authorities

- **Mint Authority**: Ability to create new tokens (⚠️ centralization risk)
- **Freeze Authority**: Ability to freeze token accounts (⚠️ censorship risk)

**Best Practice**: Disable both authorities after initial distribution for maximum decentralization.

#### 4. Save Draft

Click **Save Draft** to create a launch configuration. This stores your settings locally without executing any transactions.

#### 5. Simulate & Check Safety

Click **Simulate & Check Safety** to:

- Test transaction execution
- Get fee estimates
- Receive safety compliance report
- Identify potential issues

Review the **Safety & Compliance** results:

- Security Score (0-100)
- Risk Level classification
- Individual check results
- Recommendations for improvement

#### 6. Launch Token

Once satisfied with simulation results, click **Launch Token** to execute the creation transaction.

**Cost**: ~0.001-0.01 SOL for token creation + rent-exempt balance for accounts

### Example Safety Check Results

```json
{
  "securityScore": 85,
  "riskLevel": "low",
  "checks": [
    {
      "checkName": "Token Supply",
      "passed": true,
      "severity": "info",
      "message": "Token supply is within safe limits"
    },
    {
      "checkName": "Token Authorities",
      "passed": false,
      "severity": "high",
      "message": "Mint or freeze authority is enabled",
      "recommendation": "Consider disabling authorities after launch"
    }
  ]
}
```

## Liquidity Locking

### Purpose

Liquidity locking demonstrates commitment to your project by time-locking liquidity pool tokens, preventing rug pulls and building community trust.

### Configuration

#### Required Parameters

- **Token Mint**: Address of the token mint
- **Pool Address**: Address of the liquidity pool
- **Amount**: Number of LP tokens to lock
- **Duration**: Lock period in days (minimum: 1 day, recommended: 180+ days)
- **Beneficiary**: Address to receive tokens after unlock
- **Revocable**: Whether the lock can be revoked early

#### Best Practices

- **Minimum Duration**: Lock for at least 180 days (6 months)
- **Graduated Unlocks**: Consider multiple smaller locks with staggered unlock dates
- **Irrevocable Locks**: Use non-revocable locks for maximum trust
- **Public Announcement**: Announce lock details to community

### Compliance Check

The system validates liquidity lock requests against best practices:

```
Minimum Duration: 180 days
Recommended: Non-revocable
Security Impact: Reduces rug pull risk
```

## Vesting Schedules

### Vesting Types

#### Linear Vesting

Tokens release gradually over the vesting period.

**Example**: 1,000,000 tokens over 365 days = ~2,740 tokens/day

#### Cliff Vesting

No tokens release until cliff period expires, then full amount unlocks.

**Example**: 30-day cliff, then 100% unlock

#### Staged Vesting

Custom percentage releases at specific dates.

**Example**:
- Month 1: 25%
- Month 3: 25%
- Month 6: 25%
- Month 12: 25%

### Configuration

```
Token Mint: <token_address>
Beneficiary: <beneficiary_address>
Total Amount: 1,000,000
Start Date: 2024-01-01
Cliff Duration: 30 days
Vesting Duration: 365 days
Vesting Type: linear
```

### Use Cases

- **Team Allocation**: Lock team tokens with 1-4 year vesting
- **Advisor Tokens**: Cliff + vesting for consultants
- **Strategic Partners**: Milestone-based staged vesting
- **Community Rewards**: Time-locked incentives

### Release Management

Beneficiaries can release vested tokens through the platform:

1. Navigate to **Vesting** tab
2. View releasable amount
3. Click **Release Tokens**
4. Confirm transaction

## Airdrop Management

### Planning Your Airdrop

#### Distribution Methods

1. **Immediate Transfer**: Tokens sent directly to recipients
2. **Merkle Tree**: Recipients claim tokens (gas-efficient)
3. **Vested Airdrop**: Combine airdrop with vesting schedule

#### Recipient Selection

Consider distributing to:

- Early community members
- Active Discord/Telegram members
- Holders of specific NFTs
- Participants in testnet
- Liquidity providers
- Snapshot of existing token holders

### Creating an Airdrop

#### 1. Prepare Recipient List

Create a CSV file with format:

```csv
address,amount
TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA,1000
So11111111111111111111111111111111111111112,2000
EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v,3000
```

#### 2. Upload and Parse

Paste CSV content into the **Recipients CSV** field and click **Parse CSV**.

#### 3. Review Recipients

Verify recipient count and total distribution amount.

#### 4. Configure Timing

- **Start Date**: When airdrop becomes active
- **End Date**: Optional deadline for claims
- **Claim Type**: Immediate or Merkle Tree

#### 5. Create Airdrop

Click **Create Airdrop** to initialize the distribution.

### Monitoring Claims

For Merkle Tree airdrops, track:

- Total recipients
- Claimed count
- Unclaimed count
- Claimed amount
- Unclaimed amount

## Distribution Monitoring

### Metrics Dashboard

Access real-time distribution metrics for any token:

#### Key Metrics

- **Total Distributed**: Cumulative tokens distributed
- **Total Recipients**: Number of unique recipients
- **Successful Transfers**: Completed distributions
- **Failed Transfers**: Distribution errors
- **Vesting Active**: Ongoing vesting schedules
- **Vesting Completed**: Finished vesting schedules
- **Liquidity Locked**: Total LP tokens locked

#### Using the Monitor

1. Navigate to **Monitor** tab
2. Enter token mint address
3. Click **Refresh Metrics**
4. Review distribution status

## API Reference

### Tauri Commands

#### Token Creation

```rust
// Create launch configuration
create_launch_config(
  name: String,
  symbol: String,
  decimals: u8,
  total_supply: u64,
  description: String,
  metadata: TokenMetadata
) -> Result<TokenLaunchConfig, String>

// Simulate token creation
simulate_token_creation(
  request: CreateTokenRequest
) -> Result<TransactionSimulation, String>

// Execute token creation
launchpad_create_token(
  request: CreateTokenRequest
) -> Result<CreateTokenResponse, String>
```

#### Safety Checks

```rust
// Check launch safety
check_launch_safety(
  config: TokenLaunchConfig
) -> Result<LaunchSafetyCheck, String>

// Check vesting compliance
check_vesting_compliance(
  request: CreateVestingRequest
) -> Result<SafetyCheckResult, String>

// Check liquidity lock compliance
check_liquidity_lock_compliance(
  request: LockLiquidityRequest
) -> Result<SafetyCheckResult, String>
```

#### Liquidity Locking

```rust
// Create liquidity lock
create_liquidity_lock(
  request: LockLiquidityRequest
) -> Result<LiquidityLockConfig, String>

// Unlock liquidity
unlock_liquidity(
  lock_id: String
) -> Result<String, String>

// Get lock details
get_liquidity_lock(
  lock_id: String
) -> Result<LiquidityLockConfig, String>

// List all locks
list_liquidity_locks() -> Result<Vec<LiquidityLockConfig>, String>
```

#### Vesting

```rust
// Create vesting schedule
create_vesting_schedule(
  request: CreateVestingRequest
) -> Result<VestingSchedule, String>

// Release vested tokens
release_vested_tokens(
  schedule_id: String,
  amount: u64
) -> Result<VestingSchedule, String>

// Get schedule
get_vesting_schedule(
  schedule_id: String
) -> Result<VestingSchedule, String>

// List schedules
list_vesting_schedules(
  token_mint: Option<String>,
  beneficiary: Option<String>
) -> Result<Vec<VestingSchedule>, String>
```

#### Airdrop

```rust
// Create airdrop
create_airdrop(
  request: CreateAirdropRequest
) -> Result<AirdropConfig, String>

// Activate airdrop
activate_airdrop(
  airdrop_id: String
) -> Result<AirdropConfig, String>

// Claim tokens
claim_airdrop_tokens(
  airdrop_id: String,
  recipient_address: String
) -> Result<AirdropRecipient, String>

// Get airdrop metrics
get_airdrop_metrics(
  airdrop_id: String
) -> Result<AirdropMetrics, String>
```

#### Monitoring

```rust
// Get distribution metrics
get_distribution_metrics(
  token_mint: String
) -> Result<DistributionMetrics, String>
```

## Risks and Warnings

### ⚠️ Critical Warnings

1. **Irreversible Actions**: Token creation and distribution transactions cannot be undone
2. **Loss of Funds**: Incorrect addresses or parameters can result in permanent loss
3. **Regulatory Risk**: Token offerings may violate securities laws
4. **Smart Contract Risk**: Bugs in SPL token program could affect your token
5. **Key Management**: Lost private keys mean lost access to authorities

### Security Risks

- **Phishing**: Verify all addresses manually before transactions
- **Rug Pulls**: Even with liquidity locks, other attack vectors exist
- **Flash Loan Attacks**: Not applicable to SPL tokens but be aware of DeFi risks
- **Oracle Manipulation**: If using price oracles, ensure they're secure

### Operational Risks

- **RPC Failures**: Transactions may fail due to RPC issues
- **Network Congestion**: High fees during Solana congestion
- **Account Rent**: Accounts require rent-exempt balance to persist
- **Duplicate Launches**: Reusing token names/symbols can confuse users

### Legal Risks

- **Securities Law**: Tokens may be classified as securities
- **Tax Obligations**: Token creation and distribution have tax implications
- **AML/KYC**: Large distributions may trigger compliance requirements
- **Fraud Liability**: Misrepresentation can result in legal action

## Best Practices

### Pre-Launch Checklist

- [ ] Complete tokenomics design
- [ ] Audit smart contracts (if custom)
- [ ] Legal review and compliance check
- [ ] Community building and marketing plan
- [ ] Test on devnet first
- [ ] Prepare for liquidity provision
- [ ] Set up vesting for team and advisors
- [ ] Plan airdrop distribution
- [ ] Arrange for CEX/DEX listings
- [ ] Prepare incident response plan

### Launch Day Best Practices

1. **Triple-Check Addresses**: Verify all recipient and pool addresses
2. **Gradual Distribution**: Don't distribute everything at once
3. **Monitor Activity**: Watch for unusual trading patterns
4. **Communicate**: Keep community updated throughout launch
5. **Document Everything**: Record all transactions and decisions
6. **Have Support Ready**: Prepare team to handle questions

### Post-Launch Monitoring

- Monitor token holder distribution
- Track liquidity levels
- Watch for large transfers or sells
- Engage with community feedback
- Address issues promptly
- Maintain transparency

### Token Authority Management

```
Launch → Test period (authorities enabled)
  ↓
Stabilize → Team distribution (mint authority still enabled)
  ↓
Mature → Revoke authorities (maximum decentralization)
```

### Recommended Timelines

- **Team Vesting**: 2-4 years with 6-12 month cliff
- **Advisor Vesting**: 1-2 years with 3-6 month cliff
- **Liquidity Lock**: Minimum 180 days, ideally 1-2 years
- **Airdrop Claims**: 30-90 days claim period

## Troubleshooting

### Common Issues

#### Transaction Failed

**Causes**:
- Insufficient SOL for fees
- Invalid addresses
- RPC node issues
- Network congestion

**Solutions**:
- Check SOL balance
- Verify all addresses
- Switch RPC endpoint
- Retry during low congestion

#### Simulation Failed

**Causes**:
- Invalid token parameters
- Missing required fields
- Authority permission issues

**Solutions**:
- Review all input fields
- Check decimals (max 9)
- Ensure wallet is connected

#### Liquidity Lock Failed

**Causes**:
- Invalid pool address
- Insufficient LP token balance
- Duration too short

**Solutions**:
- Verify pool address on Solana Explorer
- Check LP token balance
- Increase duration to at least 1 day

#### Vesting Release Failed

**Causes**:
- Tokens not yet vested
- Incorrect schedule ID
- Revoked schedule

**Solutions**:
- Check releasable amount
- Verify schedule ID
- Ensure schedule is active

#### Airdrop Claim Failed

**Causes**:
- Airdrop not active
- Already claimed
- Recipient not eligible
- Airdrop expired

**Solutions**:
- Check airdrop status
- Verify recipient address
- Ensure within claim period

### Getting Help

**Discord**: Join Eclipse Market Pro Discord for community support
**Documentation**: Refer to inline help tooltips
**Support**: Contact support team for critical issues
**GitHub**: Report bugs via GitHub issues

## Conclusion

The Eclipse Market Pro Launchpad provides professional-grade tooling for token launches with built-in security, compliance, and monitoring features. Always prioritize security, follow best practices, and consult legal professionals before launching.

**Remember**: Token creation is powerful but comes with significant responsibility. Use this platform wisely and ethically.

---

*Last Updated: 2024*
*Version: 1.0.0*
*For educational and informational purposes only*
